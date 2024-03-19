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

use parity_scale_codec::{Decode, Encode};
use serde::{Deserialize, Serialize};

use crate::aleph_primitives::BlockNumber;

#[derive(PartialEq, Eq, Clone, Debug)]
pub struct SessionBoundaries {
    first_block: BlockNumber,
    last_block: BlockNumber,
}

impl SessionBoundaries {
    pub fn first_block(&self) -> BlockNumber {
        self.first_block
    }

    pub fn last_block(&self) -> BlockNumber {
        self.last_block
    }
}

#[derive(Clone, Debug)]
pub struct SessionBoundaryInfo {
    session_period: SessionPeriod,
}

/// Struct for getting the session boundaries.
impl SessionBoundaryInfo {
    pub const fn new(session_period: SessionPeriod) -> Self {
        Self { session_period }
    }

    pub fn boundaries_for_session(&self, session_id: SessionId) -> SessionBoundaries {
        SessionBoundaries {
            first_block: self.first_block_of_session(session_id),
            last_block: self.last_block_of_session(session_id),
        }
    }

    /// Returns session id of the session that block belongs to.
    pub fn session_id_from_block_num(&self, n: BlockNumber) -> SessionId {
        SessionId(n / self.session_period.0)
    }

    /// Returns block number which is the last block of the session.
    pub fn last_block_of_session(&self, session_id: SessionId) -> BlockNumber {
        (session_id.0 + 1) * self.session_period.0 - 1
    }

    /// Returns block number which is the first block of the session.
    pub fn first_block_of_session(&self, session_id: SessionId) -> BlockNumber {
        session_id.0 * self.session_period.0
    }
}

#[cfg(test)]
pub mod testing {
    use sp_runtime::testing::UintAuthorityId;

    use crate::aleph_primitives::SessionAuthorityData;

    pub fn authority_data(from: u32, to: u32) -> SessionAuthorityData {
        SessionAuthorityData::new(
            (from..to)
                .map(|id| UintAuthorityId(id.into()).to_public_key())
                .collect(),
            None,
        )
    }
}

#[derive(
    Copy,
    Clone,
    Debug,
    Default,
    Eq,
    PartialEq,
    Hash,
    Ord,
    PartialOrd,
    Encode,
    Decode,
    Serialize,
    Deserialize,
)]
pub struct SessionId(pub u32);

impl SessionId {
    /// The id of the session following this one.
    pub fn next(&self) -> Self {
        SessionId(self.0 + 1)
    }
}

#[derive(Copy, Clone, Debug, Default, Eq, PartialEq, Hash, Ord, PartialOrd, Encode, Decode)]
pub struct SessionPeriod(pub u32);
