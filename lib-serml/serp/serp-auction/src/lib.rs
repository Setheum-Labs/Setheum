// This file is part of Acala.

// Copyright (C) 2020-2021 Acala Foundation.
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
//! Auction the assets of the system for maintain the normal operation of the
//! business. Auction types include:
//!   - `reserve auction`: sell reserve assets for getting stable currency to eliminate the
//!     system's bad debit by auction

#![cfg_attr(not(feature = "std"), no_std)]
#![allow(clippy::unused_unit)]
#![allow(clippy::upper_case_acronyms)]

use frame_support::{log, pallet_prelude::*, transactional};
use frame_system::{
	offchain::SendTransactionTypes,
	pallet_prelude::*,
};
use orml_traits::{Auction, AuctionHandler, Change, MultiCurrency, OnNewBidResult};
use primitives::{AuctionId, Balance, CurrencyId};
use sp_runtime::{
	traits::{CheckedDiv, Saturating, Zero},
	transaction_validity::TransactionPriority,
	DispatchError, DispatchResult, FixedPointNumber, RuntimeDebug,
};
use sp_std::prelude::*;
use support::{SerpAuctionManager, SerpTreasury, SerpTreasuryExtended, DEXManager, PriceProvider, Rate};

mod mock;
mod tests;
pub mod weights;

pub use module::*;
pub use weights::WeightInfo;

pub const OFFCHAIN_WORKER_DATA: &[u8] = b"acala/auction-manager/data/";
pub const OFFCHAIN_WORKER_LOCK: &[u8] = b"acala/auction-manager/lock/";
pub const OFFCHAIN_WORKER_MAX_ITERATIONS: &[u8] = b"acala/auction-manager/max-iterations/";
pub const LOCK_DURATION: u64 = 100;
pub const DEFAULT_MAX_ITERATIONS: u32 = 1000;

/// Information of an reserve auction
#[cfg_attr(feature = "std", derive(PartialEq, Eq))]
#[derive(Encode, Decode, Clone, RuntimeDebug)]
pub struct DinarAuctionItem<AccountId, BlockNumber> {
	/// Refund recipient for may receive refund
	refund_recipient: AccountId,
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

impl<AccountId, BlockNumber> DinarAuctionItem<AccountId, BlockNumber> {
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

#[frame_support::pallet]
pub mod module {
	use super::*;

	#[pallet::config]
	pub trait Config: frame_system::Config + SendTransactionTypes<Call<Self>> {
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;

		/// The minimum increment size of each bid compared to the previous one
		#[pallet::constant]
		type MinimumIncrementSize: Get<Rate>;

		/// The extended time for the auction to end after each successful bid
		#[pallet::constant]
		type AuctionTimeToClose: Get<Self::BlockNumber>;

		/// When the total duration of the auction exceeds this soft cap, push
		/// the auction to end more faster
		#[pallet::constant]
		type AuctionDurationSoftCap: Get<Self::BlockNumber>;

		/// The Dinar currency id
		#[pallet::constant]
		type NativeCurrencyId: Get<CurrencyId>;

		/// The Setter currency id
		#[pallet::constant]
		type SetterCurrencyId: Get<CurrencyId>;

		/// Currency to transfer assets
		type Currency: MultiCurrency<Self::AccountId, CurrencyId = CurrencyId, Balance = Balance>;

		/// Auction to manager the auction process
		type Auction: Auction<Self::AccountId, Self::BlockNumber, AuctionId = AuctionId, Balance = Balance>;

		/// CDP treasury to escrow assets related to auction
		type SerpTreasury: SerpTreasuryExtended<Self::AccountId, Balance = Balance, CurrencyId = CurrencyId>;

		/// DEX to get exchange info
		type DEX: DEXManager<Self::AccountId, CurrencyId, Balance>;

		/// The price source of currencies
		type PriceSource: PriceProvider<CurrencyId>;

		/// A configuration for base priority of unsigned transactions.
		///
		/// This is exposed so that it can be tuned for particular runtime, when
		/// multiple modules send unsigned transactions.
		#[pallet::constant]
		type UnsignedPriority: Get<TransactionPriority>;

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
		/// Bid price is invalid
		InvalidBidPrice,
		/// Invalid input amount
		InvalidAmount,
	}

