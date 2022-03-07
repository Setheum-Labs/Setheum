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

// TODO: Implement this module propperly for production
//! # QMA FlashLoans Module
//!
//! ## Overview
//!
//! This module creates QMA flashloans and buysback distributes profit
//! to the transaction origin, the QMA pool and buybacks of T1-Tokens and burning -
//! Governed from an update-origin (FinancialCouncil).
//! The module for distributing Setheum Qurud Mudarabah Assaree`ah (QMA) Flashloans.

#![cfg_attr(not(feature = "std"), no_std)]
#![allow(clippy::unused_unit)]

use frame_support::{pallet_prelude::*, transactional, PalletId, traits::Get};
use frame_system::pallet_prelude::*;
use orml_traits::{MultiCurrency, MultiCurrencyExtended};
use primitives::{AccountId, Balance, CurrencyId};
use sp_std::vec::Vec;
use sp_runtime::traits::AccountIdConversion;
use support::{
	DEXManager, PriceProvider, Ratio, SerpTreasury, SerpTreasuryExtended,
};

mod mock;

// pub use module::*;

// type AmountOf<T> =
// 	<<T as Config>::MultiCurrency as MultiCurrencyExtended<<T as frame_system::Config>::AccountId>>::Amount;
// type BalanceOf<T> = <<T as Config>::MultiCurrency as MultiCurrency<<T as frame_system::Config>::AccountId>>::Balance;
// type CurrencyIdOf<T> =
// 	<<T as Config>::MultiCurrency as MultiCurrency<<T as frame_system::Config>::AccountId>>::CurrencyId;

// #[frame_support::pallet]
// pub mod module {
// 	use super::*;

// 	#[pallet::config]
// 	pub trait Config: frame_system::Config {
// 		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;

// 		/// The Currency for managing assets related to the SERP (Setheum Elastic Reserve Protocol).
// 		type MultiCurrency: MultiCurrencyExtended<Self::AccountId, CurrencyId = CurrencyId, Balance = Balance>;

// 		/// SERP Treasury for issuing/burning stable currency adjust standard value
// 		/// adjustment
// 		type SerpTreasury: SerpTreasury<Self::AccountId, Balance = BalanceOf<Self>, CurrencyId = CurrencyId>;

// 		/// The stable currency ids
// 		type StableCurrencyIds: Get<Vec<CurrencyId>>;
		
// 		#[pallet::constant]
// 		/// Setter (SETR) currency id
// 		/// 
// 		type SetterCurrencyId: Get<CurrencyId>;

// 		#[pallet::constant]
// 		/// The SetDollar (SETUSD) currency id
// 		type GetSetUSDId: Get<CurrencyId>;

// 		#[pallet::constant]
// 		/// Native Setheum (SETM) currency id. [P]Pronounced "set M" or "setem"
// 		/// 
// 		type GetNativeCurrencyId: Get<CurrencyId>;

// 		#[pallet::constant]
// 		/// Serp (SERP) currency id.
// 		/// 
// 		type GetSerpCurrencyId: Get<CurrencyId>;

// 		#[pallet::constant]
// 		/// The Dinar (DNAR) currency id.
// 		/// 
// 		type GetDinarCurrencyId: Get<CurrencyId>;

// 		#[pallet::constant]
// 		/// HighEnd LaunchPad (HELP) currency id. (LaunchPad Token)
// 		/// 
// 		type GetHelpCurrencyId: Get<CurrencyId>;

// 		/// The minimum amount of flashloan that may be got.
// 		type MinFlashLoan: GetByKey<CurrencyId, Balance>;

// 		/// The price source to provider external market price.
// 		type PriceSource: PriceProvider<CurrencyId>;

// 		/// The limit for length of trading path used in flashloans.
// 		#[pallet::constant]
// 		type TradingPathLimit: Get<u32>;

// 		/// DEX provide liquidity info.
// 		type DEX: DEXManager<Self::AccountId, CurrencyId, Balance>;

// 		#[pallet::constant]
// 		/// The origin which may lock and unlock prices feed to system.
// 		type UpdateOrigin: EnsureOrigin<Self::Origin>;
		
// 		#[pallet::constant]
// 		/// The QMA module pallet id, keeps airdrop funds.
// 		type PalletId: Get<PalletId>;

// 		/// Weight information for the extrinsics in this module.
// 		type WeightInfo: WeightInfo;
// 	}

// 	#[pallet::error]
// 	pub enum Error<T> {
// 		// Duplicate QMA Account
// 		DuplicateAccounts,
// 	}

