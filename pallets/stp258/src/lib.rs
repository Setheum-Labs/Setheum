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
mod tests;

/// Expected price oracle interface. `fetch_price` must return the amount of Coins exchanged for the tracked value.
pub trait FetchPrice<Balance> {
	/// Fetch the current price.
	fn fetch_price() -> Balance;
}

/// The type used to represent the account balance for the stablecoin.
pub type Coins = u64;

/// The pallet's configuration trait.
pub trait Trait: system::Trait {
	/// The overarching event type.
	type Event: From<Event<Self>> + Into<<Self as system::Trait>::Event>;

	/// The amount of Coins necessary to buy the tracked value. (e.g., 1_100 for 1$)
    type CoinPrice: FetchPrice<Coins>; 

    /// The expiration time of a bond.
	/// The [Basis Whitepaper](https://www.basis.io/basis_whitepaper_en.pdf) recommends an expiration
	/// period of 5 years.
	type ExpirationPeriod: Get<<Self as system::Trait>::BlockNumber>;
    
    /// The maximum amount of bids allowed in the queue. Used to prevent the queue from growing forever.
    type MaximumBids: Get<u64>;
    
    /// The minimum percentage to pay for a bond.. recommending a minimum
	/// bond price of 10%.
	type MinimumBondPrice: Get<Perbill>;
    
    /// The amount of Coins that are meant to track the value. Example: A value of 1_000 when tracking
	/// Dollars means that the Stablecoin will try to maintain a price of 1_000 Coins for 1$.
	type BaseUnit: Get<Coins>;
    
    /// The initial supply of Coins.
	type InitialSupply: Get<Coins>;
    
    /// The minimum amount of Coins in circulation.
	/// Must be lower than `InitialSupply`.
	type MinimumSupply: Get<Coins>;
}

/// A bond representing (potential) future payout of Coins.
///
/// Expires at block `expiration` so it will be discarded if payed out after that block.
///
/// + `account` is the recipient of the bond payout.
/// + `payout` is the amount of Coins payed out.
#[derive(Encode, Decode, Default, Clone, PartialEq, PartialOrd, Eq, Ord, RuntimeDebug)]
pub struct Bond<AccountId, BlockNumber> {
	account: AccountId,
	payout: Coins,
	expiration: BlockNumber,
}

decl_event!(
	pub enum Event<T>
	where
		AccountId = <T as system::Trait>::AccountId,
		BlockNumber = <T as system::Trait>::BlockNumber,
	{
		/// Successful transfer from the first to the second account.
		Transfer(AccountId, AccountId, u64),
		/// The supply was expanded by the amount.
        ExpandedSupply(u64),
        /// A new bond was created for the account with payout and expiration.
		NewBond(AccountId, u64, BlockNumber),
		/// A bond was payed out to the account.
		BondFulfilled(AccountId, u64),
		/// A bond was partially payed out to the account.
		BondPartiallyFulfilled(AccountId, u64),
	}
);

decl_error! {
	/// The possible errors returned by calls to this pallet's functions.
	pub enum Error for Module<T: Trait> {
		/// While trying to expand the supply, it overflowed.
		CoinSupplyOverflow,
		/// While trying to contract the supply, it underflowed.
		CoinSupplyUnderflow,
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
	}
}

