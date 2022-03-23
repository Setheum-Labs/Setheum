// بِسْمِ اللَّهِ الرَّحْمَنِ الرَّحِيم

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
//! SERP Treasury manages the SERP, and handle excess serplus
//! and stabilize SetCurrencies standards timely in order to keep the
//! system healthy. It manages the TES (Token Elasticity of Supply).

#![cfg_attr(not(feature = "std"), no_std)]
#![allow(clippy::unused_unit)]

use frame_support::{log, pallet_prelude::*, transactional, PalletId};
use frame_system::pallet_prelude::*;
use orml_traits::{GetByKey, MultiCurrency, MultiCurrencyExtended};
use primitives::{Balance, CurrencyId, SerpStableCurrencyId};
use sp_core::U256;
use sp_runtime::{
	DispatchResult, 
	traits::{
		AccountIdConversion, Bounded, Saturating, One, Zero,
	},
	FixedPointNumber,
};
use sp_std::{convert::TryInto, prelude::*, vec};
use support::{
	DEXManager, PriceProvider, Ratio, SerpTreasury, SerpTreasuryExtended, SwapLimit
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

		#[pallet::constant]
		/// A duration period for performing SERP-TES Operations.
		type StableCurrencyInflationPeriod: Get<Self::BlockNumber>;

		/// The minimum total supply/issuance required to keep a currency live on SERP.
		/// This can be bypassed through the CDP-Treasury if necessary.
		type GetStableCurrencyMinimumSupply: GetByKey<CurrencyId, Balance>;

		#[pallet::constant]
		/// Native (SETM) currency id
		type GetNativeCurrencyId: Get<CurrencyId>;

		#[pallet::constant]
		/// The Serp (SERP) currency id
		type GetSerpCurrencyId: Get<CurrencyId>;

		#[pallet::constant]
		/// The Dinar (DNAR) currency id
		type GetDinarCurrencyId: Get<CurrencyId>;

		#[pallet::constant]
		/// HighEnd LaunchPad (HELP) currency id. (LaunchPad Token)
		/// 
		type GetHelpCurrencyId: Get<CurrencyId>;

		#[pallet::constant]
		/// Setter (SETR) currency id
		/// 
		type SetterCurrencyId: Get<CurrencyId>;

		#[pallet::constant]
		/// The SetUSD currency id, it should be SETUSD in Setheum.
		type GetSetUSDId: Get<CurrencyId>;

		// CashDrop period for transferring cashdrop.
		// The ideal period is after every `24 hours`.
		// type CashDropPeriod: Get<Self::BlockNumber>;

		/// CDP-Treasury account for processing serplus funds 
		/// CDPTreasury account.
		#[pallet::constant]
		type CDPTreasuryAccountId: Get<Self::AccountId>;

		/// When swap with DEX, the acceptable max slippage for the price from oracle.
		type MaxSwapSlippageCompareToOracle: Get<Ratio>;

		/// The price source to provider external market price.
		type PriceSource: PriceProvider<CurrencyId>;

		/// The alternative swap path joint list, which can be concated to
		/// alternative swap path when SERP-Treasury swaps for buyback.
		#[pallet::constant]
		type AlternativeSwapPathJointList: Get<Vec<Vec<CurrencyId>>>;

		/// Dex manager is used to swap reserve asset (Setter) for propper (SetCurrency).
		type Dex: DEXManager<Self::AccountId, CurrencyId, Balance>;

		/// The minimum transfer amounts for SETR, that is eligible for cashdrop.
		type SetterMinimumClaimableTransferAmounts: Get<Balance>;

		/// The maximum transfer amounts for SETR, that is eligible for cashdrop.
		type SetterMaximumClaimableTransferAmounts: Get<Balance>;

		/// The minimum transfer amounts for SETUSD, that is eligible for cashdrop.
		type SetDollarMinimumClaimableTransferAmounts: Get<Balance>;

		/// The maximum transfer amounts for SETUSD, that is eligible for cashdrop.
		type SetDollarMaximumClaimableTransferAmounts: Get<Balance>;

		/// The origin which may update inflation related params
		type UpdateOrigin: EnsureOrigin<Self::Origin>;

		#[pallet::constant]
		/// The SERP Treasury's module id, keeps serplus and reserve asset.
		type PalletId: Get<PalletId>;

		/// Weight information for the extrinsics in this module.
		type WeightInfo: WeightInfo;
	}

	#[pallet::error]
	pub enum Error<T> {
		/// The SERP Cannot Deposit for buyback
		CannotDeposit,
		/// The SERP Cannot Swap for buyback
		CannotSwap,
		/// The SetSwap DEX is not available for this operation
		DexNotAvailable,
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
		InvalidSwapPath
	}

	#[pallet::event]
	#[pallet::generate_deposit(pub(crate) fn deposit_event)]
	pub enum Event<T: Config> {
		/// Currency SerpTes has been triggered.
		SerpTes(CurrencyId),
		/// Currency SerpUp has been delivered successfully.
		SerpUpDelivery(Balance, CurrencyId),
		/// Currency SerpUp has been delivered successfully.
		SerplusDelivery(Balance, CurrencyId),
		/// Currency SerpUp has been completed successfully.
		SerpUp(Balance, CurrencyId),
		/// Currency SerpDown has been triggered successfully.
		SerpDown(Balance, CurrencyId),
		/// CashDrop has been claimed successfully.
		CashDropClaim(CurrencyId, T::AccountId, Balance),
		/// CashDrop has been deposited to vault successfully.
		CashDropToVault(Balance, CurrencyId),
		/// Stable Currency Inflation Rate Updated
		StableCurrencyInflationRateUpdated(SerpStableCurrencyId, Balance),
		/// Stable Currency Inflation Rate Delivered
		InflationDelivery(CurrencyId, Balance),
		/// SerpSwapDinarToExactSetter
		SerpSwapDinarToExactSetter(CurrencyId, CurrencyId, Balance, Balance),
		/// SerpSwapSerpToExactSetter
		SerpSwapSerpToExactSetter(CurrencyId, CurrencyId, Balance, Balance),
		/// SerpSwapDinarToExactStable
		SerpSwapDinarToExactStable(CurrencyId, CurrencyId, Balance, Balance),
		/// SerpSwapSetterToExactSetDollar
		SerpSwapSetterToExactSetDollar(CurrencyId, CurrencyId, Balance, Balance),
		/// SerpSwapSerpToExactStable
		SerpSwapSerpToExactStable(CurrencyId, CurrencyId, Balance, Balance),
		/// SerpSwapExactStableToDinar
		SerpSwapExactStableToDinar(CurrencyId, CurrencyId, Balance, Balance),
		/// SerpSwapExactStableToSetter
		SerpSwapExactStableToSetter(CurrencyId, CurrencyId, Balance, Balance),
		/// SerplusSwapExactStableToSetter
		SerplusSwapExactStableToSetter(CurrencyId, CurrencyId, Balance, Balance),
		/// SerplusSwapExactStableToNative
		SerplusSwapExactStableToNative(CurrencyId, CurrencyId, Balance, Balance),
		/// SerpSwapExactStableToNative
		SerpSwapExactStableToNative(CurrencyId, CurrencyId, Balance, Balance),
		/// SerplusSwapExactStableToHelp
		SerplusSwapExactStableToHelp(CurrencyId, CurrencyId, Balance, Balance),
		/// SerpSwapExactStableToHelp
		SerpSwapExactStableToHelp(CurrencyId, CurrencyId, Balance, Balance),
		/// SerpSwapExactStableToSerpToken
		SerpSwapExactStableToSerpToken(CurrencyId, CurrencyId, Balance, Balance),
		/// SerpSwapExactStableToNative
		TopUpCashDropPool(CurrencyId, Balance),
		/// SerpSwapExactStableToSerpToken
		IssueCashDropFromPool(T::AccountId, CurrencyId, Balance),
		/// Force SerpDown Risk Management
		ForceSerpDown(CurrencyId, Balance)
	}

	/// The CashDrop Pool
	///
	/// CashDropPool: map CurrencyId => Balance
	#[pallet::storage]
	#[pallet::getter(fn cashdrop_pool)]
	pub type CashDropPool<T: Config> = StorageMap<_, Twox64Concat, CurrencyId, Balance, ValueQuery>;

	/// The Total count of CashDrops successfully been claimed
	///
	/// CashDropCount: map CurrencyId => u32
	#[pallet::storage]
	#[pallet::getter(fn cashdrop_count)]
	pub type CashDropCount<T: Config> = StorageMap<_, Twox64Concat, CurrencyId, u32, ValueQuery>;

	/// The Total amounts of CashDrops successfully been claimed
	///
	/// CashDrops: map CurrencyId => Balance
	#[pallet::storage]
	#[pallet::getter(fn total_cashdrops)]
	pub type CashDrops<T: Config> = StorageMap<_, Twox64Concat, CurrencyId, Balance, ValueQuery>;

	/// The inflation rate amount per StableCurrencyInflationPeriod of specific
	/// stable currency type.
	///
	/// StableCurrencyInflationRate: map CurrencyId => Balance
	#[pallet::storage]
	#[pallet::getter(fn stable_currency_inflation_rate)]
	pub type StableCurrencyInflationRate<T: Config> = StorageMap<_, Twox64Concat, CurrencyId, Balance, ValueQuery>;

	#[pallet::genesis_config]
	pub struct GenesisConfig {
		pub stable_currency_inflation_rate: Vec<(CurrencyId, Balance)>,
		pub stable_currency_cashdrop: Vec<(CurrencyId, Balance)>,
	}

	#[cfg(feature = "std")]
	impl Default for GenesisConfig {
		fn default() -> Self {
			GenesisConfig {
				stable_currency_inflation_rate: vec![],
				stable_currency_cashdrop: vec![],
			}
		}
	}

	#[pallet::genesis_build]
	impl<T: Config> GenesisBuild<T> for GenesisConfig {
		fn build(&self) {
			self.stable_currency_inflation_rate
				.iter()
				.for_each(|(currency_id, size)| {
					StableCurrencyInflationRate::<T>::insert(currency_id, size);
				});
			self.stable_currency_cashdrop
				.iter()
				.for_each(|(currency_id, amount)| {
					// ensure that the currency is a SetCurrency
					if T::StableCurrencyIds::get().contains(&currency_id) {
						CashDropPool::<T>::insert(currency_id, amount);
					}
			});
		}
	}
	
	#[pallet::pallet]
	pub struct Pallet<T>(PhantomData<T>);

	#[pallet::hooks]
	impl<T: Config> Hooks<T::BlockNumber> for Pallet<T> {
		///
		/// NOTE: This function is called BEFORE ANY extrinsic in a block is applied,
		/// including inherent extrinsics. Hence for instance, if you runtime includes
		/// `pallet_timestamp`, the `timestamp` is not yet up to date at this point.
		/// Handle excessive surplus or debits of system when block end
		///
		/// Triggers Serping for all system stablecoins at every block.
		fn on_initialize(now: T::BlockNumber) -> Weight {
			// SERP-TES Adjustment Frequency and SetCurrency Inflation frequency.
			// Schedule for when to trigger SERP-TES and SERP-Inflation
			// (Blocktime/BlockNumber - every blabla block)

			if now % T::StableCurrencyInflationPeriod::get() == Zero::zero() {
				let mut count: u32 = 0;
				count += 1;
				
				Self::stable_inflation_on_initialize();
				count += 1;
				Self::serp_tes_on_initialize();
				count += 1;

				T::WeightInfo::on_initialize(count)
			} else {
				0
			}
		}
	}

	/// set alternative swap path for SERP.
	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// Update parameters related to stable currency inflation rate under specific
		/// stable currency type
		///
		/// The dispatch origin of this call must be `UpdateOrigin`.
		///
		/// - `currency_id`: stable currency type
		/// - `amount`: inflation rate amount size
		#[pallet::weight((T::WeightInfo::set_stable_currency_inflation_rate(), DispatchClass::Operational))]
		#[transactional]
		pub fn set_stable_currency_inflation_rate(
			origin: OriginFor<T>,
			currency_id: SerpStableCurrencyId,
			size: Balance,
		) -> DispatchResultWithPostInfo {
			T::UpdateOrigin::ensure_origin(origin)?;
			if currency_id == SerpStableCurrencyId::SETR {
				StableCurrencyInflationRate::<T>::insert(T::SetterCurrencyId::get(), size);
			} else if currency_id == SerpStableCurrencyId::SETUSD {
				StableCurrencyInflationRate::<T>::insert(T::GetSetUSDId::get(), size);
			}
			Self::deposit_event(Event::StableCurrencyInflationRateUpdated(currency_id, size));
			Ok(().into())
		}
		/// Update parameters related to stable currency serpdown for specific
		/// stable currency type
		///
		/// The dispatch origin of this call must be `UpdateOrigin`.
		///
		/// - `currency_id`: stable currency type
		/// - `amount`: serpdown contraction amount size
		#[pallet::weight((T::WeightInfo::force_serpdown(), DispatchClass::Operational))]
		#[transactional]
		pub fn force_serpdown(
			origin: OriginFor<T>,
			currency_id: CurrencyId,
			size: Balance,
		) -> DispatchResultWithPostInfo {
			T::UpdateOrigin::ensure_origin(origin)?;
			<Self as SerpTreasury<T::AccountId>>::on_serpdown(currency_id, size)?;
			Self::deposit_event(Event::ForceSerpDown(currency_id, size));
			Ok(().into())
		}
	}
}

