// This file is part of Setheum.

// Copyright (C) 2019-2021 Setheum Labs.
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

use std::{sync::Arc, collections::BTreeMap};
use codec::Codec;
use sp_blockchain::HeaderBackend;
use sp_runtime::{generic::BlockId, traits::Block as BlockT};
use jsonrpc_core::Result;
use jsonrpc_derive::rpc;
use sp_api::ProvideRuntimeApi;

use slixon_reactions::{ReactionId, ReactionKind, rpc::FlatReaction};
use slixon_utils::{PostId, rpc::map_rpc_error};
pub use reactions_runtime_api::ReactionsApi as ReactionsRuntimeApi;

#[rpc]
pub trait ReactionsApi<BlockHash, AccountId, BlockNumber> {
    #[rpc(name = "reactions_getReactionsByIds")]
    fn get_reactions_by_ids(
        &self,
        at: Option<BlockHash>,
        reaction_ids: Vec<ReactionId>,
    ) -> Result<Vec<FlatReaction<AccountId, BlockNumber>>>;

    #[rpc(name = "reactions_getReactionsByPostId")]
    fn get_reactions_by_post_id(
        &self,
        at: Option<BlockHash>,
        post_id: PostId,
        limit: u64,
        offset: u64,
    ) -> Result<Vec<FlatReaction<AccountId, BlockNumber>>>;

    #[rpc(name = "reactions_getReactionKindsByPostIdsAndReactor")]
    fn get_reaction_kinds_by_post_ids_and_reactor(
        &self,
        at: Option<BlockHash>,
        post_ids: Vec<PostId>,
        reactor: AccountId,
    ) -> Result<BTreeMap<PostId, ReactionKind>>;
}

pub struct Reactions<C, M> {
    client: Arc<C>,
    _marker: std::marker::PhantomData<M>,
}

impl<C, M> Reactions<C, M> {
    pub fn new(client: Arc<C>) -> Self {
        Self {
            client,
            _marker: Default::default(),
        }
    }
}

impl<C, Block, AccountId, BlockNumber> ReactionsApi<<Block as BlockT>::Hash, AccountId, BlockNumber>
    for Reactions<C, Block>
where
    Block: BlockT,
    AccountId: Codec,
    BlockNumber: Codec,
    C: Send + Sync + 'static + ProvideRuntimeApi<Block> + HeaderBackend<Block>,
    C::Api: ReactionsRuntimeApi<Block, AccountId, BlockNumber>,
{
    fn get_reactions_by_ids(
        &self,
        at: Option<<Block as BlockT>::Hash>,
        reaction_ids: Vec<u64>,
    ) -> Result<Vec<FlatReaction<AccountId, BlockNumber>>> {
        let api = self.client.runtime_api();
        let at = BlockId::hash(at.unwrap_or_else(|| self.client.info().best_hash));

        let runtime_api_result = api.get_reactions_by_ids(&at, reaction_ids);
        runtime_api_result.map_err(map_rpc_error)
    }

    fn get_reactions_by_post_id(
        &self,
        at: Option<<Block as BlockT>::Hash>,
        post_id: u64,
        limit: u64,
        offset: u64,
    ) -> Result<Vec<FlatReaction<AccountId, BlockNumber>>> {
        let api = self.client.runtime_api();
        let at = BlockId::hash(at.unwrap_or_else(|| self.client.info().best_hash));

        let runtime_api_result = api.get_reactions_by_post_id(&at, post_id, limit, offset);
        runtime_api_result.map_err(map_rpc_error)
    }

    fn get_reaction_kinds_by_post_ids_and_reactor(
        &self,
        at: Option<<Block as BlockT>::Hash>,
        post_ids: Vec<PostId>,
        reactor: AccountId,
    ) -> Result<BTreeMap<PostId, ReactionKind>> {
        let api = self.client.runtime_api();
        let at = BlockId::hash(at.unwrap_or_else(|| self.client.info().best_hash));

        let runtime_api_result = api.get_reaction_kinds_by_post_ids_and_reactor(&at, post_ids, reactor);
        runtime_api_result.map_err(map_rpc_error)
    }
}
