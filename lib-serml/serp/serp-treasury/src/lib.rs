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

//! # SERP Treasury Module
//!
//! ## Overview
//!
//! SERP Treasury manages the Settmint, and handle excess serplus
//! and stabilize SettCurrencies standards timely in order to keep the
//! system healthy. It manages the TES (Token Elasticity of Supply).

#![cfg_attr(not(feature = "std"), no_std)]
#![allow(clippy::unused_unit)]

use frame_support::{pallet_prelude::*, transactional, PalletId};
use frame_system::pallet_prelude::*;
use orml_traits::{GetByKey, MultiCurrency, MultiCurrencyExtended};
use primitives::{Balance, BlockNumber, CurrencyId};
use sp_core::U256;
use sp_runtime::{
	traits::{AccountIdConversion, One, Zero},
	DispatchError, DispatchResult, FixedPointNumber,
};
use support::{DEXManager, Ratio, SerpTreasury};
use prices::*;
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

		/// The Currency for managing assets related to the SERP (Setheum Elastic Reserve Protocol).
		type Currency: MultiCurrencyExtended<Self::AccountId, CurrencyId = CurrencyId, Balance = Balance>;

		/// The stable currency ids
		type StableCurrencyIds: Get<Vec<CurrencyId>>;

		/// Native (DNAR) currency Stablecoin currency id
		type GetStableCurrencyMinimumSupply: GetByKey<Self::CurrencyId, Self::Balance>;

		#[pallet::constant]
		/// Native (DNAR) currency Stablecoin currency id
		type GetNativeCurrencyId: Get<CurrencyId>;

		#[pallet::constant]
		/// Setter (SETT) currency Stablecoin currency id
		type SetterCurrencyId: Get<CurrencyId>;

		#[pallet::constant]
		/// SettinDes (DRAM) dexer currency id
		type DirhamCurrencyId: Get<CurrencyId>;

		/// SERP-TES Adjustment Frequency.
		/// Schedule for when to trigger SERP-TES
		/// (Blocktime/BlockNumber - every blabla block)
		type SerpTesSchedule: Get<BlockNumber>;

		/// SerpUp ratio for Serplus Auctions / Swaps
		type SerplusSerpupRatio: Get<Rate>;

		/// SerpUp ratio for SettPay Cashdrops
		type SettPaySerpupRatio: Get<Rate>;

		/// SerpUp ratio for Setheum Treasury
		type SetheumTreasurySerpupRatio: Get<Rate>;

		/// SerpUp ratio for Setheum Foundation's Charity Fund
		type CharityFundSerpupRatio: Get<Rate>;

		#[pallet::constant]
		/// SerpUp pool/account for receiving funds SettPay Cashdrops
		/// SettPayTreasury account.
		type SettPayTreasuryAcc: Get<PalletId>;

		#[pallet::constant]
		/// SerpUp pool/account for receiving funds Setheum Treasury
		/// SetheumTreasury account.
		type SetheumTreasuryAcc: Get<PalletId>;

		/// SerpUp pool/account for receiving funds Setheum Foundation's Charity Fund
		/// CharityFund account.
		type CharityFundAcc: Get<AccountId>;

		/// Auction manager creates different types of auction to handle system serplus and standard.
		type SerpAuctionManagerHandler: SerpAuctionManager<Self::AccountId, CurrencyId = CurrencyId, Balance = Balance>;

		/// Dex manager is used to swap reserve asset (Setter) for propper (SettCurrency).
		type Dex: DEXManager<Self::AccountId, CurrencyId, Balance>;

		// TODO: Update!
		#[pallet::constant]
		/// The cap of lots when an auction is created
		type MaxAuctionsCount: Get<u32>;

		#[pallet::constant]
		/// The SERP Treasury's module id, keeps serplus and reserve asset.
		type PalletId: Get<PalletId>;

		/// Weight information for the extrinsics in this module.
		type WeightInfo: WeightInfo;
	}

	#[pallet::error]
	pub enum Error<T> {
		/// The Stablecoin Price is stable and indifferent from peg
		/// therefore cannot serp
		PriceIsStableCannotSerp,
		/// Invalid Currency Type
		InvalidCurrencyType
	}

	#[pallet::event]
	#[pallet::generate_deposit(fn deposit_event)]
	pub enum Event<T: Config> {
		/// Currency SerpUp has been delivered successfully.
		CurrencySerpUpDelivered(Balance, CurrencyId),
		/// Currency SerpUp has been completed successfully.
		CurrencySerpedUp(Balance, CurrencyId),
		/// Currency SerpDown has been triggered successfully.
		CurrencySerpDownTriggered(Balance, CurrencyId),
	}

	#[pallet::pallet]
	pub struct Pallet<T>(_);

	#[pallet::hooks]
	impl<T: Config> Hooks<T::BlockNumber> for Pallet<T> {
		///
		/// NOTE: This function is called BEFORE ANY extrinsic in a block is applied,
		/// including inherent extrinsics. Hence for instance, if you runtime includes
		/// `pallet_timestamp`, the `timestamp` is not yet up to date at this point.
		/// Handle excessive surplus or debits of system when block end
		///
		// TODO: Migrate `BlockNumber` to `Timestamp`
		/// Triggers Serping for all system stablecoins at every block.
		fn on_initialize(now: T::BlockNumber) {
			// TODO: Update for a global-adjustment-frequency to have it's own governed custom adjustment-frequency, 
			// TODO: - and call serp_tes at a timestamp e.g. every 10 minutes
			///
			/// SERP-TES Adjustment Frequency.
			/// Schedule for when to trigger SERP-TES
			/// (Blocktime/BlockNumber - every blabla block)
			if now % T::SerpTesSchedule::get() == Zero::zero() {
				// SERP TES (Token Elasticity of Supply).
				// Triggers Serping for all system stablecoins to stabilize stablecoin prices.
				Self::on_serp_tes();
			} else {
				Ok(())
			}
		}
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		#[pallet::weight(T::WeightInfo::auction_serplus())]
		#[transactional]
		pub fn auction_serplus(origin: OriginFor<T>, amount: Balance, currency_id: CurrencyId) -> DispatchResultWithPostInfo {
			T::UpdateOrigin::ensure_origin(origin)?;// ensure currency_id is accepted for serplus auction (Only SettCurrencies are accepted (SETT))
			ensure!(
				T::StableCurrencyIds::get().contains(&currency_id),
				Error::<T>::InvalidCurrencyType,
			);
			T::SerpAuctionManagerHandler::new_serplus_auction(amount, &currency_id)?;
			Ok(().into())
		}

		#[pallet::weight(T::WeightInfo::auction_diamond())]
		#[transactional]
		pub fn auction_diamond(
			origin: OriginFor<T>,
			setter_amount: Balance,
			initial_price: Balance,
		) -> DispatchResultWithPostInfo {
			T::UpdateOrigin::ensure_origin(origin)?;
			T::SerpAuctionManagerHandler::new_diamond_auction(initial_price, setter_amount)?;
			Ok(().into())
		}

		#[pallet::weight(T::WeightInfo::auction_setter())]
		#[transactional]
		pub fn auction_setter(
			origin: OriginFor<T>,
			accepted_currency: CurrencyId,
			currency_amount: Balance,
			initial_price: Balance,
		) -> DispatchResultWithPostInfo {
			T::UpdateOrigin::ensure_origin(origin)?;
			T::SerpAuctionManagerHandler::new_setter_auction(initial_price, currency_amount, accepted_currency)?;
			Ok(().into())
		}
	}
}

