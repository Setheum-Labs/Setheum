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

//! # Auction Manager Module
//!
//! ## Overview
//!
//! Auction the assets of the system to maintain the normal operation of the
//! business.
//! Auction types include:
//!   - `reserve auction`: sell reserve asset (Setter) to buy back SettCurrency.
//!   - `surplus auction`: sell SettCurrency surplus to buy back native coin.
//!   - `standard auction`: mint some native coin for sale to buy back Setter
//!     for burning excess standard

#![cfg_attr(not(feature = "std"), no_std)]
#![allow(clippy::unused_unit)]
#![allow(clippy::upper_case_acronyms)]

use frame_support::{pallet_prelude::*, transactional};
use frame_system::{
	offchain::{SendTransactionTypes, SubmitTransaction},
	pallet_prelude::*,
};
use orml_traits::{Auction, AuctionHandler, Change, MultiCurrency, OnNewBidResult};
use orml_utilities::{IterableStorageMapExtended, OffchainErr};
use primitives::{AuctionId, Balance, CurrencyId};
use sp_runtime::{
	offchain::{
		storage::StorageValueRef,
		storage_lock::{StorageLock, Time},
		Duration,
	},
	traits::{BlakeTwo256, CheckedDiv, Hash, Saturating, Zero},
	transaction_validity::{
		InvalidTransaction, TransactionPriority, TransactionSource, TransactionValidity, ValidTransaction,
	},
	DispatchError, DispatchResult, FixedPointNumber, RandomNumberGenerator, RuntimeDebug,
};
use sp_std::prelude::*;
use support::{AuctionManager, SerpTreasury, SerpTreasuryExtended, DEXManager, EmergencyShutdown, PriceProvider, Rate};

mod mock;
mod tests;
pub mod weights;

pub use module::*;
pub use weights::WeightInfo;

pub const OFFCHAIN_WORKER_DATA: &[u8] = b"setheum/auction-manager/data/";
pub const OFFCHAIN_WORKER_LOCK: &[u8] = b"setheum/auction-manager/lock/";
pub const OFFCHAIN_WORKER_MAX_ITERATIONS: &[u8] = b"setheum/auction-manager/max-iterations/";
pub const LOCK_DURATION: u64 = 100;
pub const DEFAULT_MAX_ITERATIONS: u32 = 1000;

/// Information of a reserve auction
#[cfg_attr(feature = "std", derive(PartialEq, Eq))]
#[derive(Encode, Decode, Clone, RuntimeDebug)]
pub struct ReserveAuctionItem<AccountId, BlockNumber> {
	/// Refund recipient for may receive refund
	refund_recipient: AccountId,
	/// Reserve type for sale
	currency_id: CurrencyId,
	/// Initial reserve amount for sale
	#[codec(compact)]
	initial_amount: Balance,
	/// Current reserve amount for sale
	#[codec(compact)]
	amount: Balance,
	/// Target sales amount of this auction
	/// if zero, reserve auction will never be reverse stage,
	/// otherwise, target amount is the actual payment amount of active
	/// bidder
	#[codec(compact)]
	target: Balance,
	/// Auction start time
	start_time: BlockNumber,
}

impl<AccountId, BlockNumber> ReserveAuctionItem<AccountId, BlockNumber> {
	/// Return the reserve auction will never be reverse stage
	fn always_forward(&self) -> bool {
		self.target.is_zero()
	}

	/// Return whether the reserve auction is in reverse stage at
	/// specific bid price
	fn in_reverse_stage(&self, bid_price: Balance) -> bool {
		!self.always_forward() && bid_price >= self.target
	}

	/// Return the actual number of stablecoins to be paid
	fn payment_amount(&self, bid_price: Balance) -> Balance {
		if self.always_forward() {
			bid_price
		} else {
			sp_std::cmp::min(self.target, bid_price)
		}
	}

	/// Return new reserve amount at specific last bid price and new bid
	/// price
	fn reserve_amount(&self, last_bid_price: Balance, new_bid_price: Balance) -> Balance {
		if self.in_reverse_stage(new_bid_price) && new_bid_price > last_bid_price {
			Rate::checked_from_rational(sp_std::cmp::max(last_bid_price, self.target), new_bid_price)
				.and_then(|n| n.checked_mul_int(self.amount))
				.unwrap_or(self.amount)
		} else {
			self.amount
		}
	}
}

/// Information of an standard auction
#[cfg_attr(feature = "std", derive(PartialEq, Eq))]
#[derive(Encode, Decode, Clone, RuntimeDebug)]
pub struct StandardAuctionItem<BlockNumber> {
	/// Initial amount of native currency for sale
	#[codec(compact)]
	initial_amount: Balance,
	/// Current amount of native currency for sale
	#[codec(compact)]
	amount: Balance,
	/// Fix amount of standard value(stable currency) which want to get by this
	/// auction
	#[codec(compact)]
	fix: Balance,
	/// Auction start time
	start_time: BlockNumber,
}

impl<BlockNumber> StandardAuctionItem<BlockNumber> {
	/// Return amount for sale at specific last bid price and new bid price
	fn amount_for_sale(&self, last_bid_price: Balance, new_bid_price: Balance) -> Balance {
		if new_bid_price > last_bid_price && new_bid_price > self.fix {
			Rate::checked_from_rational(sp_std::cmp::max(last_bid_price, self.fix), new_bid_price)
				.and_then(|n| n.checked_mul_int(self.amount))
				.unwrap_or(self.amount)
		} else {
			self.amount
		}
	}
}

