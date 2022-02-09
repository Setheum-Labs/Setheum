// بِسْمِ اللَّهِ الرَّحْمَنِ الرَّحِيم

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

//! # Vesting Module
//!
//! ## Overview
//!
//! Vesting module provides a means of scheduled balance lock on an account. It
//! uses the *graded vesting* way, which unlocks a specific amount of balance
//! every period of time, until all balance unlocked.
//!
//! ### Vesting Schedule
//!
//! The schedule of a vesting is described by data structure `VestingSchedule`:
//! from the block number of `start`, for every `period` amount of blocks,
//! `per_period` amount of balance would unlocked, until number of periods
//! `period_count` reached. Note in vesting schedules, *time* is measured by
//! block number. All `VestingSchedule`s under an account could be queried in
//! chain state.
//!
//! ## Interface
//!
//! ### Dispatchable Functions
//!
//! - `vested_transfer` - Add a new vesting schedule for an account.
//! - `claim` - Claim unlocked balances.
//! - `update_vesting_schedules` - Update all vesting schedules under an
//!   account, `root` origin required.

#![cfg_attr(not(feature = "std"), no_std)]
#![allow(clippy::unused_unit)]

use codec::{HasCompact, MaxEncodedLen};
use frame_support::{
	ensure,
	pallet_prelude::*,
	traits::{EnsureOrigin, Get},
	transactional, BoundedVec,
};
use frame_system::{ensure_signed, pallet_prelude::*};
use sp_runtime::{
	traits::{AtLeast32Bit, BlockNumberProvider, CheckedAdd, Saturating, StaticLookup, Zero},
	ArithmeticError, DispatchResult, RuntimeDebug,
};
use sp_std::{
	cmp::{Eq, PartialEq},
	convert::TryInto,
	vec::Vec,
};use orml_traits::{
	LockIdentifier, MultiCurrency, MultiLockableCurrency,
};
use primitives::CurrencyId;

mod mock;
mod tests;
mod weights;

pub use module::*;
pub use weights::WeightInfo;

pub const VESTING_LOCK_ID: LockIdentifier = *b"set/vest";

/// The vesting schedule.
///
/// Benefits would be granted gradually, `per_period` amount every `period`
/// of blocks after `start`.
#[derive(Clone, Encode, Decode, PartialEq, Eq, RuntimeDebug, MaxEncodedLen)]
pub struct VestingSchedule<BlockNumber, Balance: HasCompact> {
	/// Vesting starting block
	pub start: BlockNumber,
	/// Number of blocks between vest
	pub period: BlockNumber,
	/// Number of vest
	pub period_count: u32,
	/// Amount of tokens to release per vest
	#[codec(compact)]
	pub per_period: Balance,
}

impl<BlockNumber: AtLeast32Bit + Copy, Balance: AtLeast32Bit + Copy> VestingSchedule<BlockNumber, Balance> {
	/// Returns the end of all periods, `None` if calculation overflows.
	pub fn end(&self) -> Option<BlockNumber> {
		// period * period_count + start
		self.period
			.checked_mul(&self.period_count.into())?
			.checked_add(&self.start)
	}

	/// Returns all locked amount, `None` if calculation overflows.
	pub fn total_amount(&self) -> Option<Balance> {
		self.per_period.checked_mul(&self.period_count.into())
	}

	/// Returns locked amount for a given `time`.
	///
	/// Note this func assumes schedule is a valid one(non-zero period and
	/// non-overflow total amount), and it should be guaranteed by callers.
	pub fn locked_amount(&self, time: BlockNumber) -> Balance {
		// full = (time - start) / period
		// unrealized = period_count - full
		// per_period * unrealized
		let full = time
			.saturating_sub(self.start)
			.checked_div(&self.period)
			.expect("ensured non-zero period; qed");
		let unrealized = self.period_count.saturating_sub(full.unique_saturated_into());
		self.per_period
			.checked_mul(&unrealized.into())
			.expect("ensured non-overflow total amount; qed")
	}
}

#[frame_support::pallet]
pub mod module {
	use super::*;

	pub(crate) type BalanceOf<T> =
		<<T as Config>::MultiCurrency as MultiCurrency<<T as frame_system::Config>::AccountId>>::Balance;
	pub(crate) type CurrencyIdOf<T> =
		<<T as Config>::MultiCurrency as MultiCurrency<<T as frame_system::Config>::AccountId>>::CurrencyId;
	pub(crate) type VestingScheduleOf<T> = VestingSchedule<<T as frame_system::Config>::BlockNumber, BalanceOf<T>>;
	pub type ScheduledItem<T> = (
		<T as frame_system::Config>::AccountId,
		CurrencyIdOf<T>,
		<T as frame_system::Config>::BlockNumber,
		<T as frame_system::Config>::BlockNumber,
		u32,
		BalanceOf<T>,
	);

