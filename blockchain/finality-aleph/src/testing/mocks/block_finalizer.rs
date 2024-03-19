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

use sp_blockchain::Error;
use sp_runtime::{traits::Block, Justification};

use crate::{
    finalization::BlockFinalizer,
    testing::mocks::{single_action_mock::SingleActionMock, TBlock},
    BlockId,
};
type CallArgs = (BlockId, Justification);

#[derive(Clone, Default)]
pub struct MockedBlockFinalizer {
    mock: SingleActionMock<CallArgs>,
}

impl MockedBlockFinalizer {
    pub fn new() -> Self {
        Self {
            mock: Default::default(),
        }
    }

    pub async fn has_not_been_invoked(&self) -> bool {
        self.mock.has_not_been_invoked().await
    }

    pub async fn has_been_invoked_with(&self, block: TBlock) -> bool {
        self.mock
            .has_been_invoked_with(|(id, _)| {
                block.hash() == id.hash() && block.header.number == id.number()
            })
            .await
    }
}

impl BlockFinalizer for MockedBlockFinalizer {
    fn finalize_block(&self, block_id: BlockId, justification: Justification) -> Result<(), Error> {
        self.mock.invoke_with((block_id, justification));
        Ok(())
    }
}
