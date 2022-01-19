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

//! # Airdrop Module
//!
//! ## Overview
//!
//! This module creates airdrops and distributes airdrops to the -
//! acccounts in the airdrops from an update origin. 
//! The module for distributing Setheum Airdrops,
//! it will be used for the Setheum IAE (Initial Airdrop Event).

#![cfg_attr(not(feature = "std"), no_std)]
#![allow(clippy::unused_unit)]

use frame_support::{pallet_prelude::*, transactional, PalletId, traits::Get};
use frame_system::pallet_prelude::*;
use orml_traits::{MultiCurrency, MultiCurrencyExtended};
use primitives::{AirDropCurrencyId, Balance, CurrencyId};
use sp_std::vec::Vec;
use sp_runtime::traits::AccountIdConversion;

mod mock;

pub use module::*;

type BalanceOf<T> = <<T as Config>::MultiCurrency as MultiCurrency<<T as frame_system::Config>::AccountId>>::Balance;

#[frame_support::pallet]
pub mod module {
	use super::*;

	#[pallet::config]
	pub trait Config: frame_system::Config {
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;

		/// The Currency for managing assets related to the SERP (Setheum Elastic Reserve Protocol).
		type MultiCurrency: MultiCurrencyExtended<Self::AccountId, CurrencyId = CurrencyId, Balance = Balance>;

		#[pallet::constant]
		/// Setter (SETR) currency id
		/// 
		type SetterCurrencyId: Get<CurrencyId>;

		#[pallet::constant]
		/// The SetDollar (SETUSD) currency id
		type GetSetUSDId: Get<CurrencyId>;

		#[pallet::constant]
		/// Native Setheum (SETM) currency id. [P]Pronounced "set M" or "setem"
		/// 
		type GetNativeCurrencyId: Get<CurrencyId>;

		#[pallet::constant]
		/// Serp (SERP) currency id.
		/// 
		type GetSerpCurrencyId: Get<CurrencyId>;

		#[pallet::constant]
		/// The Dinar (DNAR) currency id.
		/// 
		type GetDinarCurrencyId: Get<CurrencyId>;

		#[pallet::constant]
		/// HighEnd LaunchPad (HELP) currency id. (LaunchPad Token)
		/// 
		type GetHelpCurrencyId: Get<CurrencyId>;

		#[pallet::constant]
		/// The Airdrop module pallet id, keeps airdrop funds.
		type FundingOrigin: Get<Self::AccountId>;

		/// The origin which may lock and unlock prices feed to system.
		type DropOrigin: EnsureOrigin<Self::Origin>;
		
		#[pallet::constant]
		/// The Airdrop module pallet id, keeps airdrop funds.
		type PalletId: Get<PalletId>;

	}

	#[pallet::error]
	pub enum Error<T> {
		// Duplicate Airdrop Account
		DuplicateAccounts,
	}

	#[pallet::event]
	#[pallet::generate_deposit(pub(crate) fn deposit_event)]
	#[pallet::metadata(T::AccountId = "AccountId", BalanceOf<T> = "Balance", AirDropCurrencyId = "AirDropCurrencyId")]
	pub enum Event<T: Config> {
		/// Drop Airdrop \[currency_id\]
		Airdrop(AirDropCurrencyId),
		/// Donate to Airdrop Treasury \[from, currency_id, amount\]
		DonateToAirdropTreasury(T::AccountId, AirDropCurrencyId, BalanceOf<T>),
		/// Fund the Airdrop Treasury from `FundingOrigin` \[from, currency_id, amount\]
		FundAirdropTreasury(T::AccountId, AirDropCurrencyId, BalanceOf<T>)
	}

	#[pallet::pallet]
	pub struct Pallet<T>(PhantomData<T>);

	#[pallet::hooks]
	impl<T: Config> Hooks<T::BlockNumber> for Pallet<T> {}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// Fund Airdrop Treasury from deposit creation.
		///
		/// The dispatch origin of this call must be `DropOrigin`.
		///
		/// - `currency_id`: `AirDropCurrencyId` funding currency type.
		/// - `amount`: `BalanceOf<T>` funding amounts.
		#[pallet::weight((100_000_000 as Weight, DispatchClass::Operational))]
		#[transactional]
		pub fn fund_airdrop_treasury(
			origin: OriginFor<T>,
			currency_id: AirDropCurrencyId,
			amount: BalanceOf<T>,
		) -> DispatchResult {
			T::DropOrigin::ensure_origin(origin)?;

			if currency_id == AirDropCurrencyId::SETR {
				T::MultiCurrency::transfer(T::SetterCurrencyId::get(), &T::FundingOrigin::get(), &Self::account_id(), amount)?;
			} else if currency_id == AirDropCurrencyId::SETUSD {
				T::MultiCurrency::transfer(T::GetSetUSDId::get(), &T::FundingOrigin::get(), &Self::account_id(), amount)?;
			} else if currency_id == AirDropCurrencyId::SETM {
				T::MultiCurrency::transfer(T::GetNativeCurrencyId::get(), &T::FundingOrigin::get(), &Self::account_id(), amount)?;
			} else if currency_id == AirDropCurrencyId::SERP {
				T::MultiCurrency::transfer(T::GetSerpCurrencyId::get(), &T::FundingOrigin::get(), &Self::account_id(), amount)?;
			} else if currency_id == AirDropCurrencyId::DNAR {
				T::MultiCurrency::transfer(T::GetDinarCurrencyId::get(), &T::FundingOrigin::get(), &Self::account_id(), amount)?;
			} else if currency_id == AirDropCurrencyId::HELP {
				T::MultiCurrency::transfer(T::GetHelpCurrencyId::get(), &T::FundingOrigin::get(), &Self::account_id(), amount)?;
			}
			
			Self::deposit_event(Event::FundAirdropTreasury(T::FundingOrigin::get(), currency_id, amount));
			Ok(())
		}

