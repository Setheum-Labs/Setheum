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

//! # Settway Module
//!
//! ## Overview
//!
//! The entry of the Settmint protocol for users, user can manipulate their Settmint
//! position to setter/payback, and can also authorize others to manage the their
//! Settmint under specific reserve type.

#![cfg_attr(not(feature = "std"), no_std)]
#![allow(clippy::unused_unit)]

use frame_support::{pallet_prelude::*, traits::ReservableCurrency, transactional};
use frame_system::pallet_prelude::*;
use primitives::{Amount, Balance, CurrencyId};
use sp_runtime::{
	traits::{StaticLookup, Zero},
	DispatchResult,
};
use sp_std::vec::Vec;

mod mock;
mod tests;
pub mod weights;

pub use module::*;
pub use weights::WeightInfo;

#[frame_support::pallet]
pub mod module {
	use super::*;

	#[pallet::config]
	pub trait Config: frame_system::Config + settmint_engine::Config {
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;

		/// Currency for authorization reserved.
		type Currency: ReservableCurrency<Self::AccountId, Balance = Balance>;

		/// Reserved amount per authorization.
		type DepositPerAuthorization: Get<Balance>;

		/// Weight information for the extrinsics in this module.
		type WeightInfo: WeightInfo;
	}

	#[pallet::error]
	pub enum Error<T> {
		// No authorization
		NoAuthorization,
	}

	#[pallet::event]
	#[pallet::generate_deposit(fn deposit_event)]
	pub enum Event<T: Config> {
		/// Authorize someone to operate the setter of specific reserve.
		/// \[authorizer, authorizee, reserve_type\]
		Authorization(T::AccountId, T::AccountId, CurrencyId),
		/// Cancel the authorization of specific reserve for someone.
		/// \[authorizer, authorizee, reserve_type\]
		UnAuthorization(T::AccountId, T::AccountId, CurrencyId),
		/// Cancel all authorization. \[authorizer\]
		UnAuthorizationAll(T::AccountId),
	}

	/// The authorization relationship map from
	/// Authorizer -> (CollateralType, Authorizee) -> Authorized
	///
	/// Authorization: double_map AccountId, (CurrencyId, T::AccountId) => Option<Balance>
	#[pallet::storage]
	#[pallet::getter(fn authorization)]
	pub type Authorization<T: Config> = StorageDoubleMap<
		_,
		Twox64Concat,
		T::AccountId,
		Blake2_128Concat,
		(CurrencyId, T::AccountId),
		Balance,
		OptionQuery,
	>;

	#[pallet::pallet]
	pub struct Pallet<T>(_);

	#[pallet::hooks]
	impl<T: Config> Hooks<T::BlockNumber> for Pallet<T> {}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// Adjust the standard of `currency_id` by specific
		/// `reserve_adjustment` and `standard_adjustment`
		///
		/// - `currency_id`: standard currency id.
		/// - `reserve_adjustment`: signed amount, positive means to deposit
		///   reserve currency into Settmint, negative means withdraw reserve
		///   currency from Settmint.
		/// - `standard_adjustment`: signed amount, positive means to issue some
		///   amount of `currency_id` to caller according to the standard adjustment,
		///   negative means caller will payback some amount of `currency_id` (standard settcurrency) to
		///   Settmint according to to the standard adjustment.
		#[pallet::weight(<T as Config>::WeightInfo::adjust_position())]
		#[transactional]
		pub fn adjust_position(
			origin: OriginFor<T>,
			currency_id: CurrencyId,
			reserve_adjustment: Amount,
			standard_adjustment: Amount,
		) -> DispatchResultWithPostInfo {
			let who = ensure_signed(origin)?;
			// ensure the currency is a settcurrency standard
			ensure!(
				T::StandardCurrencyIds::get().contains(&currency_id),
				Error::<T>::InvalidStandardType,
			);
			<settmint_engine::Module<T>>::adjust_position(&who, currency_id, reserve_adjustment, standard_adjustment)?;
			Ok(().into())
		}

		/// Transfer the whole Settmint of `from` under `currency_id` to caller's Settmint
		/// under the same `currency_id`, caller must have the authorization of
		/// `from` for the specific reserve type
		///
		/// - `currency_id`: reserve currency id.
		/// - `from`: authorizer account
		#[pallet::weight(<T as Config>::WeightInfo::transfer_position_from())]
		#[transactional]
		pub fn transfer_position_from(
			origin: OriginFor<T>,
			currency_id: CurrencyId,
			from: <T::Lookup as StaticLookup>::Source,
		) -> DispatchResultWithPostInfo {
			let to = ensure_signed(origin)?;
			let from = T::Lookup::lookup(from)?;
			Self::check_authorization(&from, &to, currency_id)?;
			<settmint_manager::Module<T>>::transfer_reserve(&from, &to, currency_id)?;
			Ok(().into())
		}

		/// Authorize `to` to manipulate the setter under `currency_id`
		///
		/// - `currency_id`: reserve currency id.
		/// - `to`: authorizee account
		#[pallet::weight(<T as Config>::WeightInfo::authorize())]
		#[transactional]
		pub fn authorize(
			origin: OriginFor<T>,
			currency_id: CurrencyId,
			to: <T::Lookup as StaticLookup>::Source,
		) -> DispatchResultWithPostInfo {
			let from = ensure_signed(origin)?;
			let to = T::Lookup::lookup(to)?;
			<Authorization<T>>::insert(&from, (currency_id, &to), true);
			Self::deposit_event(Event::Authorization(from, to, currency_id));
			Ok(().into())
		}

		/// Cancel the authorization for `to` under `currency_id`
		///
		/// - `currency_id`: reserve currency id.
		/// - `to`: authorizee account
		#[pallet::weight(<T as Config>::WeightInfo::unauthorize())]
		#[transactional]
		pub fn unauthorize(
			origin: OriginFor<T>,
			currency_id: CurrencyId,
			to: <T::Lookup as StaticLookup>::Source,
		) -> DispatchResultWithPostInfo {
			let from = ensure_signed(origin)?;
			let to = T::Lookup::lookup(to)?;
			<Authorization<T>>::remove(&from, (currency_id, &to));
			Self::deposit_event(Event::UnAuthorization(from, to, currency_id));
			Ok(().into())
		}

		/// Cancel all authorization of caller
		#[pallet::weight(<T as Config>::WeightInfo::unauthorize_all(<T as settmint_engine::Config>::ReserveCurrencyIds::get().len() as u32))]
		#[transactional]
		pub fn unauthorize_all(origin: OriginFor<T>) -> DispatchResultWithPostInfo {
			let from = ensure_signed(origin)?;
			<Authorization<T>>::remove_prefix(&from);
			Self::deposit_event(Event::UnAuthorizationAll(from));
			Ok(().into())
		}
	}
}

impl<T: Config> Pallet<T> {
	/// Check if `from` has the authorization of `to` under `currency_id`
	fn check_authorization(from: &T::AccountId, to: &T::AccountId, currency_id: CurrencyId) -> DispatchResult {
		ensure!(
			from == to || Self::authorization(from, (currency_id, to)),
			Error::<T>::NoAuthorization
		);
		Ok(())
	}
}