impl<T: Config> Pallet<T> {
	/// Get account of SERP Treasury module.
	pub fn account_id() -> T::AccountId {
		T::PalletId::get().into_account()
	}

	/// Called in `on_initialize` for SERP-TES
	fn serp_tes_on_initialize() {
		// SERP-TES Adjustment Frequency and SetCurrency Inflation frequency.
		// Schedule for when to trigger SERP-TES and SERP-Inflation
		// (Blocktime/BlockNumber - every blabla block)
		
		let res = <Self as SerpTreasury<T::AccountId>>::serp_tes_now();

		match res {
			Ok(_) => {}
			Err(e) => {
				log::warn!(
					target: "serp_treasury",
					"serp_tes_now: failed to algorithmically serp setcurrencies: {:?}.\
					This is unexpected but should be safe",
					e
				);
			}
		}
	}

	/// Called in `on_initialize` for SERP-StableInflation
	fn stable_inflation_on_initialize() {
		// SERP-TES Adjustment Frequency and SetCurrency Inflation frequency.
		// Schedule for when to trigger SERP-TES
		// (Blocktime/BlockNumber - every blabla block)
		
		let res = <Self as SerpTreasury<T::AccountId>>::serp_tes_now();

		match res {
			Ok(_) => {}
			Err(e) => {
				log::warn!(
					target: "serp_treasury",
					"issue_stablecurrency_inflation: failed to algorithmically serp setcurrencies: {:?}.\
					This is unexpected but should be safe",
					e
				);
			}
		}
	}
}

