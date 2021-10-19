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

//! # Channels Module
//! 
//! Channels are the primary components of Subsocial. This module allows you to create a Channel
//! and customize it by updating its' owner(s), content, unique handle, and permissions.
//! 
//! To understand how Channels fit into the Subsocial ecosystem, you can think of how 
//! folders and files work in a file system. Channels are similar to folders, that can contain Posts, 
//! in this sense. The permissions of the Channel and Posts can be customized so that a Channel 
//! could be as simple as a personal blog (think of a page on Facebook) or as complex as community 
//! (think of a subreddit) governed DAO.
//! 
//! Channels can be compared to existing entities on web 2.0 platforms such as:
//! 
//! - Blogs on Blogger,
//! - Publications on Medium,
//! - Groups or pages on Facebook,
//! - Accounts on Twitter and Instagram,
//! - Channels on YouTube,
//! - Servers on Discord,
//! - Forums on Discourse.

#![cfg_attr(not(feature = "std"), no_std)]

use codec::{Decode, Encode};
use frame_support::{
    decl_error, decl_event, decl_module, decl_storage, ensure,
    dispatch::{DispatchError, DispatchResult},
    traits::{Get, Currency, ExistenceRequirement, ReservableCurrency},
};
use sp_runtime::RuntimeDebug;
use sp_std::prelude::*;
use frame_system::{self as system, ensure_signed};

use module_support::{
    ChannelForRoles, ChannelForRolesProvider, PermissionChecker, ChannelFollowsProvider,
    moderation::{IsAccountBlocked, IsContentBlocked},
};
use slixon_permissions::{Module as Permissions, ChannelPermission, ChannelPermissions, ChannelPermissionsContext};
use slixon_utils::{Module as Utils, Error as UtilsError, ChannelId, WhoAndWhen, Content};

pub mod rpc;

/// Information about a channel's owner, its' content, visibility and custom permissions.
#[derive(Encode, Decode, Clone, Eq, PartialEq, RuntimeDebug)]
pub struct Channel<T: Trait> {

    /// Unique sequential identifier of a channel. Examples of channel ids: `1`, `2`, `3`, and so on.
    pub id: ChannelId,

    pub created: WhoAndWhen<T>,
    pub updated: Option<WhoAndWhen<T>>,

    /// The current owner of a given channel.
    pub owner: T::AccountId,

    // The next fields can be updated by the owner:

    pub parent_id: Option<ChannelId>,

    /// Unique alpha-numeric identifier that can be used in a channel's URL.
    /// Handle can only contain numbers, letter and underscore: `0`-`9`, `a`-`z`, `_`.
    pub handle: Option<Vec<u8>>,

    pub content: Content,

    /// Hidden field is used to recommend to end clients (web and mobile apps) that a particular 
    /// channel and its' posts should not be shown.
    pub hidden: bool,

    /// The total number of posts in a given channel.
    pub posts_count: u32,

    /// The number of hidden posts in a given channel.
    pub hidden_posts_count: u32,

    /// The number of account following a given channel.
    pub followers_count: u32,

    pub score: i32,

    /// This allows you to override Subsocial's default permissions by enabling or disabling role 
    /// permissions.
    pub permissions: Option<ChannelPermissions>,
}

#[derive(Encode, Decode, Clone, Eq, PartialEq, RuntimeDebug)]
#[allow(clippy::option_option)]
pub struct ChannelUpdate {
    pub parent_id: Option<Option<ChannelId>>,
    pub handle: Option<Option<Vec<u8>>>,
    pub content: Option<Content>,
    pub hidden: Option<bool>,
    pub permissions: Option<Option<ChannelPermissions>>,
}

type BalanceOf<T> =
  <<T as Trait>::Currency as Currency<<T as system::Trait>::AccountId>>::Balance;

/// The pallet's configuration trait.
pub trait Trait: system::Trait
    + slixon_utils::Trait
    + slixon_permissions::Trait
{
    /// The overarching event type.
    type Event: From<Event<Self>> + Into<<Self as system::Trait>::Event>;

    type Currency: ReservableCurrency<Self::AccountId>;

    type Roles: PermissionChecker<AccountId=Self::AccountId>;

    type ChannelFollows: ChannelFollowsProvider<AccountId=Self::AccountId>;

    type BeforeChannelCreated: BeforeChannelCreated<Self>;

    type AfterChannelUpdated: AfterChannelUpdated<Self>;

    type IsAccountBlocked: IsAccountBlocked<Self::AccountId>;

    type IsContentBlocked: IsContentBlocked;

    type HandleDeposit: Get<BalanceOf<Self>>;
}

