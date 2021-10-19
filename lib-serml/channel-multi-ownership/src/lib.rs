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
use sp_std::prelude::*;
use sp_std::collections::{btree_map::BTreeMap, btree_set::BTreeSet};
use sp_runtime::{RuntimeDebug, traits::Zero};

use frame_support::{
  decl_error, decl_event, decl_module, decl_storage, ensure,
  traits::Get
};
use frame_system::{self as system, ensure_signed};

use slixon_utils::{ChannelId, WhoAndWhen};

pub mod functions;

#[cfg(test)]
mod tests;

#[derive(Encode, Decode, Clone, Eq, PartialEq, RuntimeDebug)]
pub struct ChannelOwners<T: Trait> {
  pub created: WhoAndWhen<T>,
  pub channel_id: ChannelId,
  pub owners: Vec<T::AccountId>,
  pub threshold: u16,
  pub changes_count: u16,
}

#[derive(Encode, Decode, Clone, Eq, PartialEq, RuntimeDebug)]
pub struct Change<T: Trait> {
  pub created: WhoAndWhen<T>,
  pub id: ChangeId,
  pub channel_id: ChannelId,
  pub add_owners: Vec<T::AccountId>,
  pub remove_owners: Vec<T::AccountId>,
  pub new_threshold: Option<u16>,
  pub notes: Vec<u8>,
  pub confirmed_by: Vec<T::AccountId>,
  pub expires_at: T::BlockNumber,
}

type ChangeId = u64;

/// The pallet's configuration trait.
pub trait Trait: system::Trait
  + pallet_timestamp::Trait
  + slixon_utils::Trait
{
  /// The overarching event type.
  type Event: From<Event<Self>> + Into<<Self as system::Trait>::Event>;

  /// Minimum channel owners allowed.
  type MinChannelOwners: Get<u16>;

  /// Maximum channel owners allowed.
  type MaxChannelOwners: Get<u16>;

  /// Maximum length of change notes.
  type MaxChangeNotesLength: Get<u16>;

  /// Expiration time for change proposal.
  type BlocksToLive: Get<Self::BlockNumber>;

  /// Period in blocks for which change proposal is can remain in a pending state until deleted.
  type DeleteExpiredChangesPeriod: Get<Self::BlockNumber>;
}

decl_error! {
  pub enum Error for Module<T: Trait> {
    /// Channel owners was not found by id
    ChannelOwnersNotFound,
    /// Change was not found by id
    ChangeNotFound,
    /// Channel owners already exist on this channel
    ChannelOwnersAlreadyExist,

    /// There can not be less owners than allowed
    NotEnoughOwners,
    /// There can not be more owners than allowed
    TooManyOwners,
    /// Account is not a channel owner
    NotAChannelOwner,

    /// The threshold can not be less than 1
    ZeroThershold,
    /// The required confirmation count can not be greater than owners count"
    TooBigThreshold,
    /// Change notes are too long
    ChangeNotesOversize,
    /// No channel owners will left in result of change
    NoChannelOwnersLeft,
    /// No updates proposed with this change
    NoUpdatesProposed,
    /// No fields update in result of change proposal
    NoFieldsUpdatedOnProposal,

    /// Account has already confirmed this change
    ChangeAlreadyConfirmed,
    /// There are not enough confirmations for this change
    NotEnoughConfirms,
    /// Change is already executed
    ChangeAlreadyExecuted,
    /// Change is not related to this channel
    ChangeNotRelatedToChannel,
    /// Pending change already exists
    PendingChangeAlreadyExists,
    /// Pending change doesn't exist
    PendingChangeDoesNotExist,

    /// Account is not a proposal creator
    NotAChangeCreator,

    /// Overflow when incrementing a counter of executed changes
    ChangesCountOverflow,
  }
}

// This pallet's storage items.
decl_storage! {
  trait Store for Module<T: Trait> as ChannelMultiOwnershipModule {
    ChannelOwnersByChannelById get(fn channel_owners_by_channel_id):
      map hasher(twox_64_concat) ChannelId => Option<ChannelOwners<T>>;

    ChannelIdsOwnedByAccountId get(fn channel_ids_owned_by_account_id):
      map hasher(twox_64_concat) T::AccountId => BTreeSet<ChannelId> = BTreeSet::new();

    NextChangeId get(fn next_change_id): ChangeId = 1;

    ChangeById get(fn change_by_id):
      map hasher(twox_64_concat) ChangeId => Option<Change<T>>;

    PendingChangeIdByChannelId get(fn pending_change_id_by_channel_id):
      map hasher(twox_64_concat) ChannelId => Option<ChangeId>;

    PendingChangeIds get(fn pending_change_ids): BTreeSet<ChangeId> = BTreeSet::new();

    ExecutedChangeIdsByChannelId get(fn executed_change_ids_by_channel_id):
      map hasher(twox_64_concat) ChannelId => Vec<ChangeId>;
  }
}

