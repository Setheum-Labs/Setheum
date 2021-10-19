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

use std::sync::Arc;
use codec::Codec;
use sp_blockchain::HeaderBackend;
use sp_runtime::{generic::BlockId, traits::Block as BlockT};
use jsonrpc_core::Result;
use jsonrpc_derive::rpc;
use sp_api::ProvideRuntimeApi;

use slixon_utils::{ChannelId, rpc::map_rpc_error};
pub use channel_follows_runtime_api::ChannelFollowsApi as ChannelFollowsRuntimeApi;

#[rpc]
pub trait ChannelFollowsApi<BlockHash, AccountId> {
    #[rpc(name = "channelFollows_getChannelIdsFollowedByAccount")]
    fn get_channel_ids_followed_by_account(
        &self,
        at: Option<BlockHash>,
        account: AccountId,
    ) -> Result<Vec<ChannelId>>;

    #[rpc(name = "channelFollows_filterFollowedChannelIds")]
    fn filter_followed_channel_ids(
        &self,
        at: Option<BlockHash>,
        account: AccountId,
        channel_ids: Vec<ChannelId>,
    ) -> Result<Vec<ChannelId>>;
}

pub struct ChannelFollows<C, M> {
    client: Arc<C>,
    _marker: std::marker::PhantomData<M>,
}

impl<C, M> ChannelFollows<C, M> {
    pub fn new(client: Arc<C>) -> Self {
        Self {
            client,
            _marker: Default::default(),
        }
    }
}

impl<C, Block, AccountId> ChannelFollowsApi<<Block as BlockT>::Hash, AccountId>
    for ChannelFollows<C, Block>
where
    Block: BlockT,
    AccountId: Codec,
    C: Send + Sync + 'static + ProvideRuntimeApi<Block> + HeaderBackend<Block>,
    C::Api: ChannelFollowsRuntimeApi<Block, AccountId>,
{
    fn get_channel_ids_followed_by_account(
        &self,
        at: Option<<Block as BlockT>::Hash>,
        account: AccountId,
    ) -> Result<Vec<ChannelId>> {
        let api = self.client.runtime_api();
        let at = BlockId::hash(at.unwrap_or_else(|| self.client.info().best_hash));

        let runtime_api_result = api.get_channel_ids_followed_by_account(&at, account);
        runtime_api_result.map_err(map_rpc_error)
    }

    fn filter_followed_channel_ids(
        &self,
        at: Option<<Block as BlockT>::Hash>,
        account: AccountId,
        channel_ids: Vec<u64>,
    ) -> Result<Vec<ChannelId>> {
        let api = self.client.runtime_api();
        let at = BlockId::hash(at.unwrap_or_else(|| self.client.info().best_hash));

        let runtime_api_result = api.filter_followed_channel_ids(&at, account, channel_ids);
        runtime_api_result.map_err(map_rpc_error)
    }
}