decl_error! {
  pub enum Error for Module<T: Trait> {
    /// Channel was not found by id.
    ChannelNotFound,
    /// Channel handle is not unique.
    ChannelHandleIsNotUnique,
    /// Nothing to update in this channel.
    NoUpdatesForChannel,
    /// Only channel owners can manage this channel.
    NotAChannelOwner,
    /// User has no permission to update this channel.
    NoPermissionToUpdateChannel,
    /// User has no permission to create subchannels within this channel.
    NoPermissionToCreateSubchannels,
    /// Channel is at root level, no `parent_id` specified.
    ChannelIsAtRoot,
  }
}

pub const FIRST_SPACE_ID: u64 = 1;
pub const RESERVED_SPACE_COUNT: u64 = 1000;

// This pallet's storage items.
decl_storage! {
    trait Store for Module<T: Trait> as ChannelsModule {

        /// The next channel id.
        pub NextChannelId get(fn next_channel_id): ChannelId = RESERVED_SPACE_COUNT + 1;

        /// Get the details of a channel by its' id.
        pub ChannelById get(fn channel_by_id) build(|config: &GenesisConfig<T>| {
          let mut channels: Vec<(ChannelId, Channel<T>)> = Vec::new();
          let endowed_account = config.endowed_account.clone();
          for id in FIRST_SPACE_ID..=RESERVED_SPACE_COUNT {
            channels.push((id, Channel::<T>::new(id, None, endowed_account.clone(), Content::None, None, None)));
          }
          channels
        }):
            map hasher(twox_64_concat) ChannelId => Option<Channel<T>>;

        /// Find a given channel id by its' unique handle.
        /// If a handle is not registered, nothing will be returned (`None`).
        pub ChannelIdByHandle get(fn channel_id_by_handle):
            map hasher(blake2_128_concat) Vec<u8> => Option<ChannelId>;

        /// Find the ids of all channels owned, by a given account.
        pub ChannelIdsByOwner get(fn channel_ids_by_owner):
            map hasher(twox_64_concat) T::AccountId => Vec<ChannelId>;
    }
    add_extra_genesis {
      config(endowed_account): T::AccountId;
    }
}

decl_event!(
    pub enum Event<T> where
        <T as system::Trait>::AccountId,
    {
        ChannelCreated(AccountId, ChannelId),
        ChannelUpdated(AccountId, ChannelId),
        ChannelDeleted(AccountId, ChannelId),
    }
);

