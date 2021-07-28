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

//! # Prices Module
//!
//! ## Overview
//!
//! The data from Oracle cannot be used in production, prices module will 
//! do some process and feed prices for setheum, this includes constructing 
//! the Setter (SETT) currency basket price. 
//! Process include:
//!   - specify a fixed price for stable currencies
//!   - specify the Setter basket currency price
//!   - feed price in USD or related price bewteen two currencies
//!   - lock/unlock the price data get from oracle

#![cfg_attr(not(feature = "std"), no_std)]
#![allow(clippy::unused_unit)]

use frame_support::{pallet_prelude::*, transactional};
use frame_system::pallet_prelude::*;
use orml_traits::{DataFeeder, DataProvider, GetByKey, MultiCurrency};
use primitives::{
	currency::{DexShare},
	Balance, CurrencyId,
};
use sp_core::U256;
use sp_runtime::{
	traits::{CheckedDiv, CheckedMul},
	FixedPointNumber,
};
use sp_std::{convert::TryInto, prelude::*, vec};
use support::{CurrencyIdMapping, DEXManager, ExchangeRateProvider, Price, PriceProvider};

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

		/// The data source, such as Oracle.
		type Source: DataProvider<CurrencyId, Price> + DataFeeder<CurrencyId, Price, Self::AccountId>;

		#[pallet::constant]
		/// The Setter currency id, it should be SETT in Setheum.
		type SetterCurrencyId: Get<CurrencyId>;

		#[pallet::constant]
		/// The SettUSD currency id, it should be USDJ in Setheum.
		type GetSettUSDCurrencyId: Get<CurrencyId>;

		/// The stable currency ids
		type StableCurrencyIds: Get<Vec<CurrencyId>>;

		/// The peg currency of a stablecoin.
		type PegCurrencyIds: GetByKey<CurrencyId, CurrencyId>;

		/// The list of valid Fiat currency types that define the stablecoin pegs
		type FiatCurrencyIds: Get<Vec<CurrencyId>>;

		#[pallet::constant]
		/// The FiatUSD currency id, it should be USD.
		type GetFiatUSDCurrencyId: Get<CurrencyId>;

		/// The fixed price of SettUsd, it should be 1 USD in Setheum.
		/// This represents the value of the US dollar which is used -
		/// for price feed provided in USD by the oracles in Setheum.
		#[pallet::constant]
		type FiatUsdFixedPrice: Get<Price>;

		#[pallet::constant]
		/// The Setter Peg One currency id, it should be USDJ in Setheum.
		type GetSetterPegOneCurrencyId: Get<CurrencyId>;

		#[pallet::constant]
		/// The Setter Peg Two currency id, it should be GBPJ in Setheum.
		type GetSetterPegTwoCurrencyId: Get<CurrencyId>;

		#[pallet::constant]
		/// The Setter Peg Three currency id, it should be EURJ in Setheum.
		type GetSetterPegThreeCurrencyId: Get<CurrencyId>;

		#[pallet::constant]
		/// The Setter Peg Four currency id, it should be KWDJ in Setheum.
		type GetSetterPegFourCurrencyId: Get<CurrencyId>;

		#[pallet::constant]
		/// The Setter Peg Five currency id, it should be JODJ in Setheum.
		type GetSetterPegFiveCurrencyId: Get<CurrencyId>;

		#[pallet::constant]
		/// The Setter Peg Six currency id, it should be BHDJ in Setheum.
		type GetSetterPegSixCurrencyId: Get<CurrencyId>;

		#[pallet::constant]
		/// The Setter Peg Seven currency id, it should be KYDJ in Setheum.
		type GetSetterPegSevenCurrencyId: Get<CurrencyId>;

		#[pallet::constant]
		/// The Setter Peg Eight currency id, it should be OMRJ in Setheum.
		type GetSetterPegEightCurrencyId: Get<CurrencyId>;

		#[pallet::constant]
		/// The Setter Peg Nine currency id, it should be CHFJ in Setheum.
		type GetSetterPegNineCurrencyId: Get<CurrencyId>;

		#[pallet::constant]
		/// The Setter Peg Ten currency id, it should be GIPJ in Setheum.
		type GetSetterPegTenCurrencyId: Get<CurrencyId>;

		/// The origin which may lock and unlock prices feed to system.
		type LockOrigin: EnsureOrigin<Self::Origin>;

		/// DEX to provide liquidity info.
		type DEX = DEXManager<Self::AccountId, CurrencyId, Balance>;

		/// Currency provide the total insurance of LPToken.
		type Currency: MultiCurrency<Self::AccountId, CurrencyId = CurrencyId, Balance = Balance>;

		/// Mapping between CurrencyId and ERC20 address so user can use Erc20.
		type CurrencyIdMapping: CurrencyIdMapping;

		/// Weight information for the extrinsics in this module.
		type WeightInfo: WeightInfo;
	}

	#[pallet::error]
	pub enum Error<T> {
		/// Invalid fiat currency id
		InvalidFiatCurrencyType,
		/// Invalid stable currency id
		InvalidCurrencyType,
		/// Invalid peg pair (peg-to-currency-by-key-pair)
		InvalidPegPair,
	}

	#[pallet::event]
	#[pallet::generate_deposit(pub(crate) fn deposit_event)]
	pub enum Event<T: Config> {
		/// Lock price. \[currency_id, locked_price\]
		LockPrice(CurrencyId, Price),
		/// Unlock price. \[currency_id\]
		UnlockPrice(CurrencyId),
	}

	/// Mapping from currency id to it's locked price
	#[pallet::storage]
	#[pallet::getter(fn locked_price)]
	pub type LockedPrice<T: Config> = StorageMap<_, Twox64Concat, CurrencyId, Price, OptionQuery>;

	#[pallet::pallet]
	pub struct Pallet<T>(_);

	#[pallet::hooks]
	impl<T: Config> Hooks<T::BlockNumber> for Pallet<T> {}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// Lock the price and feed it to system.
		///
		/// The dispatch origin of this call must be `LockOrigin`.
		///
		/// - `currency_id`: currency type.
		#[pallet::weight((T::WeightInfo::lock_price(), DispatchClass::Operational))]
		#[transactional]
		pub fn lock_price(origin: OriginFor<T>, currency_id: CurrencyId) -> DispatchResultWithPostInfo {
			T::LockOrigin::ensure_origin(origin)?;
			<Pallet<T> as PriceProvider<CurrencyId>>::lock_price(currency_id);
			Ok(().into())
		}

		/// Unlock the price and get the price from `PriceProvider` again
		///
		/// The dispatch origin of this call must be `LockOrigin`.
		///
		/// - `currency_id`: currency type.
		#[pallet::weight((T::WeightInfo::unlock_price(), DispatchClass::Operational))]
		#[transactional]
		pub fn unlock_price(origin: OriginFor<T>, currency_id: CurrencyId) -> DispatchResultWithPostInfo {
			T::LockOrigin::ensure_origin(origin)?;
			<Pallet<T> as PriceProvider<CurrencyId>>::unlock_price(currency_id);
			Ok(().into())
		}
	}
}

