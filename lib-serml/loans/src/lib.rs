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

//! # Loans Module
//!
//! ## Overview
//!
//! Loans module manages CDP's collateral assets and the debits backed by these
//! assets.

#![cfg_attr(not(feature = "std"), no_std)]
#![allow(clippy::unused_unit)]
#![allow(clippy::collapsible_if)]

use frame_support::{debug, pallet_prelude::*, transactional};
use frame_system::pallet_prelude::*;
use orml_traits::{Happened, MultiCurrency, MultiCurrencyExtended};
use primitives::{Amount, Balance, CurrencyId};
use sp_runtime::{
	traits::{AccountIdConversion, Convert, Zero},
	DispatchError, DispatchResult, ModuleId, RuntimeDebug,
};
use sp_std::{convert::TryInto, result};
use support::{CDPTreasury, RiskManager};

mod mock;
mod tests;

pub use module::*;

/// A collateralized debit position.
#[derive(Encode, Decode, Eq, PartialEq, Copy, Clone, RuntimeDebug, Default)]
pub struct Position {
	/// The amount of collateral.
	pub collateral: Balance,
	/// The amount of debit.
	pub debit: Balance,
}

#[frame_support::pallet]
pub mod module {
	use super::*;

	#[pallet::config]
	pub trait Config: frame_system::Config {
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;

		/// Convert debit amount under specific collateral type to debit
		/// value(SETR)
		type SetterConvert: Convert<(CurrencyId, Balance), Balance>;

		/// Convert debit amount under specific collateral type to debit
		/// value(SETUSD)
		type SetDollarConvert: Convert<(CurrencyId, Balance), Balance>;

		/// Convert debit amount under specific collateral type to debit
		/// value(SETEUR)
		type SetEuroConvert: Convert<(CurrencyId, Balance), Balance>;

		/// Currency type for deposit/withdraw collateral assets to/from loans
		/// module
		type Currency: MultiCurrencyExtended<
			Self::AccountId,
			CurrencyId = CurrencyId,
			Balance = Balance,
			Amount = Amount,
		>;

		/// The stable currency ids
		type StableCurrencyIds: Get<Vec<CurrencyId>>;

		#[pallet::constant]
		/// The Setter currency id, it should be SETR in Setheum.
		type SetterCurrencyId: Get<CurrencyId>;

		#[pallet::constant]
		/// The SETUSD currency id, it should be SETUSD in Setheum.
		type GetSetUSDCurrencyId: Get<CurrencyId>;

		#[pallet::constant]
		/// The SETEUR currency id, it should be SETEUR in Setheum.
		type GetSetEURCurrencyId: Get<CurrencyId>;

		/// Risk manager is used to limit the debit size of CDP
		type RiskManager: RiskManager<Self::AccountId, CurrencyId, Balance, Balance>;

		/// CDP treasury for issuing/burning stable currency adjust debit value
		/// adjustment
		type CDPTreasury: CDPTreasury<Self::AccountId, Balance = Balance, CurrencyId = CurrencyId>;

		/// The loan's module id, keep all collaterals of CDPs.
		#[pallet::constant]
		type ModuleId: Get<ModuleId>;

		/// Event handler which calls when update loan.
		type OnUpdateLoan: Happened<(Self::AccountId, CurrencyId, Amount, Balance)>;
	}

	#[pallet::error]
	pub enum Error<T> {
		AmountConvertFailed,
		DebitOverflow,
		DebitTooLow,
		CollateralOverflow,
		CollateralTooLow,
		InvalidCurrencyType,
	}

	#[pallet::event]
	#[pallet::generate_deposit(pub(crate) fn deposit_event)]
	#[pallet::metadata(T::AccountId = "AccountId")]
	pub enum Event<T: Config> {
		/// Position updated. \[owner, collateral_type, collateral_adjustment,
		/// debit_adjustment\]
		PositionUpdated(T::AccountId, CurrencyId, CurrencyId, Amount, Amount),
		/// Confiscate CDP's collateral assets and eliminate its debit. \[owner,
		/// collateral_type, confiscated_collateral_amount,
		/// deduct_debit_amount\]
		ConfiscateCollateralAndDebit(T::AccountId, CurrencyId, CurrencyId, Balance, Balance),
		/// Transfer loan. \[from, to, currency_id\]
		TransferLoan(T::AccountId, T::AccountId, CurrencyId, CurrencyId),
	}

