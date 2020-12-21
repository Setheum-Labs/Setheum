/// pallet-coins

/// pallet-coins

#![cfg_attr(not(feature = "std"), no_std)]

use sp_std::prelude::*;

use adapters::{BoundedPriorityQueue, BoundedDeque};
use codec::{Decode, Encode};
use core::cmp::{max, min, Ord, Ordering};
use fixed::{types::extra::U64, FixedU128};
use frame_support::{
	debug::native,
	decl_error, decl_event, decl_module, decl_storage,
	dispatch::{DispatchError, DispatchResult},
	ensure,
	traits::Get,
};
use num_rational::Ratio;
use orml_traits::BasicCurrency;
use sp_runtime::{
	traits::{CheckedMul, Zero},
	PerThing, Perbill, RuntimeDebug,
};
use sp_std::collections::vec_deque::VecDeque;
use frame_system::ensure_signed;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

/// Expected price oracle interface. `fetch_price` must return the amount of Coins exchanged for the tracked value.
pub trait FetchPrice<Balance> {
	/// Fetch the current price.
	fn fetch_price() -> Balance;
}

/// The pallet's configuration trait.
pub trait Trait: system::Trait {
	/// The overarching event type.
	type Event: From<Event<Self>> + Into<<Self as system::Trait>::Event>;

	/// The amount of Coins necessary to buy the tracked value. (e.g., 1_100 for 1$)
	type CoinPrice: FetchPrice<Coins>;
	/// The frequency of adjustments of the coin supply.
	type AdjustmentFrequency: Get<<Self as system::Trait>::BlockNumber>;
	/// The amount of Coins that are meant to track the value. Example: A value of 1_000 when tracking
	/// Dollars means that the Stablecoin will try to maintain a price of 1_000 Coins for 1$.
	type BaseUnit: Get<Coins>;
}
// The pallet's runtime storage items.
// https://substrate.dev/docs/en/knowledgebase/runtime/storage
decl_storage! {
    // A unique name is used to ensure that the pallet's storage items are isolated.
    // This name may be updated, but each pallet in the runtime must use a unique name.
    // ---------------------------------vvvvvvvvvvvvvv
    trait Store for Module<T: Trait> as SerpTes {
        // Learn more about declaring storage items:
        // https://substrate.dev/docs/en/knowledgebase/runtime/storage#declaring-storage-items
        Something get(fn something): Option<u32>;
    }
}

// Pallets use events to inform users when important changes are made.
// https://substrate.dev/docs/en/knowledgebase/runtime/events
decl_event!(
	pub enum Event<T>
	where
		AccountId = <T as system::Trait>::AccountId,
		BlockNumber = <T as system::Trait>::BlockNumber,
	{
		/// The supply was expanded by the amount.
		ExpandedSupply(u64),
		/// The supply was contracted by the amount.
		ContractedSupply(u64),
	}
);

// Errors inform users that something went wrong.
decl_error! {
	/// The possible errors returned by calls to this pallet's functions.
	pub enum Error for Module<T: Trait> {
		/// While trying to expand the supply, it overflowed.
		CoinSupplyOverflow,
		/// While trying to contract the supply, it underflowed.
		CoinSupplyUnderflow,
		BalanceOverflow,
		/// Something went very wrong and the price of the currency is zero.
		ZeroPrice,
		/// An arithmetic operation caused an overflow.
		GenericOverflow,
		/// An arithmetic operation caused an underflow.
		GenericUnderflow,
	}
}

// Dispatchable functions allows users to interact with the pallet and invoke state changes.
// These functions materialize as "extrinsics", which are often compared to transactions.
// Dispatchable functions must be annotated with a weight and must return a DispatchResult.
decl_module! {
	/// The pallet's dispatchable functions.
	pub struct Module<T: Trait> for enum Call where origin: T::Origin {
		const AdjustmentFrequency: T::BlockNumber = T::AdjustmentFrequency::get();

		/// Adjust the amount of Coins according to the price.
		///
		/// **Weight:**
		/// - complexity: `O(F + P)`
		///   - `F` being the complexity of `CoinPrice::fetch_price()`
		///   - `P` being the complexity of `on_block_with_price`
		fn on_initialize(n: T::BlockNumber) {
			let price = T::CoinPrice::fetch_price();
			Self::on_block_with_price(n, price).unwrap_or_else(|e| {
				native::error!("could not adjust supply: {:?}", e);
			});
		}
	}
}