// The pallet's dispatchable functions.
decl_module! {
  pub struct Module<T: Trait> for enum Call where origin: T::Origin {

    const HandleDeposit: BalanceOf<T> = T::HandleDeposit::get();

    // Initializing errors
    type Error = Error<T>;

    // Initializing events
    fn deposit_event() = default;

    #[weight = 500_000 + T::DbWeight::get().reads_writes(4, 4)]
    pub fn create_channel(
      origin,
      parent_id_opt: Option<ChannelId>,
      handle_opt: Option<Vec<u8>>,
      content: Content,
      permissions_opt: Option<ChannelPermissions>
    ) -> DispatchResult {
      let owner = ensure_signed(origin)?;

      Utils::<T>::is_valid_content(content.clone())?;

      // TODO: add tests for this case
      if let Some(parent_id) = parent_id_opt {
        let parent_channel = Self::require_channel(parent_id)?;

        ensure!(T::IsAccountBlocked::is_allowed_account(owner.clone(), parent_id), UtilsError::<T>::AccountIsBlocked);
        ensure!(T::IsContentBlocked::is_allowed_content(content.clone(), parent_id), UtilsError::<T>::ContentIsBlocked);

        Self::ensure_account_has_channel_permission(
          owner.clone(),
          &parent_channel,
          ChannelPermission::CreateSubchannels,
          Error::<T>::NoPermissionToCreateSubchannels.into()
        )?;
      }

      let permissions = permissions_opt.map(|perms| {
        Permissions::<T>::override_permissions(perms)
      });

      let channel_id = Self::next_channel_id();
      let new_channel = &mut Channel::new(channel_id, parent_id_opt, owner.clone(), content, handle_opt.clone(), permissions);

      if let Some(handle) = handle_opt {
        Self::reserve_handle(&new_channel, handle)?;
      }

      T::BeforeChannelCreated::before_channel_created(owner.clone(), new_channel)?;

      <ChannelById<T>>::insert(channel_id, new_channel);
      <ChannelIdsByOwner<T>>::mutate(owner.clone(), |ids| ids.push(channel_id));
      NextChannelId::mutate(|n| { *n += 1; });

      Self::deposit_event(RawEvent::ChannelCreated(owner, channel_id));
      Ok(())
    }

    #[weight = 500_000 + T::DbWeight::get().reads_writes(2, 3)]
    pub fn update_channel(origin, channel_id: ChannelId, update: ChannelUpdate) -> DispatchResult {
      let owner = ensure_signed(origin)?;

      let has_updates =
        update.parent_id.is_some() ||
        update.handle.is_some() ||
        update.content.is_some() ||
        update.hidden.is_some() ||
        update.permissions.is_some();

      ensure!(has_updates, Error::<T>::NoUpdatesForChannel);

      let mut channel = Self::require_channel(channel_id)?;

      ensure!(T::IsAccountBlocked::is_allowed_account(owner.clone(), channel.id), UtilsError::<T>::AccountIsBlocked);

      Self::ensure_account_has_channel_permission(
        owner.clone(),
        &channel,
        ChannelPermission::UpdateChannel,
        Error::<T>::NoPermissionToUpdateChannel.into()
      )?;

      let mut is_update_applied = false;
      let mut old_data = ChannelUpdate::default();

      // TODO: add tests for this case
      if let Some(parent_id_opt) = update.parent_id {
        if parent_id_opt != channel.parent_id {

          if let Some(parent_id) = parent_id_opt {
            let parent_channel = Self::require_channel(parent_id)?;

            Self::ensure_account_has_channel_permission(
              owner.clone(),
              &parent_channel,
              ChannelPermission::CreateSubchannels,
              Error::<T>::NoPermissionToCreateSubchannels.into()
            )?;
          }

          old_data.parent_id = Some(channel.parent_id);
          channel.parent_id = parent_id_opt;
          is_update_applied = true;
        }
      }

      if let Some(content) = update.content {
        if content != channel.content {
          Utils::<T>::is_valid_content(content.clone())?;

          ensure!(T::IsContentBlocked::is_allowed_content(content.clone(), channel.id), UtilsError::<T>::ContentIsBlocked);
          if let Some(parent_id) = channel.parent_id {
            ensure!(T::IsContentBlocked::is_allowed_content(content.clone(), parent_id), UtilsError::<T>::ContentIsBlocked);
          }

          old_data.content = Some(channel.content);
          channel.content = content;
          is_update_applied = true;
        }
      }

      if let Some(hidden) = update.hidden {
        if hidden != channel.hidden {
          old_data.hidden = Some(channel.hidden);
          channel.hidden = hidden;
          is_update_applied = true;
        }
      }

      if let Some(overrides_opt) = update.permissions {
        if channel.permissions != overrides_opt {
          old_data.permissions = Some(channel.permissions);

          if let Some(overrides) = overrides_opt.clone() {
            channel.permissions = Some(Permissions::<T>::override_permissions(overrides));
          } else {
            channel.permissions = overrides_opt;
          }

          is_update_applied = true;
        }
      }

      let is_handle_updated = Self::update_handle(&channel, update.handle.clone())?;
      if is_handle_updated {
          old_data.handle = Some(channel.handle);
          channel.handle = update.handle.unwrap();
          is_update_applied = true
        }

      // Update this channel only if at least one field should be updated:
      if is_update_applied {
        channel.updated = Some(WhoAndWhen::<T>::new(owner.clone()));

        <ChannelById<T>>::insert(channel_id, channel.clone());
        T::AfterChannelUpdated::after_channel_updated(owner.clone(), &channel, old_data);

        Self::deposit_event(RawEvent::ChannelUpdated(owner, channel_id));
      }
      Ok(())
    }
  }
}

impl<T: Trait> Channel<T> {
    pub fn new(
        id: ChannelId,
        parent_id: Option<ChannelId>,
        created_by: T::AccountId,
        content: Content,
        handle: Option<Vec<u8>>,
        permissions: Option<ChannelPermissions>,
    ) -> Self {
        Channel {
            id,
            created: WhoAndWhen::<T>::new(created_by.clone()),
            updated: None,
            owner: created_by,
            parent_id,
            handle,
            content,
            hidden: false,
            posts_count: 0,
            hidden_posts_count: 0,
            followers_count: 0,
            score: 0,
            permissions,
        }
    }