	#[pallet::config]
	pub trait Config: frame_system::Config {
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;

		type MultiCurrency: MultiLockableCurrency<Self::AccountId, CurrencyId = CurrencyId>;

		#[pallet::constant]
		/// Native Setheum (SETM) currency id. [P]Pronounced "set M" or "setem"
		/// 
		type GetNativeCurrencyId: Get<CurrencyId>;

		#[pallet::constant]
		/// Serp (SERP) currency id.
		/// 
		type GetSerpCurrencyId: Get<CurrencyId>;

		#[pallet::constant]
		/// The Dinar (DNAR) currency id.
		/// 
		type GetDinarCurrencyId: Get<CurrencyId>;

		#[pallet::constant]
		/// HighEnd LaunchPad (HELP) currency id. (LaunchPad Token)
		/// 
		type GetHelpCurrencyId: Get<CurrencyId>;

		#[pallet::constant]
		/// Setter (SETR) currency id
		/// 
		type SetterCurrencyId: Get<CurrencyId>;

		#[pallet::constant]
		/// The SetDollar (SETUSD) currency id
		type GetSetUSDId: Get<CurrencyId>;

		#[pallet::constant]
		/// The minimum amount transferred to call `vested_transfer`.
		type MinVestedTransfer: Get<BalanceOf<Self>>;

		/// SetheumTreasury account. For Vested Transfer
		#[pallet::constant]
		type TreasuryAccount: Get<Self::AccountId>;

		/// The origin which may update inflation related params
		type UpdateOrigin: EnsureOrigin<Self::Origin>;

		/// Weight information for extrinsics in this module.
		type WeightInfo: WeightInfo;

		/// The maximum vesting schedules for SETM
		type MaxNativeVestingSchedules: Get<u32>;

		/// The maximum vesting schedules for SERP
		type MaxSerpVestingSchedules: Get<u32>;

		/// The maximum vesting schedules for DNAR
		type MaxDinarVestingSchedules: Get<u32>;

		/// The maximum vesting schedules for HELP
		type MaxHelpVestingSchedules: Get<u32>;

		/// The maximum vesting schedules for SETR
		type MaxSetterVestingSchedules: Get<u32>;

		/// The maximum vesting schedules for SETUSD
		type MaxSetUSDVestingSchedules: Get<u32>;
	}

	#[pallet::error]
	pub enum Error<T> {
		/// Vesting period is zero
		ZeroVestingPeriod,
		/// Number of vests is zero
		ZeroVestingPeriodCount,
		/// Insufficient amount of balance to lock
		InsufficientBalanceToLock,
		/// This account have too many vesting schedules
		TooManyVestingSchedules,
		/// The vested transfer amount is too low
		AmountLow,
		/// Failed because the maximum vesting schedules was exceeded
		MaxVestingSchedulesExceeded,
	}

	#[pallet::event]
	#[pallet::generate_deposit(fn deposit_event)]
	#[pallet::metadata(T::AccountId = "AccountId", VestingScheduleOf<T> = "VestingScheduleOf", BalanceOf<T> = "Balance")]
	pub enum Event<T: Config> {
		/// Added new vesting schedule. \[currency_id, from, to, vesting_schedule\]
		VestingScheduleAdded(CurrencyIdOf<T>, T::AccountId, T::AccountId, VestingScheduleOf<T>),
		/// Claimed vesting. \[who, currency_id, locked_amount\]
		Claimed(T::AccountId, CurrencyIdOf<T>, BalanceOf<T>),
		/// Updated vesting schedules. \[currency_id, who\]
		VestingSchedulesUpdated(CurrencyIdOf<T>, T::AccountId),
	}

	/// Vesting schedules of an account under SETM currency.
	///
	/// NativeVestingSchedules: map AccountId => Vec<VestingSchedule>
	#[pallet::storage]
	#[pallet::getter(fn native_vesting_schedules)]
	pub type NativeVestingSchedules<T: Config> = StorageMap<
		_,
		Blake2_128Concat,
		T::AccountId,
		BoundedVec<VestingScheduleOf<T>, T::MaxNativeVestingSchedules>,
		ValueQuery,
	>;

	/// Vesting schedules of an account under SERP currency.
	///
	/// SerpVestingSchedules: map AccountId => Vec<VestingSchedule>
	#[pallet::storage]
	#[pallet::getter(fn serp_vesting_schedules)]
	pub type SerpVestingSchedules<T: Config> = StorageMap<
		_,
		Blake2_128Concat,
		T::AccountId,
		BoundedVec<VestingScheduleOf<T>, T::MaxSerpVestingSchedules>,
		ValueQuery,
	>;

