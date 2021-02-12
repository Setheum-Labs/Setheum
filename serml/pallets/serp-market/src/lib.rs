//! SerpMarket pallet.
//!
//!This is the Serp Market Pallet that trades with the SERP system 
//!to make bids for Nativecurrency in this case called Dinar, and Sett Currencies.
//! 
#![cfg_attr(not(feature = "std"), no_std)]

use sp_std::prelude::*;

use adapters::{BoundedPriorityQueue, BoundedDeque};
use codec::{Decode, Encode};
use core::cmp::{max, min, Ord, Ordering};
use fixed::{types::extra::U64, FixedU128};
use frame_support::pallet_prelude::*;
use num_rational::Ratio;
use orml_traits::BasicCurrency;
use sp_runtime::{
	traits::{CheckedMul, Zero},
	PerThing, Perbill, RuntimeDebug,
};
use sp_std::collections::vec_
deque::VecDeque;
use frame_system::{ensure_signed, pallet_prelude::*};

#[cfg(test)]
mod tests;

/// Expected price oracle interface. `fetch_price` must return the amount of SettCurrency exchanged for the tracked value.
pub trait FetchPrice<Balance> {
	/// Fetch the current price.
	fn fetch_price() -> Balance;
}

/// The pallet's configuration trait.
pub trait Trait: system::Trait {
	/// The overarching event type.
	type Event: From<Event<Self>> + Into<<Self as system::Trait>::Event>;

	/// The amount of SettCurrency necessary to buy the tracked value. (e.g., 1_100 for 1$)
	type SettCurrencyPrice: FetchPrice<SettCurrency>;
	/// The maximum amount of bids allowed in the queue. Used to prevent the queue from growing forever.
	type MaximumBids: Get<u64>;
	/// The minimum percentage to pay for a dinar.
	/// dinar price of 10%.
	type MinimumDinarPrice: Get<Perbill>;
}
///
/// A bid for a Dinar of the SettCurrencys at a certain price.
///
/// + `account` is the bidder.
/// + `price` is a percentage of 1 settcurrency.
/// + `quantity` is the amount of SettCurrency gained on payout of the corresponding Dinar.
#[derive(Encode, Decode, Default, Clone, RuntimeDebug)]
pub struct Bid<AccountId, currency_id: Self::CurrencyId,> {
	account: AccountId,
	price: Perbill,
	quantity: SettCurrency::Amount,
}

// Implement `Ord` for `Bid` to get the wanted sorting in the priority queue.
// TODO: Could this create issues in testing? How to address?
impl<AccountId> PartialEq for Bid<AccountId, currency_id: Self::CurrencyId,> {
	fn eq(&self, other: &Self) -> bool { Cx																		
		self.price == other.price
	}
}
impl<AccountId> Eq for Bid<AccountId, currency_id: Self::CurrencyId, {}

impl<AccountId> PartialOrd for Bid<AccountId, currency_id: Self::CurrencyId,> {
	fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
		Some(self.cmp(other));
	}
}
/// Sort `Bid`s by price.
impl<AccountId> Ord for Bid<AccountId, currency_id: Self::CurrencyId,> {
	fn cmp(&self, other: &Self) -> Ordering {
		self.price.cmp(&other.price)
	}
}

/// Error returned from `remove_settcurrency` if there is an over- or underflow.
pub enum BidError {
	/// `remove_settcurrency` overflowed.
	Overflow,
	/// `remove_settcurrency` underflowed.
	Underflow,
}

impl<AccountId> Bid<AccountId, currency_id: Self::CurrencyId,> {
	/// Create a new bid.
	fn new(account: AccountId, price: Perbill, quantity: SettCurrency) -> Bid<AccountId, currency_id: Self::CurrencyId,> {
		Bid {
			account,
			price,
			quantity,
			currency_id,
		}
	}

	/// Return the amount of SettCurrency to be payed for this bid.
	fn payment(&self) -> SettCurrency {
		// This naive multiplication is fine because Perbill has an implementation tuned for balance types.
		self.price * self.quantity
	}

