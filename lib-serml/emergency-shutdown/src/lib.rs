// This file is part of Setheum.

// Copyright (C) 2020-2021 Setheum Labs.
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

//! # Emergency Shutdown Module
//!
//! ## Overview
//!
//! When a black swan occurs such as price plunge or fatal bug, the highest
//! priority is to minimize user losses as much as possible. When the decision
//! to shutdown system is made, emergency shutdown module needs to trigger all
//! related module to halt, and start a series of operations including close
//! some user entry, freeze feed prices, run offchain worker to settle
//! Settmint has standard, cancel all active auctions module, when standards and gaps are
//! settled, the stable currency holder are allowed to refund a basket of
//! remaining reserve assets.

#![cfg_attr(not(feature = "std"), no_std)]
#![allow(clippy::unused_unit)]

use frame_support::{pallet_prelude::*, transactional};
use frame_system::{ensure_signed, pallet_prelude::*};
use primitives::{Balance, CurrencyId};
use sp_runtime::{traits::Zero, FixedPointNumber};
use sp_std::prelude::*;
use support::{AuctionManager, SerpTreasury, EmergencyShutdown, PriceProvider, Ratio};

mod mock;
mod tests;
pub mod weights;

pub use module::*;
pub use weights::WeightInfo;

#[frame_support::pallet]
pub mod module {
	use super::*;

	#[pallet::config]
	pub trait Config: frame_system::Config + setters::Config {
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;

		/// The list of valid reserve currency types
		#[pallet::constant]
		type ReserveCurrencyIds: Get<Vec<CurrencyId>>;

		/// Price source to freeze currencies' price
		type PriceSource: PriceProvider<CurrencyId>;

		/// SERP Treasury to escrow reserve assets after settlement
		type SerpTreasury: SerpTreasury<Self::AccountId, Balance = Balance, CurrencyId = CurrencyId>;

		/// Check the auction cancellation to decide whether to open the final
		/// redemption
		type AuctionManagerHandler: AuctionManager<Self::AccountId, Balance = Balance, CurrencyId = CurrencyId>;

		/// The origin which may trigger emergency shutdown. Root can always do
		/// this.
		type ShutdownOrigin: EnsureOrigin<Self::Origin>;

		/// Weight information for the extrinsics in this module.
		type WeightInfo: WeightInfo;
	}

	#[pallet::error]
	pub enum Error<T> {
		/// System has already been shutdown
		AlreadyShutdown,
		/// Must after system shutdown
		MustAfterShutdown,
		/// Final redemption is still not opened
		CanNotRefund,
		/// Exist potential surplus, means settlement has not been completed
		ExistPotentialSurplus,
		/// Exist unhandled standard, means settlement has not been completed
		ExistUnhandledStandard,
	}

	#[pallet::event]
	#[pallet::generate_deposit(fn deposit_event)]
	pub enum Event<T: Config> {
		/// Emergency shutdown occurs. \[block_number\]
		Shutdown(T::BlockNumber),
		/// The final redemption opened. \[block_number\]
		OpenRefund(T::BlockNumber),
		/// Refund info. \[caller, stable_coin_amount, refund_list\]
		Refund(T::AccountId, Balance, Vec<(CurrencyId, Balance)>),
	}

	/// Emergency shutdown flag
	#[pallet::storage]
	#[pallet::getter(fn is_shutdown)]
	pub type IsShutdown<T: Config> = StorageValue<_, bool, ValueQuery>;

	/// Open final redemption flag
	#[pallet::storage]
	#[pallet::getter(fn can_refund)]
	pub type CanRefund<T: Config> = StorageValue<_, bool, ValueQuery>;

	#[pallet::pallet]
	pub struct Pallet<T>(PhantomData<T>);

