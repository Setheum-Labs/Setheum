// This file is part of Setheum.

// Copyright (C) 2019-2021 Setheum Labs.
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
#![allow(clippy::borrowed_box)]

use crate::cli::{Cli, Subcommand};
use sc_cli::{ChainSpec, Result, Role, RuntimeVersion, SubstrateCli};
use service::{chain_spec, IdentifyVariant};
use sc_service::config::PrometheusConfig;
use sp_runtime::traits::Block as BlockT;

fn get_exec_name() -> Option<String> {
	std::env::current_exe()
		.ok()
		.and_then(|pb| pb.file_name().map(|s| s.to_os_string()))
		.and_then(|s| s.into_string().ok())
}

impl SubstrateCli for Cli {
	fn impl_name() -> String {
		"Setheum Node".into()
	}

	fn impl_version() -> String {
		env!("SUBSTRATE_CLI_IMPL_VERSION").into()
	}

	fn description() -> String {
		env!("CARGO_PKG_DESCRIPTION").into()
	}

	fn author() -> String {
		env!("CARGO_PKG_AUTHORS").into()
	}

	fn support_url() -> String {
		"https://github.com/Setheum-Labs/Setheum/issues".into()
	}

	fn copyright_start_year() -> i32 {
		2019
	}

	fn load_spec(&self, id: &str) -> std::result::Result<Box<dyn sc_service::ChainSpec>, String> {
		if id.is_empty() {
			return Err("Not specific which chain to run.".into());
		}

		Ok(match id {
			#[cfg(feature = "with-newrome-runtime")]
			"dev" => Box::new(chain_spec::newrome::development_testnet_config()?),
			#[cfg(feature = "with-newrome-runtime")]
			"local" => Box::new(chain_spec::newrome::local_testnet_config()?),
			#[cfg(feature = "with-newrome-runtime")]
			"newrome" => Box::new(chain_spec::newrome::newrome_testnet_config()?),
			#[cfg(feature = "with-newrome-runtime")]
			"newrome-latest" => Box::new(chain_spec::newrome::latest_newrome_testnet_config()?),
			#[cfg(feature = "with-setheum-runtime")]
			"setheum" => Box::new(chain_spec::setheum::setheum_config()?),
			#[cfg(feature = "with-setheum-runtime")]
			"setheum-latest" => Box::new(chain_spec::setheum::latest_setheum_config()?),
			path => {
				let path = std::path::PathBuf::from(path);

				let starts_with = |prefix: &str| {
					path.file_name()
						.map(|f| f.to_str().map(|s| s.starts_with(&prefix)))
						.flatten()
						.unwrap_or(false)
				};

				if starts_with("setheum") {
					#[cfg(feature = "with-setheum-runtime")]
					{
						Box::new(chain_spec::setheum::ChainSpec::from_json_file(path)?)
					}
					#[cfg(not(feature = "with-setheum-runtime"))]
					return Err(service::SETHEUM_RUNTIME_NOT_AVAILABLE.into());
				} else {
					#[cfg(feature = "with-newrome-runtime")]
					{
						Box::new(chain_spec::newrome::ChainSpec::from_json_file(path)?)
					}
					#[cfg(not(feature = "with-newrome-runtime"))]
					return Err(service::NEWROME_RUNTIME_NOT_AVAILABLE.into());
				}
			}
		})
	}

	fn native_runtime_version(spec: &Box<dyn sc_service::ChainSpec>) -> &'static RuntimeVersion {
		if spec.is_setheum() {
			#[cfg(feature = "with-setheum-runtime")]
			return &service::setheum_runtime::VERSION;
			#[cfg(not(feature = "with-setheum-runtime"))]
			panic!("{}", service::SETHEUM_RUNTIME_NOT_AVAILABLE);
		} else {
			#[cfg(feature = "with-newrome-runtime")]
			return &service::newrome_runtime::VERSION;
			#[cfg(not(feature = "with-newrome-runtime"))]
			panic!("{}", service::NEWROME_RUNTIME_NOT_AVAILABLE);
		}
	}
}

fn set_default_ss58_version(spec: &Box<dyn service::ChainSpec>) {
	use sp_core::crypto::Ss58AddressFormat;

	let ss58_version = if spec.is_setheum() {
		Ss58AddressFormat::SetheumAccount
	} else {
		Ss58AddressFormat::SubstrateAccount
	};

	sp_core::crypto::set_default_ss58_version(ss58_version);
}

macro_rules! with_runtime_or_err {
	($chain_spec:expr, { $( $code:tt )* }) => {
		if $chain_spec.is_setheum() {
			#[cfg(feature = "with-setheum-runtime")]
			#[allow(unused_imports)]
			use service::{setheum_runtime::{Block, RuntimeApi}, SetheumExecutor as Executor};
			#[cfg(feature = "with-setheum-runtime")]
			$( $code )*

			#[cfg(not(feature = "with-setheum-runtime"))]
			return Err(service::SETHEUM_RUNTIME_NOT_AVAILABLE.into());
		} else {
			#[cfg(feature = "with-newrome-runtime")]
			#[allow(unused_imports)]
			use service::{newrome_runtime::{Block, RuntimeApi}, NewRomeExecutor as Executor};
			#[cfg(feature = "with-newrome-runtime")]
			$( $code )*

			#[cfg(not(feature = "with-newrome-runtime"))]
			return Err(service::NEWROME_RUNTIME_NOT_AVAILABLE.into());
		}
	}
}

