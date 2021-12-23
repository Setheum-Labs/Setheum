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

//! # SERP Treasury Module
//!
//! ## Overview
//!
//! SERP Treasury manages the SERP, and handle excess serplus
//! and stabilize SetCurrencies standards timely in order to keep the
//! system healthy. It manages the TES (Token Elasticity of Supply).

#![cfg_attr(not(feature = "std"), no_std)]
#![allow(clippy::unused_unit)]

use frame_support::{pallet_prelude::*, transactional, PalletId};
use frame_system::pallet_prelude::*;
use orml_traits::{GetByKey, MultiCurrency, MultiCurrencyExtended};
use primitives::{Balance, CurrencyId, SerpStableCurrencyId};
use sp_core::U256;
use sp_runtime::{
	DispatchResult, 
	traits::{
		AccountIdConversion, Bounded, Saturating, UniqueSaturatedInto, One, Zero,
	},
	FixedPointNumber,
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

		#[pallet::constant]
		/// A duration period of inflation injection.
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

		/// Default fee swap path list
		type DefaultSwapParitalPathList: Get<Vec<Vec<CurrencyId>>>;

		/// When swap with DEX, the acceptable max slippage for the price from oracle.
		type MaxSwapSlippageCompareToOracle: Get<Ratio>;

		/// The limit for length of trading path
		#[pallet::constant]
		type TradingPathLimit: Get<u32>;

		/// The price source to provider external market price.
		type PriceSource: PriceProvider<CurrencyId>;

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
		/// SERP-TES is Triggered
		SerpTesNow(),
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
		/// SerpSwapExactStableToSerpToken
		SerpSwapExactStableToSerpToken(CurrencyId, CurrencyId, Balance, Balance),
		/// SerpSwapExactStableToNative
		TopUpCashDropPool(CurrencyId, Balance),
		/// SerpSwapExactStableToSerpToken
		IssueCashDropFromPool(T::AccountId, CurrencyId, Balance),
		/// Force SerpDown Risk Management
		ForceSerpDown(CurrencyId, Balance)
	}

	/// Mapping to Minimum Claimable Transfer.
	#[pallet::storage]
	#[pallet::getter(fn minimum_claimable_transfer)]
	pub type MinimumClaimableTransfer<T: Config> = StorageMap<_, Twox64Concat, CurrencyId, Balance, OptionQuery>;

	/// The CashDrop Pool
	///
	/// CashDropPool: map CurrencyId => Balance
	#[pallet::storage]
	#[pallet::getter(fn cashdrop_pool)]
	pub type CashDropPool<T: Config> = StorageMap<_, Twox64Concat, CurrencyId, Balance, ValueQuery>;

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
				// SERP TES (Token Elasticity of Supply).
				// Triggers Serping for all system stablecoins to stabilize stablecoin prices.
				let mut count: u32 = 0;
				if Self::issue_stablecurrency_inflation().is_ok() {
					count += 1;
				};

				if Self::serp_tes_now().is_ok() {
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
	
	/// Deliver System StableCurrency stability for the Setter - SETR
	fn serp_tes_now() -> DispatchResult {

		let setter_token = T::SetterCurrencyId::get();

		let setter_token: CurrencyId = setter_token.into();
		let setdollar_token: CurrencyId = setter_token.into();
		
		// TODO: Check if this matches with our initialized pool
		let (setter_pool, setdollar_pool) = T::Dex::get_liquidity_pool(setter_token, setdollar_token);

		let two: Balance = 2;

		let base_unit = setter_pool.saturating_mul(two);

		match setdollar_pool {
			0 => {}
			setdollar_pool if setdollar_pool > base_unit => {
				// safe from underflow because `setdollar_pool` is checked to be greater than `base_unit`
				let supply = T::Currency::total_issuance(setter_token);
				let expand_by = Self::calculate_supply_change(setdollar_pool, base_unit, supply);
				Self::on_serpup(setter_token, expand_by)?;
			}
			setdollar_pool if setdollar_pool < base_unit => {
				// safe from underflow because `setdollar_pool` is checked to be less than `base_unit`
				let supply = T::Currency::total_issuance(setter_token);
				let contract_by = Self::calculate_supply_change(base_unit, setdollar_pool, supply);
				Self::on_serpdown(setter_token, contract_by)?;
			}
			_ => {}
		}
		Self::deposit_event(Event::SerpTesNow());
		Ok(())
	}

	/// Deliver System StableCurrency Inflation
	fn issue_stablecurrency_inflation() -> DispatchResult {

		for currency_id in T::StableCurrencyIds::get() {
			// Amounts are 20% of the inflation rate amount for each distro.
			let one: Balance = 1;
			let inflation_amount = Self::stable_currency_inflation_rate(currency_id);
			let inflamounts: Balance = one.saturating_mul(inflation_amount / 4);

			// inflation distros
			// 1
			Self::add_cashdrop_to_pool(currency_id, inflamounts)?;
			// 2
			<Self as SerpTreasuryExtended<T::AccountId>>::swap_exact_setcurrency_to_dinar(
				currency_id,
				inflamounts,
			);
			// 3
			<Self as SerpTreasuryExtended<T::AccountId>>::swap_exact_setcurrency_to_serp(
				currency_id,
				inflamounts,
			);
			// 4
			<Self as SerpTreasuryExtended<T::AccountId>>::swap_exact_setcurrency_to_native(
				currency_id,
				inflamounts,
			);

			Self::deposit_event(Event::InflationDelivery(currency_id, inflation_amount));
		}
		Ok(())
	}

	/// SerpUp ratio for BuyBack Swaps to burn bought assets.
	fn get_buyback_serpup(amount: Balance, currency_id: Self::CurrencyId) -> DispatchResult {
		// Buyback with 50:50 with DNAR:SERP
		let one: Balance = 1;
		let amount_50percent: Balance = one.saturating_mul(amount / 2);
		
		
		<Self as SerpTreasuryExtended<T::AccountId>>::swap_exact_setcurrency_to_dinar(
			currency_id,
			amount_50percent,
		);
		<Self as SerpTreasuryExtended<T::AccountId>>::swap_exact_setcurrency_to_serp(
			currency_id,
			amount_50percent,
		);

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

		CashDropPool::<T>::insert(currency_id, updated_cashdrop_pool_balance);
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
		// Buyback with 25:25:25:25 with SETR:DNAR:SERP:SETM
		let one: Balance = 1;
		let amount_25percent: Balance = one.saturating_mul(amount / 4);
		
		<Self as SerpTreasuryExtended<T::AccountId>>::serplus_swap_exact_setcurrency_to_setter(
			currency_id,
			amount_25percent,
		);
		<Self as SerpTreasuryExtended<T::AccountId>>::swap_exact_setcurrency_to_dinar(
			currency_id,
			amount_25percent,
		);
		<Self as SerpTreasuryExtended<T::AccountId>>::swap_exact_setcurrency_to_serp(
			currency_id,
			amount_25percent,
		);
		<Self as SerpTreasuryExtended<T::AccountId>>::serplus_swap_exact_setcurrency_to_native(
			currency_id,
			amount_25percent,
		);
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
		
		Self::get_buyback_serplus(amount / 2, currency_id).unwrap();	// 50%
		Self::get_cashdrop_serplus(amount / 2, currency_id).unwrap();	// 50%

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
		Self::get_buyback_serpup(amount / 2, currency_id).unwrap();		// 50%
		Self::get_cashdrop_serpup(amount / 2, currency_id).unwrap();	// 50%

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
		let two: Balance = 2;
		let half = amount / two;

		// serpdown with 50:50 with DNAR:SERP
		<Self as SerpTreasuryExtended<T::AccountId>>::swap_dinar_to_exact_setcurrency(
			currency_id,
			half,
		);
		<Self as SerpTreasuryExtended<T::AccountId>>::swap_serp_to_exact_setcurrency(
			currency_id,
			half,
		);

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
				let balance_cashdrop_amount = transfer_amount / 25; // 4% cashdrop claim reward
				let buyback_cashdrop_amount = transfer_amount / 25; // 4% cashdrop serp buyback
				let cashdrop_pool_reward = transfer_amount / 50; // 2% cashdrop pool topup reward
				let cashdrop_pool_balance = Self::cashdrop_pool(currency_id);
				if balance_cashdrop_amount <= cashdrop_pool_balance {
					// Issue the CashDrop claim from the CashDropPool
					Self::issue_cashdrop_from_pool(who, currency_id, balance_cashdrop_amount)?;
					// Add the system CashDrop reward to the CashDropPool
					Self::add_cashdrop_to_pool(currency_id, cashdrop_pool_reward)?;
		
					// buyback for 50:50 of DNAR:SERP
					<Self as SerpTreasuryExtended<T::AccountId>>::swap_exact_setcurrency_to_dinar(
						currency_id,
						buyback_cashdrop_amount / 2,
					);
					// 4
					<Self as SerpTreasuryExtended<T::AccountId>>::swap_exact_setcurrency_to_serp(
						currency_id,
						buyback_cashdrop_amount / 2,
					);
		
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
				let balance_cashdrop_amount = transfer_amount / 50; // 2% cashdrop claim reward
				let buyback_cashdrop_amount = transfer_amount / 14; // 7.14285714% cashdrop serp buyback
				let cashdrop_pool_reward = transfer_amount / 100; // 1% cashdrop pool topup reward
				let cashdrop_pool_balance = Self::cashdrop_pool(currency_id);
				if balance_cashdrop_amount <= cashdrop_pool_balance {
					// Issue the CashDrop claim from the CashDropPool
					Self::issue_cashdrop_from_pool(who, currency_id, balance_cashdrop_amount)?;
					// Add the system CashDrop reward to the CashDropPool
					Self::add_cashdrop_to_pool(currency_id, cashdrop_pool_reward)?;
		
					// buyback for 50:50 of DNAR:SERP
					<Self as SerpTreasuryExtended<T::AccountId>>::swap_exact_setcurrency_to_dinar(
						currency_id,
						buyback_cashdrop_amount / 2,
					);
					// 4
					<Self as SerpTreasuryExtended<T::AccountId>>::swap_exact_setcurrency_to_serp(
						currency_id,
						buyback_cashdrop_amount / 2,
					);
		
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
	/// swap Dinar to get exact Setter,
	/// return actual supply Dinar amount
	#[allow(unused_variables)]
	fn swap_dinar_to_exact_setter(
		target_amount: Balance,
	) {
		let dinar_currency_id = T::GetDinarCurrencyId::get();
		let setter_currency_id = T::SetterCurrencyId::get();

		let default_swap_parital_path_list: Vec<Vec<CurrencyId>> = T::DefaultSwapParitalPathList::get();

		// calculate the supply limit according to oracle price and the slippage limit,
		// if oracle price is not avalible, do not limit
		let max_supply_limit = if let Some(target_price) =
			T::PriceSource::get_relative_price(setter_currency_id, dinar_currency_id)
		{
			Ratio::one()
				.saturating_sub(T::MaxSwapSlippageCompareToOracle::get())
				.reciprocal()
				.unwrap_or_else(Ratio::max_value)
				.saturating_mul_int(target_price.saturating_mul_int(target_amount))
		} else {
			CurrencyBalanceOf::<T>::max_value()
		};
		
		// iterate default_swap_parital_path_list to try swap until swap succeeds.
		for partial_path in default_swap_parital_path_list {
			let partial_path_len = partial_path.len();

			// check currency_id and partial_path can form a valid swap path.
			if partial_path_len > 0 && dinar_currency_id != partial_path[0] {
				let mut swap_path = vec![dinar_currency_id, setter_currency_id];
				swap_path.extend(partial_path);

				if T::Currency::deposit(
					dinar_currency_id,
					&Self::account_id(),
					max_supply_limit.unique_saturated_into()
				).is_ok() && T::Dex::buyback_swap_with_exact_target(
						&Self::account_id(),
						&swap_path,
						target_amount.unique_saturated_into(),
				)
				.is_ok()
				{
					// successfully swap, break iteration
					break;
				}
			}
		}
		Self::deposit_event(Event::SerpSwapDinarToExactSetter(dinar_currency_id, setter_currency_id, max_supply_limit, target_amount));
	}

	/// swap Serp to get exact Setter,
	/// return actual supply Serp amount
	#[allow(unused_variables)]
	fn swap_serp_to_exact_setter(
		target_amount: Balance,
	) {
		let serptoken_currency_id = T::GetSerpCurrencyId::get();
		let setter_currency_id = T::SetterCurrencyId::get();
		
		let default_swap_parital_path_list: Vec<Vec<CurrencyId>> = T::DefaultSwapParitalPathList::get();

		// calculate the supply limit according to oracle price and the slippage limit,
		// if oracle price is not avalible, do not limit
		let max_supply_limit = if let Some(target_price) =
			T::PriceSource::get_relative_price(setter_currency_id, serptoken_currency_id)
		{
			Ratio::one()
				.saturating_sub(T::MaxSwapSlippageCompareToOracle::get())
				.reciprocal()
				.unwrap_or_else(Ratio::max_value)
				.saturating_mul_int(target_price.saturating_mul_int(target_amount))
		} else {
			CurrencyBalanceOf::<T>::max_value()
		};
		
		// iterate default_swap_parital_path_list to try swap until swap succeeds.
		for partial_path in default_swap_parital_path_list {
			let partial_path_len = partial_path.len();

			// check currency_id and partial_path can form a valid swap path.
			if partial_path_len > 0 && serptoken_currency_id != partial_path[0] {
				let mut swap_path = vec![serptoken_currency_id, setter_currency_id];
				swap_path.extend(partial_path);

				if T::Currency::deposit(
					serptoken_currency_id,
					&Self::account_id(),
					max_supply_limit.unique_saturated_into()
				).is_ok() && T::Dex::buyback_swap_with_exact_target(
						&Self::account_id(),
						&swap_path,
						target_amount.unique_saturated_into(),
				)
				.is_ok()
				{
					// successfully swap, break iteration
					break;
				}
			}
		}
		Self::deposit_event(Event::SerpSwapSerpToExactSetter(serptoken_currency_id, setter_currency_id, max_supply_limit, target_amount));
	}

	/// swap Dinar to get exact Setter,
	/// return actual supply Dinar amount
	#[allow(unused_variables)]
	fn swap_dinar_to_exact_setcurrency(
		currency_id: CurrencyId,
		target_amount: Balance,
	) {
		let dinar_currency_id = T::GetDinarCurrencyId::get();
		
		let default_swap_parital_path_list: Vec<Vec<CurrencyId>> = T::DefaultSwapParitalPathList::get();

		// calculate the supply limit according to oracle price and the slippage limit,
		// if oracle price is not avalible, do not limit
		let max_supply_limit = if let Some(target_price) =
			T::PriceSource::get_relative_price(currency_id, dinar_currency_id)
		{
			Ratio::one()
				.saturating_sub(T::MaxSwapSlippageCompareToOracle::get())
				.reciprocal()
				.unwrap_or_else(Ratio::max_value)
				.saturating_mul_int(target_price.saturating_mul_int(target_amount))
		} else {
			CurrencyBalanceOf::<T>::max_value()
		};
		
		// iterate default_swap_parital_path_list to try swap until swap succeeds.
		for partial_path in default_swap_parital_path_list {
			let partial_path_len = partial_path.len();

			// check currency_id and partial_path can form a valid swap path.
			if partial_path_len > 0 && dinar_currency_id != partial_path[0] {
				let mut swap_path = vec![dinar_currency_id, currency_id];
				swap_path.extend(partial_path);

				if T::Currency::deposit(
					dinar_currency_id,
					&Self::account_id(),
					max_supply_limit.unique_saturated_into()
				).is_ok() && T::Dex::buyback_swap_with_exact_target(
						&Self::account_id(),
						&swap_path,
						target_amount.unique_saturated_into(),
				)
				.is_ok()
				{
					// successfully swap, break iteration
					break;
				}
			}
		}
		Self::deposit_event(Event::SerpSwapDinarToExactStable(dinar_currency_id, currency_id, max_supply_limit, target_amount));
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
		let setter_currency_id = T::SetterCurrencyId::get();

		let default_swap_parital_path_list: Vec<Vec<CurrencyId>> = T::DefaultSwapParitalPathList::get();

		// calculate the supply limit according to oracle price and the slippage limit,
		// if oracle price is not avalible, do not limit
		let max_supply_limit = if let Some(target_price) =
			T::PriceSource::get_relative_price(currency_id, setter_currency_id)
		{
			Ratio::one()
				.saturating_sub(T::MaxSwapSlippageCompareToOracle::get())
				.reciprocal()
				.unwrap_or_else(Ratio::max_value)
				.saturating_mul_int(target_price.saturating_mul_int(target_amount))
		} else {
			CurrencyBalanceOf::<T>::max_value()
		};
		
		// iterate default_swap_parital_path_list to try swap until swap succeeds.
		for partial_path in default_swap_parital_path_list {
			let partial_path_len = partial_path.len();

			// check currency_id and partial_path can form a valid swap path.
			if partial_path_len > 0 && setter_currency_id != partial_path[0] {
				let mut swap_path = vec![setter_currency_id, currency_id];
				swap_path.extend(partial_path);

				if T::Currency::deposit(
					setter_currency_id,
					&Self::account_id(),
					max_supply_limit.unique_saturated_into()
				).is_ok() && T::Dex::buyback_swap_with_exact_target(
						&Self::account_id(),
						&swap_path,
						target_amount.unique_saturated_into(),
				)
				.is_ok()
				{
					// successfully swap, break iteration
					break;
				}
			}
		}
		Self::deposit_event(Event::SerpSwapSetterToExactSetDollar(setter_currency_id, currency_id, max_supply_limit, target_amount));
	}

	/// Swap exact amount of Serp currrency to SetCurrency,
	/// return actual target SetCurrency amount
	///
	/// 
	/// When SetCurrency needs SerpDown
	/// 
	#[allow(unused_variables)]
	fn swap_serp_to_exact_setcurrency(
		currency_id: CurrencyId,
		target_amount: Balance,
	) {
		let serp_currency_id  = T::GetSerpCurrencyId::get();

		let default_swap_parital_path_list: Vec<Vec<CurrencyId>> = T::DefaultSwapParitalPathList::get();

		// calculate the supply limit according to oracle price and the slippage limit,
		// if oracle price is not avalible, do not limit
		let max_supply_limit = if let Some(target_price) =
			T::PriceSource::get_relative_price(currency_id, serp_currency_id)
		{
			Ratio::one()
				.saturating_sub(T::MaxSwapSlippageCompareToOracle::get())
				.reciprocal()
				.unwrap_or_else(Ratio::max_value)
				.saturating_mul_int(target_price.saturating_mul_int(target_amount))
		} else {
			CurrencyBalanceOf::<T>::max_value()
		};
		
		// iterate default_swap_parital_path_list to try swap until swap succeeds.
		for partial_path in default_swap_parital_path_list {
			let partial_path_len = partial_path.len();

			// check currency_id and partial_path can form a valid swap path.
			if partial_path_len > 0 && serp_currency_id != partial_path[0] {
				let mut swap_path = vec![serp_currency_id, currency_id];
				swap_path.extend(partial_path);

				if T::Currency::deposit(
					serp_currency_id,
					&Self::account_id(),
					max_supply_limit.unique_saturated_into()
				).is_ok() && T::Dex::buyback_swap_with_exact_target(
						&Self::account_id(),
						&swap_path,
						target_amount.unique_saturated_into(),
				)
				.is_ok()
				{
					// successfully swap, break iteration
					break;
				}
			}
		}
		Self::deposit_event(Event::SerpSwapSerpToExactStable(serp_currency_id, currency_id, max_supply_limit, target_amount));
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
		let dinar_currency_id = T::GetDinarCurrencyId::get();
		let currency_id = T::SetterCurrencyId::get();

		let default_swap_parital_path_list: Vec<Vec<CurrencyId>> = T::DefaultSwapParitalPathList::get();
		
		// calculate the target limit according to oracle price and the slippage limit,
		// if oracle price is not avalible, do not limit
		let min_target_limit = if let Some(target_price) =
			T::PriceSource::get_relative_price(dinar_currency_id, currency_id)
		{
			Ratio::one()
				.saturating_sub(T::MaxSwapSlippageCompareToOracle::get())
				.reciprocal()
				.unwrap_or_else(Ratio::min_value)
				.saturating_mul_int(target_price.saturating_mul_int(supply_amount))
		} else {
			CurrencyBalanceOf::<T>::min_value()
		};

		// iterate default_swap_parital_path_list to try swap until swap succeeds.
		for partial_path in default_swap_parital_path_list {
			let partial_path_len = partial_path.len();

			// check currency_id and partial_path can form a valid swap path.
			if partial_path_len > 0 && currency_id != partial_path[0] {
				let mut swap_path = vec![currency_id, dinar_currency_id];
				swap_path.extend(partial_path);

				if T::Currency::deposit(
					currency_id,
					&Self::account_id(),
					supply_amount.unique_saturated_into()
				).is_ok() && T::Dex::buyback_swap_with_exact_supply(
					&Self::account_id(),
					&swap_path,
					supply_amount.unique_saturated_into(),
					// min_target_limit.unique_saturated_into(),
				)
				.is_ok()
				{
					// successfully swap, break iteration.
					break;
				}
			}
		}
		Self::deposit_event(Event::SerpSwapExactStableToDinar(currency_id, dinar_currency_id, min_target_limit, supply_amount));
	}

	/// Swap exact amount of SetCurrency to Setter,
	/// return actual supply SetCurrency amount
	///
	/// 
	/// When SetCurrency gets inflation deposit
	#[allow(unused_variables)]
	fn swap_exact_setcurrency_to_setter(
		currency_id: CurrencyId,
		supply_amount: Balance,
	) {
		let setter_currency_id = T::SetterCurrencyId::get();

		let default_swap_parital_path_list: Vec<Vec<CurrencyId>> = T::DefaultSwapParitalPathList::get();
		
		// calculate the target limit according to oracle price and the slippage limit,
		// if oracle price is not avalible, do not limit
		let min_target_limit = if let Some(target_price) =
			T::PriceSource::get_relative_price(setter_currency_id, currency_id)
		{
			Ratio::one()
				.saturating_sub(T::MaxSwapSlippageCompareToOracle::get())
				.reciprocal()
				.unwrap_or_else(Ratio::min_value)
				.saturating_mul_int(target_price.saturating_mul_int(supply_amount))
		} else {
			CurrencyBalanceOf::<T>::min_value()
		};

		// iterate default_swap_parital_path_list to try swap until swap succeeds.
		for partial_path in default_swap_parital_path_list {
			let partial_path_len = partial_path.len();

			// check currency_id and partial_path can form a valid swap path.
			if partial_path_len > 0 && currency_id != partial_path[0] {
				let mut swap_path = vec![currency_id, setter_currency_id];
				swap_path.extend(partial_path);

				if T::Currency::deposit(
					currency_id,
					&Self::account_id(),
					supply_amount.unique_saturated_into()
				).is_ok() && T::Dex::buyback_swap_with_exact_supply(
					&Self::account_id(),
					&swap_path,
					supply_amount.unique_saturated_into(),
					// min_target_limit.unique_saturated_into(),
				)
				.is_ok()
				{
					// successfully swap, break iteration.
					break;
				}
			}
		}
		Self::deposit_event(Event::SerpSwapExactStableToSetter(currency_id, setter_currency_id, min_target_limit, supply_amount));
	}

	/// Swap exact amount of SetCurrency to Setter,
	/// return actual supply SetCurrency amount
	///
	/// 
	/// When SetCurrency gets serplus deposit
	#[allow(unused_variables)]
	fn serplus_swap_exact_setcurrency_to_setter(
		currency_id: CurrencyId,
		supply_amount: Balance,
	) {
		let setter_currency_id = T::SetterCurrencyId::get();

		let default_swap_parital_path_list: Vec<Vec<CurrencyId>> = T::DefaultSwapParitalPathList::get();
		
		// calculate the target limit according to oracle price and the slippage limit,
		// if oracle price is not avalible, do not limit
		let min_target_limit = if let Some(target_price) =
			T::PriceSource::get_relative_price(setter_currency_id, currency_id)
		{
			Ratio::one()
				.saturating_sub(T::MaxSwapSlippageCompareToOracle::get())
				.reciprocal()
				.unwrap_or_else(Ratio::min_value)
				.saturating_mul_int(target_price.saturating_mul_int(supply_amount))
		} else {
			CurrencyBalanceOf::<T>::min_value()
		};

		// iterate default_swap_parital_path_list to try swap until swap succeeds.
		for partial_path in default_swap_parital_path_list {
			let partial_path_len = partial_path.len();

			// check currency_id and partial_path can form a valid swap path.
			if partial_path_len > 0 && currency_id != partial_path[0] {
				let mut swap_path = vec![currency_id, setter_currency_id];
				swap_path.extend(partial_path);

				if T::Currency::deposit(
					currency_id,
					&Self::account_id(),
					supply_amount.unique_saturated_into()
				).is_ok() && T::Dex::buyback_swap_with_exact_supply(
					&Self::account_id(),
					&swap_path,
					supply_amount.unique_saturated_into(),
					// min_target_limit.unique_saturated_into(),
				)
				.is_ok()
				{
					// successfully swap, break iteration.
					break;
				}
			}
		}
		Self::deposit_event(Event::SerplusSwapExactStableToSetter(currency_id, setter_currency_id, min_target_limit, supply_amount));
	}

	/// Swap exact amount of SetCurrency to Setter,
	/// return actual supply SetCurrency amount
	///
	/// 
	/// When SetCurrency gets serplus deposit
	#[allow(unused_variables)]
	fn serplus_swap_exact_setcurrency_to_native(
		currency_id: CurrencyId,
		supply_amount: Balance,
	) {
		let setm_currency_id = T::GetNativeCurrencyId::get();

		let default_swap_parital_path_list: Vec<Vec<CurrencyId>> = T::DefaultSwapParitalPathList::get();
		
		// calculate the target limit according to oracle price and the slippage limit,
		// if oracle price is not avalible, do not limit
		let min_target_limit = if let Some(target_price) =
			T::PriceSource::get_relative_price(setm_currency_id, currency_id)
		{
			Ratio::one()
				.saturating_sub(T::MaxSwapSlippageCompareToOracle::get())
				.reciprocal()
				.unwrap_or_else(Ratio::min_value)
				.saturating_mul_int(target_price.saturating_mul_int(supply_amount))
		} else {
			CurrencyBalanceOf::<T>::min_value()
		};

		// iterate default_swap_parital_path_list to try swap until swap succeeds.
		for partial_path in default_swap_parital_path_list {
			let partial_path_len = partial_path.len();

			// check currency_id and partial_path can form a valid swap path.
			if partial_path_len > 0 && currency_id != partial_path[0] {
				let mut swap_path = vec![currency_id, setm_currency_id];
				swap_path.extend(partial_path);

				if T::Currency::deposit(
					currency_id,
					&Self::account_id(),
					supply_amount.unique_saturated_into()
				).is_ok() && T::Dex::buyback_swap_with_exact_supply(
					&Self::account_id(),
					&swap_path,
					supply_amount.unique_saturated_into(),
					// min_target_limit.unique_saturated_into(),
				)
				.is_ok()
				{
					// successfully swap, break iteration.
					break;
				}
			}
		}
		Self::deposit_event(Event::SerplusSwapExactStableToNative(currency_id, setm_currency_id, min_target_limit, supply_amount));
	}

	/// Swap exact amount of SetCurrency to Setheum,
	/// return actual supply SetCurrency amount
	///
	/// 
	/// When SetCurrency gets inflation deposit
	#[allow(unused_variables)]
	fn swap_exact_setcurrency_to_native(
		currency_id: CurrencyId,
		supply_amount: Balance,
	) {
		let native_currency_id = T::GetNativeCurrencyId::get();

		let default_swap_parital_path_list: Vec<Vec<CurrencyId>> = T::DefaultSwapParitalPathList::get();
		
		// calculate the target limit according to oracle price and the slippage limit,
		// if oracle price is not avalible, do not limit
		let min_target_limit = if let Some(target_price) =
			T::PriceSource::get_relative_price(native_currency_id, currency_id)
		{
			Ratio::one()
				.saturating_sub(T::MaxSwapSlippageCompareToOracle::get())
				.reciprocal()
				.unwrap_or_else(Ratio::min_value)
				.saturating_mul_int(target_price.saturating_mul_int(supply_amount))
		} else {
			CurrencyBalanceOf::<T>::min_value()
		};

		// iterate default_swap_parital_path_list to try swap until swap succeeds.
		for partial_path in default_swap_parital_path_list {
			let partial_path_len = partial_path.len();

			// check currency_id and partial_path can form a valid swap path.
			if partial_path_len > 0 && currency_id != partial_path[0] {
				let mut swap_path = vec![currency_id, native_currency_id];
				swap_path.extend(partial_path);

				if T::Currency::deposit(
					currency_id,
					&Self::account_id(),
					supply_amount.unique_saturated_into()
				).is_ok() && T::Dex::buyback_swap_with_exact_supply(
					&Self::account_id(),
					&swap_path,
					supply_amount.unique_saturated_into(),
					// min_target_limit.unique_saturated_into(),
				)
				.is_ok()
				{
					// successfully swap, break iteration.
					break;
				}
			}
		}
		Self::deposit_event(Event::SerpSwapExactStableToNative(currency_id, native_currency_id, min_target_limit, supply_amount));
	}
	
	/// Swap exact amount of Setter to Serp,
	/// return actual supply Setter amount
	///
	/// 
	/// When Setter gets SerpUp
	#[allow(unused_variables)]
	fn swap_exact_setcurrency_to_serp(
		currency_id: CurrencyId,
		supply_amount: Balance,
	) {
		let serptoken_currency_id = T::GetSerpCurrencyId::get(); 

		let default_swap_parital_path_list: Vec<Vec<CurrencyId>> = T::DefaultSwapParitalPathList::get();
		
		// calculate the target limit according to oracle price and the slippage limit,
		// if oracle price is not avalible, do not limit
		let min_target_limit = if let Some(target_price) =
			T::PriceSource::get_relative_price(serptoken_currency_id, currency_id)
		{
			Ratio::one()
				.saturating_sub(T::MaxSwapSlippageCompareToOracle::get())
				.reciprocal()
				.unwrap_or_else(Ratio::min_value)
				.saturating_mul_int(target_price.saturating_mul_int(supply_amount))
		} else {
			CurrencyBalanceOf::<T>::min_value()
		};

		// iterate default_swap_parital_path_list to try swap until swap succeeds.
		for partial_path in default_swap_parital_path_list {
			let partial_path_len = partial_path.len();

			// check currency_id and partial_path can form a valid swap path.
			if partial_path_len > 0 && currency_id != partial_path[0] {
				let mut swap_path = vec![currency_id, serptoken_currency_id];
				swap_path.extend(partial_path);

				if T::Currency::deposit(
					currency_id,
					&Self::account_id(),
					supply_amount.unique_saturated_into()
				).is_ok() && T::Dex::buyback_swap_with_exact_supply(
					&Self::account_id(),
					&swap_path,
					supply_amount.unique_saturated_into(),
					// min_target_limit.unique_saturated_into(),
				)
				.is_ok()
				{
					// successfully swap, break iteration.
					break;
				}
			}
		}
		Self::deposit_event(Event::SerpSwapExactStableToSerpToken(currency_id, serptoken_currency_id, min_target_limit, supply_amount));
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
