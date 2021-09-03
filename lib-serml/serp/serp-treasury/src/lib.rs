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
//! SERP Treasury manages the Setmint, and handle excess serplus
//! and stabilize SetCurrencies standards timely in order to keep the
//! system healthy. It manages the TES (Token Elasticity of Supply).

#![cfg_attr(not(feature = "std"), no_std)]
#![allow(clippy::unused_unit)]

use frame_support::{
	pallet_prelude::*,
	PalletId,
};
use frame_system::pallet_prelude::*;
use orml_traits::{GetByKey, MultiCurrency, MultiCurrencyExtended};
use primitives::{Balance, CurrencyId};
use sp_runtime::{
	DispatchResult, 
	traits::{
		AccountIdConversion, Bounded, One, Saturating, UniqueSaturatedInto, Zero,
	},
	FixedPointNumber
};
use sp_std::{convert::TryInto, prelude::*, vec};
use support::{
	DEXManager, PriceProvider, Ratio, SerpTreasury, SerpTreasuryExtended,
};

mod mock;
mod tests;
pub mod weights;

pub use module::*;
pub use weights::WeightInfo;

type CurrencyBalanceOf<T> = <<T as Config>::Currency as MultiCurrency<<T as frame_system::Config>::AccountId>>::Balance;

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
		/// Setter (SETR) currency Stablecoin currency id
		type SetterCurrencyId: Get<CurrencyId>;

		#[pallet::constant]
		/// The SetUSD currency id, it should be SETUSD in Setheum.
		type GetSetUSDCurrencyId: Get<CurrencyId>;

		#[pallet::constant]
		/// SettinDes (SETHEUM) dexer currency id
		type DirhamCurrencyId: Get<CurrencyId>;

		// CashDrop period for transferring cashdrop from 
		// the `SettPayTreasuryAccountId`.
		// The ideal period is after every `24 hours`.
		type CashDropPeriod: Get<Self::BlockNumber>;

		/// SerpUp pool/account for receiving funds SettPay Cashdrops
		/// SettPayTreasury account.
		#[pallet::constant]
		type SettPayTreasuryAccountId: Get<Self::AccountId>;

		/// The vault account to keep the accumulated Cashdrop doses from the `SettPayTreasuryAccountId`.
		#[pallet::constant]
		type CashDropVaultAccountId: Get<Self::AccountId>;

		/// SerpUp pool/account for receiving funds Setheum Foundation's Charity Fund
		/// CharityFund account.
		type CharityFundAccountId: Get<Self::AccountId>;

		/// Default fee swap path list
		#[pallet::constant]
		type DefaultSwapPathList: Get<Vec<Vec<CurrencyId>>>;

		/// When swap with DEX, the acceptable max slippage for the price from oracle.
		#[pallet::constant]
		type MaxSwapSlippageCompareToOracle: Get<Ratio>;

		/// The limit for length of trading path
		#[pallet::constant]
		type TradingPathLimit: Get<u32>;

		/// The price source to provider external market price.
		type PriceSource: PriceProvider<CurrencyId>;

		/// Dex manager is used to swap reserve asset (Setter) for propper (SetCurrency).
		type Dex: DEXManager<Self::AccountId, CurrencyId, Balance>;

		/// The minimum transfer amounts by currency_id,  to secure cashdrop from dusty claims.
		type MinimumClaimableTransferAmounts: GetByKey<CurrencyId, Balance>;

		/// The origin which may update incentive related params
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
		TransferTooLowForCashDrop,
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
		/// CashDrop has been claimed successfully.
		CashDropClaim(CurrencyId, T::AccountId, Balance),
		/// CashDrop has been deposited to vault successfully.
		CashDropToVault(Balance, CurrencyId),
	}

	/// Mapping to Minimum Claimable Transfer.
	#[pallet::storage]
	#[pallet::getter(fn minimum_claimable_transfer)]
	pub type MinimumClaimableTransfer<T: Config> = StorageMap<_, Twox64Concat, CurrencyId, Balance, OptionQuery>;

	/// The alternative fee swap path of accounts.
	#[pallet::storage]
	#[pallet::getter(fn alternative_fee_swap_path)]
	pub type AlternativeFeeSwapPath<T: Config> =
		StorageMap<_, Twox64Concat, T::AccountId, BoundedVec<CurrencyId, T::TradingPathLimit>, OptionQuery>;

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
			// CashDrop period for transferring cashdrop from 
			// the `SettPayTreasuryAccountId`.
			// The ideal period is after every `24 hours`.
			//
			if now % T::CashDropPeriod::get() == Zero::zero() {
				// Release CashDrop to vault.
				let mut count: u32 = 0;
				if Self::setter_cashdrop_to_vault().is_ok() {
					count += 1;
				};
				if Self::setusd_cashdrop_to_vault().is_ok() {
					count += 1;
				};

				T::WeightInfo::on_initialize(count)
			} else {
				0
			}
		}
	}
	/// set alternative swap path for SERP.
	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// Set fee swap path
		#[pallet::weight(<T as Config>::WeightInfo::set_alternative_swap_path())]
		pub fn set_alternative_swap_path(
			origin: OriginFor<T>,
			fee_swap_path: Option<Vec<CurrencyId>>,
		) -> DispatchResult {
			T::UpdateOrigin::ensure_origin(origin)?;

			if let Some(path) = fee_swap_path {
				let path: BoundedVec<CurrencyId, T::TradingPathLimit> =
					path.try_into().map_err(|_| Error::<T>::InvalidSwapPath)?;
				ensure!(
					path.len() > 1
						&& path[0] != T::GetNativeCurrencyId::get()
						&& path[path.len() - 1] == T::GetNativeCurrencyId::get(),
					Error::<T>::InvalidSwapPath
				);
				AlternativeFeeSwapPath::<T>::insert(&Self::account_id(), &path);
			} else {
				AlternativeFeeSwapPath::<T>::remove(&Self::account_id());
			}
			Ok(())
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
		// Setheum Treasury SerpUp Pool - 40%
		let four: Balance = 4;
		let serping_amount: Balance = four.saturating_mul(amount / 10);
		
		if currency_id == T::SetterCurrencyId::get() {
			// Mint stable currency for buyback swap.
			T::Currency::deposit(currency_id, &Self::account_id(), serping_amount)?;
	
			<Self as SerpTreasuryExtended<T::AccountId>>::swap_exact_setter_to_dinar(
				serping_amount,
			);
		} else {
			// Mint stable currency for buyback swap.
			T::Currency::deposit(currency_id, &Self::account_id(), serping_amount)?;
			<Self as SerpTreasuryExtended<T::AccountId>>::swap_exact_setcurrency_to_dinar(
				currency_id,
				serping_amount,
			);
		} 

		<Pallet<T>>::deposit_event(Event::SerpUpDelivery(amount, currency_id));
		Ok(())
	}

	/// SerpUp ratio for Setheum Foundation's Charity Fund
	fn get_charity_fund_serpup(amount: Balance, currency_id: Self::CurrencyId) -> DispatchResult {
		let charity_fund_account = T::CharityFundAccountId::get();
		// Charity Fund SerpUp Pool - 10%
		let serping_amount: Balance = amount / 10;
		// Issue the SerpUp propper to the SettPayVault
		Self::issue_standard(currency_id, &charity_fund_account, serping_amount)?;

		<Pallet<T>>::deposit_event(Event::SerpUpDelivery(amount, currency_id));
		Ok(())
	}

	/// SerpUp ratio for SettPay Cashdrops
	fn get_cashdrop_serpup(amount: Balance, currency_id: Self::CurrencyId) -> DispatchResult {
		let settpay_account = &T::SettPayTreasuryAccountId::get();

		// SettPay SerpUp Pool - 50%
		let five: Balance = 5;
		let serping_amount: Balance = five.saturating_mul(amount / 10);
		// Issue the SerpUp propper to the SettPayVault
		Self::issue_standard(currency_id, &settpay_account, serping_amount)?;

		<Pallet<T>>::deposit_event(Event::SerpUpDelivery(amount, currency_id));
		Ok(())
	}
	// TODO: Update to 1% per day not 50% per day.
	/// Reward SETR cashdrop to vault
	fn setter_cashdrop_to_vault() -> DispatchResult {
		let free_balance = T::Currency::free_balance(T::SetterCurrencyId::get(), &T::SettPayTreasuryAccountId::get());

		// Send 50% of funds to the CashDropVault
		let five: Balance = 5;
		let cashdrop_amount: Balance = five.saturating_mul(free_balance / 10);
		
		// Transfer the CashDrop propper Rewards to the CashDropVault	
		T::Currency::transfer(T::SetterCurrencyId::get(), &T::SettPayTreasuryAccountId::get(), &T::CashDropVaultAccountId::get(), cashdrop_amount)?;

		<Pallet<T>>::deposit_event(Event::CashDropToVault(cashdrop_amount, T::SetterCurrencyId::get()));
		Ok(())
	}
	// TODO: Update to 1% per day not 50% per day. and rename `setusd` to `setusd`
	/// SerpUp ratio for SettPay Cashdrops
	fn setusd_cashdrop_to_vault() -> DispatchResult {
		let free_balance = T::Currency::free_balance(T::GetSetUSDCurrencyId::get(), &T::SettPayTreasuryAccountId::get());

		// Send 50% of funds to the CashDropVault
		let five: Balance = 5;
		let cashdrop_amount: Balance = five.saturating_mul(free_balance / 10);
		
		// Transfer the CashDrop propper Rewards to the CashDropVault	
		T::Currency::transfer(T::GetSetUSDCurrencyId::get(), &T::SettPayTreasuryAccountId::get(), &T::CashDropVaultAccountId::get(), cashdrop_amount)?;

		<Pallet<T>>::deposit_event(Event::CashDropToVault(cashdrop_amount, T::GetSetUSDCurrencyId::get()));
		Ok(())
	}

	/// issue serpup surplus(stable currencies) to their destinations according to the serpup_ratio.
	fn on_serpup(currency_id: CurrencyId, amount: Balance) -> DispatchResult {
		// ensure that the currency is a SetCurrency
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
		Self::get_cashdrop_serpup(amount, currency_id)?;
		Self::get_charity_fund_serpup(amount, currency_id)?;

		<Pallet<T>>::deposit_event(Event::SerpUp(amount, currency_id));
		Ok(())
	}

	// buy back and burn surplus(stable currencies) with swap on DEX
	// Create the necessary serp down parameters and swap currencies then burn swapped currencies.
	//
	// TODO: Update to add the burning of the stablecoins!
	//
	fn on_serpdown(currency_id: CurrencyId, amount: Balance) -> DispatchResult {
		// ensure that the currency is a SetCurrency
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
			);
		} else {
			<Self as SerpTreasuryExtended<T::AccountId>>::swap_setter_to_exact_setcurrency(
				currency_id,
				amount,
			);
		} 

		<Pallet<T>>::deposit_event(Event::SerpDown(amount, currency_id));
		Ok(())
	}

	// get the minimum supply of a setcurrency - by key
	fn get_minimum_supply(currency_id: CurrencyId) -> Balance {
		T::GetStableCurrencyMinimumSupply::get(&currency_id)
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

	/// Burn Reserve asset (Setter (SETR))
	fn burn_setter(who: &T::AccountId, setter: Self::Balance) -> DispatchResult {
		T::Currency::withdraw(T::SetterCurrencyId::get(), who, setter)
	}

	/// deposit reserve asset (Setter (SETR)) to serp treasury by `who`
	fn deposit_setter(from: &T::AccountId, amount: Self::Balance) -> DispatchResult {
		T::Currency::transfer(T::SetterCurrencyId::get(), from, &Self::account_id(), amount)
	}

	/// claim cashdrop of `currency_id` relative to `transfer_amount` for `who`
	fn claim_cashdrop(currency_id: CurrencyId, who: &T::AccountId, transfer_amount: Balance) -> DispatchResult {
		ensure!(
			T::StableCurrencyIds::get().contains(&currency_id),
			Error::<T>::InvalidCurrencyType,
		);
		let minimum_claimable_transfer = T::MinimumClaimableTransferAmounts::get(&currency_id);
		ensure!(
			transfer_amount >= minimum_claimable_transfer,
			Error::<T>::TransferTooLowForCashDrop,
		);

		if currency_id == T::SetterCurrencyId::get() {
			let balance_cashdrop_amount = transfer_amount / 50; // 2% cashdrop
			let serp_balance = T::Currency::free_balance(currency_id, &Self::account_id());
			ensure!(
				balance_cashdrop_amount <= serp_balance,
				Error::<T>::CashdropNotAvailable,
			);

			T::Currency::transfer(T::SetterCurrencyId::get(), &Self::account_id(), who, balance_cashdrop_amount)?;
			Self::deposit_event(Event::CashDropClaim(T::SetterCurrencyId::get(), who.clone(), balance_cashdrop_amount.clone()));
		} else {
			let balance_cashdrop_amount = transfer_amount / 25; // 4% cashdrop
			let serp_balance = T::Currency::free_balance(currency_id, &Self::account_id());
			ensure!(
				balance_cashdrop_amount <= serp_balance,
				Error::<T>::CashdropNotAvailable,
			);

			T::Currency::transfer(currency_id, &Self::account_id(), who, balance_cashdrop_amount)?;
			Self::deposit_event(Event::CashDropClaim(currency_id, who.clone(), balance_cashdrop_amount.clone()));
		}
		Ok(())
	}
}

