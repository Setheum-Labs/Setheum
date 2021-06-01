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

//! # Settmint Engine Module
//!
//! ## Overview
//!
//! The core module of the Settmint protocol.
//! The Settmint engine is responsible for handling
//! internal processes of Settmint, including settlement and risk
//! management.

#![cfg_attr(not(feature = "std"), no_std)]
#![allow(clippy::unused_unit)]
#![allow(clippy::upper_case_acronyms)]

use frame_support::{pallet_prelude::*, transactional};
use frame_system::{
	offchain::{SendTransactionTypes, SubmitTransaction},
	pallet_prelude::*,
};
use setters::Position;
use orml_traits::Change;
use orml_utilities::{IterableStorageDoubleMapExtended, OffchainErr};
use primitives::{Amount, Balance, CurrencyId};
use sp_runtime::{
	offchain::{
		storage::StorageValueRef,
		storage_lock::{StorageLock, Time},
		Duration,
	},
	traits::{BlakeTwo256, Bounded, Convert, Hash, Saturating, StaticLookup, Zero},
	transaction_validity::{
		InvalidTransaction, TransactionPriority, TransactionSource, TransactionValidity, ValidTransaction,
	},
	DispatchError, DispatchResult, FixedPointNumber, RandomNumberGenerator, RuntimeDebug,
};
use sp_std::prelude::*;
use support::{
	DEXManager, EmergencyShutdown, ExchangeRate, Price, PriceProvider, Rate, Ratio,
	RiskManager,
};

mod standard_exchange_rate_convertor;
mod mock;
mod tests;
pub mod weights;

pub use standard_exchange_rate_convertor::StandardExchangeRateConvertor;
pub use module::*;
pub use weights::WeightInfo;

pub const OFFCHAIN_WORKER_DATA: &[u8] = b"setheum/settmint-engine/data/";
pub const OFFCHAIN_WORKER_LOCK: &[u8] = b"setheum/settmint-engine/lock/";
pub const OFFCHAIN_WORKER_MAX_ITERATIONS: &[u8] = b"setheum/settmint-engine/max-iterations/";
pub const LOCK_DURATION: u64 = 100;
pub const DEFAULT_MAX_ITERATIONS: u32 = 1000;

pub type SettersOf<T> = setters::Module<T>;

/// Risk management params
#[derive(Encode, Decode, Clone, RuntimeDebug, PartialEq, Eq, Default)]
pub struct RiskManagementParams {
	/// Extra stability fee rate, `None` value means not set
	pub stability_fee: Option<Rate>,

	/// Required reserve ratio, if it's set, cannot adjust the position
	/// of Settmint so that the current reserve ratio is lower than the
	/// required reserve ratio. `None` value means not set
	pub required_reserve_ratio: Option<Ratio>,
}

// typedef to help polkadot.js disambiguate Change with different generic
// parameters
type ChangeOptionRate = Change<Option<Rate>>;
type ChangeOptionRatio = Change<Option<Ratio>>;
type ChangeBalance = Change<Balance>;

#[frame_support::pallet]
pub mod module {
	use super::*;

	#[pallet::config]
	pub trait Config: frame_system::Config + setters::Config + SendTransactionTypes<Call<Self>> {
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;

		/// The origin which may update risk management parameters. Root can
		/// always do this.
		type UpdateOrigin: EnsureOrigin<Self::Origin>;

		#[pallet::constant]
		/// The list of valid reserve currency types
		type ReserveCurrencyIds: Get<Vec<CurrencyId>>;

		#[pallet::constant]
		/// The default standard exchange rate for all reserve types
		type DefaultStandardExchangeRate: Get<ExchangeRate>;

		#[pallet::constant]
		/// The minimum standard value to avoid standard dust
		type MinimumStandardValue: Get<Balance>;

		#[pallet::constant]
		/// Stablecoin currency id
		type GetStableCurrencyId: Get<CurrencyId>;

		/// The price source of all types of currencies related to Settmint
		type PriceSource: PriceProvider<CurrencyId>;

		#[pallet::constant]
		/// A configuration for base priority of unsigned transactions.
		///
		/// This is exposed so that it can be tuned for particular runtime, when
		/// multiple modules send unsigned transactions.
		type UnsignedPriority: Get<TransactionPriority>;

