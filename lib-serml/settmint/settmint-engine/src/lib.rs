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

//! # Settmint Engine Module
//!
//! ## Overview
//!
//! The core module of the Settmint protocol.
//! The Settmint engine is responsible for handling
//! internal processes of Settmint.

#![cfg_attr(not(feature = "std"), no_std)]
#![allow(clippy::unused_unit)]
#![allow(clippy::upper_case_acronyms)]

use frame_support::pallet_prelude::*;
use frame_system::{
	offchain::SendTransactionTypes,
	pallet_prelude::*,
};
use primitives::{Amount, Balance, CurrencyId};
use sp_runtime::{
	traits::Convert,
	DispatchResult, FixedPointNumber,
};
use sp_std::prelude::*;
use support::{ExchangeRate, Price, Ratio};

mod standard_exchange_rate_convertor;
mod mock;
mod tests;

pub use standard_exchange_rate_convertor::StandardExchangeRateConvertor;
pub use module::*;

pub type SettmintOf<T> = settmint::Pallet<T>;

#[frame_support::pallet]
pub mod module {
	use super::*;

	#[pallet::config]
	pub trait Config: frame_system::Config + settmint::Config + SendTransactionTypes<Call<Self>> {
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;

		/// The list of valid standard currency types
		type StandardCurrencies: Get<Vec<CurrencyId>>;

		#[pallet::constant]
		/// The default standard exchange rate for all reserve types
		type DefaultStandardExchangeRate: Get<ExchangeRate>;

		#[pallet::constant]
		/// The minimum standard value to avoid standard dust
		type MinimumStandardValue: Get<Balance>;

		#[pallet::constant]
		/// Setter (Valid Reserve) currency id
		type ReserveCurrencyId: Get<CurrencyId>;
	}

	#[pallet::error]
	pub enum Error<T> {
		/// Invalid reserve type.
		InvalidStandardType,
		/// Remaining standard value in Settmint below the dust amount
		RemainStandardValueTooSmall,
		/// Feed price is invalid
		InvalidFeedPrice,
	}

	#[pallet::event]
	pub enum Event<T: Config> {}

	/// Mapping from standard currency type to its exchange rate of standard units and
	/// standard value (rate of reserve to standard) - the Setter reserve.
	/// StandardExchangeRate: CurrencyId => Option<ExchangeRate>
	#[pallet::storage]
	#[pallet::getter(fn standard_exchange_rate)]
	pub type StandardExchangeRate<T: Config> = StorageMap<_, Twox64Concat, CurrencyId, ExchangeRate, OptionQuery>;

	#[pallet::pallet]
	pub struct Pallet<T>(_);

	#[pallet::hooks]
	impl<T: Config> Hooks<T::BlockNumber> for Pallet<T> {}

	#[pallet::call]
	impl<T: Config> Pallet<T> {}
}

impl<T: Config> Pallet<T> {
	pub fn get_standard_exchange_rate(currency_id: CurrencyId) -> ExchangeRate {
		Self::standard_exchange_rate(currency_id).unwrap_or_else(T::DefaultStandardExchangeRate::get)
	}

	pub fn get_standard_value(currency_id: CurrencyId, standard_balance: Balance) -> Balance {
		crate::StandardExchangeRateConvertor::<T>::convert((currency_id, standard_balance))
	}

	pub fn calculate_reserve_ratio(
		currency_id: CurrencyId,
		reserve_balance: Balance,
		standard_balance: Balance,
		price: Price,
	) -> Ratio {
		let locked_reserve_value = price.saturating_mul_int(reserve_balance);
		let standard_value = Self::get_standard_value(currency_id, standard_balance);

		Ratio::checked_from_rational(locked_reserve_value, standard_value).unwrap_or_default()
	}

	pub fn adjust_position(
		who: &T::AccountId,
		currency_id: CurrencyId,
		reserve_adjustment: Amount,
		standard_adjustment: Amount,
	) -> DispatchResult {
		<SettmintOf<T>>::adjust_position(who, currency_id, reserve_adjustment, standard_adjustment)?;
		Ok(())
	}
}
