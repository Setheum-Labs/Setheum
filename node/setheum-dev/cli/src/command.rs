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
#![allow(clippy::borrowed_box)]

use crate::cli::{Cli, Subcommand};
use sc_cli::{Role, RuntimeVersion, SubstrateCli};
use sc_service::ChainType;
use service::{chain_spec, IdentifyVariant};

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
		"Setheum Labs".into()
	}

	fn support_url() -> String {
		"https://github.com/SetheumNetwork/Setheum/issues".into()
	}

	fn copyright_start_year() -> i32 {
		2019
	}

	fn load_spec(&self, id: &str) -> std::result::Result<Box<dyn sc_service::ChainSpec>, String> {
		let id = if id.is_empty() {
			// The binary prefix is always setheum.
			// Make Newrome the default chain spec.
			"newrome"
		} else {
			id
		};

		Ok(match id {
			#[cfg(feature = "with-newrome-runtime")]
			"dev" => Box::new(chain_spec::newrome::development_testnet_config()?),
			#[cfg(feature = "with-newrome-runtime")]
			"local" => Box::new(chain_spec::newrome::local_testnet_config()?),
			#[cfg(feature = "with-newrome-runtime")]
			"newrome" => Box::new(chain_spec::newrome::newrome_testnet_config()?),
			#[cfg(feature = "with-newrome-runtime")]
			"newrome-latest" => Box::new(chain_spec::newrome::latest_newrome_testnet_config()?),
			#[cfg(feature = "with-neom-runtime")]
			"neom" => Box::new(chain_spec::neom::neom_config()?),
			#[cfg(feature = "with-neom-runtime")]
			"neom-latest" => Box::new(chain_spec::neom::latest_neom_config()?),
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

				if starts_with("neom") {
					#[cfg(feature = "with-neom-runtime")]
					{
						Box::new(chain_spec::neom::ChainSpec::from_json_file(path)?)
					}

					#[cfg(not(feature = "with-neom-runtime"))]
					return Err("Neom runtime is not available. Please compile the node with `--features with-neom-runtime` to enable it.".into());
				} else if starts_with("setheum") {
					#[cfg(feature = "with-setheum-runtime")]
					{
						Box::new(chain_spec::setheum::ChainSpec::from_json_file(path)?)
					}
					#[cfg(not(feature = "with-setheum-runtime"))]
					return Err("Setheum runtime is not available. Please compile the node with `--features with-setheum-runtime` to enable it.".into());
				} else {
					#[cfg(feature = "with-newrome-runtime")]
					{
						Box::new(chain_spec::newrome::ChainSpec::from_json_file(path)?)
					}
					#[cfg(not(feature = "with-newrome-runtime"))]
					return Err("Newrome runtime is not available. Please compile the node with `--features with-newrome-runtime` to enable it.".into());
				}
			}
		})
	}

	fn native_runtime_version(spec: &Box<dyn sc_service::ChainSpec>) -> &'static RuntimeVersion {
		if spec.is_setheum() {
			#[cfg(feature = "with-setheum-runtime")]
			return &service::setheum_runtime::VERSION;
			#[cfg(not(feature = "with-setheum-runtime"))]
			panic!("Setheum runtime is not available. Please compile the node with `--features with-setheum-runtime` to enable it.");
		} else if spec.is_neom() {
			#[cfg(feature = "with-neom-runtime")]
			return &service::neom_runtime::VERSION;
			#[cfg(not(feature = "with-neom-runtime"))]
			panic!("Neom runtime is not available. Please compile the node with `--features with-neom-runtime` to enable it.");
		} else {
			#[cfg(feature = "with-newrome-runtime")]
			return &service::newrome_runtime::VERSION;
			#[cfg(not(feature = "with-newrome-runtime"))]
			panic!("Newrome runtime is not available. Please compile the node with `--features with-newrome-runtime` to enable it.");
		}
	}
}

fn set_default_ss58_version(spec: &Box<dyn service::ChainSpec>) {
	use sp_core::crypto::Ss58AddressFormat;

	let ss58_version = if spec.is_neom() {
		Ss58AddressFormat::NeomAccount
	} else if spec.is_setheum() {
		Ss58AddressFormat::SetheumAccount
	} else {
		Ss58AddressFormat::SubstrateAccount
	};

	sp_core::crypto::set_default_ss58_version(ss58_version);
}

/// Parses setheum specific CLI arguments and run the service.
pub fn run() -> sc_cli::Result<()> {
	let cli = Cli::from_args();

	match &cli.subcommand {
		None => {
			let runner = cli.create_runner(&cli.run)?;
			let chain_spec = &runner.config().chain_spec;

			set_default_ss58_version(chain_spec);

			if cli.instant_sealing && chain_spec.chain_type() != ChainType::Development {
				return Err("Instant sealing can be turned on only in `--dev` mode".into());
			}

			runner
				.run_node_until_exit(|config| async move {
					match config.role {
						Role::Light => service::build_light(config),
						_ => service::build_full(config, cli.instant_sealing, false)
							.map(|(_, _, task_manager)| task_manager),
					}
				})
				.map_err(sc_cli::Error::Service)
		}

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

			#[cfg(feature = "with-setheum-runtime")]
			return runner.sync_run(|config| cmd.run::<service::setheum_runtime::Block, service::SetheumExecutor>(config));

			#[cfg(feature = "with-neom-runtime")]
			return runner
				.sync_run(|config| cmd.run::<service::neom_runtime::Block, service::NeomExecutor>(config));

			#[cfg(feature = "with-newrome-runtime")]
			return runner
				.sync_run(|config| cmd.run::<service::newrome_runtime::Block, service::NewromeExecutor>(config));
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
	}
}
