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
//! the Setter (SETR) currency basket price. 
//! Process include:
//!   - specify a fixed price for stable currencies
//!   - specify the Setter basket currency price
//!   - feed price in USD or related price bewteen two currencies
//!   - lock/unlock the price data get from oracle

#![cfg_attr(not(feature = "std"), no_std)]
#![allow(clippy::unused_unit)]

use frame_support::{pallet_prelude::*, transactional};
use frame_system::pallet_prelude::*;
use orml_traits::{DataFeeder, DataProvider, MultiCurrency};
use primitives::{
	currency::DexShare,
	Balance, CurrencyId, TokenSymbol,
};
use sp_core::U256;
use sp_runtime::{
	traits::CheckedDiv,
	FixedPointNumber
};
use sp_std::{
	convert::TryInto,
};
use support::{CurrencyIdMapping, DEXManager, Price, PriceProvider};

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
		/// The Setter currency id, it should be SETR in Setheum.
		type SetterCurrencyId: Get<CurrencyId>;

		#[pallet::constant]
		/// The SettUSD currency id, it should be SETUSD in Setheum.
		type GetSettUSDCurrencyId: Get<CurrencyId>;

		#[pallet::constant]
		/// The FiatUSD currency id, it should be USD.
		type GetFiatUSDCurrencyId: Get<CurrencyId>;

		#[pallet::constant]
		/// The FiatEUR currency id, it should be EUR.
		type GetFiatEURCurrencyId: Get<CurrencyId>;

		#[pallet::constant]
		/// The FiatGBP currency id, it should be GBP.
		type GetFiatGBPCurrencyId: Get<CurrencyId>;

		#[pallet::constant]
		/// The FiatCHF currency id, it should be CHF.
		type GetFiatCHFCurrencyId: Get<CurrencyId>;

		#[pallet::constant]
		/// The FiatSAR currency id, it should be SAR.
		type GetFiatSARCurrencyId: Get<CurrencyId>;

		/// The fixed price of SettUsd, it should be 1 USD in Setheum.
		/// This represents the value of the US dollar which is used -
		/// for price feed provided in USD by the oracles in Setheum.
		#[pallet::constant]
		type FiatUsdFixedPrice: Get<Price>;

		#[pallet::constant]
		/// The Setter Peg One currency id, it should be SETUSD in Setheum.
		type GetSetterPegOneCurrencyId: Get<CurrencyId>;

		#[pallet::constant]
		/// The Setter Peg Two currency id, it should be SETGBP in Setheum.
		type GetSetterPegTwoCurrencyId: Get<CurrencyId>;

		#[pallet::constant]
		/// The Setter Peg Three currency id, it should be SETEUR in Setheum.
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
		/// The Setter Peg Nine currency id, it should be SETCHF in Setheum.
		type GetSetterPegNineCurrencyId: Get<CurrencyId>;

		#[pallet::constant]
		/// The Setter Peg Ten currency id, it should be GIPJ in Setheum.
		type GetSetterPegTenCurrencyId: Get<CurrencyId>;

		/// The origin which may lock and unlock prices feed to system.
		type LockOrigin: EnsureOrigin<Self::Origin>;

		/// DEX provide liquidity info.
		type DEX: DEXManager<Self::AccountId, CurrencyId, Balance>;

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
	///
	/// map CurrencyId => Option<Price>
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
		pub fn lock_price(origin: OriginFor<T>, currency_id: CurrencyId) -> DispatchResult {
			T::LockOrigin::ensure_origin(origin)?;
			<Pallet<T> as PriceProvider<CurrencyId>>::lock_price(currency_id);
			Ok(())
		}

		/// Unlock the price and get the price from `PriceProvider` again
		///
		/// The dispatch origin of this call must be `LockOrigin`.
		///
		/// - `currency_id`: currency type.
		#[pallet::weight((T::WeightInfo::unlock_price(), DispatchClass::Operational))]
		#[transactional]
		pub fn unlock_price(origin: OriginFor<T>, currency_id: CurrencyId) -> DispatchResult {
			T::LockOrigin::ensure_origin(origin)?;
			<Pallet<T> as PriceProvider<CurrencyId>>::unlock_price(currency_id);
			Ok(())
		}
	}
}