impl<T: Config> Pallet<T> {
	/// Get account of SERP Treasury module.
	pub fn account_id() -> T::AccountId {
		T::PalletId::get().into_account()
	}

	pub fn adjustment_frequency() -> BlockNumber {
		T::SerpTesSchedule::get()
	}
}

impl<T: Config> SerpTreasury<T::AccountId> for Pallet<T> {
	type Amount = Amount;
	type Balance = Balance;
	type CurrencyId = CurrencyId;
	type BlockNumber = BlockNumber;

	fn get_adjustment_frequency() -> Self::BlockNumber {
		Self::adjustment_frequency()
	}

	/// calculate the proportion of specific currency amount for the whole system
	fn get_propper_proportion(amount: Self::Balance, currency_id: Self::CurrencyId) -> Ratio {
		ensure!(
			T::StableCurrencyIds::get().contains(currency_id),
			Error::<T>::InvalidCyrrencyType,
		);
		let stable_total_supply = T::Currency::total_issuance(currency_id);
		Ratio::checked_from_rational(amount, stable_total_supply).unwrap_or_default()
	}

	/// SerpUp ratio for Serplus Auctions / Swaps
	fn get_serplus_serpup(amount: Balance, currency_id: Self::CurrencyId) -> DispatchResult {
		// Serplus SerpUp Pool - 10%
		let serplus_account = &Self::account_id();
		let serplus_propper = T::SerplusSerpupRatio::get() * amount;
		Self::issue_propper(currency_id, serplus_account, serplus_propper);

		Self::deposit_event(Event::CurrencySerpUpDelivered(amount, currency_id));
		Ok(())
	}

