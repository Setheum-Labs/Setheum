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

use std::{
    fmt::{Display, Formatter, Result as FmtResult},
    sync::{
        atomic::{AtomicBool, AtomicUsize},
        Arc,
    },
};

use futures::stream::StreamExt;
use log::{debug, trace, warn};
use sc_network::config::FullNetworkConfiguration;
use sc_network_common::sync::SyncEvent;
use sc_network_sync::{service::chain_sync::ToServiceCommand, SyncingService};
use sc_utils::mpsc::{tracing_unbounded, TracingUnboundedReceiver, TracingUnboundedSender};
use sp_runtime::traits::{Block, Header};

use crate::{
    base_protocol::{handler::Handler, LOG_TARGET},
    BlockHash, BlockNumber,
};

#[derive(Debug)]
pub enum Error {
    NoIncomingCommands,
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        use Error::*;
        match self {
            NoIncomingCommands => write!(f, "Channel with commands from user closed."),
        }
    }
}

/// A service that needs to be run to have the base protocol of the network work.
pub struct Service<B>
where
    B: Block<Hash = BlockHash>,
    B::Header: Header<Number = BlockNumber>,
{
    handler: Handler<B>,
    commands_from_user: TracingUnboundedReceiver<ToServiceCommand<B>>,
    events_for_users: Vec<TracingUnboundedSender<SyncEvent>>,
}

impl<B> Service<B>
where
    B: Block<Hash = BlockHash>,
    B::Header: Header<Number = BlockNumber>,
{
    /// Create a new service.
    // TODO: This shouldn't need to return the substrate type after replacing RPCs.
    // In particular, it shouldn't depend on `B`. This is also the only reason why
    // the `major_sync` argument is needed.
    pub fn new(
        major_sync: Arc<AtomicBool>,
        genesis_hash: B::Hash,
        net_config: &FullNetworkConfiguration,
    ) -> (Self, SyncingService<B>) {
        let (commands_for_service, commands_from_user) =
            tracing_unbounded("mpsc_base_protocol", 100_000);
        (
            Service {
                handler: Handler::new(genesis_hash, net_config),
                commands_from_user,
                events_for_users: Vec::new(),
            },
            SyncingService::new(
                commands_for_service,
                // We don't care about this one, so a dummy value.
                Arc::new(AtomicUsize::new(0)),
                major_sync,
            ),
        )
    }

    fn handle_command(&mut self, command: ToServiceCommand<B>) {
        use ToServiceCommand::*;
        match command {
            EventStream(events_for_user) => self.events_for_users.push(events_for_user),
            PeersInfo(response) => {
                if response.send(self.handler.peers_info()).is_err() {
                    debug!(
                        target: LOG_TARGET,
                        "Failed to send response to peers info request."
                    );
                }
            }
            BestSeenBlock(response) => {
                if response.send(None).is_err() {
                    debug!(
                        target: LOG_TARGET,
                        "Failed to send response to best block request."
                    );
                }
            }
            Status(_) => {
                // We are explicitly dropping the response channel to cause an `Err(())` to be returned in the interface, as this produces the desired results for us.
                trace!(target: LOG_TARGET, "Got status request, ignoring.");
            }
            _ => {
                warn!(target: LOG_TARGET, "Got unexpected service command.");
            }
        }
    }

    /// Run the service managing the base protocol.
    pub async fn run(mut self) -> Result<(), Error> {
        use Error::*;
        loop {
            let command = self
                .commands_from_user
                .next()
                .await
                .ok_or(NoIncomingCommands)?;
            self.handle_command(command);
        }
    }
}
