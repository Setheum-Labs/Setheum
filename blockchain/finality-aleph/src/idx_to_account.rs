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

use std::sync::Arc;

use primitives::{AccountId, AlephSessionApi, AuraId, BlockHash, BlockNumber};
use sc_client_api::Backend;
use sp_consensus_aura::AuraApi;
use sp_runtime::traits::{Block, Header};

use crate::{
    abft::NodeIndex,
    runtime_api::RuntimeApi,
    session::{SessionBoundaryInfo, SessionId},
    session_map::{AuthorityProvider, AuthorityProviderImpl},
    ClientForAleph,
};

pub trait ValidatorIndexToAccountIdConverter {
    fn account(&self, session: SessionId, validator_index: NodeIndex) -> Option<AccountId>;
}

pub struct ValidatorIndexToAccountIdConverterImpl<C, B, BE, RA>
where
    C: ClientForAleph<B, BE> + Send + Sync + 'static,
    C::Api: crate::aleph_primitives::AlephSessionApi<B> + AuraApi<B, AuraId>,
    B: Block<Hash = BlockHash>,
    BE: Backend<B> + 'static,
    RA: RuntimeApi,
{
    client: Arc<C>,
    session_boundary_info: SessionBoundaryInfo,
    authority_provider: AuthorityProviderImpl<C, B, BE, RA>,
}

impl<C, B, BE, RA> ValidatorIndexToAccountIdConverterImpl<C, B, BE, RA>
where
    C: ClientForAleph<B, BE> + Send + Sync + 'static,
    C::Api: crate::aleph_primitives::AlephSessionApi<B> + AuraApi<B, AuraId>,
    B: Block<Hash = BlockHash>,
    B::Header: Header<Number = BlockNumber>,
    BE: Backend<B> + 'static,
    RA: RuntimeApi,
{
    pub fn new(client: Arc<C>, session_boundary_info: SessionBoundaryInfo, api: RA) -> Self {
        Self {
            client: client.clone(),
            session_boundary_info,
            authority_provider: AuthorityProviderImpl::new(client, api),
        }
    }
}

impl<C, B, BE, RA> ValidatorIndexToAccountIdConverter
    for ValidatorIndexToAccountIdConverterImpl<C, B, BE, RA>
where
    C: ClientForAleph<B, BE> + Send + Sync + 'static,
    C::Api: crate::aleph_primitives::AlephSessionApi<B> + AuraApi<B, AuraId>,
    B: Block<Hash = BlockHash>,
    B::Header: Header<Number = BlockNumber>,
    BE: Backend<B> + 'static,
    RA: RuntimeApi,
{
    fn account(&self, session: SessionId, validator_index: NodeIndex) -> Option<AccountId> {
        let block_number = self
            .session_boundary_info
            .boundaries_for_session(session)
            .first_block();
        let block_hash = self.client.block_hash(block_number).ok()??;

        let authority_data = self.authority_provider.authority_data(block_number)?;
        let aleph_key = authority_data.authorities()[validator_index.0].clone();
        self.client
            .runtime_api()
            .key_owner(block_hash, aleph_key)
            .ok()?
    }
}

#[cfg(test)]
pub struct MockConverter;

#[cfg(test)]
impl ValidatorIndexToAccountIdConverter for MockConverter {
    fn account(&self, _: SessionId, _: NodeIndex) -> Option<AccountId> {
        None
    }
}