	/// The collateralized debit positions for SETR, map from
	/// Owner -> CollateralType -> Position
	///
	/// SetterPositions: double_map CollateralType, AccountId => Position
	#[pallet::storage]
	#[pallet::getter(fn setter_positions)]
	pub type SetterPositions<T: Config> =
		StorageDoubleMap<_, Twox64Concat, CurrencyId, Twox64Concat, T::AccountId, Position, ValueQuery>;

	/// The collateralized debit positions for SETUSD, map from
	/// Owner -> CollateralType -> Position
	///
	/// SetDollarPositions: double_map CollateralType, AccountId => Position
	#[pallet::storage]
	#[pallet::getter(fn setdollar_positions)]
	pub type SetDollarPositions<T: Config> =
		StorageDoubleMap<_, Twox64Concat, CurrencyId, Twox64Concat, T::AccountId, Position, ValueQuery>;

	/// The collateralized debit positions for SETEUR, map from
	/// Owner -> CollateralType -> Position
	///
	/// SetEuroPositions: double_map CollateralType, AccountId => Position
	#[pallet::storage]
	#[pallet::getter(fn seteuro_positions)]
	pub type SetEuroPositions<T: Config> =
		StorageDoubleMap<_, Twox64Concat, CurrencyId, Twox64Concat, T::AccountId, Position, ValueQuery>;

	/// The total collateralized debit positions for SETR, map from
	/// CollateralType -> Position
	///
	/// TotalSetterPositions: CollateralType => Position
	#[pallet::storage]
	#[pallet::getter(fn total_setter_positions)]
	pub type TotalSetterPositions<T: Config> = StorageMap<_, Twox64Concat, CurrencyId, Position, ValueQuery>;

	/// The total collateralized debit positions for SETUSD, map from
	/// CollateralType -> Position
	///
	/// TotalSetDollarPositions: CollateralType => Position
	#[pallet::storage]
	#[pallet::getter(fn total_setdollar_positions)]
	pub type TotalSetDollarPositions<T: Config> = StorageMap<_, Twox64Concat, CurrencyId, Position, ValueQuery>;

	/// The total collateralized debit positions for SETEUR, map from
	/// CollateralType -> Position
	///
	/// TotalSetEuroPositions: CollateralType => Position
	#[pallet::storage]
	#[pallet::getter(fn total_seteuro_positions)]
	pub type TotalSetEuroPositions<T: Config> = StorageMap<_, Twox64Concat, CurrencyId, Position, ValueQuery>;

	#[pallet::pallet]
	pub struct Pallet<T>(PhantomData<T>);

	#[pallet::hooks]
	impl<T: Config> Hooks<T::BlockNumber> for Pallet<T> {}

	#[pallet::call]
	impl<T: Config> Pallet<T> {}
}

impl<T: Config> Pallet<T> {
	pub fn account_id() -> T::AccountId {
		T::ModuleId::get().into_account()
	}

	/// confiscate collateral and debit to cdp treasury.
	///
	/// Ensured atomic.
	#[transactional]
	pub fn confiscate_collateral_and_debit(
		who: &T::AccountId,
		collateral_currency_id: CurrencyId,
		stable_currency_id: CurrencyId,
		collateral_confiscate: Balance,
		debit_decrease: Balance,
	) -> DispatchResult {
		// convert balance type to amount type
		let collateral_adjustment = Self::amount_try_from_balance(collateral_confiscate)?;
		let debit_adjustment = Self::amount_try_from_balance(debit_decrease)?;

		ensure!(
			T::StableCurrencyIds::get().contains(&stable_currency_id),
			Error::<T>::InvalidCurrencyType,
		);

		// transfer collateral to cdp treasury
		T::CDPTreasury::deposit_collateral(&Self::account_id(), collateral_currency_id, collateral_confiscate)?;

		// deposit debit to cdp treasury
		let bad_debt_value = T::RiskManager::get_bad_debt_value(collateral_currency_id, stable_currency_id, debit_decrease);
		T::CDPTreasury::on_system_debit(stable_currency_id, bad_debt_value)?;

		// update loan
		Self::update_loan(
			&who,
			collateral_currency_id,
			stable_currency_id,
			collateral_adjustment.saturating_neg(),
			debit_adjustment.saturating_neg(),
		)?;

		Self::deposit_event(Event::ConfiscateCollateralAndDebit(
			who.clone(),
			collateral_currency_id,
			stable_currency_id,
			collateral_confiscate,
			debit_decrease,
		));
		Ok(())
	}

