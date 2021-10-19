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

use slixon_posts::rpc::{FlatPost, FlatPostKind, RepliesByPostId};
use slixon_utils::{PostId, ChannelId, rpc::map_rpc_error};
pub use posts_runtime_api::PostsApi as PostsRuntimeApi;

#[rpc]
pub trait PostsApi<BlockHash, AccountId, BlockNumber> {
    #[rpc(name = "posts_getPostsByIds")]
    fn get_posts_by_ids(
        &self,
        at: Option<BlockHash>,
        post_ids: Vec<PostId>,
        offset: u64,
        limit: u16,
    ) -> Result<Vec<FlatPost<AccountId, BlockNumber>>>;

    #[rpc(name = "posts_getPublicPosts")]
    fn get_public_posts(
        &self,
        at: Option<BlockHash>,
        kind_filter: Vec<FlatPostKind>,
        start_id: u64,
        limit: u16
    ) -> Result<Vec<FlatPost<AccountId, BlockNumber>>>;

    #[rpc(name = "posts_getPublicPostsByChannelId")]
    fn get_public_posts_by_channel_id(
        &self,
        at: Option<BlockHash>,
        channel_id: ChannelId,
        offset: u64,
        limit: u16,
    ) -> Result<Vec<FlatPost<AccountId, BlockNumber>>>;

    #[rpc(name = "posts_getUnlistedPostsByChannelId")]
    fn get_unlisted_posts_by_channel_id(
        &self,
        at: Option<BlockHash>,
        channel_id: ChannelId,
        offset: u64,
        limit: u16,
    ) -> Result<Vec<FlatPost<AccountId, BlockNumber>>>;

    #[rpc(name = "posts_getReplyIdsByParentId")]
    fn get_reply_ids_by_parent_id(
        &self,
        at: Option<BlockHash>,
        post_id: PostId,
    ) -> Result<Vec<PostId>>;

    #[rpc(name = "posts_getReplyIdsByParentIds")]
    fn get_reply_ids_by_parent_ids(
        &self,
        at: Option<BlockHash>,
        post_ids: Vec<PostId>,
    ) -> Result<BTreeMap<PostId, Vec<PostId>>>;

    #[rpc(name = "posts_getRepliesByParentId")]
    fn get_replies_by_parent_id(
        &self,
        at: Option<BlockHash>,
        parent_id: PostId,
        offset: u64,
        limit: u16,
    ) -> Result<Vec<FlatPost<AccountId, BlockNumber>>>;

    #[rpc(name = "posts_getRepliesByParentIds")]
    fn get_replies_by_parent_ids(
        &self,
        at: Option<BlockHash>,
        parent_ids: Vec<PostId>,
        offset: u64,
        limit: u16,
    ) -> Result<RepliesByPostId<AccountId, BlockNumber>>;

    #[rpc(name = "posts_getUnlistedPostIdsByChannelId")]
    fn get_unlisted_post_ids_by_channel_id(
        &self,
        at: Option<BlockHash>,
        channel_id: ChannelId,
    ) -> Result<Vec<PostId>>;

    #[rpc(name = "posts_getPublicPostIdsByChannelId")]
    fn get_public_post_ids_by_channel_id(
        &self,
        at: Option<BlockHash>,
        channel_id: ChannelId,
    ) -> Result<Vec<PostId>>;

    #[rpc(name = "posts_nextPostId")]
    fn get_next_post_id(&self, at: Option<BlockHash>) -> Result<PostId>;

    #[rpc(name = "posts_getFeed")]
    fn get_feed(
        &self,
        at: Option<BlockHash>,
        account: AccountId,
        offset: u64,
        limit: u16,
    ) -> Result<Vec<FlatPost<AccountId, BlockNumber>>>;
}

pub struct Posts<C, M> {
    client: Arc<C>,
    _marker: std::marker::PhantomData<M>,
}

impl<C, M> Posts<C, M> {
    pub fn new(client: Arc<C>) -> Self {
        Self {
            client,
            _marker: Default::default(),
        }
    }
}

impl<C, Block, AccountId, BlockNumber> PostsApi<<Block as BlockT>::Hash, AccountId, BlockNumber>
    for Posts<C, Block>