// The pallet's dispatchable functions.
decl_module! {
  pub struct Module<T: Trait> for enum Call where origin: T::Origin {
    /// Minimum channel owners allowed.
    const MinChannelOwners: u16 = T::MinChannelOwners::get();

    /// Maximum channel owners allowed.
    const MaxChannelOwners: u16 = T::MaxChannelOwners::get();

    /// Maximum length of change notes.
    const MaxChangeNotesLength: u16 = T::MaxChangeNotesLength::get();

    /// Period in blocks for which change proposal is can remain in a pending state until deleted.
    const BlocksToLive: T::BlockNumber = T::BlocksToLive::get();

    /// Period in blocks to initialize deleting of pending changes that are outdated.
    const DeleteExpiredChangesPeriod: T::BlockNumber = T::DeleteExpiredChangesPeriod::get();

    // Initializing events
    fn deposit_event() = default;

    fn on_finalize(n: T::BlockNumber) {
      Self::delete_expired_changes(n);
    }

    #[weight = T::DbWeight::get().reads_writes(2, 2) + 10_000]
    pub fn create_channel_owners(
      origin,
      channel_id: ChannelId,
      owners: Vec<T::AccountId>,
      threshold: u16
    ) {
      let who = ensure_signed(origin)?;

      ensure!(Self::channel_owners_by_channel_id(channel_id).is_none(), Error::<T>::ChannelOwnersAlreadyExist);

      let mut owners_map: BTreeMap<T::AccountId, bool> = BTreeMap::new();
      let mut unique_owners: Vec<T::AccountId> = Vec::new();

      for owner in owners.iter() {
        if !owners_map.contains_key(&owner) {
          owners_map.insert(owner.clone(), true);
          unique_owners.push(owner.clone());
        }
      }

      let owners_count = unique_owners.len() as u16;
      ensure!(owners_count >= T::MinChannelOwners::get(), Error::<T>::NotEnoughOwners);
      ensure!(owners_count <= T::MaxChannelOwners::get(), Error::<T>::TooManyOwners);

      ensure!(threshold <= owners_count, Error::<T>::TooBigThreshold);
      ensure!(threshold > 0, Error::<T>::ZeroThershold);

      let new_channel_owners = ChannelOwners {
        created: WhoAndWhen::<T>::new(who.clone()),
        channel_id,
        owners: unique_owners.clone(),
        threshold,
        changes_count: 0
      };

      <ChannelOwnersByChannelById<T>>::insert(channel_id, new_channel_owners);

      for owner in unique_owners.iter() {
        <ChannelIdsOwnedByAccountId<T>>::mutate(owner.clone(), |ids| ids.insert(channel_id));
      }

      Self::deposit_event(RawEvent::ChannelOwnersCreated(who, channel_id));
    }

    #[weight = T::DbWeight::get().reads_writes(5, 4) + 10_000]
    pub fn propose_change(
      origin,
      channel_id: ChannelId,
      add_owners: Vec<T::AccountId>,
      remove_owners: Vec<T::AccountId>,
      new_threshold: Option<u16>,
      notes: Vec<u8>
    ) {
      let who = ensure_signed(origin)?;

      let has_updates =
        !add_owners.is_empty() ||
        !remove_owners.is_empty() ||
        new_threshold.is_some();

      ensure!(has_updates, Error::<T>::NoUpdatesProposed);
      ensure!(notes.len() <= T::MaxChangeNotesLength::get() as usize, Error::<T>::ChangeNotesOversize);

      let channel_owners = Self::channel_owners_by_channel_id(channel_id).ok_or(Error::<T>::ChannelOwnersNotFound)?;
      ensure!(Self::pending_change_id_by_channel_id(channel_id).is_none(), Error::<T>::PendingChangeAlreadyExists);

      let is_channel_owner = channel_owners.owners.iter().any(|owner| *owner == who.clone());
      ensure!(is_channel_owner, Error::<T>::NotAChannelOwner);

      let mut fields_updated : u16 = 0;

      let result_owners = Self::transform_new_owners_to_vec(channel_owners.owners.clone(), add_owners.clone(), remove_owners.clone());
      ensure!(!result_owners.is_empty(), Error::<T>::NoChannelOwnersLeft);
      if result_owners != channel_owners.owners {
        fields_updated += 1;
      }

      if let Some(threshold) = new_threshold {
        if channel_owners.threshold != threshold {
          ensure!(threshold as usize <= result_owners.len(), Error::<T>::TooBigThreshold);
          ensure!(threshold > 0, Error::<T>::ZeroThershold);
          fields_updated += 1;
        }
      }

      let change_id = Self::next_change_id();
      let mut new_change = Change {
        created: WhoAndWhen::<T>::new(who.clone()),
        id: change_id,
        channel_id,
        add_owners,
        remove_owners,
        new_threshold,
        notes,
        confirmed_by: Vec::new(),
        expires_at: <system::Module<T>>::block_number() + T::BlocksToLive::get()
      };

      if fields_updated > 0 {
        new_change.confirmed_by.push(who.clone());
        <ChangeById<T>>::insert(change_id, new_change);
        PendingChangeIdByChannelId::insert(channel_id, change_id);
        PendingChangeIds::mutate(|set| set.insert(change_id));
        NextChangeId::mutate(|n| { *n += 1; });

        Self::deposit_event(RawEvent::ChangeProposed(who, channel_id, change_id));
      } else {
        return Err(Error::<T>::NoFieldsUpdatedOnProposal.into());
      }
    }

    #[weight = T::DbWeight::get().reads_writes(3, 1) + 10_000]
    pub fn confirm_change(
      origin,
      channel_id: ChannelId,
      change_id: ChangeId
    ) {
      let who = ensure_signed(origin)?;

      let channel_owners = Self::channel_owners_by_channel_id(channel_id).ok_or(Error::<T>::ChannelOwnersNotFound)?;

      let is_channel_owner = channel_owners.owners.iter().any(|owner| *owner == who.clone());
      ensure!(is_channel_owner, Error::<T>::NotAChannelOwner);

      let mut change = Self::change_by_id(change_id).ok_or(Error::<T>::ChangeNotFound)?;

      let pending_change_id = Self::pending_change_id_by_channel_id(channel_id).ok_or(Error::<T>::PendingChangeDoesNotExist)?;
      ensure!(pending_change_id == change_id, Error::<T>::ChangeNotRelatedToChannel);

      // Check whether sender confirmed change or not
      ensure!(!change.confirmed_by.iter().any(|account| *account == who.clone()), Error::<T>::ChangeAlreadyConfirmed);

      change.confirmed_by.push(who.clone());

      if change.confirmed_by.len() == channel_owners.threshold as usize {
        Self::update_channel_owners(who.clone(), channel_owners, change)?;
      } else {
        <ChangeById<T>>::insert(change_id, change);
      }

      Self::deposit_event(RawEvent::ChangeConfirmed(who, channel_id, change_id));
    }

    #[weight = T::DbWeight::get().reads_writes(4, 3) + 10_000]
    pub fn cancel_change(
      origin,
      channel_id: ChannelId,
      change_id: ChangeId
    ) {
      let who = ensure_signed(origin)?;

      let channel_owners = Self::channel_owners_by_channel_id(channel_id).ok_or(Error::<T>::ChannelOwnersNotFound)?;

      let is_channel_owner = channel_owners.owners.iter().any(|owner| *owner == who.clone());
      ensure!(is_channel_owner, Error::<T>::NotAChannelOwner);

      let pending_change_id = Self::pending_change_id_by_channel_id(channel_id).ok_or(Error::<T>::PendingChangeDoesNotExist)?;
      ensure!(pending_change_id == change_id, Error::<T>::ChangeNotRelatedToChannel);

      let change = Self::change_by_id(change_id).ok_or(Error::<T>::ChangeNotFound)?;
      ensure!(change.created.account == who, Error::<T>::NotAChangeCreator);

      <ChangeById<T>>::remove(change_id);
      PendingChangeIdByChannelId::remove(channel_id);
      PendingChangeIds::mutate(|set| set.remove(&change_id));

      Self::deposit_event(RawEvent::ProposalCanceled(who, channel_id));
    }
  }
}

decl_event!(
  pub enum Event<T> where
    <T as system::Trait>::AccountId,
   {
    ChannelOwnersCreated(AccountId, ChannelId),
    ChangeProposed(AccountId, ChannelId, ChangeId),
    ProposalCanceled(AccountId, ChannelId),
    ChangeConfirmed(AccountId, ChannelId, ChangeId),
    ChannelOwnersUpdated(AccountId, ChannelId, ChangeId),
  }
);
