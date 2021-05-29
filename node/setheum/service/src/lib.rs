// This file is part of Setheum.

// Copyright (C) 2020-2021 Setheum Labs.
// SPDX-License-Identifier: GPL-3.0-or-later WITH Classpath-exception-2.0

// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with this program. If not, see <https://www.gnu.org/licenses/>.

// Disable the following lints
#![allow(clippy::type_complexity)]

//! Setheum service. Specialized wrapper over substrate service.

use std::sync::Arc;

use setheum_primitives::Block;
use prometheus_endpoint::Registry;
use sc_client_api::{ExecutorProvider, RemoteBackend};
use sc_executor::native_executor_instance;
use sc_finality_grandpa::FinalityProofProvider as GrandpaFinalityProofProvider;
use sc_service::{config::Configuration, error::Error as ServiceError, PartialComponents, RpcHandlers, TaskManager};
use sp_inherents::InherentDataProviders;
use sp_runtime::traits::{BlakeTwo256, Block as BlockT};

pub use client::*;

#[cfg(feature = "with-setheum-runtime")]
pub use setheum_runtime;
#[cfg(feature = "with-neom-runtime")]
pub use neom_runtime;
#[cfg(feature = "with-newrome-runtime")]
pub use newrome_runtime;

pub use sc_executor::NativeExecutionDispatch;
pub use sc_service::{
	config::{DatabaseConfig, PrometheusConfig},
	ChainSpec,
};
pub use sp_api::ConstructRuntimeApi;

pub mod chain_spec;
mod client;
mod mock_timestamp_data_provider;

#[cfg(feature = "with-newrome-runtime")]
native_executor_instance!(
	pub NewromeExecutor,
	newrome_runtime::api::dispatch,
	newrome_runtime::native_version,
	frame_benchmarking::benchmarking::HostFunctions,
);

#[cfg(feature = "with-neom-runtime")]
native_executor_instance!(
	pub NeomExecutor,
	neom_runtime::api::dispatch,
	neom_runtime::native_version,
	frame_benchmarking::benchmarking::HostFunctions,
);

#[cfg(feature = "with-setheum-runtime")]
native_executor_instance!(
	pub SetheumExecutor,
	setheum_runtime::api::dispatch,
	setheum_runtime::native_version,
	frame_benchmarking::benchmarking::HostFunctions,
);

/// Can be called for a `Configuration` to check if it is a configuration for
/// the `Setheum` network.
pub trait IdentifyVariant {
	/// Returns if this is a configuration for the `Setheum` network.
	fn is_setheum(&self) -> bool;

	/// Returns if this is a configuration for the `Neom` network.
	fn is_neom(&self) -> bool;

	/// Returns if this is a configuration for the `Newrome` network.
	fn is_newrome(&self) -> bool;
}

impl IdentifyVariant for Box<dyn ChainSpec> {
	fn is_setheum(&self) -> bool {
		self.id().starts_with("setheum") || self.id().starts_with("set")
	}

	fn is_neom(&self) -> bool {
		self.id().starts_with("neom") || self.id().starts_with("neo")
	}

	fn is_newrome(&self) -> bool {
		self.id().starts_with("newrome") || self.id().starts_with("rom")
	}
}

/// Setheum's full backend.
type FullBackend = sc_service::TFullBackend<Block>;

/// Setheum's select chain.
type FullSelectChain = sc_consensus::LongestChain<FullBackend, Block>;

/// Setheum's full client.
type FullClient<RuntimeApi, Executor> = sc_service::TFullClient<Block, RuntimeApi, Executor>;

/// Setheum's full Grandpa block import.
type FullGrandpaBlockImport<RuntimeApi, Executor> =
	sc_finality_grandpa::GrandpaBlockImport<FullBackend, Block, FullClient<RuntimeApi, Executor>, FullSelectChain>;

/// Setheum's light backend.
type LightBackend = sc_service::TLightBackendWithHash<Block, BlakeTwo256>;

/// Setheum's light client.
type LightClient<RuntimeApi, Executor> = sc_service::TLightClientWithBackend<Block, RuntimeApi, Executor, LightBackend>;

