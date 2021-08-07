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

use fixed::{types::extra::U128, FixedU128};
use frame_support::{pallet_prelude::*, transactional, PalletId};
use frame_system::pallet_prelude::*;
use orml_traits::{GetByKey, MultiCurrency, MultiCurrencyExtended};
use primitives::{Balance, CurrencyId};
// use sp_core::U256;
use sp_runtime::{
	traits::{
		AccountIdConversion, One,
		Saturating, Zero,
	},
	DispatchError, DispatchResult,
};
use sp_std::{
	convert::{TryFrom, TryInto},
	prelude::*, result::Result, vec
};
use support::{
	DEXManager, PriceProvider, Ratio, SerpAuctionManager, SerpTreasury, SerpTreasuryExtended
};

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

		/// The minimum total supply/issuance required to keep a currency live on SERP.
		type GetStableCurrencyMinimumSupply: GetByKey<CurrencyId, Balance>;

		#[pallet::constant]
		/// Native (DNAR) currency Stablecoin currency id
		type GetNativeCurrencyId: Get<CurrencyId>;

		#[pallet::constant]
		/// Setter (SETT) currency Stablecoin currency id
		type SetterCurrencyId: Get<CurrencyId>;

		#[pallet::constant]
		/// The SettUSD currency id, it should be USDJ in Setheum.
		type GetSettUSDCurrencyId: Get<CurrencyId>;

		#[pallet::constant]
		/// SettinDes (DRAM) dexer currency id
		type DirhamCurrencyId: Get<CurrencyId>;

		/// SERP-TES Adjustment Frequency.
		/// Schedule for when to trigger SERP-TES
		/// (Blocktime/BlockNumber - every blabla block)
		type SerpTesSchedule: Get<Self::BlockNumber>;

		/// SerpUp ratio for Serplus Auctions / Swaps
		/// The first item of the tuple is the numerator of the serpup rate, second
		/// item is the denominator, serpup_rate = numerator / denominator,
		/// use (u32, u32) over `Rate` type to minimize internal division
		/// operation.
		type BuybackSerpupRatio: Get<(u32, u32)>;

		/// SerpUp ratio for SettPay Cashdrops
		/// The first item of the tuple is the numerator of the serpup rate, second
		/// item is the denominator, serpup_rate = numerator / denominator,
		/// use (u32, u32) over `Rate` type to minimize internal division
		/// operation.
		type SettPaySerpupRatio: Get<(u32, u32)>;

		/// SerpUp ratio for Setheum Treasury
		/// The first item of the tuple is the numerator of the serpup rate, second
		/// item is the denominator, serpup_rate = numerator / denominator,
		/// use (u32, u32) over `Rate` type to minimize internal division
		/// operation.
		type SetheumTreasurySerpupRatio: Get<(u32, u32)>;

		/// SerpUp ratio for Setheum Foundation's Charity Fund
		/// The first item of the tuple is the numerator of the serpup rate, second
		/// item is the denominator, serpup_rate = numerator / denominator,
		/// use (u32, u32) over `Rate` type to minimize internal division
		/// operation.
		type CharityFundSerpupRatio: Get<(u32, u32)>;

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
		type CharityFundAcc: Get<Self::AccountId>;

		/// Auction manager creates different types of auction to handle system serplus and standard.
		type SerpAuctionManagerHandler: SerpAuctionManager<Self::AccountId, CurrencyId = CurrencyId, Balance = Balance>;

		/// Dex manager is used to swap reserve asset (Setter) for propper (SettCurrency).
		type Dex: DEXManager<Self::AccountId, CurrencyId, Balance>;

		/// The max slippage allowed when swap fee with DEX
		#[pallet::constant]
		type MaxSlippageSwapWithDEX: Get<Ratio>;

		/// The price source of currencies
		type PriceSource: PriceProvider<CurrencyId>;

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
		InvalidCurrencyType,
		/// Feed price is invalid
		InvalidFeedPrice,
		/// Amount is invalid
		InvalidAmount,
		/// Minimum Supply is reached
		MinSupplyReached,
		/// Dinar is not enough
		DinarNotEnough,
		/// Swap Path is invalid
		InvalidSwapPath,
	}

	#[pallet::event]
	#[pallet::generate_deposit(pub(crate) fn deposit_event)]
	pub enum Event<T: Config> {
		/// Currency SerpTes has been triggered.
		SerpTes(CurrencyId),
		/// Currency SerpUp has been delivered successfully.
		SerpUpDelivery(Balance, CurrencyId),
		/// Currency SerpUp has been completed successfully.
		SerpUp(Balance, CurrencyId),
		/// Currency SerpDown has been triggered successfully.
		SerpDown(Balance, CurrencyId),
		/// The expected amount size for per lot Dinar auction of specific
		/// Dinar type updated. \[Dinar_type, new_size\]
		ExpectedDinarAuctionSizeUpdated(Balance),
	}

	/// The expected amount size for per lot Dinar auction of specific
	/// Dinar type.
	///
	/// ExpectedDinarAuctionSize: Balance
	#[pallet::storage]
	#[pallet::getter(fn expected_dinar_auction_size)]
	pub type ExpectedDinarAuctionSize<T: Config> = StorageValue<_, Balance, ValueQuery>;

	#[pallet::genesis_config]
	pub struct GenesisConfig {
		pub expected_dinar_auction_size: Balance,
	}

	#[cfg(feature = "std")]
	impl Default for GenesisConfig {
		fn default() -> Self {
			GenesisConfig {
				expected_dinar_auction_size: Default::default(),
			}
		}
	}

	#[pallet::genesis_build]
	impl<T: Config> GenesisBuild<T> for GenesisConfig {
		fn build(&self) {
			ExpectedDinarAuctionSize::<T>::put(&self.expected_dinar_auction_size);
		}
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
		fn on_initialize(now: T::BlockNumber) -> Weight {
			// TODO: Update for a global-adjustment-frequency to have it's own governed custom adjustment-frequency, 
			// TODO: - and call serp_tes at a timestamp e.g. every 10 minutes
			//
			// SERP-TES Adjustment Frequency.
			// Schedule for when to trigger SERP-TES
			// (Blocktime/BlockNumber - every blabla block)
			if now % T::SerpTesSchedule::get() == Zero::zero() {
				// SERP TES (Token Elasticity of Supply).
				// Triggers Serping for all system stablecoins to stabilize stablecoin prices.
				let mut count: u32 = 0;
				let native_currency_id = T::GetNativeCurrencyId::get();
				let setter_currency_id = T::SetterCurrencyId::get();
				Self::setter_on_tes();
				count += 1;
				Self::usdj_on_tes();
				count += 1;

				T::WeightInfo::on_initialize(count)
			} else {
				0
			}
		}
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		#[pallet::weight(T::WeightInfo::auction_dinar())]
		#[transactional]
		pub fn auction_dinar(
			origin: OriginFor<T>,
			amount: Balance,
			target: Balance,
			splited: bool,
		) -> DispatchResultWithPostInfo {
			T::UpdateOrigin::ensure_origin(origin)?;
			<Self as SerpTreasuryExtended<T::AccountId>>::create_dinar_auction(
				amount,
				target,
				Self::account_id(),
				splited,
			)?;
			Ok(().into())
		}
	}
}

