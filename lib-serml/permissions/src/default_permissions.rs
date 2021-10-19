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

use crate::{ChannelPermission as SP, ChannelPermissions};

use sp_std::vec;
use frame_support::parameter_types;

parameter_types! {
  pub DefaultChannelPermissions: ChannelPermissions = ChannelPermissions {

    // No permissions disabled by default
    none: None,

    everyone: Some(vec![
      SP::UpdateOwnSubchannels,
      SP::DeleteOwnSubchannels,
      SP::HideOwnSubchannels,

      SP::UpdateOwnPosts,
      SP::DeleteOwnPosts,
      SP::HideOwnPosts,

      SP::CreateComments,
      SP::UpdateOwnComments,
      SP::DeleteOwnComments,
      SP::HideOwnComments,

      SP::Upvote,
      SP::Downvote,
      SP::Share,
    ].into_iter().collect()),

    // Followers can do everything that everyone else can.
    follower: None,

    channel_owner: Some(vec![
      SP::ManageRoles,
      SP::RepresentChannelInternally,
      SP::RepresentChannelExternally,
      SP::OverrideSubchannelPermissions,
      SP::OverridePostPermissions,

      SP::CreateSubchannels,
      SP::CreatePosts,

      SP::UpdateChannel,
      SP::UpdateAnySubchannel,
      SP::UpdateAnyPost,

      SP::DeleteAnySubchannel,
      SP::DeleteAnyPost,

      SP::HideAnySubchannel,
      SP::HideAnyPost,
      SP::HideAnyComment,

      SP::SuggestEntityStatus,
      SP::UpdateEntityStatus,

      SP::UpdateChannelSettings,
    ].into_iter().collect()),
  };
}