	/// SerpUp ratio for SettPay Cashdrops
	fn get_settpay_serpup(amount: Balance, currency_id: Self::CurrencyId) -> DispatchResult {
		// SettPay SerpUp Pool - 60%
		let settpay_account = T::SettPayTreasuryAcc::get().into_account();
		let settpay_propper = T::SettPaySerpupRatio::get() * amount;
		Self::issue_propper(currency_id, settpay_account, settpay_propper);

		Self::deposit_event(Event::CurrencySerpUpDelivered(amount, currency_id));
		Ok(())
	}

	/// SerpUp ratio for Setheum Treasury
	fn get_treasury_serpup(amount: Balance, currency_id: Self::CurrencyId) -> DispatchResult {
		// Setheum Treasury SerpUp Pool - 10%
		let treasury_account = T::SetheumTreasuryAcc::get().into_account();
		let treasury_propper = T::SetheumTreasurySerpupRatio::get() * amount;
		Self::issue_propper(currency_id, treasury_account, treasury_propper);

		Self::deposit_event(Event::CurrencySerpUpDelivered(amount, currency_id));
		Ok(())
	}

	/// SerpUp ratio for Setheum Foundation's Charity Fund
	fn get_charity_fund_serpup(amount: Balance, currency_id: Self::CurrencyId) -> DispatchResult {
		// TODO: update to 20%
		// Charity Fund SerpUp Pool - 20%
		let charity_fund_account = T::CharityFundAcc::get().into_account();
		let charity_fund_propper = T::CharityFundSerpupRatio::get() * amount;
		Self::issue_propper(currency_id, charity_fund_account, charity_fund_propper);

		Self::deposit_event(Event::CurrencySerpUpDelivered(amount, currency_id));
		Ok(())
	}

	/// issue serpup surplus(stable currencies) to their destinations according to the serpup_ratio.
	fn on_serpup(currency_id: CurrencyId, amount: Amount) -> DispatchResult {
		/// ensure that the currency is a SettCurrency
		ensure!(
			T::StableCurrencyIds::get().contains(&currency_id),
			Error::<T>::InvalidCyrrencyType,
		);
		/// ensure that the amount is not zero
		ensure!(
			!amount.is_zero(),
			Error::<T>::InvalidAmount,
		);
		get_serplus_serpup(amount, currency_id);
		get_settpay_serpup(amount, currency_id);
		get_treasury_serpup(amount, currency_id);
		get_charity_fund_serpup(amount, currency_id);

		Self::deposit_event(Event::CurrencySerpedUp(amount, currency_id));
		Ok(())
	}