	#[pallet::event]
	#[pallet::generate_deposit(pub(crate) fn deposit_event)]
	#[pallet::metadata(T::AccountId = "AccountId")]
	pub enum Event<T: Config> {
		/// Reserve auction created. \[auction_id, reserve_type,
		/// reserve_amount, target_bid_price\]
		NewDinarAuction(AuctionId, Balance, Balance),
		/// Active auction cancelled. \[auction_id\]
		CancelAuction(AuctionId),
		/// Reserve auction dealt. \[auction_id, reserve_type,
		/// reserve_amount, winner, payment_amount\]
		DinarAuctionDealt(AuctionId, Balance, T::AccountId, Balance),
		/// Dex take reserve auction. \[auction_id, reserve_type,
		/// reserve_amount, turnover\]
		DEXTakeDinarAuction(AuctionId, Balance, Balance),
	}

	/// Mapping from auction id to reserve auction info
	///
	/// DinarAuction: map AuctionId => Option<DinarAuctionItem>
	#[pallet::storage]
	#[pallet::getter(fn dinar_auctions)]
	pub type DinarAuction<T: Config> =
		StorageMap<_, Twox64Concat, AuctionId, DinarAuctionItem<T::AccountId, T::BlockNumber>, OptionQuery>;

	/// Record of the total Dinar amount of all active Dinar auctions  -> TotalAmount
	///
	/// TotalDinarInAuction: Balance
	#[pallet::storage]
	#[pallet::getter(fn total_dinar_in_auction)]
	pub type TotalDinarInAuction<T: Config> = StorageValue<_, Balance, ValueQuery>;

	/// Record of total target sales of all active reserve auctions
	///
	/// TotalTargetInAuction: Balance
	#[pallet::storage]
	#[pallet::getter(fn total_target_in_auction)]
	pub type TotalTargetInAuction<T: Config> = StorageValue<_, Balance, ValueQuery>;

	#[pallet::pallet]
	pub struct Pallet<T>(_);

	#[pallet::hooks]
	impl<T: Config> Hooks<T::BlockNumber> for Pallet<T> {}
}

impl<T: Config> Pallet<T> {
	fn get_last_bid(auction_id: AuctionId) -> Option<(T::AccountId, Balance)> {
		T::Auction::auction_info(auction_id).and_then(|auction_info| auction_info.bid)
	}