// This pallet's storage items.
decl_storage! {
	trait Store for Module<T: Trait> as Stp258 {
        /// *Shares*
		/// The allocation of shares to accounts.
		/// This is a `Vec` and thus should be limited to few shareholders (< 1_000).
		/// In principle it would be possible to make shares tradeable. In that case
		/// we would have to use a map similar to the `Balance` one.
        Shares get(fn shares): Vec<(T::AccountId, u64)>;
        
        /// *Coins*
		/// The balance of stablecoin associated with each account.
		Balance get(fn get_balance): map hasher(blake2_128_concat) T::AccountId => Coins;

		/// The total amount of Coins in circulation.
        CoinSupply get(fn coin_supply): Coins = 0;
        

        /// *Bonds*
        /// The available bonds for contracting supply.
		Bonds get(fn get_bond): map hasher(twox_64_concat) BondIndex => Bond<T::AccountId, T::BlockNumber>;
		/// Start and end index pair used to implement a ringbuffer on top of the `Bonds` map.
		BondsRange get(fn bonds_range): (BondIndex, BondIndex) = (0, 0);
	}
	add_extra_genesis {
		/// The shareholders to initialize the stablecoin with.
		config(shareholders):
			Vec<(T::AccountId, u64)>;
		build(|config: &GenesisConfig<T>| {
			assert!(
				T::MinimumSupply::get() < T::InitialSupply::get(),
				"initial coin supply needs to be greater than the minimum"
			);

			assert!(!config.shareholders.is_empty(), "need at least one shareholder");
			// TODO: make sure shareholders are unique?

			// Hand out the initial coin supply to the shareholders.
			<Module<T>>::hand_out_coins(&config.shareholders, T::InitialSupply::get(), <Module<T>>::coin_supply())
				.expect("initialization handout should not fail");

			// Store the shareholders with their shares.
			<Shares<T>>::put(&config.shareholders);
		});
	}
}

decl_module! {
	/// The pallet's dispatchable functions.
	pub struct Module<T: Trait> for enum Call where origin: T::Origin {
		/// The amount of stablecoins that represent 1 external value (e.g., 1$).
		const BaseUnit: Coins = T::BaseUnit::get();
		/// The minimum amount of Coins that will be in circulation.
		const MinimumSupply: Coins = T::MinimumSupply::get();

		fn deposit_event() = default;

		/// Transfer `amount` Coins from the sender to the account `to`.
		///
		/// **Weight:**
		/// - complexity: `O(1)`
		/// - DB access: 2 storage map reads + 2 storage map writes
		pub fn send_coins(origin, to: T::AccountId, amount: u64) -> DispatchResult {
			let sender = ensure_signed(origin)?;
			Self::transfer_from_to(&sender, &to, amount)?;
			Self::deposit_event(RawEvent::Transfer(sender, to, amount));
			Ok(())
		}
	}
}


// ------------------------------------------------------------
	// bonds

	/// Create a new bond for the given `account` with the given `payout`.
	///
	/// Expiration is calculated based on the current `block_number` and the configured
	/// `ExpirationPeriod`.
	fn new_bond(account: T::AccountId, payout: Coins) -> Bond<T::AccountId, T::BlockNumber> {
		let expiration = <system::Module<T>>::block_number() + T::ExpirationPeriod::get();
		Bond {
			account,
			payout,
			expiration,
		}
	}

	/// Create a new transient storage adapter that manages the bonds.
	///
	/// Allows pushing and popping on a ringbuffer without managing the storage details.
	fn bonds_transient() -> BoundedDeque<
		Bond<T::AccountId, T::BlockNumber>,
		<Self as Store>::BondsRange,
		<Self as Store>::Bonds,
		BondIndex,
	> {
		BoundedDeque::<
			Bond<T::AccountId, T::BlockNumber>,
			<Self as Store>::BondsRange,
			<Self as Store>::Bonds,
			BondIndex,
		>::new()
    }

// Implement the BasicCurrency to allow other pallets to interact programmatically
// with the Stablecoin.
impl<T: Trait> BasicCurrency<T::AccountId> for Module<T> {
	type Balance = Coins;

	/// Return the amount of Coins in circulation.
	///
	/// **Weight:**
	/// - complexity: `O(1)`
	/// - DB access: 1 read
	fn total_issuance() -> Self::Balance {
		Self::coin_supply()
	}

	/// Return the balance of the given account.
	///
	/// **Weight:**
	/// - complexity: `O(1)`
	/// - DB access: 1 read from balance storage map
	fn total_balance(who: &T::AccountId) -> Self::Balance {
		Self::get_balance(who)
	}

	/// Return the free balance of the given account.
	///
	/// Equal to `total_balance` for this stablecoin.
	///
	/// **Weight:**
	/// - complexity: `O(1)`
	/// - DB access: 1 read from balance storage map
	fn free_balance(who: &T::AccountId) -> Self::Balance {
		Self::get_balance(who)
	}