/// Information of an surplus auction
#[cfg_attr(feature = "std", derive(PartialEq, Eq))]
#[derive(Encode, Decode, Clone, RuntimeDebug)]
pub struct SurplusAuctionItem<BlockNumber> {
	/// Fixed amount of surplus(stable currency) for sale
	#[codec(compact)]
	amount: Balance,
	/// Auction start time
	start_time: BlockNumber,
}

#[frame_support::pallet]
pub mod module {
	use super::*;

	#[pallet::config]
	pub trait Config: frame_system::Config + SendTransactionTypes<Call<Self>> {
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;

		#[pallet::constant]
		/// The minimum increment size of each bid compared to the previous one
		type MinimumIncrementSize: Get<Rate>;

		#[pallet::constant]
		/// The extended time for the auction to end after each successful bid
		type AuctionTimeToClose: Get<Self::BlockNumber>;

		#[pallet::constant]
		/// When the total duration of the auction exceeds this soft cap, push
		/// the auction to end more faster
		type AuctionDurationSoftCap: Get<Self::BlockNumber>;

		#[pallet::constant]
		/// The stable currency id
		type GetStableCurrencyId: Get<CurrencyId>;

		#[pallet::constant]
		/// The native currency id
		type GetNativeCurrencyId: Get<CurrencyId>;

		/// Currency to transfer assets
		type Currency: MultiCurrency<Self::AccountId, CurrencyId = CurrencyId, Balance = Balance>;

		/// Auction to manager the auction process
		type Auction: Auction<Self::AccountId, Self::BlockNumber, AuctionId = AuctionId, Balance = Balance>;

		/// SERP Treasury to escrow assets related to auction
		type SerpTreasury: SerpTreasuryExtended<Self::AccountId, Balance = Balance, CurrencyId = CurrencyId>;

		/// DEX to get exchange info
		type DEX: DEXManager<Self::AccountId, CurrencyId, Balance>;

		/// The price source of currencies
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
		/// The auction dose not exist
		AuctionNotExists,
		/// The reserve auction is in reverse stage now
		InReverseStage,
		/// Feed price is invalid
		InvalidFeedPrice,
		/// Must after system shutdown
		MustAfterShutdown,
		/// Bid price is invalid
		InvalidBidPrice,
		/// Invalid input amount
		InvalidAmount,
	}

	#[pallet::event]
	#[pallet::generate_deposit(pub(crate) fn deposit_event)]
	pub enum Event<T: Config> {
		/// Reserve auction created. \[auction_id, reserve_type,
		/// reserve_amount, target_bid_price\]
		NewReserveAuction(AuctionId, CurrencyId, Balance, Balance),
		/// Standard auction created. \[auction_id, initial_supply_amount,
		/// fix_payment_amount\]
		NewStandardAuction(AuctionId, Balance, Balance),
		/// Surplus auction created. \[auction_id, fix_surplus_amount\]
		NewSurplusAuction(AuctionId, Balance),
		/// Active auction cancelled. \[auction_id\]
		CancelAuction(AuctionId),
		/// Reserve auction dealt. \[auction_id, reserve_type,
		/// reserve_amount, winner, payment_amount\]
		ReserveAuctionDealt(AuctionId, CurrencyId, Balance, T::AccountId, Balance),
		/// Surplus auction dealt. \[auction_id, surplus_amount, winner,
		/// payment_amount\]
		SurplusAuctionDealt(AuctionId, Balance, T::AccountId, Balance),
		/// Standard auction dealt. \[auction_id, standard_currency_amount, winner,
		/// payment_amount\]
		StandardAuctionDealt(AuctionId, Balance, T::AccountId, Balance),
		/// Dex take reserve auction. \[auction_id, reserve_type,
		/// reserve_amount, turnover\]
		DEXTakeReserveAuction(AuctionId, CurrencyId, Balance, Balance),
	}

	/// Mapping from auction id to reserve auction info
	#[pallet::storage]
	#[pallet::getter(fn reserve_auctions)]
	pub type ReserveAuctions<T: Config> =
		StorageMap<_, Twox64Concat, AuctionId, ReserveAuctionItem<T::AccountId, T::BlockNumber>, OptionQuery>;

	/// Mapping from auction id to standard auction info
	#[pallet::storage]
	#[pallet::getter(fn standard_auctions)]
	pub type StandardAuctions<T: Config> =
		StorageMap<_, Twox64Concat, AuctionId, StandardAuctionItem<T::BlockNumber>, OptionQuery>;

	/// Mapping from auction id to surplus auction info
	#[pallet::storage]
	#[pallet::getter(fn surplus_auctions)]
	pub type SurplusAuctions<T: Config> =
		StorageMap<_, Twox64Concat, AuctionId, SurplusAuctionItem<T::BlockNumber>, OptionQuery>;

	/// Record of the total reserve amount of all active reserve auctions
	/// under specific reserve type ReserveType -> TotalAmount
	#[pallet::storage]
	#[pallet::getter(fn total_reserve_in_auction)]
	pub type TotalReserveInAuction<T: Config> = StorageMap<_, Twox64Concat, CurrencyId, Balance, ValueQuery>;

