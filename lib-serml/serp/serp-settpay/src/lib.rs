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

//! # SERP SettPay Module
//!
//! ## Overview
//!
//! SERP SettPay manages the cashdrop (cashback) rewards by the SERP serplus.

#![cfg_attr(not(feature = "std"), no_std)]
#![allow(clippy::unused_unit)]

use frame_support::{pallet_prelude::*, transactional, PalletId};
use frame_system::pallet_prelude::*;
use orml_traits::MultiCurrency;
use primitives::{Amount, Balance, CurrencyId};
use sp_runtime::{
	traits::{AccountIdConversion, Convert, One, Zero},
	DispatchError, DispatchResult, FixedPointNumber,
};
use support::{SerpTreasury, CashDropRate, Price, Rate, Ratio};
mod mock;
mod tests;
pub mod weights;

pub use module::*;
pub use weights::WeightInfo;

#[frame_support::pallet]
pub mod module {
	use super::*;

	#[pallet::config]
	pub trait Config: frame_system::Config {
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;

		/// The origin which may update parameters and handle
		/// serplus/standard/reserve. Root can always do this.
		type UpdateOrigin: EnsureOrigin<Self::Origin>;

		/// The stable currency ids.
		type StableCurrencyIds: Get<Vec<CurrencyId>>;

		/// The entire cashdrop currency ids.
		type CashDropCurrencyIds: Get<Vec<CurrencyId>>;

		/// The cashdrop currency ids that receive Setter.
		type SetterDropCurrencyIds: Get<Vec<CurrencyId>>;

		/// The cashdrop currency ids that receive SettCurrencies.
		type SetCurrencyDropCurrencyIds: Get<Vec<CurrencyId>>;

		#[pallet::constant]
		/// Setter (SETT) currency Stablecoin currency id.
		type GetSetterCurrencyId: Get<CurrencyId>;

		#[pallet::constant]
		/// The default minimum transfer value to secure settpay from dusty claims.
		type DefaultMinimumClaimableTransfer: Get<Balance>;

		#[pallet::constant]
		/// The default cashdrop rate to be issued to claims.
		type DefaultCashDropRate: Get<Balance>;

		/// SERP Treasury for depositing cashdrop.
		type SerpTreasury: SerpTreasury<Self::AccountId, Balance = Balance, CurrencyId = CurrencyId>;

		#[pallet::constant]
		/// The SERP Treasury's module id, keeps serplus and reserve asset.
		type PalletId: Get<PalletId>;

		/// Weight information for the extrinsics in this module.
		type WeightInfo: WeightInfo;
	}

	#[pallet::error]
	pub enum Error<T> {
		/// CashDrop is not available.
		CashdropNotAvailable,
	}

	#[pallet::event]
	#[pallet::generate_deposit(fn deposit_event)]
	pub enum Event<T: Config> {
		/// CashDrop has been completed successfully.
		CashDrops(CurrencyId, AccountId, Balance),
	}

	/// Mapping to Cashdrop rate.
	/// The first item of the tuple is the numerator of the cashdrop rate, second
	/// item is the denominator, cashdrop_rate = numerator / denominator,
	/// use (u32, u32) over `Rate` type to minimize internal division
	/// operation.
	#[pallet::storage]
	#[pallet::getter(fn cashdrop_rate)]
	pub type GetCashDropRate<T: Config> = StorageMap<_, Twox64Concat, CurrencyId, u32, u32, ValueQuery>;

	/// Mapping to Minimum Claimable Transfer.
	#[pallet::storage]
	#[pallet::getter(fn cashdrop_rate)]
	pub type MinimumClaimableTransfer<T: Config> = StorageMap<_, Twox64Concat, CurrencyId, Balance, ValueQuery>;

	#[pallet::pallet]
	pub struct Pallet<T>(_);

	#[pallet::hooks]
	impl<T: Config> Hooks<T::BlockNumber> for Pallet<T> {}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		#[pallet::weight(<T as Config>::WeightInfo::update_cashdrop_rate(
			currency.len() as CurrencyId,
			numerator.len() as u32,
			denominator.len() as u32)
		)]
		#[transactional]
		pub fn update_cashdrop_rate(
			origin: OriginFor<T>,
			numerator: u32,
			denominator: u32,
		) -> DispatchResultWithPostInfo {
			T::UpdateOrigin::ensure_origin(origin)?;
			for (numerator) in numerator && denominator in (denominator) {
				GetCashDropRate::<T>::insert(numerator, denominator);
			}
			Ok(().into())
		}
	}
}

impl<T: Config> Pallet<T> {
	/// Get account of SERP SettPay module.
	pub fn account_id() -> T::AccountId {
		T::PalletId::get().into_account()
	}

	pub fn get_cashdrop_ratio(currency_id: T::CurrencyId) -> <u64, u64> {
		ensure!(
			T::CashDropCurrencyIds.contains(currency_id),
			Error::<T>::InvalidCurrencyType,
		)
		Self::cashdrop_rate(currency_id).unwrap_or_else(T::DefaultCashDropRate::get)
	}
}

impl<T: Config> SettPay<T::AccountId> for Pallet<T> {
	type Balance = Balance;
	type CurrencyId = CurrencyId;

	/// claim cashdrop of `currency_id` relative to `transfer_amount` for `who`
	fn claim_cashdrop(currency_id: Self::CurrencyId, who: &AccountId, transfer_amount: Self::Balance) -> DispatchResult {
		ensure!(
			T::CashDropCurrencyIds.contains(currency_id),
			Error::<T>::InvalidCurrencyType,
		)
		ensure!(
			T::CashDropCurrencyIds.contains(currency_id),
			Error::<T>::InvalidCurrencyType,
		)
	}

	/// deposit cashdrop of `currency_id` relative to `transfer_amount` for `who`
	fn deposit_cashdrop(currency_id: Self::CurrencyId, who: &AccountId, transfer_amount: Self::Balance) -> DispatchResult {

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
		T::Currency::free_balance(&reserve_currency, &Self::account_id())
	}
	
	fn get_total_reserve() -> Self::Balance {
		Self::total_reserve()
	}
}
