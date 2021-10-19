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

#![cfg_attr(not(feature = "std"), no_std)]

use codec::Codec;
use sp_std::vec::Vec;

use slixon_channels::rpc::FlatChannel;
use slixon_utils::ChannelId;

sp_api::decl_runtime_apis! {
    pub trait ChannelsApi<AccountId, BlockNumber> where
        AccountId: Codec,
        BlockNumber: Codec
    {
        fn get_next_channel_id() -> ChannelId;

        fn get_channels(start_id: u64, limit: u64) -> Vec<FlatChannel<AccountId, BlockNumber>>;

        fn get_channels_by_ids(channel_ids: Vec<ChannelId>) -> Vec<FlatChannel<AccountId, BlockNumber>>;

        fn get_public_channels(start_id: u64, limit: u64) -> Vec<FlatChannel<AccountId, BlockNumber>>;

        fn get_unlisted_channels(start_id: u64, limit: u64) -> Vec<FlatChannel<AccountId, BlockNumber>>;

        fn get_public_channel_ids_by_owner(owner: AccountId) -> Vec<ChannelId>;

        fn get_unlisted_channel_ids_by_owner(owner: AccountId) -> Vec<ChannelId>;

        fn get_channel_by_handle(handle: Vec<u8>) -> Option<FlatChannel<AccountId, BlockNumber>>;

        fn get_channel_id_by_handle(handle: Vec<u8>) -> Option<ChannelId>;
    }
}