	/// Record of total target sales of all active reserve auctions
	#[pallet::storage]
	#[pallet::getter(fn total_target_in_auction)]
	pub type TotalTargetInAuction<T: Config> = StorageValue<_, Balance, ValueQuery>;

	/// Record of total fix amount of all active standard auctions
	#[pallet::storage]
	#[pallet::getter(fn total_standard_in_auction)]
	pub type TotalStandardInAuction<T: Config> = StorageValue<_, Balance, ValueQuery>;

	/// Record of total surplus amount of all active surplus auctions
	#[pallet::storage]
	#[pallet::getter(fn total_surplus_in_auction)]
	pub type TotalSurplusInAuction<T: Config> = StorageValue<_, Balance, ValueQuery>;

	#[pallet::pallet]
	pub struct Pallet<T>(PhantomData<T>);

	#[pallet::hooks]
	impl<T: Config> Hooks<T::BlockNumber> for Pallet<T> {
		/// Start offchain worker in order to submit unsigned tx to cancel
		/// active auction after system shutdown.
		fn offchain_worker(now: T::BlockNumber) {
			if T::EmergencyShutdown::is_shutdown() && sp_io::offchain::is_validator() {
				if let Err(e) = Self::_offchain_worker() {
					debug::info!(
						target: "auction-manager offchain worker",
						"cannot run offchain worker at {:?}: {:?}",
						now,
						e,
					);
				} else {
					debug::debug!(
						target: "auction-manager offchain worker",
						"offchain worker start at block: {:?} already done!",
						now,
					);
				}
			}
		}
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// Cancel active auction after system shutdown
		///
		/// The dispatch origin of this call must be _None_.
		#[pallet::weight(T::WeightInfo::cancel_reserve_auction())]
		#[transactional]
		pub fn cancel(origin: OriginFor<T>, id: AuctionId) -> DispatchResultWithPostInfo {
			ensure_none(origin)?;
			ensure!(T::EmergencyShutdown::is_shutdown(), Error::<T>::MustAfterShutdown);
			<Self as AuctionManager<T::AccountId>>::cancel_auction(id)?;
			Self::deposit_event(Event::CancelAuction(id));
			Ok(().into())
		}
	}

	#[pallet::validate_unsigned]
	impl<T: Config> ValidateUnsigned for Pallet<T> {
		type Call = Call<T>;
		fn validate_unsigned(_source: TransactionSource, call: &Self::Call) -> TransactionValidity {
			if let Call::cancel(auction_id) = call {
				if !T::EmergencyShutdown::is_shutdown() {
					return InvalidTransaction::Call.into();
				}

				if let Some(reserve_auction) = Self::reserve_auctions(auction_id) {
					if let Some((_, bid_price)) = Self::get_last_bid(*auction_id) {
						// if reserve auction is in reverse stage, shouldn't cancel
						if reserve_auction.in_reverse_stage(bid_price) {
							return InvalidTransaction::Stale.into();
						}
					}
				} else if !<SurplusAuctions<T>>::contains_key(auction_id)
					&& !<StandardAuctions<T>>::contains_key(auction_id)
				{
					return InvalidTransaction::Stale.into();
				}

				ValidTransaction::with_tag_prefix("AuctionManagerOffchainWorker")
					.priority(T::UnsignedPriority::get())
					.and_provides(auction_id)
					.longevity(64_u64)
					.propagate(true)
					.build()
			} else {
				InvalidTransaction::Call.into()
			}
		}
	}
}

impl<T: Config> Pallet<T> {
	fn get_last_bid(auction_id: AuctionId) -> Option<(T::AccountId, Balance)> {
		T::Auction::auction_info(auction_id).and_then(|auction_info| auction_info.bid)
	}

	fn submit_cancel_auction_tx(auction_id: AuctionId) {
		let call = Call::<T>::cancel(auction_id);
		if let Err(err) = SubmitTransaction::<T, Call<T>>::submit_unsigned_transaction(call.into()) {
			debug::info!(
				target: "auction-manager offchain worker",
				"submit unsigned auction cancel tx for \nAuctionId {:?} \nfailed: {:?}",
				auction_id,
				err,
			);
		}
	}

