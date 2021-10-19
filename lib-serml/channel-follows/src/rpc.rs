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

use sp_std::prelude::*;

use slixon_utils::ChannelId;

use crate::{Module, Trait};

impl<T: Trait> Module<T> {
    pub fn get_channel_ids_followed_by_account(account: T::AccountId) -> Vec<ChannelId> {
        Self::channels_followed_by_account(account)
    }

    pub fn filter_followed_channel_ids(account: T::AccountId, channel_ids: Vec<ChannelId>) -> Vec<ChannelId> {
        channel_ids.iter()
            .filter(|channel_id| Self::channel_followed_by_account((&account, channel_id)))
            .cloned().collect()
    }
}