	/// Cannot withdraw from stablecoin accounts. Returns `Ok(())` if `amount` is 0, otherwise returns an error.
	fn ensure_can_withdraw(_who: &T::AccountId, amount: Self::Balance) -> DispatchResult {
		if amount.is_zero() {
			return Ok(());
		}
		Err(DispatchError::Other("cannot change issuance for stablecoins"))
	}

	/// Transfer `amount` from one account to another.
	///
	/// **Weight:**
	/// - complexity: `O(1)`
	/// - DB access: 2 reads and write from and to balance storage map
	fn transfer(from: &T::AccountId, to: &T::AccountId, amount: Self::Balance) -> DispatchResult {
		Self::transfer_from_to(from, to, amount)
	}

	/// Noop that returns an error. Cannot change the issuance of a stablecoin.
	fn deposit(_who: &T::AccountId, _amount: Self::Balance) -> DispatchResult {
		Err(DispatchError::Other("cannot change issuance for stablecoins"))
	}

	/// Noop that returns an error. Cannot change the issuance of a stablecoin.
	fn withdraw(_who: &T::AccountId, _amount: Self::Balance) -> DispatchResult {
		Err(DispatchError::Other("cannot change issuance for stablecoins"))
	}

	/// Test whether the given account can be slashed with `value`.
	///
	/// **Weight:**
	/// - complexity: `O(1)`
	/// - DB access: 1 read from balance storage map
	fn can_slash(who: &T::AccountId, value: Self::Balance) -> bool {
		if value.is_zero() {
			return true;
		}
		Self::get_balance(who) >= value
	}

	/// Slash account `who` by `amount` returning the actual amount slashed.
	///
	/// If the account does not have `amount` Coins it will be slashed to 0
	/// and that amount returned.
	///
	/// **Weight:**
	/// - complexity: `O(1)`
	/// - DB access: 1 write to balance storage map
	fn slash(who: &T::AccountId, amount: Self::Balance) -> Self::Balance {
		let mut remaining: Coins = 0;
		<Balance<T>>::mutate(who, |b: &mut u64| {
			if *b < amount {
				remaining = amount - *b;
				*b = 0;
			} else {
				*b = b.saturating_sub(amount);
			}
		});
		remaining
	}
}

impl<T: Trait> Module<T> {
	// ------------------------------------------------------------
	// balances

	/// Transfer `amount` of Coins from one account to another.
	///
	/// **Weight:**
	/// - complexity: `O(1)`
	/// - DB access: 2 storage map reads + 2 storage map writes
	fn transfer_from_to(from: &T::AccountId, to: &T::AccountId, amount: Coins) -> DispatchResult {
		let from_balance = Self::get_balance(from);
		let updated_from_balance = from_balance
			.checked_sub(amount)
			.ok_or(Error::<T>::InsufficientBalance)?;
		let receiver_balance = Self::get_balance(&to);
		let updated_to_balance = receiver_balance
			.checked_add(amount)
			.ok_or(Error::<T>::BalanceOverflow)?;

		// ↑ verify ↑
		// ↓ update ↓

		// reduce from's balance
		<Balance<T>>::insert(&from, updated_from_balance);
		// increase receiver's balance
		<Balance<T>>::insert(&to, updated_to_balance);

		Ok(())
	}

	/// Add `amount` Coins to the balance for `account`.
	///
	/// **Weight:**
	/// - complexity: `O(1)`
	/// - DB access: 1 write to balance storage map
	fn add_balance(account: &T::AccountId, amount: Coins) {
		<Balance<T>>::mutate(account, |b: &mut u64| {
			*b = b.saturating_add(amount);
			*b
		});
	}

	/// Remove `amount` Coins from the balance of `account`.
	///
	/// **Weight:**
	/// - complexity: `O(1)`
	/// - DB access: 1 write to balance storage map
	fn remove_balance(account: &T::AccountId, amount: Coins) -> DispatchResult {
		<Balance<T>>::try_mutate(&account, |b: &mut u64| -> DispatchResult {
			*b = b.checked_sub(amount).ok_or(Error::<T>::InsufficientBalance)?;
			Ok(())
		})
	}