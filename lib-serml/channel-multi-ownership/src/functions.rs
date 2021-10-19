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

use super::*;

use sp_std::collections::btree_set::BTreeSet;
use frame_support::{dispatch::DispatchResult};

impl<T: Trait> Module<T> {

  pub fn update_channel_owners(who: T::AccountId, mut channel_owners: ChannelOwners<T>, change: Change<T>) -> DispatchResult {
    let channel_id = channel_owners.channel_id;
    let change_id = change.id;

    ensure!(change.confirmed_by.len() >= channel_owners.threshold as usize, Error::<T>::NotEnoughConfirms);
    Self::move_change_from_pending_state_to_executed(channel_id, change_id)?;

    channel_owners.changes_count = channel_owners.changes_count.checked_add(1).ok_or(Error::<T>::ChangesCountOverflow)?;
    if !change.add_owners.is_empty() || !change.remove_owners.is_empty() {
      channel_owners.owners = Self::transform_new_owners_to_vec(
        channel_owners.owners.clone(), change.add_owners.clone(), change.remove_owners.clone());
    }

    if let Some(threshold) = change.new_threshold {
      channel_owners.threshold = threshold;
    }

    for account in &change.add_owners {
      <ChannelIdsOwnedByAccountId<T>>::mutate(account, |ids| ids.insert(channel_id));
    }
    for account in &change.remove_owners {
      <ChannelIdsOwnedByAccountId<T>>::mutate(account, |ids| ids.remove(&channel_id));
    }

    <ChannelOwnersByChannelById<T>>::insert(channel_id, channel_owners);
    <ChangeById<T>>::insert(change_id, change);
    Self::deposit_event(RawEvent::ChannelOwnersUpdated(who, channel_id, change_id));

    Ok(())
  }

  pub fn move_change_from_pending_state_to_executed(channel_id: ChannelId, change_id: ChangeId) -> DispatchResult {
    ensure!(Self::channel_owners_by_channel_id(channel_id).is_some(), Error::<T>::ChannelOwnersNotFound);
    ensure!(Self::change_by_id(change_id).is_some(), Error::<T>::ChangeNotFound);
    ensure!(!Self::executed_change_ids_by_channel_id(channel_id).iter().any(|&x| x == change_id), Error::<T>::ChangeAlreadyExecuted);

    PendingChangeIdByChannelId::remove(&channel_id);
    PendingChangeIds::mutate(|set| set.remove(&change_id));
    ExecutedChangeIdsByChannelId::mutate(channel_id, |ids| ids.push(change_id));

    Ok(())
  }

  pub fn transform_new_owners_to_vec(current_owners: Vec<T::AccountId>, add_owners: Vec<T::AccountId>, remove_owners: Vec<T::AccountId>) -> Vec<T::AccountId> {
    let mut owners_set: BTreeSet<T::AccountId> = BTreeSet::new();
    let mut new_owners_set: BTreeSet<T::AccountId> = BTreeSet::new();

    // Extract current channel owners
    current_owners.iter().for_each(|x| { owners_set.insert(x.clone()); });
    // Extract owners that should be added
    add_owners.iter().for_each(|x| { new_owners_set.insert(x.clone()); });
    // Unite both sets
    owners_set = owners_set.union(&new_owners_set).cloned().collect();
    // Remove accounts that exist in remove_owners from set
    remove_owners.iter().for_each(|x| { owners_set.remove(x); });

    owners_set.iter().cloned().collect()
  }

  pub fn delete_expired_changes(block_number: T::BlockNumber) {
    if (block_number % T::DeleteExpiredChangesPeriod::get()).is_zero() {
      for change_id in Self::pending_change_ids() {
        if let Some(change) = Self::change_by_id(change_id) {
          if block_number >= change.expires_at {
            PendingChangeIdByChannelId::remove(&change.channel_id);
            <ChangeById<T>>::remove(&change_id);
            PendingChangeIds::mutate(|set| set.remove(&change_id));
          }
        }
      }
    }
  }
}