	fn _offchain_worker() -> Result<(), OffchainErr> {
		// acquire offchain worker lock.
		let lock_expiration = Duration::from_millis(LOCK_DURATION);
		let mut lock = StorageLock::<'_, Time>::with_deadline(&OFFCHAIN_WORKER_LOCK, lock_expiration);
		let mut guard = lock.try_lock().map_err(|_| OffchainErr::OffchainLock)?;

		let mut to_be_continue = StorageValueRef::persistent(&OFFCHAIN_WORKER_DATA);

		// get to_be_continue record,
		// if it exsits, iterator map storage start with previous key
		let (auction_type_num, start_key) = if let Some(Some((auction_type_num, last_iterator_previous_key))) =
			to_be_continue.get::<(u32, Vec<u8>)>()
		{
			(auction_type_num, Some(last_iterator_previous_key))
		} else {
			let random_seed = sp_io::offchain::random_seed();
			let mut rng = RandomNumberGenerator::<BlakeTwo256>::new(BlakeTwo256::hash(&random_seed[..]));
			(rng.pick_u32(2), None)
		};

		// get the max iterationns config
		let max_iterations = StorageValueRef::persistent(&OFFCHAIN_WORKER_MAX_ITERATIONS)
			.get::<u32>()
			.unwrap_or(Some(DEFAULT_MAX_ITERATIONS));

		debug::debug!(target: "auction-manager offchain worker", "max iterations is {:?}", max_iterations);

		// Randomly choose to start iterations to cancel reserve/surplus/standard
		// auctions
		match auction_type_num {
			0 => {
				let mut iterator =
					<StandardAuctions<T> as IterableStorageMapExtended<_, _>>::iter(max_iterations, start_key);
				while let Some((standard_auction_id, _)) = iterator.next() {
					Self::submit_cancel_auction_tx(standard_auction_id);
					guard.extend_lock().map_err(|_| OffchainErr::OffchainLock)?;
				}

				// if iteration for map storage finished, clear to be continue record
				// otherwise, update to be continue record
				if iterator.finished {
					to_be_continue.clear();
				} else {
					to_be_continue.set(&(auction_type_num, iterator.storage_map_iterator.previous_key));
				}
			}
			1 => {
				let mut iterator =
					<SurplusAuctions<T> as IterableStorageMapExtended<_, _>>::iter(max_iterations, start_key);
				while let Some((surplus_auction_id, _)) = iterator.next() {
					Self::submit_cancel_auction_tx(surplus_auction_id);
					guard.extend_lock().map_err(|_| OffchainErr::OffchainLock)?;
				}

				if iterator.finished {
					to_be_continue.clear();
				} else {
					to_be_continue.set(&(auction_type_num, iterator.storage_map_iterator.previous_key));
				}
			}
			_ => {
				let mut iterator =
					<ReserveAuctions<T> as IterableStorageMapExtended<_, _>>::iter(max_iterations, start_key);
				while let Some((reserve_auction_id, _)) = iterator.next() {
					if let (Some(reserve_auction), Some((_, last_bid_price))) = (
						Self::reserve_auctions(reserve_auction_id),
						Self::get_last_bid(reserve_auction_id),
					) {
						// if reserve auction has already been in reverse stage,
						// should skip it.
						if reserve_auction.in_reverse_stage(last_bid_price) {
							continue;
						}
					}
					Self::submit_cancel_auction_tx(reserve_auction_id);
					guard.extend_lock().map_err(|_| OffchainErr::OffchainLock)?;
				}

				if iterator.finished {
					to_be_continue.clear();
				} else {
					to_be_continue.set(&(auction_type_num, iterator.storage_map_iterator.previous_key));
				}
			}
		}

		// Consume the guard but **do not** unlock the underlying lock.
		guard.forget();

		Ok(())
	}

	fn cancel_surplus_auction(id: AuctionId, surplus_auction: SurplusAuctionItem<T::BlockNumber>) -> DispatchResult {
		// if there's bid
		if let Some((bidder, bid_price)) = Self::get_last_bid(id) {
			// refund native token to the bidder
			// TODO: transfer from RESERVED TREASURY instead of issuing
			T::Currency::deposit(T::GetNativeCurrencyId::get(), &bidder, bid_price)?;

			// decrease account ref of bidder
			frame_system::Module::<T>::dec_consumers(&bidder);
		}

		// decrease total surplus in auction
		TotalSurplusInAuction::<T>::mutate(|balance| *balance = balance.saturating_sub(surplus_auction.amount));

		Ok(())
	}

	fn cancel_standard_auction(id: AuctionId, standard_auction: StandardAuctionItem<T::BlockNumber>) -> DispatchResult {
		// if there's bid
		if let Some((bidder, _)) = Self::get_last_bid(id) {
			// refund stable token to the bidder
			T::SerpTreasury::issue_standard(&bidder, standard_auction.fix, false)?;

			// decrease account ref of bidder
			frame_system::Module::<T>::dec_consumers(&bidder);
		}

		// decrease total standard in auction
		TotalStandardInAuction::<T>::mutate(|balance| *balance = balance.saturating_sub(standard_auction.fix));

		Ok(())
	}

	fn cancel_reserve_auction(
		id: AuctionId,
		reserve_auction: ReserveAuctionItem<T::AccountId, T::BlockNumber>,
	) -> DispatchResult {
		let last_bid = Self::get_last_bid(id);

		// reserve auction must not be in reverse stage
		if let Some((_, bid_price)) = last_bid {
			ensure!(
				!reserve_auction.in_reverse_stage(bid_price),
				Error::<T>::InReverseStage,
			);
		}

		// calculate how much reserve to offset target in settle price
		let stable_currency_id = T::GetStableCurrencyId::get();
		let settle_price = T::PriceSource::get_relative_price(stable_currency_id, reserve_auction.currency_id)
			.ok_or(Error::<T>::InvalidFeedPrice)?;
		let confiscate_reserve_amount = if reserve_auction.always_forward() {
			reserve_auction.amount
		} else {
			sp_std::cmp::min(
				settle_price.saturating_mul_int(reserve_auction.target),
				reserve_auction.amount,
			)
		};
		let refund_reserve_amount = reserve_auction.amount.saturating_sub(confiscate_reserve_amount);

		// refund remaining reserve to `refund_recipient` from SERP Treasury
		T::SerpTreasury::withdraw_reserve(
			&reserve_auction.refund_recipient,
			reserve_auction.currency_id,
			refund_reserve_amount,
		)?;

		// if there's bid
		if let Some((bidder, bid_price)) = last_bid {
			// refund stable token to the bidder
			T::SerpTreasury::issue_standard(&bidder, bid_price, false)?;

			// decrease account ref of bidder
			frame_system::Module::<T>::dec_consumers(&bidder);
		}

		// decrease account ref of refund recipient
		frame_system::Module::<T>::dec_consumers(&reserve_auction.refund_recipient);

		// decrease total reserve and target in auction
		TotalReserveInAuction::<T>::mutate(reserve_auction.currency_id, |balance| {
			*balance = balance.saturating_sub(reserve_auction.amount)
		});
		TotalTargetInAuction::<T>::mutate(|balance| *balance = balance.saturating_sub(reserve_auction.target));

		Ok(())
	}