impl<T: Config> PriceProvider<CurrencyId> for Pallet<T> {
	/// get stablecoin's fiat peg currency type by id
	fn get_peg_currency_by_currency_id(currency_id: CurrencyId) -> Self::CurrencyId {
		T::PegCurrencyIds::get(&currency_id)
	}

	/// get the price of a stablecoin's fiat peg
	fn get_peg_price(currency_id: CurrencyId) -> Option<Price> {
		ensure!(
			T::StableCurrencyIds::get().contains(&currency_id),
			Error::<T>::InvalidCurrencyType,
		);
		let fiat_currency_id = Self::get_peg_currency_by_currency_id(&currency_id);
		ensure!(
			T::FiatCurrencyIds::get().contains(&fiat_currency_id),
			Error::<T>::InvalidFiatCurrencyType,
		);
		ensure!(
			T::PegCurrencyIds::get(&currency_id) == &fiat_currency_id,
			Error::<T>::InvalidPegPair,
		);
		if currency_id == T::SetterCurrencyId::get() {
			Self::get_setter_fixed_price()
		} else if currency_id == T::GetSettUSDCurrencyId::get() {
			Self::get_settusd_fixed_price()
		} else {
			// if locked price exists, return it, otherwise return latest price from oracle.
			Self::locked_price(fiat_currency_id).or_else(|| T::Source::get(&fiat_currency_id));
		}
	}