		/// Emergency shutdown.
		type EmergencyShutdown: EmergencyShutdown;

		/// Weight information for the extrinsics in this module.
		type WeightInfo: WeightInfo;
	}

	#[pallet::error]
	pub enum Error<T> {
		/// The reserve ratio is below the required reserve ratio
		BelowRequiredReserveRatio,
		/// Invalid reserve type
		InvalidReserveType,
		/// Remaining standard value in Settmint below the dust amount
		RemainStandardValueTooSmall,
		/// Feed price is invalid
		InvalidFeedPrice,
		/// No standard value in Settmint so that it cannot be settled
		NoStandardValue,
		/// System has already been shutdown
		AlreadyShutdown,
		/// Must after system shutdown
		MustAfterShutdown,
	}

	#[pallet::event]
	#[pallet::generate_deposit(pub(crate) fn deposit_event)]
	pub enum Event<T: Config> {
		/// Settle the Settmint that has standard. [reserve_type, owner]
		SettleSettmintInStandard(CurrencyId, T::AccountId),
		/// No need to Settle the Settmint that has standard. [reserve_type, owner]
		NoNeedToSettle(CurrencyId, T::AccountId),
		/// The stability fee for the reserve type updated.
		/// \[reserve_type, new_stability_fee\]
		StabilityFeeUpdated(CurrencyId, Option<Rate>),
		/// The required reserve ratio for the reserve type (change to `standard type`)
		/// updated. \[reserve_type, new_required_reserve_ratio\]
		RequiredReserveRatioUpdated(CurrencyId, Option<Ratio>),
		/// The global stability fee for the reserve updated.
		/// \[new_global_stability_fee\]
		GlobalStabilityFeeUpdated(Rate),
	}

	/// Mapping from reserve type to its exchange rate of standard units and
	/// standard value
	#[pallet::storage]
	#[pallet::getter(fn standard_exchange_rate)]
	pub type StandardExchangeRate<T: Config> = StorageMap<_, Twox64Concat, CurrencyId, ExchangeRate, OptionQuery>;

	/// Global stability fee rate for reserve
	#[pallet::storage]
	#[pallet::getter(fn global_stability_fee)]
	pub type GlobalStabilityFee<T: Config> = StorageValue<_, Rate, ValueQuery>;

	/// Mapping from reserve type to its risk management params
	#[pallet::storage]
	#[pallet::getter(fn reserve_params)]
	pub type ReserveParams<T: Config> = StorageMap<_, Twox64Concat, CurrencyId, RiskManagementParams, ValueQuery>;

	#[pallet::genesis_config]
	pub struct GenesisConfig {
		#[allow(clippy::type_complexity)]
		pub reserves_params: Vec<(
			CurrencyId,
			Option<Rate>,
			Option<Ratio>,
			Option<Rate>,
			Option<Ratio>,
			Balance,
		)>,
		pub global_stability_fee: Rate,
	}

	#[cfg(feature = "std")]
	impl Default for GenesisConfig {
		fn default() -> Self {
			GenesisConfig {
				reserves_params: vec![],
				global_stability_fee: Default::default(),
			}
		}
	}

	#[pallet::genesis_build]
	impl<T: Config> GenesisBuild<T> for GenesisConfig {
		fn build(&self) {
			self.reserves_params.iter().for_each(
				|(
					currency_id,
					stability_fee,
					required_reserve_ratio,
				)| {
					ReserveParams::<T>::insert(
						currency_id,
						RiskManagementParams {
							stability_fee: *stability_fee,
							required_reserve_ratio: *required_reserve_ratio,
						},
					);
				},
			);
			GlobalStabilityFee::<T>::put(self.global_stability_fee);
		}
	}

	#[pallet::pallet]
	pub struct Pallet<T>(PhantomData<T>);