pub fn new_partial<RuntimeApi, Executor>(
	config: &mut Configuration,
	instant_sealing: bool,
	test: bool,
) -> Result<
	PartialComponents<
		FullClient<RuntimeApi, Executor>,
		FullBackend,
		FullSelectChain,
		sp_consensus::DefaultImportQueue<Block, FullClient<RuntimeApi, Executor>>,
		sc_transaction_pool::FullPool<Block, FullClient<RuntimeApi, Executor>>,
		(
			impl Fn(setheum_rpc::DenyUnsafe, setheum_rpc::SubscriptionTaskExecutor) -> setheum_rpc::RpcExtension,
			(
				sc_consensus_babe::BabeBlockImport<
					Block,
					FullClient<RuntimeApi, Executor>,
					FullGrandpaBlockImport<RuntimeApi, Executor>,
				>,
				sc_finality_grandpa::LinkHalf<Block, FullClient<RuntimeApi, Executor>, FullSelectChain>,
				sc_consensus_babe::BabeLink<Block>,
			),
			sc_finality_grandpa::SharedVoterState,
		),
	>,
	sc_service::Error,
>
where
	RuntimeApi: ConstructRuntimeApi<Block, FullClient<RuntimeApi, Executor>> + Send + Sync + 'static,
	RuntimeApi::RuntimeApi: RuntimeApiCollection<StateBackend = sc_client_api::StateBackendFor<FullBackend, Block>>,
	Executor: NativeExecutionDispatch + 'static,
{
	if !test {
		// If we're using prometheus, use a registry with a prefix of `setheum`.
		if let Some(PrometheusConfig { registry, .. }) = config.prometheus_config.as_mut() {
			*registry = Registry::new_custom(Some("setheum".into()), None)?;
		}
	}

	let (client, backend, keystore_container, task_manager) =
		sc_service::new_full_parts::<Block, RuntimeApi, Executor>(&config)?;
	let client = Arc::new(client);

	let select_chain = sc_consensus::LongestChain::new(backend.clone());

	let transaction_pool = sc_transaction_pool::BasicPool::new_full(
		config.transaction_pool.clone(),
		config.role.is_authority().into(),
		config.prometheus_registry(),
		task_manager.spawn_handle(),
		client.clone(),
	);

	let (grandpa_block_import, grandpa_link) =
		sc_finality_grandpa::block_import(client.clone(), &(client.clone() as Arc<_>), select_chain.clone())?;
	let justification_import = grandpa_block_import.clone();

	let (block_import, babe_link) = sc_consensus_babe::block_import(
		sc_consensus_babe::Config::get_or_compute(&*client)?,
		grandpa_block_import,
		client.clone(),
	)?;

	let inherent_data_providers = sp_inherents::InherentDataProviders::new();

	let import_queue = if instant_sealing {
		inherent_data_providers
			.register_provider(mock_timestamp_data_provider::MockTimestampInherentDataProvider)
			.map_err(Into::into)
			.map_err(sp_consensus::error::Error::InherentData)?;

		sc_consensus_manual_seal::import_queue(
			Box::new(client.clone()),
			&task_manager.spawn_handle(),
			config.prometheus_registry(),
		)
	} else {
		sc_consensus_babe::import_queue(
			babe_link.clone(),
			block_import.clone(),
			Some(Box::new(justification_import)),
			client.clone(),
			select_chain.clone(),
			inherent_data_providers.clone(),
			&task_manager.spawn_handle(),
			config.prometheus_registry(),
			sp_consensus::CanAuthorWithNativeVersion::new(client.executor().clone()),
		)?
	};

	let justification_stream = grandpa_link.justification_stream();
	let shared_authority_set = grandpa_link.shared_authority_set().clone();
	let shared_voter_state = sc_finality_grandpa::SharedVoterState::empty();

	let import_setup = (block_import, grandpa_link, babe_link.clone());
	let rpc_setup = shared_voter_state.clone();

	let finality_proof_provider =
		GrandpaFinalityProofProvider::new_for_service(backend.clone(), Some(shared_authority_set.clone()));

	let babe_config = babe_link.config().clone();
	let shared_epoch_changes = babe_link.epoch_changes().clone();

	let rpc_extensions_builder = {
		let client = client.clone();
		let keystore = keystore_container.sync_keystore();
		let transaction_pool = transaction_pool.clone();
		let select_chain = select_chain.clone();

		move |deny_unsafe, subscription_executor| -> setheum_rpc::RpcExtension {
			let deps = setheum_rpc::FullDeps {
				client: client.clone(),
				pool: transaction_pool.clone(),
				select_chain: select_chain.clone(),
				deny_unsafe,
				babe: setheum_rpc::BabeDeps {
					babe_config: babe_config.clone(),
					shared_epoch_changes: shared_epoch_changes.clone(),
					keystore: keystore.clone(),
				},
				grandpa: setheum_rpc::GrandpaDeps {
					shared_voter_state: shared_voter_state.clone(),
					shared_authority_set: shared_authority_set.clone(),
					justification_stream: justification_stream.clone(),
					subscription_executor,
					finality_provider: finality_proof_provider.clone(),
				},
			};

			setheum_rpc::create_full(deps)
		}
	};

	Ok(PartialComponents {
		client,
		backend,
		task_manager,
		keystore_container,
		select_chain,
		import_queue,
		transaction_pool,
		inherent_data_providers,
		other: (rpc_extensions_builder, import_setup, rpc_setup),
	})
}