impl<T: Config> SerpTreasury<T::AccountId> for Pallet<T> {
	type Balance = Balance;
	type CurrencyId = CurrencyId;

	/// Calculate the amount of supply change from a fraction given as `numerator` and `denominator`.
	fn calculate_supply_change(numerator: Balance, denominator: Balance, supply: Balance) -> Balance {
		if numerator.is_zero() || denominator.is_zero() || supply.is_zero() {
			Zero::zero()
		} else {
			let one: Balance = 1;
			let the_one: U256 = U256::from(one);
			let fraction: U256 = U256::from(numerator) / U256::from(denominator);
			let supply_make: U256 = U256::from(supply)
				.saturating_mul(U256::from(one)).saturating_sub(the_one);

			fraction.saturating_mul(U256::from(supply_make))
			.checked_div(the_one)
			.and_then(|n| TryInto::<Balance>::try_into(n).ok())
			.unwrap_or_else(Zero::zero)
		}
	}
	
	/// Deliver System StableCurrency stability for the SetCurrencies.
	fn serp_tes_now() -> DispatchResult {

		// Currencies
		let setr = T::SetterCurrencyId::get();
		let dinar = T::GetDinarCurrencyId::get();
		let serp = T::GetSerpCurrencyId::get();
		let setusd = T::GetSetUSDId::get();

		// SETR Pools
		let (stable_pool_1d, dnar_pool1) = T::Dex::get_liquidity_pool(setr, dinar);
		let (stable_pool_1s,serp_pool1) = T::Dex::get_liquidity_pool(setr, serp);

		// SETUSD Pools
		let (stable_pool_2d, dnar_pool2) = T::Dex::get_liquidity_pool(setusd, dinar);
		let (stable_pool_2s,serp_pool2) = T::Dex::get_liquidity_pool(setusd, serp);

		// ensure that the SetSwap DEX liquidity pools are not empty
		if stable_pool_1d.is_zero() || dnar_pool1.is_zero() || stable_pool_1s.is_zero() || serp_pool1.is_zero() {
			return Ok(());
		} else if !stable_pool_1d.is_zero() || !dnar_pool1.is_zero() || !stable_pool_1s.is_zero() || !serp_pool1.is_zero() {
			// SETR prices
			let stable_pool_1d_relative_price = stable_pool_1d.saturating_div(dnar_pool1);
			let stable_pool_1s_relative_price = stable_pool_1s.saturating_div(serp_pool1);
			let stable_pool_1cumulative_prices: Balance = stable_pool_1d_relative_price.saturating_add(stable_pool_1s_relative_price);
			let stable_pool_1average_price: Balance = stable_pool_1cumulative_prices.saturating_div(2);

			// SETUSD prices
			let stable_pool_2d_relative_price = stable_pool_2d.saturating_div(dnar_pool2);
			let stable_pool_2s_relative_price = stable_pool_2s.saturating_div(serp_pool2);
			let stable_pool_2cumulative_prices: Balance = stable_pool_2d_relative_price.saturating_add(stable_pool_2s_relative_price);
			let stable_pool_2average_price: Balance = stable_pool_2cumulative_prices.saturating_div(2);


			let base_peg: Balance = 4;

			let base_unit = stable_pool_2average_price.saturating_mul(base_peg);

			match stable_pool_1average_price {
				0 => {} 
				stable_pool_1average_price if stable_pool_1average_price < base_unit => {
					// safe from underflow because `stable_pool_1average_price` is checked to be greater than `base_unit`
					let supply = T::Currency::total_issuance(setr);
					let expand_setr_by = Self::calculate_supply_change(stable_pool_1average_price, base_unit, supply);
					let contract_setusd_by = expand_setr_by.saturating_div(base_peg);
					// serpup SETR and serpdown SETUSD both to meet halfway
					Self::on_serpup(setr, expand_setr_by)?;
					Self::on_serpdown(setusd, contract_setusd_by)?;
				}
				stable_pool_1average_price if stable_pool_1average_price > base_unit => {
					// safe from underflow because `stable_pool_1average_price` is checked to be less than `base_unit`
					let supply = T::Currency::total_issuance(setr);
					let contract_setr_by = Self::calculate_supply_change(base_unit, stable_pool_1average_price, supply);
					let expand_setusd_by = contract_setr_by.saturating_div(base_peg);
					// serpup SETR and serpdown SETUSD both to meet halfway
					Self::on_serpdown(setr, contract_setr_by)?;
					Self::on_serpup(setusd, expand_setusd_by)?;
				}
				_ => {}
			}
		}
		Ok(())
	}