// 	#[pallet::event]
// 	#[pallet::generate_deposit(pub(crate) fn deposit_event)]
// 	#[pallet::metadata(T::AccountId = "AccountId", BalanceOf<T> = "Balance", AirDropCurrencyId = "AirDropCurrencyId")]
// 	pub enum Event<T: Config> {
// 		/// Make FlashLoan \[currency_id, loan_amount, profit_amount\]
// 		FlashLoan(CurrencyIdOf<T>, BalanceOf<T>, BalanceOf<T>),
// 		/// Update the Maximum FlashLoan Amount \[currency_id, amount\]
// 		UpdateMaxFlashLoan(CurrencyIdOf<T>, BalanceOf<T>)
// 	}

// 	/// FlashLoan pool for CurrencyId.
// 	///
// 	/// FlashLoanPool: map CurrencyId => Balance
// 	#[pallet::storage]
// 	#[pallet::getter(fn liquidity_pool)]
// 	pub type FlashLoanPool<T: Config> = StorageMap<_, Twox64Concat, CurrencyId, Balance, ValueQuery>;

// 	#[pallet::pallet]
// 	pub struct Pallet<T>(PhantomData<T>);

// 	#[pallet::hooks]
// 	impl<T: Config> Hooks<T::BlockNumber> for Pallet<T> {}

// 	#[pallet::call]
// 	impl<T: Config> Pallet<T> {
// 		/// Update the Maximum amount of FlashLoan an account could get per loan.
// 		///
// 		/// The dispatch origin of this call must be `UpdateOrigin`.
// 		///
// 		/// - `currency_id`: `CurrencyIdOf<T>` FlashLoan currency type.
// 		/// - `amount`: `BalanceOf<T>` Maximum Loan amount.
// 		#[pallet::weight((100_000_000 as Weight, DispatchClass::Operational))]
// 		#[transactional]
// 		pub fn update_maxloan(
// 			origin: OriginFor<T>,
// 			currency_id: AirDropCurrencyId,
// 			amount: BalanceOf<T>,
// 		) -> DispatchResult {
// 			T::UpdateOrigin::ensure_origin(origin)?;

// 			if currency_id == AirDropCurrencyId::SETR {
// 				T::MultiCurrency::transfer(T::SetterCurrencyId::get(), &T::FundingOrigin::get(), &Self::account_id(), amount)?;
// 			} else if currency_id == AirDropCurrencyId::SETUSD {
// 				T::MultiCurrency::transfer(T::GetSetUSDId::get(), &T::FundingOrigin::get(), &Self::account_id(), amount)?;
// 			} else if currency_id == AirDropCurrencyId::SETM {
// 				T::MultiCurrency::transfer(T::GetNativeCurrencyId::get(), &T::FundingOrigin::get(), &Self::account_id(), amount)?;
// 			} else if currency_id == AirDropCurrencyId::SERP {
// 				T::MultiCurrency::transfer(T::GetSerpCurrencyId::get(), &T::FundingOrigin::get(), &Self::account_id(), amount)?;
// 			} else if currency_id == AirDropCurrencyId::DNAR {
// 				T::MultiCurrency::transfer(T::GetDinarCurrencyId::get(), &T::FundingOrigin::get(), &Self::account_id(), amount)?;
// 			} else if currency_id == AirDropCurrencyId::HELP {
// 				T::MultiCurrency::transfer(T::GetHelpCurrencyId::get(), &T::FundingOrigin::get(), &Self::account_id(), amount)?;
// 			}
			
// 			Self::deposit_event(Event::FundAirdropTreasury(T::FundingOrigin::get(), currency_id, amount));
// 			Ok(())
// 		}

// 		/// Make Airdrop to beneficiaries.
// 		///
// 		/// The dispatch origin of this call must be `DropOrigin`.
// 		///
// 		/// - `currency_id`: `AirDropCurrencyId` airdrop currency type.
// 		/// - `airdrop_list_json`: airdrop accounts and respective amounts in json format.
// 		#[pallet::weight((100_000_000 as Weight, DispatchClass::Operational))]
// 		#[transactional]
// 		pub fn make_flashloan(
// 			origin: OriginFor<T>,
// 			currency_id: AirDropCurrencyId,
// 			airdrop_list: Vec<(T::AccountId, Balance)>,
// 		) -> DispatchResult {
// 			T::DropOrigin::ensure_origin(origin)?;

// 			Self::do_make_flashloan(currency_id, airdrop_list)?;
// 			Ok(())
// 		}
// 	}
// }

// impl<T: Config> Pallet<T> {
// 	/// Get account of SQMA FlashLoans module.
// 	pub fn account_id() -> T::AccountId {
// 		T::PalletId::get().into_account()
// 	}

// 	fn do_make_flashloan(currency_id: AirDropCurrencyId, airdrop_list: Vec<(T::AccountId, Balance)>) -> DispatchResult {

