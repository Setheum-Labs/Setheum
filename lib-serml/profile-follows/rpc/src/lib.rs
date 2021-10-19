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

use slixon_utils::rpc::map_rpc_error;
pub use profile_follows_runtime_api::ProfileFollowsApi as ProfileFollowsRuntimeApi;

#[rpc]
pub trait ProfileFollowsApi<BlockHash, AccountId> {
    #[rpc(name = "profileFollows_filterFollowedAccounts")]
    fn filter_followed_accounts(
        &self,
        at: Option<BlockHash>,
        account: AccountId,
        maybe_following: Vec<AccountId>,
    ) -> Result<Vec<AccountId>>;
}

pub struct ProfileFollows<C, M> {
    client: Arc<C>,
    _marker: std::marker::PhantomData<M>,
}

impl<C, M> ProfileFollows<C, M> {
    pub fn new(client: Arc<C>) -> Self {
        Self {
            client,
            _marker: Default::default(),
        }
    }
}

impl<C, Block, AccountId> ProfileFollowsApi<<Block as BlockT>::Hash, AccountId>
    for ProfileFollows<C, Block>
where
    Block: BlockT,
    AccountId: Codec,
    C: Send + Sync + 'static + ProvideRuntimeApi<Block> + HeaderBackend<Block>,
    C::Api: ProfileFollowsRuntimeApi<Block, AccountId>,
{
    fn filter_followed_accounts(
        &self, at:
        Option<<Block as BlockT>::Hash>,
        account: AccountId,
        maybe_following: Vec<AccountId>,
    ) -> Result<Vec<AccountId>> {
        let api = self.client.runtime_api();
        let at = BlockId::hash(at.unwrap_or_else(|| self.client.info().best_hash));

        let runtime_api_result = api.filter_followed_accounts(&at, account, maybe_following);
        runtime_api_result.map_err(map_rpc_error)
    }
}