    pub fn is_owner(&self, account: &T::AccountId) -> bool {
        self.owner == *account
    }

    pub fn is_follower(&self, account: &T::AccountId) -> bool {
        T::ChannelFollows::is_channel_follower(account.clone(), self.id)
    }

    pub fn ensure_channel_owner(&self, account: T::AccountId) -> DispatchResult {
        ensure!(self.is_owner(&account), Error::<T>::NotAChannelOwner);
        Ok(())
    }

    pub fn inc_posts(&mut self) {
        self.posts_count = self.posts_count.saturating_add(1);
    }

    pub fn dec_posts(&mut self) {
        self.posts_count = self.posts_count.saturating_sub(1);
    }

    pub fn inc_hidden_posts(&mut self) {
        self.hidden_posts_count = self.hidden_posts_count.saturating_add(1);
    }

    pub fn dec_hidden_posts(&mut self) {
        self.hidden_posts_count = self.hidden_posts_count.saturating_sub(1);
    }

    pub fn inc_followers(&mut self) {
        self.followers_count = self.followers_count.saturating_add(1);
    }

    pub fn dec_followers(&mut self) {
        self.followers_count = self.followers_count.saturating_sub(1);
    }

    #[allow(clippy::comparison_chain)]
    pub fn change_score(&mut self, diff: i16) {
        if diff > 0 {
            self.score = self.score.saturating_add(diff.abs() as i32);
        } else if diff < 0 {
            self.score = self.score.saturating_sub(diff.abs() as i32);
        }
    }

    pub fn try_get_parent(&self) -> Result<ChannelId, DispatchError> {
        self.parent_id.ok_or_else(|| Error::<T>::ChannelIsAtRoot.into())
    }

    pub fn is_public(&self) -> bool {
        !self.hidden && self.content.is_some()
    }

    pub fn is_unlisted(&self) -> bool {
        !self.is_public()
    }
}

impl Default for ChannelUpdate {
    fn default() -> Self {
        ChannelUpdate {
            parent_id: None,
            handle: None,
            content: None,
            hidden: None,
            permissions: None,
        }
    }
}

impl<T: Trait> Module<T> {

    /// Check that there is a `Channel` with such `channel_id` in the storage
    /// or return`ChannelNotFound` error.
    pub fn ensure_channel_exists(channel_id: ChannelId) -> DispatchResult {
        ensure!(<ChannelById<T>>::contains_key(channel_id), Error::<T>::ChannelNotFound);
        Ok(())
    }

    /// Get `Channel` by id from the storage or return `ChannelNotFound` error.
    pub fn require_channel(channel_id: ChannelId) -> Result<Channel<T>, DispatchError> {
        Ok(Self::channel_by_id(channel_id).ok_or(Error::<T>::ChannelNotFound)?)
    }

    pub fn ensure_account_has_channel_permission(
        account: T::AccountId,
        channel: &Channel<T>,
        permission: ChannelPermission,
        error: DispatchError,
    ) -> DispatchResult {
        let is_owner = channel.is_owner(&account);
        let is_follower = channel.is_follower(&account);

        let ctx = ChannelPermissionsContext {
            channel_id: channel.id,
            is_channel_owner: is_owner,
            is_channel_follower: is_follower,
            channel_perms: channel.permissions.clone(),
        };

        T::Roles::ensure_account_has_channel_permission(
            account,
            ctx,
            permission,
            error,
        )
    }

    pub fn try_move_channel_to_root(channel_id: ChannelId) -> DispatchResult {
        let mut channel = Self::require_channel(channel_id)?;
        channel.parent_id = None;

        ChannelById::<T>::insert(channel_id, channel);
        Ok(())
    }

    pub fn mutate_channel_by_id<F: FnOnce(&mut Channel<T>)> (
        channel_id: ChannelId,
        f: F
    ) -> Result<Channel<T>, DispatchError> {
        <ChannelById<T>>::mutate(channel_id, |channel_opt| {
            if let Some(ref mut channel) = channel_opt.clone() {
                f(channel);
                *channel_opt = Some(channel.clone());

                return Ok(channel.clone());
            }

            Err(Error::<T>::ChannelNotFound.into())
        })
    }

    /// Lowercase a handle and ensure that it's unique, i.e. no channel reserved this handle yet.
    fn lowercase_and_ensure_unique_handle(handle: Vec<u8>) -> Result<Vec<u8>, DispatchError> {
        let handle_in_lowercase = Utils::<T>::lowercase_and_validate_a_handle(handle)?;

        // Check if a handle is unique across all channels' handles:
        ensure!(Self::channel_id_by_handle(handle_in_lowercase.clone()).is_none(), Error::<T>::ChannelHandleIsNotUnique);

        Ok(handle_in_lowercase)
    }

