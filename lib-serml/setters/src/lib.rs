// This file is part of Setheum.

// Copyright (C) 2020-2021 Setheum Labs.
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

//! # Setters Module
//!
//! ## Overview
//!
//! Setters module manages Settmint's reserve assets and the standards backed by these
//! assets.

#![cfg_attr(not(feature = "std"), no_std)]
#![allow(clippy::unused_unit)]
#![allow(clippy::collapsible_if)]

use frame_support::pallet_prelude::*;
use frame_support::transactional;
use orml_traits::{Happened, MultiCurrency, MultiCurrencyExtended};
use primitives::{Amount, Balance, CurrencyId};
use sp_runtime::{
	traits::{AccountIdConversion, Convert, Zero},
	DispatchResult, ModuleId, RuntimeDebug,
};
use sp_std::{convert::TryInto, result};
use support::{SerpTreasury, RiskManager};

mod mock;
mod tests;

pub use module::*;

/// A reserveized standard position.
#[derive(Encode, Decode, Eq, PartialEq, Copy, Clone, RuntimeDebug, Default)]
pub struct Position {
	/// The amount of reserve.
	pub reserve: Balance,
	/// The amount of standard.
	pub standard: Balance,
}

#[frame_support::pallet]
pub mod module {
	use super::*;

	#[pallet::config]
	pub trait Config: frame_system::Config {
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;

		/// Convert standard amount under specific reserve type to standard
		/// value(stable currency)
		type Convert: Convert<(CurrencyId, Balance), Balance>;

		/// Currency type for deposit/withdraw reserve assets to/from setters
		/// module
		type Currency: MultiCurrencyExtended<
			Self::AccountId,
			CurrencyId = CurrencyId,
			Balance = Balance,
			Amount = Amount,
		>;

		/// Risk manager is used to limit the standard size of Settmint
		type RiskManager: RiskManager<Self::AccountId, CurrencyId, Balance, Balance>;

		/// SERP Treasury for issuing/burning stable currency adjust standard value
		/// adjustment
		type SerpTreasury: SerpTreasury<Self::AccountId, Balance = Balance, CurrencyId = CurrencyId>;

		/// The setter's module id, keep all reserves of Settmint.
		#[pallet::constant]
		type ModuleId: Get<ModuleId>;

		/// Event handler which calls when update setter.
		type OnUpdateSetter: Happened<(Self::AccountId, CurrencyId, Amount, Balance)>;
	}

	#[pallet::error]
	pub enum Error<T> {
		StandardOverflow,
		StandardTooLow,
		ReserveOverflow,
		ReserveTooLow,
		AmountConvertFailed,
	}

	#[pallet::event]
	#[pallet::generate_deposit(pub(crate) fn deposit_event)]
	pub enum Event<T: Config> {
		/// Position updated. \[owner, reserve_type, reserve_adjustment,
		/// standard_adjustment\]
		PositionUpdated(T::AccountId, CurrencyId, Amount, Amount),
		/// Confiscate Settmint's reserve assets and eliminate its standard. \[owner,
		/// reserve_type, confiscated_reserve_amount,
		/// deduct_standard_amount\]
		ConfiscateReserveAndStandard(T::AccountId, CurrencyId, Balance, Balance),
		/// Transfer setter. \[from, to, currency_id\]
		TransferSetter(T::AccountId, T::AccountId, CurrencyId),
	}

	/// The reserveized standard positions, map from
	/// Owner -> ReserveType -> Position
	#[pallet::storage]
	#[pallet::getter(fn positions)]
	pub type Positions<T: Config> =
		StorageDoubleMap<_, Twox64Concat, CurrencyId, Twox64Concat, T::AccountId, Position, ValueQuery>;

	/// The total reserveized standard positions, map from
	/// ReserveType -> Position
	#[pallet::storage]
	#[pallet::getter(fn total_positions)]
	pub type TotalPositions<T: Config> = StorageMap<_, Twox64Concat, CurrencyId, Position, ValueQuery>;

	#[pallet::pallet]
	pub struct Pallet<T>(PhantomData<T>);

	#[pallet::hooks]
	impl<T: Config> Hooks<T::BlockNumber> for Pallet<T> {}