	#[pallet::hooks]
	impl<T: Config> Hooks<T::BlockNumber> for Pallet<T> {
		/// Issue interest in stable currency for the reserve that has
		/// standard when block finalizes at `on-finalize`, and update their standard exchange rate
		fn on_finalize(_now: T::BlockNumber) {
			// collect stability fee for the reserve
			if !T::EmergencyShutdown::is_shutdown() {
				for currency_id in T::ReserveCurrencyIds::get() {
					let standard_exchange_rate = Self::get_standard_exchange_rate(currency_id);
					let stability_fee_rate = Self::get_stability_fee(currency_id);
					let total_standards = <SettersOf<T>>::total_positions(currency_id).standard;
					if !stability_fee_rate.is_zero() && !total_standards.is_zero() {
						let standard_exchange_rate_increment = standard_exchange_rate.saturating_mul(stability_fee_rate);
						let total_standard_value = Self::get_standard_value(currency_id, total_standards);
						let issued_stable_coin_balance =
							standard_exchange_rate_increment.saturating_mul_int(total_standard_value);
					}
				}
			}
		}

		/// Runs after every block. Start offchain worker to check Settmint and
		/// submit unsigned tx to trigger settlement if in emergency shutdown.
		fn offchain_worker(now: T::BlockNumber) {
			if let Err(e) = Self::_offchain_worker() {
				debug::info!(
					target: "settmint-engine offchain worker",
					"cannot run offchain worker at {:?}: {:?}",
					now,
					e,
				);
			} else {
				debug::debug!(
					target: "settmint-engine offchain worker",
					"offchain worker start at block: {:?} already done!",
					now,
				);
			}
		}
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// Settle Settmint that has standard after system shutdown
		///
		/// The dispatch origin of this call must be _None_.
		///
		/// - `currency_id`: Settmint's reserve type.
		/// - `who`: Settmint's owner.
		#[pallet::weight(T::WeightInfo::settle())]
		#[transactional]
		pub fn settle(
			origin: OriginFor<T>,
			currency_id: CurrencyId,
			who: <T::Lookup as StaticLookup>::Source,
		) -> DispatchResultWithPostInfo {
			ensure_none(origin)?;
			let who = T::Lookup::lookup(who)?;
			ensure!(T::EmergencyShutdown::is_shutdown(), Error::<T>::MustAfterShutdown);
			Self::settle_settmint_has_standard(who, currency_id)?;
			Ok(().into())
		}

		/// Update global parameters related to risk management of Settmint
		///
		/// The dispatch origin of this call must be `UpdateOrigin`.
		///
		/// - `global_stability_fee`: global stability fee rate.
		#[pallet::weight((T::WeightInfo::set_global_params(), DispatchClass::Operational))]
		#[transactional]
		pub fn set_global_params(origin: OriginFor<T>, global_stability_fee: Rate) -> DispatchResultWithPostInfo {
			T::UpdateOrigin::ensure_origin(origin)?;
			GlobalStabilityFee::<T>::put(global_stability_fee);
			Self::deposit_event(Event::GlobalStabilityFeeUpdated(global_stability_fee));
			Ok(().into())
		}

		/// Update parameters related to risk management of Settmint under the reserve type
		///
		/// The dispatch origin of this call must be `UpdateOrigin`.
		///
		/// - `currency_id`: reserve type.
		/// - `stability_fee`: extra stability fee rate, `None` means do not
		///   update, `Some(None)` means update it to `None`.
		/// - `required_reserve_ratio`: required reserve ratio, `None`
		///   means do not update, `Some(None)` means update it to `None`.
		#[pallet::weight((T::WeightInfo::set_reserve_params(), DispatchClass::Operational))]
		#[transactional]
		pub fn set_reserve_params(
			origin: OriginFor<T>,
			currency_id: CurrencyId,
			stability_fee: ChangeOptionRate,
			required_reserve_ratio: ChangeOptionRatio,
		) -> DispatchResultWithPostInfo {
			T::UpdateOrigin::ensure_origin(origin)?;
			ensure!(
				T::ReserveCurrencyIds::get().contains(&currency_id),
				Error::<T>::InvalidReserveType,
			);

			let mut reserve_params = Self::reserve_params(currency_id);
			if let Change::NewValue(update) = stability_fee {
				reserve_params.stability_fee = update;
				Self::deposit_event(Event::StabilityFeeUpdated(currency_id, update));
			}
			if let Change::NewValue(update) = required_reserve_ratio {
				reserve_params.required_reserve_ratio = update;
				Self::deposit_event(Event::RequiredReserveRatioUpdated(currency_id, update));
			}
			ReserveParams::<T>::insert(currency_id, reserve_params);
			Ok(().into())
		}
	}