	/// adjust the position.
	///
	/// Ensured atomic.
	#[transactional]
	pub fn adjust_position(
		who: &T::AccountId,
		collateral_currency_id: CurrencyId,
		stable_currency_id: CurrencyId,
		collateral_adjustment: Amount,
		debit_adjustment: Amount,
	) -> DispatchResult {

		ensure!(
			T::StableCurrencyIds::get().contains(&stable_currency_id),
			Error::<T>::InvalidCurrencyType,
		);

		// mutate collateral and debit
		Self::update_loan(who, collateral_currency_id, stable_currency_id, collateral_adjustment, debit_adjustment)?;

		let collateral_balance_adjustment = Self::balance_try_from_amount_abs(collateral_adjustment)?;
		let debit_balance_adjustment = Self::balance_try_from_amount_abs(debit_adjustment)?;
		let module_account = Self::account_id();

		if collateral_adjustment.is_positive() {
			T::Currency::transfer(collateral_currency_id, who, &module_account, collateral_balance_adjustment)?;
		} else if collateral_adjustment.is_negative() {
			T::Currency::transfer(collateral_currency_id, &module_account, who, collateral_balance_adjustment)?;
		}


		if stable_currency_id == T::SetterCurrencyId::get() {
			if debit_adjustment.is_positive() {
				// check debit cap when increase debit
				T::RiskManager::check_debit_cap(collateral_currency_id, stable_currency_id, Self::total_setter_positions(collateral_currency_id).debit)?;

				// issue debit with collateral backed by cdp treasury
				T::CDPTreasury::issue_debit(who, stable_currency_id, T::SetterConvert::convert((collateral_currency_id, debit_balance_adjustment)), true)?;
			} else if debit_adjustment.is_negative() {
				// repay debit
				// burn debit by cdp treasury
				T::CDPTreasury::burn_debit(who, stable_currency_id, T::SetterConvert::convert((collateral_currency_id, debit_balance_adjustment)))?;
			}

			// ensure pass risk check
			let Position { collateral, debit } = Self::setter_positions(collateral_currency_id, who);
			T::RiskManager::check_position_valid(collateral_currency_id, stable_currency_id, collateral, debit)?;
		} else if stable_currency_id == T::GetSetUSDCurrencyId::get() {
			if debit_adjustment.is_positive() {
				// check debit cap when increase debit
				T::RiskManager::check_debit_cap(collateral_currency_id, stable_currency_id, Self::total_setdollar_positions(collateral_currency_id).debit)?;

				// issue debit with collateral backed by cdp treasury
				T::CDPTreasury::issue_debit(who, stable_currency_id, T::SetDollarConvert::convert((collateral_currency_id, debit_balance_adjustment)), true)?;
			} else if debit_adjustment.is_negative() {
				// repay debit
				// burn debit by cdp treasury
				T::CDPTreasury::burn_debit(who, stable_currency_id, T::SetDollarConvert::convert((collateral_currency_id, debit_balance_adjustment)))?;
			}

			// ensure pass risk check
			let Position { collateral, debit } = Self::setdollar_positions(collateral_currency_id, who);
			T::RiskManager::check_position_valid(collateral_currency_id, stable_currency_id, collateral, debit)?;
		} else {
			if debit_adjustment.is_positive() {
				// check debit cap when increase debit
				T::RiskManager::check_debit_cap(collateral_currency_id, stable_currency_id, Self::total_seteuro_positions(collateral_currency_id).debit)?;

				// issue debit with collateral backed by cdp treasury
				T::CDPTreasury::issue_debit(who, stable_currency_id, T::SetEuroConvert::convert((collateral_currency_id, debit_balance_adjustment)), true)?;
			} else if debit_adjustment.is_negative() {
				// repay debit
				// burn debit by cdp treasury
				T::CDPTreasury::burn_debit(who, stable_currency_id, T::SetEuroConvert::convert((collateral_currency_id, debit_balance_adjustment)))?;
			}

			// ensure pass risk check
			let Position { collateral, debit } = Self::seteuro_positions(collateral_currency_id, who);
			T::RiskManager::check_position_valid(collateral_currency_id, stable_currency_id, collateral, debit)?;
		}

		Self::deposit_event(Event::PositionUpdated(
			who.clone(),
			collateral_currency_id,
			stable_currency_id,
			collateral_adjustment,
			debit_adjustment,
		));
		Ok(())
	}