/// Tries to contract the supply by `amount` by converting bids to bonds.
	///
	/// Note: Could contract the supply by less than `amount` if there are not enough bids.
	///
	/// **Weight:**
	/// - complexity: `O(BI + BO + C)`
	///   - `BI` being the number of bids in the bidding auction, limited to `MaximumBids`
	///   - `BO` being the number of newly created bonds, limited to `BI`
	///   - `C` being a constant amount of storage reads and writes for coin supply and bonds queue bounds bookkeeping
	/// - DB access:
	///   - 1 write for `coin_supply`
	///   - read and write bids
	///   - write `BO` newly created bonds + read and write bonds queue bounds
	///   - potentially refund up to `BI` bids
	fn contract_supply(coin_supply: Coins, amount: Coins) -> DispatchResult {
		// Checking whether coin supply would underflow.
		let remaining_supply = coin_supply
			.checked_sub(amount)
			.ok_or(Error::<T>::CoinSupplyUnderflow)?;
		if remaining_supply < T::MinimumSupply::get() {
			return Err(DispatchError::from(Error::<T>::CoinSupplyUnderflow));
		}
		// ↑ verify ↑
		let mut bids = Self::bids_transient();
		let mut remaining = amount;
		let mut new_bonds = VecDeque::new();
		// ↓ update ↓
		while remaining > 0 && !bids.is_empty() {
			let mut bid = bids
				.pop()
				.expect("checked whether queue is empty on previous line; qed");
			// the current bid can cover all the remaining contraction
			if bid.payment() >= remaining {
				match bid.remove_coins(remaining) {
					Err(_e) => {
						native::warn!("unable to remove coins from bid --> refunding bid: {:?}", bid);
						Self::refund_bid(&bid);
					}
					Ok(removed_quantity) => {
						new_bonds.push_back(Self::new_bond(bid.account.clone(), removed_quantity));
						// re-add bid with reduced amount
						if bid.quantity > 0 {
							bids.push(bid).map(|to_refund| Self::refund_bid(&to_refund));
						}
						remaining = 0;
					}
				}
			} else {
				let payment = bid.payment();
				let Bid {
					account, quantity, ..
				} = bid;
				new_bonds.push_back(Self::new_bond(account, quantity));
				remaining -= payment;
			}
		}
		debug_assert!(
			remaining <= amount,
			"remaining is never greater than the original amount"
		);
		let burned = amount.saturating_sub(remaining);
		debug_assert!(
			burned <= coin_supply,
			"burned <= amount < coin_supply is checked by coin underflow check in first lines"
		);
		let new_supply = coin_supply.saturating_sub(burned);
		for bond in new_bonds.iter() {
			Self::deposit_event(RawEvent::NewBond(
				bond.account.clone(),
				bond.payout,
				bond.expiration,
			));
		}
		let mut bonds = Self::bonds_transient();
		for bond in new_bonds {
			bonds.push_back(bond);
		}
		<CoinSupply>::put(new_supply);
		native::info!("contracted supply by: {}", burned);
		Self::deposit_event(RawEvent::ContractedSupply(burned));
		Ok(())
    }
    
    // ------------------------------------------------------------
	// expand supply

	/// Expand the supply by `amount` by paying out bonds and shares.
	///
	/// Will first pay out bonds and only pay out shares if there are no remaining
	/// bonds.
	///
	/// **Weight:**
	/// - complexity: `O(B + C + H)`
	///   - `B` being the number of bonds, bounded by ringbuffer size, currently `u16::max_value()`
	///   - `C` being a constant amount of storage reads and writes for coin supply and bonds queue bounds bookkeeping
	///   - `H` being the complexity of `hand_out_coins`
	/// - DB access:
	///   - read bonds + read and write bonds queue bounds
	///   - potentially write back 1 bond
	///   - 1 write for `coin_supply` OR read shares and execute `hand_out_coins` which has DB accesses
	fn expand_supply(coin_supply: Coins, amount: Coins) -> DispatchResult {
		// Checking whether the supply will overflow.
		coin_supply
			.checked_add(amount)
			.ok_or(Error::<T>::CoinSupplyOverflow)?;
		// ↑ verify ↑
		let mut remaining = amount;
		let mut bonds = Self::bonds_transient();
		// ↓ update ↓
		while let Some(Bond {
			account,
			payout,
			expiration,
		}) = if remaining > 0 { bonds.pop_front() } else { None }
		{
			// bond has expired --> discard
			if <system::Module<T>>::block_number() >= expiration {
				Self::deposit_event(RawEvent::BondExpired(account, payout));
				continue;
			}
			// bond does not cover the remaining amount --> resolve and continue
			if payout <= remaining {
				// this is safe because we are in the branch where remaining >= payout
				remaining -= payout;
				Self::add_balance(&account, payout);
				Self::deposit_event(RawEvent::BondFulfilled(account, payout));
			}
			// bond covers the remaining amount --> update and finish up
			else {
				// this is safe because we are in the else branch where payout > remaining
				let payout = payout - remaining;
				Self::add_balance(&account, remaining);
				bonds.push_front(Bond {
					account: account.clone(),
					payout,
					expiration,
				});
				Self::deposit_event(RawEvent::BondPartiallyFulfilled(account, payout));
				break;
			}
		}
		// safe to do this late because of the test in the first line of the function
		// safe to subtract remaining because we initialize it with amount and never increase it
		let new_supply = coin_supply + amount - remaining;
		native::info!("expanded supply by paying out bonds: {}", amount - remaining);
		if remaining > 0 {
			// relies on supply being updated in `hand_out_coins`
			Self::hand_out_coins(&Self::shares(), remaining, new_supply)
				.expect("coin supply overflow was checked at the beginning of function; qed");
		} else {
			<CoinSupply>::put(new_supply);
		}
		Self::deposit_event(RawEvent::ExpandedSupply(amount));
		Ok(())
	}

	/// Hand out Coins to shareholders according to their number of shares.
	///
	/// Will hand out more Coins to shareholders at the beginning of the list
	/// if the handout cannot be equal.
	///
	/// **Weight:**
	/// - complexity: `O(S + C)`
	///   - `S` being `shares.len()` (the number of shareholders)
	///   - `C` being a constant amount of storage reads and writes for coin supply
	/// - DB access:
	///   - 1 write for `coin_supply`
	///   - `S` amount of writes
	fn hand_out_coins(shares: &[(T::AccountId, u64)], amount: Coins, coin_supply: Coins) -> DispatchResult {
		// Checking whether the supply will overflow.
		coin_supply
			.checked_add(amount)
			.ok_or(Error::<T>::CoinSupplyOverflow)?;
		// ↑ verify ↑
		let share_supply: u64 = shares.iter().map(|(_a, s)| s).sum();
		let len = shares.len() as u64;
		// No point in giving out less than 1 coin.
		let coins_per_share = max(1, amount / share_supply);
		let pay_extra = coins_per_share * len < amount;
		let mut amount_payed = 0;
		// ↓ update ↓
		for (i, (acc, num_shares)) in shares.iter().enumerate() {
			if amount_payed >= amount {
				break;
			}
			let max_payout = amount - amount_payed;
			let is_in_first_mod_len = (i as u64) < amount % len;
			let extra_payout = if pay_extra && is_in_first_mod_len { 1 } else { 0 };
			let payout = min(max_payout, num_shares * coins_per_share + extra_payout);
			debug_assert!(
				amount_payed + payout <= amount,
				"amount payed out should be less or equal target amount"
			);
			Self::add_balance(&acc, payout);
			amount_payed += payout;
		}
		debug_assert!(
			amount_payed == amount,
			"amount payed out should equal target amount"
		);

		// safe to do this late because of the test in the first line of the function
		let new_supply = coin_supply + amount;
		<CoinSupply>::put(new_supply);
		native::info!("expanded supply by handing out coins: {}", amount);
		Ok(())
	}

	// ------------------------------------------------------------
	// on block

	/// Contracts or expands the supply based on conditions.
	///
	/// **Weight:**
	/// Calls `expand_or_contract_on_price` every `AdjustmentFrequency` blocks.
	/// - complexity: `O(P)` with `P` being the complexity of `expand_or_contract_on_price`
	fn on_block_with_price(block: T::BlockNumber, price: Coins) -> DispatchResult {
		// This can be changed to only correct for small or big price swings.
		if block % T::AdjustmentFrequency::get() == 0.into() {
			Self::expand_or_contract_on_price(price)
		} else {
			Ok(())
		}
	}

	/// Expands (if the price is too high) or contracts (if the price is too low) the coin supply.
	///
	/// **Weight:**
	/// - complexity: `O(S + C)`
	///   - `S` being the complexity of executing either `expand_supply` or `contract_supply`
	///   - `C` being a constant amount of storage reads for coin supply
	/// - DB access:
	///   - 1 read for coin_supply
	///   - execute `expand_supply` OR execute `contract_supply` which have DB accesses
	fn expand_or_contract_on_price(price: Coins) -> DispatchResult {
		match price {
			0 => {
				native::error!("coin price is zero!");
				return Err(DispatchError::from(Error::<T>::ZeroPrice));
			}
			price if price > T::BaseUnit::get() => {
				// safe from underflow because `price` is checked to be greater than `BaseUnit`
				let supply = Self::coin_supply();
				let contract_by = Self::calculate_supply_change(price, T::BaseUnit::get(), supply);
				Self::contract_supply(supply, contract_by)?;
			}
			price if price < T::BaseUnit::get() => {
				// safe from underflow because `price` is checked to be less than `BaseUnit`
				let supply = Self::coin_supply();
				let expand_by = Self::calculate_supply_change(T::BaseUnit::get(), price, supply);
				Self::expand_supply(supply, expand_by)?;
			}
			_ => {
				native::info!("coin price is equal to base as is desired --> nothing to do");
			}
		}
		Ok(())
	}

	/// Calculate the amount of supply change from a fraction given as `numerator` and `denominator`.
	fn calculate_supply_change(numerator: u64, denominator: u64, supply: u64) -> u64 {
		type Fix = FixedU128<U64>;
		let fraction = Fix::from_num(numerator) / Fix::from_num(denominator) - Fix::from_num(1);
		fraction.saturating_mul_int(supply as u128).to_num::<u64>()
	}