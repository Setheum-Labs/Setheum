// بِسْمِ اللَّهِ الرَّحْمَنِ الرَّحِيم
// ٱلَّذِينَ يَأْكُلُونَ ٱلرِّبَوٰا۟ لَا يَقُومُونَ إِلَّا كَمَا يَقُومُ ٱلَّذِى يَتَخَبَّطُهُ ٱلشَّيْطَـٰنُ مِنَ ٱلْمَسِّ ۚ ذَٰلِكَ بِأَنَّهُمْ قَالُوٓا۟ إِنَّمَا ٱلْبَيْعُ مِثْلُ ٱلرِّبَوٰا۟ ۗ وَأَحَلَّ ٱللَّهُ ٱلْبَيْعَ وَحَرَّمَ ٱلرِّبَوٰا۟ ۚ فَمَن جَآءَهُۥ مَوْعِظَةٌ مِّن رَّبِّهِۦ فَٱنتَهَىٰ فَلَهُۥ مَا سَلَفَ وَأَمْرُهُۥٓ إِلَى ٱللَّهِ ۖ وَمَنْ عَادَ فَأُو۟لَـٰٓئِكَ أَصْحَـٰبُ ٱلنَّارِ ۖ هُمْ فِيهَا خَـٰلِدُونَ

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
//! The data from Oracle cannot be used in business, prices module will do some
//! process and feed prices for Setheum. Process include:
//!   - specify a fixed price for stable currency
//!   - feed price in USD or related price bewteen two currencies
//!   - lock/unlock the price data get from oracle

#![cfg_attr(not(feature = "std"), no_std)]
#![allow(clippy::unused_unit)]

use frame_support::{pallet_prelude::*, transactional};
use frame_system::pallet_prelude::*;
use orml_traits::{DataFeeder, DataProvider, MultiCurrency};
use primitives::{Balance, CurrencyId};
use sp_core::U256;
use sp_runtime::{FixedPointNumber, traits::CheckedDiv};
use sp_std::{convert::TryInto, marker::PhantomData};
use support::{CurrencyIdMapping, DEXManager, LockablePrice, Price, PriceProvider};

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

		/// The stable currency id, it should be SETUSD in Setheum.
		#[pallet::constant]
		type GetSetUSDId: Get<CurrencyId>;

		/// The stable currency id, it should be SETR in Setheum.
		#[pallet::constant]
		type SetterCurrencyId: Get<CurrencyId>;

		/// The fixed prices of stable currency SETUSD, it should be 1 USD in Setheum.
		#[pallet::constant]
		type SetUSDFixedPrice: Get<Price>;

		/// The fixed prices of stable currency SETR, it should be 2 USD in Setheum.
		#[pallet::constant]
		type SetterFixedPrice: Get<Price>;

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
		/// Failed to access price
		AccessPriceFailed,
		/// There's no locked price
		NoLockedPrice,
	}

	#[pallet::event]
	#[pallet::generate_deposit(pub(crate) fn deposit_event)]
	pub enum Event<T: Config> {
		/// Lock price. \[currency_id, locked_price\]
		LockPrice(CurrencyId, Price),
		/// Unlock price. \[currency_id\]
		UnlockPrice(CurrencyId),
		/// Unlock price. \[relative_price\]
		RelativePrice(Option<Price>),
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
			<Pallet<T> as LockablePrice<CurrencyId>>::lock_price(currency_id)?;
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
			<Pallet<T> as LockablePrice<CurrencyId>>::unlock_price(currency_id)?;
			Ok(())
		}

		/// access the exchange rate of specific currency to USD,
		/// it always access the real-time price directly.
		///
		/// Note: this returns the price for 1 basic unit
		#[pallet::weight((T::WeightInfo::lock_price(), DispatchClass::Operational))]
		#[transactional]
		pub fn fetch_price(origin: OriginFor<T>, currency_id: CurrencyId) -> DispatchResultWithPostInfo {
			Self::access_price(currency_id);
			Pallet::<T>::deposit_event(Event::UnlockPrice(currency_id));
			Ok(().into())
		}

		/// access the exchange rate of specific currency to USD,
		/// it always access the real-time price directly.
		///
		/// Note: this returns the price for 1 basic unit
		#[pallet::weight((T::WeightInfo::lock_price(), DispatchClass::Operational))]
		#[transactional]
		pub fn fetch_relative_price(origin: OriginFor<T>, base_currency_id: CurrencyId, quote_currency_id: CurrencyId) -> DispatchResultWithPostInfo {
			let price = Self::get_relative_price(base_currency_id, quote_currency_id);
			Pallet::<T>::deposit_event(Event::RelativePrice(price));
			Ok(().into())
		}
	}
}