/// Creates a full service from the configuration.
pub fn new_full<RuntimeApi, Executor>(
	mut config: Configuration,
	instant_sealing: bool,
	test: bool,
) -> Result<
	(
		TaskManager,
		InherentDataProviders,
		Arc<FullClient<RuntimeApi, Executor>>,
		Arc<sc_network::NetworkService<Block, <Block as BlockT>::Hash>>,
		Arc<sc_transaction_pool::FullPool<Block, FullClient<RuntimeApi, Executor>>>,
		sc_service::NetworkStatusSinks<Block>,
	),
	ServiceError,
>
where
	RuntimeApi: ConstructRuntimeApi<Block, FullClient<RuntimeApi, Executor>> + Send + Sync + 'static,
	RuntimeApi::RuntimeApi: RuntimeApiCollection<StateBackend = sc_client_api::StateBackendFor<FullBackend, Block>>,
	Executor: NativeExecutionDispatch + 'static,
{
	let PartialComponents {
		client,
		backend,
		mut task_manager,
		import_queue,
		keystore_container,
		select_chain,
		transaction_pool,
		inherent_data_providers,
		other: (rpc_extensions_builder, import_setup, rpc_setup),
	} = new_partial::<RuntimeApi, Executor>(&mut config, instant_sealing, test)?;

	let shared_voter_state = rpc_setup;

	let (network, network_status_sinks, system_rpc_tx, network_starter) =
		sc_service::build_network(sc_service::BuildNetworkParams {
			config: &config,
			client: client.clone(),
			transaction_pool: transaction_pool.clone(),
			spawn_handle: task_manager.spawn_handle(),
			import_queue,
			on_demand: None,
			block_announce_validator_builder: None,
		})?;

	if config.offchain_worker.enabled {
		sc_service::build_offchain_workers(
			&config,
			backend.clone(),
			task_manager.spawn_handle(),
			client.clone(),
			network.clone(),
		);
	}

	let role = config.role.clone();
	let force_authoring = config.force_authoring;
	let backoff_authoring_blocks = Some(sc_consensus_slots::BackoffAuthoringOnFinalizedHeadLagging::default());
	let name = config.network.node_name.clone();
	let enable_grandpa = !config.disable_grandpa;
	let prometheus_registry = config.prometheus_registry().cloned();

	let (_rpc_handlers, telemetry_connection_notifier) = sc_service::spawn_tasks(sc_service::SpawnTasksParams {
		config,
		backend,
		client: client.clone(),
		keystore: keystore_container.sync_keystore(),
		network: network.clone(),
		rpc_extensions_builder: Box::new(rpc_extensions_builder),
		transaction_pool: transaction_pool.clone(),
		task_manager: &mut task_manager,
		on_demand: None,
		remote_blockchain: None,
		network_status_sinks: network_status_sinks.clone(),
		system_rpc_tx,
	})?;

	let (block_import, grandpa_link, babe_link) = import_setup;

	if instant_sealing {
		if role.is_authority() {
			let env = sc_basic_authorship::ProposerFactory::new(
				task_manager.spawn_handle(),
				client.clone(),
				transaction_pool.clone(),
				prometheus_registry.as_ref(),
			);
			let authorship_future =
				sc_consensus_manual_seal::run_instant_seal(sc_consensus_manual_seal::InstantSealParams {
					block_import: client.clone(),
					env,
					client: client.clone(),
					pool: transaction_pool.pool().clone(),
					select_chain,
					consensus_data_provider: None,
					inherent_data_providers: inherent_data_providers.clone(),
				});
			// we spawn the future on a background thread managed by service.
			task_manager
				.spawn_essential_handle()
				.spawn_blocking("instant-seal", authorship_future);
		}
	} else {
		if let sc_service::config::Role::Authority { .. } = &role {
			let proposer = sc_basic_authorship::ProposerFactory::new(
				task_manager.spawn_handle(),
				client.clone(),
				transaction_pool.clone(),
				prometheus_registry.as_ref(),
			);

			let can_author_with = sp_consensus::CanAuthorWithNativeVersion::new(client.executor().clone());

			let babe_config = sc_consensus_babe::BabeParams {
				keystore: keystore_container.sync_keystore(),
				client: client.clone(),
				select_chain,
				env: proposer,
				block_import,
				sync_oracle: network.clone(),
				inherent_data_providers: inherent_data_providers.clone(),
				force_authoring,
				backoff_authoring_blocks,
				babe_link,
				can_author_with,
			};

			let babe = sc_consensus_babe::start_babe(babe_config)?;
			task_manager
				.spawn_essential_handle()
				.spawn_blocking("babe-proposer", babe);
		}

		// if the node isn't actively participating in consensus then it doesn't
		// need a keystore, regardless of which protocol we use below.
		let keystore = if role.is_authority() {
			Some(keystore_container.sync_keystore())
		} else {
			None
		};

		let config = sc_finality_grandpa::Config {
			// FIXME #1578 make this available through chainspec
			gossip_duration: std::time::Duration::from_millis(333),
			justification_period: 512,
			name: Some(name),
			observer_enabled: false,
			keystore,
			is_authority: role.is_authority(),
		};

		if enable_grandpa {
			// start the full GRANDPA voter
			// NOTE: non-authorities could run the GRANDPA observer protocol, but at
			// this point the full voter should provide better guarantees of block
			// and vote data availability than the observer. The observer has not
			// been tested extensively yet and having most nodes in a network run it
			// could lead to finality stalls.
			let grandpa_config = sc_finality_grandpa::GrandpaParams {
				config,
				link: grandpa_link,
				network: network.clone(),
				telemetry_on_connect: telemetry_connection_notifier.map(|x| x.on_connect_stream()),
				voting_rule: sc_finality_grandpa::VotingRulesBuilder::default().build(),
				prometheus_registry,
				shared_voter_state,
			};

			// the GRANDPA voter task is considered infallible, i.e.
			// if it fails we take down the service with it.
			task_manager
				.spawn_essential_handle()
				.spawn_blocking("grandpa-voter", sc_finality_grandpa::run_grandpa_voter(grandpa_config)?);
		}
	}

	network_starter.start_network();
	Ok((
		task_manager,
		inherent_data_providers,
		client,
		network,
		transaction_pool,
		network_status_sinks,
	))
}

