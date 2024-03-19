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

use std::fmt::Display;

use async_trait::async_trait;

use crate::{
    aleph_primitives::BlockNumber,
    party::{backup::ABFTBackup, manager::AuthorityTask},
    AuthorityId, NodeIndex, SessionId,
};

/// Abstraction of the chain state.
pub trait ChainState {
    /// Returns best block number.
    fn best_block_number(&self) -> BlockNumber;
    /// Returns last finalized block number.
    fn finalized_number(&self) -> BlockNumber;
}

#[async_trait]
/// Abstraction over session related tasks.
pub trait NodeSessionManager {
    type Error: Display;

    /// Spawns every task needed for an authority to run in a session.
    async fn spawn_authority_task_for_session(
        &self,
        session: SessionId,
        node_id: NodeIndex,
        backup: ABFTBackup,
        authorities: &[AuthorityId],
    ) -> AuthorityTask;

    /// Prepare validator session.
    fn early_start_validator_session(
        &self,
        session: SessionId,
        node_id: NodeIndex,
        authorities: &[AuthorityId],
    ) -> Result<(), Self::Error>;

    /// Starts nonvalidator session.
    fn start_nonvalidator_session(
        &self,
        session: SessionId,
        authorities: &[AuthorityId],
    ) -> Result<(), Self::Error>;

    /// Terminates the session.
    fn stop_session(&self, session: SessionId) -> Result<(), Self::Error>;

    /// Returns idx of the node if it is in the authority set, None otherwise
    fn node_idx(&self, authorities: &[AuthorityId]) -> Option<NodeIndex>;
}
