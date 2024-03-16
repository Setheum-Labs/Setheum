// بِسْمِ اللَّهِ الرَّحْمَنِ الرَّحِيم

// This file is part of Setheum.

// Copyright (C) 2019-Present Setheum Labs.
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

use primitives::{FinalityCommitteeManager, SessionIndex};
use sp_std::vec::Vec;

use crate::{
    Config, Event, FinalityScheduledVersionChange, FinalityVersion, NextFinalityCommittee, Pallet,
};

impl<T> pallet_session::SessionManager<T::AccountId> for Pallet<T>
where
    T: Config,
{
    fn new_session(new_index: SessionIndex) -> Option<Vec<T::AccountId>> {
        <T as Config>::SessionManager::new_session(new_index)
    }

    fn new_session_genesis(new_index: SessionIndex) -> Option<Vec<T::AccountId>> {
        <T as Config>::SessionManager::new_session_genesis(new_index)
    }

    fn end_session(end_index: SessionIndex) {
        <T as Config>::SessionManager::end_session(end_index);
    }

    fn start_session(start_index: SessionIndex) {
        <T as Config>::SessionManager::start_session(start_index);
        Self::update_version_change_history();
    }
}

impl<T> Pallet<T>
where
    T: Config,
{
    // Check if a schedule version change has moved into the past. Update history, even if there is
    // no change. Resets the scheduled version.
    fn update_version_change_history() {
        let current_session = Self::current_session();

        if let Some(scheduled_version_change) = <FinalityScheduledVersionChange<T>>::get() {
            let scheduled_session = scheduled_version_change.session;
            let scheduled_version = scheduled_version_change.version_incoming;

            // Record the scheduled version as the current version as it moves into the past.
            if scheduled_session == current_session {
                <FinalityVersion<T>>::put(scheduled_version);

                // Reset the scheduled version.
                <FinalityScheduledVersionChange<T>>::kill();

                Self::deposit_event(Event::FinalityVersionChange(scheduled_version_change));
            }
        }
    }
}

impl<T: Config> FinalityCommitteeManager<T::AccountId> for Pallet<T> {
    fn on_next_session_finality_committee(committee: Vec<T::AccountId>) {
        NextFinalityCommittee::<T>::put(committee);
    }
}