	#[pallet::validate_unsigned]
	impl<T: Config> ValidateUnsigned for Pallet<T> {
		type Call = Call<T>;

		/// make it an if else statement.
		fn validate_unsigned(_source: TransactionSource, call: &Self::Call) -> TransactionValidity {
			match call {
				Call::settle(currency_id, who) => {
					let account = T::Lookup::lookup(who.clone())?;
					let Position { standard, .. } = <SettersOf<T>>::positions(currency_id, account);
					if standard.is_zero() || !T::EmergencyShutdown::is_shutdown() {
						return InvalidTransaction::Stale.into();
					}

					ValidTransaction::with_tag_prefix("SettmintEngineOffchainWorker")
						.priority(T::UnsignedPriority::get())
						.and_provides((currency_id, who))
						.longevity(64_u64)
						.propagate(true)
						.build()
				}
				_ => InvalidTransaction::Call.into(),
			}
		}
	}
}

impl<T: Config> Pallet<T> {
	fn submit_unsigned_settlement_tx(currency_id: CurrencyId, who: T::AccountId) {
		let who = T::Lookup::unlookup(who);
		let call = Call::<T>::settle(currency_id, who.clone());
		if SubmitTransaction::<T, Call<T>>::submit_unsigned_transaction(call.into()).is_err() {
			debug::info!(
				target: "settmint-engine offchain worker",
				"submit unsigned settlement tx for \nSettmint - AccountId {:?} CurrencyId {:?} \nfailed!",
				who, currency_id,
			);
		}
	}

	fn _offchain_worker() -> Result<(), OffchainErr> {
		let reserve_currency_ids = T::ReserveCurrencyIds::get();
		if reserve_currency_ids.len().is_zero() {
			return Ok(());
		}

		// check if we are a potential validator
		if !sp_io::offchain::is_validator() {
			return Err(OffchainErr::NotValidator);
		}

		// acquire offchain worker lock
		let lock_expiration = Duration::from_millis(LOCK_DURATION);
		let mut lock = StorageLock::<'_, Time>::with_deadline(&OFFCHAIN_WORKER_LOCK, lock_expiration);
		let mut guard = lock.try_lock().map_err(|_| OffchainErr::OffchainLock)?;

		let reserve_currency_ids = T::ReserveCurrencyIds::get();
		let to_be_continue = StorageValueRef::persistent(&OFFCHAIN_WORKER_DATA);

		// get to_be_continue record
		let (reserve_position, start_key) =
			if let Some(Some((last_reserve_position, maybe_last_iterator_previous_key))) =
				to_be_continue.get::<(u32, Option<Vec<u8>>)>()
			{
				(last_reserve_position, maybe_last_iterator_previous_key)
			} else {
				let random_seed = sp_io::offchain::random_seed();
				let mut rng = RandomNumberGenerator::<BlakeTwo256>::new(BlakeTwo256::hash(&random_seed[..]));
				(
					rng.pick_u32(reserve_currency_ids.len().saturating_sub(1) as u32),
					None,
				)
			};

		// get the max iterationns config
		let max_iterations = StorageValueRef::persistent(&OFFCHAIN_WORKER_MAX_ITERATIONS)
			.get::<u32>()
			.unwrap_or(Some(DEFAULT_MAX_ITERATIONS));

		let currency_id = reserve_currency_ids[(reserve_position as usize)];
		let is_shutdown = T::EmergencyShutdown::is_shutdown();
		let mut map_iterator = <setters::Positions<T> as IterableStorageDoubleMapExtended<_, _, _>>::iter_prefix(
			currency_id,
			max_iterations,
			start_key.clone(),
		);

		let mut iteration_count = 0;
		let iteration_start_time = sp_io::offchain::timestamp();
		while let Some((who, Position { reserve, standard })) = map_iterator.next() {
			if is_shutdown && !standard.is_zero() {
				// settle Settmint with standard after emergency shutdown occurs.
				Self::submit_unsigned_settlement_tx(currency_id, who);
			} else {
				Self::deposit_event(Event::NoNeedToSettle(currency_id, who));
			}

			iteration_count += 1;

			// extend offchain worker lock
			guard.extend_lock().map_err(|_| OffchainErr::OffchainLock)?;
		}
		let iteration_end_time = sp_io::offchain::timestamp();
		debug::debug!(
			target: "settmint-engine offchain worker",
			"iteration info:\n max iterations is {:?}\n currency id: {:?}, start key: {:?}, iterate count: {:?}\n iteration start at: {:?}, end at: {:?}, execution time: {:?}\n",
			max_iterations,
			currency_id,
			start_key,
			iteration_count,
			iteration_start_time,
			iteration_end_time,
			iteration_end_time.diff(&iteration_start_time)
		);

		/// if iteration for map storage finished, clear `to_be_continue` record
		/// otherwise, update `to_be_continue` record
		if map_iterator.finished {
			let next_reserve_position =
				if reserve_position < reserve_currency_ids.len().saturating_sub(1) as u32 {
					reserve_position + 1
				} else {
					0
				};
			to_be_continue.set(&(next_reserve_position, Option::<Vec<u8>>::None));
		} else {
			to_be_continue.set(&(reserve_position, Some(map_iterator.map_iterator.previous_key)));
		}

		// Consume the guard but **do not** unlock the underlying lock.
		guard.forget();

		Ok(())
	}