// 		// Make sure only unique accounts receive Airdrop
//         let unique_accounts = airdrop_list
// 		.iter()
// 		.map(|(x,_)| x)
// 		.cloned();
//         ensure!(
//             unique_accounts.len() == airdrop_list.len(),
//             Error::<T>::DuplicateAccounts,
//         );

// 		match currency_id {
// 			AirDropCurrencyId::SETR => {
// 				for (beneficiary, amount) in airdrop_list.iter() {
// 					T::MultiCurrency::transfer(T::SetterCurrencyId::get(), &Self::account_id(), beneficiary, *amount)?;
// 				}
// 			}
// 			id if id == AirDropCurrencyId::SETUSD => {
// 				for (beneficiary, amount) in  airdrop_list.iter() {
// 					T::MultiCurrency::transfer(T::GetSetUSDId::get(), &Self::account_id(), beneficiary, *amount)?;
// 				}
// 			}
// 			id if id == AirDropCurrencyId::SETM => {
// 				for (beneficiary, amount) in  airdrop_list.iter() {
// 					T::MultiCurrency::transfer(T::GetNativeCurrencyId::get(), &Self::account_id(), beneficiary, *amount)?;
// 				}
// 			}
// 			id if id == AirDropCurrencyId::SERP => {
// 				for (beneficiary, amount) in  airdrop_list.iter() {
// 					T::MultiCurrency::transfer(T::GetSerpCurrencyId::get(), &Self::account_id(), beneficiary, *amount)?;
// 				}
// 			}
// 			id if id == AirDropCurrencyId::DNAR => {
// 				for (beneficiary, amount) in  airdrop_list.iter() {
// 					T::MultiCurrency::transfer(T::GetDinarCurrencyId::get(), &Self::account_id(), beneficiary, *amount)?;
// 				}
// 			}
// 			id if id == AirDropCurrencyId::HELP => {
// 				for (beneficiary, amount) in  airdrop_list.iter() {
// 					T::MultiCurrency::transfer(T::GetHelpCurrencyId::get(), &Self::account_id(), beneficiary, *amount)?;
// 				}
// 			} _ => {}
// 		}
		
// 		Self::deposit_event(Event::Airdrop(currency_id));
// 		Ok(())
// 	}

// 	// Distribute the FlashLoan profit to the beneficiaries
// 	fn distribute_profit(currency_id: AirDropCurrencyId, airdrop_list: Vec<(T::AccountId, Balance)>) -> DispatchResult {

// 		// Make sure only unique accounts receive Airdrop
//         let unique_accounts = airdrop_list
// 		.iter()
// 		.map(|(x,_)| x)
// 		.cloned();
//         ensure!(
//             unique_accounts.len() == airdrop_list.len(),
//             Error::<T>::DuplicateAccounts,
//         );

// 		match currency_id {
// 			AirDropCurrencyId::SETR => {
// 				for (beneficiary, amount) in airdrop_list.iter() {
// 					T::MultiCurrency::transfer(T::SetterCurrencyId::get(), &Self::account_id(), beneficiary, *amount)?;
// 				}
// 			}
// 			id if id == AirDropCurrencyId::SETUSD => {
// 				for (beneficiary, amount) in  airdrop_list.iter() {
// 					T::MultiCurrency::transfer(T::GetSetUSDId::get(), &Self::account_id(), beneficiary, *amount)?;
// 				}
// 			}
// 			id if id == AirDropCurrencyId::SETM => {
// 				for (beneficiary, amount) in  airdrop_list.iter() {
// 					T::MultiCurrency::transfer(T::GetNativeCurrencyId::get(), &Self::account_id(), beneficiary, *amount)?;
// 				}
// 			}
// 			id if id == AirDropCurrencyId::SERP => {
// 				for (beneficiary, amount) in  airdrop_list.iter() {
// 					T::MultiCurrency::transfer(T::GetSerpCurrencyId::get(), &Self::account_id(), beneficiary, *amount)?;
// 				}
// 			}
// 			id if id == AirDropCurrencyId::DNAR => {
// 				for (beneficiary, amount) in  airdrop_list.iter() {
// 					T::MultiCurrency::transfer(T::GetDinarCurrencyId::get(), &Self::account_id(), beneficiary, *amount)?;
// 				}
// 			}
// 			id if id == AirDropCurrencyId::HELP => {
// 				for (beneficiary, amount) in  airdrop_list.iter() {
// 					T::MultiCurrency::transfer(T::GetHelpCurrencyId::get(), &Self::account_id(), beneficiary, *amount)?;
// 				}
// 			} _ => {}
// 		}
		
// 		Self::deposit_event(Event::Airdrop(currency_id));
// 		Ok(())
// 	}
// }
