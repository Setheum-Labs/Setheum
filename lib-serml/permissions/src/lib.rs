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

use codec::{Decode, Encode};
#[cfg(feature = "std")]
use serde::{Serialize, Deserialize};
use frame_support::{
  decl_module,
  traits::Get
};
use sp_runtime::RuntimeDebug;
use sp_std::{
  collections::btree_set::BTreeSet,
  prelude::*
};
use frame_system::{self as system};

use slixon_utils::ChannelId;

pub mod default_permissions;

#[derive(Encode, Decode, Ord, PartialOrd, Clone, Eq, PartialEq, RuntimeDebug)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub enum ChannelPermission {
  /// Create, update, delete, grant and revoke roles in this channel.
  ManageRoles,

  /// Act on behalf of this channel within this channel.
  RepresentChannelInternally,
  /// Act on behalf of this channel outside of this channel.
  RepresentChannelExternally,

  /// Update this channel.
  UpdateChannel,

  // Related to subchannels in this channel:
  CreateSubchannels,
  UpdateOwnSubchannels,
  DeleteOwnSubchannels,
  HideOwnSubchannels,

  UpdateAnySubchannel,
  DeleteAnySubchannel,
  HideAnySubchannel,

  // Related to posts in this channel:
  CreatePosts,
  UpdateOwnPosts,
  DeleteOwnPosts,
  HideOwnPosts,

  UpdateAnyPost,
  DeleteAnyPost,
  HideAnyPost,

  // Related to comments in this channel:
  CreateComments,
  UpdateOwnComments,
  DeleteOwnComments,
  HideOwnComments,

  // NOTE: It was made on purpose that it's not possible to update or delete not own comments.
  // Instead it's possible to allow to hide and block comments.
  HideAnyComment,

  /// Upvote any post or comment in this channel.
  Upvote,
  /// Downvote any post or comment in this channel.
  Downvote,
  /// Share any post or comment from this channel to another outer channel.
  Share,

  /// Override permissions per subchannel in this channel.
  OverrideSubchannelPermissions,
  /// Override permissions per post in this channel.
  OverridePostPermissions,

  // Related to the moderation pallet:

  /// Suggest new entity status in channel (whether it's blocked or allowed)
  SuggestEntityStatus,
  /// Update entity status in channel
  UpdateEntityStatus,

  // Related to channel settings:

  /// Allows to update channel settings across different pallets.
  UpdateChannelSettings,
}

pub type ChannelPermissionSet = BTreeSet<ChannelPermission>;

/// These are a set of built-in roles which can be given different permissions within a given channel.
/// For example: everyone can comment (`CreateComments`), but only followers can post 
/// (`CreatePosts`).
#[derive(Encode, Decode, Clone, Eq, PartialEq, RuntimeDebug)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub struct ChannelPermissions {

  /// None represents a set of permissions which is not capable of being performed by anyone.
  /// For example, if you want to create a channel similar to Twitter, you would set the permissions 
  /// for `UpdateOwnPosts`, `UpdateOwnComments`, and `Downvote` to `none`.
  pub none: Option<ChannelPermissionSet>,

  /// Everyone represents a set of permissions which are capable of being performed by every account
  /// in a given channel.
  pub everyone: Option<ChannelPermissionSet>,

  /// Follower represents a set of permissions which are capable of being performed by every account
  /// that follows a given channel.
  pub follower: Option<ChannelPermissionSet>,

  /// Channel owner represents a set of permissions which are capable of being performed by an account
  /// that is a current owner of a given channel.
  pub channel_owner: Option<ChannelPermissionSet>,
}

impl Default for ChannelPermissions {
  fn default() -> ChannelPermissions {
    ChannelPermissions {
      none: None,
      everyone: None,
      follower: None,
      channel_owner: None,
    }
  }
}

#[derive(Encode, Decode, Clone, Eq, PartialEq, RuntimeDebug)]
pub struct ChannelPermissionsContext {
  pub channel_id: ChannelId,
  pub is_channel_owner: bool,
  pub is_channel_follower: bool,
  pub channel_perms: Option<ChannelPermissions>
}

/// The pallet's configuration trait.
pub trait Trait: system::Trait {
  type DefaultChannelPermissions: Get<ChannelPermissions>;
}

decl_module! {
  pub struct Module<T: Trait> for enum Call where origin: T::Origin {
    const DefaultChannelPermissions: ChannelPermissions = T::DefaultChannelPermissions::get();
  }
}

impl ChannelPermission {
  fn is_present_in_role(&self, perms_opt: Option<ChannelPermissionSet>) -> bool {
    if let Some(perms) = perms_opt {
      if perms.contains(self) {
        return true
      }
    }
    false
  }
}

impl<T: Trait> Module<T> {

  fn get_overrides_or_defaults(
    overrides: Option<ChannelPermissionSet>,
    defaults: Option<ChannelPermissionSet>
  ) -> Option<ChannelPermissionSet> {

    if overrides.is_some() {
      overrides
    } else {
      defaults
    }
  }

  fn resolve_channel_perms(
    channel_perms: Option<ChannelPermissions>,
  ) -> ChannelPermissions {

    let defaults = T::DefaultChannelPermissions::get();
    let overrides = channel_perms.unwrap_or_default();

    ChannelPermissions {
      none: Self::get_overrides_or_defaults(overrides.none, defaults.none),
      everyone: Self::get_overrides_or_defaults(overrides.everyone, defaults.everyone),
      follower: Self::get_overrides_or_defaults(overrides.follower, defaults.follower),
      channel_owner: Self::get_overrides_or_defaults(overrides.channel_owner, defaults.channel_owner)
    }
  }

  pub fn has_user_a_channel_permission(
    ctx: ChannelPermissionsContext,
    permission: ChannelPermission,
  ) -> Option<bool> {

    let perms_by_role = Self::resolve_channel_perms(ctx.channel_perms);

    // Check if this permission is forbidden:
    if permission.is_present_in_role(perms_by_role.none) {
      return Some(false)
    }

    let is_channel_owner = ctx.is_channel_owner;
    let is_follower = is_channel_owner || ctx.is_channel_follower;

    if
      permission.is_present_in_role(perms_by_role.everyone) ||
      is_follower && permission.is_present_in_role(perms_by_role.follower) ||
      is_channel_owner && permission.is_present_in_role(perms_by_role.channel_owner)
    {
      return Some(true)
    }

    None
  }

  pub fn override_permissions(mut overrides: ChannelPermissions) -> ChannelPermissions {
    overrides.none = overrides.none.map(
      |mut none_permissions_set| {
        none_permissions_set.extend(T::DefaultChannelPermissions::get().none.unwrap_or_default());
        none_permissions_set
      }
    );

    overrides
  }
}