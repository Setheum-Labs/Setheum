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

use frame_support::{
    decl_error, decl_event, decl_module, decl_storage, ensure,
    dispatch::DispatchResult,
    traits::Get
};
use sp_std::prelude::*;
use frame_system::{self as system, ensure_signed};

use module_support::{
    ChannelFollowsProvider,
    moderation::IsAccountBlocked,
};
use slixon_profiles::{Module as Profiles, SocialAccountById};
use slixon_channels::{BeforeChannelCreated, Module as Channels, Channel, ChannelById};
use slixon_utils::{Error as UtilsError, ChannelId, remove_from_vec};

pub mod rpc;

/// The pallet's configuration trait.
pub trait Trait: system::Trait
    + slixon_utils::Trait
    + slixon_channels::Trait
    + slixon_profiles::Trait
{
    /// The overarching event type.
    type Event: From<Event<Self>> + Into<<Self as system::Trait>::Event>;

    type BeforeChannelFollowed: BeforeChannelFollowed<Self>;

    type BeforeChannelUnfollowed: BeforeChannelUnfollowed<Self>;
}

decl_error! {
    pub enum Error for Module<T: Trait> {
        /// Social account was not found by id.
        SocialAccountNotFound,
        /// Account is already a channel follower.
        AlreadyChannelFollower,
        /// Account is not a channel follower.
        NotChannelFollower,
        /// Not allowed to follow a hidden channel.
        CannotFollowHiddenChannel,
    }
}

// This pallet's storage items.
decl_storage! {
    trait Store for Module<T: Trait> as ChannelFollowsModule {
        pub ChannelFollowers get(fn channel_followers):
            map hasher(twox_64_concat) ChannelId => Vec<T::AccountId>;

        pub ChannelFollowedByAccount get(fn channel_followed_by_account):
            map hasher(blake2_128_concat) (T::AccountId, ChannelId) => bool;

        pub ChannelsFollowedByAccount get(fn channels_followed_by_account):
            map hasher(blake2_128_concat) T::AccountId => Vec<ChannelId>;
    }
}

decl_event!(
    pub enum Event<T> where
        <T as system::Trait>::AccountId,
    {
        ChannelFollowed(/* follower */ AccountId, /* following */ ChannelId),
        ChannelUnfollowed(/* follower */ AccountId, /* unfollowing */ ChannelId),
    }
);

// The pallet's dispatchable functions.
decl_module! {
  pub struct Module<T: Trait> for enum Call where origin: T::Origin {
    // Initializing errors
    type Error = Error<T>;

    // Initializing events
    fn deposit_event() = default;

    #[weight = 10_000 + T::DbWeight::get().reads_writes(5, 5)]
    pub fn follow_channel(origin, channel_id: ChannelId) -> DispatchResult {
      let follower = ensure_signed(origin)?;

      ensure!(!Self::channel_followed_by_account((follower.clone(), channel_id)), Error::<T>::AlreadyChannelFollower);

      let channel = &mut Channels::require_channel(channel_id)?;
      ensure!(!channel.hidden, Error::<T>::CannotFollowHiddenChannel);

      ensure!(T::IsAccountBlocked::is_allowed_account(follower.clone(), channel.id), UtilsError::<T>::AccountIsBlocked);

      Self::add_channel_follower(follower, channel)?;
      <ChannelById<T>>::insert(channel_id, channel);

      Ok(())
    }

    #[weight = 10_000 + T::DbWeight::get().reads_writes(5, 5)]
    pub fn unfollow_channel(origin, channel_id: ChannelId) -> DispatchResult {
      let follower = ensure_signed(origin)?;

      ensure!(Self::channel_followed_by_account((follower.clone(), channel_id)), Error::<T>::NotChannelFollower);

      Self::unfollow_channel_by_account(follower, channel_id)
    }
  }
}

impl<T: Trait> Module<T> {
    fn add_channel_follower(follower: T::AccountId, channel: &mut Channel<T>) -> DispatchResult {
        channel.inc_followers();

        let mut social_account = Profiles::get_or_new_social_account(follower.clone());
        social_account.inc_following_channels();

        T::BeforeChannelFollowed::before_channel_followed(
            follower.clone(), social_account.reputation, channel)?;

        let channel_id = channel.id;
        <ChannelFollowers<T>>::mutate(channel_id, |followers| followers.push(follower.clone()));
        <ChannelFollowedByAccount<T>>::insert((follower.clone(), channel_id), true);
        <ChannelsFollowedByAccount<T>>::mutate(follower.clone(), |channel_ids| channel_ids.push(channel_id));
        <SocialAccountById<T>>::insert(follower.clone(), social_account);

        Self::deposit_event(RawEvent::ChannelFollowed(follower, channel_id));

        Ok(())
    }

    pub fn unfollow_channel_by_account(follower: T::AccountId, channel_id: ChannelId) -> DispatchResult {
        let channel = &mut Channels::require_channel(channel_id)?;
        channel.dec_followers();

        let mut social_account = Profiles::social_account_by_id(follower.clone()).ok_or(Error::<T>::SocialAccountNotFound)?;
        social_account.dec_following_channels();

        T::BeforeChannelUnfollowed::before_channel_unfollowed(follower.clone(), channel)?;

        <ChannelsFollowedByAccount<T>>::mutate(follower.clone(), |channel_ids| remove_from_vec(channel_ids, channel_id));
        <ChannelFollowers<T>>::mutate(channel_id, |account_ids| remove_from_vec(account_ids, follower.clone()));
        <ChannelFollowedByAccount<T>>::remove((follower.clone(), channel_id));
        <SocialAccountById<T>>::insert(follower.clone(), social_account);
        <ChannelById<T>>::insert(channel_id, channel);

        Self::deposit_event(RawEvent::ChannelUnfollowed(follower, channel_id));
        Ok(())
    }
}

impl<T: Trait> ChannelFollowsProvider for Module<T> {
    type AccountId = T::AccountId;

    fn is_channel_follower(account: Self::AccountId, channel_id: ChannelId) -> bool {
        Module::<T>::channel_followed_by_account((account, channel_id))
    }
}

impl<T: Trait> BeforeChannelCreated<T> for Module<T> {
    fn before_channel_created(creator: T::AccountId, channel: &mut Channel<T>) -> DispatchResult {
        // Make a channel creator the first follower of this channel:
        Module::<T>::add_channel_follower(creator, channel)
    }
}

/// Handler that will be called right before the channel is followed.
pub trait BeforeChannelFollowed<T: Trait> {
    fn before_channel_followed(follower: T::AccountId, follower_reputation: u32, channel: &mut Channel<T>) -> DispatchResult;
}

impl<T: Trait> BeforeChannelFollowed<T> for () {
    fn before_channel_followed(_follower: T::AccountId, _follower_reputation: u32, _channel: &mut Channel<T>) -> DispatchResult {
        Ok(())
    }
}

/// Handler that will be called right before the channel is unfollowed.
pub trait BeforeChannelUnfollowed<T: Trait> {
    fn before_channel_unfollowed(follower: T::AccountId, channel: &mut Channel<T>) -> DispatchResult;
}

impl<T: Trait> BeforeChannelUnfollowed<T> for () {
    fn before_channel_unfollowed(_follower: T::AccountId, _channel: &mut Channel<T>) -> DispatchResult {
        Ok(())
    }
}