	/// Deliver System StableCurrency Inflation
	fn issue_stablecurrency_inflation() -> DispatchResult {

		for currency_id in T::StableCurrencyIds::get() {
			// Amounts are 20% of the inflation rate amount for each distro.
			// (CashDropPool, DNAR, SERP, SETM, HELP)
			let one: Balance = 1;
			let inflation_amount = Self::stable_currency_inflation_rate(currency_id);
			let inflamounts: Balance = one.saturating_mul(inflation_amount.saturating_div(5));

			if inflation_amount != 0 {
				// calculate the target amount limit according to oracle price and the slippage limit,
				// if oracle price is not avalible, do not limit amount (get min_value)
				let dinar_min_target_amount = if let Some(target_price) =
					T::PriceSource::get_relative_price(T::GetDinarCurrencyId::get(), currency_id)
				{
					Ratio::one()
						.saturating_sub(T::MaxSwapSlippageCompareToOracle::get())
						.reciprocal()
						.unwrap_or_else(Ratio::min_value)
						.saturating_mul_int(target_price.saturating_mul_int(inflamounts))
				} else {
					CurrencyBalanceOf::<T>::min_value()
				};
				// calculate the target amount limit according to oracle price and the slippage limit,
				// if oracle price is not avalible, do not limit amount (get min_value)
				let serp_min_target_amount = if let Some(target_price) =
					T::PriceSource::get_relative_price(T::GetSerpCurrencyId::get(), currency_id)
				{
					Ratio::one()
						.saturating_sub(T::MaxSwapSlippageCompareToOracle::get())
						.reciprocal()
						.unwrap_or_else(Ratio::min_value)
						.saturating_mul_int(target_price.saturating_mul_int(inflamounts))
				} else {
					CurrencyBalanceOf::<T>::min_value()
				};
				// calculate the target amount limit according to oracle price and the slippage limit,
				// if oracle price is not avalible, do not limit amount (get min_value)
				let native_min_target_amount = if let Some(target_price) =
					T::PriceSource::get_relative_price(T::GetNativeCurrencyId::get(), currency_id)
				{
					Ratio::one()
						.saturating_sub(T::MaxSwapSlippageCompareToOracle::get())
						.reciprocal()
						.unwrap_or_else(Ratio::min_value)
						.saturating_mul_int(target_price.saturating_mul_int(inflamounts))
				} else {
					CurrencyBalanceOf::<T>::min_value()
				};
				// calculate the target amount limit according to oracle price and the slippage limit,
				// if oracle price is not avalible, do not limit amount (get min_value)
				let help_min_target_amount = if let Some(target_price) =
					T::PriceSource::get_relative_price(T::GetHelpCurrencyId::get(), currency_id)
				{
					Ratio::one()
						.saturating_sub(T::MaxSwapSlippageCompareToOracle::get())
						.reciprocal()
						.unwrap_or_else(Ratio::min_value)
						.saturating_mul_int(target_price.saturating_mul_int(inflamounts))
				} else {
					CurrencyBalanceOf::<T>::min_value()
				};
		
				// inflation distros
				// 1
				Self::add_cashdrop_to_pool(currency_id, inflamounts)?;
				// 2
				<Self as SerpTreasuryExtended<T::AccountId>>::buyback_swap_with_exact_supply(
					currency_id,
					T::GetDinarCurrencyId::get(),
					SwapLimit::ExactSupply(inflamounts, dinar_min_target_amount),
				)?;
				// 3
				<Self as SerpTreasuryExtended<T::AccountId>>::buyback_swap_with_exact_supply(
					currency_id,
					T::GetSerpCurrencyId::get(),
					SwapLimit::ExactSupply(inflamounts, serp_min_target_amount),
				)?;
				// 4
				<Self as SerpTreasuryExtended<T::AccountId>>::buyback_swap_with_exact_supply(
					currency_id,
					T::GetNativeCurrencyId::get(),
					SwapLimit::ExactSupply(inflamounts, native_min_target_amount),
				)?;
				// 5
				<Self as SerpTreasuryExtended<T::AccountId>>::buyback_swap_with_exact_supply(
					currency_id,
					T::GetHelpCurrencyId::get(),
					SwapLimit::ExactSupply(inflamounts, help_min_target_amount),
				)?;
				Self::deposit_event(Event::InflationDelivery(currency_id, inflation_amount));
			};
		}
		Ok(())
	}

