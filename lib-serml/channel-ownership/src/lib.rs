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
    decl_error, decl_event, decl_module, decl_storage,
    ensure,
    dispatch::DispatchResult,
    traits::Get
};
use sp_std::prelude::*;
use frame_system::{self as system, ensure_signed};

use module_support::moderation::IsAccountBlocked;
use slixon_channels::{Module as Channels, ChannelById, ChannelIdsByOwner};
use slixon_utils::{Error as UtilsError, ChannelId, remove_from_vec};

/// The pallet's configuration trait.
pub trait Trait: system::Trait
    + slixon_utils::Trait
    + slixon_channels::Trait
{
    /// The overarching event type.
    type Event: From<Event<Self>> + Into<<Self as system::Trait>::Event>;
}

decl_error! {
  pub enum Error for Module<T: Trait> {
    /// The current channel owner cannot transfer ownership to themself.
    CannotTranferToCurrentOwner,
    /// Account is already an owner of a channel.
    AlreadyAChannelOwner,
    /// There is no pending ownership transfer for a given channel.
    NoPendingTransferOnChannel,
    /// Account is not allowed to accept ownership transfer.
    NotAllowedToAcceptOwnershipTransfer,
    /// Account is not allowed to reject ownership transfer.
    NotAllowedToRejectOwnershipTransfer,
  }
}

// This pallet's storage items.
decl_storage! {
    trait Store for Module<T: Trait> as ChannelOwnershipModule {
        pub PendingChannelOwner get(fn pending_channel_owner):
            map hasher(twox_64_concat) ChannelId => Option<T::AccountId>;
    }
}

decl_event!(
    pub enum Event<T> where
        <T as system::Trait>::AccountId,
    {
        ChannelOwnershipTransferCreated(/* current owner */ AccountId, ChannelId, /* new owner */ AccountId),
        ChannelOwnershipTransferAccepted(AccountId, ChannelId),
        ChannelOwnershipTransferRejected(AccountId, ChannelId),
    }
);

// The pallet's dispatchable functions.
decl_module! {
  pub struct Module<T: Trait> for enum Call where origin: T::Origin {

    // Initializing errors
    type Error = Error<T>;

    // Initializing events
    fn deposit_event() = default;

    #[weight = 10_000 + T::DbWeight::get().reads_writes(1, 1)]
    pub fn transfer_channel_ownership(origin, channel_id: ChannelId, transfer_to: T::AccountId) -> DispatchResult {
      let who = ensure_signed(origin)?;

      let channel = Channels::<T>::require_channel(channel_id)?;
      channel.ensure_channel_owner(who.clone())?;

      ensure!(who != transfer_to, Error::<T>::CannotTranferToCurrentOwner);
      ensure!(T::IsAccountBlocked::is_allowed_account(transfer_to.clone(), channel_id), UtilsError::<T>::AccountIsBlocked);

      <PendingChannelOwner<T>>::insert(channel_id, transfer_to.clone());

      Self::deposit_event(RawEvent::ChannelOwnershipTransferCreated(who, channel_id, transfer_to));
      Ok(())
    }

    #[weight = 10_000 + T::DbWeight::get().reads_writes(2, 2)]
    pub fn accept_pending_ownership(origin, channel_id: ChannelId) -> DispatchResult {
      let new_owner = ensure_signed(origin)?;

      let mut channel = Channels::require_channel(channel_id)?;
      ensure!(!channel.is_owner(&new_owner), Error::<T>::AlreadyAChannelOwner);

      let transfer_to = Self::pending_channel_owner(channel_id).ok_or(Error::<T>::NoPendingTransferOnChannel)?;
      ensure!(new_owner == transfer_to, Error::<T>::NotAllowedToAcceptOwnershipTransfer);

      // Here we know that the origin is eligible to become a new owner of this channel.
      <PendingChannelOwner<T>>::remove(channel_id);

      Channels::maybe_transfer_handle_deposit_to_new_channel_owner(&channel, &new_owner)?;

      let old_owner = channel.owner;
      channel.owner = new_owner.clone();
      <ChannelById<T>>::insert(channel_id, channel);

      // Remove channel id from the list of channels by old owner
      <ChannelIdsByOwner<T>>::mutate(old_owner, |channel_ids| remove_from_vec(channel_ids, channel_id));

      // Add channel id to the list of channels by new owner
      <ChannelIdsByOwner<T>>::mutate(new_owner.clone(), |ids| ids.push(channel_id));

      // TODO add a new owner as a channel follower? See T::BeforeChannelCreated::before_channel_created(new_owner.clone(), channel)?;

      Self::deposit_event(RawEvent::ChannelOwnershipTransferAccepted(new_owner, channel_id));
      Ok(())
    }

    #[weight = 10_000 + T::DbWeight::get().reads_writes(2, 1)]
    pub fn reject_pending_ownership(origin, channel_id: ChannelId) -> DispatchResult {
      let who = ensure_signed(origin)?;

      let channel = Channels::<T>::require_channel(channel_id)?;
      let transfer_to = Self::pending_channel_owner(channel_id).ok_or(Error::<T>::NoPendingTransferOnChannel)?;
      ensure!(who == transfer_to || who == channel.owner, Error::<T>::NotAllowedToRejectOwnershipTransfer);

      <PendingChannelOwner<T>>::remove(channel_id);

      Self::deposit_event(RawEvent::ChannelOwnershipTransferRejected(who, channel_id));
      Ok(())
    }
  }
}