	/// Return `true` if price increment rate is greater than or equal to
	/// minimum.
	///
	/// Formula: new_price - last_price >=
	///     max(last_price, target_price) * minimum_increment
	fn check_minimum_increment(
		new_price: Balance,
		last_price: Balance,
		target_price: Balance,
		minimum_increment: Rate,
	) -> bool {
		if let (Some(target), Some(result)) = (
			minimum_increment.checked_mul_int(sp_std::cmp::max(target_price, last_price)),
			new_price.checked_sub(last_price),
		) {
			result >= target
		} else {
			false
		}
	}

	fn get_minimum_increment_size(now: T::BlockNumber, start_block: T::BlockNumber) -> Rate {
		if now >= start_block + T::AuctionDurationSoftCap::get() {
			// double the minimum increment size when reach soft cap
			T::MinimumIncrementSize::get().saturating_mul(Rate::saturating_from_integer(2))
		} else {
			T::MinimumIncrementSize::get()
		}
	}

	fn get_auction_time_to_close(now: T::BlockNumber, start_block: T::BlockNumber) -> T::BlockNumber {
		if now >= start_block + T::AuctionDurationSoftCap::get() {
			// halve the extended time of bid when reach soft cap
			T::AuctionTimeToClose::get()
				.checked_div(&2u32.into())
				.expect("cannot overflow with positive divisor; qed")
		} else {
			T::AuctionTimeToClose::get()
		}
	}

	/// Handles reserve auction new bid. Returns
	/// `Ok(new_auction_end_time)` if bid accepted.
	///
	/// Ensured atomic.
	#[transactional]
	pub fn reserve_auction_bid_handler(
		now: T::BlockNumber,
		id: AuctionId,
		new_bid: (T::AccountId, Balance),
		last_bid: Option<(T::AccountId, Balance)>,
	) -> sp_std::result::Result<T::BlockNumber, DispatchError> {
		let (new_bidder, new_bid_price) = new_bid;
		ensure!(!new_bid_price.is_zero(), Error::<T>::InvalidBidPrice);

		<ReserveAuctions<T>>::try_mutate_exists(
			id,
			|reserve_auction| -> sp_std::result::Result<T::BlockNumber, DispatchError> {
				let mut reserve_auction = reserve_auction.as_mut().ok_or(Error::<T>::AuctionNotExists)?;
				let last_bid_price = last_bid.clone().map_or(Zero::zero(), |(_, price)| price); // get last bid price

				// ensure new bid price is valid
				ensure!(
					Self::check_minimum_increment(
						new_bid_price,
						last_bid_price,
						reserve_auction.target,
						Self::get_minimum_increment_size(now, reserve_auction.start_time),
					),
					Error::<T>::InvalidBidPrice
				);

				let last_bidder = last_bid.as_ref().map(|(who, _)| who);

				let mut payment = reserve_auction.payment_amount(new_bid_price);

				// if there's bid before, return stablecoin from new bidder to last bidder
				if let Some(last_bidder) = last_bidder {
					let refund = reserve_auction.payment_amount(last_bid_price);
					T::Currency::transfer(T::GetStableCurrencyId::get(), &new_bidder, last_bidder, refund)?;

					payment = payment
						.checked_sub(refund)
						// This should never fail because new bid payment are always greater or equal to last bid
						// payment.
						.ok_or(Error::<T>::InvalidBidPrice)?;
				}

				// transfer remain payment from new bidder to SERP Treasury
				T::SerpTreasury::deposit_surplus(&new_bidder, payment)?;

				// if reserve auction will be in reverse stage, refund reserve to it's
				// origin from auction SERP Treasury
				if reserve_auction.in_reverse_stage(new_bid_price) {
					let new_reserve_amount = reserve_auction.reserve_amount(last_bid_price, new_bid_price);
					let refund_reserve_amount = reserve_auction.amount.saturating_sub(new_reserve_amount);

					if !refund_reserve_amount.is_zero() {
						T::SerpTreasury::withdraw_reserve(
							&(reserve_auction.refund_recipient),
							reserve_auction.currency_id,
							refund_reserve_amount,
						)?;

						// update total reserve in auction after refund
						TotalReserveInAuction::<T>::mutate(reserve_auction.currency_id, |balance| {
							*balance = balance.saturating_sub(refund_reserve_amount)
						});
						reserve_auction.amount = new_reserve_amount;
					}
				}

				Self::swap_bidders(&new_bidder, last_bidder);

				Ok(now + Self::get_auction_time_to_close(now, reserve_auction.start_time))
			},
		)
	}

