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

use std::fmt::{Display, Error as FmtError, Formatter};

pub use acceptance_policy::AcceptancePolicy;
pub use block_finalizer::MockedBlockFinalizer;
pub use client::{Backend, TestClient, TestClientBuilder, TestClientBuilderExt};
pub use proposal::{
    aleph_data_from_blocks, aleph_data_from_headers, unvalidated_proposal_from_headers,
};
use sp_core::H256;

use crate::{
    aleph_primitives::{Block, Header},
    block::{EquivocationProof, HeaderVerifier, VerifiedHeader},
};

pub type TBlock = Block;
pub type THeader = Header;
pub type THash = H256;

#[derive(Clone)]
pub struct TestVerifier;

pub struct TestEquivocationProof;

impl Display for TestEquivocationProof {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), FmtError> {
        write!(f, "this should never get created")
    }
}

impl EquivocationProof for TestEquivocationProof {
    fn are_we_equivocating(&self) -> bool {
        false
    }
}

#[derive(Debug)]
pub struct TestVerificationError;

impl Display for TestVerificationError {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), FmtError> {
        write!(f, "this should never get created")
    }
}

impl HeaderVerifier<THeader> for TestVerifier {
    type EquivocationProof = TestEquivocationProof;
    type Error = TestVerificationError;

    fn verify_header(
        &mut self,
        header: THeader,
        _just_created: bool,
    ) -> Result<VerifiedHeader<THeader, Self::EquivocationProof>, Self::Error> {
        Ok(VerifiedHeader {
            header,
            maybe_equivocation_proof: None,
        })
    }

    fn own_block(&self, _header: &THeader) -> bool {
        false
    }
}

mod acceptance_policy;
mod block_finalizer;
mod client;
mod proposal;
mod single_action_mock;