	#[pallet::hooks]
	impl<T: Config> Hooks<T::BlockNumber> for Pallet<T> {}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// Start emergency shutdown
		///
		/// The dispatch origin of this call must be `ShutdownOrigin`.
		#[pallet::weight((T::WeightInfo::emergency_shutdown(T::ReserveCurrencyIds::get().len() as u32), DispatchClass::Operational))]
		#[transactional]
		pub fn emergency_shutdown(origin: OriginFor<T>) -> DispatchResultWithPostInfo {
			T::ShutdownOrigin::ensure_origin(origin)?;
			ensure!(!Self::is_shutdown(), Error::<T>::AlreadyShutdown);

			// get all reserve types
			let reserve_currency_ids = T::ReserveCurrencyIds::get();

			// lock price for every reserve
			for currency_id in reserve_currency_ids {
				<T as Config>::PriceSource::lock_price(currency_id);
			}

			IsShutdown::<T>::put(true);
			Self::deposit_event(Event::Shutdown(<frame_system::Module<T>>::block_number()));
			Ok(().into())
		}

		/// Open final redemption if settlement is completed.
		///
		/// The dispatch origin of this call must be `ShutdownOrigin`.
		#[pallet::weight((T::WeightInfo::open_reserve_refund(), DispatchClass::Operational))]
		#[transactional]
		pub fn open_reserve_refund(origin: OriginFor<T>) -> DispatchResultWithPostInfo {
			T::ShutdownOrigin::ensure_origin(origin)?;
			ensure!(Self::is_shutdown(), Error::<T>::MustAfterShutdown); // must after shutdown

			// Ensure there's no standard and surplus auction now, they may bring uncertain
			// surplus to system. Cancel all surplus auctions and standard auctions to pass the
			// check!
			ensure!(
				<T as Config>::AuctionManagerHandler::get_total_standard_in_auction().is_zero()
					&& <T as Config>::AuctionManagerHandler::get_total_surplus_in_auction().is_zero(),
				Error::<T>::ExistPotentialSurplus,
			);

			// Ensure all standards of Settmint have been settled, and all reserve auction has
			// been done or canceled. Settle all reserves type Settmint which have standard,
			// cancel all reserve auctions in forward stage and wait for all reserve
			// auctions in reverse stage to be ended.
			let reserve_currency_ids = T::ReserveCurrencyIds::get();
			for currency_id in reserve_currency_ids {
				// there's no reserve auction
				ensure!(
					<T as Config>::AuctionManagerHandler::get_total_reserve_in_auction(currency_id).is_zero(),
					Error::<T>::ExistPotentialSurplus,
				);
				// there's on standard in Settmint
				ensure!(
					<setters::Module<T>>::total_positions(currency_id).standard.is_zero(),
					Error::<T>::ExistUnhandledStandard,
				);
			}

			// Open refund stage
			CanRefund::<T>::put(true);
			Self::deposit_event(Event::OpenRefund(<frame_system::Module<T>>::block_number()));
			Ok(().into())
		}

		/// Refund a basket of remaining reserve assets to caller
		///
		/// - `amount`: stable currency amount used to refund.
		#[pallet::weight(T::WeightInfo::refund_reserves(T::ReserveCurrencyIds::get().len() as u32))]
		#[transactional]
		pub fn refund_reserves(
			origin: OriginFor<T>,
			#[pallet::compact] amount: Balance,
		) -> DispatchResultWithPostInfo {
			let who = ensure_signed(origin)?;
			ensure!(Self::can_refund(), Error::<T>::CanNotRefund);

			let refund_ratio: Ratio = <T as Config>::SerpTreasury::get_standard_proportion(amount);
			let reserve_currency_ids = T::ReserveCurrencyIds::get();

			// burn caller's stable currency by SERP Treasury
			<T as Config>::SerpTreasury::burn_standard(&who, amount)?;

			let mut refund_assets: Vec<(CurrencyId, Balance)> = vec![];
			// refund reserves to caller by SERP Treasury
			for currency_id in reserve_currency_ids {
				let refund_amount =
					refund_ratio.saturating_mul_int(<T as Config>::SerpTreasury::get_total_reserves(currency_id));

				if !refund_amount.is_zero() {
					<T as Config>::SerpTreasury::withdraw_reserve(&who, currency_id, refund_amount)?;
					refund_assets.push((currency_id, refund_amount));
				}
			}

			Self::deposit_event(Event::Refund(who, amount, refund_assets));
			Ok(().into())
		}
	}
}

impl<T: Config> EmergencyShutdown for Pallet<T> {
	fn is_shutdown() -> bool {
		Self::is_shutdown()
	}
}