impl<T: Config> Pallet<T> {
	/// Get account of SERP Treasury module.
	pub fn account_id() -> T::AccountId {
		T::PalletId::get().into_account()
	}
}

impl<T: Config> SerpTreasury<T::AccountId> for Pallet<T> {
	type Balance = Balance;
	type CurrencyId = CurrencyId;

	/// SerpUp ratio for Serplus Auctions / Swaps
	fn get_buyback_serpup(amount: Balance, currency_id: Self::CurrencyId) -> DispatchResult {
		// Setheum Treasury SerpUp Pool - 10%
		let serping_amount: Balance = amount / 10;
		
		// try to use stable currency to swap reserve asset by exchange with DEX - to burn the Dinar (DNAR).
		let dinar_currency_id = T::GetNativeCurrencyId::get();
		let relative_price = T::PriceSource::get_relative_price(currency_id, dinar_currency_id)
			.ok_or(Error::<T>::InvalidFeedPrice);
		let price_impact_limit = Some(T::MaxSlippageSwapWithDEX::get());
		
		<Self as SerpTreasuryExtended<T::AccountId>>::swap_exact_stablecurrency_to_dinar(
			currency_id,
			serping_amount,
			None,
		);

		// Burn Native Reserve asset (Dinar (DNAR))
		Self::burn_dinar(&Self::account_id(), serping_amount);

		<Pallet<T>>::deposit_event(Event::SerpUpDelivery(amount, currency_id));
		Ok(())
	}