	fn cancel_dinar_auction(
		id: AuctionId,
		dinar_auction: DinarAuctionItem<T::AccountId, T::BlockNumber>,
	) -> DispatchResult {
		let last_bid = Self::get_last_bid(id);

		// reserve auction must not be in reverse stage
		if let Some((_, bid_price)) = last_bid {
			ensure!(
				!dinar_auction.in_reverse_stage(bid_price),
				Error::<T>::InReverseStage,
			);
		}

		// calculate how much reserve to offset target in settle price
		let setter_currency_id = T::SetterCurrencyId::get();
		let dinar_currency_id = T::NativeCurrencyId::get();
		let settle_price = T::PriceSource::get_relative_price(setter_currency_id, dinar_currency_id)
			.ok_or(Error::<T>::InvalidFeedPrice)?;
		let confiscate_reserve_amount = if dinar_auction.always_forward() {
			dinar_auction.amount
		} else {
			sp_std::cmp::min(
				settle_price.saturating_mul_int(dinar_auction.target),
				dinar_auction.amount,
			)
		};
		let refund_reserve_amount = dinar_auction.amount.saturating_sub(confiscate_reserve_amount);

		// refund remain reserve to refund recipient from CDP treasury
		T::SerpTreasury::withdraw_dinar(
			&dinar_auction.refund_recipient,
			refund_reserve_amount,
		)?;

		// if there's bid
		if let Some((bidder, bid_price)) = last_bid {
			// refund stable token to the bidder
			T::SerpTreasury::issue_setter(&bidder, bid_price)?;

			// decrease account ref of bidder
			frame_system::Pallet::<T>::dec_consumers(&bidder);
		}

		// decrease account ref of refund recipient
		frame_system::Pallet::<T>::dec_consumers(&dinar_auction.refund_recipient);

		// decrease total reserve and target in auction
		TotalDinarInAuction::<T>::mutate(|balance| {
			*balance = balance.saturating_sub(dinar_auction.amount)
		});
		TotalTargetInAuction::<T>::mutate(|balance| *balance = balance.saturating_sub(dinar_auction.target));

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
	pub fn dinar_auction_bid_handler(
		now: T::BlockNumber,
		id: AuctionId,
		new_bid: (T::AccountId, Balance),
		last_bid: Option<(T::AccountId, Balance)>,
	) -> sp_std::result::Result<T::BlockNumber, DispatchError> {
		let (new_bidder, new_bid_price) = new_bid;
		ensure!(!new_bid_price.is_zero(), Error::<T>::InvalidBidPrice);

		<DinarAuction<T>>::try_mutate_exists(
			id,
			|dinar_auction| -> sp_std::result::Result<T::BlockNumber, DispatchError> {
				let mut dinar_auction = dinar_auction.as_mut().ok_or(Error::<T>::AuctionNotExists)?;
				let last_bid_price = last_bid.clone().map_or(Zero::zero(), |(_, price)| price); // get last bid price

				// ensure new bid price is valid
				ensure!(
					Self::check_minimum_increment(
						new_bid_price,
						last_bid_price,
						dinar_auction.target,
						Self::get_minimum_increment_size(now, dinar_auction.start_time),
					),
					Error::<T>::InvalidBidPrice
				);

				let last_bidder = last_bid.as_ref().map(|(who, _)| who);

				let mut payment = dinar_auction.payment_amount(new_bid_price);

				// if there's bid before, return stablecoin from new bidder to last bidder
				if let Some(last_bidder) = last_bidder {
					let refund = dinar_auction.payment_amount(last_bid_price);
					T::Currency::transfer(T::SetterCurrencyId::get(), &new_bidder, last_bidder, refund)?;

					payment = payment
						.checked_sub(refund)
						// This should never fail because new bid payment are always greater or equal to last bid
						// payment.
						.ok_or(Error::<T>::InvalidBidPrice)?;
				}

				// transfer remain payment from new bidder to CDP treasury
				T::SerpTreasury::deposit_setter(&new_bidder, payment)?;

				// if reserve auction will be in reverse stage, refund reserve to it's
				// origin from auction CDP treasury
				if dinar_auction.in_reverse_stage(new_bid_price) {
					let new_reserve_amount = dinar_auction.reserve_amount(last_bid_price, new_bid_price);
					let refund_reserve_amount = dinar_auction.amount.saturating_sub(new_reserve_amount);

					if !refund_reserve_amount.is_zero() {
						T::SerpTreasury::withdraw_dinar(
							&(dinar_auction.refund_recipient),
							refund_reserve_amount,
						)?;

						// update total reserve in auction after refund
						TotalDinarInAuction::<T>::mutate(|balance| {
							*balance = balance.saturating_sub(refund_reserve_amount)
						});
						dinar_auction.amount = new_reserve_amount;
					}
				}

				Self::swap_bidders(&new_bidder, last_bidder);

				Ok(now + Self::get_auction_time_to_close(now, dinar_auction.start_time))
			},
		)
	}

	fn dinar_auction_end_handler(
		auction_id: AuctionId,
		dinar_auction: DinarAuctionItem<T::AccountId, T::BlockNumber>,
		winner: Option<(T::AccountId, Balance)>,
	) {
		let dinar_currency_id = T::NativeCurrencyId::get();
		
		if let Some((bidder, bid_price)) = winner {
			let mut should_deal = true;

			// if bid_price doesn't reach target and trading with DEX will get better result
			if !dinar_auction.in_reverse_stage(bid_price)
				&& bid_price
					< T::DEX::get_swap_target_amount(
						&[dinar_currency_id, T::SetterCurrencyId::get()],
						dinar_auction.amount,
						None,
					)
					.unwrap_or_default()
			{
				// try swap reserve in auction with DEX to get stable
				if let Ok(stable_amount) = T::SerpTreasury::swap_exact_dinar_to_setter(
					dinar_auction.amount,
					Zero::zero(),
					None,
					None,
					true,
				) {
					// swap successfully, will not deal
					should_deal = false;

					// refund stable currency to the last bidder, it shouldn't fail and affect the
					// process. but even it failed, just the winner did not get the bid price. it
					// can be fixed by treasury council.
					let res = T::SerpTreasury::issue_setter(&bidder, bid_price);
					if let Err(e) = res {
						log::warn!(
							target: "auction-manager",
							"issue_setter: failed to issue stable {:?} to {:?}: {:?}. \
							This is unexpected but should be safe",
							bid_price, bidder, e
						);
						debug_assert!(false);
					}

					if dinar_auction.in_reverse_stage(stable_amount) {
						// refund extra stable currency to recipient
						let refund_amount = stable_amount
							.checked_sub(dinar_auction.target)
							.expect("ensured stable_amount > target; qed");
						// it shouldn't fail and affect the process.
						// but even it failed, just the winner did not get the refund amount. it can be
						// fixed by treasury council.
						let res =
							T::SerpTreasury::issue_setter(&dinar_auction.refund_recipient, refund_amount);
						if let Err(e) = res {
							log::warn!(
								target: "auction-manager",
								"issue_setter: failed to issue Setter {:?} to {:?}: {:?}. \
								This is unexpected but should be safe",
								refund_amount, dinar_auction.refund_recipient, e
							);
							debug_assert!(false);
						}
					}

					Self::deposit_event(Event::DEXTakeDinarAuction(
						auction_id,
						dinar_auction.amount,
						stable_amount,
					));
				}
			}

			if should_deal {
				// transfer dinar to winner from SERP Treasury, it shouldn't fail and affect
				// the process. but even it failed, just the winner did not get the amount. it
				// can be fixed by treasury council.
				let res = T::SerpTreasury::withdraw_dinar(
					&bidder,
					dinar_auction.amount,
				);
				if let Err(e) = res {
					log::warn!(
						target: "auction-manager",
						"withdraw_dinar: failed to withdraw Dinar {:?} from SERP Treasury to {:?}: {:?}. \
						This is unexpected but should be safe",
						dinar_auction.amount, bidder, e
					);
					debug_assert!(false);
				}

				let payment_amount = dinar_auction.payment_amount(bid_price);
				Self::deposit_event(Event::DinarAuctionDealt(
					auction_id,
					dinar_auction.amount,
					bidder,
					payment_amount,
				));
			}
		} else {
			Self::deposit_event(Event::CancelAuction(auction_id));
		}

		// decrement recipient account reference
		frame_system::Pallet::<T>::dec_consumers(&dinar_auction.refund_recipient);

		// update auction records
		TotalDinarInAuction::<T>::mutate(|balance| {
			*balance = balance.saturating_sub(dinar_auction.amount)
		});
		TotalTargetInAuction::<T>::mutate(|balance| *balance = balance.saturating_sub(dinar_auction.target));
	}

	/// increment `new_bidder` reference and decrement `last_bidder`
	/// reference if any
	fn swap_bidders(new_bidder: &T::AccountId, last_bidder: Option<&T::AccountId>) {
		if frame_system::Pallet::<T>::inc_consumers(new_bidder).is_err() {
			// No providers for the locks. This is impossible under normal circumstances
			// since the funds that are under the lock will themselves be stored in the
			// account and therefore will need a reference.
			log::warn!(
				target: "auction-manager",
				"inc_consumers: failed for {:?}. \
				This is impossible under normal circumstances.",
				new_bidder.clone()
			);
		}

		if let Some(who) = last_bidder {
			frame_system::Pallet::<T>::dec_consumers(who);
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
		let bid_result = Self::dinar_auction_bid_handler(now, id, new_bid, last_bid);

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
		if let Some(dinar_auction) = <DinarAuction<T>>::take(id) {
			Self::dinar_auction_end_handler(id, dinar_auction, winner.clone());
		}

		if let Some((bidder, _)) = &winner {
			// decrease account ref of winner
			frame_system::Pallet::<T>::dec_consumers(bidder);
		}
	}
}

impl<T: Config> SerpAuctionManager<T::AccountId> for Pallet<T> {
	type CurrencyId = CurrencyId;
	type Balance = Balance;
	type AuctionId = AuctionId;

	fn new_dinar_auction(
		refund_recipient: &T::AccountId,
		amount: Self::Balance,
		target: Self::Balance,
	) -> DispatchResult {
		ensure!(!amount.is_zero(), Error::<T>::InvalidAmount);
		TotalDinarInAuction::<T>::try_mutate(|total| -> DispatchResult {
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

		let start_time = <frame_system::Pallet<T>>::block_number();

		// do not set end time for reserve auction
		let auction_id = T::Auction::new_auction(start_time, None)?;

		<DinarAuction<T>>::insert(
			auction_id,
			DinarAuctionItem {
				refund_recipient: refund_recipient.clone(),
				initial_amount: amount,
				amount,
				target,
				start_time,
			},
		);

		// increment recipient account reference
		if frame_system::Pallet::<T>::inc_consumers(refund_recipient).is_err() {
			// No providers for the locks. This is impossible under normal circumstances
			// since the funds that are under the lock will themselves be stored in the
			// account and therefore will need a reference.
			log::warn!(
				target: "auction-manager",
				"Attempt to `inc_consumers` for {:?} failed. \
				This is unexpected but should be safe.",
				refund_recipient.clone()
			);
		}

		Self::deposit_event(Event::NewDinarAuction(auction_id, amount, target));
		Ok(())
	}

	fn cancel_auction(id: Self::AuctionId) -> DispatchResult {
		let dinar_auction = <DinarAuction<T>>::take(id).ok_or(Error::<T>::AuctionNotExists)?;
		Self::cancel_dinar_auction(id, dinar_auction)?;
		T::Auction::remove_auction(id);
		Ok(())
	}

	fn get_total_dinar_in_auction() -> Self::Balance {
		Self::total_dinar_in_auction()
	}

	fn get_total_target_in_auction() -> Self::Balance {
		Self::total_target_in_auction()
	}
}
