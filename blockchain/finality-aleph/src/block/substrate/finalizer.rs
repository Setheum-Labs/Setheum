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

use sc_client_api::{Backend, Finalizer as SubstrateFinalizer, HeaderBackend, LockImportRun};
use sp_blockchain::Error as ClientError;
use sp_runtime::traits::Header as SubstrateHeader;

use crate::{
    primitives ::Block,
    block::{
        substrate::{InnerJustification, Justification},
        Finalizer,
    },
    finalization::{AlephFinalizer, BlockFinalizer},
};

impl<BE, C> Finalizer<Justification> for AlephFinalizer<Block, BE, C>
where
    BE: Backend<Block>,
    C: HeaderBackend<Block> + LockImportRun<Block, BE> + SubstrateFinalizer<Block, BE>,
{
    type Error = ClientError;

    fn finalize(&self, justification: Justification) -> Result<(), Self::Error> {
        match justification.inner_justification {
            InnerJustification::AlephJustification(aleph_justification) => self.finalize_block(
                (justification.header.hash(), *justification.header.number()).into(),
                aleph_justification.into(),
            ),
            _ => Err(Self::Error::BadJustification(
                "Trying fo finalize the genesis block using virtual sync justification."
                    .to_string(),
            )),
        }
    }
}