	/// SerpUp ratio for SettPay Cashdrops
	fn get_settpay_serpup(amount: Balance, currency_id: Self::CurrencyId) -> DispatchResult {
		let settpay_account = T::SettPayTreasuryAcc::get().into_account();

		// SettPay SerpUp Pool - 70%
		let seven: Balance = 7;
		let serping_amount: Balance = seven.saturating_mul(amount / 10);
		// Issue the SerpUp propper to the SettPayVault
		Self::issue_propper(currency_id, &settpay_account, serping_amount);

		<Pallet<T>>::deposit_event(Event::SerpUpDelivery(amount, currency_id));
		Ok(())
	}

	/// SerpUp ratio for Setheum Foundation's Charity Fund
	fn get_charity_fund_serpup(amount: Balance, currency_id: Self::CurrencyId) -> DispatchResult {
		let charity_fund_account = T::CharityFundAcc::get();
		// Charity Fund SerpUp Pool - 10%
		let serping_amount: Balance = amount / 10;
		// Issue the SerpUp propper to the SettPayVault
		Self::issue_propper(currency_id, &charity_fund_account, serping_amount);

		<Pallet<T>>::deposit_event(Event::SerpUpDelivery(amount, currency_id));
		Ok(())
	}

	/// SerpUp ratio for Setheum Foundation's Charity Fund
	fn get_setheum_treasury_serpup(amount: Balance, currency_id: Self::CurrencyId) -> DispatchResult {
		let charity_fund_account = T::SetheumTreasuryAcc::get().into_account();
		// Charity Fund SerpUp Pool - 10%
		let serping_amount: Balance = amount / 10;
		// Issue the SerpUp propper to the SettPayVault
		Self::issue_propper(currency_id, &charity_fund_account, serping_amount);

		<Pallet<T>>::deposit_event(Event::SerpUpDelivery(amount, currency_id));
		Ok(())
	}

	fn setter_on_tes() -> DispatchResult {
		type Fix = FixedU128<U128>;
		let currency_id = T::SetterCurrencyId::get();
		let one: u128 = 1;
		let Some(market_price) = T::PriceSource::get_market_price(currency_id);
		let market_to_num = Fix::from_num(market_price).to_num::<u128>();
			.ok_or(Error::<T>::InvalidFeedPrice);
		let peg_price: U256 = U256::from(T::PriceSource::get_peg_price(currency_id))
			.ok_or(Error::<T>::InvalidFeedPrice);
		let total_supply = T::Currency::total_issuance(currency_id);

		match market_price {
			market_price if market_price > peg_price => {
	
				// safe from underflow because `peg_price` is checked to be less than `market_price`
				let expand_by = get_supply_change(market_price, peg_price, total_supply);
				Self::on_serpup(currency_id, expand_by)?;
			}
			market_price if market_price < peg_price => {
				// safe from underflow because `peg_price` is checked to be greater than `market_price`
				let contract_by = get_supply_change(peg_price, market_price, total_supply);
				Self::on_serpdown(currency_id, contract_by)?;
			}
			_ => {}
		}
		<Pallet<T>>::deposit_event(Event::SerpTes(currency_id));
		Ok(())
	}