/// Creates a light service from the configuration.
pub fn new_light<RuntimeApi, Executor>(
	mut config: Configuration,
) -> Result<
	(
		TaskManager,
		RpcHandlers,
		Arc<LightClient<RuntimeApi, Executor>>,
		Arc<sc_network::NetworkService<Block, <Block as BlockT>::Hash>>,
		Arc<
			sc_transaction_pool::LightPool<
				Block,
				LightClient<RuntimeApi, Executor>,
				sc_network::config::OnDemand<Block>,
			>,
		>,
	),
	ServiceError,
>
where
	RuntimeApi: ConstructRuntimeApi<Block, LightClient<RuntimeApi, Executor>> + Send + Sync + 'static,
	<RuntimeApi as ConstructRuntimeApi<Block, LightClient<RuntimeApi, Executor>>>::RuntimeApi:
		RuntimeApiCollection<StateBackend = sc_client_api::StateBackendFor<LightBackend, Block>>,
	Executor: NativeExecutionDispatch + 'static,
{
	let (client, backend, keystore_container, mut task_manager, on_demand) =
		sc_service::new_light_parts::<Block, RuntimeApi, Executor>(&config)?;

	config
		.network
		.extra_sets
		.push(sc_finality_grandpa::grandpa_peers_set_config());

	let select_chain = sc_consensus::LongestChain::new(backend.clone());

	let transaction_pool = Arc::new(sc_transaction_pool::BasicPool::new_light(
		config.transaction_pool.clone(),
		config.prometheus_registry(),
		task_manager.spawn_handle(),
		client.clone(),
		on_demand.clone(),
	));

	let (grandpa_block_import, _) =
		sc_finality_grandpa::block_import(client.clone(), &(client.clone() as Arc<_>), select_chain.clone())?;
	let justification_import = grandpa_block_import.clone();

	let (babe_block_import, babe_link) = sc_consensus_babe::block_import(
		sc_consensus_babe::Config::get_or_compute(&*client)?,
		grandpa_block_import,
		client.clone(),
	)?;

	let inherent_data_providers = sp_inherents::InherentDataProviders::new();

	let import_queue = sc_consensus_babe::import_queue(
		babe_link,
		babe_block_import,
		Some(Box::new(justification_import)),
		client.clone(),
		select_chain,
		inherent_data_providers,
		&task_manager.spawn_handle(),
		config.prometheus_registry(),
		sp_consensus::NeverCanAuthor,
	)?;

	let (network, network_status_sinks, system_rpc_tx, network_starter) =
		sc_service::build_network(sc_service::BuildNetworkParams {
			config: &config,
			client: client.clone(),
			transaction_pool: transaction_pool.clone(),
			spawn_handle: task_manager.spawn_handle(),
			import_queue,
			on_demand: Some(on_demand.clone()),
			block_announce_validator_builder: None,
		})?;
	network_starter.start_network();

	if config.offchain_worker.enabled {
		sc_service::build_offchain_workers(
			&config,
			backend.clone(),
			task_manager.spawn_handle(),
			client.clone(),
			network.clone(),
		);
	}

	let light_deps = setheum_rpc::LightDeps {
		remote_blockchain: backend.remote_blockchain(),
		fetcher: on_demand.clone(),
		client: client.clone(),
		pool: transaction_pool.clone(),
	};

	let rpc_extensions = setheum_rpc::create_light(light_deps);

	let (rpc_handlers, _telemetry_connection_notifier) = sc_service::spawn_tasks(sc_service::SpawnTasksParams {
		on_demand: Some(on_demand),
		remote_blockchain: Some(backend.remote_blockchain()),
		rpc_extensions_builder: Box::new(sc_service::NoopRpcExtensionBuilder(rpc_extensions)),
		client: client.clone(),
		transaction_pool: transaction_pool.clone(),
		config,
		keystore: keystore_container.sync_keystore(),
		backend,
		network_status_sinks,
		system_rpc_tx,
		network: network.clone(),
		task_manager: &mut task_manager,
	})?;

	Ok((task_manager, rpc_handlers, client, network, transaction_pool))
}

