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
//! system.
//! Auction types include:
//!   - `setter auction`: sell reserve asset (Setter) to buy back SettCurrency.
//!   - `serplus auction`: sell SettCurrency and Setter serplus to buy back native currency.
//!   - `diamond auction`: mint some NativeCurrency to buy back Setter.

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
use support::{AuctionManager, SerpTreasury, SerpTreasuryExtended, DEXManager, PriceProvider, Rate};

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

/// Information of a setter auction
#[cfg_attr(feature = "std", derive(PartialEq, Eq))]
#[derive(Encode, Decode, Clone, RuntimeDebug)]
pub struct SetterAuctionItem<AccountId, BlockNumber> {
	/// Refund recipient in case the system may pass refunds.
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
	/// if zero, setter auction will never be reverse stage,
	/// otherwise, target amount is the actual payment amount of active
	/// bidder
	#[codec(compact)]
	target: Balance,
	/// Auction start time
	start_time: BlockNumber,
}

/// TODO: Rename `SetterAuction` to `SetterAuction`.
/// Because the Sether is the only reserve currency in Settmint.
impl<AccountId, BlockNumber> SetterAuctionItem<AccountId, BlockNumber> {
	/// Return `true` if the setter auction will never be reverse stage.
	fn always_forward(&self) -> bool {
		self.target.is_zero()
	}

	/// Return whether the setter auction is in reverse stage at
	/// specific bid price
	fn in_reverse_stage(&self, bid_price: Balance) -> bool {
		!self.always_forward() && bid_price >= self.target
	}

	/// Return the actual number of settcurrency to be paid to the Serp.
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

/// Information of a diamond auction
#[cfg_attr(feature = "std", derive(PartialEq, Eq))]
#[derive(Encode, Decode, Clone, RuntimeDebug)]
pub struct DiamondAuctionItem<BlockNumber> {
	/// Initial amount of native currency for sale to buy back Setter stablecoin.
	#[codec(compact)]
	initial_amount: Balance,
	/// Current amount of native currency for sale to buy back Setter stablecoin.
	#[codec(compact)]
	amount: Balance,
	/// Fix amount of Setter stablecoin value needed to be got back by this auction.
	#[codec(compact)]
	fix: Balance,
	/// Auction start time
	start_time: BlockNumber,
}

impl<BlockNumber> DiamondAuctionItem<BlockNumber> {
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

/// Information of an serplus auction
#[cfg_attr(feature = "std", derive(PartialEq, Eq))]
#[derive(Encode, Decode, Clone, RuntimeDebug)]
pub struct SerplusAuctionItem<BlockNumber> {
	/// Fixed amount of serplus [serplus](stable currency) for sale to get back native currency.
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

		/// Currency identifier to transfer assets
		type Currency: MultiCurrency<Self::AccountId, CurrencyId = CurrencyId, Balance = Balance>;

		/// Auction identifier to manager the auction process
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

		/// Weight information for the extrinsics in this module.
		type WeightInfo: WeightInfo;
	}

	#[pallet::error]
	pub enum Error<T> {
		/// The auction dose not exist
		AuctionNotExists,
		/// The setter auction is in reverse stage now
		InReverseStage,
		/// Feed price is invalid
		InvalidFeedPrice,
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
		NewSetterAuction(AuctionId, CurrencyId, Balance, Balance),
		/// Diamond Auction created. \[auction_id, initial_supply_amount,
		/// fix_payment_amount\]
		NewDiamondAuction(AuctionId, Balance, Balance),
		/// serplus auction created. \[auction_id, fix_serplusamount\]
		NewSerplusAuction(AuctionId, Balance),
		/// Active auction cancelled. \[auction_id\]
		CancelAuction(AuctionId),
		/// Reserve auction dealt. \[auction_id, reserve_type,
		/// reserve_amount, winner, payment_amount\]
		SetterAuctionDealt(AuctionId, CurrencyId, Balance, T::AccountId, Balance),
		/// serplus auction dealt. \[auction_id, serplusamount, winner,
		/// payment_amount\]
		SerplusAuctionDealt(AuctionId, Balance, T::AccountId, Balance),
		/// Diamond Auction dealt. \[auction_id, standard_currency_amount, winner,
		/// payment_amount\]
		DiamondAuctionDealt(AuctionId, Balance, T::AccountId, Balance),
		/// Dex take setter auction. \[auction_id, reserve_type,
		/// reserve_amount, turnover\]
		DEXTakeSetterAuction(AuctionId, CurrencyId, Balance, Balance),
	}