	/// get the minimum supply of a settcurrency - by key
	fn get_minimum_supply(currency_id: CurrencyId) -> Balance {
		T::GetStableCurrencyMinimumSupply::get(currency_id)
	}
	/// buy back and burn surplus(stable currencies) with auction
	/// Create the necessary serp down parameters and starts new auction.
	fn on_serpdown(currency_id: CurrencyId, amount: Amount) -> DispatchResult {
		/// ensure that the currency is a SettCurrency
		ensure!(
			T::StableCurrencyIds::get().contains(currency_id),
			Error::<T>::InvalidCyrrencyType,
		);
		let setter_fixed_price = T::Price::get_setter_fixed_price();
		
		let total_supply = T::Currency::total_issuance(currency_id);
		let minimum_supply = Self::get_minimum_supply(currency_id);
		let removed_amount = &total_supply.saturating_sub(amount);

		if removed_amount >= &minimum_supply {
			if currency_id == T::SetterCurrencyId::get() {
				let dinar = T::GetNativeCurrencyId::get();
				let dinar_price = T::Price::get_price(&dinar);
				let relative_price = dinar_price.checked_div(&setter_fixed_price);
				/// the initial amount is the equivalent of the serpdown amount -
				/// but in the (higher) fixed price not the (lower) market price
				// TODO: Check to update-vvvvvvvvvvvvvvvvv!
				let initial_amount = amount.checked_div(&relative_price);
				/// ensure that the amounts are not zero
				ensure!(
					!initial_amount.is_zero() && !amount.is_zero(),
					Error::<T>::InvalidAmount,
				);
				/// Diamond Auction if it's to serpdown Setter.
				T::SerpAuctionManagerHandler::new_diamond_auction(&initial_amount, &amount)

			} else {
				let settcurrency_fixed_price = T::Price::get_stablecoin_fixed_price(currency_id)?;
				let relative_price = setter_fixed_price.checked_div(&settcurrency_fixed_price);
				/// the initial amount is the equivalent of the serpdown amount -
				/// but in the (higher) fixed price not the (lower) market price
				let initial_setter_amount = amount.checked_div(&relative_price);
				/// ensure that the amounts are not zero
				ensure!(
					!initial_setter_amount.is_zero() && !amount.is_zero(),
					Error::<T>::InvalidAmount,
				);
				/// Setter Auction if it's not to serpdown Setter.
				T::SerpAuctionManagerHandler::new_setter_auction(&initial_setter_amount, &amount, &currency_id)
			}
		} else {
			let balanced_amount = total_supply.saturating_sub(&minimum_supply);
			ensure!(
				total_supply.saturating_sub(&balanced_amount) >= &minimum_supply,
				Error::<T>::MinSupplyReached,
			);
			
			if currency_id == T::SetterCurrencyId::get() {
				let dinar = T::GetNativeCurrencyId::get();
				let dinar_price = T::Price::get_price(&dinar);
				let relative_price = dinar_price.checked_div(&setter_fixed_price);
				/// the initial amount is the equivalent of the serpdown amount -
				/// but in the (higher) fixed price not the (lower) market price
				// TODO: Check to update-vvvvvvvvvvvvvvvvv!
				let initial_amount = &balanced_amount.checked_div(&relative_price);
				/// ensure that the amounts are not zero
				ensure!(
					!initial_amount.is_zero() && !balanced_amount.is_zero(),
					Error::<T>::InvalidAmount,
				);
				/// Diamond Auction if it's to serpdown Setter.
				T::SerpAuctionManagerHandler::new_diamond_auction(&initial_amount, &balanced_amount)

			} else {
				let settcurrency_fixed_price = T::Price::get_stablecoin_fixed_price(currency_id)?;
				let relative_price = setter_fixed_price.checked_div(&settcurrency_fixed_price);
				/// the initial amount is the equivalent of the serpdown amount -
				/// but in the (higher) fixed price not the (lower) market price
				let initial_setter_amount = &balanced_amount.checked_div(&relative_price);
				/// ensure that the amounts are not zero
				ensure!(
					!initial_setter_amount.is_zero() && !balanced_amount.is_zero(),
					Error::<T>::InvalidAmount,
				);
				/// Setter Auction if it's not to serpdown Setter.
				T::SerpAuctionManagerHandler::new_setter_auction(&initial_setter_amount, &balanced_amount, &currency_id)
			}
		}

		Self::deposit_event(Event::CurrencySerpDownTriggered(amount, currency_id));
		Ok(())
	}

	/// Determines whether to SerpUp or SerpDown based on price swing (+/-)).
	/// positive means "Serp Up", negative means "Serp Down".
	/// Then it calls the necessary option to serp the currency supply (up/down).
	fn serp_tes(currency_id: CurrencyId) -> DispatchResult {
		ensure!(
			T::StableCurrencyIds::get().contains(&currency_id),
			Error::<T>::InvalidCurrencyType,
		);
		let market_price = T::Prices::get_stablecoin_market_price(&currency_id)?;
		let fixed_price = T::Prices::get_stablecoin_fixed_price(&currency_id)?;

		let fixed_price_amount = U256 = U256::from(&fixed_price)?;
		let market_price_amount = U256 = U256::from(&market_price)?;
		
		let market_price_percent_amount = U256 = U256::from(&market_price_amount).checked_div(U256::from(100));
		let fixed_price_percent_amount = U256 = U256::from(&fixed_price_amount).checked_div(U256::from(100));
		
		let total_supply = T::Currency::total_issuance(currency_id);

		/// if price difference is positive -> SerpUp, else if negative ->SerpDown.
		if &market_price_amount > &fixed_price_amount {
			let percentage = &market_price_amount.checked_div(&fixed_price_percent_amount);
			let differed_percentage = &percentage.checked_sub(U256::from(100));

			let inverted_serp_supply: U256 = U256::from(&total_supply)
				.saturating_mul(U256::from(100.saturating_sub(&differed_percentage)));
			let serp_supply: U256 = U256::from(&total_supply).saturating_sub(&inverted_serp_supply);
			let serp_supply_balance = &serp_supply.and_then(|n| TryInto::<Balance>::try_into(n).ok())
			.unwrap_or_else(Zero::zero);

			/// ensure that the differed amount is not zero
			ensure!(
				!serp_supply_balance.is_zero(),
				Error::<T>::PriceIsStableCannotSerp,
			);
			T::SerpTreasury::on_serpup(&currency_id, &serp_supply_balance)?;

		} else if &fixed_price_amount > &market_price_amount {
			let percentage = &fixed_price_amount.checked_div(&market_price_percent_amount);
			let differed_percentage = &percentage.checked_sub(U256::from(100));

			let inverted_serp_supply: U256 = U256::from(&total_supply)
				.saturating_mul(U256::from(100.saturating_sub(&differed_percentage)));
			let serp_supply: U256 = U256::from(&total_supply).saturating_sub(&inverted_serp_supply);
			let serp_supply_balance = &serp_supply.and_then(|n| TryInto::<Balance>::try_into(n).ok())
			.unwrap_or_else(Zero::zero);

			/// ensure that the differed amount is not zero
			ensure!(
				!serp_supply_balance.is_zero(),
				Error::<T>::PriceIsStableCannotSerp,
			);
			T::SerpTreasury::on_serpdown(&currency_id, &serp_supply_balance)?;
			
		}
		Ok(())
	}

