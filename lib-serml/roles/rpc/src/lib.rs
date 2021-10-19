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
use slixon_permissions::ChannelPermission;

pub use roles_runtime_api::RolesApi as RolesRuntimeApi;

#[rpc]
pub trait RolesApi<BlockHash, AccountId> {
    #[rpc(name = "roles_getChannelPermissionsByAccount")]
    fn get_channel_permissions_by_account(
        &self,
        at: Option<BlockHash>,
        account: AccountId,
        channel_id: ChannelId
    ) -> Result<Vec<ChannelPermission>>;

    #[rpc(name = "roles_getAccountsWithAnyRoleInChannel")]
    fn get_accounts_with_any_role_in_channel(
        &self,
        at: Option<BlockHash>,
        channel_id: ChannelId
    ) -> Result<Vec<AccountId>>;

    #[rpc(name = "roles_getChannelIdsForAccountWithAnyRole")]
    fn get_channel_ids_for_account_with_any_role(
        &self,
        at: Option<BlockHash>,
        account_id: AccountId
    ) -> Result<Vec<ChannelId>>;
}

pub struct Roles<C, M> {
    client: Arc<C>,
    _marker: std::marker::PhantomData<M>,
}

impl<C, M> Roles<C, M> {
    pub fn new(client: Arc<C>) -> Self {
        Self {
            client,
            _marker: Default::default(),
        }
    }
}

impl<C, Block, AccountId> RolesApi<<Block as BlockT>::Hash, AccountId>
    for Roles<C, Block>
where
    Block: BlockT,
    AccountId: Codec,
    C: Send + Sync + 'static + ProvideRuntimeApi<Block> + HeaderBackend<Block>,
    C::Api: RolesRuntimeApi<Block, AccountId>,
{
    fn get_channel_permissions_by_account(
        &self, at:
        Option<<Block as BlockT>::Hash>,
        account: AccountId,
        channel_id: ChannelId
    ) -> Result<Vec<ChannelPermission>> {
        let api = self.client.runtime_api();
        let at = BlockId::hash(at.unwrap_or_else(|| self.client.info().best_hash));

        let runtime_api_result = api.get_channel_permissions_by_account(&at, account, channel_id);
        runtime_api_result.map_err(map_rpc_error)
    }

    fn get_accounts_with_any_role_in_channel(
        &self, at:
        Option<<Block as BlockT>::Hash>,
        channel_id: ChannelId
    ) -> Result<Vec<AccountId>> {
        let api = self.client.runtime_api();
        let at = BlockId::hash(at.unwrap_or_else(|| self.client.info().best_hash));

        let runtime_api_result = api.get_accounts_with_any_role_in_channel(&at, channel_id);
        runtime_api_result.map_err(map_rpc_error)
    }

    fn get_channel_ids_for_account_with_any_role(
        &self, at:
        Option<<Block as BlockT>::Hash>,
        account_id: AccountId
    ) -> Result<Vec<ChannelId>> {
        let api = self.client.runtime_api();
        let at = BlockId::hash(at.unwrap_or_else(|| self.client.info().best_hash));

        let runtime_api_result = api.get_channel_ids_for_account_with_any_role(&at, account_id);
        runtime_api_result.map_err(map_rpc_error)
    }
}