	/// get the price of a fiat currency
	fn get_fiat_price(fiat_currency_id: CurrencyId) -> Option<Price>{
		ensure!(
			!T::StableCurrencyIds::get().contains(&fiat_currency_id),
			Error::<T>::InvalidFiatCurrencyType,
		);
		ensure!(
			T::FiatCurrencyIds::get().contains(&fiat_currency_id),
			Error::<T>::InvalidFiatCurrencyType,
		);
		if fiat_currency_id == T::GetFiatUSDCurrencyId::get() {
			Self::get_fiat_usd_fixed_price()
		}
		// if locked price exists, return it, otherwise return latest price from oracle.
		Self::locked_price(fiat_currency_id).or_else(|| T::Source::get(&fiat_currency_id));
	}

	fn get_fiat_usd_fixed_price() -> Option<Price>{
		Some(T::FiatUsdFixedPrice::get())
	}
	fn get_settusd_fixed_price() -> Option<Price>{
		Self::get_fiat_usd_fixed_price()
	}

	/// get the fixed price of a specific settcurrency/stablecoin currency type
	fn get_stablecoin_fixed_price(currency_id: CurrencyId) -> Option<Price> {
		ensure!(
			T::StableCurrencyIds::get().contains(&currency_id),
			Error::<T>::InvalidCurrencyType,
		);
		Self::get_peg_price(&currency_id)
	}

	/// get the market price (not fixed price, for SERP-TES) of a
	/// specific settcurrency/stablecoin currency type from oracle.
	fn get_stablecoin_market_price(currency_id: CurrencyId) -> Option<Price> {
		ensure!(
			T::StableCurrencyIds::get().contains(&currency_id),
			Error::<T>::InvalidCurrencyType,
		);
		Self::get_market_price(currency_id)
	}

	/// get exchange rate between two currency types
	/// Note: this returns the price for 1 basic unit
	fn get_relative_price(base_currency_id: CurrencyId, quote_currency_id: CurrencyId) -> Option<Price> {
		if let (Some(base_price), Some(quote_price)) =
			(Self::get_price(base_currency_id), Self::get_price(quote_currency_id))
		{
			base_price.checked_div(&quote_price)
		} else {
			None
		}
	}

	/// get exchange rate between two currency types
	/// Note: this returns the price for 1 basic unit
	fn get_market_relative_price(base_currency_id: CurrencyId, quote_currency_id: CurrencyId) -> Option<Price> {
		if let (Some(base_price), Some(quote_price)) =
			(Self::get_market_price(base_currency_id), Self::get_market_price(quote_currency_id))
		{
			base_price.checked_div(&quote_price)
		} else {
			None
		}
	}

	fn get_coin_to_peg_relative_price(currency_id: CurrencyId) -> Option<Price> {
		ensure!(
			T::StableCurrencyIds::get().contains(&currency_id),
			Error::<T>::InvalidCurrencyType,
		);
		if currency_id == T::SetterCurrencyId::get() {
			let basket_price = Self::get_setter_fixed_price();
			let coin_price = Self::get_market_price(currency_id);
			coin_price.checked_div(&basket_price)
		} else if !currency_id == T::SetterCurrencyId::get() {
			let fiat_currency_id = Self::get_peg_currency_by_currency_id(&currency_id);
			ensure!(
				T::FiatCurrencyIds::get().contains(&fiat_currency_id),
				Error::<T>::InvalidFiatCurrencyType,
			);
			Self::get_market_relative_price(&currency_id, &fiat_currency_id)
		} else {
			None
		}
	}