	/// Vesting schedules of an account under DNAR currency.
	///
	/// DinarVestingSchedules: map AccountId => Vec<VestingSchedule>
	#[pallet::storage]
	#[pallet::getter(fn dinar_vesting_schedules)]
	pub type DinarVestingSchedules<T: Config> = StorageMap<
		_,
		Blake2_128Concat,
		T::AccountId,
		BoundedVec<VestingScheduleOf<T>, T::MaxDinarVestingSchedules>,
		ValueQuery,
	>;

	/// Vesting schedules of an account under HELP currency.
	///
	/// HelpVestingSchedules: map AccountId => Vec<VestingSchedule>
	#[pallet::storage]
	#[pallet::getter(fn help_vesting_schedules)]
	pub type HelpVestingSchedules<T: Config> = StorageMap<
		_,
		Blake2_128Concat,
		T::AccountId,
		BoundedVec<VestingScheduleOf<T>, T::MaxHelpVestingSchedules>,
		ValueQuery,
	>;

	/// Vesting schedules of an account under SETR currency.
	///
	/// SetterVestingSchedules: map AccountId => Vec<VestingSchedule>
	#[pallet::storage]
	#[pallet::getter(fn setter_vesting_schedules)]
	pub type SetterVestingSchedules<T: Config> = StorageMap<
		_,
		Blake2_128Concat,
		T::AccountId,
		BoundedVec<VestingScheduleOf<T>, T::MaxSetterVestingSchedules>,
		ValueQuery,
	>;

	/// Vesting schedules of an account under SETUSD currency.
	///
	/// SetUSDVestingSchedules: map AccountId => Vec<VestingSchedule>
	#[pallet::storage]
	#[pallet::getter(fn setusd_vesting_schedules)]
	pub type SetUSDVestingSchedules<T: Config> = StorageMap<
		_,
		Blake2_128Concat,
		T::AccountId,
		BoundedVec<VestingScheduleOf<T>, T::MaxSetUSDVestingSchedules>,
		ValueQuery,
	>;

	#[pallet::genesis_config]
	pub struct GenesisConfig<T: Config> {
		pub vesting: Vec<ScheduledItem<T>>,
	}

	#[cfg(feature = "std")]
	impl<T: Config> Default for GenesisConfig<T> {
		fn default() -> Self {
			GenesisConfig { vesting: vec![] }
		}
	}

