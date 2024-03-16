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

//! Traits for the Operations Module.

use frame_support::{traits::StoredMap, WeakBoundedVec};
use pallet_balances::BalanceLock;
use sp_runtime::traits::Zero;

pub trait AccountInfoProvider {
    type AccountId;
    type RefCount;

    fn get_consumers(who: &Self::AccountId) -> Self::RefCount;
}

impl<T> AccountInfoProvider for frame_system::Pallet<T>
where
    T: frame_system::Config,
{
    type AccountId = T::AccountId;
    type RefCount = frame_system::RefCount;

    fn get_consumers(who: &Self::AccountId) -> Self::RefCount {
        frame_system::Pallet::<T>::consumers(who)
    }
}

pub trait BalancesProvider {
    type AccountId;
    type Balance;
    type MaxLocks;

    fn is_reserved_not_zero(who: &Self::AccountId) -> bool;

    fn locks(who: &Self::AccountId) -> WeakBoundedVec<BalanceLock<Self::Balance>, Self::MaxLocks>;
}

impl<T: pallet_balances::Config<I>, I: 'static> BalancesProvider for pallet_balances::Pallet<T, I> {
    type AccountId = T::AccountId;
    type Balance = T::Balance;
    type MaxLocks = T::MaxLocks;

    fn is_reserved_not_zero(who: &Self::AccountId) -> bool {
        !T::AccountStore::get(who).reserved.is_zero()
    }

    fn locks(who: &Self::AccountId) -> WeakBoundedVec<BalanceLock<Self::Balance>, Self::MaxLocks> {
        pallet_balances::Locks::<T, I>::get(who)
    }
}

pub trait NextKeysSessionProvider {
    type AccountId;

    fn has_next_session_keys(who: &Self::AccountId) -> bool;
}

impl<T> NextKeysSessionProvider for pallet_session::Pallet<T>
where
    T: pallet_session::Config<ValidatorId = <T as frame_system::Config>::AccountId>,
{
    type AccountId = T::AccountId;

    fn has_next_session_keys(who: &Self::AccountId) -> bool {
        pallet_session::NextKeys::<T>::get(who).is_some()
    }
}