/// Parses setheum specific CLI arguments and run the service.
pub fn run() -> sc_cli::Result<()> {
	let cli = Cli::from_args();

	match &cli.subcommand {

		Some(Subcommand::Inspect(cmd)) => {
			let runner = cli.create_runner(cmd)?;
			let chain_spec = &runner.config().chain_spec;

			set_default_ss58_version(chain_spec);

			runner.sync_run(|config| {
				let (client, _, _) = service::build_full(config, false, false)?;
				cmd.run(client)
			})
		}

		Some(Subcommand::Benchmark(cmd)) => {
			let runner = cli.create_runner(cmd)?;
			let chain_spec = &runner.config().chain_spec;

			set_default_ss58_version(chain_spec);

			with_runtime_or_err!(chain_spec, {
				return runner.sync_run(|config| cmd.run::<Block, Executor>(config));
			})
		}

		Some(Subcommand::Key(cmd)) => cmd.run(&cli),
		Some(Subcommand::Sign(cmd)) => cmd.run(),
		Some(Subcommand::Verify(cmd)) => cmd.run(),
		Some(Subcommand::Vanity(cmd)) => cmd.run(),

		Some(Subcommand::BuildSpec(cmd)) => {
			let runner = cli.create_runner(cmd)?;
			runner.sync_run(|config| cmd.run(config.chain_spec, config.network))
		}

		Some(Subcommand::CheckBlock(cmd)) => {
			let runner = cli.create_runner(cmd)?;
			let chain_spec = &runner.config().chain_spec;

			set_default_ss58_version(chain_spec);

			runner.async_run(|mut config| {
				let (client, _, import_queue, task_manager) = service::new_chain_ops(&mut config)?;
				Ok((cmd.run(client, import_queue), task_manager))
			})
		}

		Some(Subcommand::ExportBlocks(cmd)) => {
			let runner = cli.create_runner(cmd)?;
			let chain_spec = &runner.config().chain_spec;

			set_default_ss58_version(chain_spec);

			runner.async_run(|mut config| {
				let (client, _, _, task_manager) = service::new_chain_ops(&mut config)?;
				Ok((cmd.run(client, config.database), task_manager))
			})
		}

		Some(Subcommand::ExportState(cmd)) => {
			let runner = cli.create_runner(cmd)?;
			let chain_spec = &runner.config().chain_spec;

			set_default_ss58_version(chain_spec);

			runner.async_run(|mut config| {
				let (client, _, _, task_manager) = service::new_chain_ops(&mut config)?;
				Ok((cmd.run(client, config.chain_spec), task_manager))
			})
		}

		Some(Subcommand::ImportBlocks(cmd)) => {
			let runner = cli.create_runner(cmd)?;
			let chain_spec = &runner.config().chain_spec;

			set_default_ss58_version(chain_spec);

			runner.async_run(|mut config| {
				let (client, _, import_queue, task_manager) = service::new_chain_ops(&mut config)?;
				Ok((cmd.run(client, import_queue), task_manager))
			})
		}

		Some(Subcommand::PurgeChain(cmd)) => {
			let runner = cli.create_runner(cmd)?;
			runner.sync_run(|config| cmd.run(config.database))
		}

		Some(Subcommand::Revert(cmd)) => {
			let runner = cli.create_runner(cmd)?;
			let chain_spec = &runner.config().chain_spec;

			set_default_ss58_version(chain_spec);

			runner.async_run(|mut config| {
				let (client, backend, _, task_manager) = service::new_chain_ops(&mut config)?;
				Ok((cmd.run(client, backend), task_manager))
			})
		}

		#[cfg(feature = "try-runtime")]
		Some(Subcommand::TryRuntime(cmd)) => {
			let runner = cli.create_runner(cmd)?;
			let chain_spec = &runner.config().chain_spec;

			set_default_ss58_version(chain_spec);

			with_runtime_or_err!(chain_spec, {
				return runner.async_run(|config| {
					let registry = config.prometheus_config.as_ref().map(|cfg| &cfg.registry);
					let task_manager = sc_service::TaskManager::new(config.task_executor.clone(), registry)
						.map_err(|e| sc_cli::Error::Service(sc_service::Error::Prometheus(e)))?;
					Ok((cmd.run::<Block, Executor>(config), task_manager))
				});
			})
		}

		None => {
			let runner = cli.create_runner(&cli.run)?;
			let chain_spec = &runner.config().chain_spec;

			set_default_ss58_version(chain_spec);

			runner.run_node_until_exit(|config| async move {
				match config.role {
					Role::Light => service::build_light(config),
					_ => {
						service::build_full(config, cli.instant_sealing, false).map(|(_, _, task_manager)| task_manager)
					}
				}
				.map_err(sc_cli::Error::Service)
			})
		}
	}
}