	#[pallet::genesis_build]
	impl<T: Config> GenesisBuild<T> for GenesisConfig<T> {
		fn build(&self) {
			self.vesting
				.iter()
				.for_each(|(who, currency_id, start, period, period_count, per_period)| {
					if currency_id == &T::GetNativeCurrencyId::get() {
						let total = *per_period * Into::<BalanceOf<T>>::into(*period_count);
	
						let bounded_schedule: BoundedVec<VestingScheduleOf<T>, T::MaxNativeVestingSchedules> =
							vec![VestingSchedule {
								start: *start,
								period: *period,
								period_count: *period_count,
								per_period: *per_period,
							}]
							.try_into()
							.expect("Max vesting schedules exceeded");
	
						assert!(
							T::MultiCurrency::free_balance(T::GetNativeCurrencyId::get(), who) >= total,
							"Account do not have enough balance"
						);

						T::MultiCurrency::set_lock(VESTING_LOCK_ID, T::GetNativeCurrencyId::get(), who, total).unwrap();
						NativeVestingSchedules::<T>::insert(who, bounded_schedule);
					} else if currency_id == &T::GetSerpCurrencyId::get() {
						let total = *per_period * Into::<BalanceOf<T>>::into(*period_count);
	
						let bounded_schedule: BoundedVec<VestingScheduleOf<T>, T::MaxSerpVestingSchedules> =
							vec![VestingSchedule {
								start: *start,
								period: *period,
								period_count: *period_count,
								per_period: *per_period,
							}]
							.try_into()
							.expect("Max vesting schedules exceeded");
	
						assert!(
							T::MultiCurrency::free_balance(T::GetSerpCurrencyId::get(), who) >= total,
							"Account do not have enough balance"
						);
	
						T::MultiCurrency::set_lock(VESTING_LOCK_ID, T::GetSerpCurrencyId::get(), who, total).unwrap();
						SerpVestingSchedules::<T>::insert(who, bounded_schedule);
					} else if currency_id == &T::GetDinarCurrencyId::get() {
						let total = *per_period * Into::<BalanceOf<T>>::into(*period_count);
	
						let bounded_schedule: BoundedVec<VestingScheduleOf<T>, T::MaxDinarVestingSchedules> =
							vec![VestingSchedule {
								start: *start,
								period: *period,
								period_count: *period_count,
								per_period: *per_period,
							}]
							.try_into()
							.expect("Max vesting schedules exceeded");
	
						assert!(
							T::MultiCurrency::free_balance(T::GetDinarCurrencyId::get(), who) >= total,
							"Account do not have enough balance"
						);
	
						T::MultiCurrency::set_lock(VESTING_LOCK_ID, T::GetDinarCurrencyId::get(), who, total).unwrap();
						DinarVestingSchedules::<T>::insert(who, bounded_schedule);
					} else if currency_id == &T::GetHelpCurrencyId::get() {
						let total = *per_period * Into::<BalanceOf<T>>::into(*period_count);
	
						let bounded_schedule: BoundedVec<VestingScheduleOf<T>, T::MaxHelpVestingSchedules> =
							vec![VestingSchedule {
								start: *start,
								period: *period,
								period_count: *period_count,
								per_period: *per_period,
							}]
							.try_into()
							.expect("Max vesting schedules exceeded");
	
						assert!(
							T::MultiCurrency::free_balance(T::GetHelpCurrencyId::get(), who) >= total,
							"Account do not have enough balance"
						);
	
						T::MultiCurrency::set_lock(VESTING_LOCK_ID, T::GetHelpCurrencyId::get(), who, total).unwrap();
						HelpVestingSchedules::<T>::insert(who, bounded_schedule);
					} else if currency_id == &T::SetterCurrencyId::get() {
						let total = *per_period * Into::<BalanceOf<T>>::into(*period_count);
	
						let bounded_schedule: BoundedVec<VestingScheduleOf<T>, T::MaxSetterVestingSchedules> =
							vec![VestingSchedule {
								start: *start,
								period: *period,
								period_count: *period_count,
								per_period: *per_period,
							}]
							.try_into()
							.expect("Max vesting schedules exceeded");
	
						assert!(
							T::MultiCurrency::free_balance(T::SetterCurrencyId::get(), who) >= total,
							"Account do not have enough balance"
						);
	
						T::MultiCurrency::set_lock(VESTING_LOCK_ID, T::SetterCurrencyId::get(), who, total).unwrap();
						SetterVestingSchedules::<T>::insert(who, bounded_schedule);
					} else if currency_id == &T::GetSetUSDId::get() {
						let total = *per_period * Into::<BalanceOf<T>>::into(*period_count);
	
						let bounded_schedule: BoundedVec<VestingScheduleOf<T>, T::MaxSetUSDVestingSchedules> =
							vec![VestingSchedule {
								start: *start,
								period: *period,
								period_count: *period_count,
								per_period: *per_period,
							}]
							.try_into()
							.expect("Max vesting schedules exceeded");
	
						assert!(
							T::MultiCurrency::free_balance(T::GetSetUSDId::get(), who) >= total,
							"Account do not have enough balance"
						);
	
						T::MultiCurrency::set_lock(VESTING_LOCK_ID, T::GetSetUSDId::get(), who, total).unwrap();
						SetUSDVestingSchedules::<T>::insert(who, bounded_schedule);
					}
				});
		}
	}

	#[pallet::pallet]
	pub struct Pallet<T>(_);

	#[pallet::hooks]
	impl<T: Config> Hooks<T::BlockNumber> for Pallet<T> {}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		#[pallet::weight(T::WeightInfo::claim((<T as Config>::MaxNativeVestingSchedules::get() / 2) as u32))]
		pub fn claim(origin: OriginFor<T>, currency_id: CurrencyIdOf<T>) -> DispatchResult {
			let who = ensure_signed(origin)?;
			let locked_amount = Self::do_claim(currency_id, &who);

			Self::deposit_event(Event::Claimed(who, currency_id, locked_amount));
			Ok(())
		}

		#[pallet::weight(T::WeightInfo::vested_transfer())]
		pub fn vested_transfer(
			origin: OriginFor<T>,
			currency_id: CurrencyIdOf<T>,
			dest: <T::Lookup as StaticLookup>::Source,
			schedule: VestingScheduleOf<T>
		) -> DispatchResult {
			T::UpdateOrigin::ensure_origin(origin)?;
			let from = T::TreasuryAccount::get();
			let to = T::Lookup::lookup(dest)?;
			Self::do_vested_transfer(currency_id, &from, &to, schedule.clone())?;

			Self::deposit_event(Event::VestingScheduleAdded(currency_id, from, to, schedule));
			Ok(())
		}