		/// Make Airdrop to beneficiaries.
		///
		/// The dispatch origin of this call must be `DropOrigin`.
		///
		/// - `currency_id`: `AirDropCurrencyId` airdrop currency type.
		/// - `airdrop_list_json`: airdrop accounts and respective amounts in json format.
		#[pallet::weight((100_000_000 as Weight, DispatchClass::Operational))]
		#[transactional]
		pub fn make_airdrop(
			origin: OriginFor<T>,
			currency_id: AirDropCurrencyId,
			airdrop_list: Vec<(T::AccountId, Balance)>,
		) -> DispatchResult {
			T::DropOrigin::ensure_origin(origin)?;

			Self::do_make_airdrop(currency_id, airdrop_list)?;
			Ok(())
		}
	}
}

impl<T: Config> Pallet<T> {
	/// Get account of SERP Treasury module.
	pub fn account_id() -> T::AccountId {
		T::PalletId::get().into_account()
	}

	fn do_make_airdrop(currency_id: AirDropCurrencyId, airdrop_list: Vec<(T::AccountId, Balance)>) -> DispatchResult {

		// Make sure only unique accounts receive Airdrop
        let unique_accounts = airdrop_list
		.iter()
		.map(|(x,_)| x)
		.cloned();
        ensure!(
            unique_accounts.len() == airdrop_list.len(),
            Error::<T>::DuplicateAccounts,
        );

		match currency_id {
			AirDropCurrencyId::SETR => {
				for (beneficiary, amount) in airdrop_list.iter() {
					T::MultiCurrency::transfer(T::SetterCurrencyId::get(), &Self::account_id(), beneficiary, *amount)?;
				}
			}
			id if id == AirDropCurrencyId::SETUSD => {
				for (beneficiary, amount) in  airdrop_list.iter() {
					T::MultiCurrency::transfer(T::GetSetUSDId::get(), &Self::account_id(), beneficiary, *amount)?;
				}
			}
			id if id == AirDropCurrencyId::SETM => {
				for (beneficiary, amount) in  airdrop_list.iter() {
					T::MultiCurrency::transfer(T::GetNativeCurrencyId::get(), &Self::account_id(), beneficiary, *amount)?;
				}
			}
			id if id == AirDropCurrencyId::SERP => {
				for (beneficiary, amount) in  airdrop_list.iter() {
					T::MultiCurrency::transfer(T::GetSerpCurrencyId::get(), &Self::account_id(), beneficiary, *amount)?;
				}
			}
			id if id == AirDropCurrencyId::DNAR => {
				for (beneficiary, amount) in  airdrop_list.iter() {
					T::MultiCurrency::transfer(T::GetDinarCurrencyId::get(), &Self::account_id(), beneficiary, *amount)?;
				}
			}
			id if id == AirDropCurrencyId::HELP => {
				for (beneficiary, amount) in  airdrop_list.iter() {
					T::MultiCurrency::transfer(T::GetHelpCurrencyId::get(), &Self::account_id(), beneficiary, *amount)?;
				}
			} _ => {}
		}
		
		Self::deposit_event(Event::Airdrop(currency_id));
		Ok(())
	}
}

// /* AccountId to Hex */
// pub fn account_to_hex(account: AccountIdOf<T>) -> AccountId {
// 	let account_id = account.as_bytes();
// 	let mut account_id_hex = AccountId::default();
// 	for i in 0..account_id.len() {
// 		account_id_hex[i] = account_id[i];
// 	}
// 	account_id_hex
// }


// /* Hex to AccountId */
// pub fn hex_to_account(account: AccountId) -> AccountIdOf<T> {
// 	let account_id = H256::from(account).as_bytes();
// 	let account_id_hex = account_id.to_vec().as_slice();
// 	let account_id_hex2 = AccountId::encode(account_id_hex);
// 	let account_id_hex3 = account_id_hex2.unwrap();
// 	account_id_hex3.into_iter().collect::<Vec<u8>>()
// }
