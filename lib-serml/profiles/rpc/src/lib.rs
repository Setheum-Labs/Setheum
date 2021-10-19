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

use slixon_profiles::rpc::FlatSocialAccount;
use slixon_utils::rpc::map_rpc_error;
pub use profiles_runtime_api::ProfilesApi as ProfilesRuntimeApi;

#[rpc]
pub trait ProfilesApi<BlockHash, AccountId, BlockNumber> {
    #[rpc(name = "profiles_getSocialAccountsByIds")]
    fn get_social_accounts_by_ids(
        &self,
        at: Option<BlockHash>,
        account_ids: Vec<AccountId>,
    ) -> Result<Vec<FlatSocialAccount<AccountId, BlockNumber>>>;
}

pub struct Profiles<C, M> {
    client: Arc<C>,
    _marker: std::marker::PhantomData<M>,
}

impl<C, M> Profiles<C, M> {
    pub fn new(client: Arc<C>) -> Self {
        Self {
            client,
            _marker: Default::default(),
        }
    }
}

impl<C, Block, AccountId, BlockNumber> ProfilesApi<<Block as BlockT>::Hash, AccountId, BlockNumber>
    for Profiles<C, Block>
where
    Block: BlockT,
    AccountId: Codec,
    BlockNumber: Codec,
    C: Send + Sync + 'static + ProvideRuntimeApi<Block> + HeaderBackend<Block>,
    C::Api: ProfilesRuntimeApi<Block, AccountId, BlockNumber>,
{
    fn get_social_accounts_by_ids(&self, at: Option<<Block as BlockT>::Hash>, account_ids: Vec<AccountId>) -> Result<Vec<FlatSocialAccount<AccountId, BlockNumber>>> {
        let api = self.client.runtime_api();
        let at = BlockId::hash(at.unwrap_or_else(|| self.client.info().best_hash));

        let runtime_api_result = api.get_social_accounts_by_ids(&at, account_ids);
        runtime_api_result.map_err(map_rpc_error)
    }
}