	/// Handles standard auction new bid. Returns `Ok(new_auction_end_time)` if
	/// bid accepted.
	///
	/// Ensured atomic.
	#[transactional]
	pub fn standard_auction_bid_handler(
		now: T::BlockNumber,
		id: AuctionId,
		new_bid: (T::AccountId, Balance),
		last_bid: Option<(T::AccountId, Balance)>,
	) -> sp_std::result::Result<T::BlockNumber, DispatchError> {
		<StandardAuctions<T>>::try_mutate_exists(
			id,
			|standard_auction| -> sp_std::result::Result<T::BlockNumber, DispatchError> {
				let mut standard_auction = standard_auction.as_mut().ok_or(Error::<T>::AuctionNotExists)?;
				let (new_bidder, new_bid_price) = new_bid;
				let last_bid_price = last_bid.clone().map_or(Zero::zero(), |(_, price)| price); // get last bid price

				ensure!(
					Self::check_minimum_increment(
						new_bid_price,
						last_bid_price,
						standard_auction.fix,
						Self::get_minimum_increment_size(now, standard_auction.start_time),
					) && new_bid_price >= standard_auction.fix,
					Error::<T>::InvalidBidPrice,
				);

				let last_bidder = last_bid.as_ref().map(|(who, _)| who);

				if let Some(last_bidder) = last_bidder {
					// there's bid before, transfer the stablecoin from new bidder to last bidder
					T::Currency::transfer(
						T::GetStableCurrencyId::get(),
						&new_bidder,
						last_bidder,
						standard_auction.fix,
					)?;
				} else {
					// there's no bid before, transfer stablecoin to SERP Treasury
					T::SerpTreasury::deposit_surplus(&new_bidder, standard_auction.fix)?;
				}

				Self::swap_bidders(&new_bidder, last_bidder);

				standard_auction.amount = standard_auction.amount_for_sale(last_bid_price, new_bid_price);

				Ok(now + Self::get_auction_time_to_close(now, standard_auction.start_time))
			},
		)
	}

	/// Handles surplus auction new bid. Returns `Ok(new_auction_end_time)`
	/// if bid accepted.
	///
	/// Ensured atomic.
	#[transactional]
	pub fn surplus_auction_bid_handler(
		now: T::BlockNumber,
		id: AuctionId,
		new_bid: (T::AccountId, Balance),
		last_bid: Option<(T::AccountId, Balance)>,
	) -> sp_std::result::Result<T::BlockNumber, DispatchError> {
		let (new_bidder, new_bid_price) = new_bid;
		ensure!(!new_bid_price.is_zero(), Error::<T>::InvalidBidPrice);

		let surplus_auction = Self::surplus_auctions(id).ok_or(Error::<T>::AuctionNotExists)?;
		let last_bid_price = last_bid.clone().map_or(Zero::zero(), |(_, price)| price); // get last bid price
		let native_currency_id = T::GetNativeCurrencyId::get();

		ensure!(
			Self::check_minimum_increment(
				new_bid_price,
				last_bid_price,
				Zero::zero(),
				Self::get_minimum_increment_size(now, surplus_auction.start_time),
			),
			Error::<T>::InvalidBidPrice,
		);

		let last_bidder = last_bid.as_ref().map(|(who, _)| who);

		let burn_amount = if let Some(last_bidder) = last_bidder {
			// refund last bidder
			T::Currency::transfer(native_currency_id, &new_bidder, last_bidder, last_bid_price)?;
			new_bid_price.saturating_sub(last_bid_price)
		} else {
			new_bid_price
		};

		// burn remain native token from new bidder
		T::Currency::withdraw(native_currency_id, &new_bidder, burn_amount)?;

		Self::swap_bidders(&new_bidder, last_bidder);

		Ok(now + Self::get_auction_time_to_close(now, surplus_auction.start_time))
	}

	fn reserve_auction_end_handler(
		auction_id: AuctionId,
		reserve_auction: ReserveAuctionItem<T::AccountId, T::BlockNumber>,
		winner: Option<(T::AccountId, Balance)>,
	) {
		if let Some((bidder, bid_price)) = winner {
			let mut should_deal = true;

			// if bid_price doesn't reach target and trading with DEX will get better result
			if !reserve_auction.in_reverse_stage(bid_price)
				&& bid_price
					< T::DEX::get_swap_target_amount(
						&[reserve_auction.currency_id, T::GetStableCurrencyId::get()],
						reserve_auction.amount,
						None,
					)
					.unwrap_or_default()
			{
				// try swap reserve in auction with DEX to get stable
				if let Ok(stable_amount) = T::SerpTreasury::swap_exact_reserve_in_auction_to_stable(
					reserve_auction.currency_id,
					reserve_auction.amount,
					Zero::zero(),
					None,
				) {
					// swap successfully, will not deal
					should_deal = false;

					// refund stable currency to the last bidder, it shouldn't fail and affect the
					// process. but even it failed, just the winner did not get the bid price. it
					// can be fixed by treasury council.
					let _ = T::SerpTreasury::issue_standard(&bidder, bid_price, false);

					if reserve_auction.in_reverse_stage(stable_amount) {
						// refund extra stable currency to recipient
						let refund_amount = stable_amount
							.checked_sub(reserve_auction.target)
							.expect("ensured stable_amount > target; qed");
						// it shouldn't fail and affect the process.
						// but even it failed, just the winner did not get the refund amount. it can be
						// fixed by treasury council.
						let _ = T::SerpTreasury::issue_standard(&reserve_auction.refund_recipient, refund_amount, false);
					}

					Self::deposit_event(Event::DEXTakeReserveAuction(
						auction_id,
						reserve_auction.currency_id,
						reserve_auction.amount,
						stable_amount,
					));
				}
			}

			if should_deal {
				// transfer reserve to winner from SERP Treasury, it shouldn't fail and affect
				// the process. but even it failed, just the winner did not get the amount. it
				// can be fixed by treasury council.
				let _ = T::SerpTreasury::withdraw_reserve(
					&bidder,
					reserve_auction.currency_id,
					reserve_auction.amount,
				);

				let payment_amount = reserve_auction.payment_amount(bid_price);
				Self::deposit_event(Event::ReserveAuctionDealt(
					auction_id,
					reserve_auction.currency_id,
					reserve_auction.amount,
					bidder,
					payment_amount,
				));
			}
		} else {
			Self::deposit_event(Event::CancelAuction(auction_id));
		}

		// decrement recipient account reference
		frame_system::Module::<T>::dec_consumers(&reserve_auction.refund_recipient);

		// update auction records
		TotalReserveInAuction::<T>::mutate(reserve_auction.currency_id, |balance| {
			*balance = balance.saturating_sub(reserve_auction.amount)
		});
		TotalTargetInAuction::<T>::mutate(|balance| *balance = balance.saturating_sub(reserve_auction.target));
	}