	/// Get the price of a Setter (SETT basket coin - basket of currencies) -
	/// aggregate the setter price.
	/// the final price = total_price_of_basket(all currencies prices combined) -
	/// divided by the amount of currencies in the basket.
	fn get_setter_basket_peg_price() -> Option<Price> {
		/// pegged to Pound Sterling (GBP)
		let peg_one_currency_id: CurrencyId = T::GetSetterPegOneCurrencyId::get();
		/// pegged to Euro (EUR)
		let peg_two_currency_id: CurrencyId = T::GetSetterPegTwoCurrencyId::get();
		/// pegged to Kuwaiti Dinar (KWD)
		let peg_three_currency_id: CurrencyId = T::GetSetterPegThreeCurrencyId::get();
		/// pegged to Jordanian Dinar (JOD)
		let peg_four_currency_id: CurrencyId = T::GetSetterPegFourCurrencyId::get();
		/// pegged to Bahraini Dirham (BHD)
		let peg_five_currency_id: CurrencyId = T::GetSetterPegFiveCurrencyId::get();
		/// pegged to Cayman Islands Dollar (KYD)
		let peg_six_currency_id: CurrencyId = T::GetSetterPegSixCurrencyId::get();
		/// pegged to Omani Riyal (OMR)
		let peg_seven_currency_id: CurrencyId = T::GetSetterPegSevenCurrencyId::get();
		/// pegged to Swiss Franc (CHF)
		let peg_eight_currency_id: CurrencyId = T::GetSetterPegEightCurrencyId::get();
		/// pegged to Gibraltar Pound (GIP)
		let peg_nine_currency_id: CurrencyId = T::GetSetterPegNineCurrencyId::get();
		/// pegged to US Dollar (USD)
		let peg_ten_currency_id: CurrencyId = T::GetSetterPegTenCurrencyId::get();

		let peg_one_price = Self::get_fiat_price(&peg_one_currency_id);
		let peg_two_price = Self::get_fiat_price(&peg_two_currency_id);
		let peg_three_price = Self::get_fiat_price(&peg_three_currency_id);
		let peg_four_price = Self::get_fiat_price(&peg_four_currency_id);
		let peg_five_price = Self::get_fiat_price(&peg_five_currency_id);
		let peg_six_price = Self::get_fiat_price(&peg_six_currency_id);
		let peg_seven_price = Self::get_fiat_price(&peg_seven_currency_id);
		let peg_eight_price = Self::get_fiat_price(&peg_eight_currency_id);
		let peg_nine_price = Self::get_fiat_price(&peg_nine_currency_id);
		let peg_ten_price = Self::get_fiat_price(&peg_ten_currency_id);

		let total_basket_worth: Price = peg_one_price
										+ peg_two_price
										+ peg_three_price
										+ peg_four_price
										+ peg_five_price
										+ peg_six_price
										+ peg_seven_price
										+ peg_eight_price
										+ peg_nine_price
										+ peg_ten_price;
		let currencies_amount: U256 = U256::from(10);
		let basket_worth: U256 = U256::from(&total_basket_worth);
		&basket_worth.checked_div(&currencies_amount).and_then(|n| TryInto::<Balance>::try_into(n).ok())
			.unwrap_or_else(Zero::zero);
	}

	/// Get the fixed price of Setter currency (SETT)
	fn get_setter_fixed_price() -> Option<Price> {
		Self::get_setter_basket_peg_price()
	}

	/// get the exchange rate of a specific SettCurrency to USD
	/// Note: this returns the price for 1 basic unit
	/// For the SERP TO USE WHEN STABILISING SettCurrency prices.
	fn get_market_price(currency_id: CurrencyId) -> Option<Price>{
		let maybe_feed_price = if T::FiatCurrencyIds::get().contains(&currency_id) {
			// if it is a FiatCurrency, return fiat price
			let fiat_currency_id = &currency_id;
			Self::get_fiat_price(&fiat_currency_id);
		} else if let CurrencyId::DexShare(symbol_0, symbol_1) = currency_id {
			let token_0 = match symbol_0 {
				DexShare::Token(token) => CurrencyId::Token(token),
				DexShare::Erc20(address) => CurrencyId::Erc20(address),
			};
			let token_1 = match symbol_1 {
				DexShare::Token(token) => CurrencyId::Token(token),
				DexShare::Erc20(address) => CurrencyId::Erc20(address),
			};
			let (pool_0, _) = T::DEX::get_liquidity_pool(token_0, token_1);
			let total_shares = T::Currency::total_issuance(currency_id);

			return {
				if let (Some(ratio), Some(price_0)) = (
					Price::checked_from_rational(pool_0, total_shares),
					Self::get_price(token_0),
				) {
					ratio
						.checked_mul(&price_0)
						.and_then(|n| n.checked_mul(&Price::saturating_from_integer(2)))
				} else {
					None
				}
			};
		} else {
			// if locked price exists, return it, otherwise return latest price from oracle.
			Self::locked_price(currency_id).or_else(|| T::Source::get(&currency_id))
		};
		let maybe_adjustment_multiplier = 10u128.checked_pow(T::CurrencyIdMapping::decimals(currency_id)?.into());

		if let (Some(feed_price), Some(adjustment_multiplier)) = (maybe_feed_price, maybe_adjustment_multiplier) {
			Price::checked_from_rational(feed_price.into_inner(), adjustment_multiplier)
		} else {
			None
		}
	}