		#[pallet::weight(T::WeightInfo::update_vesting_schedules(vesting_schedules.len() as u32))]
		pub fn update_vesting_schedules(
			origin: OriginFor<T>,
			currency_id: CurrencyIdOf<T>,
			who: <T::Lookup as StaticLookup>::Source,
			vesting_schedules: Vec<VestingScheduleOf<T>>,
		) -> DispatchResult {
			T::UpdateOrigin::ensure_origin(origin)?;

			let account = T::Lookup::lookup(who)?;
			Self::do_update_vesting_schedules(currency_id, &account, vesting_schedules)?;

			Self::deposit_event(Event::VestingSchedulesUpdated(currency_id, account));
			Ok(())
		}

		#[pallet::weight(T::WeightInfo::claim((<T as Config>::MaxNativeVestingSchedules::get() / 2) as u32))]
		pub fn claim_for(
			origin: OriginFor<T>,
			currency_id: CurrencyIdOf<T>,
			dest: <T::Lookup as StaticLookup>::Source
		) -> DispatchResult {
			let _ = ensure_signed(origin)?;
			let who = T::Lookup::lookup(dest)?;
			let locked_amount = Self::do_claim(currency_id, &who);

			Self::deposit_event(Event::Claimed(who, currency_id, locked_amount));
			Ok(())
		}
	}
}

impl<T: Config> BlockNumberProvider for Pallet<T> {
	type BlockNumber = <T as frame_system::Config>::BlockNumber;

	fn current_block_number() -> Self::BlockNumber {
		<frame_system::Pallet<T>>::block_number()
	}
}

impl<T: Config> Pallet<T> {
	fn do_claim(currency_id: CurrencyIdOf<T>, who: &T::AccountId) -> BalanceOf<T> {
		let locked = Self::locked_balance(currency_id, who);
		if locked.is_zero() {
			if currency_id == T::GetNativeCurrencyId::get() {
				// cleanup the storage and unlock the fund
				<NativeVestingSchedules<T>>::remove(who);
				T::MultiCurrency::remove_lock(VESTING_LOCK_ID, currency_id, who).unwrap();
			} else if currency_id == T::GetSerpCurrencyId::get() {
				// cleanup the storage and unlock the fund
				<SerpVestingSchedules<T>>::remove(who);
				T::MultiCurrency::remove_lock(VESTING_LOCK_ID, currency_id, who).unwrap();
			} else if currency_id == T::GetDinarCurrencyId::get() {
				// cleanup the storage and unlock the fund
				<DinarVestingSchedules<T>>::remove(who);
				T::MultiCurrency::remove_lock(VESTING_LOCK_ID, currency_id, who).unwrap();
			} else if currency_id == T::GetHelpCurrencyId::get() {
				// cleanup the storage and unlock the fund
				<HelpVestingSchedules<T>>::remove(who);
				T::MultiCurrency::remove_lock(VESTING_LOCK_ID, currency_id, who).unwrap();
			} else if currency_id == T::SetterCurrencyId::get() {
				// cleanup the storage and unlock the fund
				<SetterVestingSchedules<T>>::remove(who);
				T::MultiCurrency::remove_lock(VESTING_LOCK_ID, currency_id, who).unwrap();
			} else if currency_id == T::GetSetUSDId::get() {
				// cleanup the storage and unlock the fund
				<SetUSDVestingSchedules<T>>::remove(who);
				T::MultiCurrency::remove_lock(VESTING_LOCK_ID, currency_id, who).unwrap();
			}
		} else {
			T::MultiCurrency::set_lock(VESTING_LOCK_ID, currency_id, who, locked).unwrap();
		}
		locked
	}