	/// transfer whole loan of `from` to `to`
	pub fn transfer_loan(
		from: &T::AccountId,
		to: &T::AccountId,
		collateral_currency_id: CurrencyId,
		stable_currency_id: CurrencyId
	) -> DispatchResult {

		ensure!(
			T::StableCurrencyIds::get().contains(&stable_currency_id),
			Error::<T>::InvalidCurrencyType,
		);

		if stable_currency_id == T::SetterCurrencyId::get() {
			// get `from` position data
			let Position { collateral, debit } = Self::setter_positions(collateral_currency_id, from);
	
			let Position {
				collateral: to_collateral,
				debit: to_debit,
			} = Self::setter_positions(collateral_currency_id, to);
			let new_to_collateral_balance = to_collateral
				.checked_add(collateral)
				.expect("existing collateral balance cannot overflow; qed");
			let new_to_debit_balance = to_debit
				.checked_add(debit)
				.expect("existing debit balance cannot overflow; qed");
	
			// check new position
			T::RiskManager::check_position_valid(collateral_currency_id, stable_currency_id, new_to_collateral_balance, new_to_debit_balance)?;
	
			// balance -> amount
			let collateral_adjustment = Self::amount_try_from_balance(collateral)?;
			let debit_adjustment = Self::amount_try_from_balance(debit)?;
	
			Self::update_loan(
				from,
				collateral_currency_id,
				stable_currency_id, 
				collateral_adjustment.saturating_neg(),
				debit_adjustment.saturating_neg(),
			)?;
			Self::update_loan(to, collateral_currency_id, stable_currency_id, collateral_adjustment, debit_adjustment)?;	
		} else if stable_currency_id == T::GetSetUSDCurrencyId::get() {
			// get `from` position data
			let Position { collateral, debit } = Self::setdollar_positions(collateral_currency_id, from);
	
			let Position {
				collateral: to_collateral,
				debit: to_debit,
			} = Self::setdollar_positions(collateral_currency_id, to);
			let new_to_collateral_balance = to_collateral
				.checked_add(collateral)
				.expect("existing collateral balance cannot overflow; qed");
			let new_to_debit_balance = to_debit
				.checked_add(debit)
				.expect("existing debit balance cannot overflow; qed");
	
			// check new position
			T::RiskManager::check_position_valid(collateral_currency_id, stable_currency_id, new_to_collateral_balance, new_to_debit_balance)?;
	
			// balance -> amount
			let collateral_adjustment = Self::amount_try_from_balance(collateral)?;
			let debit_adjustment = Self::amount_try_from_balance(debit)?;
	
			Self::update_loan(
				from,
				collateral_currency_id,
				stable_currency_id, 
				collateral_adjustment.saturating_neg(),
				debit_adjustment.saturating_neg(),
			)?;
			Self::update_loan(to, collateral_currency_id, stable_currency_id, collateral_adjustment, debit_adjustment)?;
		} else {
			// get `from` position data
			let Position { collateral, debit } = Self::seteuro_positions(collateral_currency_id, from);
	
			let Position {
				collateral: to_collateral,
				debit: to_debit,
			} = Self::seteuro_positions(collateral_currency_id, to);
			let new_to_collateral_balance = to_collateral
				.checked_add(collateral)
				.expect("existing collateral balance cannot overflow; qed");
			let new_to_debit_balance = to_debit
				.checked_add(debit)
				.expect("existing debit balance cannot overflow; qed");
	
			// check new position
			T::RiskManager::check_position_valid(collateral_currency_id, stable_currency_id, new_to_collateral_balance, new_to_debit_balance)?;
	
			// balance -> amount
			let collateral_adjustment = Self::amount_try_from_balance(collateral)?;
			let debit_adjustment = Self::amount_try_from_balance(debit)?;
	
			Self::update_loan(
				from,
				collateral_currency_id,
				stable_currency_id, 
				collateral_adjustment.saturating_neg(),
				debit_adjustment.saturating_neg(),
			)?;
			Self::update_loan(to, collateral_currency_id, stable_currency_id, collateral_adjustment, debit_adjustment)?;
		}

		Self::deposit_event(Event::TransferLoan(from.clone(), to.clone(), collateral_currency_id, stable_currency_id));
		Ok(())
	}

