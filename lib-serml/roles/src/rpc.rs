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

use crate::{Module, Trait, Role, RoleIdsByUserInChannel};

use frame_support::storage::IterableStorageDoubleMap;
use sp_std::prelude::*;
use sp_std::collections::{ btree_set::BTreeSet };

use slixon_utils::{ChannelId, User};
use slixon_permissions::{ChannelPermission};

impl<T: Trait> Module<T> {
    pub fn get_channel_permissions_by_account(
        account: T::AccountId,
        channel_id: ChannelId
    ) -> Vec<ChannelPermission> {

        Self::role_ids_by_user_in_channel(User::Account(account), channel_id)
            .iter()
            .filter_map(Self::role_by_id)
            .flat_map(|role: Role<T>| role.permissions.into_iter())
            .collect::<BTreeSet<_>>()
            .iter().cloned().collect()
    }

    pub fn get_accounts_with_any_role_in_channel(channel_id: ChannelId) -> Vec<T::AccountId> {

        Self::role_ids_by_channel_id(channel_id)
            .iter()
            .flat_map(Self::users_by_role_id)
            .filter_map(|user| user.maybe_account())
            .collect::<BTreeSet<_>>()
            .iter().cloned().collect()
    }

    pub fn get_channel_ids_for_account_with_any_role(account_id: T::AccountId) -> Vec<ChannelId> {
        let user = &User::Account(account_id);
        let mut channel_ids = Vec::new();

        RoleIdsByUserInChannel::<T>::iter_prefix(user)
            .for_each(|(channel_id, role_ids)| {
                if !role_ids.is_empty() {
                    channel_ids.push(channel_id);
                }
            });

        channel_ids
    }
}