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

use frame_support::{pallet_prelude::*, transactional, traits::Get};
use frame_system::pallet_prelude::*;
use orml_traits::MultiCurrency;
use primitives::{AirDropCurrencyId, Balance, CurrencyId};
use sp_std::vec::Vec;

mod mock;

pub use module::*;

#[frame_support::pallet]
pub mod module {
	use super::*;

	#[pallet::config]
	pub trait Config: frame_system::Config {
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;

		/// Currency provide the total insurance of LPToken.
		type Currency: MultiCurrency<Self::AccountId, CurrencyId = CurrencyId, Balance = Balance>;

		#[pallet::constant]
		/// Setter (SETR) currency id
		/// 
		type SetterCurrencyId: Get<CurrencyId>;

		#[pallet::constant]
		/// The SetUSD currency id, it should be SETUSD in Setheum.
		type GetSetUSDId: Get<CurrencyId>;

		/// The origin which may lock and unlock prices feed to system.
		type DropOrigin: EnsureOrigin<Self::Origin>;
	}

	#[pallet::error]
	pub enum Error<T> {
		/// Invalid AirDrop Currency Type
		InvalidCurrencyType,
	}

	#[pallet::event]
	#[pallet::generate_deposit(pub(crate) fn deposit_event)]
	pub enum Event<T: Config> {
		/// Drop Airdrop \[currency_id\]
		Airdrop(AirDropCurrencyId)
	}

	#[pallet::pallet]
	pub struct Pallet<T>(_);

	#[pallet::hooks]
	impl<T: Config> Hooks<T::BlockNumber> for Pallet<T> {}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// Update the storeed Airdrop details in the AirDrops storage map.
		/// Once an Airdrop is delivered, the entire AirDrops storage map will be cleared.
		///
		/// The dispatch origin of this call must be `DropOrigin`.
		///
		/// - `currency_id`: `CurrencyId` currency type.
		/// - `amount`: airdrop amount.
		#[pallet::weight((100_000_000 as Weight, DispatchClass::Operational))]
		#[transactional]
		pub fn airdrop(
			origin: OriginFor<T>,
			currency_id: AirDropCurrencyId,
			beneficiaries: Vec<(T::AccountId, Balance)>,
		) -> DispatchResult {
			T::DropOrigin::ensure_origin(origin)?;
			if currency_id == AirDropCurrencyId::SETR {
				for (account, balance) in beneficiaries {
					T::Currency::deposit(T::SetterCurrencyId::get(), &account, balance)?;
				}
			} else if currency_id == AirDropCurrencyId::SETUSD {
				for (account, balance) in beneficiaries {
					T::Currency::deposit(T::GetSetUSDId::get(), &account, balance)?;
				}
			}
			
			Self::deposit_event(Event::Airdrop(currency_id));
			Ok(())
		}
	}
}

impl<T: Config> Pallet<T> {}