impl<T: Config> SerpTreasuryExtended<T::AccountId> for Pallet<T> {
	/// swap Dinar to get exact Setter,
	/// return actual supply Dinar amount
	#[allow(unused_variables)]
	fn swap_dinar_to_exact_setter(
		target_amount: Balance,
	) {
		let native_currency_id = T::GetNativeCurrencyId::get();
		let dinar_currency_id = T::GetNativeCurrencyId::get();
		let setter_currency_id = T::SetterCurrencyId::get();
		
		let default_fee_swap_path_list = T::DefaultSwapPathList::get();
		let swap_path: Vec<Vec<CurrencyId>> = 
			if let Some(path) = AlternativeFeeSwapPath::<T>::get(&Self::account_id()) {
				vec![vec![path.into_inner()], default_fee_swap_path_list].concat()
			} else {
				default_fee_swap_path_list
			};

		for path in swap_path {
			match path.last() {
				Some(setter_currency_id) if *setter_currency_id == native_currency_id => {
					let dinar_currency_id = *path.first().expect("these's first guaranteed by match");
					// calculate the supply limit according to oracle price and the slippage limit,
					// if oracle price is not avalible, do not limit
					let max_supply_limit = if let Some(target_price) =
						T::PriceSource::get_relative_price(*setter_currency_id, dinar_currency_id)
					{
						Ratio::one()
							.saturating_sub(T::MaxSwapSlippageCompareToOracle::get())
							.reciprocal()
							.unwrap_or_else(Ratio::max_value)
							.saturating_mul_int(target_price.saturating_mul_int(target_amount))
					} else {
						CurrencyBalanceOf::<T>::max_value()
					};

					if T::Currency::deposit(
						dinar_currency_id,
						&Self::account_id(),
						max_supply_limit.unique_saturated_into()
					).is_ok() {
						if T::Dex::swap_with_exact_target(
							&Self::account_id(),
							&path,
							target_amount.unique_saturated_into(),
							<T as Config>::Currency::free_balance(dinar_currency_id, &Self::account_id())
								.min(max_supply_limit.unique_saturated_into()),
						)
						.is_ok()
						{
							// successfully swap, break iteration
							break;
						}
					}
				}
				_ => {}
			}
		}
	}

