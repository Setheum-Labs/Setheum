// بِسْمِ اللَّهِ الرَّحْمَنِ الرَّحِيم

// This file is part of Setheum.

// Copyright (C) 2019-Present Setheum Labs.
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

use std::path::PathBuf;

use setheum_client::BlockNumber;
use anyhow::Result;
use clap::{Parser, Subcommand};

use crate::commands::{status, try_finalize, Connections};
mod commands;

#[derive(Debug, Parser, Clone)]
#[clap(version = "1.0")]
struct Config {
    /// WS endpoint that we use to send finalization requests. Needs to accept unsafe queries.
    #[clap(long, default_value = "ws://127.0.0.1:9944")]
    pub primary_endpoint: String,

    /// Additional ws endpoints that are used for doublechecking. Good to have at least one.
    #[clap(long, value_delimiter = ',')]
    pub secondary_endpoints: Vec<String>,

    /// Specific command that executes either a signed transaction or is an auxiliary command
    #[clap(subcommand)]
    pub command: Command,
}

#[derive(Debug, Clone, Subcommand)]
pub enum Command {
    /// Show status according to primary and secondary endpoints (read-only method)
    Status,
    /// Attempt finalizing the specified number of blocks
    TryFinalize {
        /// Path to the seed phrase to emergency finalizer.
        #[clap(long, default_value = "seed.txt")]
        seed_path: PathBuf,

        /// The number of blocks to finalize. Should be no more than 20.
        #[clap(long)]
        how_many: BlockNumber,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    let Config {
        primary_endpoint,
        secondary_endpoints,
        command,
    } = Config::parse();
    println!(
        "Running with primary {} and {} secondaries {:?} ...\n",
        primary_endpoint,
        secondary_endpoints.len(),
        secondary_endpoints
    );
    let connections = Connections::new(primary_endpoint, secondary_endpoints).await;
    match command {
        Command::Status => {
            status(connections).await?;
        }
        Command::TryFinalize {
            seed_path,
            how_many,
        } => try_finalize(connections, seed_path, how_many).await?,
    }
    Ok(())
}
