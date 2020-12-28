#![cfg_attr(not(feature = "std"), no_std)]

use codec::{Decode, Encode};
use core::cmp::Ord;
use sp_runtime::Perbill;
use frame_support::{
	sp_runtime, decl_error, decl_event, decl_module, decl_storage,traits::Get,
};
use frame_system::Module;

#[cfg(test)]
mod tests;

/// Expected price oracle interface. `fetch_price` must return the amount of SettCurrency exchanged for the tracked value.
pub trait FetchPrice<Balance> {
	/// Fetch the current price.
	fn fetch_price() -> Balance;
}

/// The type used to represent the account balance for the Setheum SettCurrencys.
pub type SettCurrency = u32;

pub type DinarIndex = u32;

/// The pallet's configuration trait.
pub trait Trait: frame_system::Trait {
	/// The overarching event type.
	type Event: From<Event<Self>> + Into<<Self as frame_system::Trait>::Event>;

	/// The amount of SettCurrency necessary to buy the tracked value. (e.g., 1_100 for 1$)
    type SettCurrencyPrice: FetchPrice<SettCurrency>; 
    
    /// The maximum amount of bids allowed in the queue. Used to prevent the queue from growing forever.
    type MaximumBids: Get<u64>;
    
    /// The minimum percentage to pay for a Dinar.. 
	/// minimum price of 10%.
	type MinimumDinarPrice: Get<Perbill>;
    
    /// The amount of SettCurrency that are meant to track the value. Example: A value of 1_000 when tracking
	/// Dollars means that the SettCurrency will try to maintain a price of 1_000 SettCurrency for 1$.
	type BaseUnit: Get<SettCurrency>;
    
    /// The initial supply of SettCurrency.
	type InitialSupply: Get<SettCurrency>;
    
    /// The minimum amount of SettCurrency in circulation.
	/// Must be lower than `InitialSupply`.
	type MinimumSupply: Get<SettCurrency>;
}


/// A Dinar representing (potential) future payout of SettCurrency.+
///
/// + `account` is the recipient of the Dinar payout.
/// + `payout` is the amount of SettCurrency payed out.
#[derive(Encode, Decode, Default, Clone, PartialEq, PartialOrd, Eq, Ord)]
pub struct Dinar<AccountId> {
	account: AccountId,
	payout: SettCurrency,
}

decl_event!(
	pub enum Event<T>
	where
		AccountId = <T as frame_system::Trait>::AccountId,
	{
		/// Successful transfer from the first to the second account.
		Transfer(AccountId, AccountId, u64),
		/// The supply was expanded by the amount.
        ExpandedSupply(u64),
        /// A new Dinar was created for the account with payout.
		NewDinar(AccountId, u64,),
		/// A Dinar was payed out to the account.
		DinarFulfilled(AccountId, u64),
		/// A Dinar was partially payed out to the account.
		DinarPartiallyFulfilled(AccountId, u64),
	}
);

decl_error! {
	/// The possible errors returned by calls to this pallet's functions.
	pub enum Error for Module<T: Trait> {
		/// While trying to expand the supply, it overflowed.
		SettCurrencySupplyOverflow,
		/// While trying to contract the supply, it underflowed.
		SettCurrencySupplyUnderflow,
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
        /// *Slot Shares*
		/// The allocation of slot shares to accounts.
		/// This is a `Vec` and thus should be limited to few shareholders (< 1_000).
		/// In principle it would be possible to make shares tradeable. In that case
		/// we would have to use a map similar to the `Balance` one.
        Shares get(fn shares): Vec<(T::AccountId, u64)>;
        
        /// *SettCurrency*
		/// The balance of SettCurrencys associated with each account.
		Balance get(fn get_balance): map hasher(blake2_128_concat) T::AccountId => SettCurrency;

		/// The total amount of SettCurrency in circulation.
        SettCurrencySupply get(fn settcurrency_supply): SettCurrency = 0;
		
		/// *Dinar*
		/// The available dinar for contracting supply.
		Dinar get(fn get_dinar): map hasher(twox_64_concat) DinarIndex => Dinar<T::AccountId>;
		/// Start and end index pair used to implement a ringbuffer on top of the `Dinar` map.
		DinarRange get(fn dinar_range): (DinarIndex, DinarIndex) = (0, 0);
	}

	add_extra_genesis {
		/// The shareholders to initialize the SettCurrencys with.
		config(shareholders):
			Vec<(T::AccountId, u64)>;
		build(|config: &GenesisConfig<T>| {
			assert!(
				T::MinimumSupply::get() < T::InitialSupply::get(),
				"initial settcurrency supply needs to be greater than the minimum"
			);

			assert!(!config.shareholders.is_empty(), "need at least one shareholder");
			// TODO: make sure shareholders are unique?

			// Hand out the initial settcurrency supply to the shareholders.
			<Module<T>>::hand_out_settcurrency(&config.shareholders, T::InitialSupply::get(), <Module<T>>::settcurrency_supply())
				.expect("initialization handout should not fail");

			// Store the shareholders with their shares.
			<Shares<T>>::put(&config.shareholders);
		});
	}

	///-----------------------------------------------------------------------------
	/// Dinar
	///
	/// Create a new dinar for the given account with the given payout.
	
	fn new_dinar(account: T::AccountId, payout: SettCurrency) -> Dinar<T::AccountId> {
		Dinar {
			account,
			payout,

		}
	}

	/// Create a new transient storage adapter that manages the Dinar.
	///
	/// Allows pushing and popping on a ringbuffer without managing the storage details.
	fn dinar_transient() -> BoundedDeque<
		Dinar<T::AccountId>,
		<Self as Store>::DinarRange,
		<Self as Store>::Dinar,
		DinarIndex,
	> {
		BoundedDeque::<
			Dinar<T::AccountId>,
			<Self as Store>::DinarRange,
			<Self as Store>::Dinar,
			DinarIndex,
		>::new()
	}

}