	/// Swap exact amount of Setter to SetCurrency,
	/// return actual target SetCurrency amount
	///
	/// 
	/// When SetCurrency needs SerpDown
	/// 
	#[allow(unused_variables)]
	fn swap_setter_to_exact_setcurrency(
		currency_id: CurrencyId,
		target_amount: Balance,
	) {
		let native_currency_id = T::GetNativeCurrencyId::get();
		let setter_currency_id = T::SetterCurrencyId::get();

		let default_fee_swap_path_list = T::DefaultSwapPathList::get();
		let swap_path: Vec<Vec<CurrencyId>> = 
			if let Some(path) = AlternativeFeeSwapPath::<T>::get(&Self::account_id()) {
				vec![vec![path.into_inner()], default_fee_swap_path_list].concat()
			} else {
				default_fee_swap_path_list
			};

		for path in swap_path {
			match path.last() {
				Some(currency_id) if *currency_id == native_currency_id => {
					let setter_currency_id = *path.first().expect("these's first guaranteed by match");
					// calculate the supply limit according to oracle price and the slippage limit,
					// if oracle price is not avalible, do not limit
					let max_supply_limit = if let Some(target_price) =
						T::PriceSource::get_relative_price(*currency_id, setter_currency_id)
					{
						Ratio::one()
							.saturating_sub(T::MaxSwapSlippageCompareToOracle::get())
							.reciprocal()
							.unwrap_or_else(Ratio::max_value)
							.saturating_mul_int(target_price.saturating_mul_int(target_amount))
					} else {
						CurrencyBalanceOf::<T>::max_value()
					};

					if T::Currency::deposit(
						setter_currency_id,
						&Self::account_id(),
						max_supply_limit.unique_saturated_into()
					).is_ok() {
						if T::Dex::swap_with_exact_target(
							&Self::account_id(),
							&path,
							target_amount.unique_saturated_into(),
							<T as Config>::Currency::free_balance(setter_currency_id, &Self::account_id())
								.min(max_supply_limit.unique_saturated_into()),
						)
						.is_ok()
						{
							// successfully swap, break iteration
							break;
						}
					}
				}
				_ => {}
			}
		}
	}