	/// Returns locked balance based on current block number.
	fn locked_balance(currency_id: CurrencyIdOf<T>, who: &T::AccountId) -> BalanceOf<T> {
		let now = <Self as BlockNumberProvider>::current_block_number();
		if currency_id == T::GetNativeCurrencyId::get() {
			// cleanup the storage and unlock the fund
			<NativeVestingSchedules<T>>::mutate_exists(who, |maybe_schedules| {
				let total = if let Some(schedules) = maybe_schedules.as_mut() {
					let mut total: BalanceOf<T> = Zero::zero();
					schedules.retain(|s| {
						let amount = s.locked_amount(now);
						total = total.saturating_add(amount);
						!amount.is_zero()
					});
					total
				} else {
					Zero::zero()
				};
				if total.is_zero() {
					*maybe_schedules = None;
				}
				return total
			})
		} else if currency_id == T::GetSerpCurrencyId::get() {
			// cleanup the storage and unlock the fund
			<SerpVestingSchedules<T>>::mutate_exists(who, |maybe_schedules| {
				let total = if let Some(schedules) = maybe_schedules.as_mut() {
					let mut total: BalanceOf<T> = Zero::zero();
					schedules.retain(|s| {
						let amount = s.locked_amount(now);
						total = total.saturating_add(amount);
						!amount.is_zero()
					});
					total
				} else {
					Zero::zero()
				};
				if total.is_zero() {
					*maybe_schedules = None;
				}
				return total
			})
		} else if currency_id == T::GetDinarCurrencyId::get() {
			// cleanup the storage and unlock the fund
			<DinarVestingSchedules<T>>::mutate_exists(who, |maybe_schedules| {
				let total = if let Some(schedules) = maybe_schedules.as_mut() {
					let mut total: BalanceOf<T> = Zero::zero();
					schedules.retain(|s| {
						let amount = s.locked_amount(now);
						total = total.saturating_add(amount);
						!amount.is_zero()
					});
					total
				} else {
					Zero::zero()
				};
				if total.is_zero() {
					*maybe_schedules = None;
				}
				return total
			})
		} else if currency_id == T::GetHelpCurrencyId::get() {
			// cleanup the storage and unlock the fund
			<HelpVestingSchedules<T>>::mutate_exists(who, |maybe_schedules| {
				let total = if let Some(schedules) = maybe_schedules.as_mut() {
					let mut total: BalanceOf<T> = Zero::zero();
					schedules.retain(|s| {
						let amount = s.locked_amount(now);
						total = total.saturating_add(amount);
						!amount.is_zero()
					});
					total
				} else {
					Zero::zero()
				};
				if total.is_zero() {
					*maybe_schedules = None;
				}
				return total
			})
		} else if currency_id == T::SetterCurrencyId::get() {
			// cleanup the storage and unlock the fund
			<SetterVestingSchedules<T>>::mutate_exists(who, |maybe_schedules| {
				let total = if let Some(schedules) = maybe_schedules.as_mut() {
					let mut total: BalanceOf<T> = Zero::zero();
					schedules.retain(|s| {
						let amount = s.locked_amount(now);
						total = total.saturating_add(amount);
						!amount.is_zero()
					});
					total
				} else {
					Zero::zero()
				};
				if total.is_zero() {
					*maybe_schedules = None;
				}
				return total
			})
		} else if currency_id == T::GetSetUSDId::get() {
			// cleanup the storage and unlock the fund
			<SetUSDVestingSchedules<T>>::mutate_exists(who, |maybe_schedules| {
				let total = if let Some(schedules) = maybe_schedules.as_mut() {
					let mut total: BalanceOf<T> = Zero::zero();
					schedules.retain(|s| {
						let amount = s.locked_amount(now);
						total = total.saturating_add(amount);
						!amount.is_zero()
					});
					total
				} else {
					Zero::zero()
				};
				if total.is_zero() {
					*maybe_schedules = None;
				}
				return total
			})
		} else {
			Zero::zero()
		}
	}

	#[transactional]
	fn do_vested_transfer(
		currency_id: CurrencyIdOf<T>,
		from: &T::AccountId,
		to: &T::AccountId,
		schedule: VestingScheduleOf<T>
	) -> DispatchResult {
		let schedule_amount = Self::ensure_valid_vesting_schedule(&schedule)?;

		let total_amount = Self::locked_balance(currency_id, to)
			.checked_add(&schedule_amount)
			.ok_or(ArithmeticError::Overflow)?;

		if currency_id == T::GetNativeCurrencyId::get() {
			T::MultiCurrency::transfer(currency_id, from, to, schedule_amount)?;
			T::MultiCurrency::set_lock(VESTING_LOCK_ID, currency_id, to, total_amount)?;
			<NativeVestingSchedules<T>>::try_append(to, schedule).map_err(|_| Error::<T>::MaxVestingSchedulesExceeded)?;
		} else if currency_id == T::GetSerpCurrencyId::get() {
			T::MultiCurrency::transfer(currency_id, from, to, schedule_amount)?;
			T::MultiCurrency::set_lock(VESTING_LOCK_ID, currency_id, to, total_amount)?;
			<SerpVestingSchedules<T>>::try_append(to, schedule).map_err(|_| Error::<T>::MaxVestingSchedulesExceeded)?;
		} else if currency_id == T::GetDinarCurrencyId::get() {
			T::MultiCurrency::transfer(currency_id, from, to, schedule_amount)?;
			T::MultiCurrency::set_lock(VESTING_LOCK_ID, currency_id, to, total_amount)?;
			<DinarVestingSchedules<T>>::try_append(to, schedule).map_err(|_| Error::<T>::MaxVestingSchedulesExceeded)?;
		} else if currency_id == T::GetHelpCurrencyId::get() {
			T::MultiCurrency::transfer(currency_id, from, to, schedule_amount)?;
			T::MultiCurrency::set_lock(VESTING_LOCK_ID, currency_id, to, total_amount)?;
			<HelpVestingSchedules<T>>::try_append(to, schedule).map_err(|_| Error::<T>::MaxVestingSchedulesExceeded)?;
		} else if currency_id == T::SetterCurrencyId::get() {
			T::MultiCurrency::transfer(currency_id, from, to, schedule_amount)?;
			T::MultiCurrency::set_lock(VESTING_LOCK_ID, currency_id, to, total_amount)?;
			<SetterVestingSchedules<T>>::try_append(to, schedule).map_err(|_| Error::<T>::MaxVestingSchedulesExceeded)?;
		} else if currency_id == T::GetSetUSDId::get() {
			T::MultiCurrency::transfer(currency_id, from, to, schedule_amount)?;
			T::MultiCurrency::set_lock(VESTING_LOCK_ID, currency_id, to, total_amount)?;
			<SetUSDVestingSchedules<T>>::try_append(to, schedule).map_err(|_| Error::<T>::MaxVestingSchedulesExceeded)?;
		};
		Ok(())
	}

