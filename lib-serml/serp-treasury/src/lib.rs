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

use frame_support::{pallet_prelude::*, transactional};
use frame_system::pallet_prelude::*;
use orml_traits::{GetByKey, MultiCurrency, MultiCurrencyExtended};
use primitives::{Balance, CurrencyId};
use sp_runtime::{
	DispatchResult, 
	traits::{
		AccountIdConversion, Bounded, Saturating, UniqueSaturatedInto, Zero,
	},
	FixedPointNumber, ModuleId,
};
use sp_std::{prelude::*, vec};
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
		type GetSetUSDCurrencyId: Get<CurrencyId>;

		// CashDrop period for transferring cashdrop.
		// The ideal period is after every `24 hours`.
		// type CashDropPeriod: Get<Self::BlockNumber>;

		/// The vault account to keep the Cashdrops for claiming.
		#[pallet::constant]
		type CashDropPoolAccountId: Get<Self::AccountId>;

		/// SerpUp pool/account for receiving funds Setheum Foundation's Charity Fund
		/// PublicFund account.
		#[pallet::constant]
		type PublicFundAccountId: Get<Self::AccountId>;

		/// CDP-Treasury account for processing serplus funds 
		/// CDPTreasury account.
		#[pallet::constant]
		type CDPTreasuryAccountId: Get<Self::AccountId>;

		/// SerpUp pool/account for receiving funds Setheum Foundation's Charity Fund
		/// SetheumTreasury account.
		#[pallet::constant]
		type SetheumTreasuryAccountId: Get<Self::AccountId>;

		/// Default fee swap path list
		type DefaultSwapPathList: Get<Vec<Vec<CurrencyId>>>;

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

		/// The origin which may update incentive related params
		type UpdateOrigin: EnsureOrigin<Self::Origin>;

		#[pallet::constant]
		/// The SERP Treasury's module id, keeps serplus and reserve asset.
		type ModuleId: Get<ModuleId>;

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
		/// Transfer is too low or too high for CashDrop.
		TransferNotEligibleForCashDrop,
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
		StableCurrencyInflationRateUpdated(CurrencyId, Balance),
		/// Stable Currency Inflation Rate Delivered
		InflationDelivery(CurrencyId, Balance),
	}

	/// Mapping to Minimum Claimable Transfer.
	#[pallet::storage]
	#[pallet::getter(fn minimum_claimable_transfer)]
	pub type MinimumClaimableTransfer<T: Config> = StorageMap<_, Twox64Concat, CurrencyId, Balance, OptionQuery>;

	/// The alternative fee swap path of accounts.
	#[pallet::storage]
	#[pallet::getter(fn alternative_fee_swap_path)]
	pub type AlternativeSwapPath<T: Config> =
		StorageMap<_, Twox64Concat, T::AccountId, Vec<CurrencyId>, OptionQuery>;

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
	}

	#[cfg(feature = "std")]
	impl Default for GenesisConfig {
		fn default() -> Self {
			GenesisConfig {
				stable_currency_inflation_rate: vec![],
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
			// SERP-TES Adjustment Frequency.
			// Schedule for when to trigger SERP-TES
			// (Blocktime/BlockNumber - every blabla block)
			if now % T::StableCurrencyInflationPeriod::get() == Zero::zero() {
				// SERP TES (Token Elasticity of Supply).
				// Triggers Serping for all system stablecoins to stabilize stablecoin prices.
				let mut count: u32 = 0;
				if Self::issue_stablecurrency_inflation().is_ok() {
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
			currency_id: CurrencyId,
			size: Balance,
		) -> DispatchResultWithPostInfo {
			T::UpdateOrigin::ensure_origin(origin)?;
			StableCurrencyInflationRate::<T>::insert(currency_id, size);
			Self::deposit_event(Event::StableCurrencyInflationRateUpdated(currency_id, size));
			Ok(().into())
		}
	}
}

impl<T: Config> Pallet<T> {
	/// Get account of SERP Treasury module.
	pub fn account_id() -> T::AccountId {
		T::ModuleId::get().into_account()
	}
}

impl<T: Config> SerpTreasury<T::AccountId> for Pallet<T> {
	type Balance = Balance;
	type CurrencyId = CurrencyId;

	/// Deliver System StableCurrency Inflation
	fn issue_stablecurrency_inflation() -> DispatchResult {

		// the inflation receiving accounts.
		let cashdrop_account = &T::CashDropPoolAccountId::get();
		let public_fund_account = T::PublicFundAccountId::get();
		let treasury_account = T::SetheumTreasuryAccountId::get();

		for currency_id in T::StableCurrencyIds::get() {

			// the inflation rate amount.
			let inflation_amount = Self::stable_currency_inflation_rate(currency_id);

			// IF Setter, Setter distribution allocations,
			// else, SetCurrency distribution allocations.
			if currency_id == T::SetterCurrencyId::get() {
				// CashDrop Pool Distribution - 40%
				let four: Balance = 4;
				let cashdrop_amount: Balance = four.saturating_mul(inflation_amount / 10);
				// Deposit inflation
				T::Currency::deposit(currency_id, &cashdrop_account, cashdrop_amount)?;
		
				// DNAR - BuyBack Pool Distribution - 30%
				let three: Balance = 3;
				let dinar_buyback_amount: Balance = three.saturating_mul(inflation_amount / 10);
				<Self as SerpTreasuryExtended<T::AccountId>>::swap_exact_setter_to_dinar(
					dinar_buyback_amount,
				);

				// PublicFund Distribution - 20%
				let two: Balance = 2;
				let public_fund_amount: Balance = two.saturating_mul(inflation_amount / 10);
				// Deposit inflation
				T::Currency::deposit(currency_id, &public_fund_account, public_fund_amount)?;
		
				// Setheum Treasury Distribution - 10%
				let one: Balance = 1;
				let treasury_amount: Balance = one.saturating_mul(inflation_amount / 10);
				// Deposit inflation
				T::Currency::deposit(currency_id, &treasury_account, treasury_amount)?;
			} else {
				// CashDrop Pool Distribution - 40%
				let four: Balance = 4;
				let cashdrop_amount: Balance = four.saturating_mul(inflation_amount / 10);
				// Deposit inflation
				T::Currency::deposit(currency_id, &cashdrop_account, cashdrop_amount)?;
		
				// DNAR - BuyBack Pool Distribution - 20%
				let two: Balance = 2;
				let dinar_buyback_amount: Balance = two.saturating_mul(inflation_amount / 10);
				<Self as SerpTreasuryExtended<T::AccountId>>::swap_exact_setcurrency_to_dinar(
					currency_id,
					dinar_buyback_amount,
				);

				// SETR - BuyBack Pool Distribution - 20%
				let setter_buyback_amount: Balance = two.saturating_mul(inflation_amount / 10);
				<Self as SerpTreasuryExtended<T::AccountId>>::swap_exact_setcurrency_to_setter(
					currency_id,
					setter_buyback_amount,
				);

				// PublicFund Distribution - 10%
				let one: Balance = 1;
				let public_fund_amount: Balance = one.saturating_mul(inflation_amount / 10);
				// Deposit inflation
				T::Currency::deposit(currency_id, &public_fund_account, public_fund_amount)?;
		
				// Setheum Treasury Distribution - 10%
				let treasury_amount: Balance = one.saturating_mul(inflation_amount / 20);
				// Deposit inflation
				T::Currency::deposit(currency_id, &treasury_account, treasury_amount)?;

				// SETM - BuyBack Pool Distribution - 5%
				let setheum_buyback_amount: Balance = one.saturating_mul(inflation_amount / 20);
				<Self as SerpTreasuryExtended<T::AccountId>>::swap_exact_setcurrency_to_native(
					currency_id,
					setheum_buyback_amount,
				);
			}

			Self::deposit_event(Event::InflationDelivery(currency_id, inflation_amount));
		}
		Ok(())
	}

	/// SerpUp ratio for BuyBack Swaps to burn bought assets.
	fn get_buyback_serpup(amount: Balance, currency_id: Self::CurrencyId) -> DispatchResult {
		// BuyBack Pool - 40%
		//
		if currency_id == T::SetterCurrencyId::get() {
			// Buyback with 50:50 with DNAR:SERP
			let two: Balance = 2;
			let serping_amount_20percent: Balance = two.saturating_mul(amount / 10);
			
			<Self as SerpTreasuryExtended<T::AccountId>>::swap_exact_setter_to_dinar(
				serping_amount_20percent,
			);
			<Self as SerpTreasuryExtended<T::AccountId>>::swap_exact_setter_to_serp(
				serping_amount_20percent,
			);
		} else {
			// Buyback with 25:25:50 with DNAR:SERP:SETR
			let one: Balance = 1;
			let two: Balance = 2;
			let serping_amount_10percent: Balance = one.saturating_mul(amount / 10);
			let serping_amount_20percent: Balance = two.saturating_mul(amount / 10);
			
			
			<Self as SerpTreasuryExtended<T::AccountId>>::swap_exact_setcurrency_to_dinar(
				currency_id,
				serping_amount_10percent,
			);
			<Self as SerpTreasuryExtended<T::AccountId>>::swap_exact_setcurrency_to_serp(
				currency_id,
				serping_amount_10percent,
			);
			<Self as SerpTreasuryExtended<T::AccountId>>::swap_exact_setcurrency_to_setter(
				currency_id,
				serping_amount_20percent,
			);
		} 

		Self::deposit_event(Event::SerpUpDelivery(amount, currency_id));
		Ok(())
	}

	/// SerpUp ratio for Setheum Foundation's Charity Fund
	fn get_public_fund_serpup(amount: Balance, currency_id: Self::CurrencyId) -> DispatchResult {
		let public_fund_account = T::PublicFundAccountId::get();
		// Charity Fund SerpUp Pool - 10%
		let serping_amount: Balance = amount / 10;
		// Issue the SerpUp propper to the Charity Fund
		Self::issue_standard(currency_id, &public_fund_account, serping_amount)?;

		Self::deposit_event(Event::SerpUpDelivery(amount, currency_id));
		Ok(())
	}

	/// SerpUp ratio for SetPay Cashdrops
	fn get_cashdrop_serpup(amount: Balance, currency_id: Self::CurrencyId) -> DispatchResult {
		let setpay_account = &T::CashDropPoolAccountId::get();

		// SetPay SerpUp Pool - 50%
		let five: Balance = 5;
		let serping_amount: Balance = five.saturating_mul(amount / 10);
		// Issue the SerpUp propper to the SetPayVault
		Self::issue_standard(currency_id, &setpay_account, serping_amount)?;

		Self::deposit_event(Event::SerpUpDelivery(amount, currency_id));
		Ok(())
	}

	/// Serplus ratio for BuyBack Swaps to burn Setter and Setheum (SETR:SETM)
	fn get_buyback_serplus(amount: Balance, currency_id: Self::CurrencyId) -> DispatchResult {
		// BuyBack Pool - 50%
		// Buyback with 50:50 with SETR:SETM
		let two: Balance = 2;
		let serping_amount_25percent: Balance = two.saturating_mul(amount / 4);
		
		<Self as SerpTreasuryExtended<T::AccountId>>::serplus_swap_exact_setcurrency_to_setter(
			currency_id,
			serping_amount_25percent,
		);
		<Self as SerpTreasuryExtended<T::AccountId>>::serplus_swap_exact_setcurrency_to_native(
			currency_id,
			serping_amount_25percent,
		);
		Ok(())
	}

	/// Serplus ratio for Setheum Foundation's Charity Fund
	fn get_public_fund_serplus(amount: Balance, currency_id: Self::CurrencyId) -> DispatchResult {
		let public_fund = T::PublicFundAccountId::get();
		let cdp_treasury = T::CDPTreasuryAccountId::get();
		// Public Fund Pool - 50%
		let two: Balance = 2;
		// Charity Fund Serplus Pool - 10%
		let serping_amount_25percent: Balance = two.saturating_mul(amount / 4);
		// Transfer the Serplus propper to the Charity Fund
		T::Currency::transfer(currency_id, &cdp_treasury, &public_fund, serping_amount_25percent)?;
		Ok(())
	}

	/// issue system surplus(SETUSD) to their destinations according to the serpup_ratio.
	pub fn on_serplus(currency_id: CurrencyId, amount: Balance) -> DispatchResult {
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
		Self::get_buyback_serplus(amount, currency_id)?;
		Self::get_public_fund_serplus(amount, currency_id)?;

		Self::deposit_event(Event::SerplusDelivery(amount, currency_id));
		Ok(())
	}

	/// issue serpup surplus(stable currencies) to their destinations according to the serpup_ratio.
	pub fn on_serpup(currency_id: CurrencyId, amount: Balance) -> DispatchResult {
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
		Self::get_public_fund_serpup(amount, currency_id)?;
		Self::get_cashdrop_serpup(amount, currency_id)?;
		Self::get_buyback_serpup(amount, currency_id)?;

		Self::deposit_event(Event::SerpUp(amount, currency_id));
		Ok(())
	}

	// buy back and burn surplus(stable currencies) with swap on DEX
	// Create the necessary serp down parameters and swap currencies then burn swapped currencies.
	//
	// TODO: Update to add the burning of the stablecoins!
	//
	pub fn on_serpdown(currency_id: CurrencyId, amount: Balance) -> DispatchResult {
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
	pub fn claim_cashdrop(currency_id: CurrencyId, who: &T::AccountId, transfer_amount: Self::Balance) -> DispatchResult {
		ensure!(
			T::StableCurrencyIds::get().contains(&currency_id),
			Error::<T>::InvalidCurrencyType,
		);

		// IF Setter, use Setter claim rate (4%),
		// else, use SetDollar claim rate (2%).
		if currency_id == T::SetterCurrencyId::get() {
			let minimum_claimable_transfer = T::SetterMinimumClaimableTransferAmounts::get();
			let maximum_claimable_transfer = T::SetterMaximumClaimableTransferAmounts::get();

			ensure!(
				transfer_amount <= minimum_claimable_transfer || transfer_amount >= maximum_claimable_transfer,
				Error::<T>::TransferNotEligibleForCashDrop,
			);

			let balance_cashdrop_amount = transfer_amount / 25; // 4% cashdrop
			let cashdrop_pool_reward = transfer_amount / 50; // 2% cashdrop_pool_reward
			let serp_balance = T::Currency::free_balance(currency_id, &T::CashDropPoolAccountId::get());
			ensure!(
				balance_cashdrop_amount <= serp_balance,
				Error::<T>::CashdropNotAvailable,
			);

			T::Currency::transfer(currency_id, &T::CashDropPoolAccountId::get(), who, balance_cashdrop_amount)?;
			T::Currency::deposit(currency_id, &T::CashDropPoolAccountId::get(), cashdrop_pool_reward)?;
		} else {
			// for the SetDollar ---vvvvvvvvv---
			let minimum_claimable_transfer = T::SetDollarMinimumClaimableTransferAmounts::get();
			let maximum_claimable_transfer = T::SetDollarMaximumClaimableTransferAmounts::get();

			ensure!(
				transfer_amount <= minimum_claimable_transfer || transfer_amount >= maximum_claimable_transfer,
				Error::<T>::TransferNotEligibleForCashDrop,
			);

			let balance_cashdrop_amount = transfer_amount / 50; // 2% cashdrop
			let cashdrop_pool_reward = transfer_amount / 100; // 1% cashdrop_pool_reward
			let serp_balance = T::Currency::free_balance(currency_id, &T::CashDropPoolAccountId::get());
			ensure!(
				balance_cashdrop_amount <= serp_balance,
				Error::<T>::CashdropNotAvailable,
			);

			T::Currency::transfer(currency_id, &T::CashDropPoolAccountId::get(), who, balance_cashdrop_amount)?;
			T::Currency::deposit(currency_id, &T::CashDropPoolAccountId::get(), cashdrop_pool_reward)?;

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
		let dinar_currency_id = T::GetDinarCurrencyId::get();
		let setter_currency_id = T::SetterCurrencyId::get();
		
		let swap_path = T::DefaultSwapPathList::get();

		for path in swap_path {
			match path.last() {
				Some(setter_currency_id) if *setter_currency_id == dinar_currency_id => {
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
						if T::Dex::buyback_swap_with_exact_target(
							&Self::account_id(),
							&path,
							target_amount.unique_saturated_into(),
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

	/// swap Serp to get exact Setter,
	/// return actual supply Serp amount
	#[allow(unused_variables)]
	fn swap_serp_to_exact_setter(
		target_amount: Balance,
	) {
		let serptoken_currency_id = T::GetSerpCurrencyId::get();
		let setter_currency_id = T::SetterCurrencyId::get();
		
		let swap_path = T::DefaultSwapPathList::get();

		for path in swap_path {
			match path.last() {
				Some(setter_currency_id) if *setter_currency_id == serptoken_currency_id => {
					let serptoken_currency_id = *path.first().expect("these's first guaranteed by match");
					// calculate the supply limit according to oracle price and the slippage limit,
					// if oracle price is not avalible, do not limit
					let max_supply_limit = if let Some(target_price) =
						T::PriceSource::get_relative_price(*setter_currency_id, serptoken_currency_id)
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
						serptoken_currency_id,
						&Self::account_id(),
						max_supply_limit.unique_saturated_into()
					).is_ok() {
						if T::Dex::buyback_swap_with_exact_target(
							&Self::account_id(),
							&path,
							target_amount.unique_saturated_into(),
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

	/// swap Dinar to get exact Setter,
	/// return actual supply Dinar amount
	#[allow(unused_variables)]
	fn swap_dinar_to_exact_setcurrency(
		currency_id: CurrencyId,
		target_amount: Balance,
	) {
		let dinar_currency_id = T::GetDinarCurrencyId::get();
		
		let swap_path = T::DefaultSwapPathList::get();

		for path in swap_path {
			match path.last() {
				Some(currency_id) if *currency_id == dinar_currency_id => {
					let dinar_currency_id = *path.first().expect("these's first guaranteed by match");
					// calculate the supply limit according to oracle price and the slippage limit,
					// if oracle price is not avalible, do not limit
					let max_supply_limit = if let Some(target_price) =
						T::PriceSource::get_relative_price(*currency_id, dinar_currency_id)
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
						if T::Dex::buyback_swap_with_exact_target(
							&Self::account_id(),
							&path,
							target_amount.unique_saturated_into(),
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
		let setter_currency_id = T::SetterCurrencyId::get();

		let swap_path = T::DefaultSwapPathList::get();

		for path in swap_path {
			match path.last() {
				Some(currency_id) if *currency_id == setter_currency_id => {
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
						if T::Dex::buyback_swap_with_exact_target(
							&Self::account_id(),
							&path,
							target_amount.unique_saturated_into(),
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

		let swap_path = T::DefaultSwapPathList::get();

		for path in swap_path {
			match path.last() {
				Some(currency_id) if *currency_id == serp_currency_id => {
					let serp_currency_id = *path.first().expect("these's first guaranteed by match");
					// calculate the supply limit according to oracle price and the slippage limit,
					// if oracle price is not avalible, do not limit
					let max_supply_limit = if let Some(target_price) =
						T::PriceSource::get_relative_price(*currency_id, serp_currency_id)
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
						serp_currency_id,
						&Self::account_id(),
						max_supply_limit.unique_saturated_into()
					).is_ok() {
						if T::Dex::buyback_swap_with_exact_target(
							&Self::account_id(),
							&path,
							target_amount.unique_saturated_into(),
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
		let dinar_currency_id = T::GetDinarCurrencyId::get();
		let currency_id = T::SetterCurrencyId::get();

		let swap_path = T::DefaultSwapPathList::get();
		
		for path in swap_path {
			// match path.last() {
			// 	Some(dinar_currency_id) if *dinar_currency_id == dinar_currency_id => {
			// 		let currency_id = *path.first().expect("these's first guaranteed by match");
			// 		// calculate the target limit according to oracle price and the slippage limit,
			// 		// if oracle price is not avalible, do not limit
			// 		let min_target_limit = if let Some(target_price) =
			// 			T::PriceSource::get_relative_price(*dinar_currency_id, currency_id)
			// 		{
			// 			Ratio::one()
			// 				.saturating_sub(T::MaxSwapSlippageCompareToOracle::get())
			// 				.reciprocal()
			// 				.unwrap_or_else(Ratio::max_value)
			// 				.saturating_mul_int(target_price.saturating_mul_int(supply_amount))
			// 		} else {
			// 			CurrencyBalanceOf::<T>::max_value()
			// 		};

					if T::Currency::deposit(
						currency_id,
						&Self::account_id(),
						supply_amount.unique_saturated_into()
					).is_ok() {
						// Swap and burn Native Reserve asset (Dinar (DNAR))
						if T::Dex::buyback_swap_with_exact_supply(
							&Self::account_id(),
							&path,
							supply_amount.unique_saturated_into(),
							// min_target_limit.unique_saturated_into(),
						)
						.is_ok()
						{
							// successfully swap, break iteration.
							break;
						}
					}
			// 	}
			// 	_ => {}
			// }
		}
	}

	/// Swap exact amount of Setter to Serp,
	/// return actual supply Setter amount
	///
	/// 
	/// When Setter gets SerpUp
	#[allow(unused_variables)]
	fn swap_exact_setter_to_serp(
		supply_amount: Balance,
	) {
		let serptoken_currency_id = T::GetSerpCurrencyId::get();
		let currency_id = T::SetterCurrencyId::get();

		let swap_path = T::DefaultSwapPathList::get();
		
		for path in swap_path {
			// match path.last() {
			// 	Some(serptoken_currency_id) if *serptoken_currency_id == serptoken_currency_id => {
			// 		let currency_id = *path.first().expect("these's first guaranteed by match");
			// 		// calculate the target limit according to oracle price and the slippage limit,
			// 		// if oracle price is not avalible, do not limit
			// 		let min_target_limit = if let Some(target_price) =
			// 			T::PriceSource::get_relative_price(*serptoken_currency_id, currency_id)
			// 		{
			// 			Ratio::one()
			// 				.saturating_sub(T::MaxSwapSlippageCompareToOracle::get())
			// 				.reciprocal()
			// 				.unwrap_or_else(Ratio::max_value)
			// 				.saturating_mul_int(target_price.saturating_mul_int(supply_amount))
			// 		} else {
			// 			CurrencyBalanceOf::<T>::max_value()
			// 		};

					if T::Currency::deposit(
						currency_id,
						&Self::account_id(),
						supply_amount.unique_saturated_into()
					).is_ok() {
						// Swap and burn Native Reserve asset (Serp (SERP))
						if T::Dex::buyback_swap_with_exact_supply(
							&Self::account_id(),
							&path,
							supply_amount.unique_saturated_into(),
							// min_target_limit.unique_saturated_into(),
						)
						.is_ok()
						{
							// successfully swap, break iteration.
							break;
						}
					}
			// 	}
			// 	_ => {}
			// }
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
		let dinar_currency_id = T::GetDinarCurrencyId::get();

		let swap_path = T::DefaultSwapPathList::get();
		
		for path in swap_path {
			// match path.last() {
			// 	Some(dinar_currency_id) if *dinar_currency_id == dinar_currency_id => {
			// 		let currency_id = *path.first().expect("these's first guaranteed by match");
			// 		// calculate the target limit according to oracle price and the slippage limit,
			// 		// if oracle price is not avalible, do not limit
			// 		let min_target_limit = if let Some(target_price) =
			// 			T::PriceSource::get_relative_price(*dinar_currency_id, currency_id)
			// 		{
			// 			Ratio::one()
			// 				.saturating_sub(T::MaxSwapSlippageCompareToOracle::get())
			// 				.reciprocal()
			// 				.unwrap_or_else(Ratio::max_value)
			// 				.saturating_mul_int(target_price.saturating_mul_int(supply_amount))
			// 		} else {
			// 			CurrencyBalanceOf::<T>::max_value()
			// 		};

					if T::Currency::deposit(
						currency_id,
						&Self::account_id(),
						supply_amount.unique_saturated_into()
					).is_ok() {
						// Swap and burn The Dinar Reserve asset (Dinar (DNAR))
						if T::Dex::buyback_swap_with_exact_supply(
							&Self::account_id(),
							&path,
							supply_amount.unique_saturated_into(),
							// min_target_limit.unique_saturated_into(),
						)
						.is_ok()
						{
							// successfully swap, break iteration.
							break;
						}
					}
			// 	}
			// 	_ => {}
			// }
		}
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

		let swap_path = T::DefaultSwapPathList::get();
		
		for path in swap_path {
			// match path.last() {
				// Some(setter_currency_id) if *setter_currency_id == setter_currency_id => {
				// 	let currency_id = *path.first().expect("these's first guaranteed by match");
				// 	// calculate the supply limit according to oracle price and the slippage limit,
				// 	// if oracle price is not avalible, do not limit
				// 	let min_target_limit = if let Some(target_price) =
				// 		T::PriceSource::get_relative_price(*setter_currency_id, currency_id)
				// 	{
				// 		Ratio::one()
				// 			.saturating_sub(T::MaxSwapSlippageCompareToOracle::get())
				// 			.reciprocal()
				// 			.unwrap_or_else(Ratio::max_value)
				// 			.saturating_mul_int(target_price.saturating_mul_int(supply_amount))
				// 	} else {
				// 		CurrencyBalanceOf::<T>::max_value()
				// 	};

					if T::Currency::deposit(
						currency_id,
						&Self::account_id(),
						supply_amount.unique_saturated_into()
					).is_ok() {
						if T::Dex::buyback_swap_with_exact_supply(
							&Self::account_id(),
							&path,
							supply_amount.unique_saturated_into(),
							// min_target_limit.unique_saturated_into(),
						)
						.is_ok()
						{
							// successfully swap, break iteration
							break;
						}
					}
			// 	}
			// 	_ => {}
			// }
		}
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

		let swap_path = T::DefaultSwapPathList::get();
		
		for path in swap_path {
			// match path.last() {
				// Some(setter_currency_id) if *setter_currency_id == setter_currency_id => {
				// 	let currency_id = *path.first().expect("these's first guaranteed by match");
				// 	// calculate the supply limit according to oracle price and the slippage limit,
				// 	// if oracle price is not avalible, do not limit
				// 	let min_target_limit = if let Some(target_price) =
				// 		T::PriceSource::get_relative_price(*setter_currency_id, currency_id)
				// 	{
				// 		Ratio::one()
				// 			.saturating_sub(T::MaxSwapSlippageCompareToOracle::get())
				// 			.reciprocal()
				// 			.unwrap_or_else(Ratio::max_value)
				// 			.saturating_mul_int(target_price.saturating_mul_int(supply_amount))
				// 	} else {
				// 		CurrencyBalanceOf::<T>::max_value()
				// 	};

					if T::Currency::transfer(
						T::GetSetUSDCurrencyId::get(),
						&T::CDPTreasuryAccountId::get(),
						&Self::account_id(),
						supply_amount.unique_saturated_into()
					).is_ok() {
						if T::Dex::buyback_swap_with_exact_supply(
							&Self::account_id(),
							&path,
							supply_amount.unique_saturated_into(),
							// min_target_limit.unique_saturated_into(),
						)
						.is_ok()
						{
							// successfully swap, break iteration
							break;
						}
					}
			// 	}
			// 	_ => {}
			// }
		}
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

		let swap_path = T::DefaultSwapPathList::get();
		
		for path in swap_path {
			// match path.last() {
				// Some(setm_currency_id) if *setm_currency_id == setm_currency_id => {
				// 	let currency_id = *path.first().expect("these's first guaranteed by match");
				// 	// calculate the supply limit according to oracle price and the slippage limit,
				// 	// if oracle price is not avalible, do not limit
				// 	let min_target_limit = if let Some(target_price) =
				// 		T::PriceSource::get_relative_price(*setm_currency_id, currency_id)
				// 	{
				// 		Ratio::one()
				// 			.saturating_sub(T::MaxSwapSlippageCompareToOracle::get())
				// 			.reciprocal()
				// 			.unwrap_or_else(Ratio::max_value)
				// 			.saturating_mul_int(target_price.saturating_mul_int(supply_amount))
				// 	} else {
				// 		CurrencyBalanceOf::<T>::max_value()
				// 	};

					if T::Currency::transfer(
						T::GetSetUSDCurrencyId::get(),
						&T::CDPTreasuryAccountId::get(),
						&Self::account_id(),
						supply_amount.unique_saturated_into()
					).is_ok() {
						if T::Dex::buyback_swap_with_exact_supply(
							&Self::account_id(),
							&path,
							supply_amount.unique_saturated_into(),
							// min_target_limit.unique_saturated_into(),
						)
						.is_ok()
						{
							// successfully swap, break iteration
							break;
						}
					}
			// 	}
			// 	_ => {}
			// }
		}
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

		let swap_path = T::DefaultSwapPathList::get();
		
		for path in swap_path {
			// match path.last() {
			// 	Some(native_currency_id) if *native_currency_id == native_currency_id => {
			// 		let currency_id = *path.first().expect("these's first guaranteed by match");
			// 		// calculate the supply limit according to oracle price and the slippage limit,
			// 		// if oracle price is not avalible, do not limit
			// 		let min_target_limit = if let Some(target_price) =
			// 			T::PriceSource::get_relative_price(*native_currency_id, currency_id)
			// 		{
			// 			Ratio::one()
			// 				.saturating_sub(T::MaxSwapSlippageCompareToOracle::get())
			// 				.reciprocal()
			// 				.unwrap_or_else(Ratio::max_value)
			// 				.saturating_mul_int(target_price.saturating_mul_int(supply_amount))
			// 		} else {
			// 			CurrencyBalanceOf::<T>::max_value()
			// 		};

					if T::Currency::deposit(
						currency_id,
						&Self::account_id(),
						supply_amount.unique_saturated_into()
					).is_ok() {
						// Swap and burn Native Currency (Setheum (SETM))
						if T::Dex::buyback_swap_with_exact_supply(
							&Self::account_id(),
							&path,
							supply_amount.unique_saturated_into(),
							// min_target_limit.unique_saturated_into(),
						)
						.is_ok()
						{
							// successfully swap, break iteration
							break;
						}
					}
			// 	}
			// 	_ => {}
			// }
		}
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

		let swap_path = T::DefaultSwapPathList::get();
		
		for path in swap_path {
			// match path.last() {
			// 	Some(serptoken_currency_id) if *serptoken_currency_id == serptoken_currency_id => {
			// 		let currency_id = *path.first().expect("these's first guaranteed by match");
			// 		// calculate the target limit according to oracle price and the slippage limit,
			// 		// if oracle price is not avalible, do not limit
			// 		let min_target_limit = if let Some(target_price) =
			// 			T::PriceSource::get_relative_price(*serptoken_currency_id, currency_id)
			// 		{
			// 			Ratio::one()
			// 				.saturating_sub(T::MaxSwapSlippageCompareToOracle::get())
			// 				.reciprocal()
			// 				.unwrap_or_else(Ratio::max_value)
			// 				.saturating_mul_int(target_price.saturating_mul_int(supply_amount))
			// 		} else {
			// 			CurrencyBalanceOf::<T>::max_value()
			// 		};

					if T::Currency::deposit(
						currency_id,
						&Self::account_id(),
						supply_amount.unique_saturated_into()
					).is_ok() {
						// Swap and burn Serp Reserve asset (Serp (SERP))
						if T::Dex::buyback_swap_with_exact_supply(
							&Self::account_id(),
							&path,
							supply_amount.unique_saturated_into(),
							// min_target_limit.unique_saturated_into(),
						)
						.is_ok()
						{
							// successfully swap, break iteration.
							break;
						}
					}
			// 	}
			// 	_ => {}
			// }
		}
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
