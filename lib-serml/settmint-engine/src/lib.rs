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

use frame_support::{pallet_prelude::*, transactional};
use frame_system::{
	offchain::{SendTransactionTypes, SubmitTransaction},
	pallet_prelude::*,
};
use settmint_manager::Position;
use orml_traits::Change;
use orml_utilities::{IterableStorageDoubleMapExtended, OffchainErr};
use primitives::{Amount, Balance, CurrencyId};
use sp_runtime::{
	offchain::{
		storage::StorageValueRef,
		storage_lock::{StorageLock, Time},
		Duration,
	},
	traits::{BlakeTwo256, Bounded, Convert, Hash, Saturating, StaticLookup, Zero},
	transaction_validity::{
		InvalidTransaction, TransactionPriority, TransactionSource, TransactionValidity, ValidTransaction,
	},
	DispatchError, DispatchResult, FixedPointNumber, RandomNumberGenerator, RuntimeDebug,
};
use sp_std::prelude::*;
use support::{
	DEXManager, ExchangeRate, Price, PriceProvider, Rate, Ratio,
};

mod mock;
mod tests;

pub use module::*;

pub const OFFCHAIN_WORKER_DATA: &[u8] = b"setheum/settmint-engine/data/";
pub const OFFCHAIN_WORKER_LOCK: &[u8] = b"setheum/settmint-engine/lock/";
pub const OFFCHAIN_WORKER_MAX_ITERATIONS: &[u8] = b"setheum/settmint-engine/max-iterations/";
pub const LOCK_DURATION: u64 = 100;
pub const DEFAULT_MAX_ITERATIONS: u32 = 1000;

pub type SettmintManagerOf<T> = settmint_manager::Module<T>;

// typedef to help polkadot.js disambiguate Change with different generic
// parameters
type ChangeOptionRate = Change<Option<Rate>>;
type ChangeOptionRatio = Change<Option<Ratio>>;
type ChangeBalance = Change<Balance>;

#[frame_support::pallet]
pub mod module {
	use super::*;

	#[pallet::config]
	pub trait Config: frame_system::Config + settmint_manager::Config + SendTransactionTypes<Call<Self>> {
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;

		/// The list of valid standard currency types
		type StandardCurrencyIds: Get<Vec<CurrencyId>>;

		#[pallet::constant]
		/// The default standard exchange rate for all reserve types
		type DefaultStandardExchangeRate: Get<ExchangeRate>;

		#[pallet::constant]
		/// The minimum standard value to avoid standard dust
		type MinimumStandardValue: Get<Balance>;

		#[pallet::constant]
		/// Setter (Valid Reserve) currency id
		type GetReserveCurrencyId: Get<CurrencyId>;

		/// The price source of all types of currencies related to Settmint
		type PriceSource: PriceProvider<CurrencyId>;

		#[pallet::constant]
		/// A configuration for base priority of unsigned transactions.
		///
		/// This is exposed so that it can be tuned for particular runtime, when
		/// multiple modules send unsigned transactions.
		type UnsignedPriority: Get<TransactionPriority>;

		/// Weight information for the extrinsics in this module.
		type WeightInfo: WeightInfo;
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
	#[pallet::generate_deposit(pub(crate) fn deposit_event)]
	pub enum Event<T: Config> {}

	/// Mapping from standard currency type to its exchange rate of standard units and
	/// standard value (rate of reserve to standard) - the Setter reserve.
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
		// ensure the currency is a settcurrency standard
		ensure!(
			T::StandardCurrencyIds::get().contains(&currency_id),
			Error::<T>::InvalidStandardType,
		);
		Self::standard_exchange_rate(currency_id).unwrap_or_else(T::DefaultStandardExchangeRate::get)
	}

	pub fn get_standard_value(currency_id: CurrencyId, standard_balance: Balance) -> Balance {
		// ensure the currency is a settcurrency standard
		ensure!(
			T::StandardCurrencyIds::get().contains(&currency_id),
			Error::<T>::InvalidStandardType,
		);
		crate::StandardExchangeRateConvertor::<T>::convert((currency_id, standard_balance))
	}

	pub fn calculate_reserve_ratio(
		currency_id: CurrencyId,
		reserve_balance: Balance,
		standard_balance: Balance,
		price: Price,
	) -> Ratio {
		// ensure the currency is a settcurrency standard
		ensure!(
			T::StandardCurrencyIds::get().contains(&currency_id),
			Error::<T>::InvalidStandardType,
		);
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
		// ensure the currency is a settcurrency standard
		ensure!(
			T::StandardCurrencyIds::get().contains(&currency_id),
			Error::<T>::InvalidStandardType,
		);
		<SettmintManagerOf<T>>::adjust_position(who, currency_id, reserve_adjustment, standard_adjustment)?;
		Ok(())
	}
}

impl<T: Config> StandardValidator<T::AccountId, CurrencyId, Balance, Balance> for Pallet<T> {

	fn check_position_valid(
		currency_id: CurrencyId,
		reserve_balance: Balance,
		standard_balance: Balance,
	) -> DispatchResult {
		if !standard_balance.is_zero() {
			let standard_value = Self::get_standard_value(currency_id, standard_balance);
			let feed_price = <T as Config>::PriceSource::get_relative_price(T::GetReserveCurrencyId::get(), currency_id)
				.ok_or(Error::<T>::InvalidFeedPrice)?;
			let reserve_ratio =
				Self::calculate_reserve_ratio(currency_id, reserve_balance, standard_balance, feed_price);

			// ensure the currency is a settcurrency standard
			ensure!(
				T::StandardCurrencyIds::get().contains(&currency_id),
				Error::<T>::InvalidStandardType,
			);
			// check the minimum_standard_value
			ensure!(
				standard_value >= T::MinimumStandardValue::get(),
				Error::<T>::RemainStandardValueTooSmall,
			);
		}

		Ok(())
	}
}