	/// Swap exact amount of Setter to Dinar,
	/// return actual supply Setter amount
	///
	/// 
	/// When Setter gets SerpUp
	#[allow(unused_variables)]
	fn swap_exact_setter_to_dinar(
		supply_amount: Balance,
	) {
		let native_currency_id = T::GetNativeCurrencyId::get();
		let dinar_currency_id = T::GetNativeCurrencyId::get();
		let currency_id = T::SetterCurrencyId::get();

		let default_fee_swap_path_list = T::DefaultSwapPathList::get();
		let swap_path: Vec<Vec<CurrencyId>> = 
			if let Some(path) = AlternativeFeeSwapPath::<T>::get(&Self::account_id()) {
				vec![vec![path.into_inner()], default_fee_swap_path_list].concat()
			} else {
				default_fee_swap_path_list
			};

		for path in swap_path {
			match path.last() {
				Some(dinar_currency_id) if *dinar_currency_id == native_currency_id => {
					let currency_id = *path.first().expect("these's first guaranteed by match");
					// calculate the supply limit according to oracle price and the slippage limit,
					// if oracle price is not avalible, do not limit
					let min_target_limit = if let Some(target_price) =
						T::PriceSource::get_relative_price(*dinar_currency_id, currency_id)
					{
						Ratio::one()
							.saturating_sub(T::MaxSwapSlippageCompareToOracle::get())
							.reciprocal()
							.unwrap_or_else(Ratio::max_value)
							.saturating_mul_int(target_price.saturating_mul_int(supply_amount))
					} else {
						CurrencyBalanceOf::<T>::max_value()
					};

					// Swap and burn Native Reserve asset (Dinar (DNAR))
					if T::Dex::swap_with_exact_supply(
						&Self::account_id(),
						&path,
						supply_amount.unique_saturated_into(),
						min_target_limit.unique_saturated_into(),
					)
					.is_ok()
					&& T::Currency::withdraw( T::GetNativeCurrencyId::get(), &Self::account_id(), min_target_limit)
					.is_ok()
					{
						// successfully swap, break iteration.
						break;
					}
				}
				_ => {}
			}
		}
	}

