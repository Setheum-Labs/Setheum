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
use support::{SerpAuction, SerpTreasury, SerpTreasuryExtended, DEXManager, PriceProvider, Rate};

mod mock;
mod tests;
pub mod weights;

pub use module::*;
pub use weights::WeightInfo;

pub const OFFCHAIN_WORKER_DATA: &[u8] = b"setheum/serp-auction/data/";
pub const OFFCHAIN_WORKER_LOCK: &[u8] = b"setheum/serp-auction/lock/";
pub const OFFCHAIN_WORKER_MAX_ITERATIONS: &[u8] = b"setheum/serp-auction/max-iterations/";
pub const LOCK_DURATION: u64 = 100;
pub const DEFAULT_MAX_ITERATIONS: u32 = 1000;

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

/// Information of a setter auction
#[cfg_attr(feature = "std", derive(PartialEq, Eq))]
#[derive(Encode, Decode, Clone, RuntimeDebug)]
pub struct SetterAuctionItem<BlockNumber> {
	/// Initial amount of setter stablecurrency for sale to buy back SettCurrency stablecoin.
	#[codec(compact)]
	initial_amount: Balance,
	/// Current amount of setter stablecurrency for sale to buy back SettCurrency stablecoin.
	#[codec(compact)]
	amount: Balance,
	/// Fix amount of SettCurrency stablecoin value needed to be got back by this auction.
	#[codec(compact)]
	fix: Balance,
	/// Auction start time
	start_time: BlockNumber,
}

impl<BlockNumber> SetterAuctionItem<BlockNumber> {
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

/// Information on a serplus auction
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
		type DiamondAuctionMinimumIncrementSize: Get<Rate>;

		#[pallet::constant]
		/// The minimum increment size of each bid compared to the previous one
		type SetterAuctionMinimumIncrementSize: Get<Rate>;

