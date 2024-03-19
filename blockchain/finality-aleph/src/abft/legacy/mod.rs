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

use legacy_aleph_bft::{default_config, Config, LocalIO, Terminator};
use log::debug;
use network_clique::SpawnHandleT;

mod network;
mod traits;

pub use network::NetworkData;

use super::common::{sanity_check_round_delays, unit_creation_delay_fn, MAX_ROUNDS};
pub use crate::aleph_primitives::{BlockHash, BlockNumber, LEGACY_FINALITY_VERSION as VERSION};
use crate::{
    abft::NetworkWrapper,
    block::{Header, HeaderBackend},
    data_io::{
        legacy::{AlephData, OrderedDataInterpreter},
        SubstrateChainInfoProvider,
    },
    network::data::Network,
    oneshot,
    party::{
        backup::ABFTBackup,
        manager::{Task, TaskCommon},
    },
    Keychain, LegacyNetworkData, NodeIndex, SessionId, UnitCreationDelay,
};

pub fn run_member<H, C, ADN>(
    subtask_common: TaskCommon,
    multikeychain: Keychain,
    config: Config,
    network: NetworkWrapper<LegacyNetworkData, ADN>,
    data_provider: impl legacy_aleph_bft::DataProvider<AlephData> + Send + 'static,
    ordered_data_interpreter: OrderedDataInterpreter<SubstrateChainInfoProvider<H, C>>,
    backup: ABFTBackup,
) -> Task
where
    H: Header,
    C: HeaderBackend<H> + Send + 'static,
    ADN: Network<LegacyNetworkData> + 'static,
{
    // Remove this check once one is implemented on the AlephBFT side.
    // Checks that the total time of a session is at least 7 days.
    sanity_check_round_delays(
        config.max_round,
        config.delay_config.unit_creation_delay.clone(),
    );
    let TaskCommon {
        spawn_handle,
        session_id,
    } = subtask_common;
    let (stop, exit) = oneshot::channel();
    let member_terminator = Terminator::create_root(exit, "member");
    let local_io = LocalIO::new(data_provider, ordered_data_interpreter, backup.0, backup.1);

    let task = {
        let spawn_handle = spawn_handle.clone();
        async move {
            debug!(target: "aleph-party", "Running the member task for {:?}", session_id);
            legacy_aleph_bft::run_session(
                config,
                local_io,
                network,
                multikeychain,
                spawn_handle,
                member_terminator,
            )
            .await;
            debug!(target: "aleph-party", "Member task stopped for {:?}", session_id);
        }
    };

    let handle = spawn_handle.spawn_essential("aleph/consensus_session_member", task);
    Task::new(handle, stop)
}

pub fn create_aleph_config(
    n_members: usize,
    node_id: NodeIndex,
    session_id: SessionId,
    unit_creation_delay: UnitCreationDelay,
) -> Config {
    let mut config = default_config(n_members.into(), node_id.into(), session_id.0 as u64);
    config.delay_config.unit_creation_delay = unit_creation_delay_fn(unit_creation_delay);
    config.max_round = MAX_ROUNDS;
    config
}