	/// Swap exact amount of Setter to Dinar,
	/// return actual supply Setter amount
	///
	/// 
	/// When Setter gets SerpUp
	#[allow(unused_variables)]
	fn swap_exact_setcurrency_to_dinar(
		currency_id: CurrencyId,
		supply_amount: Balance,
	) {
		let native_currency_id = T::GetNativeCurrencyId::get();
		let dinar_currency_id = T::GetNativeCurrencyId::get();

		let default_fee_swap_path_list = T::DefaultSwapPathList::get();
		let swap_path: Vec<Vec<CurrencyId>> = 
			if let Some(path) = AlternativeFeeSwapPath::<T>::get(&Self::account_id()) {
				vec![vec![path.into_inner()], default_fee_swap_path_list].concat()
			} else {
				default_fee_swap_path_list
			};

		for path in swap_path {
			match path.last() {
				Some(dinar_currency_id) if *dinar_currency_id == native_currency_id => {
					let currency_id = *path.first().expect("these's first guaranteed by match");
					// calculate the supply limit according to oracle price and the slippage limit,
					// if oracle price is not avalible, do not limit
					let min_target_limit = if let Some(target_price) =
						T::PriceSource::get_relative_price(*dinar_currency_id, currency_id)
					{
						Ratio::one()
							.saturating_sub(T::MaxSwapSlippageCompareToOracle::get())
							.reciprocal()
							.unwrap_or_else(Ratio::max_value)
							.saturating_mul_int(target_price.saturating_mul_int(supply_amount))
					} else {
						CurrencyBalanceOf::<T>::max_value()
					};

					if T::Dex::swap_with_exact_supply(
						&Self::account_id(),
						&path,
						supply_amount.unique_saturated_into(),
						min_target_limit.unique_saturated_into(),
					)
					.is_ok()
					&& T::Currency::withdraw( T::GetNativeCurrencyId::get(), &Self::account_id(), min_target_limit)
					.is_ok()
					{
						// successfully swap, break iteration
						break;
					}
				}
				_ => {}
			}
		}
	}
}
