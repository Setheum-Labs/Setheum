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
use frame_system::{pallet_prelude::*,Pallet};
use orml_traits::{GetByKey, MultiCurrency, MultiCurrencyExtended};
use primitives::{AccountId, Balance, CurrencyId};
use sp_runtime::{
	traits::{AccountIdConversion, Convert, One, Zero},
	DispatchError, DispatchResult, FixedPointNumber,
};
use sp_core::U256;
use sp_std::{convert::TryInto, result};
use support::{SerpTreasury, CashDrop, CashDropRate, Price, Rate, Ratio};
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

		/// The Currency for managing assets related to the SERP (Setheum Elastic Reserve Protocol).
		type Currency: MultiCurrencyExtended<Self::AccountId, CurrencyId = CurrencyId, Balance = Balance>;

		#[pallet::constant]
		/// Setter (SETT) currency Stablecoin currency id.
		type SetterCurrencyId: Get<CurrencyId>;

		/// The stable currency ids.
		type StableCurrencyIds: Get<Vec<CurrencyId>>;

		/// The cashdrop currency ids that can be rewarded with CashDrop.
		type RewardableCurrencyIds: Get<Vec<CurrencyId>>;

		/// The cashdrop currency ids that receive Setter.
		type NonStableDropCurrencyIds: Get<Vec<CurrencyId>>;

		/// The cashdrop currency ids that receive SettCurrencies.
		type SetCurrencyDropCurrencyIds: Get<Vec<CurrencyId>>;

		#[pallet::constant]
		/// The cashdrop rate to be issued to claims by `currency_id`.
		/// 
		/// The first item of the tuple is the numerator of the cashdrop rate, second
		/// item is the denominator, cashdrop_rate = numerator / denominator,
		/// use (u32, u32) over `Rate` type to minimize internal division
		/// operation.
		type DefaultCashDropRate: Get<CashDropRate>;

		/// The default cashdrop rate to be issued to claims.
		/// 
		/// The first item of the tuple is the numerator of the cashdrop rate, second
		/// item is the denominator, cashdrop_rate = numerator / denominator,
		/// use (u32, u32) over `Rate` type to minimize internal division
		/// operation.
		type GetCashDropRates: GetByKey<CurrencyId, CashDropRate>;

		#[pallet::constant]
		/// The default minimum transfer value to secure settpay from dusty claims.
		type DefaultMinimumClaimableTransfer: Get<Balance>;

		/// SERP Treasury for depositing cashdrop.
		type SerpTreasury: SerpTreasury<Self::AccountId, Balance = Balance, CurrencyId = CurrencyId>;

		/// The origin which may update parameters and handle
		/// serplus/standard/reserve. Root can always do this.
		type UpdateOrigin: EnsureOrigin<Self::Origin>;

		#[pallet::constant]
		/// The SERP Treasury's module id, keeps serplus and reserve asset.
		type PalletId: Get<PalletId>;

		/// Weight information for the extrinsics in this module.
		type WeightInfo: WeightInfo;
	}

	#[pallet::error]
	pub enum Error<T> {
		/// Invalid Currency Type.
		InvalidCurrencyType,
		/// CashDrop is not available.
		CashdropNotAvailable,
		/// Transfer is too low for CashDrop.
		TransferTooLowForCashDrop
	}

	#[pallet::event]
	#[pallet::generate_deposit(fn deposit_event)]
	pub enum Event<T: Config> {
		/// CashDrop has been completed successfully.
		CashDrops(CurrencyId, AccountId, Balance),
	}

	/// Mapping to Minimum Claimable Transfer.
	#[pallet::storage]
	#[pallet::getter(fn minimum_claimable_transfer)]
	pub type MinimumClaimableTransfer<T: Config> = StorageMap<_, Twox64Concat, CurrencyId, Balance, OptionQuery>;

	#[pallet::pallet]
	pub struct Pallet<T>(_);

	#[pallet::hooks]
	impl<T: Config> Hooks<T::BlockNumber> for Pallet<T> {}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		#[pallet::weight(<T as Config>::WeightInfo::update_minimum_claimable_transfer(
			currency.len() as CurrencyId, amount.len() as Balance))]
		#[transactional]
		pub fn update_minimum_claimable_transfer(
			origin: OriginFor<T>,
			currency: CurrencyId,
			amount: Balance,
		) -> DispatchResultWithPostInfo {
			T::UpdateOrigin::ensure_origin(origin)?;
			for (currency, amount) in MinimumClaimableTransfer::<T>::iter() {
				MinimumClaimableTransfer::<T>::insert(currency, amount);
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

	pub fn get_cashdrop_rate(currency_id: CurrencyId) -> (u32, u32) {
		ensure!(
			T::RewardableCurrencyIds.contains(currency_id),
			Error::<T>::InvalidCurrencyType,
		);
		T::GetCashDropRates::get(currency_id).unwrap_or_else(T::DefaultCashDropRate::get)
	}

	pub fn get_minimum_claimable_transfer(currency_id: CurrencyId) -> Balance {
		ensure!(
			T::RewardableCurrencyIds.contains(currency_id),
			Error::<T>::InvalidCurrencyType,
		);
		Self::minimum_claimable_transfer(currency_id).unwrap_or_else(T::DefaultMinimumClaimableTransfer::get)
	}
}

impl<T: Config> CashDrop<AccountId> for Pallet<T> {
	type Balance = Balance;
	type CurrencyId = CurrencyId;

	/// claim cashdrop of `currency_id` relative to `transfer_amount` for `who`
	fn claim_cashdrop(currency_id: CurrencyId, who: &AccountId, transfer_amount: Balance) -> DispatchResult {
		ensure!(
			T::RewardableCurrencyIds.contains(currency_id),
			Error::<T>::InvalidCurrencyType,
		);
		let minimum_claimable_transfer = Self::get_minimum_claimable_transfer(currency_id);
		ensure!(
			transfer_amount >= minimum_claimable_transfer,
			Error::<T>::TransferTooLowForCashDrop,
		);
		let setter_fixed_price = T::Price::get_setter_fixed_price();

		if currency_id == T::SetterCurrencyId::get() {
			let (cashdrop_numerator, cashdrop_denominator) = Self::get_cashdrop_rate(currency_id);
			let transfer_drop = transfer_amount.saturating_mul(cashdrop_denominator.saturating_sub(cashdrop_numerator).unique_saturated_into());
			let into_cashdrop_amount: U256 = U256::from(transfer_amount).saturating_sub(U256::from(transfer_drop));
			let balance_cashdrop_amount = into_cashdrop_amount.and_then(|n| TryInto::<Balance>::try_into(n).ok()).unwrap_or_else(Zero::zero);
			let serp_balance = T::Currency::free_balance(currency_id, &Self::account_id());
			ensure!(
				balance_cashdrop_amount <= serp_balance,
				Error::<T>::CashdropNotAvailable,
			);

			Self::deposit_setter_drop(who, balance_cashdrop_amount)?;
		} else if T::NonStableDropCurrencyIds.contains(currency_id) {
			let (cashdrop_numerator, cashdrop_denominator) = Self::get_cashdrop_rate(currency_id);
			let transfer_drop = transfer_amount.saturating_mul(cashdrop_denominator.saturating_sub(cashdrop_numerator).unique_saturated_into());
			let into_cashdrop_amount: U256 = U256::from(transfer_amount).saturating_sub(U256::from(transfer_drop));
			let balance_cashdrop_amount = into_cashdrop_amount.and_then(|n| TryInto::<Balance>::try_into(n).ok()).unwrap_or_else(Zero::zero);
			let serp_balance = T::Currency::free_balance(T::SetterCurrencyId::get(), &Self::account_id());
			ensure!(
				balance_cashdrop_amount <= serp_balance,
				Error::<T>::CashdropNotAvailable,
			);
			let currency_price = T::Price::get_price(currency_id);
			let relative_price = currency_price.saturating_mul(&setter_fixed_price);
			let relative_cashdrop = relative_price.saturating_mul(balance_cashdrop_amount);

			Self::deposit_setter_drop(who, relative_cashdrop)?;
		} else if T::SetCurrencyDropCurrencyIds.contains(currency_id) {
			let (cashdrop_numerator, cashdrop_denominator) = Self::get_cashdrop_rate(currency_id);
			let transfer_drop = transfer_amount.saturating_mul(cashdrop_denominator.saturating_sub(cashdrop_numerator).unique_saturated_into());
			let into_cashdrop_amount: U256 = U256::from(transfer_amount).saturating_sub(U256::from(transfer_drop));
			let balance_cashdrop_amount = into_cashdrop_amount.and_then(|n| TryInto::<Balance>::try_into(n).ok()).unwrap_or_else(Zero::zero);
			let serp_balance = T::Currency::free_balance(currency_id, &Self::account_id());
			ensure!(
				balance_cashdrop_amount <= serp_balance,
				Error::<T>::CashdropNotAvailable,
			);

			Self::deposit_settcurrency_drop(currency_id, who, balance_cashdrop_amount)?;
		}
		Ok(())
	}

	/// deposit cashdrop of `SETT` of `cashdrop_amount` to `who`
	fn deposit_setter_drop(who: &AccountId, cashdrop_amount: Balance) -> DispatchResult {
		T::Currency::transfer(T::SetterCurrencyId::get(), &Self::account_id(), who, cashdrop_amount);
		Self::deposit_event(Event::CashDrops(T::SetterCurrencyId::get(), who, cashdrop_amount));
	}

	/// deposit cashdrop of `currency_id` of `cashdrop_amount` to `who`
	fn deposit_settcurrency_drop(currency_id: CurrencyId, who: &AccountId, cashdrop_amount: Balance) -> DispatchResult {
		T::Currency::transfer(currency_id, &Self::account_id(), who, cashdrop_amount);
		Self::deposit_event(Event::CashDrops(currency_id, who, cashdrop_amount));
	}
}
