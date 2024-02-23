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

use frame_support::{
	ensure,
	pallet_prelude::*,
	traits::{Currency, EnsureOrigin, ExistenceRequirement, Get, LockIdentifier, LockableCurrency, WithdrawReasons},
	BoundedVec,
};
use frame_system::{ensure_root, ensure_signed, pallet_prelude::*};
use parity_scale_codec::{HasCompact, MaxEncodedLen};
use scale_info::TypeInfo;
use sp_runtime::{
	traits::{AtLeast32Bit, BlockNumberProvider, CheckedAdd, Saturating, StaticLookup, Zero},
	ArithmeticError, DispatchResult, RuntimeDebug,
};
use sp_std::{
	cmp::{Eq, PartialEq},
	vec::Vec,
};
use orml_traits::{
	LockIdentifier, MultiCurrency, MultiLockableCurrency,
};
use primitives::CurrencyId;

mod mock;
mod tests;
mod weights;

pub use module::*;
pub use weights::WeightInfo;

pub const VESTING_LOCK_ID: LockIdentifier = *b"setvest";

/// The vesting schedule.
///
/// Benefits would be granted gradually, `per_period` amount every `period`
/// of blocks after `start`.
#[derive(Clone, Encode, Decode, PartialEq, Eq, RuntimeDebug, MaxEncodedLen, TypeInfo)]
pub struct VestingSchedule<BlockNumber, Balance: MaxEncodedLen + HasCompact> {
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

impl<BlockNumber: AtLeast32Bit + Copy, Balance: AtLeast32Bit + MaxEncodedLen + Copy>
	VestingSchedule<BlockNumber, Balance>
{
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
	pub(crate) type VestingScheduleOf<T> = VestingSchedule<BlockNumberFor<T>, BalanceOf<T>>;
	pub type ScheduledItem<T> = (
		<T as frame_system::Config>::AccountId,
		CurrencyIdOf<T>,
		BlockNumberFor<T>,
		BlockNumberFor<T>,
		u32,
		BalanceOf<T>,
	);

	#[pallet::config]
	pub trait Config: frame_system::Config {
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;

		type MultiCurrency: MultiLockableCurrency<Self::AccountId,  CurrencyId = CurrencyId, Moment = BlockNumberFor<Self>>;
		
		#[pallet::constant]
		/// Native Setheum (SEE) currency id.
		type GetNativeCurrencyId: Get<CurrencyId>;

		#[pallet::constant]
		/// Ethical DeFi (EDF) currency id.
		type GetEDFCurrencyId: Get<CurrencyId>;

		#[pallet::constant]
		/// The minimum amount of SEE transferred to call `vested_transfer`.
		type MinNativeVestedTransfer: Get<BalanceOf<Self>>;

		#[pallet::constant]
		/// The minimum amount of EDF transferred to call `vested_transfer`.
		type MinEDFVestedTransfer: Get<BalanceOf<Self>>;

		/// Required origin for vested transfer.
		type VestedTransferOrigin: EnsureOrigin<Self::RuntimeOrigin, Success = Self::AccountId>;

		/// Weight information for extrinsics in this module.
		type WeightInfo: WeightInfo;

		/// The maximum vesting schedules for SEE
		type MaxNativeVestingSchedules: Get<u32>;

		/// The maximum vesting schedules for EDF
		type MaxEDFVestingSchedules: Get<u32>;

		// The block number provider
		type BlockNumberProvider: BlockNumberProvider<BlockNumber = BlockNumberFor<Self>>;
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
		/// Failed because the maximum vesting schedules for SEE was exceeded
		MaxNativeVestingSchedulesExceeded,
		/// Failed because the maximum vesting schedules for EDF was exceeded
		MaxEDFVestingSchedulesExceeded,
	}

	#[pallet::event]
	#[pallet::generate_deposit(fn deposit_event)]
	pub enum Event<T: Config> {
		/// Added new vesting schedule.
		VestingScheduleAdded {
			currency_id: CurrencyIdOf<T>,
			from: T::AccountId,
			to: T::AccountId,
			vesting_schedule: VestingScheduleOf<T>,
		},
		/// Claimed vesting.
		Claimed { currency_id: CurrencyIdOf<T>, who: T::AccountId, amount: BalanceOf<T> },
		/// Updated vesting schedules.
		VestingSchedulesUpdated {currency_id: CurrencyIdOf<T>, who: T::AccountId },
	}

	/// Vesting schedules of an account under Native Currency (SEE).
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
	
	/// Vesting schedules of an account under EDF Currency.
	///
	/// EDFVestingSchedules: map AccountId => Vec<VestingSchedule>
	#[pallet::storage]
	#[pallet::getter(fn edf_vesting_schedules)]
	pub type EDFVestingSchedules<T: Config> = StorageMap<
		_,
		Blake2_128Concat,
		T::AccountId,
		BoundedVec<VestingScheduleOf<T>, T::MaxEDFVestingSchedules>,
		ValueQuery,
	>;
	

	#[pallet::genesis_config]
	pub struct GenesisConfig<T: Config> {
		pub vesting: Vec<ScheduledItem<T>>,
	}

	impl<T: Config> Default for GenesisConfig<T> {
		fn default() -> Self {
			GenesisConfig {
				vesting: Default::default(),
			}
		}
	}

	#[pallet::genesis_build]
	impl<T: Config> BuildGenesisConfig for GenesisConfig<T> {
		fn build(&self) {
			self.vesting
				.iter()
				.for_each(|(who, currency_id, start, period, period_count, per_period)| {
					if currency_id == &T::GetNativeCurrencyId::get() {
						let mut bounded_schedules = VestingSchedules::<T>::get(who);
						bounded_schedules
							.try_push(VestingSchedule {
								start: *start,
								period: *period,
								period_count: *period_count,
								per_period: *per_period,
							})
							.expect("Max vesting schedules exceeded");
						let total_amount = bounded_schedules
							.iter()
							.try_fold::<_, _, Result<BalanceOf<T>, DispatchError>>(Zero::zero(), |acc_amount, schedule| {
								let amount = ensure_valid_vesting_schedule::<T>(T::GetNativeCurrencyId::get(), schedule)?;
								acc_amount
									.checked_add(&amount)
									.ok_or_else(|| ArithmeticError::Overflow.into())
							})
							.expect("Invalid vesting schedule");

						}
						
						assert!(
							T::MultiCurrency::free_balance(T::GetNativeCurrencyId::get(), who) >= total_amount,
							"Account does not have enough balance"
						);

						T::MultiCurrency::set_lock(VESTING_LOCK_ID, T::GetNativeCurrencyId::get(), who, total_amount);
						NativeVestingSchedules::<T>::insert(who, bounded_schedules);
					} else if currency_id == &T::GetEDFCurrencyId::get() {
						let mut bounded_schedules = VestingSchedules::<T>::get(who);
						bounded_schedules
							.try_push(VestingSchedule {
								start: *start,
								period: *period,
								period_count: *period_count,
								per_period: *per_period,
							})
							.expect("Max vesting schedules exceeded");
						let total_amount = bounded_schedules
							.iter()
							.try_fold::<_, _, Result<BalanceOf<T>, DispatchError>>(Zero::zero(), |acc_amount, schedule| {
								let amount = ensure_valid_vesting_schedule::<T>(T::GetEDFCurrencyId::get(), schedule)?;
								acc_amount
									.checked_add(&amount)
									.ok_or_else(|| ArithmeticError::Overflow.into())
							})
							.expect("Invalid vesting schedule");

						}
						
						assert!(
							T::MultiCurrency::free_balance(T::GetEDFCurrencyId::get(), who) >= total_amount,
							"Account does not have enough balance"
						);

						T::MultiCurrency::set_lock(VESTING_LOCK_ID, T::GetEDFCurrencyId::get(), who, total_amount);
						EDFVestingSchedules::<T>::insert(who, bounded_schedules);
					}
				});
		}
	}

	#[pallet::pallet]
	pub struct Pallet<T>(_);

	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		#[pallet::call_index(0)]
		#[pallet::weight(T::WeightInfo::claim(<T as Config>::MaxVestingSchedules::get() / 2))]
		pub fn claim(origin: OriginFor<T>, currency_id: CurrencyIdOf<T>) -> DispatchResult {
			let who = ensure_signed(origin)?;
			let locked_amount = Self::do_claim(currency_id, &who);

			Self::deposit_event(Event::Claimed {
				currency_id,
				who,
				amount: locked_amount,
			});
			Ok(())
		}

		#[pallet::call_index(1)]
		#[pallet::weight(T::WeightInfo::vested_transfer())]
		pub fn vested_transfer(
			origin: OriginFor<T>,
			currency_id: CurrencyIdOf<T>,
			dest: <T::Lookup as StaticLookup>::Source,
			schedule: VestingScheduleOf<T>,
		) -> DispatchResult {
			let from = T::VestedTransferOrigin::ensure_origin(origin)?;
			let to = T::Lookup::lookup(dest)?;

			if to == from {
				ensure!(
					T::MultiCurrency::free_balance(currency_id, &from) >= schedule.total_amount().ok_or(ArithmeticError::Overflow)?,
					Error::<T>::InsufficientBalanceToLock,
				);
			}

			Self::do_vested_transfer(currency_id, &from, &to, schedule.clone())?;

			Self::deposit_event(Event::VestingScheduleAdded {
				currency_id,
				from,
				to,
				vesting_schedule: schedule,
			});
			Ok(())
		}

		#[pallet::call_index(2)]
		#[pallet::weight(T::WeightInfo::update_vesting_schedules(vesting_schedules.len() as u32))]
		pub fn update_vesting_schedules(
			origin: OriginFor<T>,
			currency_id: CurrencyIdOf<T>,
			who: <T::Lookup as StaticLookup>::Source,
			vesting_schedules: Vec<VestingScheduleOf<T>>,
		) -> DispatchResult {
			ensure_root(origin)?;

			let account = T::Lookup::lookup(who)?;
			Self::do_update_vesting_schedules(currency_id, &account, vesting_schedules)?;

			Self::deposit_event(Event::VestingSchedulesUpdated { currency_id, who: account });
			Ok(())
		}

		#[pallet::call_index(3)]
		#[pallet::weight(T::WeightInfo::claim(<T as Config>::MaxVestingSchedules::get() / 2))]
		pub fn claim_for(
			origin: OriginFor<T>,
			currency_id: CurrencyIdOf<T>,
			dest: <T::Lookup as StaticLookup>::Source
		) -> DispatchResult {
			let _ = ensure_signed(origin)?;
			let who = T::Lookup::lookup(dest)?;
			let locked_amount = Self::do_claim(currency_id, &who);

			Self::deposit_event(Event::Claimed {
				currency_id,
				who,
				amount: locked_amount,
			});
			Ok(())
		}
	}
}

