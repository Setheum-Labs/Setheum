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

use std::{default::Default, sync::Arc};

use sc_block_builder::BlockBuilderProvider;
use sc_client_api::HeaderBackend;
use sp_consensus::BlockOrigin;
use sp_core::hash::H256;
use sp_runtime::{traits::Block as BlockT, DigestItem};
use substrate_test_runtime::ExtrinsicBuilder;
use substrate_test_runtime_client::{ClientBlockImportExt, ClientExt};

use crate::{
    primitives ::BlockNumber,
    testing::mocks::{TBlock, THeader, TestClient},
    BlockId,
};
// A helper struct that allows to build blocks without importing/finalizing them right away.
pub struct ClientChainBuilder {
    pub client: Arc<TestClient>,
    // client_builder is used for the purpose of creating blocks only. It is necessary as we cannot create a block
    // in the "main" client without importing it.
    // We keep the invariant that all blocks are first created and imported by `client_builder` and only afterwards
    // can be possibly imported by `client`.
    pub client_builder: Arc<TestClient>,
    pub unique_seed: u32,
}

fn assert_no_blocks_except_genesis(client: &TestClient) {
    assert!(
        client.hash(1).unwrap().is_none(),
        "Client is aware of some blocks beyond genesis"
    );
}

impl ClientChainBuilder {
    pub fn new(client: Arc<TestClient>, client_builder: Arc<TestClient>) -> Self {
        // Below we enforce that both clients are "empty" and agree with each other.
        assert_eq!(client.info(), client_builder.info());
        assert_no_blocks_except_genesis(&client);
        assert_no_blocks_except_genesis(&client_builder);
        ClientChainBuilder {
            client,
            client_builder,
            unique_seed: 0,
        }
    }

    /// Import block in test client
    pub async fn import_block(&mut self, block: TBlock) {
        self.client
            .import(BlockOrigin::Own, block.clone())
            .await
            .unwrap();
    }

    /// Finalize block with given hash without providing justification.
    pub fn finalize_block(&self, hash: &H256) {
        self.client.finalize_block(*hash, None).unwrap();
    }

    pub fn genesis_id(&self) -> BlockId {
        BlockId::new(self.client.info().genesis_hash, 0)
    }

    pub fn genesis_hash(&self) -> H256 {
        self.genesis_id().hash()
    }

    pub fn get_unique_bytes(&mut self) -> Vec<u8> {
        self.unique_seed += 1;
        self.unique_seed.to_be_bytes().to_vec()
    }

    pub async fn build_block_above(&mut self, parent: &H256) -> TBlock {
        let unique_bytes: Vec<u8> = self.get_unique_bytes();
        let mut builder = self
            .client_builder
            .new_block_at(*parent, Default::default(), false)
            .unwrap();
        builder
            .push(
                ExtrinsicBuilder::new_deposit_log_digest_item(DigestItem::Other(unique_bytes))
                    .build()
                    .into(),
            )
            .unwrap();
        let block = builder.build().unwrap().block;

        self.client_builder
            .import(BlockOrigin::Own, block.clone())
            .await
            .unwrap();
        block
    }

    /// Builds a sequence of blocks extending from `hash` of length `len`
    pub async fn build_branch_above(&mut self, parent: &H256, len: usize) -> Vec<TBlock> {
        let mut blocks = Vec::new();
        let mut prev_hash = *parent;
        for _ in 0..len {
            let block = self.build_block_above(&prev_hash).await;
            prev_hash = block.hash();
            blocks.push(block);
        }

        blocks
    }

    /// imports a sequence of blocks, should be in correct order
    pub async fn import_branch(&mut self, blocks: Vec<TBlock>) {
        for block in blocks {
            self.import_block(block.clone()).await;
        }
    }

    /// Builds a sequence of blocks extending from `hash` of length `len` and imports them
    pub async fn build_and_import_branch_above(
        &mut self,
        parent: &H256,
        len: usize,
    ) -> Vec<TBlock> {
        let blocks = self.build_branch_above(parent, len).await;
        self.import_branch(blocks.clone()).await;
        blocks
    }

    pub fn get_header_at(&self, num: BlockNumber) -> THeader {
        self.client_builder
            .header(self.client_builder.hash(num).unwrap().unwrap())
            .unwrap()
            .unwrap()
    }

    /// Builds a sequence of blocks extending from genesis of length `len`
    pub async fn initialize_single_branch(&mut self, len: usize) -> Vec<TBlock> {
        self.build_branch_above(&self.genesis_hash(), len).await
    }

    /// Builds and imports a sequence of blocks extending from genesis of length `len`
    pub async fn initialize_single_branch_and_import(&mut self, len: usize) -> Vec<TBlock> {
        self.build_and_import_branch_above(&self.genesis_hash(), len)
            .await
    }
}