	/// Mapping from auction id to setter auction info
	#[pallet::storage]
	#[pallet::getter(fn setter_auctions)]
	pub type SetterAuctions<T: Config> =
		StorageMap<_, Twox64Concat, AuctionId, SetterAuctionItem<T::AccountId, T::BlockNumber>, OptionQuery>;

	/// Mapping from auction id to diamond auction info
	#[pallet::storage]
	#[pallet::getter(fn diamond_auctions)]
	pub type DiamondAuctions<T: Config> =
		StorageMap<_, Twox64Concat, AuctionId, DiamondAuctionItem<T::BlockNumber>, OptionQuery>;

	/// Mapping from auction id to serplus auction info
	#[pallet::storage]
	#[pallet::getter(fn serplus_auctions)]
	pub type SerplusAuctions<T: Config> =
		StorageMap<_, Twox64Concat, AuctionId, SerplusAuctionItem<T::BlockNumber>, OptionQuery>;

	/// Record of the total reserve amount of all active setter auctions
	/// under specific reserve type ReserveType -> TotalAmount
	#[pallet::storage]
	#[pallet::getter(fn total_reserve_in_auction)]
	pub type TotalReserveInAuction<T: Config> = StorageMap<_, Twox64Concat, CurrencyId, Balance, ValueQuery>;

	/// Record of total target sales of all active setter auctions
	#[pallet::storage]
	#[pallet::getter(fn total_target_in_auction)]
	pub type TotalTargetInAuction<T: Config> = StorageValue<_, Balance, ValueQuery>;

	/// Record of total fix amount of all active diamond auctions
	#[pallet::storage]
	#[pallet::getter(fn total_standard_in_auction)]
	pub type TotalStandardInAuction<T: Config> = StorageValue<_, Balance, ValueQuery>;

	/// Record of total serplus amount of all active serplus auctions
	#[pallet::storage]
	#[pallet::getter(fn total_serplusin_auction)]
	pub type TotalSerplusInAuction<T: Config> = StorageValue<_, Balance, ValueQuery>;

	#[pallet::pallet]
	pub struct Pallet<T>(PhantomData<T>);

	#[pallet::hooks]
	impl<T: Config> Hooks<T::BlockNumber> for Pallet<T> {}

	#[pallet::call]
	impl<T: Config> Pallet<T> {}

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