	#[pallet::call]
	impl<T: Config> Pallet<T> {}
}

impl<T: Config> Pallet<T> {
	pub fn account_id() -> T::AccountId {
		T::ModuleId::get().into_account()
	}

	/// confiscate reserve and standard to SERP Treasury.
	///
	/// Ensured atomic.
	#[transactional]
	pub fn confiscate_reserve_and_standard(
		who: &T::AccountId,
		currency_id: CurrencyId,
		reserve_confiscate: Balance,
		standard_decrease: Balance,
	) -> DispatchResult {
		// convert balance type to amount type
		let reserve_adjustment = Self::amount_try_from_balance(reserve_confiscate)?;
		let standard_adjustment = Self::amount_try_from_balance(standard_decrease)?;

		// transfer reserve to SERP Treasury
		T::SerpTreasury::deposit_reserve(&Self::account_id(), currency_id, reserve_confiscate)?;

		// deposit standard to SERP Treasury
		let bad_standard_value = T::RiskManager::get_bad_standard_value(currency_id, standard_decrease);
		T::SerpTreasury::on_system_standard(bad_standard_value)?;

		// update setter
		Self::update_setter(
			&who,
			currency_id,
			reserve_adjustment.saturating_neg(),
			standard_adjustment.saturating_neg(),
		)?;

		Self::deposit_event(Event::ConfiscateReserveAndStandard(
			who.clone(),
			currency_id,
			reserve_confiscate,
			standard_decrease,
		));
		Ok(())
	}

	/// adjust the position.
	///
	/// Ensured atomic.
	#[transactional]
	pub fn adjust_position(
		who: &T::AccountId,
		currency_id: CurrencyId,
		reserve_adjustment: Amount,
		standard_adjustment: Amount,
	) -> DispatchResult {
		// mutate reserve and standard
		Self::update_setter(who, currency_id, reserve_adjustment, standard_adjustment)?;

		let reserve_balance_adjustment = Self::balance_try_from_amount_abs(reserve_adjustment)?;
		let standard_balance_adjustment = Self::balance_try_from_amount_abs(standard_adjustment)?;
		let setheum_account = Self::account_id();

		if reserve_adjustment.is_positive() {
			T::Currency::transfer(currency_id, who, &setheum_account, reserve_balance_adjustment)?;
		} else if reserve_adjustment.is_negative() {
			T::Currency::transfer(currency_id, &setheum_account, who, reserve_balance_adjustment)?;
		}

		if standard_adjustment.is_positive() {
			// check standard cap when increase standard
			T::RiskManager::check_standard_cap(currency_id, Self::total_positions(currency_id).standard)?;

			// issue standard with reserve backed by SERP Treasury
			T::SerpTreasury::issue_standard(who, T::Convert::convert((currency_id, standard_balance_adjustment)), true)?;
		} else if standard_adjustment.is_negative() {
			// repay standard
			// burn standard by SERP Treasury
			T::SerpTreasury::burn_standard(who, T::Convert::convert((currency_id, standard_balance_adjustment)))?;
		}

		// ensure pass risk check
		let Position { reserve, standard } = Self::positions(currency_id, who);
		T::RiskManager::check_position_valid(currency_id, reserve, standard)?;

		Self::deposit_event(Event::PositionUpdated(
			who.clone(),
			currency_id,
			reserve_adjustment,
			standard_adjustment,
		));
		Ok(())
	}

	/// transfer whole setter of `from` to `to`
	pub fn transfer_setter(from: &T::AccountId, to: &T::AccountId, currency_id: CurrencyId) -> DispatchResult {
		// get `from` position data
		let Position { reserve, standard } = Self::positions(currency_id, from);

		let Position {
			reserve: to_reserve,
			standard: to_standard,
		} = Self::positions(currency_id, to);
		let new_to_reserve_balance = to_reserve
			.checked_add(reserve)
			.expect("existing reserve balance cannot overflow; qed");
		let new_to_standard_balance = to_standard
			.checked_add(standard)
			.expect("existing standard balance cannot overflow; qed");

		// check new position
		T::RiskManager::check_position_valid(currency_id, new_to_reserve_balance, new_to_standard_balance)?;

		// balance -> amount
		let reserve_adjustment = Self::amount_try_from_balance(reserve)?;
		let standard_adjustment = Self::amount_try_from_balance(standard)?;

		Self::update_setter(
			from,
			currency_id,
			reserve_adjustment.saturating_neg(),
			standard_adjustment.saturating_neg(),
		)?;
		Self::update_setter(to, currency_id, reserve_adjustment, standard_adjustment)?;

		Self::deposit_event(Event::TransferSetter(from.clone(), to.clone(), currency_id));
		Ok(())
	}