/// Builds a new object suitable for chain operations.
pub fn new_chain_ops(
	mut config: &mut Configuration,
) -> Result<
	(
		Arc<Client>,
		Arc<FullBackend>,
		sp_consensus::import_queue::BasicQueue<Block, sp_trie::PrefixedMemoryDB<BlakeTwo256>>,
		TaskManager,
	),
	ServiceError,
> {
	config.keystore = sc_service::config::KeystoreConfig::InMemory;
	if config.chain_spec.is_newrome() {
		#[cfg(feature = "with-newrome-runtime")]
		{
			let PartialComponents {
				client,
				backend,
				import_queue,
				task_manager,
				..
			} = new_partial::<newrome_runtime::RuntimeApi, NewromeExecutor>(config, false, false)?;
			Ok((Arc::new(Client::Newrome(client)), backend, import_queue, task_manager))
		}
		#[cfg(not(feature = "with-newrome-runtime"))]
		Err("Newrome runtime is not available. Please compile the node with `--features with-newrome-runtime` to enable it.".into())
	} else if config.chain_spec.is_neom() {
		#[cfg(feature = "with-neom-runtime")]
		{
			let PartialComponents {
				client,
				backend,
				import_queue,
				task_manager,
				..
			} = new_partial::<neom_runtime::RuntimeApi, NeomExecutor>(config, false, false)?;
			Ok((Arc::new(Client::Neom(client)), backend, import_queue, task_manager))
		}
		#[cfg(not(feature = "with-neom-runtime"))]
		Err("Neom runtime is not available. Please compile the node with `--features with-neom-runtime` to enable it.".into())
	} else {
		#[cfg(feature = "with-setheum-runtime")]
		{
			let PartialComponents {
				client,
				backend,
				import_queue,
				task_manager,
				..
			} = new_partial::<setheum_runtime::RuntimeApi, SetheumExecutor>(config, false, false)?;
			Ok((Arc::new(Client::Setheum(client)), backend, import_queue, task_manager))
		}
		#[cfg(not(feature = "with-setheum-runtime"))]
		Err("Setheum runtime is not available. Please compile the node with `--features with-setheum-runtime` to enable it.".into())
	}
}