impl<T: Config> Pallet<T> {
	/// access the exchange rate of specific currency to USD,
	/// it always access the real-time price directly.
	///
	/// Note: this returns the price for 1 basic unit
	fn access_price(currency_id: CurrencyId) -> Option<Price> {
		let maybe_price = if currency_id == T::GetSetUSDId::get() {
			// if is SETUSD, use fixed price
			Some(T::SetUSDFixedPrice::get())
		} else if currency_id == T::SetterCurrencyId::get() {
			// if is SETR, return Setter fixed price (currently $2.0)
			Some(T::SetterFixedPrice::get())
		} else if let CurrencyId::DexShare(symbol_0, symbol_1) = currency_id {
			let token_0: CurrencyId = symbol_0.into();
			let token_1: CurrencyId = symbol_1.into();

			// directly return the fair price
			return {
				if let (Some(price_0), Some(price_1)) = (Self::access_price(token_0), Self::access_price(token_1)) {
					let (pool_0, pool_1) = T::DEX::get_liquidity_pool(token_0, token_1);
					let total_shares = T::Currency::total_issuance(currency_id);
					lp_token_fair_price(total_shares, pool_0, pool_1, price_0, price_1)
				} else {
					None
				}
			};
		} else {
			// get real-time price from oracle
			T::Source::get(&currency_id)
		};

		let maybe_adjustment_multiplier = 10u128.checked_pow(T::CurrencyIdMapping::decimals(currency_id)?.into());

		if let (Some(price), Some(adjustment_multiplier)) = (maybe_price, maybe_adjustment_multiplier) {
			// return the price for 1 basic unit
			Price::checked_from_rational(price.into_inner(), adjustment_multiplier)
		} else {
			None
		}
	}

	fn get_relative_price(base: CurrencyId, quote: CurrencyId) -> Option<Price> {
		if let (Some(base_price), Some(quote_price)) = (Self::access_price(base), Self::access_price(quote)) {
			base_price.checked_div(&quote_price)
		} else {
			None
		}
	}
}

impl<T: Config> LockablePrice<CurrencyId> for Pallet<T> {
	/// Record the real-time price from oracle as the locked price
	fn lock_price(currency_id: CurrencyId) -> DispatchResult {
		let price = Self::access_price(currency_id).ok_or(Error::<T>::AccessPriceFailed)?;
		LockedPrice::<T>::insert(currency_id, price);
		Pallet::<T>::deposit_event(Event::LockPrice(currency_id, price));
		Ok(())
	}

	/// Unlock the locked price
	fn unlock_price(currency_id: CurrencyId) -> DispatchResult {
		let _ = LockedPrice::<T>::take(currency_id).ok_or(Error::<T>::NoLockedPrice)?;
		Pallet::<T>::deposit_event(Event::UnlockPrice(currency_id));
		Ok(())
	}
}

/// PriceProvider that always provider real-time prices from oracle
pub struct RealTimePriceProvider<T>(PhantomData<T>);
impl<T: Config> PriceProvider<CurrencyId> for RealTimePriceProvider<T> {
	fn get_price(currency_id: CurrencyId) -> Option<Price> {
		Pallet::<T>::access_price(currency_id)
	}
}

/// PriceProvider that priority access to the locked price, if it is none,
/// will access to real-time price
pub struct PriorityLockedPriceProvider<T>(PhantomData<T>);
impl<T: Config> PriceProvider<CurrencyId> for PriorityLockedPriceProvider<T> {
	fn get_price(currency_id: CurrencyId) -> Option<Price> {
		Pallet::<T>::locked_price(currency_id).or_else(|| Pallet::<T>::access_price(currency_id))
	}
}

/// PriceProvider that always provider locked prices from prices module
pub struct LockedPriceProvider<T>(PhantomData<T>);
impl<T: Config> PriceProvider<CurrencyId> for LockedPriceProvider<T> {
	fn get_price(currency_id: CurrencyId) -> Option<Price> {
		Pallet::<T>::locked_price(currency_id)
	}
}

fn integer_sqrt(x: U256) -> U256 {
    let one: U256 = 1.into();
    let two: U256 = 2.into();

    let mut z: U256 = (x + U256::one()) >> one;

    let mut y = x;

    while z < y {
        y = z;
        z = (x / z + z) / two;
    }

    y
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
	integer_sqrt(
		U256::from(pool_a)
			.saturating_mul(U256::from(pool_b))
	)
	.saturating_mul(
		integer_sqrt(
			U256::from(price_a.into_inner())
				.saturating_mul(U256::from(price_b.into_inner()))
		)
	)
	.checked_div(U256::from(total_shares))
	.and_then(|n| n.checked_mul(U256::from(2)))
	.and_then(|r| TryInto::<u128>::try_into(r).ok())
	.map(Price::from_inner)
}