	fn usdj_on_tes() -> DispatchResult {
		let currency_id = T::GetSettUSDCurrencyId::get();
		let market_price: U256 = U256::from(T::PriceSource::get_market_price(currency_id))
			.ok_or(Error::<T>::InvalidFeedPrice);
		let peg_price: U256 = U256::from(T::PriceSource::get_peg_price(currency_id))
			.ok_or(Error::<T>::InvalidFeedPrice);
		let total_supply = T::Currency::total_issuance(currency_id);

		match market_price {
			market_price if market_price > peg_price => {
	
				// safe from underflow because `peg_price` is checked to be less than `market_price`
				let expand_by = get_supply_change(market_price, peg_price, total_supply);
				Self::on_serpup(currency_id, expand_by)?;
			}
			market_price if market_price < peg_price => {
				// safe from underflow because `peg_price` is checked to be greater than `market_price`
				let contract_by = get_supply_change(peg_price, market_price, total_supply);
				Self::on_serpdown(currency_id, contract_by)?;
			}
			_ => {}
		}
		<Pallet<T>>::deposit_event(Event::SerpTes(currency_id));
		Ok(())
	}

	/// issue serpup surplus(stable currencies) to their destinations according to the serpup_ratio.
	fn on_serpup(currency_id: CurrencyId, amount: Balance) -> DispatchResult {
		// ensure that the currency is a SettCurrency
		ensure!(
			T::StableCurrencyIds::get().contains(&currency_id),
			Error::<T>:: InvalidCurrencyType,
		);
		// ensure that the amount is not zero
		ensure!(
			!amount.is_zero(),
			Error::<T>::InvalidAmount,
		);
		Self::get_buyback_serpup(amount, currency_id);
		Self::get_settpay_serpup(amount, currency_id);
		Self::get_setheum_treasury_serpup(amount, currency_id);
		Self::get_charity_fund_serpup(amount, currency_id);

		<Pallet<T>>::deposit_event(Event::SerpUp(amount, currency_id));
		Ok(())
	}