where
    Block: BlockT,
    AccountId: Codec,
    BlockNumber: Codec,
    C: Send + Sync + 'static + ProvideRuntimeApi<Block> + HeaderBackend<Block>,
    C::Api: PostsRuntimeApi<Block, AccountId, BlockNumber>,
{
    fn get_posts_by_ids(
        &self,
        at: Option<<Block as BlockT>::Hash>,
        post_ids: Vec<PostId>,
        offset: u64,
        limit: u16,
    ) -> Result<Vec<FlatPost<AccountId, BlockNumber>>> {
        let api = self.client.runtime_api();
        let at = BlockId::hash(at.unwrap_or_else(|| self.client.info().best_hash));

        let runtime_api_result = api.get_posts_by_ids(&at, post_ids, offset, limit);
        runtime_api_result.map_err(map_rpc_error)
    }

    fn get_public_posts(
        &self,
        at: Option<<Block as BlockT>::Hash>,
        kind_filter: Vec<FlatPostKind>,
        start_id: u64,
        limit: u16
    ) -> Result<Vec<FlatPost<AccountId, BlockNumber>>> {
        let api = self.client.runtime_api();
        let at = BlockId::hash(at.unwrap_or_else(|| self.client.info().best_hash));

        let runtime_api_result = api.get_public_posts(&at, kind_filter, start_id, limit);
        runtime_api_result.map_err(map_rpc_error)
    }

    fn get_public_posts_by_channel_id(
        &self,
        at: Option<<Block as BlockT>::Hash>,
        channel_id: u64,
        offset: u64,
        limit: u16,
    ) -> Result<Vec<FlatPost<AccountId, BlockNumber>>> {
        let api = self.client.runtime_api();
        let at = BlockId::hash(at.unwrap_or_else(|| self.client.info().best_hash));

        let runtime_api_result = api.get_public_posts_by_channel_id(&at, channel_id, offset, limit);
        runtime_api_result.map_err(map_rpc_error)
    }

    fn get_unlisted_posts_by_channel_id(
        &self,
        at: Option<<Block as BlockT>::Hash>,
        channel_id: u64,
        offset: u64,
        limit: u16,
    ) -> Result<Vec<FlatPost<AccountId, BlockNumber>>> {
        let api = self.client.runtime_api();
        let at = BlockId::hash(at.unwrap_or_else(|| self.client.info().best_hash));

        let runtime_api_result = api.get_unlisted_posts_by_channel_id(&at, channel_id, offset, limit);
        runtime_api_result.map_err(map_rpc_error)
    }

    fn get_reply_ids_by_parent_id(&self, at: Option<<Block as BlockT>::Hash>, parent_id: PostId) -> Result<Vec<PostId>> {
        let api = self.client.runtime_api();
        let at = BlockId::hash(at.unwrap_or_else(|| self.client.info().best_hash));

        let runtime_api_result = api.get_reply_ids_by_parent_id(&at, parent_id);
        runtime_api_result.map_err(map_rpc_error)
    }

    fn get_reply_ids_by_parent_ids(&self, at: Option<<Block as BlockT>::Hash>, parent_ids: Vec<PostId>) -> Result<BTreeMap<PostId, Vec<PostId>>> {
        let api = self.client.runtime_api();
        let at = BlockId::hash(at.unwrap_or_else(|| self.client.info().best_hash));

        let runtime_api_result = api.get_reply_ids_by_parent_ids(&at, parent_ids);
        runtime_api_result.map_err(map_rpc_error)
    }

    fn get_replies_by_parent_id(
        &self,
        at: Option<<Block as BlockT>::Hash>,
        parent_id: PostId,
        offset: u64,
        limit: u16
    ) -> Result<Vec<FlatPost<AccountId, BlockNumber>>> {
        let api = self.client.runtime_api();
        let at = BlockId::hash(at.unwrap_or_else(|| self.client.info().best_hash));

        let runtime_api_result = api.get_replies_by_parent_id(&at, parent_id, offset, limit);
        runtime_api_result.map_err(map_rpc_error)
    }

    fn get_replies_by_parent_ids(
        &self,
        at: Option<<Block as BlockT>::Hash>,
        parent_ids: Vec<PostId>,
        offset: u64,
        limit: u16
    ) -> Result<RepliesByPostId<AccountId, BlockNumber>> {
        let api = self.client.runtime_api();
        let at = BlockId::hash(at.unwrap_or_else(|| self.client.info().best_hash));

        let runtime_api_result = api.get_replies_by_parent_ids(&at, parent_ids, offset, limit);
        runtime_api_result.map_err(map_rpc_error)
    }

    fn get_unlisted_post_ids_by_channel_id(&self, at: Option<<Block as BlockT>::Hash>, channel_id: u64) -> Result<Vec<u64>> {
        let api = self.client.runtime_api();
        let at = BlockId::hash(at.unwrap_or_else(|| self.client.info().best_hash));

        let runtime_api_result = api.get_unlisted_post_ids_by_channel_id(&at, channel_id);
        runtime_api_result.map_err(map_rpc_error)
    }

    fn get_public_post_ids_by_channel_id(&self, at: Option<<Block as BlockT>::Hash>, channel_id: u64) -> Result<Vec<u64>> {
        let api = self.client.runtime_api();
        let at = BlockId::hash(at.unwrap_or_else(|| self.client.info().best_hash));

        let runtime_api_result = api.get_public_post_ids_by_channel_id(&at, channel_id);
        runtime_api_result.map_err(map_rpc_error)
    }

    fn get_next_post_id(&self, at: Option<<Block as BlockT>::Hash>) -> Result<u64> {
        let api = self.client.runtime_api();
        let at = BlockId::hash(at.unwrap_or_else(|| self.client.info().best_hash));

        let runtime_api_result = api.get_next_post_id(&at);
        runtime_api_result.map_err(map_rpc_error)
    }

    fn get_feed(
        &self,
        at: Option<<Block as BlockT>::Hash>,
        account: AccountId,
        offset: u64,
        limit: u16
    ) -> Result<Vec<FlatPost<AccountId, BlockNumber>>> {
        let api = self.client.runtime_api();
        let at = BlockId::hash(at.unwrap_or_else(|| self.client.info().best_hash));

        let runtime_api_result = api.get_feed(&at, account, offset, limit);
        runtime_api_result.map_err(map_rpc_error)
    }
}
