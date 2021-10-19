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

use slixon_channels::rpc::FlatChannel;
use slixon_utils::{ChannelId, rpc::map_rpc_error};
pub use channels_runtime_api::ChannelsApi as ChannelsRuntimeApi;

#[rpc]
pub trait ChannelsApi<BlockHash, AccountId, BlockNumber> {
    #[rpc(name = "channels_getChannels")]
    fn get_channels(
        &self,
        at: Option<BlockHash>,
        start_id: u64,
        limit: u64,
    ) -> Result<Vec<FlatChannel<AccountId, BlockNumber>>>;

    #[rpc(name = "channels_getChannelsByIds")]
    fn get_channels_by_ids(
        &self,
        at: Option<BlockHash>,
        channel_ids: Vec<ChannelId>,
    ) -> Result<Vec<FlatChannel<AccountId, BlockNumber>>>;

    #[rpc(name = "channels_getPublicChannels")]
    fn get_public_channels(
        &self,
        at: Option<BlockHash>,
        start_id: u64,
        limit: u64,
    ) -> Result<Vec<FlatChannel<AccountId, BlockNumber>>>;

    #[rpc(name = "channels_getUnlistedChannels")]
    fn get_unlisted_channels(
        &self,
        at: Option<BlockHash>,
        start_id: u64,
        limit: u64,
    ) -> Result<Vec<FlatChannel<AccountId, BlockNumber>>>;

    #[rpc(name = "channels_getChannelIdByHandle")]
    fn get_channel_id_by_handle(
        &self,
        at: Option<BlockHash>,
        handle: Vec<u8>,
    ) -> Result<Option<ChannelId>>;

    #[rpc(name = "channels_getChannelByHandle")]
    fn get_channel_by_handle(
        &self,
        at: Option<BlockHash>,
        handle: Vec<u8>,
    ) -> Result<Option<FlatChannel<AccountId, BlockNumber>>>;

    #[rpc(name = "channels_getPublicChannelIdsByOwner")]
    fn get_public_channel_ids_by_owner(
        &self,
        at: Option<BlockHash>,
        owner: AccountId,
    ) -> Result<Vec<ChannelId>>;

    #[rpc(name = "channels_getUnlistedChannelIdsByOwner")]
    fn get_unlisted_channel_ids_by_owner(
        &self,
        at: Option<BlockHash>,
        owner: AccountId,
    ) -> Result<Vec<ChannelId>>;

    #[rpc(name = "channels_nextChannelId")]
    fn get_next_channel_id(&self, at: Option<BlockHash>) -> Result<ChannelId>;
}

pub struct Channels<C, M> {
    client: Arc<C>,
    _marker: std::marker::PhantomData<M>,
}

impl<C, M> Channels<C, M> {
    pub fn new(client: Arc<C>) -> Self {
        Self {
            client,
            _marker: Default::default(),
        }
    }
}

impl<C, Block, AccountId, BlockNumber> ChannelsApi<<Block as BlockT>::Hash, AccountId, BlockNumber>
    for Channels<C, Block>
where
    Block: BlockT,
    AccountId: Codec,
    BlockNumber: Codec,
    C: Send + Sync + 'static + ProvideRuntimeApi<Block> + HeaderBackend<Block>,
    C::Api: ChannelsRuntimeApi<Block, AccountId, BlockNumber>,
{
    fn get_channels(
        &self,
        at: Option<<Block as BlockT>::Hash>,
        start_id: u64,
        limit: u64,
    ) -> Result<Vec<FlatChannel<AccountId, BlockNumber>>> {
        let api = self.client.runtime_api();
        let at = BlockId::hash(at.unwrap_or_else(|| self.client.info().best_hash));

        let runtime_api_result = api.get_channels(&at, start_id, limit);
        runtime_api_result.map_err(map_rpc_error)
    }

    fn get_channels_by_ids(
        &self,
        at: Option<<Block as BlockT>::Hash>,
        channel_ids: Vec<ChannelId>,
    ) -> Result<Vec<FlatChannel<AccountId, BlockNumber>>> {
        let api = self.client.runtime_api();
        let at = BlockId::hash(at.unwrap_or_else(|| self.client.info().best_hash));

        let runtime_api_result = api.get_channels_by_ids(&at, channel_ids);
        runtime_api_result.map_err(map_rpc_error)
    }

    fn get_public_channels(
        &self,
        at: Option<<Block as BlockT>::Hash>,
        start_id: u64,
        limit: u64,
    ) -> Result<Vec<FlatChannel<AccountId, BlockNumber>>> {
        let api = self.client.runtime_api();
        let at = BlockId::hash(at.unwrap_or_else(|| self.client.info().best_hash));

        let runtime_api_result = api.get_public_channels(&at, start_id, limit);
        runtime_api_result.map_err(map_rpc_error)
    }

    fn get_unlisted_channels(
        &self,
        at: Option<<Block as BlockT>::Hash>,
        start_id: u64,
        limit: u64,
    ) -> Result<Vec<FlatChannel<AccountId, BlockNumber>>> {
        let api = self.client.runtime_api();
        let at = BlockId::hash(at.unwrap_or_else(|| self.client.info().best_hash));

        let runtime_api_result = api.get_unlisted_channels(&at, start_id, limit);
        runtime_api_result.map_err(map_rpc_error)
    }

    fn get_channel_id_by_handle(&self, at: Option<<Block as BlockT>::Hash>, handle: Vec<u8>) -> Result<Option<u64>> {
        let api = self.client.runtime_api();
        let at = BlockId::hash(at.unwrap_or_else(|| self.client.info().best_hash));

        let runtime_api_result = api.get_channel_id_by_handle(&at, handle);
        runtime_api_result.map_err(map_rpc_error)
    }

    fn get_channel_by_handle(
        &self,
        at: Option<<Block as BlockT>::Hash>,
        handle: Vec<u8>,
    ) -> Result<Option<FlatChannel<AccountId, BlockNumber>>> {
        let api = self.client.runtime_api();
        let at = BlockId::hash(at.unwrap_or_else(|| self.client.info().best_hash));

        let runtime_api_result = api.get_channel_by_handle(&at, handle);
        runtime_api_result.map_err(map_rpc_error)
    }

    fn get_public_channel_ids_by_owner(&self, at: Option<<Block as BlockT>::Hash>, owner: AccountId) -> Result<Vec<u64>> {
        let api = self.client.runtime_api();
        let at = BlockId::hash(at.unwrap_or_else(|| self.client.info().best_hash));

        let runtime_api_result = api.get_public_channel_ids_by_owner(&at, owner);
        runtime_api_result.map_err(map_rpc_error)
    }

    fn get_unlisted_channel_ids_by_owner(&self, at: Option<<Block as BlockT>::Hash>, owner: AccountId) -> Result<Vec<u64>> {
        let api = self.client.runtime_api();
        let at = BlockId::hash(at.unwrap_or_else(|| self.client.info().best_hash));

        let runtime_api_result = api.get_unlisted_channel_ids_by_owner(&at, owner);
        runtime_api_result.map_err(map_rpc_error)
    }

    fn get_next_channel_id(&self, at: Option<<Block as BlockT>::Hash>) -> Result<u64> {
        let api = self.client.runtime_api();
        let at = BlockId::hash(at.unwrap_or_else(|| self.client.info().best_hash));

        let runtime_api_result = api.get_next_channel_id(&at);
        runtime_api_result.map_err(map_rpc_error)
    }
}