		#[pallet::constant]
		/// The minimum increment size of each bid compared to the previous one
		type SerplusAuctionMinimumIncrementSize: Get<Rate>;

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
		AuctionNonExistent,
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
		/// Diamond Auction created. \[auction_id, initial_supply_amount,
		/// fix_payment_amount\]
		NewDiamondAuction(AuctionId, Balance, Balance),
		/// Setter auction created. \[auction_id, reserve_type,
		/// reserve_amount, target_bid_price\]
		NewSetterAuction(AuctionId, CurrencyId, Balance, Balance),
		/// serplus auction created. \[auction_id, fix_serplus_amount\]
		NewSerplusAuction(AuctionId, Balance),
		/// Active auction cancelled. \[auction_id\]
		CancelAuction(AuctionId),
		/// Diamond Auction dealt. \[auction_id, standard_currency_amount, winner,
		/// payment_amount\]
		DiamondAuctionDealt(AuctionId, Balance, T::AccountId, Balance),
		/// Setter auction dealt. \[auction_id, reserve_type,
		/// reserve_amount, winner, payment_amount\]
		SetterAuctionDealt(AuctionId, CurrencyId, Balance, T::AccountId, Balance),
		/// serplus auction dealt. \[auction_id, serplus_amount, winner,
		/// payment_amount\]
		SerplusAuctionDealt(AuctionId, Balance, T::AccountId, Balance),
		/// Dex take setter auction. \[auction_id, reserve_type,
		/// reserve_amount, turnover\]
		DEXTakeSetterAuction(AuctionId, CurrencyId, Balance, Balance),
	}

	/// Mapping from auction id to diamond auction info
	#[pallet::storage]
	#[pallet::getter(fn diamond_auctions)]
	pub type DiamondAuctions<T: Config> =
		StorageMap<_, Twox64Concat, AuctionId, DiamondAuctionItem<T::BlockNumber>, OptionQuery>;

	/// Mapping from auction id to setter auction info
	#[pallet::storage]
	#[pallet::getter(fn setter_auctions)]
	pub type SetterAuctions<T: Config> =
		StorageMap<_, Twox64Concat, AuctionId, SetterAuctionItem<T::AccountId, T::BlockNumber>, OptionQuery>;

	/// Mapping from auction id to serplus auction info
	#[pallet::storage]
	#[pallet::getter(fn serplus_auctions)]
	pub type SerplusAuctions<T: Config> =
		StorageMap<_, Twox64Concat, AuctionId, SerplusAuctionItem<T::BlockNumber>, OptionQuery>;

	/// Record of total fixed amount of all active diamond auctions
	#[pallet::storage]
	#[pallet::getter(fn total_setter_in_auction)]
	pub type TotalSetterInAuction<T: Config> = StorageValue<_, Balance, ValueQuery>;

	/// Record of total fixed amount of all active setter auctions
	/// under specific currency type SettCurrencyType -> TotalAmount
	#[pallet::storage]
	#[pallet::getter(fn total_settcurrency_in_auction)]
	pub type TotalSettCurrencyInAuction<T: Config> = StorageValue<_, Balance, ValueQuery>;

	/// Record of total serplus amount of all active serplus auctions
	#[pallet::storage]
	#[pallet::getter(fn total_serplus_in_auction)]
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
				target: "serp-auction offchain worker",
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

		debug::debug!(target: "serp-auction offchain worker", "max iterations is {:?}", max_iterations);

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

	fn cancel_diamond_auction(id: AuctionId, diamond_auction: DiamondAuctionItem<T::BlockNumber>) -> DispatchResult {
		// if there's bid
		if let Some((bidder, _)) = Self::get_last_bid(id) {
			// refund stable token to the bidder
			T::SerpTreasury::issue_standard(&bidder, diamond_auction.fix, false)?;

			// decrease account ref of bidder
			frame_system::Module::<T>::dec_consumers(&bidder);
		}

		// decrease total propper setter in auction
		TotalSetterInAuction::<T>::mutate(|balance| *balance = balance.saturating_sub(diamond_auction.fix));

		Ok(())
	}

	fn cancel_setter_auction(id: AuctionId, setter_auction: SetterAuctionItem<T::BlockNumber>) -> DispatchResult {
		// if there's bid
		if let Some((bidder, _)) = Self::get_last_bid(id) {
			// refund stable token to the bidder
			T::SerpTreasury::issue_standard(&bidder, setter_auction.fix, false)?;

			// decrease account ref of bidder
			frame_system::Module::<T>::dec_consumers(&bidder);
		}

		// decrease total propper setter in auction
		TotalSettCurrencyInAuction::<T>::mutate(|balance| *balance = balance.saturating_sub(setter_auction.fix));

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

		// decrease total propper settcurrency serplus in auction
		TotalSerplusInAuction::<T>::mutate(|balance| *balance = balance.saturating_sub(serplus_auction.amount));

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

	// diamond-auction functions
	fn get_diamond_auction_minimum_increment_size(now: T::BlockNumber, start_block: T::BlockNumber) -> Rate {
		if now >= start_block + T::AuctionDurationSoftCap::get() {
			// double the minimum increment size when reach soft cap
				T::DiamondAuctionMinimumIncrementSize::get().saturating_mul(Rate::saturating_from_integer(2))
		} else {
			T::DiamondAuctionMinimumIncrementSize::get()
		}
	}

	// setter-auction functions
	fn get_setter_auction_minimum_increment_size(now: T::BlockNumber, start_block: T::BlockNumber) -> Rate {
		if now >= start_block + T::AuctionDurationSoftCap::get() {
			// double the minimum increment size when reach soft cap
				T::SetterAuctionMinimumIncrementSize::get().saturating_mul(Rate::saturating_from_integer(2))
		} else {
			T::SetterAuctionMinimumIncrementSize::get()
		}
	}

	// serplus-auction functions
	fn get_serplus_auction_minimum_increment_size(now: T::BlockNumber, start_block: T::BlockNumber) -> Rate {
		if now >= start_block + T::AuctionDurationSoftCap::get() {
			// double the minimum increment size when reach soft cap
				T::SerplusAuctionMinimumIncrementSize::get().saturating_mul(Rate::saturating_from_integer(2))
		} else {
			T::SerplusAuctionMinimumIncrementSize::get()
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
				let mut diamond_auction = diamond_auction.as_mut().ok_or(Error::<T>::AuctionNonExistent)?;
				let (new_bidder, new_bid_price) = new_bid;
				let last_bid_price = last_bid.clone().map_or(Zero::zero(), |(_, price)| price); // get last bid price

				ensure!(
					Self::check_minimum_increment(
						new_bid_price,
						last_bid_price,
						diamond_auction.fix,
						Self::get_diamond_auction_minimum_increment_size(now, diamond_auction.start_time),
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

	/// Handles setter auction new bid. Returns `Ok(new_auction_end_time)` if
	/// bid accepted.
	///
	/// Ensured atomic.
	#[transactional]
	pub fn setter_auction_bid_handler(
		now: T::BlockNumber,
		id: AuctionId,
		new_bid: (T::AccountId, Balance),
		last_bid: Option<(T::AccountId, Balance)>,
	) -> sp_std::result::Result<T::BlockNumber, DispatchError> {
		<DiamondAuctions<T>>::try_mutate_exists(
			id,
			|setter_auction| -> sp_std::result::Result<T::BlockNumber, DispatchError> {
				let mut setter_auction = setter_auction.as_mut().ok_or(Error::<T>::AuctionNonExistent)?;
				let (new_bidder, new_bid_price) = new_bid;
				let last_bid_price = last_bid.clone().map_or(Zero::zero(), |(_, price)| price); // get last bid price

				ensure!(
					Self::check_minimum_increment(
						new_bid_price,
						last_bid_price,
						setter_auction.fix,
						Self:: get_setter_auction_minimum_increment_size(now, setter_auction.start_time),
					) && new_bid_price >= setter_auction.fix,
					Error::<T>::InvalidBidPrice,
				);

				let last_bidder = last_bid.as_ref().map(|(who, _)| who);

				if let Some(last_bidder) = last_bidder {
					// there's bid before, transfer the stablecoin from new bidder to last bidder
					T::Currency::transfer(
						/// change to `T::GetSettCurrencyId::get()`
						T::GetStableCurrencyId::get(),
						&new_bidder,
						last_bidder,
						setter_auction.fix,
					)?;
				} else {
					// there's no bid before, transfer stablecoin to SERP Treasury
					T::SerpTreasury::deposit_serplus(&new_bidder, setter_auction.fix)?;
				}

				Self::swap_bidders(&new_bidder, last_bidder);

				setter_auction.amount = setter_auction.amount_for_sale(last_bid_price, new_bid_price);

				Ok(now + Self::get_auction_time_to_close(now, setter_auction.start_time))
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

		let serplus_auction = Self::serplus_auctions(id).ok_or(Error::<T>::AuctionNonExistent)?;
		let last_bid_price = last_bid.clone().map_or(Zero::zero(), |(_, price)| price); // get last bid price
		let native_currency_id = T::GetNativeCurrencyId::get();

		ensure!(
			Self::check_minimum_increment(
				new_bid_price,
				last_bid_price,
				Zero::zero(),
				Self::get_serplus_auction_minimum_increment_size(now, serplus_auction.start_time),
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

		TotalSetterInAuction::<T>::mutate(|balance| *balance = balance.saturating_sub(diamond_auction.fix));
	}

	fn setter_auction_end_handler(
		auction_id: AuctionId,
		setter_auction: SetterAuctionItem<T::BlockNumber>,
		winner: Option<(T::AccountId, Balance)>,
	) {
		if let Some((bidder, _)) = winner {
			// issue native token to winner, it shouldn't fail and affect the process.
			// but even it failed, just the winner did not get the amount. it can be fixed
			// by treasury council. TODO: transfer from RESERVED TREASURY instead of issuing
			let _ = T::Currency::deposit(T::GetNativeCurrencyId::get(), &bidder, setter_auction.amount);

			Self::deposit_event(Event::SetterAuctionDealt(
				auction_id,
				setter_auction.amount,
				bidder,
				setter_auction.fix,
			));
		} else {
			Self::deposit_event(Event::CancelAuction(auction_id));
		}

		TotalSettCurrencyInAuction::<T>::mutate(|balance| *balance = balance.saturating_sub(setter_auction.fix));
	}

	fn serplus_auction_end_handler(
		auction_id: AuctionId,
		serplus_auction: SerplusAuctionItem<T::BlockNumber>,
		winner: Option<(T::AccountId, Balance)>,
	) {
		if let Some((bidder, bid_price)) = winner {
			// deposit unbacked propper stablecoin (SettCurrency) to winner by SERP Treasury, it shouldn't fail
			// and affect the process. but even it failed, and just the winner did not get the
			// amount.. it could be fixed by the treasury council.
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
		let bid_result = if <DiamondAuctions<T>>::contains_key(id) {
			Self::diamond_auction_bid_handler(now, id, new_bid, last_bid)
		} else if <SetterAuctions<T>>::contains_key(id) {
			Self::setter_auction_bid_handler(now, id, new_bid, last_bid)
		} else if <SerplusAuctions<T>>::contains_key(id) {
			Self::serplus_auction_bid_handler(now, id, new_bid, last_bid)
		} else {
			Err(Error::<T>::AuctionNonExistent.into())
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
		if let Some(diamond_auction) = <DiamondAuctions<T>>::take(id) {
			Self::diamond_auction_end_handler(id, diamond_auction, winner.clone());
		} else if let Some(setter_auction) = <SetterAuctions<T>>::take(id) {
			Self::setter_auction_end_handler(id, setter_auction, winner.clone());
		} else if let Some(serplus_auction) = <SerplusAuctions<T>>::take(id) {
			Self::serplus_auction_end_handler(id, serplus_auction, winner.clone());
		}

		if let Some((bidder, _)) = &winner {
			// decrease account ref of winner
			frame_system::Module::<T>::dec_consumers(bidder);
		}
	}
}

impl<T: Config> SerpAuction<T::AccountId> for Pallet<T> {
	type CurrencyId = CurrencyId;
	type Balance = Balance;
	type AuctionId = AuctionId;

	fn new_diamond_auction(initial_amount: Self::Balance, fix_setter: Self::Balance) -> DispatchResult {
		ensure!(
			!initial_amount.is_zero() && !fix_setter.is_zero(),
			Error::<T>::InvalidAmount,
		);
		TotalSetterInAuction::<T>::try_mutate(|total| -> DispatchResult {
			*total = total.checked_add(fix_setter).ok_or(Error::<T>::InvalidAmount)?;
			Ok(())
		})?;

		let start_time = <frame_system::Module<T>>::block_number();
		let end_block = start_time + T::AuctionTimeToClose::get();

		// set ending time for Diamond Auction
		let auction_id = T::Auction::new_auction(start_time, Some(end_block))?;

		<DiamondAuctions<T>>::insert(
			auction_id,
			DiamondAuctionItem {
				initial_amount,
				amount: initial_amount,
				fix: fix_setter,
				start_time,
			},
		);

		Self::deposit_event(Event::NewDiamondAuction(auction_id, initial_amount, fix_setter));
		Ok(())
	}

	fn new_setter_auction(initial_amount: Self::Balance, fix_settcurrency: Self::Balance) -> DispatchResult {
		ensure!(
			!initial_amount.is_zero() && !fix_settcurrency.is_zero(),
			Error::<T>::InvalidAmount,
		);
		TotalSettCurrencyInAuction::<T>::try_mutate(|total| -> DispatchResult {
			*total = total.checked_add(fix_settcurrency).ok_or(Error::<T>::InvalidAmount)?;
			Ok(())
		})?;

		let start_time = <frame_system::Module<T>>::block_number();
		let end_block = start_time + T::AuctionTimeToClose::get();

		// set ending time for Setter Auction
		let auction_id = T::Auction::new_auction(start_time, Some(end_block))?;

		<SetterAuctions<T>>::insert(
			auction_id,
			SetterAuctionItem {
				initial_amount,
				amount: initial_amount,
				fix: fix_settcurrency,
				start_time,
			},
		);

		Self::deposit_event(Event::NewSetterAuction(auction_id, initial_amount, fix_settcurrency));
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
			return Err(Error::<T>::AuctionNonExistent.into());
		}
		T::Auction::remove_auction(id);
		Ok(())
	}

	fn get_total_setter_in_auction() -> Self::Balance {
		Self::total_setter_in_auction()
	}
	fn get_total_settcurrency_in_auction(id: Self::CurrencyId) -> Self::Balance {
		Self::total_settcurrency_in_auction(id)
	}

	fn get_total_serplus_in_auction() -> Self::Balance {
		Self::total_serplus_in_auction()
	}
}
