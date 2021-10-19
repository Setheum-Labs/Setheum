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

use codec::{Decode, Encode};
#[cfg(feature = "std")]
use serde::{Deserialize, Serialize};
use sp_std::prelude::*;

use slixon_utils::{bool_to_option, ChannelId, rpc::{FlatContent, FlatWhoAndWhen, ShouldSkip}};

use crate::{Module, Channel, Trait, FIRST_SPACE_ID};

#[derive(Eq, PartialEq, Encode, Decode, Default)]
#[cfg_attr(feature = "std", derive(Debug, Serialize, Deserialize))]
#[cfg_attr(feature = "std", serde(rename_all = "camelCase"))]
pub struct FlatChannel<AccountId, BlockNumber> {
    pub id: ChannelId,

    #[cfg_attr(feature = "std", serde(flatten))]
    pub who_and_when: FlatWhoAndWhen<AccountId, BlockNumber>,

    pub owner_id: AccountId,

    #[cfg_attr(feature = "std", serde(skip_serializing_if = "ShouldSkip::should_skip"))]
    pub parent_id: Option<ChannelId>,

    #[cfg_attr(feature = "std", serde(skip_serializing_if = "ShouldSkip::should_skip", serialize_with = "bytes_to_string"))]
    pub handle: Option<Vec<u8>>,

    #[cfg_attr(feature = "std", serde(flatten))]
    pub content: FlatContent,

    #[cfg_attr(feature = "std", serde(skip_serializing_if = "ShouldSkip::should_skip"))]
    pub is_hidden: Option<bool>,

    pub posts_count: u32,
    pub hidden_posts_count: u32,
    pub visible_posts_count: u32,
    pub followers_count: u32,

    pub score: i32,
}

#[cfg(feature = "std")]
fn bytes_to_string<S>(field: &Option<Vec<u8>>, serializer: S) -> Result<S::Ok, S::Error> where S: serde::Serializer {
    let field_unwrapped = field.clone().unwrap_or_default();
    // If Bytes slice is invalid, then empty string will be returned
    serializer.serialize_str(
        std::str::from_utf8(&field_unwrapped).unwrap_or_default()
    )
}

impl<T: Trait> From<Channel<T>> for FlatChannel<T::AccountId, T::BlockNumber> {
    fn from(from: Channel<T>) -> Self {
        let Channel {
            id, created, updated, owner,
            parent_id, handle, content, hidden, posts_count,
            hidden_posts_count, followers_count, score, ..
        } = from;

        Self {
            id,
            who_and_when: (created, updated).into(),
            owner_id: owner,
            parent_id,
            handle,
            content: content.into(),
            is_hidden: bool_to_option(hidden),
            posts_count,
            hidden_posts_count,
            visible_posts_count: posts_count.saturating_sub(hidden_posts_count),
            followers_count,
            score,
        }
    }
}

impl<T: Trait> Module<T> {
    pub fn get_channels_by_ids(channel_ids: Vec<ChannelId>) -> Vec<FlatChannel<T::AccountId, T::BlockNumber>> {
        channel_ids.iter()
            .filter_map(|id| Self::require_channel(*id).ok())
            .map(|channel| channel.into())
            .collect()
    }

    fn get_channels_slice<F: FnMut(&Channel<T>) -> bool>(
        start_id: u64,
        limit: u64,
        mut filter: F,
    ) -> Vec<FlatChannel<T::AccountId, T::BlockNumber>> {
        let mut channel_id = start_id;
        let mut channels = Vec::new();

        while channels.len() < limit as usize && channel_id >= FIRST_SPACE_ID {
            if let Ok(channel) = Self::require_channel(channel_id) {
                if filter(&channel) {
                    channels.push(channel.into());
                }
            }
            channel_id = channel_id.saturating_sub(1);
        }

        channels
    }

    pub fn get_channels(start_id: u64, limit: u64) -> Vec<FlatChannel<T::AccountId, T::BlockNumber>> {
        Self::get_channels_slice(start_id, limit, |_| true)
    }

    pub fn get_public_channels(start_id: u64, limit: u64) -> Vec<FlatChannel<T::AccountId, T::BlockNumber>> {
        Self::get_channels_slice(start_id, limit, |channel| channel.is_public())
    }

    pub fn get_unlisted_channels(start_id: u64, limit: u64) -> Vec<FlatChannel<T::AccountId, T::BlockNumber>> {
        Self::get_channels_slice(start_id, limit, |channel| channel.is_unlisted())
    }

    pub fn get_channel_id_by_handle(handle: Vec<u8>) -> Option<ChannelId> {
        Self::channel_id_by_handle(handle)
    }

    pub fn get_channel_by_handle(handle: Vec<u8>) -> Option<FlatChannel<T::AccountId, T::BlockNumber>> {
        Self::channel_id_by_handle(handle)
            .and_then(|channel_id| Self::require_channel(channel_id).ok())
            .map(|channel| channel.into())
    }

    fn get_channel_ids_by_owner<F: FnMut(&Channel<T>) -> bool>(owner: T::AccountId, mut compare_fn: F) -> Vec<ChannelId> {
        Self::channel_ids_by_owner(owner)
            .iter()
            .filter_map(|channel_id| Self::require_channel(*channel_id).ok())
            .filter(|channel| compare_fn(channel))
            .map(|channel| channel.id)
            .collect()
    }

    pub fn get_public_channel_ids_by_owner(owner: T::AccountId) -> Vec<ChannelId> {
        Self::get_channel_ids_by_owner(owner, |channel| !channel.hidden)
    }

    pub fn get_unlisted_channel_ids_by_owner(owner: T::AccountId) -> Vec<ChannelId> {
        Self::get_channel_ids_by_owner(owner, |channel| channel.hidden)
    }

    pub fn get_next_channel_id() -> ChannelId {
        Self::next_channel_id()
    }
}