    pub fn reserve_handle_deposit(channel_owner: &T::AccountId) -> DispatchResult {
        <T as Trait>::Currency::reserve(channel_owner, T::HandleDeposit::get())
    }

    pub fn unreserve_handle_deposit(channel_owner: &T::AccountId) -> BalanceOf<T> {
        <T as Trait>::Currency::unreserve(channel_owner, T::HandleDeposit::get())
    }

    /// This function will be performed only if a channel has a handle.
    /// Unreserve a handle deposit from the current channel owner,
    /// then transfer deposit amount to a new owner
    /// and reserve this amount from a new owner.
    pub fn maybe_transfer_handle_deposit_to_new_channel_owner(channel: &Channel<T>, new_owner: &T::AccountId) -> DispatchResult {
        if channel.handle.is_some() {
            let old_owner = &channel.owner;
            Self::unreserve_handle_deposit(old_owner);
            <T as Trait>::Currency::transfer(
                old_owner,
                new_owner,
                T::HandleDeposit::get(),
                ExistenceRequirement::KeepAlive
            )?;
            Self::reserve_handle_deposit(new_owner)?;
        }
        Ok(())
    }

    fn reserve_handle(
        channel: &Channel<T>,
        handle: Vec<u8>
    ) -> DispatchResult {
        let handle_in_lowercase = Self::lowercase_and_ensure_unique_handle(handle)?;
        Self::reserve_handle_deposit(&channel.owner)?;
        ChannelIdByHandle::insert(handle_in_lowercase, channel.id);
        Ok(())
    }

    fn unreserve_handle(
        channel: &Channel<T>,
        handle: Vec<u8>
    ) -> DispatchResult {
        let handle_in_lowercase = Utils::<T>::lowercase_handle(handle);
        Self::unreserve_handle_deposit(&channel.owner);
        ChannelIdByHandle::remove(handle_in_lowercase);
        Ok(())
    }

    fn update_handle(
        channel: &Channel<T>,
        maybe_new_handle: Option<Option<Vec<u8>>>,
    ) -> Result<bool, DispatchError> {
        let mut is_handle_updated = false;
        if let Some(new_handle_opt) = maybe_new_handle {
            if let Some(old_handle) = channel.handle.clone() {
                // If the channel has a handle

                if let Some(new_handle) = new_handle_opt {
                    if new_handle != old_handle {
                        // Change the current handle to a new one

                        // Validate data first
                        let old_handle_lc = Utils::<T>::lowercase_handle(old_handle);
                        let new_handle_lc = Self::lowercase_and_ensure_unique_handle(new_handle)?;

                        // Update storage once data is valid
                        ChannelIdByHandle::remove(old_handle_lc);
                        ChannelIdByHandle::insert(new_handle_lc, channel.id);
                        is_handle_updated = true;
                    }
                } else {
                    // Unreserve the current handle
                    Self::unreserve_handle(channel, old_handle)?;
                    is_handle_updated = true;
                }
            } else if let Some(new_handle) = new_handle_opt {
                // Reserve a handle for the channel that has no handle yet
                Self::reserve_handle(channel, new_handle)?;
                is_handle_updated = true;
            }
        }
        Ok(is_handle_updated)
    }
}

impl<T: Trait> ChannelForRolesProvider for Module<T> {
    type AccountId = T::AccountId;

    fn get_channel(id: ChannelId) -> Result<ChannelForRoles<Self::AccountId>, DispatchError> {
        let channel = Module::<T>::require_channel(id)?;

        Ok(ChannelForRoles {
            owner: channel.owner,
            permissions: channel.permissions,
        })
    }
}

pub trait BeforeChannelCreated<T: Trait> {
    fn before_channel_created(follower: T::AccountId, channel: &mut Channel<T>) -> DispatchResult;
}

impl<T: Trait> BeforeChannelCreated<T> for () {
    fn before_channel_created(_follower: T::AccountId, _channel: &mut Channel<T>) -> DispatchResult {
        Ok(())
    }
}

#[impl_trait_for_tuples::impl_for_tuples(10)]
pub trait AfterChannelUpdated<T: Trait> {
    fn after_channel_updated(sender: T::AccountId, channel: &Channel<T>, old_data: ChannelUpdate);
}
