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

use core::result::Result;
use std::{marker::PhantomData, sync::Arc};

use log::{debug, warn};
use sc_client_api::{Backend, Finalizer, HeaderBackend, LockImportRun};
use sp_blockchain::Error;
use sp_runtime::{
    traits::{Block, Header},
    Justification,
};

use crate::{
    aleph_primitives::{BlockHash, BlockNumber},
    metrics::{AllBlockMetrics, Checkpoint},
    BlockId,
};

pub trait BlockFinalizer {
    fn finalize_block(&self, block: BlockId, justification: Justification) -> Result<(), Error>;
}

pub struct AlephFinalizer<B, BE, C>
where
    B: Block,
    BE: Backend<B>,
    C: HeaderBackend<B> + LockImportRun<B, BE> + Finalizer<B, BE>,
{
    client: Arc<C>,
    metrics: AllBlockMetrics,
    phantom: PhantomData<(B, BE)>,
}

impl<B, BE, C> AlephFinalizer<B, BE, C>
where
    B: Block,
    BE: Backend<B>,
    C: HeaderBackend<B> + LockImportRun<B, BE> + Finalizer<B, BE>,
{
    pub(crate) fn new(client: Arc<C>, metrics: AllBlockMetrics) -> Self {
        AlephFinalizer {
            client,
            metrics,
            phantom: PhantomData,
        }
    }
}

impl<B, BE, C> BlockFinalizer for AlephFinalizer<B, BE, C>
where
    B: Block<Hash = BlockHash>,
    B::Header: Header<Number = BlockNumber>,
    BE: Backend<B>,
    C: HeaderBackend<B> + LockImportRun<B, BE> + Finalizer<B, BE>,
{
    fn finalize_block(&self, block: BlockId, justification: Justification) -> Result<(), Error> {
        let number = block.number();
        let hash = block.hash();

        let status = self.client.info();
        if status.finalized_number >= number {
            warn!(target: "aleph-finality", "trying to finalize a block with hash {} and number {}
               that is not greater than already finalized {}", hash, number, status.finalized_number);
        }

        debug!(target: "aleph-finality", "Finalizing block with hash {:?} and number {:?}. Previous best: #{:?}.", hash, number, status.finalized_number);

        let update_res = self.client.lock_import_and_run(|import_op| {
            // NOTE: all other finalization logic should come here, inside the lock
            self.client
                .apply_finality(import_op, hash, Some(justification), true)
        });

        let status = self.client.info();
        match &update_res {
            Ok(_) => {
                debug!(target: "aleph-finality", "Successfully finalized block with hash {:?} and number {:?}. Current best: #{:?}.", hash, number, status.best_number);
                self.metrics
                    .report_block(block, Checkpoint::Finalized, None);
            }
            Err(_) => {
                debug!(target: "aleph-finality", "Failed to finalize block with hash {:?} and number {:?}. Current best: #{:?}.", hash, number, status.best_number)
            }
        }

        update_res
    }
}