	/// SerpUp ratio for BuyBack Swaps to burn bought assets.
	fn get_buyback_serpup(amount: Balance, currency_id: Self::CurrencyId) -> DispatchResult {
		// Buyback with 50:50 with DNAR:SERP
		let one: Balance = 1;
		let amount_50percent: Balance = one.saturating_mul(amount.saturating_div(2));

		// calculate the target amount limit according to oracle price and the slippage limit,
		// if oracle price is not avalible, do not limit amount (get min_value)
		let dinar_min_target_amount = if let Some(target_price) =
			T::PriceSource::get_relative_price(T::GetDinarCurrencyId::get(), currency_id)
		{
			Ratio::one()
				.saturating_sub(T::MaxSwapSlippageCompareToOracle::get())
				.reciprocal()
				.unwrap_or_else(Ratio::min_value)
				.saturating_mul_int(target_price.saturating_mul_int(amount_50percent))
		} else {
			CurrencyBalanceOf::<T>::min_value()
		};
		// calculate the target amount limit according to oracle price and the slippage limit,
		// if oracle price is not avalible, do not limit amount (get min_value)
		let serp_min_target_amount = if let Some(target_price) =
			T::PriceSource::get_relative_price(T::GetSerpCurrencyId::get(), currency_id)
		{
			Ratio::one()
				.saturating_sub(T::MaxSwapSlippageCompareToOracle::get())
				.reciprocal()
				.unwrap_or_else(Ratio::min_value)
				.saturating_mul_int(target_price.saturating_mul_int(amount_50percent))
		} else {
			CurrencyBalanceOf::<T>::min_value()
		};

		<Self as SerpTreasuryExtended<T::AccountId>>::buyback_swap_with_exact_supply(
			currency_id,
			T::GetDinarCurrencyId::get(),
			SwapLimit::ExactSupply(amount_50percent, dinar_min_target_amount),
		)?;
		// 3
		<Self as SerpTreasuryExtended<T::AccountId>>::buyback_swap_with_exact_supply(
			currency_id,
			T::GetSerpCurrencyId::get(),
			SwapLimit::ExactSupply(amount_50percent, serp_min_target_amount),
		)?;

		Self::deposit_event(Event::SerpUpDelivery(amount, currency_id));
		Ok(())
	}

	/// Add CashDrop to the pool
	fn add_cashdrop_to_pool(currency_id: Self::CurrencyId, amount: Balance) -> DispatchResult {
		let cashdrop_pool_balance = Self::cashdrop_pool(currency_id);
		let updated_cashdrop_pool_balance = cashdrop_pool_balance + amount;

		CashDropPool::<T>::insert(currency_id, updated_cashdrop_pool_balance);
		Self::deposit_event(Event::TopUpCashDropPool(currency_id, amount));
		Ok(())
	}

