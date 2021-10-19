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

use utils::{ChannelId, Content};

pub trait IsAccountBlocked<AccountId> {
    fn is_blocked_account(account: AccountId, scope: ChannelId) -> bool;
    fn is_allowed_account(account: AccountId, scope: ChannelId) -> bool;
}

impl<AccountId> IsAccountBlocked<AccountId> for () {
    fn is_blocked_account(_account: AccountId, _scope: u64) -> bool {
        false
    }

    fn is_allowed_account(_account: AccountId, _scope: u64) -> bool {
        true
    }
}

pub trait IsChannelBlocked {
    fn is_blocked_channel(channel_id: ChannelId, scope: ChannelId) -> bool;
    fn is_allowed_channel(channel_id: ChannelId, scope: ChannelId) -> bool;
}

// TODO: reuse `type PostId` from slixon_utils in future updates
pub trait IsPostBlocked<PostId> {
    fn is_blocked_post(post_id: PostId, scope: ChannelId) -> bool;
    fn is_allowed_post(post_id: PostId, scope: ChannelId) -> bool;
}

impl<PostId> IsPostBlocked<PostId> for () {
    fn is_blocked_post(_post_id: PostId, _scope: ChannelId) -> bool {
        false
    }

    fn is_allowed_post(_post_id: PostId, _scope: u64) -> bool {
        true
    }
}

pub trait IsContentBlocked {
    fn is_blocked_content(content: Content, scope: ChannelId) -> bool;
    fn is_allowed_content(content: Content, scope: ChannelId) -> bool;
}

impl IsContentBlocked for () {
    fn is_blocked_content(_content: Content, _scope: u64) -> bool {
        false
    }
    fn is_allowed_content(_content: Content, _scope: ChannelId) -> bool {
        true
    }
}