impl<T: Config> PriceProvider<CurrencyId> for Pallet<T> {
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

	/// get the exchange rate of a specific SetCurrency to USD
	/// Note: this returns the price for 1 basic unit
	/// For the SERP TO USE WHEN STABILISING SetCurrency prices.
	fn get_market_price(currency_id: CurrencyId) -> Option<Price>{
		let maybe_feed_price = if let CurrencyId::DexShare(symbol_0, symbol_1) = currency_id {
			let token_0 = match symbol_0 {
				DexShare::Token(token) => CurrencyId::Token(token),
				DexShare::Erc20(address) => CurrencyId::Erc20(address),
			};
			let token_1 = match symbol_1 {
				DexShare::Token(token) => CurrencyId::Token(token),
				DexShare::Erc20(address) => CurrencyId::Erc20(address),
			};
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

	/// get the exchange rate of a specific SetCurrency peg to USD
	/// Note: this returns the price for 1 basic unit
	/// For the SERP TO USE WHEN STABILISING SetCurrency peg prices.
	/// The setcurrency_id matching to its peg,
	fn get_peg_price(currency_id: CurrencyId) -> Option<Price> {
		let maybe_feed_price = if currency_id == T::SetterCurrencyId::get() {
			Self::get_setter_price()
		} else if let CurrencyId::Token(TokenSymbol::SETUSD) = currency_id {
			// if locked price exists, return it, otherwise return latest price from oracle.
			Self::locked_price(T::GetFiatUSDCurrencyId::get()).or_else(|| T::Source::get(&T::GetFiatUSDCurrencyId::get()))
		} else if let CurrencyId::Token(TokenSymbol::SETEUR) = currency_id {
			// if locked price exists, return it, otherwise return latest price from oracle.
			Self::locked_price(T::GetFiatUSDCurrencyId::get()).or_else(|| T::Source::get(&T::GetFiatUSDCurrencyId::get()))
		} else if let CurrencyId::Token(TokenSymbol::SETGBP) = currency_id {
			// if locked price exists, return it, otherwise return latest price from oracle.
			Self::locked_price(T::GetFiatUSDCurrencyId::get()).or_else(|| T::Source::get(&T::GetFiatUSDCurrencyId::get()))
		} else if let CurrencyId::Token(TokenSymbol::SETCHF) = currency_id {
			// if locked price exists, return it, otherwise return latest price from oracle.
			Self::locked_price(T::GetFiatUSDCurrencyId::get()).or_else(|| T::Source::get(&T::GetFiatUSDCurrencyId::get()))
		} else if let CurrencyId::Token(TokenSymbol::SETSAR) = currency_id {
			// if locked price exists, return it, otherwise return latest price from oracle.
			Self::locked_price(T::GetFiatUSDCurrencyId::get()).or_else(|| T::Source::get(&T::GetFiatUSDCurrencyId::get()))
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

	/// get the exchange rate of a specific SetCurrency peg to USD
	/// Note: this returns the price for 1 basic unit
	/// For the SERP TO USE WHEN STABILISING SetCurrency peg prices.
	fn get_setter_price() -> Option<Price> {
		if let (Some(price_a),
			Some(price_b),
			Some(price_c),
			Some(price_d),
			Some(price_e),
			Some(price_f),
			Some(price_g),
			Some(price_h),
			Some(price_i),
			Some(price_j)
		) = (
			Self::get_price(T::GetSetterPegOneCurrencyId::get()),
			Self::get_price(T::GetSetterPegTwoCurrencyId::get()),
			Self::get_price(T::GetSetterPegThreeCurrencyId::get()),
			Self::get_price(T::GetSetterPegFourCurrencyId::get()),
			Self::get_price(T::GetSetterPegFiveCurrencyId::get()),
			Self::get_price(T::GetSetterPegSixCurrencyId::get()),
			Self::get_price(T::GetSetterPegSevenCurrencyId::get()),
			Self::get_price(T::GetSetterPegEightCurrencyId::get()),
			Self::get_price(T::GetSetterPegNineCurrencyId::get()),
			Self::get_price(T::GetSetterPegTenCurrencyId::get()))
		{
			setter_price(
				price_a,
				price_b,
				price_c,
				price_d,
				price_e,
				price_f,
				price_g,
				price_h,
				price_i,
				price_j,
			)
		} else {
			None
		}
	}

	// /// Get the SERP-TES supply needed to be serped
	// fn on_serp_tes(
	// 	currency_id: CurrencyId
	// ) -> Balance {
	// 	let (Some(peg_price), Some(market_price)) = (
	// 		Self::get_peg_price(currency_id), Self::get_market_price(currency_id)
	// 	);
	// 		
	// 	let total_supply = T::Currency::total_issuance(currency_id);
	// 	get_supply_change(
	// 		market_price,
	// 		peg_price,
	// 		total_supply
	// 	)
	// }

	/// get the exchange rate of specific currency to USD
	/// Note: this returns the price for 1 basic unit
	fn get_price(currency_id: CurrencyId) -> Option<Price> {
		let maybe_feed_price = if currency_id == T::SetterCurrencyId::get() {
			Self::get_setter_price()
		} else if let CurrencyId::DexShare(symbol_0, symbol_1) = currency_id {
			let token_0 = match symbol_0 {
				DexShare::Token(token) => CurrencyId::Token(token),
				DexShare::Erc20(address) => CurrencyId::Erc20(address),
			};
			let token_1 = match symbol_1 {
				DexShare::Token(token) => CurrencyId::Token(token),
				DexShare::Erc20(address) => CurrencyId::Erc20(address),
			};
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

/// Get the price of a Setter (SETR basket coin - basket of currencies) -
/// aggregate the setter price.
/// the final price = total_price_of_basket(all currencies prices combined) -
/// divided by the amount of currencies in the basket.
/// setter_basket_peg_price = 1 * (peg_one_price * ... * peg_three_price) / 10 * 1
fn setter_price(
	price_a: Price,
	price_b: Price,
	price_c: Price,
	price_d: Price,
	price_e: Price,
	price_f: Price,
	price_g: Price,
	price_h: Price,
	price_i: Price,
	price_j: Price,
) -> Option<Price> {
	let one: Balance = 1;
	let total_shares: Balance = 10;
	U256::from(one)
		.saturating_mul(
			U256::from(price_a.into_inner())
				.saturating_add(U256::from(price_b.into_inner()))
				.saturating_add(U256::from(price_c.into_inner()))
				.saturating_add(U256::from(price_d.into_inner()))
				.saturating_add(U256::from(price_e.into_inner()))
				.saturating_add(U256::from(price_f.into_inner()))
				.saturating_add(U256::from(price_g.into_inner()))
				.saturating_add(U256::from(price_h.into_inner()))
				.saturating_add(U256::from(price_i.into_inner()))
				.saturating_add(U256::from(price_j.into_inner()))
		)
		.checked_div(U256::from(total_shares))
		.and_then(|n| n.checked_mul(U256::from(1)))
		.and_then(|r| TryInto::<u128>::try_into(r).ok())
		.map(Price::from_inner)
}

// Get the SERP-TES supply needed to be serped
// fn serp_tes_fn<T: UniqueSaturatedInto<Balance>>(
// 	currency_id: CurrencyId,
// 	total_supply: Balance,
// 	peg_price: Price,
// 	market_price: Price,
// ) -> Balance {
// 	let peg_to_supply = total_supply
// 		.saturating_mul(peg_price.try_into(u128));
// 	let market_to_supply = total_supply.try_into(u128)
// 		.saturating_mul(market_price.try_into(u128));
// 	let relative_supply_change = total_supply
// 		.saturating_mul(market_to_supply.checked_div(peg_to_supply));
// 
// 	relative_supply_change
// 		.and_then(|n| TryInto::<Balance>::try_into(n).ok())
// 		.unwrap_or_else(Zero::zero)
// }


// Calculate the amount of supply change from a fraction given as `coin_price`, `peg_price` and  `supply`.
// fn get_supply_change(coin_price: SubFixedU128<U128>, peg_price: SubFixedU128<U128>, supply: Balance) -> Balance {
// 	type Fix = SubFixedU128<U128>;
// 	let fraction = Fix::from_num(coin_price) / Fix::from_num(peg_price) - Fix::from_num(1);
// 	fraction.saturating_mul_int(supply as u128).to_num::<u128>()
// }