	// get the minimum supply of a settcurrency - by key
	fn get_minimum_supply(currency_id: CurrencyId) -> Balance {
		T::GetStableCurrencyMinimumSupply::get(&currency_id)
	}
	// buy back and burn surplus(stable currencies) with auction
	// Create the necessary serp down parameters and starts new auction.
	fn on_serpdown(currency_id: CurrencyId, amount: Balance) -> DispatchResult {
		// ensure that the currency is a SettCurrency
		ensure!(
			T::StableCurrencyIds::get().contains(&currency_id),
			Error::<T>:: InvalidCurrencyType,
		);
		let setter_fixed_price = T::PriceSource::get_setter_price()
		.ok_or(Error::<T>::InvalidFeedPrice);
		let total_supply = T::Currency::total_issuance(currency_id);
		let minimum_supply = Self::get_minimum_supply(currency_id);

		ensure!(
			total_supply.saturating_sub(amount) >= minimum_supply,
			Error::<T>::MinSupplyReached,
		);

		if currency_id == T::SetterCurrencyId::get() {
			<Self as SerpTreasuryExtended<T::AccountId>>::swap_dinar_to_exact_setter(
				amount,
				None,
			);
		} else {
			<Self as SerpTreasuryExtended<T::AccountId>>::swap_setter_to_exact_settcurrency(
				currency_id,
				amount,
				None,
			);
		} 

		<Pallet<T>>::deposit_event(Event::SerpDown(amount, currency_id));
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

	fn issue_dinar(who: &T::AccountId, dinar: Self::Balance) -> DispatchResult {
		T::Currency::deposit(T::GetNativeCurrencyId::get(), who, dinar)?;
		Ok(())
	}

	/// Burn Native Reserve asset (Dinar (DNAR))
	fn burn_dinar(who: &T::AccountId, dinar: Self::Balance) -> DispatchResult {
		T::Currency::withdraw(T::GetNativeCurrencyId::get(), who, dinar)
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

	// refund remain Dinar to refund recipient from SERP
	fn withdraw_dinar(to: &T::AccountId, amount: Self::Balance) -> DispatchResult {
		T::Currency::transfer(T::GetNativeCurrencyId::get(), &Self::account_id(), to, amount)
	}
}

impl<T: Config> SerpTreasuryExtended<T::AccountId> for Pallet<T> {
	/// Swap exact amount of Dinar to Setter,
	/// return actual target Setter amount
	fn swap_exact_dinar_to_setter(
		supply_amount: Balance,
		maybe_path: Option<&[CurrencyId]>,
	) -> sp_std::result::Result<Balance, DispatchError> {
		let dinar_currency_id = T::GetNativeCurrencyId::get();
		T::Currency::deposit(T::GetNativeCurrencyId::get(), &Self::account_id(), supply_amount)?;

		let setter_currency_id = T::SetterCurrencyId::get();
		let default_swap_path = &[dinar_currency_id, setter_currency_id];
		let swap_path = match maybe_path {
			None => default_swap_path,
			Some(path) => {
				let path_length = path.len();
				ensure!(
					path_length >= 2 && path[0] == dinar_currency_id && path[path_length - 1] == setter_currency_id,
					Error::<T>::InvalidSwapPath
				);
				path
			}
		};
		let price_impact_limit = Some(T::MaxSlippageSwapWithDEX::get());
		let Some(min_target_amount) = T::Dex::get_swap_target_amount(&swap_path, supply_amount, price_impact_limit);

		T::Dex::swap_with_exact_supply(
			&Self::account_id(),
			swap_path,
			supply_amount,
			min_target_amount,
			price_impact_limit,
		)
	}

	/// swap Dinar which not in auction to get exact Setter,
	/// return actual supply Dinar amount
	fn swap_dinar_to_exact_setter(
		target_amount: Balance,
		maybe_path: Option<&[CurrencyId]>,
	) -> sp_std::result::Result<Balance, DispatchError> {
		let dinar_currency_id = T::GetNativeCurrencyId::get();

		let setter_currency_id = T::SetterCurrencyId::get();
		let default_swap_path = &[dinar_currency_id, setter_currency_id];
		let swap_path = match maybe_path {
			None => default_swap_path,
			Some(path) => {
				let path_length = path.len();
				ensure!(
					path_length >= 2 && path[0] == dinar_currency_id && path[path_length - 1] == setter_currency_id,
					Error::<T>::InvalidSwapPath
				);
				path
			}
		};
		let price_impact_limit = Some(T::MaxSlippageSwapWithDEX::get());
		let Some(max_supply_amount) = T::Dex::get_swap_supply_amount(&swap_path, target_amount, price_impact_limit);
				
		T::Currency::deposit(dinar_currency_id, &Self::account_id(), max_supply_amount)?;
		T::Dex::swap_with_exact_target(
			&Self::account_id(),
			swap_path,
			target_amount,
			max_supply_amount,
			price_impact_limit,
		)
	}

	fn create_dinar_auction(
		amount: Balance,
		target: Balance,
		refund_receiver: T::AccountId,
		splited: bool,
	) -> DispatchResult {

		T::Currency::deposit(T::GetNativeCurrencyId::get(), &Self::account_id(), amount)?;

		let mut unhandled_dinar_amount = amount;
		let mut unhandled_target = target;
		let expected_dinar_auction_size = Self::expected_dinar_auction_size();
		let max_auctions_count: Balance = T::MaxAuctionsCount::get().into();
		let lots_count = if !splited
			|| max_auctions_count.is_zero()
			|| expected_dinar_auction_size.is_zero()
			|| amount <= expected_dinar_auction_size
		{
			One::one()
		} else {
			let mut count = amount
				.checked_div(expected_dinar_auction_size)
				.expect("dinar auction maximum size is not zero; qed");

			let remainder = amount
				.checked_rem(expected_dinar_auction_size)
				.expect("dinar auction maximum size is not zero; qed");
			if !remainder.is_zero() {
				count = count.saturating_add(One::one());
			}
			sp_std::cmp::min(count, max_auctions_count)
		};
		let average_amount_per_lot = amount.checked_div(lots_count).expect("lots count is at least 1; qed");
		let average_target_per_lot = target.checked_div(lots_count).expect("lots count is at least 1; qed");
		let mut created_lots: Balance = Zero::zero();

		while !unhandled_dinar_amount.is_zero() {
			created_lots = created_lots.saturating_add(One::one());
			let (lot_dinar_amount, lot_target) = if created_lots == lots_count {
				// the last lot may be have some remnant than average
				(unhandled_dinar_amount, unhandled_target)
			} else {
				(average_amount_per_lot, average_target_per_lot)
			};

			T::SerpAuctionManagerHandler::new_dinar_auction(
				&refund_receiver,
				lot_dinar_amount,
				lot_target,
			)?;

			unhandled_dinar_amount = unhandled_dinar_amount.saturating_sub(lot_dinar_amount);
			unhandled_target = unhandled_target.saturating_sub(lot_target);
		}
		Ok(())
	}

	/// Swap exact amount of Setter to SettCurrency,
	/// return actual target SettCurrency amount
	///
	/// 
	/// When SettCurrency needs SerpDown
	fn swap_setter_to_exact_settcurrency(
		currency_id: CurrencyId,
		target_amount: Balance,
		maybe_path: Option<&[CurrencyId]>,
	) -> sp_std::result::Result<Balance, DispatchError> {
		let setter_currency_id = T::SetterCurrencyId::get();

		let default_swap_path = &[setter_currency_id, currency_id];
		let swap_path = match maybe_path {
			None => default_swap_path,
			Some(path) => {
				let path_length = path.len();
				ensure!(
					path_length >= 2 && path[0] == setter_currency_id && path[path_length - 1] == currency_id,
					Error::<T>::InvalidSwapPath
				);
				path
			}
		};
		let price_impact_limit = Some(T::MaxSlippageSwapWithDEX::get());
		let Some(max_supply_amount) = T::Dex::get_swap_supply_amount(&swap_path, target_amount, price_impact_limit);
				
		T::Currency::deposit(setter_currency_id, &Self::account_id(), max_supply_amount)?;
		T::Dex::swap_with_exact_target(
			&Self::account_id(),
			swap_path,
			target_amount,
			max_supply_amount,
			price_impact_limit,
		)
	}

	/// Swap exact amount of StableCurrency to Dinar,
	/// return actual supply StableCurrency amount
	///
	/// 
	/// When StableCurrency gets SerpUp
	fn swap_exact_stablecurrency_to_dinar(
		currency_id: CurrencyId,
		supply_amount: Balance,
		maybe_path: Option<&[CurrencyId]>,
	) -> sp_std::result::Result<Balance, DispatchError> {
		let dinar_currency_id = T::GetNativeCurrencyId::get();
		T::Currency::deposit(currency_id, &Self::account_id(), supply_amount)?;

		let default_swap_path = &[currency_id, dinar_currency_id];
		let swap_path = match maybe_path {
			None => default_swap_path,
			Some(path) => {
				let path_length = path.len();
				ensure!(
					path_length >= 2 && path[0] == currency_id && path[path_length - 1] == dinar_currency_id,
					Error::<T>::InvalidSwapPath
				);
				path
			}
		};
		let price_impact_limit = Some(T::MaxSlippageSwapWithDEX::get());
		let Some(min_target_amount) = T::Dex::get_swap_target_amount(&swap_path, supply_amount, price_impact_limit);

		T::Dex::swap_with_exact_supply(
			&Self::account_id(),
			swap_path,
			supply_amount,
			min_target_amount,
			price_impact_limit,
		)
	}
}

#[cfg(feature = "std")]
impl GenesisConfig {
	/// Direct implementation of `GenesisBuild::build_storage`.
	///
	/// Kept in order not to break dependency.
	pub fn build_storage<T: Config>(&self) -> Result<sp_runtime::Storage, String> {
		<Self as GenesisBuild<T>>::build_storage(self)
	}

	/// Direct implementation of `GenesisBuild::assimilate_storage`.
	///
	/// Kept in order not to break dependency.
	pub fn assimilate_storage<T: Config>(&self, storage: &mut sp_runtime::Storage) -> Result<(), String> {
		<Self as GenesisBuild<T>>::assimilate_storage(self, storage)
	}
}

/// Calculate the amount of supply change from a fraction given as `nume_fraction`, `deno_fraction` and  `supply`.
fn get_supply_change(nume_fraction: U256, deno_fraction: U256, supply: Balance) -> Balance {
	type Fix = FixedU128<U128>;
	let fraction = Fix::from_num(nume_fraction) / Fix::from_num(deno_fraction) - Fix::from_num(1);
	fraction.saturating_mul_int(supply as u128).to_num::<u128>()
}