	/// Issue CashDrop from the pool to the claimant account
	fn issue_cashdrop_from_pool(claimant_id: &T::AccountId, currency_id: Self::CurrencyId, amount: Balance) -> DispatchResult {
		Self::issue_standard(currency_id, &claimant_id, amount)?;
		let cashdrop_pool_balance = Self::cashdrop_pool(currency_id);
		let updated_cashdrop_pool_balance = cashdrop_pool_balance.saturating_sub(amount);
		// Update `CashDropCount`
		let cashdrop_count: u32 = 1 + <CashDropCount<T>>::get(currency_id);
		<CashDropCount<T>>::insert(currency_id, cashdrop_count);
		// Update `CashDropPool`
		<CashDropPool<T>>::insert(currency_id, updated_cashdrop_pool_balance);
		// Update `CashDrops` history
		<CashDrops<T>>::insert(currency_id, <CashDrops<T>>::get(currency_id) + amount);
		Self::deposit_event(Event::IssueCashDropFromPool(claimant_id.clone(), currency_id, amount));
		Ok(())
	}

	/// SerpUp ratio for cashDrop Cashdrops
	fn get_cashdrop_serpup(amount: Balance, currency_id: Self::CurrencyId) -> DispatchResult {
		// Add the SerpUp propper to CashDropPool
		Self::add_cashdrop_to_pool(currency_id, amount)?;

		Self::deposit_event(Event::SerpUpDelivery(amount, currency_id));
		Ok(())
	}

	/// Serplus ratio for BuyBack Swaps to burn Setter and Setheum (SETR:SETM)
	fn get_buyback_serplus(amount: Balance, currency_id: Self::CurrencyId) -> DispatchResult {
		// BuyBack Pool - 50%
		// Buyback with 20:20:20:20:20 with SETR:DNAR:SERP:SETM:HELP
		let one: Balance = 1;
		let amount_20percent: Balance = one.saturating_mul(amount.saturating_div(5));

		// calculate the target amount limit according to oracle price and the slippage limit,
		// if oracle price is not avalible, do not limit amount (get min_value)
		let setter_min_target_amount = if let Some(target_price) =
			T::PriceSource::get_relative_price(T::SetterCurrencyId::get(), currency_id)
		{
			Ratio::one()
				.saturating_sub(T::MaxSwapSlippageCompareToOracle::get())
				.reciprocal()
				.unwrap_or_else(Ratio::min_value)
				.saturating_mul_int(target_price.saturating_mul_int(amount_20percent))
		} else {
			CurrencyBalanceOf::<T>::min_value()
		};
		// calculate the target amount limit according to oracle price and the slippage limit,
		// if oracle price is not avalible, do not limit amount (get min_value)
		let dinar_min_target_amount = if let Some(target_price) =
			T::PriceSource::get_relative_price(T::GetDinarCurrencyId::get(), currency_id)
		{
			Ratio::one()
				.saturating_sub(T::MaxSwapSlippageCompareToOracle::get())
				.reciprocal()
				.unwrap_or_else(Ratio::min_value)
				.saturating_mul_int(target_price.saturating_mul_int(amount_20percent))
		} else {
			CurrencyBalanceOf::<T>::min_value()
		};
		// calculate the target amount limit according to oracle price and the slippage limit,
		// if oracle price is not avalible, do not limit amount (get min_value)
		let serp_min_target_amount = if let Some(target_price) =
			T::PriceSource::get_relative_price(T::GetSerpCurrencyId::get(), currency_id)
		{
			Ratio::one()
				.saturating_sub(T::MaxSwapSlippageCompareToOracle::get())
				.reciprocal()
				.unwrap_or_else(Ratio::min_value)
				.saturating_mul_int(target_price.saturating_mul_int(amount_20percent))
		} else {
			CurrencyBalanceOf::<T>::min_value()
		};
		// calculate the target amount limit according to oracle price and the slippage limit,
		// if oracle price is not avalible, do not limit amount (get min_value)
		let native_min_target_amount = if let Some(target_price) =
			T::PriceSource::get_relative_price(T::GetNativeCurrencyId::get(), currency_id)
		{
			Ratio::one()
				.saturating_sub(T::MaxSwapSlippageCompareToOracle::get())
				.reciprocal()
				.unwrap_or_else(Ratio::min_value)
				.saturating_mul_int(target_price.saturating_mul_int(amount_20percent))
		} else {
			CurrencyBalanceOf::<T>::min_value()
		};
		// calculate the target amount limit according to oracle price and the slippage limit,
		// if oracle price is not avalible, do not limit amount (get min_value)
		let help_min_target_amount = if let Some(target_price) =
			T::PriceSource::get_relative_price(T::GetHelpCurrencyId::get(), currency_id)
		{
			Ratio::one()
				.saturating_sub(T::MaxSwapSlippageCompareToOracle::get())
				.reciprocal()
				.unwrap_or_else(Ratio::min_value)
				.saturating_mul_int(target_price.saturating_mul_int(amount_20percent))
		} else {
			CurrencyBalanceOf::<T>::min_value()
		};

		<Self as SerpTreasuryExtended<T::AccountId>>::buyback_swap_with_exact_supply(
			currency_id,
			T::SetterCurrencyId::get(),
			SwapLimit::ExactSupply(amount_20percent, setter_min_target_amount),
		)?;
		<Self as SerpTreasuryExtended<T::AccountId>>::buyback_swap_with_exact_supply(
			currency_id,
			T::GetDinarCurrencyId::get(),
			SwapLimit::ExactSupply(amount_20percent, dinar_min_target_amount),
		)?;
		<Self as SerpTreasuryExtended<T::AccountId>>::buyback_swap_with_exact_supply(
			currency_id,
			T::GetSerpCurrencyId::get(),
			SwapLimit::ExactSupply(amount_20percent, serp_min_target_amount),
		)?;
		<Self as SerpTreasuryExtended<T::AccountId>>::buyback_swap_with_exact_supply(
			currency_id,
			T::GetNativeCurrencyId::get(),
			SwapLimit::ExactSupply(amount_20percent, native_min_target_amount),
		)?;
		<Self as SerpTreasuryExtended<T::AccountId>>::buyback_swap_with_exact_supply(
			currency_id,
			T::GetHelpCurrencyId::get(),
			SwapLimit::ExactSupply(amount_20percent, help_min_target_amount),
		)?;
		Ok(())
	}

