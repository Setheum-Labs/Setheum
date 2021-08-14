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

// use fixed::{types::extra::U128, FixedU128};
use frame_support::{pallet_prelude::*, transactional, PalletId};
use frame_system::pallet_prelude::*;
use orml_traits::{GetByKey, MultiCurrency, MultiCurrencyExtended};
use primitives::{Balance, CurrencyId};
use sp_core::U256;
use sp_runtime::{
	traits::{AccountIdConversion, Zero},
	DispatchError, DispatchResult
};
use sp_std::prelude::*;
use support::{
	DEXManager, PriceProvider, Ratio, SerpTreasury, SerpTreasuryExtended
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

		#[pallet::constant]
		/// SerpUp pool/account for receiving funds SettPay Cashdrops
		/// SettPayTreasury account.
		type SettPayTreasuryAcc: Get<PalletId>;

		/// SerpUp pool/account for receiving funds Setheum Foundation's Charity Fund
		/// CharityFund account.
		type CharityFundAcc: Get<Self::AccountId>;

		/// Dex manager is used to swap reserve asset (Setter) for propper (SettCurrency).
		type Dex: DEXManager<Self::AccountId, CurrencyId, Balance>;

		/// The max slippage allowed when swap fee with DEX
		type MaxSlippageSwapWithDEX: Get<Ratio>;

		/// The price source of currencies
		type PriceSource: PriceProvider<CurrencyId>;

		/// The cashdrop currency ids that can be rewarded with CashDrop.
		type RewardableCurrencyIds: Get<Vec<CurrencyId>>;

		/// The cashdrop currency ids that receive Setter.
		type NonStableDropCurrencyIds: Get<Vec<CurrencyId>>;

		/// The cashdrop currency ids that receive SettCurrencies.
		type SetCurrencyDropCurrencyIds: Get<Vec<CurrencyId>>;

		/// The default cashdrop rate to be issued to claims.
		/// 
		/// The first item of the tuple is the numerator of the cashdrop rate, second
		/// item is the denominator, cashdrop_rate = numerator / denominator,
		/// use (u32, u32) over `Rate` type to minimize internal division
		/// operation.
		type GetCashDropRate: Get<(u32, u32)>;

		/// The minimum transfer amounts by currency_id,  to secure cashdrop from dusty claims.
		type MinimumClaimableTransferAmounts: GetByKey<CurrencyId, Balance>;

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
		/// CashDrop is not available.
		CashdropNotAvailable,
		/// Transfer is too low for CashDrop.
		TransferTooLowForCashDrop
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
		/// CashDrop has been completed successfully.
		CashDrops(CurrencyId, T::AccountId, Balance),
	}

	/// Mapping to Minimum Claimable Transfer.
	#[pallet::storage]
	#[pallet::getter(fn minimum_claimable_transfer)]
	pub type MinimumClaimableTransfer<T: Config> = StorageMap<_, Twox64Concat, CurrencyId, Balance, OptionQuery>;

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
				if Self::setter_on_tes().is_ok() {
					count += 1;
				};
				if Self::usdj_on_tes().is_ok() {
					count += 1;
				}

				T::WeightInfo::on_initialize(count)
			} else {
				0
			}
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

	/// SerpUp ratio for BuyBack Swaps to burn Dinar
	fn get_buyback_serpup(amount: Balance, currency_id: Self::CurrencyId) -> DispatchResult {
		// Setheum Treasury SerpUp Pool - 30%
		let three: Balance = 3;
		let serping_amount: Balance = three.saturating_mul(amount / 10);
		
		// try to use stable currency to swap reserve asset by exchange with DEX - to burn the Dinar (DNAR).
		<Self as SerpTreasuryExtended<T::AccountId>>::swap_exact_stablecurrency_to_dinar(
			currency_id,
			serping_amount,
			None,
		)?;

		// Burn Native Reserve asset (Dinar (DNAR))
		T::Currency::withdraw( T::GetNativeCurrencyId::get(), &Self::account_id(), serping_amount)?;

		<Pallet<T>>::deposit_event(Event::SerpUpDelivery(amount, currency_id));
		Ok(())
	}

	/// SerpUp ratio for SettPay Cashdrops
	fn get_settpay_serpup(amount: Balance, currency_id: Self::CurrencyId) -> DispatchResult {
		let settpay_account = T::SettPayTreasuryAcc::get().into_account();

		// SettPay SerpUp Pool - 60%
		let six: Balance = 6;
		let serping_amount: Balance = six.saturating_mul(amount / 10);
		// Issue the SerpUp propper to the SettPayVault
		Self::issue_standard(currency_id, &settpay_account, serping_amount)?;

		<Pallet<T>>::deposit_event(Event::SerpUpDelivery(amount, currency_id));
		Ok(())
	}

	/// SerpUp ratio for Setheum Foundation's Charity Fund
	fn get_charity_fund_serpup(amount: Balance, currency_id: Self::CurrencyId) -> DispatchResult {
		let charity_fund_account = T::CharityFundAcc::get();
		// Charity Fund SerpUp Pool - 10%
		let serping_amount: Balance = amount / 10;
		// Issue the SerpUp propper to the SettPayVault
		Self::issue_standard(currency_id, &charity_fund_account, serping_amount)?;

		<Pallet<T>>::deposit_event(Event::SerpUpDelivery(amount, currency_id));
		Ok(())
	}

	fn setter_on_tes() -> DispatchResult {
		let currency_id = T::SetterCurrencyId::get();
		let market_price = <T as Config>::PriceSource::get_market_price(currency_id);
		let peg_price = <T as Config>::PriceSource::get_peg_price(currency_id);
		let total_supply = T::Currency::total_issuance(currency_id);
		
		match market_price {
			market_price if market_price > peg_price => {
	
				// safe from underflow because `peg_price` is checked to be less than `market_price`
				// expand_by = 0.2% of total_supply;
				let expand_by = total_supply / 500;
				Self::on_serpup(currency_id, expand_by)?;
			}
			market_price if market_price < peg_price => {
				// safe from underflow because `peg_price` is checked to be greater than `market_price`
				// expand_by = 0.2% of total_supply;
				let contract_by = total_supply / 500;
				Self::on_serpdown(currency_id, contract_by)?;
			}
			_ => {}
		}
		<Pallet<T>>::deposit_event(Event::SerpTes(currency_id));
		Ok(())
	}

	fn usdj_on_tes() -> DispatchResult {
		let currency_id = T::GetSettUSDCurrencyId::get();
		let market_price = <T as Config>::PriceSource::get_market_price(currency_id);
		let peg_price = <T as Config>::PriceSource::get_peg_price(currency_id);
		let total_supply = T::Currency::total_issuance(currency_id);

		match market_price {
			market_price if market_price > peg_price => {
	
				// safe from underflow because `peg_price` is checked to be less than `market_price`
				// expand_by = 0.2% of total_supply;
				let expand_by = total_supply / 500;
				Self::on_serpup(currency_id, expand_by)?;
			}
			market_price if market_price < peg_price => {
				// safe from underflow because `peg_price` is checked to be greater than `market_price`
				// expand_by = 0.2% of total_supply;
				let contract_by = total_supply / 500;
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
		Self::get_buyback_serpup(amount, currency_id)?;
		Self::get_settpay_serpup(amount, currency_id)?;
		Self::get_charity_fund_serpup(amount, currency_id)?;

		<Pallet<T>>::deposit_event(Event::SerpUp(amount, currency_id));
		Ok(())
	}

	// get the minimum supply of a settcurrency - by key
	fn get_minimum_supply(currency_id: CurrencyId) -> Balance {
		T::GetStableCurrencyMinimumSupply::get(&currency_id)
	}
	// buy back and burn surplus(stable currencies) with swap on DEX
	// Create the necessary serp down parameters and swap currencies then burn swapped currencies.
	//
	// TODO: Update to add the burning of the stablecoins!
	//
	fn on_serpdown(currency_id: CurrencyId, amount: Balance) -> DispatchResult {
		// ensure that the currency is a SettCurrency
		ensure!(
			T::StableCurrencyIds::get().contains(&currency_id),
			Error::<T>:: InvalidCurrencyType,
		);
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
			)?;
		} else {
			<Self as SerpTreasuryExtended<T::AccountId>>::swap_setter_to_exact_settcurrency(
				currency_id,
				amount,
				None,
			)?;
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

	fn issue_setter(who: &T::AccountId, setter: Self::Balance) -> DispatchResult {
		T::Currency::deposit(T::SetterCurrencyId::get(), who, setter)?;
		Ok(())
	}

	/// Burn Reserve asset (Setter (SETT))
	fn burn_setter(who: &T::AccountId, setter: Self::Balance) -> DispatchResult {
		T::Currency::withdraw(T::SetterCurrencyId::get(), who, setter)
	}

	/// deposit reserve asset (Setter (SETT)) to serp treasury by `who`
	fn deposit_setter(from: &T::AccountId, amount: Self::Balance) -> DispatchResult {
		T::Currency::transfer(T::SetterCurrencyId::get(), from, &Self::account_id(), amount)
	}

	/// claim cashdrop of `currency_id` relative to `transfer_amount` for `who`
	fn claim_cashdrop(currency_id: CurrencyId, who: &T::AccountId, transfer_amount: Balance) -> DispatchResult {
		ensure!(
			T::RewardableCurrencyIds::get().contains(&currency_id),
			Error::<T>::InvalidCurrencyType,
		);
		let minimum_claimable_transfer = T::MinimumClaimableTransferAmounts::get(&currency_id);
		ensure!(
			transfer_amount >= minimum_claimable_transfer,
			Error::<T>::TransferTooLowForCashDrop,
		);
		let setter_fixed_price = <T as Config>::PriceSource::get_setter_price();
		let (cashdrop_numerator, cashdrop_denominator) = T::GetCashDropRate::get();

		if currency_id == T::SetterCurrencyId::get() {
			let transfer_drop = transfer_amount.saturating_mul(cashdrop_denominator.saturating_sub(cashdrop_numerator).unique_saturated_into());
			let into_cashdrop_amount: U256 = U256::from(transfer_amount).saturating_sub(U256::from(transfer_drop));
			let balance_cashdrop_amount = into_cashdrop_amount.and_then(|n| TryInto::<Balance>::try_into(n).ok()).unwrap_or_else(Zero::zero);
			let serp_balance = T::Currency::free_balance(currency_id, &Self::account_id());
			ensure!(
				balance_cashdrop_amount <= serp_balance,
				Error::<T>::CashdropNotAvailable,
			);

			T::Currency::transfer(T::SetterCurrencyId::get(), &Self::account_id(), who, balance_cashdrop_amount);
			Self::deposit_event(Event::CashDrops(T::SetterCurrencyId::get(), who.clone(), balance_cashdrop_amount.clone()));
		} else if T::NonStableDropCurrencyIds::get().contains(&currency_id) {
			let transfer_drop = transfer_amount.saturating_mul(cashdrop_denominator.saturating_sub(cashdrop_numerator).unique_saturated_into());
			let into_cashdrop_amount: U256 = U256::from(transfer_amount).saturating_sub(U256::from(transfer_drop));
			let balance_cashdrop_amount = into_cashdrop_amount.and_then(|n| TryInto::<Balance>::try_into(n).ok()).unwrap_or_else(Zero::zero);
			let serp_balance = T::Currency::free_balance(T::SetterCurrencyId::get(), &Self::account_id());
			ensure!(
				balance_cashdrop_amount <= serp_balance,
				Error::<T>::CashdropNotAvailable,
			);

			// get a price relativity using the DEX pools and use it to provide Setter Cashdrops.
			let (pool_0, pool_1) = T::Dex::get_liquidity_pool(currency_id, T::SetterCurrencyId::get());
			let relative_price = pool_1 / pool_0;
			let relative_cashdrop = balance_cashdrop_amount / relative_price;
		
			T::Currency::transfer(T::SetterCurrencyId::get(), &Self::account_id(), who, relative_cashdrop);
			Self::deposit_event(Event::CashDrops(T::SetterCurrencyId::get(), who.clone(), relative_cashdrop.clone()));
		} else if T::SetCurrencyDropCurrencyIds::get().contains(&currency_id) {
			let transfer_drop = transfer_amount.saturating_mul(cashdrop_denominator.saturating_sub(cashdrop_numerator).unique_saturated_into());
			let into_cashdrop_amount: U256 = U256::from(transfer_amount).saturating_sub(U256::from(transfer_drop));
			let balance_cashdrop_amount = into_cashdrop_amount.and_then(|n| TryInto::<Balance>::try_into(n).ok()).unwrap_or_else(Zero::zero);
			let serp_balance = T::Currency::free_balance(currency_id, &Self::account_id());
			ensure!(
				balance_cashdrop_amount <= serp_balance,
				Error::<T>::CashdropNotAvailable,
			);

			T::Currency::transfer(currency_id, &Self::account_id(), who, balance_cashdrop_amount);
			Self::deposit_event(Event::CashDrops(currency_id, who.clone(), balance_cashdrop_amount.clone()));
		}
		Ok(())
	}
}

impl<T: Config> SerpTreasuryExtended<T::AccountId> for Pallet<T> {
	/// swap Dinar to get exact Setter,
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

		// get a min_target_amount of 105% of market value,
		// marking the 5% slippage of `price_impact_limit`.
		let (pool_0, pool_1) = T::Dex::get_liquidity_pool(setter_currency_id, dinar_currency_id);
		let relative_price = pool_1 / pool_0;
		let max_supply_amount_full = target_amount / relative_price;
		let max_supply_amount_fives = max_supply_amount_full / 20;
		let max_supply_amount = max_supply_amount_fives * 21;
		
		T::Currency::deposit(dinar_currency_id, &Self::account_id(), max_supply_amount)?;
		T::Dex::swap_with_exact_target(
			&Self::account_id(),
			swap_path,
			target_amount,
			max_supply_amount,
			price_impact_limit,
		)
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

		// get a min_target_amount of 105% of market value,
		// marking the 5% slippage of `price_impact_limit`.
		let (pool_0, pool_1) = T::Dex::get_liquidity_pool(currency_id, setter_currency_id);
		let relative_price = pool_1 / pool_0;
		let max_supply_amount_full = target_amount / relative_price;
		let max_supply_amount_fives = max_supply_amount_full / 20;
		let max_supply_amount = max_supply_amount_fives * 21;

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

		// get a min_target_amount of 95% of market value,
		// marking the 5% slippage of `price_impact_limit`.
		let (pool_0, pool_1) = T::Dex::get_liquidity_pool(dinar_currency_id, currency_id);
		let relative_price = pool_1 / pool_0;
		let min_target_amount_full = supply_amount / relative_price;
		let min_target_amount_fives = min_target_amount_full / 20;
		let min_target_amount = min_target_amount_fives * 19;

		T::Dex::swap_with_exact_supply(
			&Self::account_id(),
			swap_path,
			supply_amount,
			min_target_amount,
			price_impact_limit,
		)
	}
}