	fn do_update_vesting_schedules(
		currency_id: CurrencyIdOf<T>,
		who: &T::AccountId,
		schedules: Vec<VestingScheduleOf<T>>
	) -> DispatchResult {
		if currency_id == T::GetNativeCurrencyId::get() {
			let bounded_schedules: BoundedVec<VestingScheduleOf<T>, T::MaxNativeVestingSchedules> = schedules
				.try_into()
				.map_err(|_| Error::<T>::MaxVestingSchedulesExceeded)?;
	
			// empty vesting schedules cleanup the storage and unlock the fund
			if bounded_schedules.len().is_zero() {
				<NativeVestingSchedules<T>>::remove(who);
				T::MultiCurrency::remove_lock(VESTING_LOCK_ID, currency_id, who).unwrap();
				return Ok(());
			}
	
			let total_amount = bounded_schedules
				.iter()
				.try_fold::<_, _, Result<BalanceOf<T>, DispatchError>>(Zero::zero(), |acc_amount, schedule| {
					let amount = Self::ensure_valid_vesting_schedule(schedule)?;
					Ok(acc_amount + amount)
				})?;
			ensure!(
				T::MultiCurrency::free_balance(currency_id, who) >= total_amount,
				Error::<T>::InsufficientBalanceToLock,
			);
	
			T::MultiCurrency::set_lock(VESTING_LOCK_ID, currency_id, who, total_amount)?;
			<NativeVestingSchedules<T>>::insert(who, bounded_schedules);
		} else if currency_id == T::GetSerpCurrencyId::get() {
			let bounded_schedules: BoundedVec<VestingScheduleOf<T>, T::MaxSerpVestingSchedules> = schedules
				.try_into()
				.map_err(|_| Error::<T>::MaxVestingSchedulesExceeded)?;
	
			// empty vesting schedules cleanup the storage and unlock the fund
			if bounded_schedules.len().is_zero() {
				<SerpVestingSchedules<T>>::remove(who);
				T::MultiCurrency::remove_lock(VESTING_LOCK_ID, currency_id, who).unwrap();
				return Ok(());
			}
	
			let total_amount = bounded_schedules
				.iter()
				.try_fold::<_, _, Result<BalanceOf<T>, DispatchError>>(Zero::zero(), |acc_amount, schedule| {
					let amount = Self::ensure_valid_vesting_schedule(schedule)?;
					Ok(acc_amount + amount)
				})?;
			ensure!(
				T::MultiCurrency::free_balance(currency_id, who) >= total_amount,
				Error::<T>::InsufficientBalanceToLock,
			);
	
			T::MultiCurrency::set_lock(VESTING_LOCK_ID, currency_id, who, total_amount)?;
			<SerpVestingSchedules<T>>::insert(who, bounded_schedules);
		} else if currency_id == T::GetDinarCurrencyId::get() {
			let bounded_schedules: BoundedVec<VestingScheduleOf<T>, T::MaxDinarVestingSchedules> = schedules
				.try_into()
				.map_err(|_| Error::<T>::MaxVestingSchedulesExceeded)?;
	
			// empty vesting schedules cleanup the storage and unlock the fund
			if bounded_schedules.len().is_zero() {
				<DinarVestingSchedules<T>>::remove(who);
				T::MultiCurrency::remove_lock(VESTING_LOCK_ID, currency_id, who).unwrap();
				return Ok(());
			}
	
			let total_amount = bounded_schedules
				.iter()
				.try_fold::<_, _, Result<BalanceOf<T>, DispatchError>>(Zero::zero(), |acc_amount, schedule| {
					let amount = Self::ensure_valid_vesting_schedule(schedule)?;
					Ok(acc_amount + amount)
				})?;
			ensure!(
				T::MultiCurrency::free_balance(currency_id, who) >= total_amount,
				Error::<T>::InsufficientBalanceToLock,
			);
	
			T::MultiCurrency::set_lock(VESTING_LOCK_ID, currency_id, who, total_amount)?;
			<DinarVestingSchedules<T>>::insert(who, bounded_schedules);
		} else if currency_id == T::GetHelpCurrencyId::get() {
			let bounded_schedules: BoundedVec<VestingScheduleOf<T>, T::MaxHelpVestingSchedules> = schedules
				.try_into()
				.map_err(|_| Error::<T>::MaxVestingSchedulesExceeded)?;
	
			// empty vesting schedules cleanup the storage and unlock the fund
			if bounded_schedules.len().is_zero() {
				<HelpVestingSchedules<T>>::remove(who);
				T::MultiCurrency::remove_lock(VESTING_LOCK_ID, currency_id, who).unwrap();
				return Ok(());
			}
	
			let total_amount = bounded_schedules
				.iter()
				.try_fold::<_, _, Result<BalanceOf<T>, DispatchError>>(Zero::zero(), |acc_amount, schedule| {
					let amount = Self::ensure_valid_vesting_schedule(schedule)?;
					Ok(acc_amount + amount)
				})?;
			ensure!(
				T::MultiCurrency::free_balance(currency_id, who) >= total_amount,
				Error::<T>::InsufficientBalanceToLock,
			);
	
			T::MultiCurrency::set_lock(VESTING_LOCK_ID, currency_id, who, total_amount)?;
			<HelpVestingSchedules<T>>::insert(who, bounded_schedules);
		} else if currency_id == T::SetterCurrencyId::get() {
			let bounded_schedules: BoundedVec<VestingScheduleOf<T>, T::MaxSetterVestingSchedules> = schedules
				.try_into()
				.map_err(|_| Error::<T>::MaxVestingSchedulesExceeded)?;
	
			// empty vesting schedules cleanup the storage and unlock the fund
			if bounded_schedules.len().is_zero() {
				<SetterVestingSchedules<T>>::remove(who);
				T::MultiCurrency::remove_lock(VESTING_LOCK_ID, currency_id, who).unwrap();
				return Ok(());
			}
	
			let total_amount = bounded_schedules
				.iter()
				.try_fold::<_, _, Result<BalanceOf<T>, DispatchError>>(Zero::zero(), |acc_amount, schedule| {
					let amount = Self::ensure_valid_vesting_schedule(schedule)?;
					Ok(acc_amount + amount)
				})?;
			ensure!(
				T::MultiCurrency::free_balance(currency_id, who) >= total_amount,
				Error::<T>::InsufficientBalanceToLock,
			);
	
			T::MultiCurrency::set_lock(VESTING_LOCK_ID, currency_id, who, total_amount)?;
			<SetterVestingSchedules<T>>::insert(who, bounded_schedules);
		} else if currency_id == T::GetSetUSDId::get() {
			let bounded_schedules: BoundedVec<VestingScheduleOf<T>, T::MaxSetUSDVestingSchedules> = schedules
				.try_into()
				.map_err(|_| Error::<T>::MaxVestingSchedulesExceeded)?;
	
			// empty vesting schedules cleanup the storage and unlock the fund
			if bounded_schedules.len().is_zero() {
				<SetUSDVestingSchedules<T>>::remove(who);
				T::MultiCurrency::remove_lock(VESTING_LOCK_ID, currency_id, who).unwrap();
				return Ok(());
			}
	
			let total_amount = bounded_schedules
				.iter()
				.try_fold::<_, _, Result<BalanceOf<T>, DispatchError>>(Zero::zero(), |acc_amount, schedule| {
					let amount = Self::ensure_valid_vesting_schedule(schedule)?;
					Ok(acc_amount + amount)
				})?;
			ensure!(
				T::MultiCurrency::free_balance(currency_id, who) >= total_amount,
				Error::<T>::InsufficientBalanceToLock,
			);
	
			T::MultiCurrency::set_lock(VESTING_LOCK_ID, currency_id, who, total_amount)?;
			<SetUSDVestingSchedules<T>>::insert(who, bounded_schedules);
		};
		Ok(())
	}

	/// Returns `Ok(amount)` if valid schedule, or error.
	fn ensure_valid_vesting_schedule(schedule: &VestingScheduleOf<T>) -> Result<BalanceOf<T>, DispatchError> {
		ensure!(!schedule.period.is_zero(), Error::<T>::ZeroVestingPeriod);
		ensure!(!schedule.period_count.is_zero(), Error::<T>::ZeroVestingPeriodCount);
		ensure!(schedule.end().is_some(), ArithmeticError::Overflow);

		let total = schedule.total_amount().ok_or(ArithmeticError::Overflow)?;

		ensure!(total >= T::MinVestedTransfer::get(), Error::<T>::AmountLow);

		Ok(total)
	}
}