		// Randomly choose to start iterations to cancel reserve/serplus/standard
		// auctions
		match auction_type_num {
			0 => {
				let mut iterator =
					<DiamondAuctions<T> as IterableStorageMapExtended<_, _>>::iter(max_iterations, start_key);
				while let Some((diamond_auction_id, _)) = iterator.next() {
					Self::submit_cancel_auction_tx(diamond_auction_id);
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
					<SerplusAuctions<T> as IterableStorageMapExtended<_, _>>::iter(max_iterations, start_key);
				while let Some((serplus_auction_id, _)) = iterator.next() {
					Self::submit_cancel_auction_tx(serplus_auction_id);
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
					<SetterAuctions<T> as IterableStorageMapExtended<_, _>>::iter(max_iterations, start_key);
				while let Some((setter_auction_id, _)) = iterator.next() {
					if let (Some(setter_auction), Some((_, last_bid_price))) = (
						Self::setter_auctions(setter_auction_id),
						Self::get_last_bid(setter_auction_id),
					) {
						// if setter auction has already been in reverse stage,
						// should skip it.
						if setter_auction.in_reverse_stage(last_bid_price) {
							continue;
						}
					}
					Self::submit_cancel_auction_tx(setter_auction_id);
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

	fn cancel_serplus_auction(id: AuctionId, serplus_auction: SerplusAuctionItem<T::BlockNumber>) -> DispatchResult {
		// if there's bid
		if let Some((bidder, bid_price)) = Self::get_last_bid(id) {
			// refund native token to the bidder
			// TODO: transfer from RESERVED TREASURY instead of issuing
			T::Currency::deposit(T::GetNativeCurrencyId::get(), &bidder, bid_price)?;

			// decrease account ref of bidder
			frame_system::Module::<T>::dec_consumers(&bidder);
		}

		// decrease total serplus in auction
		TotalSerplusInAuction::<T>::mutate(|balance| *balance = balance.saturating_sub(serplus_auction.amount));

		Ok(())
	}

	fn cancel_diamond_auction(id: AuctionId, diamond_auction: DiamondAuctionItem<T::BlockNumber>) -> DispatchResult {
		// if there's bid
		if let Some((bidder, _)) = Self::get_last_bid(id) {
			// refund stable token to the bidder
			T::SerpTreasury::issue_standard(&bidder, diamond_auction.fix, false)?;

			// decrease account ref of bidder
			frame_system::Module::<T>::dec_consumers(&bidder);
		}

		// decrease total standard in auction
		TotalStandardInAuction::<T>::mutate(|balance| *balance = balance.saturating_sub(diamond_auction.fix));

		Ok(())
	}

	fn cancel_setter_auction(
		id: AuctionId,
		setter_auction: SetterAuctionItem<T::AccountId, T::BlockNumber>,
	) -> DispatchResult {
		let last_bid = Self::get_last_bid(id);

		// setter auction must not be in reverse stage
		if let Some((_, bid_price)) = last_bid {
			ensure!(
				!setter_auction.in_reverse_stage(bid_price),
				Error::<T>::InReverseStage,
			);
		}

		// calculate how much reserve to offset target in settle price
		let stable_currency_id = T::GetStableCurrencyId::get();
		let settle_price = T::PriceSource::get_relative_price(stable_currency_id, setter_auction.currency_id)
			.ok_or(Error::<T>::InvalidFeedPrice)?;
		let confiscate_reserve_amount = if setter_auction.always_forward() {
			setter_auction.amount
		} else {
			sp_std::cmp::min(
				settle_price.saturating_mul_int(setter_auction.target),
				setter_auction.amount,
			)
		};
		let refund_reserve_amount = setter_auction.amount.saturating_sub(confiscate_reserve_amount);

		// refund remaining reserve to `refund_recipient` from SERP Treasury
		T::SerpTreasury::withdraw_reserve(
			&setter_auction.refund_recipient,
			setter_auction.currency_id,
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
		frame_system::Module::<T>::dec_consumers(&setter_auction.refund_recipient);

		// decrease total reserve and target in auction
		TotalReserveInAuction::<T>::mutate(setter_auction.currency_id, |balance| {
			*balance = balance.saturating_sub(setter_auction.amount)
		});
		TotalTargetInAuction::<T>::mutate(|balance| *balance = balance.saturating_sub(setter_auction.target));

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

	/// Handles setter auction new bid. Returns
	/// `Ok(new_auction_end_time)` if bid accepted.
	///
	/// Ensured atomic.
	#[transactional]
	pub fn setter_auction_bid_handler(
		now: T::BlockNumber,
		id: AuctionId,
		new_bid: (T::AccountId, Balance),
		last_bid: Option<(T::AccountId, Balance)>,
	) -> sp_std::result::Result<T::BlockNumber, DispatchError> {
		let (new_bidder, new_bid_price) = new_bid;
		ensure!(!new_bid_price.is_zero(), Error::<T>::InvalidBidPrice);

		<SetterAuctions<T>>::try_mutate_exists(
			id,
			|setter_auction| -> sp_std::result::Result<T::BlockNumber, DispatchError> {
				let mut setter_auction = setter_auction.as_mut().ok_or(Error::<T>::AuctionNotExists)?;
				let last_bid_price = last_bid.clone().map_or(Zero::zero(), |(_, price)| price); // get last bid price

				// ensure new bid price is valid
				ensure!(
					Self::check_minimum_increment(
						new_bid_price,
						last_bid_price,
						setter_auction.target,
						Self::get_minimum_increment_size(now, setter_auction.start_time),
					),
					Error::<T>::InvalidBidPrice
				);

				let last_bidder = last_bid.as_ref().map(|(who, _)| who);

				let mut payment = setter_auction.payment_amount(new_bid_price);

				// if there's bid before, return stablecoin from new bidder to last bidder
				if let Some(last_bidder) = last_bidder {
					let refund = setter_auction.payment_amount(last_bid_price);
					T::Currency::transfer(T::GetStableCurrencyId::get(), &new_bidder, last_bidder, refund)?;

					payment = payment
						.checked_sub(refund)
						// This should never fail because new bid payment are always greater or equal to last bid
						// payment.
						.ok_or(Error::<T>::InvalidBidPrice)?;
				}

				// transfer remain payment from new bidder to SERP Treasury
				T::SerpTreasury::deposit_serplus(&new_bidder, payment)?;

				// if setter auction will be in reverse stage, refund reserve to it's
				// origin from auction SERP Treasury
				if setter_auction.in_reverse_stage(new_bid_price) {
					let new_reserve_amount = setter_auction.reserve_amount(last_bid_price, new_bid_price);
					let refund_reserve_amount = setter_auction.amount.saturating_sub(new_reserve_amount);

					if !refund_reserve_amount.is_zero() {
						T::SerpTreasury::withdraw_reserve(
							&(setter_auction.refund_recipient),
							setter_auction.currency_id,
							refund_reserve_amount,
						)?;

						// update total reserve in auction after refund
						TotalReserveInAuction::<T>::mutate(setter_auction.currency_id, |balance| {
							*balance = balance.saturating_sub(refund_reserve_amount)
						});
						setter_auction.amount = new_reserve_amount;
					}
				}

				Self::swap_bidders(&new_bidder, last_bidder);

				Ok(now + Self::get_auction_time_to_close(now, setter_auction.start_time))
			},
		)
	}

	/// Handles diamond auction new bid. Returns `Ok(new_auction_end_time)` if
	/// bid accepted.
	///
	/// Ensured atomic.
	#[transactional]
	pub fn diamond_auction_bid_handler(
		now: T::BlockNumber,
		id: AuctionId,
		new_bid: (T::AccountId, Balance),
		last_bid: Option<(T::AccountId, Balance)>,
	) -> sp_std::result::Result<T::BlockNumber, DispatchError> {
		<DiamondAuctions<T>>::try_mutate_exists(
			id,
			|diamond_auction| -> sp_std::result::Result<T::BlockNumber, DispatchError> {
				let mut diamond_auction = diamond_auction.as_mut().ok_or(Error::<T>::AuctionNotExists)?;
				let (new_bidder, new_bid_price) = new_bid;
				let last_bid_price = last_bid.clone().map_or(Zero::zero(), |(_, price)| price); // get last bid price

				ensure!(
					Self::check_minimum_increment(
						new_bid_price,
						last_bid_price,
						diamond_auction.fix,
						Self::get_minimum_increment_size(now, diamond_auction.start_time),
					) && new_bid_price >= diamond_auction.fix,
					Error::<T>::InvalidBidPrice,
				);

				let last_bidder = last_bid.as_ref().map(|(who, _)| who);

				if let Some(last_bidder) = last_bidder {
					// there's bid before, transfer the stablecoin from new bidder to last bidder
					T::Currency::transfer(
						T::GetStableCurrencyId::get(),
						&new_bidder,
						last_bidder,
						diamond_auction.fix,
					)?;
				} else {
					// there's no bid before, transfer stablecoin to SERP Treasury
					T::SerpTreasury::deposit_serplus(&new_bidder, diamond_auction.fix)?;
				}

				Self::swap_bidders(&new_bidder, last_bidder);

				diamond_auction.amount = diamond_auction.amount_for_sale(last_bid_price, new_bid_price);

				Ok(now + Self::get_auction_time_to_close(now, diamond_auction.start_time))
			},
		)
	}

	/// Handles serplus auction new bid. Returns `Ok(new_auction_end_time)`
	/// if bid accepted.
	///
	/// Ensured atomic.
	#[transactional]
	pub fn serplus_auction_bid_handler(
		now: T::BlockNumber,
		id: AuctionId,
		new_bid: (T::AccountId, Balance),
		last_bid: Option<(T::AccountId, Balance)>,
	) -> sp_std::result::Result<T::BlockNumber, DispatchError> {
		let (new_bidder, new_bid_price) = new_bid;
		ensure!(!new_bid_price.is_zero(), Error::<T>::InvalidBidPrice);

		let serplus_auction = Self::serplus_auctions(id).ok_or(Error::<T>::AuctionNotExists)?;
		let last_bid_price = last_bid.clone().map_or(Zero::zero(), |(_, price)| price); // get last bid price
		let native_currency_id = T::GetNativeCurrencyId::get();

		ensure!(
			Self::check_minimum_increment(
				new_bid_price,
				last_bid_price,
				Zero::zero(),
				Self::get_minimum_increment_size(now, serplus_auction.start_time),
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

		Ok(now + Self::get_auction_time_to_close(now, serplus_auction.start_time))
	}

	fn setter_auction_end_handler(
		auction_id: AuctionId,
		setter_auction: SetterAuctionItem<T::AccountId, T::BlockNumber>,
		winner: Option<(T::AccountId, Balance)>,
	) {
		if let Some((bidder, bid_price)) = winner {
			let mut should_deal = true;

			// if bid_price doesn't reach target and trading with DEX will get better result
			if !setter_auction.in_reverse_stage(bid_price)
				&& bid_price
					< T::DEX::get_swap_target_amount(
						&[setter_auction.currency_id, T::GetStableCurrencyId::get()],
						setter_auction.amount,
						None,
					)
					.unwrap_or_default()
			{
				// try swap reserve in auction with DEX to get stable
				if let Ok(stable_amount) = T::SerpTreasury::swap_exact_reserve_in_auction_to_stable(
					setter_auction.currency_id,
					setter_auction.amount,
					Zero::zero(),
					None,
				) {
					// swap successfully, will not deal
					should_deal = false;

					// refund stable currency to the last bidder, it shouldn't fail and affect the
					// process. but even it failed, just the winner did not get the bid price. it
					// can be fixed by treasury council.
					let _ = T::SerpTreasury::issue_standard(&bidder, bid_price, false);

					if setter_auction.in_reverse_stage(stable_amount) {
						// refund extra stable currency to recipient
						let refund_amount = stable_amount
							.checked_sub(setter_auction.target)
							.expect("ensured stable_amount > target; qed");
						// it shouldn't fail and affect the process.
						// but even it failed, just the winner did not get the refund amount. it can be
						// fixed by treasury council.
						let _ = T::SerpTreasury::issue_standard(&setter_auction.refund_recipient, refund_amount, false);
					}

					Self::deposit_event(Event::DEXTakeSetterAuction(
						auction_id,
						setter_auction.currency_id,
						setter_auction.amount,
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
					setter_auction.currency_id,
					setter_auction.amount,
				);

				let payment_amount = setter_auction.payment_amount(bid_price);
				Self::deposit_event(Event::SetterAuctionDealt(
					auction_id,
					setter_auction.currency_id,
					setter_auction.amount,
					bidder,
					payment_amount,
				));
			}
		} else {
			Self::deposit_event(Event::CancelAuction(auction_id));
		}

		// decrement recipient account reference
		frame_system::Module::<T>::dec_consumers(&setter_auction.refund_recipient);

		// update auction records
		TotalReserveInAuction::<T>::mutate(setter_auction.currency_id, |balance| {
			*balance = balance.saturating_sub(setter_auction.amount)
		});
		TotalTargetInAuction::<T>::mutate(|balance| *balance = balance.saturating_sub(setter_auction.target));
	}

	fn diamond_auction_end_handler(
		auction_id: AuctionId,
		diamond_auction: DiamondAuctionItem<T::BlockNumber>,
		winner: Option<(T::AccountId, Balance)>,
	) {
		if let Some((bidder, _)) = winner {
			// issue native token to winner, it shouldn't fail and affect the process.
			// but even it failed, just the winner did not get the amount. it can be fixed
			// by treasury council. TODO: transfer from RESERVED TREASURY instead of issuing
			let _ = T::Currency::deposit(T::GetNativeCurrencyId::get(), &bidder, diamond_auction.amount);

			Self::deposit_event(Event::DiamondAuctionDealt(
				auction_id,
				diamond_auction.amount,
				bidder,
				diamond_auction.fix,
			));
		} else {
			Self::deposit_event(Event::CancelAuction(auction_id));
		}

		TotalStandardInAuction::<T>::mutate(|balance| *balance = balance.saturating_sub(diamond_auction.fix));
	}

	fn serplus_auction_end_handler(
		auction_id: AuctionId,
		serplus_auction: SerplusAuctionItem<T::BlockNumber>,
		winner: Option<(T::AccountId, Balance)>,
	) {
		if let Some((bidder, bid_price)) = winner {
			// deposit unbacked stable token to winner by SERP Treasury, it shouldn't fail
			// and affect the process. but even it failed, just the winner did not get the
			// amount. it can be fixed by treasury council.
			let _ = T::SerpTreasury::issue_standard(&bidder, serplus_auction.amount, false);

			Self::deposit_event(Event::SerplusAuctionDealt(
				auction_id,
				serplus_auction.amount,
				bidder,
				bid_price,
			));
		} else {
			Self::deposit_event(Event::CancelAuction(auction_id));
		}

		TotalSerplusInAuction::<T>::mutate(|balance| *balance = balance.saturating_sub(serplus_auction.amount));
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
		let bid_result = if <SetterAuctions<T>>::contains_key(id) {
			Self::setter_auction_bid_handler(now, id, new_bid, last_bid)
		} else if <DiamondAuctions<T>>::contains_key(id) {
			Self::diamond_auction_bid_handler(now, id, new_bid, last_bid)
		} else if <SerplusAuctions<T>>::contains_key(id) {
			Self::serplus_auction_bid_handler(now, id, new_bid, last_bid)
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
		if let Some(setter_auction) = <SetterAuctions<T>>::take(id) {
			Self::setter_auction_end_handler(id, setter_auction, winner.clone());
		} else if let Some(diamond_auction) = <DiamondAuctions<T>>::take(id) {
			Self::diamond_auction_end_handler(id, diamond_auction, winner.clone());
		} else if let Some(serplus_auction) = <SerplusAuctions<T>>::take(id) {
			Self::serplus_auction_end_handler(id, serplus_auction, winner.clone());
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

	fn new_setter_auction(
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

		// do not set end time for setter auction
		let auction_id = T::Auction::new_auction(start_time, None)?;

		<SetterAuctions<T>>::insert(
			auction_id,
			SetterAuctionItem {
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

		Self::deposit_event(Event::NewSetterAuction(auction_id, currency_id, amount, target));
		Ok(())
	}

	fn new_diamond_auction(initial_amount: Self::Balance, fix_standard: Self::Balance) -> DispatchResult {
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

		// set end time for diamond auction
		let auction_id = T::Auction::new_auction(start_time, Some(end_block))?;

		<DiamondAuctions<T>>::insert(
			auction_id,
			DiamondAuctionItem {
				initial_amount,
				amount: initial_amount,
				fix: fix_standard,
				start_time,
			},
		);

		Self::deposit_event(Event::NewDiamondAuction(auction_id, initial_amount, fix_standard));
		Ok(())
	}

	fn new_serplus_auction(amount: Self::Balance) -> DispatchResult {
		ensure!(!amount.is_zero(), Error::<T>::InvalidAmount,);
		TotalSerplusInAuction::<T>::try_mutate(|total| -> DispatchResult {
			*total = total.checked_add(amount).ok_or(Error::<T>::InvalidAmount)?;
			Ok(())
		})?;

		let start_time = <frame_system::Module<T>>::block_number();

		// do not set end time for serplus auction
		let auction_id = T::Auction::new_auction(start_time, None)?;

		<SerplusAuctions<T>>::insert(auction_id, SerplusAuctionItem { amount, start_time });

		Self::deposit_event(Event::NewSerplusAuction(auction_id, amount));
		Ok(())
	}

	fn cancel_auction(id: Self::AuctionId) -> DispatchResult {
		if let Some(setter_auction) = <SetterAuctions<T>>::take(id) {
			Self::cancel_setter_auction(id, setter_auction)?;
		} else if let Some(diamond_auction) = <DiamondAuctions<T>>::take(id) {
			Self::cancel_diamond_auction(id, diamond_auction)?;
		} else if let Some(serplus_auction) = <SerplusAuctions<T>>::take(id) {
			Self::cancel_serplus_auction(id, serplus_auction)?;
		} else {
			return Err(Error::<T>::AuctionNotExists.into());
		}
		T::Auction::remove_auction(id);
		Ok(())
	}

		/// Active auction cancelled. \[auction_id\]
		CancelAuction(AuctionId),
	fn get_total_reserve_in_auction(id: Self::CurrencyId) -> Self::Balance {
		Self::total_reserve_in_auction(id)
	}

	fn get_total_serplusin_auction() -> Self::Balance {
		Self::total_serplusin_auction()
	}

	fn get_total_standard_in_auction() -> Self::Balance {
		Self::total_standard_in_auction()
	}

	fn get_total_target_in_auction() -> Self::Balance {
		Self::total_target_in_auction()
	}
}