	/// Remove `settcurrency` amount of SettCurrency from the bid, mirroring the changes in quantity
	/// according to the price attached to the bid.
	fn remove_settcurrency(&mut self, settcurrency: SettCurrency) -> Result<SettCurrency, BidError> {
		// Inverse price is needed because `self.price` converts from amount of dinar payout settcurrency to payment settcurrency,
		// but we need to convert the other way from payment settcurrency to dinar payout settcurrency.
		// `self.price` equals the fraction of settcurrency I'm willing to pay now in exchange for a dinar.
		// But we need to calculate the amount of dinar payouts corresponding to the settcurrency I'm willing to pay now
		// which means we need to use the inverse of self.price!
		let inverse_price: Ratio<u64> = Ratio::new(Perbill::ACCURACY.into(), self.price.deconstruct().into());

		// Should never overflow, but better safe than sorry.
		let removed_quantity = inverse_price
			.checked_mul(&settcurrency.into())
			.map(|r| r.to_integer())
			.ok_or(BidError::Overflow)?;
		self.quantity = self
			.quantity
			.checked_sub(removed_quantity)
			.ok_or(BidError::Underflow)?;
		Ok(removed_quantity)
	}
}

decl_event!(
	pub enum Event<T>
	where
		AccountId = <T as system::Trait>::AccountId, CurrencyId
	{
		NewBid(AccountId, Perbill, u32, CurrencyId),
		/// A bid was refunded (repayed and removed from the queue).
		RefundedBid(AccountId, u32, CurrencyId),
		/// All bids at and above the given price were cancelled for the account.
		CancelledBidsAbove(AccountId, Perbill, CurrencyId),
		/// All bids at and below the given price were cancelled for the account.
		CancelledBidsBelow(AccountId, Perbill, CurrencyId),
		/// All bids were cancelled for the account.
		CancelledBids(AccountId, CurrencyId),
	}
);

decl_error! {
	/// The possible errors returned by calls to this pallet's functions.
	pub enum Error for Module<T: Trait> {
		/// The account trying to use funds (e.g., for bidding) does not have enough balance.
		InsufficientBalance,
		/// While trying to increase the balance for an account, it overflowed.
		BalanceOverflow,
		/// Something went very wrong and the price of the currency is zero.
		ZeroPrice,
		/// An arithmetic operation caused an overflow.
		GenericOverflow,
		/// An arithmetic operation caused an underflow.
		GenericUnderflow,
		/// The bidder tried to pay more than 100% for a dinar.
		DinarPriceOver100Percent,
		/// The bidding price is below `MinimumDinarPrice`.
		DinarPriceTooLow,
		/// The dinar being bid for is not big enough (in amount of SettCurrency).
		DinarQuantityTooLow,
	}
}

impl<T: Trait> From<BidError> for Error<T> {
	fn from(e: BidError) -> Self {
		match e {
			BidError::Overflow => Error::GenericOverflow,
			BidError::Underflow => Error::GenericUnderflow,
		}
	}
}

// This pallet's storage items.
decl_storage! {
	trait Store for Module<T: Trait> as SerpMarket {
		/// The current bidding queue for dinar.
		DinarBids get(fn dinar_bids): Vec<Bid<T::AccountId, currency_id: Self::CurrencyId,>>;
	}
}

