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
//! The core module of Settmint protocol. Settmint engine is responsible for handling
//! internal processes of Settmint, including liquidation, settlement and risk
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
	SerpTreasury, SerpTreasuryExtended, DEXManager, EmergencyShutdown, ExchangeRate, Price, PriceProvider, Rate, Ratio,
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
	/// Maximum total standard value generated from it, when reach the hard
	/// cap, Settmint's owner cannot issue more stablecoin under the reserve
	/// type.
	pub maximum_total_standard_value: Balance,

	/// Extra stability fee rate, `None` value means not set
	pub stability_fee: Option<Rate>,

	/// Liquidation ratio, when the reserve ratio of
	/// Settmint under this reserve type is below the liquidation ratio, this
	/// Settmint is unsafe and can be liquidated. `None` value means not set
	pub liquidation_ratio: Option<Ratio>,

	/// Liquidation penalty rate, when liquidation occurs,
	/// Settmint will be deducted an additional penalty base on the product of
	/// penalty rate and standard value. `None` value means not set
	pub liquidation_penalty: Option<Rate>,

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

/// Liquidation strategy available
#[derive(Encode, Decode, Clone, RuntimeDebug, PartialEq, Eq)]
pub enum LiquidationStrategy {
	/// Liquidation Settmint's reserve by create reserve auction
	Auction,
	/// Liquidation Settmint's reserve by swap with DEX
	Exchange,
}

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
		/// The default liquidation ratio for all reserve types of Settmint
		type DefaultLiquidationRatio: Get<Ratio>;

		#[pallet::constant]
		/// The default standard exchange rate for all reserve types
		type DefaultStandardExchangeRate: Get<ExchangeRate>;

		#[pallet::constant]
		/// The default liquidation penalty rate when liquidate unsafe Settmint
		type DefaultLiquidationPenalty: Get<Rate>;

		#[pallet::constant]
		/// The minimum standard value to avoid standard dust
		type MinimumStandardValue: Get<Balance>;

		#[pallet::constant]
		/// Stablecoin currency id
		type GetStableCurrencyId: Get<CurrencyId>;

		#[pallet::constant]
		/// The max slippage allowed when liquidate an unsafe Settmint by swap with
		/// DEX
		type MaxSlippageSwapWithDEX: Get<Ratio>;

		/// The Settmint treasury to maintain bad standards and surplus generated by Settmint
		type SerpTreasury: SerpTreasuryExtended<Self::AccountId, Balance = Balance, CurrencyId = CurrencyId>;

		/// The price source of all types of currencies related to Settmint
		type PriceSource: PriceProvider<CurrencyId>;

		/// The DEX participating in liquidation
		type DEX: DEXManager<Self::AccountId, CurrencyId, Balance>;

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
		/// The total standard value of specific reserve type already exceed the
		/// hard cap
		ExceedStandardValueHardCap,
		/// The reserve ratio below the required reserve ratio
		BelowRequiredReserveRatio,
		/// The reserve ratio below the liquidation ratio
		BelowLiquidationRatio,
		/// The Settmint must be unsafe to be liquidated
		MustBeUnsafe,
		/// Invalid reserve type
		InvalidReserveType,
		/// Remain standard value in Settmint below the dust amount
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
		/// Liquidate the unsafe Settmint. \[reserve_type, owner,
		/// reserve_amount, bad_standard_value, liquidation_strategy\]
		LiquidateUnsafeSettmint(CurrencyId, T::AccountId, Balance, Balance, LiquidationStrategy),
		/// Settle the Settmint has standard. [reserve_type, owner]
		SettleSettmintInStandard(CurrencyId, T::AccountId),
		/// The stability fee for specific reserve type updated.
		/// \[reserve_type, new_stability_fee\]
		StabilityFeeUpdated(CurrencyId, Option<Rate>),
		/// The liquidation fee for specific reserve type updated.
		/// \[reserve_type, new_liquidation_ratio\]
		LiquidationRatioUpdated(CurrencyId, Option<Ratio>),
		/// The liquidation penalty rate for specific reserve type updated.
		/// \[reserve_type, new_liquidation_panelty\]
		LiquidationPenaltyUpdated(CurrencyId, Option<Rate>),
		/// The required reserve penalty rate for specific reserve type
		/// updated. \[reserve_type, new_required_reserve_ratio\]
		RequiredReserveRatioUpdated(CurrencyId, Option<Ratio>),
		/// The hard cap of total standard value for specific reserve type
		/// updated. \[reserve_type, new_total_standard_value\]
		MaximumTotalStandardValueUpdated(CurrencyId, Balance),
		/// The global stability fee for all types of reserve updated.
		/// \[new_global_stability_fee\]
		GlobalStabilityFeeUpdated(Rate),
	}

	/// Mapping from reserve type to its exchange rate of standard units and
	/// standard value
	#[pallet::storage]
	#[pallet::getter(fn standard_exchange_rate)]
	pub type StandardExchangeRate<T: Config> = StorageMap<_, Twox64Concat, CurrencyId, ExchangeRate, OptionQuery>;

	/// Global stability fee rate for all types of reserve
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
					liquidation_ratio,
					liquidation_penalty,
					required_reserve_ratio,
					maximum_total_standard_value,
				)| {
					ReserveParams::<T>::insert(
						currency_id,
						RiskManagementParams {
							maximum_total_standard_value: *maximum_total_standard_value,
							stability_fee: *stability_fee,
							liquidation_ratio: *liquidation_ratio,
							liquidation_penalty: *liquidation_penalty,
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
		/// Issue interest in stable currency for all types of reserve that has
		/// standard when block finalizes at `on-finalize`, and update their standard exchange rate
		fn on_finalize(_now: T::BlockNumber) {
			// collect stability fee for all types of reserve
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

						// issue stablecoin to surplus pool
						if <T as Config>::SerpTreasury::on_system_surplus(issued_stable_coin_balance).is_ok() {
							// update exchange rate when issue success
							let new_standard_exchange_rate =
								standard_exchange_rate.saturating_add(standard_exchange_rate_increment);
							StandardExchangeRate::<T>::insert(currency_id, new_standard_exchange_rate);
						}
					}
				}
			}
		}

		/// Runs after every block. Start offchain worker to check Settmint and
		/// submit unsigned tx to trigger liquidation or settlement.
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
		/// Liquidate unsafe Settmint
		///
		/// The dispatch origin of this call must be _None_.
		///
		/// - `currency_id`: Settmint's reserve type.
		/// - `who`: Settmint's owner.
		#[pallet::weight(T::WeightInfo::liquidate_by_dex())]
		#[transactional]
		pub fn liquidate(
			origin: OriginFor<T>,
			currency_id: CurrencyId,
			who: <T::Lookup as StaticLookup>::Source,
		) -> DispatchResultWithPostInfo {
			ensure_none(origin)?;
			let who = T::Lookup::lookup(who)?;
			ensure!(!T::EmergencyShutdown::is_shutdown(), Error::<T>::AlreadyShutdown);
			Self::liquidate_unsafe_settmint(who, currency_id)?;
			Ok(().into())
		}

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

		/// Update parameters related to risk management of Settmint under specific
		/// reserve type
		///
		/// The dispatch origin of this call must be `UpdateOrigin`.
		///
		/// - `currency_id`: reserve type.
		/// - `stability_fee`: extra stability fee rate, `None` means do not
		///   update, `Some(None)` means update it to `None`.
		/// - `liquidation_ratio`: liquidation ratio, `None` means do not
		///   update, `Some(None)` means update it to `None`.
		/// - `liquidation_penalty`: liquidation penalty, `None` means do not
		///   update, `Some(None)` means update it to `None`.
		/// - `required_reserve_ratio`: required reserve ratio, `None`
		///   means do not update, `Some(None)` means update it to `None`.
		/// - `maximum_total_standard_value`: maximum total standard value.
		#[pallet::weight((T::WeightInfo::set_reserve_params(), DispatchClass::Operational))]
		#[transactional]
		pub fn set_reserve_params(
			origin: OriginFor<T>,
			currency_id: CurrencyId,
			stability_fee: ChangeOptionRate,
			liquidation_ratio: ChangeOptionRatio,
			liquidation_penalty: ChangeOptionRate,
			required_reserve_ratio: ChangeOptionRatio,
			maximum_total_standard_value: ChangeBalance,
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
			if let Change::NewValue(update) = liquidation_ratio {
				reserve_params.liquidation_ratio = update;
				Self::deposit_event(Event::LiquidationRatioUpdated(currency_id, update));
			}
			if let Change::NewValue(update) = liquidation_penalty {
				reserve_params.liquidation_penalty = update;
				Self::deposit_event(Event::LiquidationPenaltyUpdated(currency_id, update));
			}
			if let Change::NewValue(update) = required_reserve_ratio {
				reserve_params.required_reserve_ratio = update;
				Self::deposit_event(Event::RequiredReserveRatioUpdated(currency_id, update));
			}
			if let Change::NewValue(val) = maximum_total_standard_value {
				reserve_params.maximum_total_standard_value = val;
				Self::deposit_event(Event::MaximumTotalStandardValueUpdated(currency_id, val));
			}
			ReserveParams::<T>::insert(currency_id, reserve_params);
			Ok(().into())
		}
	}

	#[pallet::validate_unsigned]
	impl<T: Config> ValidateUnsigned for Pallet<T> {
		type Call = Call<T>;

		fn validate_unsigned(_source: TransactionSource, call: &Self::Call) -> TransactionValidity {
			match call {
				Call::liquidate(currency_id, who) => {
					let account = T::Lookup::lookup(who.clone())?;
					let Position { reserve, standard } = <SettersOf<T>>::positions(currency_id, &account);
					if !Self::is_settmint_unsafe(*currency_id, reserve, standard) || T::EmergencyShutdown::is_shutdown() {
						return InvalidTransaction::Stale.into();
					}

					ValidTransaction::with_tag_prefix("SettmintEngineOffchainWorker")
						.priority(T::UnsignedPriority::get())
						.and_provides((<frame_system::Module<T>>::block_number(), currency_id, who))
						.longevity(64_u64)
						.propagate(true)
						.build()
				}
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
	fn submit_unsigned_liquidation_tx(currency_id: CurrencyId, who: T::AccountId) {
		let who = T::Lookup::unlookup(who);
		let call = Call::<T>::liquidate(currency_id, who.clone());
		if SubmitTransaction::<T, Call<T>>::submit_unsigned_transaction(call.into()).is_err() {
			debug::info!(
				target: "settmint-engine offchain worker",
				"submit unsigned liquidation tx for \nSettmint - AccountId {:?} CurrencyId {:?} \nfailed!",
				who, currency_id,
			);
		}
	}

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
			if !is_shutdown && Self::is_settmint_unsafe(currency_id, reserve, standard) {
				// liquidate unsafe Settmint before emergency shutdown occurs
				Self::submit_unsigned_liquidation_tx(currency_id, who);
			} else if is_shutdown && !standard.is_zero() {
				// settle Settmint with standard after emergency shutdown occurs.
				Self::submit_unsigned_settlement_tx(currency_id, who);
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

		// if iteration for map storage finished, clear to be continue record
		// otherwise, update to be continue record
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

	pub fn is_settmint_unsafe(currency_id: CurrencyId, reserve: Balance, standard: Balance) -> bool {
		let stable_currency_id = T::GetStableCurrencyId::get();

		if let Some(feed_price) = T::PriceSource::get_relative_price(currency_id, stable_currency_id) {
			let reserve_ratio = Self::calculate_reserve_ratio(currency_id, reserve, standard, feed_price);
			reserve_ratio < Self::get_liquidation_ratio(currency_id)
		} else {
			false
		}
	}

	pub fn maximum_total_standard_value(currency_id: CurrencyId) -> Balance {
		Self::reserve_params(currency_id).maximum_total_standard_value
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

	pub fn get_liquidation_ratio(currency_id: CurrencyId) -> Ratio {
		Self::reserve_params(currency_id)
			.liquidation_ratio
			.unwrap_or_else(T::DefaultLiquidationRatio::get)
	}

	pub fn get_liquidation_penalty(currency_id: CurrencyId) -> Rate {
		Self::reserve_params(currency_id)
			.liquidation_penalty
			.unwrap_or_else(T::DefaultLiquidationPenalty::get)
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

	// settle settmint that has standard when emergency shutdown occurs
	pub fn settle_settmint_has_standard(who: T::AccountId, currency_id: CurrencyId) -> DispatchResult {
		let Position { reserve, standard } = <SettersOf<T>>::positions(currency_id, &who);
		ensure!(!standard.is_zero(), Error::<T>::NoStandardValue);

		// confiscate reserve in settmint to settmint treasury
		// and decrease Settmint's standard to zero
		let settle_price: Price = T::PriceSource::get_relative_price(T::GetStableCurrencyId::get(), currency_id)
			.ok_or(Error::<T>::InvalidFeedPrice)?;
		let bad_standard_value = Self::get_standard_value(currency_id, standard);
		let confiscate_reserve_amount =
			sp_std::cmp::min(settle_price.saturating_mul_int(bad_standard_value), reserve);

		// confiscate reserve and all standard
		<SettersOf<T>>::confiscate_reserve_and_standard(&who, currency_id, confiscate_reserve_amount, standard)?;

		Self::deposit_event(Event::SettleSettmintInStandard(currency_id, who));
		Ok(())
	}

	// liquidate unsafe settmint
	pub fn liquidate_unsafe_settmint(who: T::AccountId, currency_id: CurrencyId) -> DispatchResult {
		let Position { reserve, standard } = <SettersOf<T>>::positions(currency_id, &who);

		// ensure the settmint is unsafe
		ensure!(
			Self::is_settmint_unsafe(currency_id, reserve, standard),
			Error::<T>::MustBeUnsafe
		);

		// confiscate all reserve and standard of unsafe settmint to settmint treasury
		<SettersOf<T>>::confiscate_reserve_and_standard(&who, currency_id, reserve, standard)?;

		let bad_standard_value = Self::get_standard_value(currency_id, standard);
		let target_stable_amount = Self::get_liquidation_penalty(currency_id).saturating_mul_acc_int(bad_standard_value);

		// try use reserve to swap enough native token in DEX when the price impact
		// is below the limit, otherwise create reserve auctions.
		let liquidation_strategy = (|| -> Result<LiquidationStrategy, DispatchError> {
			// swap exact stable with DEX in limit of price impact
			if let Ok(actual_supply_reserve) =
				<T as Config>::SerpTreasury::swap_reserve_not_in_auction_with_exact_stable(
					currency_id,
					target_stable_amount,
					reserve,
					Some(T::MaxSlippageSwapWithDEX::get()),
				) {
				// refund remain reserve to Settmint owner
				let refund_reserve_amount = reserve
					.checked_sub(actual_supply_reserve)
					.expect("swap succecced means reserve >= actual_supply_reserve; qed");

				<T as Config>::SerpTreasury::withdraw_reserve(&who, currency_id, refund_reserve_amount)?;

				return Ok(LiquidationStrategy::Exchange);
			}

			// create reserve auctions by settmint treasury
			<T as Config>::SerpTreasury::create_reserve_auctions(
				currency_id,
				reserve,
				target_stable_amount,
				who.clone(),
				true,
			)?;

			Ok(LiquidationStrategy::Auction)
		})()?;

		Self::deposit_event(Event::LiquidateUnsafeSettmint(
			currency_id,
			who,
			reserve,
			bad_standard_value,
			liquidation_strategy,
		));
		Ok(())
	}
}

impl<T: Config> RiskManager<T::AccountId, CurrencyId, Balance, Balance> for Pallet<T> {
	fn get_bad_standard_value(currency_id: CurrencyId, standard_balance: Balance) -> Balance {
		Self::get_standard_value(currency_id, standard_balance)
	}

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

			// check the liquidation ratio
			ensure!(
				reserve_ratio >= Self::get_liquidation_ratio(currency_id),
				Error::<T>::BelowLiquidationRatio
			);

			// check the minimum_standard_value
			ensure!(
				standard_value >= T::MinimumStandardValue::get(),
				Error::<T>::RemainStandardValueTooSmall,
			);
		}

		Ok(())
	}

	fn check_standard_cap(currency_id: CurrencyId, total_standard_balance: Balance) -> DispatchResult {
		let hard_cap = Self::maximum_total_standard_value(currency_id);
		let total_standard_value = Self::get_standard_value(currency_id, total_standard_balance);

		ensure!(total_standard_value <= hard_cap, Error::<T>::ExceedStandardValueHardCap,);

		Ok(())
	}
}