	/// mutate records of reserves and standards
	fn update_setter(
		who: &T::AccountId,
		currency_id: CurrencyId,
		reserve_adjustment: Amount,
		standard_adjustment: Amount,
	) -> DispatchResult {
		let reserve_balance = Self::balance_try_from_amount_abs(reserve_adjustment)?;
		let standard_balance = Self::balance_try_from_amount_abs(standard_adjustment)?;

		<Positions<T>>::try_mutate_exists(currency_id, who, |may_be_position| -> DispatchResult {
			let mut p = may_be_position.take().unwrap_or_default();
			let new_reserve = if reserve_adjustment.is_positive() {
				p.reserve
					.checked_add(reserve_balance)
					.ok_or(Error::<T>::ReserveOverflow)
			} else {
				p.reserve
					.checked_sub(reserve_balance)
					.ok_or(Error::<T>::ReserveTooLow)
			}?;
			let new_standard = if standard_adjustment.is_positive() {
				p.standard.checked_add(standard_balance).ok_or(Error::<T>::StandardOverflow)
			} else {
				p.standard.checked_sub(standard_balance).ok_or(Error::<T>::StandardTooLow)
			}?;

			// increase account ref if new position
			if p.reserve.is_zero() && p.standard.is_zero() {
				if frame_system::Module::<T>::inc_consumers(who).is_err() {
					// No providers for the locks. This is impossible under normal circumstances
					// since the funds that are under the lock will themselves be stored in the
					// account and therefore will need a reference.
					frame_support::debug::warn!(
						"Warning: Attempt to introduce lock consumer reference, yet no providers. \
						This is unexpected but should be safe."
					);
				}
			}

			p.reserve = new_reserve;

			T::OnUpdateSetter::happened(&(who.clone(), currency_id, standard_adjustment, p.standard));
			p.standard = new_standard;

			if p.reserve.is_zero() && p.standard.is_zero() {
				// decrease account ref if zero position
				frame_system::Module::<T>::dec_consumers(who);

				// remove position storage if zero position
				*may_be_position = None;
			} else {
				*may_be_position = Some(p);
			}

			Ok(())
		})?;

		TotalPositions::<T>::try_mutate(currency_id, |total_positions| -> DispatchResult {
			total_positions.reserve = if reserve_adjustment.is_positive() {
				total_positions
					.reserve
					.checked_add(reserve_balance)
					.ok_or(Error::<T>::ReserveOverflow)
			} else {
				total_positions
					.reserve
					.checked_sub(reserve_balance)
					.ok_or(Error::<T>::ReserveTooLow)
			}?;

			total_positions.standard = if standard_adjustment.is_positive() {
				total_positions
					.standard
					.checked_add(standard_balance)
					.ok_or(Error::<T>::StandardOverflow)
			} else {
				total_positions
					.standard
					.checked_sub(standard_balance)
					.ok_or(Error::<T>::StandardTooLow)
			}?;

			Ok(())
		})
	}
}

impl<T: Config> Pallet<T> {
	/// Convert `Balance` to `Amount`.
	fn amount_try_from_balance(b: Balance) -> result::Result<Amount, Error<T>> {
		TryInto::<Amount>::try_into(b).map_err(|_| Error::<T>::AmountConvertFailed)
	}

	/// Convert the absolute value of `Amount` to `Balance`.
	fn balance_try_from_amount_abs(a: Amount) -> result::Result<Balance, Error<T>> {
		TryInto::<Balance>::try_into(a.saturating_abs()).map_err(|_| Error::<T>::AmountConvertFailed)
	}
}