/// Build a new light node.
pub fn build_light(config: Configuration) -> Result<TaskManager, ServiceError> {
	if config.chain_spec.is_setheum() {
		#[cfg(feature = "with-setheum-runtime")]
		return new_light::<setheum_runtime::RuntimeApi, SetheumExecutor>(config).map(|r| r.0);
		#[cfg(not(feature = "with-setheum-runtime"))]
		Err("Setheum runtime is not available. Please compile the node with `--features with-setheum-runtime` to enable it.".into())
	} else if config.chain_spec.is_neom() {
		#[cfg(feature = "with-neom-runtime")]
		return new_light::<neom_runtime::RuntimeApi, NeomExecutor>(config).map(|r| r.0);
		#[cfg(not(feature = "with-neom-runtime"))]
		Err("Neom runtime is not available. Please compile the node with `--features with-neom-runtime` to enable it.".into())
	} else {
		#[cfg(feature = "with-newrome-runtime")]
		return new_light::<newrome_runtime::RuntimeApi, NewromeExecutor>(config).map(|r| r.0);
		#[cfg(not(feature = "with-newrome-runtime"))]
		Err("Newrome runtime is not available. Please compile the node with `--features with-newrome-runtime` to enable it.".into())
	}
}

pub fn build_full(
	config: Configuration,
	instant_sealing: bool,
	test: bool,
) -> Result<(Arc<Client>, sc_service::NetworkStatusSinks<Block>, TaskManager), ServiceError> {
	if config.chain_spec.is_setheum() {
		#[cfg(feature = "with-setheum-runtime")]
		{
			let (task_manager, _, client, _, _, network_status_sinks) =
				new_full::<setheum_runtime::RuntimeApi, SetheumExecutor>(config, instant_sealing, test)?;
			Ok((Arc::new(Client::Setheum(client)), network_status_sinks, task_manager))
		}
		#[cfg(not(feature = "with-setheum-runtime"))]
		Err("Setheum runtime is not available. Please compile the node with `--features with-setheum-runtime` to enable it.".into())
	} else if config.chain_spec.is_neom() {
		#[cfg(feature = "with-neom-runtime")]
		{
			let (task_manager, _, client, _, _, network_status_sinks) =
				new_full::<neom_runtime::RuntimeApi, NeomExecutor>(config, instant_sealing, test)?;
			Ok((Arc::new(Client::Neom(client)), network_status_sinks, task_manager))
		}
		#[cfg(not(feature = "with-neom-runtime"))]
		Err("Neom runtime is not available. Please compile the node with `--features with-neom-runtime` to enable it.".into())
	} else {
		#[cfg(feature = "with-newrome-runtime")]
		{
			let (task_manager, _, client, _, _, network_status_sinks) =
				new_full::<newrome_runtime::RuntimeApi, NewromeExecutor>(config, instant_sealing, test)?;
			Ok((Arc::new(Client::Newrome(client)), network_status_sinks, task_manager))
		}
		#[cfg(not(feature = "with-newrome-runtime"))]
		Err("Newrome runtime is not available. Please compile the node with `--features with-newrome-runtime` to enable it.".into())
	}
}
