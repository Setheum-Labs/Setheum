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

//! # SettmintManager Module
//!
//! ## Overview
//!
//! SettmintManager module manages Settmint's reserve asset (Setter) 
//! and the standards backed by the asset (SettCurrencies).

#![cfg_attr(not(feature = "std"), no_std)]
#![allow(clippy::unused_unit)]

use frame_support::{log, pallet_prelude::*, traits::MaxEncodedLen, transactional, PalletId};
use orml_traits::{MultiCurrency, MultiCurrencyExtended};
use primitives::{Amount, Balance, CurrencyId};
use sp_runtime::{
	traits::{AccountIdConversion, Convert, Zero},
	ArithmeticError, DispatchResult, RuntimeDebug,
};
use sp_std::{convert::TryInto, result};
use support::SerpTreasury;

mod mock;
mod tests;

pub use module::*;

/// A settmint standard position.
#[derive(Encode, Decode, Eq, PartialEq, Copy, Clone, RuntimeDebug, Default, MaxEncodedLen)]
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

		/// Convert standard amount under specific standard type to standard
		/// value(SettCurrency)
		type Convert: Convert<(CurrencyId, Balance), Balance>;

		/// Currency type for deposit/withdraw reserve assets 
		/// to/from settmint-manager module
		type Currency: MultiCurrencyExtended<
			Self::AccountId,
			CurrencyId = CurrencyId,
			Balance = Balance,
			Amount = Amount,
		>;

		/// The list of valid standard currency types
		type StandardCurrencyIds: Get<Vec<CurrencyId>>;

		#[pallet::constant]
		/// Setter (Valid Reserve) currency id
		type GetReserveCurrencyId: Get<CurrencyId>;

		/// SERP Treasury for issuing/burning stable currency adjust standard value
		/// adjustment
		type SerpTreasury: SerpTreasury<Self::AccountId, Balance = Balance, CurrencyId = CurrencyId>;

		/// The setter's module id, keep all reserves of Settmint.
		#[pallet::constant]
		type PalletId: Get<PalletId>;
	}

	#[pallet::error]
	pub enum Error<T> {
		AmountConvertFailed,
		InvalidStandardType,
	}

	#[pallet::event]
	#[pallet::generate_deposit(pub(crate) fn deposit_event)]
	#[pallet::metadata(T::AccountId = "AccountId")]
	pub enum Event<T: Config> {
		/// Position updated. \[owner, reserve_type, reserve_adjustment,
		/// standard_adjustment\]
		PositionUpdated(T::AccountId, CurrencyId, Amount, Amount),
		/// Transfer setter. \[from, to, currency_id\]
		TransferPosition(T::AccountId, T::AccountId, CurrencyId),
	}

	/// The reserved standard positions, map from
	/// Owner -> StandardType -> Position
	#[pallet::storage]
	#[pallet::getter(fn positions)]
	pub type Positions<T: Config> =
		StorageDoubleMap<_, Twox64Concat, CurrencyId, Twox64Concat, T::AccountId, Position, ValueQuery>;

	/// The total reserved standard positions, map from
	/// `StandardType -> Position`
	///
	#[pallet::storage]
	#[pallet::getter(fn total_positions)]
	pub type TotalPositions<T: Config> = StorageMap<_, Twox64Concat, CurrencyId, Position, ValueQuery>;

	#[pallet::pallet]
	pub struct Pallet<T>(_);

	#[pallet::hooks]
	impl<T: Config> Hooks<T::BlockNumber> for Pallet<T> {}

	#[pallet::call]
	impl<T: Config> Pallet<T> {}
}