impl<T: Config> Pallet<T> {
	fn do_claim(currency_id: CurrencyIdOf<T>, who: &T::AccountId) -> BalanceOf<T> {
		let locked = Self::locked_balance(who);
		if locked.is_zero() {
			if currency_id == T::GetNativeCurrencyId::get() {
				// cleanup the storage and unlock the fund
				<NativeVestingSchedules<T>>::remove(who);
				T::MultiCurrency::remove_lock(VESTING_LOCK_ID, currency_id, who);
			} else if currency_id == T::GetEDFCurrencyId::get() {
				// cleanup the storage and unlock the fund
				<EDFVestingSchedules<T>>::remove(who);
				T::MultiCurrency::remove_lock(VESTING_LOCK_ID, currency_id, who);
			}
		} else {
			T::MultiCurrency::set_lock(VESTING_LOCK_ID, currency_id, who, locked);
		}
		locked
	}

	/// Returns locked balance based on current block number.
	fn locked_balance(currency_id: CurrencyIdOf<T>, who: &T::AccountId) -> BalanceOf<T> {
		let now = T::BlockNumberProvider::current_block_number();
		if currency_id == T::GetNativeCurrencyId::get() {
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
				total
			})
		} else if currency_id == T::GetEDFCurrencyId::get() {
			<EDFVestingSchedules<T>>::mutate_exists(who, |maybe_schedules| {
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
				total
			})
		}
	}

	fn do_vested_transfer(
		currency_id: CurrencyIdOf<T>,
		from: &T::AccountId,
		to: &T::AccountId,
		schedule: VestingScheduleOf<T>
	) -> DispatchResult {
		if currency_id == T::GetNativeCurrencyId::get() {
			let schedule_amount = ensure_valid_vesting_schedule::<T>(T::GetNativeCurrencyId::get(), &schedule)?;

			let total_amount = Self::locked_balance(T::GetNativeCurrencyId::get(), to)
				.checked_add(&schedule_amount)
				.ok_or(ArithmeticError::Overflow)?;

			T::MultiCurrency::transfer(T::GetNativeCurrencyId::get(), from, to, schedule_amount)?;
			T::MultiCurrency::set_lock(VESTING_LOCK_ID, T::GetNativeCurrencyId::get(), to, total_amount)?;
			<NativeVestingSchedules<T>>::try_append(to, schedule).map_err(|_| Error::<T>::MaxNativeVestingSchedulesExceeded)?;
		} else if currency_id == T::GetEDFCurrencyId::get() {
			let schedule_amount = ensure_valid_vesting_schedule::<T>(T::GetEDFCurrencyId::get(), &schedule)?;

			let total_amount = Self::locked_balance(T::GetEDFCurrencyId::get(), to)
				.checked_add(&schedule_amount)
				.ok_or(ArithmeticError::Overflow)?;

			T::MultiCurrency::transfer(T::GetEDFCurrencyId::get(), from, to, schedule_amount)?;
			T::MultiCurrency::set_lock(VESTING_LOCK_ID, T::GetEDFCurrencyId::get(), to, total_amount)?;
			<EDFVestingSchedules<T>>::try_append(to, schedule).map_err(|_| Error::<T>::MaxEDFVestingSchedulesExceeded)?;
		}
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
				.map_err(|_| Error::<T>::MaxNativeVestingSchedulesExceeded)?;

			// empty vesting schedules cleanup the storage and unlock the fund
			if bounded_schedules.len().is_zero() {
				<NativeVestingSchedules<T>>::remove(who);
				T::MultiCurrency::remove_lock(VESTING_LOCK_ID,T::GetNativeCurrencyId::get(), who)?;
				return Ok(());
			}

			let total_amount = bounded_schedules
				.iter()
				.try_fold::<_, _, Result<BalanceOf<T>, DispatchError>>(Zero::zero(), |acc_amount, schedule| {
					let amount = ensure_valid_vesting_schedule::<T>(T::GetNativeCurrencyId::get(), schedule)?;
					acc_amount
						.checked_add(&amount)
						.ok_or_else(|| ArithmeticError::Overflow.into())
				})?;
			ensure!(
				T::MultiCurrency::free_balance(T::GetNativeCurrencyId::get(), who) >= total_amount,
				Error::<T>::InsufficientBalanceToLock,
			);

			T::MultiCurrency::set_lock(VESTING_LOCK_ID, T::GetNativeCurrencyId::get(), who, total_amount);
			<NativeVestingSchedules<T>>::insert(who, bounded_schedules);
		} else if currency_id == T::GetEDFCurrencyId::get() {
			let bounded_schedules: BoundedVec<VestingScheduleOf<T>, T::MaxEDFVestingSchedules> = schedules
				.try_into()
				.map_err(|_| Error::<T>::MaxEDFVestingSchedulesExceeded)?;

			// empty vesting schedules cleanup the storage and unlock the fund
			if bounded_schedules.len().is_zero() {
				<EDFVestingSchedules<T>>::remove(who);
				T::MultiCurrency::remove_lock(VESTING_LOCK_ID,T::GetEDFCurrencyId::get(), who)?;
				return Ok(());
			}

			let total_amount = bounded_schedules
				.iter()
				.try_fold::<_, _, Result<BalanceOf<T>, DispatchError>>(Zero::zero(), |acc_amount, schedule| {
					let amount = ensure_valid_vesting_schedule::<T>(T::GetEDFCurrencyId::get(), schedule)?;
					acc_amount
						.checked_add(&amount)
						.ok_or_else(|| ArithmeticError::Overflow.into())
				})?;
			ensure!(
				T::MultiCurrency::free_balance(T::GetEDFCurrencyId::get(), who) >= total_amount,
				Error::<T>::InsufficientBalanceToLock,
			);

			T::MultiCurrency::set_lock(VESTING_LOCK_ID, T::GetEDFCurrencyId::get(), who, total_amount);
			<EDFVestingSchedules<T>>::insert(who, bounded_schedules);
		}
		Ok(())
	}
}

/// Returns `Ok(total_total)` if valid schedule, or error.
fn ensure_valid_vesting_schedule<T: Config>(
	currency_id: CurrencyIdOf<T>,
	schedule: &VestingScheduleOf<T>
) -> Result<BalanceOf<T>, DispatchError> {
	ensure!(!schedule.period.is_zero(), Error::<T>::ZeroVestingPeriod);
	ensure!(!schedule.period_count.is_zero(), Error::<T>::ZeroVestingPeriodCount);
	ensure!(schedule.end().is_some(), ArithmeticError::Overflow);

	let total_total = schedule.total_amount().ok_or(ArithmeticError::Overflow)?;

	if currency_id == T::GetNativeCurrencyId::get() {
		ensure!(total_total >= T::MinNativeVestedTransfer::get(), Error::<T>::AmountLow);
	} else if currency_id == T::GetEDFCurrencyId::get() {
		ensure!(total_total >= T::MinEDFVestedTransfer::get(), Error::<T>::AmountLow);
	}

	Ok(total_total)
}