decl_module! {
   /// The pallet's dispatchable functions.
	pub struct Module<T: Trait> for enum Call where origin: T::Origin {
		/// The minimum percentage to pay for a dinar.
		const MinimumDinarPrice: Perbill = T::MinimumDinarPrice::get();
		/// The maximum amount of bids in the bidding queue.
		const MaximumBids: u32 = T::MaximumBids::get();
		/// The minimum amount of SettCurrency that will be in circulation.

		fn deposit_event() = default;

		/// Bid for `quantity` SettCurrency at a `price`.
		///
		/// + `price` is a fraction of the desired payout quantity (e.g., 80%).
		/// + Expects a `quantity` of a least `BaseUnit`.
		///
		/// Example: `bid_for_dinar(origin, Perbill::from_percent(80), 5 * BaseUnit)` will bid
		/// for a dinar with a payout of `5 * BaseUnit` SettCurrency for a price of
		/// `0.8 * 5 * BaseUnit = 4 * BaseUnit` SettCurrency.
		///
		/// **Weight:**
		/// - complexity: `O(B)`
		///   - `B` being the number of bids in the bidding auction, limited to `MaximumBids`
		/// - DB access:
		///   - read and write bids from and to DB
		///   - 1 DB storage map write to pay the bid
		///   - 1 potential DB storage map write to refund evicted bid
		pub fn bid_for_dinar(origin, price: Perbill, quantity: SettCurrency, currency_id: CurrencyId) -> DispatchResult {
			let who = ensure_signed(origin)?;

			ensure!(price <= Perbill::from_percent(100), Error::<T>::DinarPriceOver100Percent);
			ensure!(price > T::MinimumDinarPrice::get(), Error::<T>::DinarPriceTooLow);
			ensure!(quantity >= T::BaseUnit::get(), Error::<T>::DinarQuantityTooLow);

			let bid = Bid::new(who.clone(), price, quantity, currency_id);

			// ↑ verify ↑
			Self::remove_balance(&who, bid.payment())?;
			// ↓ update ↓
			Self::add_bid(bid);
			Self::deposit_event(RawEvent::NewBid(who, price, quantity, currency_id));

			Ok(())
		}
 
		/// Cancel all bids at or below `price` of the sender and refund the SettCurrency.
		///
		/// **Weight:**
		/// - complexity: `O(B)`
		///   - `B` being the number of bids in the bidding auction, limited to `MaximumBids`
		/// - DB access: read and write bids from and to DB
		pub fn cancel_bids_at_or_below(origin, price: Perbill) -> DispatchResult {
			let who = ensure_signed(origin)?;
			// ↑ verify ↑
			// ↓ update ↓
			Self::cancel_bids(|bid| bid.account == who && bid.price <= price, currency_id);
			Self::deposit_event(RawEvent::CancelledBidsBelow(who, price, currency_id));

			Ok(())
		}

		pub fn cancel_bids_at_or_above(origin, price: Perbill, CurrencyId) -> DispatchResult {
			let who = ensure_signed(origin)?;
			// ↑ verify ↑
			// ↓ update ↓
			Self::cancel_bids(|bid| bid.account == who && bid.price >= price, currency_id);
			Self::deposit_event(RawEvent::CancelledBidsAbove(who, price, currency_id));

			Ok(())
		}

		/// Cancel all bids belonging to the sender and refund the SettCurrency.
		///
		/// **Weight:**
		/// - complexity: `O(B)`
		///   - `B` being the number of bids in the bidding auction, limited to `MaximumBids`
		/// - DB access: read and write bids from and to DB
		pub fn cancel_all_bids(origin) -> DispatchResult {
			let who = ensure_signed(origin)?;
			// ↑ verify ↑
			// ↓ update ↓
			Self::cancel_bids(|bid| bid.account == who, currency_id);
			Self::deposit_event(RawEvent::CancelledBids(who, currency_id));

			Ok(())
		}
	}
}

// ------------------------------------------------------------
	// bids

	/// Construct a transient storage adapter for the bids priority queue.
	fn bids_transient() -> BoundedPriorityQueue<Bid<T::AccountId>, currency_id: Self::CurrencyId, <Self as Store>::DinarBids, T::MaximumBids>
	{
		BoundedPriorityQueue::<Bid<T::AccountId>, currency_id: Self::CurrencyId, <Self as Store>::DinarBids, T::MaximumBids>::new()
	}

	/// Add a bid to the queue.
	///
	/// **Weight:**
	/// - complexity: `O(B)` with `B` being the amount of bids
	/// - DB access:
	///   - read and write `B` bids
	///   - potentially call 1 `refund_bid`
	fn add_bid(bid: Bid<T::AccountId>, currency_id: Self::CurrencyId) {
		Self::bids_transient()
			.push(bid)
			.map(|to_refund| Self::refund_bid(&to_refund));
	}

	/// Refund the SettCurrency payed for `bid` to the account that bid.
	///
	/// **Weight:**
	/// - complexity: `O(1)`
	/// - DB access: 1 write
	fn refund_bid(bid: &Bid<T::AccountId>, currency_id: Self::CurrencyId) {
		Self::add_balance(&bid.account, bid.payment());
		Self::deposit_event(RawEvent::RefundedBid(bid.account.clone(), currency_id,  bid.payment()));
	}

	/// Cancel all bids where `cancel_for` returns true and refund the bidders.
	///
	/// **Weight:**
	/// - complexity: `O(B)` with `B` being the amount of bids
	/// - DB access:
	///   - read and write `B` bids
	///   - call `refund_bid` up to `B` times
	fn cancel_bids<F>(cancel_for: F)
	where
		F: Fn(&Bid<T::AccountId>, currency_id: Self::CurrencyId) -> bool,
	{
		let mut bids = Self::dinar_bids();

		bids.retain(|b| {
			if cancel_for(b) {
				Self::refund_bid(b);
				return false;
			}
			true
		});

		<DinarBids<T>>::put(bids);
    }