	/// Serplus ratio for Setheum Foundation's Charity Fund
	fn get_cashdrop_serplus(amount: Balance, currency_id: Self::CurrencyId) -> DispatchResult {
		let cdp_treasury = T::CDPTreasuryAccountId::get();
		// Add the SerpUp propper to CashDropPool
		Self::add_cashdrop_to_pool(currency_id, amount)?;
		// Withdraw the Serplus propper from CDPTreasury
		T::Currency::withdraw(currency_id, &cdp_treasury, amount)?;
		Ok(())
	}

	/// issue system surplus(SETUSD) to their destinations according to the serpup_ratio.
	fn on_serplus(currency_id: CurrencyId, amount: Balance) -> DispatchResult {
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
		
		let get_buyback_serplus = Self::get_buyback_serplus(amount.saturating_div(2), currency_id).is_ok();
		let get_cashdrop_serplus = Self::get_cashdrop_serplus(amount.saturating_div(2), currency_id).is_ok();

		if get_buyback_serplus {
			Self::get_buyback_serplus(amount.saturating_div(2), currency_id).unwrap();	// 50%
		};
		
		if get_cashdrop_serplus {
			Self::get_cashdrop_serplus(amount.saturating_div(2), currency_id).unwrap();	// 50%
		}

		Self::deposit_event(Event::SerplusDelivery(amount, currency_id));
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

		let get_buyback_serpup = Self::get_buyback_serpup(amount.saturating_div(2), currency_id).is_ok();
		let get_cashdrop_serpup = Self::get_cashdrop_serpup(amount.saturating_div(2), currency_id).is_ok();

		if get_buyback_serpup {
			Self::get_buyback_serpup(amount.saturating_div(2), currency_id).unwrap();	// 50%
		};
		
		if get_cashdrop_serpup {
			Self::get_cashdrop_serpup(amount.saturating_div(2), currency_id).unwrap();	// 50%
		}

		Self::deposit_event(Event::SerpUp(amount, currency_id));
		Ok(())
	}

	// buy back and burn surplus(stable currencies) with swap on DEX
	// Create the necessary serp down parameters and swap currencies then burn swapped currencies.
	//
	// DNAR & SERP - BuyBack SETUSD with DNAR:SERP bilateral stability contribution
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

		// Serpdown 50:50 with DNAR:SERP
		let half = amount.saturating_div(2);

		// calculate the supply limit according to oracle price and the slippage limit,
		// if oracle price is not avalible, do not limit
		let dinar_max_supply_limit = if let Some(target_price) =
			T::PriceSource::get_relative_price(currency_id, T::GetDinarCurrencyId::get())
		{
			Ratio::one()
				.saturating_sub(T::MaxSwapSlippageCompareToOracle::get())
				.reciprocal()
				.unwrap_or_else(Ratio::max_value)
				.saturating_mul_int(target_price.saturating_mul_int(half))
		} else {
			CurrencyBalanceOf::<T>::max_value()
		};
		
		// calculate the supply limit according to oracle price and the slippage limit,
		// if oracle price is not avalible, do not limit
		let serp_max_supply_limit = if let Some(target_price) =
			T::PriceSource::get_relative_price(currency_id, T::GetSerpCurrencyId::get())
		{
			Ratio::one()
				.saturating_sub(T::MaxSwapSlippageCompareToOracle::get())
				.reciprocal()
				.unwrap_or_else(Ratio::max_value)
				.saturating_mul_int(target_price.saturating_mul_int(half))
		} else {
			CurrencyBalanceOf::<T>::max_value()
		};
		
		// serpdown with 50:50 with DNAR:SERP
		<Self as SerpTreasuryExtended<T::AccountId>>::buyback_swap_with_exact_target(
			T::GetDinarCurrencyId::get(),
			currency_id,
			SwapLimit::ExactTarget(dinar_max_supply_limit, half),
		)?;
		<Self as SerpTreasuryExtended<T::AccountId>>::buyback_swap_with_exact_target(
			T::GetSerpCurrencyId::get(),
			currency_id,
			SwapLimit::ExactTarget(serp_max_supply_limit, half),
		)?;