	pub fn required_reserve_ratio(currency_id: CurrencyId) -> Option<Ratio> {
		Self::reserve_params(currency_id).required_reserve_ratio
	}

	pub fn get_stability_fee(currency_id: CurrencyId) -> Rate {
		Self::reserve_params(currency_id)
			.stability_fee
			.unwrap_or_default()
			.saturating_add(Self::global_stability_fee())
	}

	pub fn get_standard_exchange_rate(currency_id: CurrencyId) -> ExchangeRate {
		Self::standard_exchange_rate(currency_id).unwrap_or_else(T::DefaultStandardExchangeRate::get)
	}

	pub fn get_standard_value(currency_id: CurrencyId, standard_balance: Balance) -> Balance {
		crate::StandardExchangeRateConvertor::<T>::convert((currency_id, standard_balance))
	}

	pub fn calculate_reserve_ratio(
		currency_id: CurrencyId,
		reserve_balance: Balance,
		standard_balance: Balance,
		price: Price,
	) -> Ratio {
		let locked_reserve_value = price.saturating_mul_int(reserve_balance);
		let standard_value = Self::get_standard_value(currency_id, standard_balance);

		Ratio::checked_from_rational(locked_reserve_value, standard_value).unwrap_or_else(Rate::max_value)
	}

	pub fn adjust_position(
		who: &T::AccountId,
		currency_id: CurrencyId,
		reserve_adjustment: Amount,
		standard_adjustment: Amount,
	) -> DispatchResult {
		ensure!(
			T::ReserveCurrencyIds::get().contains(&currency_id),
			Error::<T>::InvalidReserveType,
		);
		<SettersOf<T>>::adjust_position(who, currency_id, reserve_adjustment, standard_adjustment)?;
		Ok(())
	}
}

impl<T: Config> RiskManager<T::AccountId, CurrencyId, Balance, Balance> for Pallet<T> {

	fn check_position_valid(
		currency_id: CurrencyId,
		reserve_balance: Balance,
		standard_balance: Balance,
	) -> DispatchResult {
		if !standard_balance.is_zero() {
			let standard_value = Self::get_standard_value(currency_id, standard_balance);
			let feed_price = <T as Config>::PriceSource::get_relative_price(currency_id, T::GetStableCurrencyId::get())
				.ok_or(Error::<T>::InvalidFeedPrice)?;
			let reserve_ratio =
				Self::calculate_reserve_ratio(currency_id, reserve_balance, standard_balance, feed_price);

			// check the required reserve ratio
			if let Some(required_reserve_ratio) = Self::required_reserve_ratio(currency_id) {
				ensure!(
					reserve_ratio >= required_reserve_ratio,
					Error::<T>::BelowRequiredReserveRatio
				);
			}

			// check the minimum_standard_value
			ensure!(
				standard_value >= T::MinimumStandardValue::get(),
				Error::<T>::RemainStandardValueTooSmall,
			);
		}

		Ok(())
	}
}