	/// get the exchange rate of specific currency to USD
	/// Note: this returns the price for 1 basic unit
	fn get_price(currency_id: CurrencyId) -> Option<Price> {
		let maybe_feed_price = if T::FiatCurrencyIds::get().contains(&currency_id) {
			// if it is a FiatCurrency, return fiat price
			let fiat_currency_id = &currency_id;
			Self::get_fiat_price(&fiat_currency_id);
		} else if T::StableCurrencyIds::get().contains(&currency_id) {
			// if it is a SettCurrency, return fixed price
			Some(Self::get_stablecoin_fixed_price(&currency_id))
		} else if let CurrencyId::DexShare(symbol_0, symbol_1) = currency_id {
			let token_0 = match symbol_0 {
				DexShare::Token(token) => CurrencyId::Token(token),
				DexShare::Erc20(address) => CurrencyId::Erc20(address),
			};
			let token_1 = match symbol_1 {
				DexShare::Token(token) => CurrencyId::Token(token),
				DexShare::Erc20(address) => CurrencyId::Erc20(address),
			};
			let (pool_0, _) = T::DEX::get_liquidity_pool(token_0, token_1);
			let total_shares = T::Currency::total_issuance(currency_id);


			return {
				if let (Some(price_0), Some(price_1)) = (Self::get_price(token_0), Self::get_price(token_1)) {
					let (pool_0, pool_1) = T::DEX::get_liquidity_pool(token_0, token_1);
					let total_shares = T::Currency::total_issuance(currency_id);
					lp_token_fair_price(total_shares, pool_0, pool_1, price_0, price_1)
				} else {
					None
				}
			};
		} else {
			// if locked price exists, return it, otherwise return latest price from oracle.
			Self::locked_price(currency_id).or_else(|| T::Source::get(&currency_id))
		};
		let maybe_adjustment_multiplier = 10u128.checked_pow(T::CurrencyIdMapping::decimals(currency_id)?.into());

		if let (Some(feed_price), Some(adjustment_multiplier)) = (maybe_feed_price, maybe_adjustment_multiplier) {
			Price::checked_from_rational(feed_price.into_inner(), adjustment_multiplier)
		} else {
			None
		}
	}

	fn lock_price(currency_id: CurrencyId) {
		// lock price when get valid price from source
		if let Some(val) = T::Source::get(&currency_id) {
			LockedPrice::<T>::insert(currency_id, val);
			<Pallet<T>>::deposit_event(Event::LockPrice(currency_id, val));
		}
	}

	fn unlock_price(currency_id: CurrencyId) {
		LockedPrice::<T>::remove(currency_id);
		<Pallet<T>>::deposit_event(Event::UnlockPrice(currency_id));
	}
}

/// The fair price is determined by the external feed price and the size of the liquidity pool:
/// https://blog.alphafinance.io/fair-lp-token-pricing/
/// fair_price = (pool_0 * pool_1)^0.5 * (price_0 * price_1)^0.5 / total_shares * 2
fn lp_token_fair_price(
	total_shares: Balance,
	pool_a: Balance,
	pool_b: Balance,
	price_a: Price,
	price_b: Price,
) -> Option<Price> {
	U256::from(pool_a)
		.saturating_mul(U256::from(pool_b))
		.integer_sqrt()
		.saturating_mul(
			U256::from(price_a.into_inner())
				.saturating_mul(U256::from(price_b.into_inner()))
				.integer_sqrt(),
		)
		.checked_div(U256::from(total_shares))
		.and_then(|n| n.checked_mul(U256::from(2)))
		.and_then(|r| TryInto::<u128>::try_into(r).ok())
		.map(Price::from_inner)
}