decl_module! {
	/// The pallet's dispatchable functions.
	pub struct Module<T: Trait> for enum Call where origin: T::Origin {
		/// The amount of SettCurrencys that represent 1 external value (e.g., 1$).
		const BaseUnit: SettCurrency = T::BaseUnit::get();
		/// The minimum amount of SettCurrency that will be in circulation.
		const MinimumSupply: SettCurrency = T::MinimumSupply::get();

		fn deposit_event() = default;

		#[weight = 10_000 + T::DbWeight::get().reads_writes(1,1)]
		// - complexity: `O(1)`
		// - DB access: 2 storage map reads + 2 storage map writes
		pub fn send_settcurrency(origin, to: T::AccountId, amount: u64) -> DispatchResult {
			let sender = ensure_signed(origin)?;
			Self::transfer_from_to(&sender, &to, amount)?;
			Self::deposit_event(RawEvent::Transfer(sender, to, amount));
			Ok(())
		}

		// Implement the BasicCurrency to allow other pallets to interact programmatically
		// with the SettCurrencys.
		impl<T: Trait> BasicCurrency<T::AccountId> for Module<T> {
			type Balance = SettCurrency;

			/// Return the amount of SettCurrency in circulation.
			///
			#[weight = 10_000 + T::DbWeight::get().reads(1)]
			/// - complexity: `O(1)`
			/// - DB access: 1 read
			fn total_issuance() -> Self::Balance {
				Self::settcurrency_supply()
			}

			/// Return the balance of the given account.
			///
			#[weight = 10_000 + T::DbWeight::get().reads(1)]
			/// - complexity: `O(1)`
			/// - DB access: 1 read from balance storage map
			fn total_balance(who: &T::AccountId) -> Self::Balance {
				Self::get_balance(who)
			}

			/// Return the free balance of the given account.
			///
			/// Equal to `total_balance` for this SettCurrencys.
			///
			#[weight = 10_000 + T::DbWeight::get().reads(1)]
			/// - complexity: `O(1)`
			/// - DB access: 1 read from balance storage map
			fn free_balance(who: &T::AccountId) -> Self::Balance {
				Self::get_balance(who)
			}

			/// Cannot withdraw from SettCurrencys accounts. Returns `Ok(())` if `amount` is 0, otherwise returns an error.
			fn ensure_can_withdraw(_who: &T::AccountId, amount: Self::Balance) -> DispatchResult {
				if amount.is_zero() {
					return Ok(());
				}
				Err(DispatchError::Other("cannot change issuance for SettCurrencys"))
			}

			/// Transfer `amount` from one account to another.
			///
			#[weight = 10_000 + T::DbWeight::get().reads_writes(1)]
			/// - complexity: `O(1)`
			/// - DB access: 2 reads and write from and to balance storage map
			fn transfer(from: &T::AccountId, to: &T::AccountId, amount: Self::Balance) -> DispatchResult {
				Self::transfer_from_to(from, to, amount)
			}

			/// Noop that returns an error. Cannot change the issuance of a stables.
			fn deposit(_who: &T::AccountId, _amount: Self::Balance) -> DispatchResult {
				Err(DispatchError::Other("cannot change issuance for SettCurrencys"))
			}

			/// Noop that returns an error. Cannot change the issuance of a SettCurrencys.
			fn withdraw(_who: &T::AccountId, _amount: Self::Balance) -> DispatchResult {
				Err(DispatchError::Other("cannot change issuance for SettCurrencys"))
			}

			/// Test whether the given account can be slashed with `value`.
			///
			#[weight = 10_000 + T::DbWeight::get().reads(1)]
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
			/// If the account does not have `amount` SettCurrency it will be slashed to 0
			/// and that amount returned.
			///
			#[weight = 10_000 + T::DbWeight::get().writes(1)]
			/// - complexity: `O(1)`
			/// - DB access: 1 write to balance storage map
			fn slash(who: &T::AccountId, amount: Self::Balance) -> Self::Balance {
				let mut remaining: SettCurrency = 0;
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

			/// Transfer `amount` of SettCurrency from one account to another.
			///
			#[weight = 10_000 + T::DbWeight::get().reads_writes(1)]
			/// - complexity: `O(1)`
			/// - DB access: 2 storage map reads + 2 storage map writes
			fn transfer_from_to(from: &T::AccountId, to: &T::AccountId, amount: SettCurrency) -> DispatchResult {
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

			/// Add `amount` SettCurrency to the balance for `account`.
			///
			#[weight = 10_000 + T::DbWeight::get().writes(1)]
			/// - complexity: `O(1)`
			/// - DB access: 1 write to balance storage map
			fn add_balance(account: &T::AccountId, amount: SettCurrency) {
				<Balance<T>>::mutate(account, |b: &mut u64| {
					*b = b.saturating_add(amount);
					*b
				});
			}

			/// Remove `amount` SettCurrency from the balance of `account`.
			///
			#[weight = 10_000 + T::DbWeight::get().writes(1)]
			/// - complexity: `O(1)`
			/// - DB access: 1 write to balance storage map
			fn remove_balance(account: &T::AccountId, amount: SettCurrency) -> DispatchResult {
				<Balance<T>>::try_mutate(&account, |b: &mut u64| -> DispatchResult {
					*b = b.checked_sub(amount).ok_or(Error::<T>::InsufficientBalance)?;
					Ok(())
				});
			}
		}
	}
}