	// TODO: Update for a global-adjustment-frequency to have it's own governed custom adjustment-frequency, 
	// TODO: - and call serp_tes at a timestamp e.g. every 10 minutes
	///
	/// Trigger SERP-TES for all stablecoins
	/// Check all stablecoins stability and elasticity
	/// and calls the serp to stabilise the unstable one(s)
	/// on SERP-TES.
	fn on_serp_tes() -> DispatchResult {
		// iterator to SERP-TES every system stablecurrency based on it's custom adjustment frequency
		for currency_id in T::StableCurrencyIds::get() {
			Self::serp_tes(currency_id)
		}
		Ok(())
	}
	
	fn issue_standard(currency_id: CurrencyId, who: &T::AccountId, standard: Self::Balance) -> DispatchResult {
		T::Currency::deposit(currency_id, who, standard)?;
		Ok(())
	}

	fn burn_standard(currency_id: CurrencyId, who: &T::AccountId, standard: Self::Balance) -> DispatchResult {
		T::Currency::withdraw(currency_id, who, standard)
	}

	fn issue_propper(currency_id: CurrencyId, who: &T::AccountId, propper: Self::Balance) -> DispatchResult {
		T::Currency::deposit(currency_id, who, propper)?;
		Ok(())
	}

	fn burn_propper(currency_id: CurrencyId, who: &T::AccountId, propper: Self::Balance) -> DispatchResult {
		T::Currency::withdraw(currency_id, who, propper)
	}

	fn issue_setter(who: &T::AccountId, setter: Self::Balance) -> DispatchResult {
		T::Currency::deposit(T::SetterCurrencyId::get(), who, setter)?;
		Ok(())
	}

	/// Burn Reserve asset (Setter (SETT))
	fn burn_setter(who: &T::AccountId, setter: Self::Balance) -> DispatchResult {
		T::Currency::withdraw(T::SetterCurrencyId::get(), who, setter)
	}

	/// Issue Dexer (`DRAM` in Setheum). `dexer` here just referring to the Dex token balance.
	fn issue_dexer(who: &T::AccountId, dexer: Self::Balance) -> DispatchResult {
		T::Currency::deposit(T::DirhamCurrencyId::get(), who, dexer)?;
		Ok(())
	}

	/// Burn Dexer (`DRAM` in Setheum). `dexer` here just referring to the Dex token balance.
	fn burn_dexer(who: &T::AccountId, dexer: Self::Balance) -> DispatchResult {
		T::Currency::withdraw(T::DirhamCurrencyId::get(), who, dexer)
	}

	/// deposit surplus(propper stable currency) to serp treasury by `from`
	fn deposit_serplus(currency_id: CurrencyId, from: &T::AccountId, serplus: Self::Balance) -> DispatchResult {
		T::Currency::transfer(currency_id, from, &Self::account_id(), serplus)
	}

	/// deposit reserve asset (Setter (SETT)) to serp treasury by `who`
	fn deposit_setter(from: &T::AccountId, amount: Self::Balance) -> DispatchResult {
		T::Currency::transfer(T::SetterCurrencyId::get(), from, &Self::account_id(), amount)
	}
}