impl<T: Config> Pallet<T> {
	pub fn account_id() -> T::AccountId {
		T::PalletId::get().into_account()
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
		// TODO: Update, see what's wrong
		Self::update_position(who, currency_id, reserve_adjustment, standard_adjustment)?;

		let reserve_balance_adjustment = Self::balance_try_from_amount_abs(reserve_adjustment)?;
		let standard_balance_adjustment = Self::balance_try_from_amount_abs(standard_adjustment)?;
		let setheum_account = Self::account_id();
		let reserve_currency = T::GetReserveCurrencyId::get();

		// ensure the currency is a settcurrency standard
		ensure!(
			T::StandardCurrencyIds::get().contains(&currency_id),
			Error::<T>::InvalidStandardType,
		);
	
		if reserve_adjustment.is_positive() {
			T::Currency::transfer(reserve_currency, &who, &setheum_account,
				T::Convert::convert((reserve_currency, reserve_balance_adjustment)))?;
		} else if reserve_adjustment.is_negative() {
			T::Currency::transfer(reserve_currency, &setheum_account, &who,
				T::Convert::convert((reserve_currency, reserve_balance_adjustment)))?;
		}

		// TODO: Update! convert from u128 to balance before acting on SerpTreasury.
		if standard_adjustment.is_positive() {
			// issue standard with reserve backed by SERP Treasury
			T::SerpTreasury::issue_standard(currency_id, who, T::Convert::convert((currency_id, standard_balance_adjustment)))?;
		} else if standard_adjustment.is_negative() {
			// repay standard
			// burn standard by SERP Treasury
			T::SerpTreasury::burn_standard(currency_id, who, T::Convert::convert((currency_id, standard_balance_adjustment)))?;
		}

		Self::deposit_event(Event::PositionUpdated(
			who.clone(),
			currency_id,
			reserve_adjustment,
			standard_adjustment,
		));
		Ok(())
	}

	/// transfer whole setter reserve of `from` to `to`
	pub fn transfer_position(from: &T::AccountId, to: &T::AccountId, currency_id: CurrencyId) -> DispatchResult {
		// get `from` position data
		let Position { reserve, standard } = Self::positions(currency_id, from);

		// balance -> amount
		let reserve_adjustment = Self::amount_try_from_balance(reserve)?;
		let standard_adjustment = Self::amount_try_from_balance(standard)?;

		Self::update_position(
			from,
			currency_id,
			reserve_adjustment.saturating_neg(),
			standard_adjustment.saturating_neg(),
		)?;
		Self::update_position(to, currency_id, reserve_adjustment, standard_adjustment)?;

		Self::deposit_event(Event::TransferPosition(from.clone(), to.clone(), currency_id));
		Ok(())
	}

	/// mutate records of reserves and standards
	fn update_position(
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
					.ok_or(ArithmeticError::Overflow)
			} else {
				p.reserve
					.checked_sub(reserve_balance)
					.ok_or(ArithmeticError::Underflow)
			}?;
			let new_standard = if standard_adjustment.is_positive() {
				p.standard.checked_add(standard_balance).ok_or(ArithmeticError::Overflow)
			} else {
				p.standard.checked_sub(standard_balance).ok_or(ArithmeticError::Underflow)
			}?;

			// increase account ref if new position
			if p.reserve.is_zero() && p.standard.is_zero() {
				if frame_system::Pallet::<T>::inc_consumers(who).is_err() {
					// No providers for the locks. This is impossible under normal circumstances
					// since the funds that are under the lock will themselves be stored in the
					// account and therefore will need a reference.
					log::warn!(
						"Warning: Attempt to introduce lock consumer reference, yet no providers. \
						This is unexpected but should be safe."
					);
				}
			}

			p.reserve = new_reserve;
			p.standard = new_standard;

			if p.reserve.is_zero() && p.standard.is_zero() {
				// decrease account ref if zero position
				frame_system::Pallet::<T>::dec_consumers(who);

				// remove position storage if zero position
				*may_be_position = None;
			} else {
				*may_be_position = Some(p);
			}

			Ok(())
		})?;

		TotalPositions::<T>::try_mutate(currency_id, |total_positions| -> DispatchResult {
			total_positions.standard = if standard_adjustment.is_positive() {
				total_positions
					.standard
					.checked_add(standard_balance)
					.ok_or(ArithmeticError::Overflow)
			} else {
				total_positions
					.standard
					.checked_sub(standard_balance)
					.ok_or(ArithmeticError::Underflow)
			}?;

			total_positions.reserve = if reserve_adjustment.is_positive() {
				total_positions
					.reserve
					.checked_add(reserve_balance)
					.ok_or(ArithmeticError::Overflow)
			} else {
				total_positions
					.reserve
					.checked_sub(reserve_balance)
					.ok_or(ArithmeticError::Underflow)
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

	pub fn total_reserve() -> Balance {
		let reserve_currency = T::GetReserveCurrencyId::get();
		T::Currency::free_balance(reserve_currency, &Self::account_id())
	}
}
