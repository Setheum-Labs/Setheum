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

//! The Operations Module.

#![allow(clippy::nonminimal_bool)]

use frame_support::{
    dispatch::DispatchResultWithPostInfo, pallet_prelude::Get, traits::LockIdentifier,
    WeakBoundedVec,
};
use pallet_balances::BalanceLock;
use parity_scale_codec::Encode;
use sp_core::hexdisplay::HexDisplay;
use sp_runtime::DispatchError;

use crate::{
    pallet::{Config, Event, Pallet},
    traits::{AccountInfoProvider, BalancesProvider, NextKeysSessionProvider},
    LOG_TARGET, STAKING_ID, VESTING_ID,
};

impl<T: Config> Pallet<T> {
    /// Checks if account has an underflow of `consumers` counter. In such case, it increments
    /// it by one.
    pub fn fix_underflow_consumer_counter(who: T::AccountId) -> DispatchResultWithPostInfo {
        let mut weight = T::DbWeight::get().reads(1);
        let consumers = T::AccountInfoProvider::get_consumers(&who);

        weight += T::DbWeight::get().reads(1);
        if Self::no_consumers_some_reserved(&who, consumers) {
            Self::increment_consumers(who)?;
            weight += T::DbWeight::get().writes(1);
            return Ok(Some(weight).into());
        }

        weight += T::DbWeight::get().reads(2);
        if Self::staker_has_consumers_underflow(&who, consumers) {
            Self::increment_consumers(who)?;
            weight += T::DbWeight::get().writes(1);
            return Ok(Some(weight).into());
        }

        log::debug!(
            target: LOG_TARGET,
            "Account {:?} has correct consumer counter, not incrementing",
            HexDisplay::from(&who.encode())
        );
        Ok(Some(weight).into())
    }

    fn staker_has_consumers_underflow(who: &T::AccountId, consumers: u32) -> bool {
        let locks = T::BalancesProvider::locks(who);
        let has_vesting_lock = Self::has_lock(&locks, VESTING_ID);
        let vester_has_consumers_underflow = consumers == 1 && has_vesting_lock;
        let has_staking_lock = Self::has_lock(&locks, STAKING_ID);
        let nominator_has_consumers_underflow = consumers == 2 && has_staking_lock;
        let has_next_session_keys = T::NextKeysSessionProvider::has_next_session_keys(who);
        let validator_has_consumers_underflow =
            consumers == 3 && has_staking_lock && has_next_session_keys;
        vester_has_consumers_underflow
            || nominator_has_consumers_underflow
            || validator_has_consumers_underflow
    }

    fn no_consumers_some_reserved(who: &T::AccountId, consumers: u32) -> bool {
        let is_reserved_not_zero = T::BalancesProvider::is_reserved_not_zero(who);

        consumers == 0 && is_reserved_not_zero
    }

    fn has_lock<U, V>(locks: &WeakBoundedVec<BalanceLock<U>, V>, id: LockIdentifier) -> bool {
        locks.iter().any(|x| x.id == id)
    }

    fn increment_consumers(who: T::AccountId) -> Result<(), DispatchError> {
        frame_system::Pallet::<T>::inc_consumers_without_limit(&who)?;
        Self::deposit_event(Event::ConsumersUnderflowFixed { who });
        Ok(())
    }
}