		Self::deposit_event(Event::SerpDown(amount, currency_id));
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
	fn claim_cashdrop(currency_id: CurrencyId, who: &T::AccountId, transfer_amount: Self::Balance) -> DispatchResult {
		ensure!(
			T::StableCurrencyIds::get().contains(&currency_id),
			Error::<T>::InvalidCurrencyType,
		);

		// IF Setter, use Setter claim rate (4%),
		// else, use SetDollar claim rate (2%).
		if currency_id == T::SetterCurrencyId::get() {
			let minimum_claimable_transfer = T::SetterMinimumClaimableTransferAmounts::get();
			let maximum_claimable_transfer = T::SetterMaximumClaimableTransferAmounts::get();

			if transfer_amount >= minimum_claimable_transfer && transfer_amount <= maximum_claimable_transfer {
				let balance_cashdrop_amount = transfer_amount.saturating_div(25); // 4% cashdrop claim reward
				let cashdrop_pool_balance = Self::cashdrop_pool(currency_id);
				if balance_cashdrop_amount <= cashdrop_pool_balance {
					// Issue the CashDrop claim from the CashDropPool
					Self::issue_cashdrop_from_pool(who, currency_id, balance_cashdrop_amount)?;
					
					Self::deposit_event(Event::CashDropClaim(currency_id, who.clone(), balance_cashdrop_amount.clone()));
				}
			} else {
				return Ok(())
			}
		} else {
			// for the SetDollar ---vvvvvvvvv---
			let minimum_claimable_transfer = T::SetDollarMinimumClaimableTransferAmounts::get();
			let maximum_claimable_transfer = T::SetDollarMaximumClaimableTransferAmounts::get();

			if transfer_amount >= minimum_claimable_transfer && transfer_amount <= maximum_claimable_transfer {
				let balance_cashdrop_amount = transfer_amount.saturating_div(50); // 2% cashdrop claim reward
				let cashdrop_pool_balance = Self::cashdrop_pool(currency_id);
				if balance_cashdrop_amount <= cashdrop_pool_balance {
					// Issue the CashDrop claim from the CashDropPool
					Self::issue_cashdrop_from_pool(who, currency_id, balance_cashdrop_amount)?;
					
					Self::deposit_event(Event::CashDropClaim(currency_id, who.clone(), balance_cashdrop_amount.clone()));
				}
			} else {
				return Ok(())
			}
		}

		Ok(())
	}
}

impl<T: Config> SerpTreasuryExtended<T::AccountId> for Pallet<T> {
	/// Swap `from_currency_id` to get exact `from_currency_id`,
	/// return actual supply Dinar amount
	#[allow(unused_variables)]
	fn buyback_swap_with_exact_supply(
		from_currency_id: CurrencyId,
		to_currency_id: CurrencyId,
		swap_limit: SwapLimit<Balance>,
	) -> sp_std::result::Result<(Balance, Balance), DispatchError> {
		let exact_supply = match swap_limit {
			SwapLimit::ExactSupply(exact_supply, _) => exact_supply,
			SwapLimit::ExactTarget(max_supply, target_amount) => max_supply,
		};
		let swap_path = T::Dex::get_best_price_swap_path(
			from_currency_id,
			to_currency_id,
			swap_limit,
			T::AlternativeSwapPathJointList::get(),
		)
		.ok_or(Error::<T>::CannotSwap)?;
		
		ensure!(
			T::Currency::deposit(
				from_currency_id,
				&Self::account_id(),
				exact_supply as Balance,
			).is_ok(),
			Error::<T>::CannotDeposit,
		);
		T::Currency::deposit(
			from_currency_id,
			&Self::account_id(),
			exact_supply as Balance,
		).unwrap();
		T::Dex::buyback_swap_with_specific_path(&Self::account_id(), &swap_path, swap_limit)
	}

	/// Swap `from_currency_id` to get exact `from_currency_id`,
	/// return actual supply Dinar amount
	#[allow(unused_variables)]
	fn buyback_swap_with_exact_target(
		from_currency_id: CurrencyId,
		to_currency_id: CurrencyId,
		swap_limit: SwapLimit<Balance>,
	) -> sp_std::result::Result<(Balance, Balance), DispatchError> {
		// let max_supply = match swap_limit {
		// 	SwapLimit::ExactSupply(supply_amount, min_target) => (supply_amount,min_target),
		// 	SwapLimit::ExactTarget(max_supply, exact_target) => (max_supply, exact_target),
		// };
		let swap_path = T::Dex::get_best_price_swap_path(
			from_currency_id,
			to_currency_id,
			swap_limit,
			T::AlternativeSwapPathJointList::get(),
		)
		.ok_or(Error::<T>::CannotSwap)?;
	
		let (supply_amount, _) = T::Dex::get_swap_amount(
			&swap_path,
			swap_limit
		)
		.ok_or(Error::<T>::DexNotAvailable)?;

		ensure!(
			T::Currency::deposit(
				from_currency_id,
				&Self::account_id(),
				supply_amount
			).is_ok(),
			Error::<T>::CannotDeposit,
		);
		
		T::Currency::deposit(
			from_currency_id,
			&Self::account_id(),
			supply_amount
		).unwrap();

		T::Dex::buyback_swap_with_specific_path(&Self::account_id(), &swap_path, swap_limit)
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