	fn standard_auction_end_handler(
		auction_id: AuctionId,
		standard_auction: StandardAuctionItem<T::BlockNumber>,
		winner: Option<(T::AccountId, Balance)>,
	) {
		if let Some((bidder, _)) = winner {
			// issue native token to winner, it shouldn't fail and affect the process.
			// but even it failed, just the winner did not get the amount. it can be fixed
			// by treasury council. TODO: transfer from RESERVED TREASURY instead of issuing
			let _ = T::Currency::deposit(T::GetNativeCurrencyId::get(), &bidder, standard_auction.amount);

			Self::deposit_event(Event::StandardAuctionDealt(
				auction_id,
				standard_auction.amount,
				bidder,
				standard_auction.fix,
			));
		} else {
			Self::deposit_event(Event::CancelAuction(auction_id));
		}

		TotalStandardInAuction::<T>::mutate(|balance| *balance = balance.saturating_sub(standard_auction.fix));
	}

	fn surplus_auction_end_handler(
		auction_id: AuctionId,
		surplus_auction: SurplusAuctionItem<T::BlockNumber>,
		winner: Option<(T::AccountId, Balance)>,
	) {
		if let Some((bidder, bid_price)) = winner {
			// deposit unbacked stable token to winner by SERP Treasury, it shouldn't fail
			// and affect the process. but even it failed, just the winner did not get the
			// amount. it can be fixed by treasury council.
			let _ = T::SerpTreasury::issue_standard(&bidder, surplus_auction.amount, false);

			Self::deposit_event(Event::SurplusAuctionDealt(
				auction_id,
				surplus_auction.amount,
				bidder,
				bid_price,
			));
		} else {
			Self::deposit_event(Event::CancelAuction(auction_id));
		}

		TotalSurplusInAuction::<T>::mutate(|balance| *balance = balance.saturating_sub(surplus_auction.amount));
	}

	/// increment `new_bidder` reference and decrement `last_bidder`
	/// reference if any
	fn swap_bidders(new_bidder: &T::AccountId, last_bidder: Option<&T::AccountId>) {
		if frame_system::Module::<T>::inc_consumers(new_bidder).is_err() {
			// No providers for the locks. This is impossible under normal circumstances
			// since the funds that are under the lock will themselves be stored in the
			// account and therefore will need a reference.
			frame_support::debug::warn!(
				"Warning: Attempt to introduce lock consumer reference, yet no providers. \
				This is unexpected but should be safe."
			);
		}

		if let Some(who) = last_bidder {
			frame_system::Module::<T>::dec_consumers(who);
		}
	}
}

impl<T: Config> AuctionHandler<T::AccountId, Balance, T::BlockNumber, AuctionId> for Pallet<T> {
	fn on_new_bid(
		now: T::BlockNumber,
		id: AuctionId,
		new_bid: (T::AccountId, Balance),
		last_bid: Option<(T::AccountId, Balance)>,
	) -> OnNewBidResult<T::BlockNumber> {
		let bid_result = if <ReserveAuctions<T>>::contains_key(id) {
			Self::reserve_auction_bid_handler(now, id, new_bid, last_bid)
		} else if <StandardAuctions<T>>::contains_key(id) {
			Self::standard_auction_bid_handler(now, id, new_bid, last_bid)
		} else if <SurplusAuctions<T>>::contains_key(id) {
			Self::surplus_auction_bid_handler(now, id, new_bid, last_bid)
		} else {
			Err(Error::<T>::AuctionNotExists.into())
		};

		match bid_result {
			Ok(new_auction_end_time) => OnNewBidResult {
				accept_bid: true,
				auction_end_change: Change::NewValue(Some(new_auction_end_time)),
			},
			Err(_) => OnNewBidResult {
				accept_bid: false,
				auction_end_change: Change::NoChange,
			},
		}
	}