	/// mutate records of collaterals and debits
	fn update_loan(
		who: &T::AccountId,
		collateral_currency_id: CurrencyId,
		stable_currency_id: CurrencyId,
		collateral_adjustment: Amount,
		debit_adjustment: Amount,
	) -> DispatchResult {
		let collateral_balance = Self::balance_try_from_amount_abs(collateral_adjustment)?;
		let debit_balance = Self::balance_try_from_amount_abs(debit_adjustment)?;

		ensure!(
			T::StableCurrencyIds::get().contains(&stable_currency_id),
			Error::<T>::InvalidCurrencyType,
		);

		if stable_currency_id == T::SetterCurrencyId::get() {
			<SetterPositions<T>>::try_mutate_exists(collateral_currency_id, who, |may_be_position| -> DispatchResult {
				let mut p = may_be_position.take().unwrap_or_default();
				let new_collateral = if collateral_adjustment.is_positive() {
					p.collateral
						.checked_add(collateral_balance)
						.ok_or(Error::<T>::CollateralOverflow)
				} else {
					p.collateral
						.checked_sub(collateral_balance)
						.ok_or(Error::<T>::CollateralTooLow)
				}?;
				let new_debit = if debit_adjustment.is_positive() {
					p.debit.checked_add(debit_balance).ok_or(Error::<T>::DebitOverflow)
				} else {
					p.debit.checked_sub(debit_balance).ok_or(Error::<T>::DebitTooLow)
				}?;
	
				// increase account ref if new position
				if p.collateral.is_zero() && p.debit.is_zero() {
					if frame_system::Module::<T>::inc_consumers(who).is_err() {
						// No providers for the locks. This is impossible under normal circumstances
						// since the funds that are under the lock will themselves be stored in the
						// account and therefore will need a reference.
						debug::warn!(
							"Warning: Attempt to introduce lock consumer reference, yet no providers. \
							This is unexpected but should be safe."
						);
					}
				}
	
				p.collateral = new_collateral;
	
				T::OnUpdateLoan::happened(&(who.clone(), collateral_currency_id, debit_adjustment, p.debit));
				p.debit = new_debit;
	
				if p.collateral.is_zero() && p.debit.is_zero() {
					// decrease account ref if zero position
					frame_system::Module::<T>::dec_consumers(who);
	
					// remove position storage if zero position
					*may_be_position = None;
				} else {
					*may_be_position = Some(p);
				}
	
				Ok(())
			})?;
	
			TotalSetterPositions::<T>::try_mutate(collateral_currency_id, |total_positions| -> DispatchResult {
				total_positions.collateral = if collateral_adjustment.is_positive() {
					total_positions
						.collateral
						.checked_add(collateral_balance)
						.ok_or(Error::<T>::CollateralOverflow)
				} else {
					total_positions
						.collateral
						.checked_sub(collateral_balance)
						.ok_or(Error::<T>::CollateralTooLow)
				}?;
	
				total_positions.debit = if debit_adjustment.is_positive() {
					total_positions
						.debit
						.checked_add(debit_balance)
						.ok_or(Error::<T>::DebitOverflow)
				} else {
					total_positions
						.debit
						.checked_sub(debit_balance)
						.ok_or(Error::<T>::DebitTooLow)
				}?;
	
				Ok(())
			})
		} else if stable_currency_id == T::GetSetUSDCurrencyId::get() {
			<SetDollarPositions<T>>::try_mutate_exists(collateral_currency_id, who, |may_be_position| -> DispatchResult {
				let mut p = may_be_position.take().unwrap_or_default();
				let new_collateral = if collateral_adjustment.is_positive() {
					p.collateral
						.checked_add(collateral_balance)
						.ok_or(Error::<T>::CollateralOverflow)
				} else {
					p.collateral
						.checked_sub(collateral_balance)
						.ok_or(Error::<T>::CollateralTooLow)
				}?;
				let new_debit = if debit_adjustment.is_positive() {
					p.debit.checked_add(debit_balance).ok_or(Error::<T>::DebitOverflow)
				} else {
					p.debit.checked_sub(debit_balance).ok_or(Error::<T>::DebitTooLow)
				}?;
	
				// increase account ref if new position
				if p.collateral.is_zero() && p.debit.is_zero() {
					if frame_system::Module::<T>::inc_consumers(who).is_err() {
						// No providers for the locks. This is impossible under normal circumstances
						// since the funds that are under the lock will themselves be stored in the
						// account and therefore will need a reference.
						debug::warn!(
							"Warning: Attempt to introduce lock consumer reference, yet no providers. \
							This is unexpected but should be safe."
						);
					}
				}
	
				p.collateral = new_collateral;
	
				T::OnUpdateLoan::happened(&(who.clone(), collateral_currency_id, debit_adjustment, p.debit));
				p.debit = new_debit;
	
				if p.collateral.is_zero() && p.debit.is_zero() {
					// decrease account ref if zero position
					frame_system::Module::<T>::dec_consumers(who);
	
					// remove position storage if zero position
					*may_be_position = None;
				} else {
					*may_be_position = Some(p);
				}
	
				Ok(())
			})?;
	
			TotalSetDollarPositions::<T>::try_mutate(collateral_currency_id, |total_positions| -> DispatchResult {
				total_positions.collateral = if collateral_adjustment.is_positive() {
					total_positions
						.collateral
						.checked_add(collateral_balance)
						.ok_or(Error::<T>::CollateralOverflow)
				} else {
					total_positions
						.collateral
						.checked_sub(collateral_balance)
						.ok_or(Error::<T>::CollateralTooLow)
				}?;
	
				total_positions.debit = if debit_adjustment.is_positive() {
					total_positions
						.debit
						.checked_add(debit_balance)
						.ok_or(Error::<T>::DebitOverflow)
				} else {
					total_positions
						.debit
						.checked_sub(debit_balance)
						.ok_or(Error::<T>::DebitTooLow)
				}?;
	
				Ok(())
			})
		} else {
			<SetEuroPositions<T>>::try_mutate_exists(collateral_currency_id, who, |may_be_position| -> DispatchResult {
				let mut p = may_be_position.take().unwrap_or_default();
				let new_collateral = if collateral_adjustment.is_positive() {
					p.collateral
						.checked_add(collateral_balance)
						.ok_or(Error::<T>::CollateralOverflow)
				} else {
					p.collateral
						.checked_sub(collateral_balance)
						.ok_or(Error::<T>::CollateralTooLow)
				}?;
				let new_debit = if debit_adjustment.is_positive() {
					p.debit.checked_add(debit_balance).ok_or(Error::<T>::DebitOverflow)
				} else {
					p.debit.checked_sub(debit_balance).ok_or(Error::<T>::DebitTooLow)
				}?;
	
				// increase account ref if new position
				if p.collateral.is_zero() && p.debit.is_zero() {
					if frame_system::Module::<T>::inc_consumers(who).is_err() {
						// No providers for the locks. This is impossible under normal circumstances
						// since the funds that are under the lock will themselves be stored in the
						// account and therefore will need a reference.
						debug::warn!(
							"Warning: Attempt to introduce lock consumer reference, yet no providers. \
							This is unexpected but should be safe."
						);
					}
				}
	
				p.collateral = new_collateral;
	
				T::OnUpdateLoan::happened(&(who.clone(), collateral_currency_id, debit_adjustment, p.debit));
				p.debit = new_debit;
	
				if p.collateral.is_zero() && p.debit.is_zero() {
					// decrease account ref if zero position
					frame_system::Module::<T>::dec_consumers(who);
	
					// remove position storage if zero position
					*may_be_position = None;
				} else {
					*may_be_position = Some(p);
				}
	
				Ok(())
			})?;
	
			TotalSetEuroPositions::<T>::try_mutate(collateral_currency_id, |total_positions| -> DispatchResult {
				total_positions.collateral = if collateral_adjustment.is_positive() {
					total_positions
						.collateral
						.checked_add(collateral_balance)
						.ok_or(Error::<T>::CollateralOverflow)
				} else {
					total_positions
						.collateral
						.checked_sub(collateral_balance)
						.ok_or(Error::<T>::CollateralTooLow)
				}?;
	
				total_positions.debit = if debit_adjustment.is_positive() {
					total_positions
						.debit
						.checked_add(debit_balance)
						.ok_or(Error::<T>::DebitOverflow)
				} else {
					total_positions
						.debit
						.checked_sub(debit_balance)
						.ok_or(Error::<T>::DebitTooLow)
				}?;
	
				Ok(())
			})
		}
	}
}

impl<T: Config> Pallet<T> {
	/// Convert `Balance` to `Amount`.
	fn amount_try_from_balance(b: Balance) -> result::Result<Amount, Error<T>> {
		TryInto::<Amount>::try_into(b).map_err(|_| Error::<T>::AmountConvertFailed)
	}

	/// Convert the absolute value of `Amount` to `Balance`.
	fn balance_try_from_amount_abs(a: Amount) -> result::Result<Balance, Error<T>> {
		TryInto::<Balance>::try_into(a.saturating_abs()).map_err(|_| Error::<T>::AmountConvertFailed)
	}
}
