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

use fixed::{types::extra::U64, FixedU128};
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

use serp_ocw::FetchPriceFor

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
		/// Native (DNAR) currency Stablecoin currency id
		type GetNativeCurrencyId: Get<CurrencyId>;

		#[pallet::constant]
		/// SettinDes (DRAM) dexer currency id
		type DirhamCurrencyId: Get<CurrencyId>;

		#[pallet::constant]
		/// The Setter currency id, it should be SETR in Setheum.
		type SetterCurrencyId: Get<CurrencyId>;

		#[pallet::constant]
		/// The SetUSD currency id, it should be SETUSD in Setheum.
		type GetSetUSDCurrencyId: Get<CurrencyId>;

		#[pallet::constant]
		/// The SETEUR currency id, it should be SETEUR in Setheum.
		type GetSetEURCurrencyId: Get<CurrencyId>;

		#[pallet::constant]
		/// The SETGBP currency id, it should be SETGBP in Setheum.
		type GetSetGBPCurrencyId: Get<CurrencyId>;

		#[pallet::constant]
		/// The SETCHF currency id, it should be SETCHF in Setheum.
		type GetSetCHFCurrencyId: Get<CurrencyId>;

		#[pallet::constant]
		/// The SETSAR currency id, it should be SETSAR in Setheum.
		type GetSetSARCurrencyId: Get<CurrencyId>;

		// The Offchain Worker => `serp_ocw`.
		type SerpOcwOffchainPrice: FetchPriceFor;

		/// The fixed price of SetUsd, it should be 1 USD in Setheum.
		/// This represents the value of the US dollar which is used -
		/// for price feed provided in USD by the oracles in Setheum.
		#[pallet::constant]
		type FiatUsdFixedPrice: Get<Price>;

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
		/// No OffChain Price available
		NoOffchainPrice,
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
	impl<T: Config> Hooks<T::BlockNumber> for Pallet<T> {
		///
		/// NOTE: This function is called BEFORE ANY extrinsic in a block is applied,
		/// including inherent extrinsics. Hence for instance, if you runtime includes
		/// `pallet_timestamp`, the `timestamp` is not yet up to date at this point.
		///
		/// Triggers Serping for all system stablecoins at every block.
		fn on_initialize(now: T::BlockNumber) -> Weight {
			// SERP-TES Adjustment Frequency.
			// Schedule for when to trigger SERP-TES
			// (Blocktime/BlockNumber - every blabla block)
			if now % T::SerpTesSchedule::get() == Zero::zero() {
				// SERP TES (Token Elasticity of Supply).
				// Triggers Serping for all system stablecoins to stabilize stablecoin prices.
				let mut count: u32 = 0;
				if Self::get_and_lock_offchain_prices().is_ok() {
					count += 1;
				}

				T::WeightInfo::on_initialize(count)
			} else {
				0
			}
		}
	}

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

	/// get the exchange rate of specific currency to USD
	/// Note: this returns the price for 1 basic unit
	fn get_price(currency_id: CurrencyId) -> Option<Price> {
		let maybe_feed_price = if currency_id == T::SetterCurrencyId::get() {
			Self::get_setter_price()
		} else if currency_id == T::GetSetUSDCurrencyId::get() {
			T::FiatUsdFixedPrice::get()
		} else if currency_id == T::GetSetEURCurrencyId::get() {
			if let Some(val) = Self::get_serp_ocw_price(T::GetSetEURCurrencyId::get()) {
				let val_price: U256 = U256::from(val)
					.and_then(|r| TryInto::<u128>::try_into(r).ok())
					.map(Price::from_inner)
				val_price
			};
		} else if currency_id == T::GetSetGBPCurrencyId::get() {
			if let Some(val) = Self::get_serp_ocw_price(T::GetSetGBPCurrencyId::get()) {
				let val_price: U256 = U256::from(val)
					.and_then(|r| TryInto::<u128>::try_into(r).ok())
					.map(Price::from_inner)
				val_price
			};
		} else if currency_id == T::GetSetCHFCurrencyId::get() {
			if let Some(val) = Self::get_serp_ocw_price(T::GetSetCHFCurrencyId::get()) {
				let val_price: U256 = U256::from(val)
					.and_then(|r| TryInto::<u128>::try_into(r).ok())
					.map(Price::from_inner)
				val_price
			};
		} else if currency_id == T::GetSetSARCurrencyId::get() {
			if let Some(val) = Self::get_serp_ocw_price(T::GetSetSARCurrencyId::get()) {
				let val_price: U256 = U256::from(val)
					.and_then(|r| TryInto::<u128>::try_into(r).ok())
					.map(Price::from_inner)
				val_price
			};
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

	fn get_and_lock_offchain_prices() -> DispatchResult {
		let dinar_currency_id = T::GetNativeCurrencyId::get();
		let dirham_currency_id = T::DirhamCurrencyId::get();
		let renbtc_currency_id = T::RenBtcCurrencyId::get();
		let setter_currency_id = T::SetterCurrencyId::get();
		let setusd_currency_id = T::GetSetUSDCurrencyId::get();
		let seteur_currency_id = T::GetSetEURCurrencyId::get();
		let setgbp_currency_id = T::GetSetGBPCurrencyId::get();
		let setchf_currency_id = T::GetSetCHFCurrencyId::get();
		let setsar_currency_id = T::GetSetSARCurrencyId::get();

		// lock price got from `serp-ocw`
		if let Some(val) = Self::get_serp_ocw_price(&dinar_currency_id) {
			let val_price: U256 = U256::from(val)
				.and_then(|r| TryInto::<u128>::try_into(r).ok())
				.map(Price::from_inner)
			LockedPrice::<T>::insert(dinar_currency_id, val_price);
			<Pallet<T>>::deposit_event(Event::LockPrice(dinar_currency_id, val));
		}
		<Pallet<T>>::deposit_event(Event::OffChainPrice(dinar_currency_id, val));

		// lock price got from `serp-ocw`
		if let Some(val) = Self::get_serp_ocw_price(&dirham_currency_id) {
			let val_price: U256 = U256::from(val)
				.and_then(|r| TryInto::<u128>::try_into(r).ok())
				.map(Price::from_inner)
			LockedPrice::<T>::insert(dirham_currency_id, val_price);
			<Pallet<T>>::deposit_event(Event::LockPrice(dirham_currency_id, val));
		}
		<Pallet<T>>::deposit_event(Event::OffChainPrice(dirham_currency_id, val));

		// lock price got from `serp-ocw`
		if let Some(val) = Self::get_serp_ocw_price(&renbtc_currency_id) {
			let val_price: U256 = U256::from(val)
				.and_then(|r| TryInto::<u128>::try_into(r).ok())
				.map(Price::from_inner)
			LockedPrice::<T>::insert(renbtc_currency_id, val_price);
			<Pallet<T>>::deposit_event(Event::LockPrice(renbtc_currency_id, val));
		}
		<Pallet<T>>::deposit_event(Event::OffChainPrice(renbtc_currency_id, val));

		// lock price got from `serp-ocw`
		if let Some(val) = Self::get_serp_ocw_price(&setter_currency_id) {
			let val_price: U256 = U256::from(val)
				.and_then(|r| TryInto::<u128>::try_into(r).ok())
				.map(Price::from_inner)
			LockedPrice::<T>::insert(setter_currency_id, val_price);
			<Pallet<T>>::deposit_event(Event::LockPrice(setter_currency_id, val));
		}
		<Pallet<T>>::deposit_event(Event::OffChainPrice(setter_currency_id, val));

		// lock price got from `serp-ocw`
		if let Some(val) = Self::get_serp_ocw_price(&setusd_currency_id) {
			let val_price: U256 = U256::from(val)
				.and_then(|r| TryInto::<u128>::try_into(r).ok())
				.map(Price::from_inner)
			LockedPrice::<T>::insert(setusd_currency_id, val_price);
			<Pallet<T>>::deposit_event(Event::LockPrice(setusd_currency_id, val));
		}
		<Pallet<T>>::deposit_event(Event::OffChainPrice(setusd_currency_id, val));

		// lock price got from `serp-ocw`
		if let Some(val) = Self::get_serp_ocw_price(&seteur_currency_id) {
			let val_price: U256 = U256::from(val)
				.and_then(|r| TryInto::<u128>::try_into(r).ok())
				.map(Price::from_inner)
			LockedPrice::<T>::insert(seteur_currency_id, val_price);
			<Pallet<T>>::deposit_event(Event::LockPrice(seteur_currency_id, val));
		}
		<Pallet<T>>::deposit_event(Event::OffChainPrice(seteur_currency_id, val));

		// lock price got from `serp-ocw`
		if let Some(val) = Self::get_serp_ocw_price(&setgbp_currency_id) {
			let val_price: U256 = U256::from(val)
				.and_then(|r| TryInto::<u128>::try_into(r).ok())
				.map(Price::from_inner)
			LockedPrice::<T>::insert(setgbp_currency_id, val_price);
			<Pallet<T>>::deposit_event(Event::LockPrice(setgbp_currency_id, val));
		}
		<Pallet<T>>::deposit_event(Event::OffChainPrice(setgbp_currency_id, val));

		// lock price got from `serp-ocw`
		if let Some(val) = Self::get_serp_ocw_price(&setchf_currency_id) {
			let val_price: U256 = U256::from(val)
				.and_then(|r| TryInto::<u128>::try_into(r).ok())
				.map(Price::from_inner)
			LockedPrice::<T>::insert(setchf_currency_id, val_price);
			<Pallet<T>>::deposit_event(Event::LockPrice(setchf_currency_id, val));
		}
		<Pallet<T>>::deposit_event(Event::OffChainPrice(setchf_currency_id, val));

		// lock price got from `serp-ocw`
		if let Some(val) = Self::get_serp_ocw_price(&setsar_currency_id) {
			let val_price: U256 = U256::from(val)
				.and_then(|r| TryInto::<u128>::try_into(r).ok())
				.map(Price::from_inner)
			LockedPrice::<T>::insert(setsar_currency_id, val_price);
			<Pallet<T>>::deposit_event(Event::LockPrice(setsar_currency_id, val));
		}
		<Pallet<T>>::deposit_event(Event::OffChainPrice(setsar_currency_id, val));
		Ok(())
	}

	fn get_serp_ocw_price(currency_id: CurrencyId) -> u64 {
		let dinar_currency_id = T::GetNativeCurrencyId::get();
		let dirham_currency_id = T::DirhamCurrencyId::get();
		let renbtc_currency_id = T::RenBtcCurrencyId::get();
		let setter_currency_id = T::SetterCurrencyId::get();
		let setusd_currency_id = T::GetSetUSDCurrencyId::get();
		let seteur_currency_id = T::GetSetEURCurrencyId::get();
		let setgbp_currency_id = T::GetSetGBPCurrencyId::get();
		let setchf_currency_id = T::GetSetCHFCurrencyId::get();
		let setsar_currency_id = T::GetSetSARCurrencyId::get();
		match currency_id {
			currency_id if currency_id = dinar_currency_id => {
				let price = T::SerpOcwOffchainPrice::get_price_for(b"DNAR");
				return price
			}
			currency_id if currency_id = dirham_currency_id => {
				let price = T::SerpOcwOffchainPrice::get_price_for(b"DRAM");
				return price
			}
			currency_id if currency_id = renbtc_currency_id => {
				let price = T::SerpOcwOffchainPrice::get_price_for(b"BTC");
				return price
			}
			currency_id if currency_id = setter_currency_id => {
				let price = T::SerpOcwOffchainPrice::get_price_for(b"SETR");
				return price
			}
			currency_id if currency_id = setusd_currency_id => {
				let price = T::SerpOcwOffchainPrice::get_price_for(b"SETUSD");
				return price
			}
			currency_id if currency_id = seteur_currency_id => {
				let price = T::SerpOcwOffchainPrice::get_price_for(b"SETEUR");
				return price
			}
			currency_id if currency_id = setgbp_currency_id => {
				let price = T::SerpOcwOffchainPrice::get_price_for(b"SETGBP");
				return price
			}
			currency_id if currency_id = setchf_currency_id => {
				let price = T::SerpOcwOffchainPrice::get_price_for(b"SETCHF");
				return price
			}
			currency_id if currency_id = setsar_currency_id => {
				let price = T::SerpOcwOffchainPrice::get_price_for(b"SETSAR");
				return price
			}
		}
	}

	fn get_serp_ocw_peg_price(currency_id: CurrencyId) -> u64 {
		let dinar_currency_id = T::GetNativeCurrencyId::get();
		let dirham_currency_id = T::DirhamCurrencyId::get();
		let renbtc_currency_id = T::RenBtcCurrencyId::get();
		let setter_currency_id = T::SetterCurrencyId::get();
		let setusd_currency_id = T::GetSetUSDCurrencyId::get();
		let seteur_currency_id = T::GetSetEURCurrencyId::get();
		let setgbp_currency_id = T::GetSetGBPCurrencyId::get();
		let setchf_currency_id = T::GetSetCHFCurrencyId::get();
		let setsar_currency_id = T::GetSetSARCurrencyId::get();
		match currency_id {
			currency_id if currency_id = renbtc_currency_id => {
				let price = T::SerpOcwOffchainPrice::get_price_for(b"DNAR");
				return price
			}
			currency_id if currency_id = dirham_currency_id => {
				let price = T::SerpOcwOffchainPrice::get_price_for(b"DRAM");
				return price
			}
			currency_id if currency_id = renbtc_currency_id => {
				let price = T::SerpOcwOffchainPrice::get_price_for(b"BTC");
				return price
			}
			currency_id if currency_id = setter_currency_id => {
				let price = Self::fetch_setter_basket();
				return price
			}
			currency_id if currency_id = setusd_currency_id => {
				let price = T::SerpOcwOffchainPrice::get_price_for(b"USD");
				return price
			}
			currency_id if currency_id = seteur_currency_id => {
				let price = T::SerpOcwOffchainPrice::get_price_for(b"EUR");
				return price
			}
			currency_id if currency_id = setgbp_currency_id => {
				let price = T::SerpOcwOffchainPrice::get_price_for(b"GBP");
				return price
			}
			currency_id if currency_id = setchf_currency_id => {
				let price = T::SerpOcwOffchainPrice::get_price_for(b"CHF");
				return price
			}
			currency_id if currency_id = setsar_currency_id => {
				let price = T::SerpOcwOffchainPrice::get_price_for(b"SAR");
				return price
			}
		}
	}

	fn get_setter_price() -> Option<Price> {
		// get setter basket into Option<Price>
		let setter_basket_value: Balance = Self::get_setter_basket();
		let from_setter_price: U256 = U256::from(setter_basket_value)
		.and_then(|r| TryInto::<u128>::try_into(r).ok())
		.map(Price::from_inner)
	}

	fn get_setter_basket() -> Balance {
		// Fetch Setter Basket
		let setter_basket: u64 = fetch_setter_basket();

		// SETTER_BASKET_VALUE
		type Fix = FixedU128<U64>;
		let value = Fix::from_num(setter_basket)
			.saturating_mul_int(1 as u128).to_num::<u128>()
	}

	fn fetch_setter_basket() -> u64 {
		// BASKET_PEG_PRICES
		let price_1 = T::SerpOcwOffchainPrice::get_price_for(b"USD");
		let price_2 = T::SerpOcwOffchainPrice::get_price_for(b"EUR");
		let price_3 = T::SerpOcwOffchainPrice::get_price_for(b"GBP");
		let price_4 = T::SerpOcwOffchainPrice::get_price_for(b"CHF");
		let price_5 = T::SerpOcwOffchainPrice::get_price_for(b"SAR");
		let price_6 = T::SerpOcwOffchainPrice::get_price_for(b"ETH");
		let price_7 = T::SerpOcwOffchainPrice::get_price_for(b"BTC");

		// BASKET_PEG_WEIGHTS - approx. $1 in total weight, making 1 SETR = $1 approx as of impl time (now).
		let weight_1: u64 = 0.25;
		let weight_2: u64 = 0.21;
		let weight_3: u64 = 0.15;
		let weight_4: u64 = 0.092;
		let weight_5: u64 = 0.38;
		let weight_6: u64 = 0.00001543;
		let weight_7: u64 = 0.00000105;

		// BASKET_PEG_SLOTS
		let slot_1 = get_basket_slot_value(price, weight);
		let slot_2 = get_basket_slot_value(price, weight);
		let slot_3 = get_basket_slot_value(price, weight);
		let slot_4 = get_basket_slot_value(price, weight);
		let slot_5 = get_basket_slot_value(price, weight);
		let slot_6 = get_basket_slot_value(price, weight);
		let slot_7 = get_basket_slot_value(price, weight);

		// SETTER_BASKET_VALUE
		let value = (slot_1 + slot_2 + slot_3 + slot_4 + slot_5 + slot_6 + slot_7);
		value
	}

	fn get_supply_change(currency_id: CurrencyId) -> Balance {
		let coin_price = Self::get_serp_ocw_price(&currency_id);
		let peg_price = Self::get_serp_ocw_peg_price(&currency_id);
		let supply = T::Currency::total_issuance(currency_id);

		let supply_change = get_serp_tes(coin_price, peg_price, supply);
		supply_change
	}
	
	fn get_min_target_amount(target_currency_id: CurrencyId, supply_currency_id: CurrencyId, supply_amount: Balance) -> Balance {
		price_0 = Self::get_serp_ocw_price(target_currency_id);
		price_1 = Self::get_serp_ocw_price(supply_currency_id);

		type Fix = FixedU128<U64>;
		let relative_price = Fix::from_num(price_0) / Fix::from_num(price_1);
		let min_target_amount_full = Fix::from_num(supply_amount) / Fix::from_num(relative_price);
		let desired_min_target_amount = Fix::from_num(min_target_amount_full) / Fix::from_num(100);
		desired_min_target_amount.saturating_mul_int(95 as u128).to_num::<u128>()
	}

	fn get_max_supply_amount(target_currency_id: CurrencyId, supply_currency_id: CurrencyId, target_amount: Balance) -> Balance {
		price_0 = Self::get_serp_ocw_price(target_currency_id);
		price_1 = Self::get_serp_ocw_price(supply_currency_id);

		type Fix = FixedU128<U64>;
		let relative_price = Fix::from_num(price_0) / Fix::from_num(price_1);
		let max_supply_amount_full = Fix::from_num(target_amount) / Fix::from_num(relative_price);
		let desired_max_supply_amount = Fix::from_num(min_target_amount_full) / Fix::from_num(100);
		desired_max_supply_amount.saturating_mul_int(105 as u128).to_num::<u128>()
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

// Calculate the amount of supply change from a fraction given as `coin_price`, `peg_price` and  `supply`.
fn get_serp_tes(coin_price: u64, peg_price: u64, supply: Balance) -> Balance {
	type Fix = FixedU128<U64>;
	let fraction = Fix::from_num(coin_price) / Fix::from_num(peg_price) - Fix::from_num(1);
	fraction.saturating_mul_int(supply as u128).to_num::<u128>()
}

// Calculate the value of a slot in the Setter Basket
get_basket_slot_value(
	price,
	weight
) -> u64 {
	type Fix = FixedU128<U64>;
	let slot = Fix::from_num(price) * Fix::from_num(weight);
	slot.to_num::<u64>()
}