	fn on_auction_ended(id: AuctionId, winner: Option<(T::AccountId, Balance)>) {
		if let Some(reserve_auction) = <ReserveAuctions<T>>::take(id) {
			Self::reserve_auction_end_handler(id, reserve_auction, winner.clone());
		} else if let Some(standard_auction) = <StandardAuctions<T>>::take(id) {
			Self::standard_auction_end_handler(id, standard_auction, winner.clone());
		} else if let Some(surplus_auction) = <SurplusAuctions<T>>::take(id) {
			Self::surplus_auction_end_handler(id, surplus_auction, winner.clone());
		}

		if let Some((bidder, _)) = &winner {
			// decrease account ref of winner
			frame_system::Module::<T>::dec_consumers(bidder);
		}
	}
}

impl<T: Config> AuctionManager<T::AccountId> for Pallet<T> {
	type CurrencyId = CurrencyId;
	type Balance = Balance;
	type AuctionId = AuctionId;

	fn new_reserve_auction(
		refund_recipient: &T::AccountId,
		currency_id: Self::CurrencyId,
		amount: Self::Balance,
		target: Self::Balance,
	) -> DispatchResult {
		ensure!(!amount.is_zero(), Error::<T>::InvalidAmount);
		TotalReserveInAuction::<T>::try_mutate(currency_id, |total| -> DispatchResult {
			*total = total.checked_add(amount).ok_or(Error::<T>::InvalidAmount)?;
			Ok(())
		})?;

		if !target.is_zero() {
			// no-op if target is zero
			TotalTargetInAuction::<T>::try_mutate(|total| -> DispatchResult {
				*total = total.checked_add(target).ok_or(Error::<T>::InvalidAmount)?;
				Ok(())
			})?;
		}

		let start_time = <frame_system::Module<T>>::block_number();

		// do not set end time for reserve auction
		let auction_id = T::Auction::new_auction(start_time, None)?;

		<ReserveAuctions<T>>::insert(
			auction_id,
			ReserveAuctionItem {
				refund_recipient: refund_recipient.clone(),
				currency_id,
				initial_amount: amount,
				amount,
				target,
				start_time,
			},
		);

		// increment recipient account reference
		if frame_system::Module::<T>::inc_consumers(refund_recipient).is_err() {
			// No providers for the locks. This is impossible under normal circumstances
			// since the funds that are under the lock will themselves be stored in the
			// account and therefore will need a reference.
			frame_support::debug::warn!(
				"Warning: Attempt to introduce lock consumer reference, yet no providers. \
				This is unexpected but should be safe."
			);
		}

		Self::deposit_event(Event::NewReserveAuction(auction_id, currency_id, amount, target));
		Ok(())
	}

	fn new_standard_auction(initial_amount: Self::Balance, fix_standard: Self::Balance) -> DispatchResult {
		ensure!(
			!initial_amount.is_zero() && !fix_standard.is_zero(),
			Error::<T>::InvalidAmount,
		);
		TotalStandardInAuction::<T>::try_mutate(|total| -> DispatchResult {
			*total = total.checked_add(fix_standard).ok_or(Error::<T>::InvalidAmount)?;
			Ok(())
		})?;

		let start_time = <frame_system::Module<T>>::block_number();
		let end_block = start_time + T::AuctionTimeToClose::get();

		// set end time for standard auction
		let auction_id = T::Auction::new_auction(start_time, Some(end_block))?;

		<StandardAuctions<T>>::insert(
			auction_id,
			StandardAuctionItem {
				initial_amount,
				amount: initial_amount,
				fix: fix_standard,
				start_time,
			},
		);

		Self::deposit_event(Event::NewStandardAuction(auction_id, initial_amount, fix_standard));
		Ok(())
	}

	fn new_surplus_auction(amount: Self::Balance) -> DispatchResult {
		ensure!(!amount.is_zero(), Error::<T>::InvalidAmount,);
		TotalSurplusInAuction::<T>::try_mutate(|total| -> DispatchResult {
			*total = total.checked_add(amount).ok_or(Error::<T>::InvalidAmount)?;
			Ok(())
		})?;

		let start_time = <frame_system::Module<T>>::block_number();

		// do not set end time for surplus auction
		let auction_id = T::Auction::new_auction(start_time, None)?;

		<SurplusAuctions<T>>::insert(auction_id, SurplusAuctionItem { amount, start_time });

		Self::deposit_event(Event::NewSurplusAuction(auction_id, amount));
		Ok(())
	}

	fn cancel_auction(id: Self::AuctionId) -> DispatchResult {
		if let Some(reserve_auction) = <ReserveAuctions<T>>::take(id) {
			Self::cancel_reserve_auction(id, reserve_auction)?;
		} else if let Some(standard_auction) = <StandardAuctions<T>>::take(id) {
			Self::cancel_standard_auction(id, standard_auction)?;
		} else if let Some(surplus_auction) = <SurplusAuctions<T>>::take(id) {
			Self::cancel_surplus_auction(id, surplus_auction)?;
		} else {
			return Err(Error::<T>::AuctionNotExists.into());
		}
		T::Auction::remove_auction(id);
		Ok(())
	}

	fn get_total_reserve_in_auction(id: Self::CurrencyId) -> Self::Balance {
		Self::total_reserve_in_auction(id)
	}

	fn get_total_surplus_in_auction() -> Self::Balance {
		Self::total_surplus_in_auction()
	}

	fn get_total_standard_in_auction() -> Self::Balance {
		Self::total_standard_in_auction()
	}

	fn get_total_target_in_auction() -> Self::Balance {
		Self::total_target_in_auction()
	}
}
