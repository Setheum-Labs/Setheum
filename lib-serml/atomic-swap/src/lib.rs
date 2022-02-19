// بِسْمِ اللَّهِ الرَّحْمَنِ الرَّحِيم

// This file is part of Setheum.

// Copyright (C) 2019-2021 Setheum Labs.
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

//! # Atomic Swap
//!
//! A pallet for atomically sending funds.
//!
//! - [`Config`]
//! - [`Call`]
//! - [`Pallet`]
//!
//! ## Overview
//!
//! A pallet for atomically sending multicurrency funds from an origin to a target.
//! A proof is used to allow the target to approve (claim) the swap. 
//! If the swap is not claimed within a specified duration of time,
//! the sender may cancel it.
//!
//! ## Interface
//!
//! ### Dispatchable Functions
//!
//! * [`create_swap`](Call::create_swap) - called by a sender to register a new atomic swap
//! * [`claim_swap`](Call::claim_swap) - called by the target to approve a swap
//! * [`cancel_swap`](Call::cancel_swap) - may be called by a sender after a specified duration

// Ensure we're `no_std` when compiling for Wasm.
#![cfg_attr(not(feature = "std"), no_std)]

mod tests;

use codec::{Decode, Encode};
use frame_support::{
	dispatch::DispatchResult,
	traits::Get,
	weights::Weight,
	RuntimeDebugNoBound,
};
use scale_info::TypeInfo;
use sp_io::hashing::blake2_256;
use sp_runtime::RuntimeDebug;
use sp_std::{
	marker::PhantomData,
	ops::{Deref, DerefMut},
	prelude::*,
};
use orml_traits::{BalanceStatus, MultiCurrency, MultiReservableCurrency};
use primitives::{Balance, BlockNumber, CurrencyId, Moment};

/// Pending atomic swap operation.
#[derive(Clone, Eq, PartialEq, RuntimeDebugNoBound, Encode, Decode, TypeInfo)]
#[scale_info(skip_type_params(T))]
pub struct PendingSwap<T: Config> {
	/// Source of the swap.
	pub source: T::AccountId,
    /// Currency of the swap.
    pub currency: CurrencyId,
	/// Action of this swap.
	pub action: T::SwapAction,
	/// End block of the lock.
	pub end_block: T::BlockNumber,
}

/// Hashed proof type.
pub type HashedProof = [u8; 32];

/// Minutes defined in blocktime (number of blocks in a minute).
pub const SECS_PER_BLOCK: Moment = 6; // 6 seconds blocktime
pub const MINUTES: BlockNumber = 60 / (SECS_PER_BLOCK as BlockNumber);

/// Minutes type
pub type Minutes = BlockNumber;


/// Definition of a pending atomic swap action. It contains the following three phrases:
///
/// - **Reserve**: reserve the resources needed for a swap. This is to make sure that **Claim**
/// succeeds with best efforts.
/// - **Claim**: claim any resources reserved in the first phrase.
/// - **Cancel**: cancel any resources reserved in the first phrase.
pub trait SwapAction<AccountId, T: Config> {
	/// Reserve the resources needed for the swap, from the given `source`. The reservation is
	/// allowed to fail. If that is the case, the the full swap creation operation is cancelled.
	fn reserve(&self, currency: CurrencyId, source: &AccountId) -> DispatchResult;
	/// Claim the reserved resources of `currency`, `source` and `target`. Returns whether the claim
	/// succeeds.
	fn claim(&self, currency: CurrencyId, source: &AccountId, target: &AccountId) -> bool;
	/// Weight for executing the operation.
	fn weight(&self) -> Weight;
	/// Cancel the resources of `currency` reserved in `source`.
	fn cancel(&self, currency: CurrencyId, source: &AccountId);
}

/// A swap action that only allows transferring balances.
#[derive(Clone, RuntimeDebug, Eq, PartialEq, Encode, Decode, TypeInfo)]
#[scale_info(skip_type_params(C))]
pub struct MultiCurrencySwapAction<AccountId, C: MultiReservableCurrency<AccountId>> {
	value: <C as MultiCurrency<<T as frame_system::Config>::AccountId>>::Balance,
	_marker: PhantomData<C>,
}

impl<AccountId, C> MultiCurrencySwapAction<AccountId, C>
where
	C: MultiReservableCurrency<AccountId>,
{
	/// Create a new swap action value of balance.
	pub fn new(value: <C as MultiCurrency<<T as frame_system::Config>::AccountId>>::Balance) -> Self {
		Self { value, _marker: PhantomData }
	}
}

impl<AccountId, C> Deref for MultiCurrencySwapAction<AccountId, C>
where
	C: MultiReservableCurrency<AccountId>,
{
	type Target = <C as MultiCurrency<<T as frame_system::Config>::AccountId>>::Balance;

	fn deref(&self) -> &Self::Target {
		&self.value
	}
}

impl<AccountId, C> DerefMut for MultiCurrencySwapAction<AccountId, C>
where
	C: MultiReservableCurrency<AccountId>,
{
	fn deref_mut(&mut self) -> &mut Self::Target {
		&mut self.value
	}
}

impl<T: Config, AccountId, C> SwapAction<AccountId, T> for MultiCurrencySwapAction<AccountId, C>
where
	C: MultiReservableCurrency<AccountId>,
{
	fn reserve(&self, currency: CurrencyId, source: &AccountId) -> DispatchResult {
		C::reserve(currency, &source, self.value)
	}

	fn claim(&self, currency: CurrencyId, source: &AccountId, target: &AccountId) -> bool {
		C::repatriate_reserved(currency, source, target, self.value, BalanceStatus::Free).is_ok()
	}

	fn weight(&self) -> Weight {
		T::DbWeight::get().reads_writes(1, 1)
	}

	fn cancel(&self, currency: CurrencyId, source: &AccountId) {
		C::unreserve(currency, source, self.value);
	}
}

pub use pallet::*;

pub(crate) type BalanceOf<T> = <<T as Config>::MultiCurrency as MultiCurrency<<T as frame_system::Config>::AccountId>>::Balance;
pub(crate) type CurrencyIdOf<T> =
	<<T as Config>::MultiCurrency as MultiCurrency<<T as frame_system::Config>::AccountId>>::CurrencyId;

#[frame_support::pallet]
pub mod pallet {
	use super::*;
	use frame_support::pallet_prelude::*;
	use frame_system::pallet_prelude::*;

	/// Atomic swap's pallet configuration trait.
	#[pallet::config]
	pub trait Config: frame_system::Config {
		/// The overarching event type.
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
		
        /// The Currency for managing assets related to atomic swaps.
		type MultiCurrency: MultiReservableCurrency<Self::AccountId, CurrencyId = CurrencyId, Balance = Balance>;
		
        /// Swap action.
		type SwapAction: SwapAction<Self::AccountId, Self> + Parameter;
		
        /// Limit of proof size.
		///
		/// Atomic swap is only atomic if once the proof is revealed, both parties can submit the
		/// proofs on-chain. If A is the one that generates the proof, then it requires that either:
		/// - A's blockchain has the same proof length limit as B's blockchain.
		/// - Or A's blockchain has shorter proof length limit as B's blockchain.
		///
		/// If B sees A is on a blockchain with larger proof length limit, then it should kindly
		/// refuse to accept the atomic swap request if A generates the proof, and asks that B
		/// generates the proof instead.
		#[pallet::constant]
		type ProofLimit: Get<u32>;
	}

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	#[pallet::without_storage_info]
	pub struct Pallet<T>(PhantomData<T>);

	#[pallet::storage]
	pub type PendingSwaps<T: Config> = StorageDoubleMap<
		_,
		Twox64Concat,
		T::AccountId,
		Blake2_128Concat,
		(CurrencyId, HashedProof),
		PendingSwap<T>,
	>;

	#[pallet::error]
	pub enum Error<T> {
		/// Swap already exists.
		AlreadyExist,
		/// Swap proof is invalid.
		InvalidProof,
		/// Proof is too large.
		ProofTooLarge,
		/// Source does not match.
		SourceMismatch,
		/// Swap has already been claimed.
		AlreadyClaimed,
		/// Swap does not exist.
		NotExist,
		/// Claim action mismatch.
		ClaimActionMismatch,
        /// Claim currency mismatch.
        ClaimCurrencyMismatch,
		/// Duration has not yet passed for the swap to be cancelled.
		DurationNotPassed,
	}

	/// Event of atomic swap pallet.
	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// Swap created.
		NewSwap { account: T::AccountId, proof: HashedProof, swap: PendingSwap<T> },
		/// Swap claimed. The last parameter indicates whether the execution succeeds.
		SwapClaimed { account: T::AccountId, proof: HashedProof, swap: PendingSwap<T>, success: bool },
		/// Swap cancelled.
		SwapCancelled { account: T::AccountId, swap: PendingSwap<T>, proof: HashedProof },
	}

	/// Old name generated by `decl_event`.
	#[deprecated(note = "use `Event` instead")]
	pub type RawEvent<T> = Event<T>;

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// Register a new atomic swap, declaring an intention to send multicurrency funds from origin to target
		/// on the blockchain. The target can claim the fund using the revealed proof. If
		/// the fund is not claimed after `duration` blocks, then the sender can cancel the swap.
		///
		/// The dispatch origin for this call must be _Signed_.
		///
		/// - `currency`: Currency of the atomic swap.
		/// - `target`: Receiver of the atomic swap.
		/// - `hashed_proof`: The blake2_256 hash of the secret proof.
		/// - `balance`: Funds to be sent from origin.
		/// - `duration`: Locked duration of the atomic swap. For safety reasons, it is recommended
		///   that the revealer uses a shorter duration than the counterparty, to prevent the
		///   situation where the revealer reveals the proof too late around the end block.
        ///   The `duration` is counted in minutes
		#[pallet::weight(T::DbWeight::get().reads_writes(1, 1).saturating_add(40_000_000))]
		pub fn create_swap(
			origin: OriginFor<T>,
			currency: CurrencyIdOf<T>,
			target: T::AccountId,
			hashed_proof: HashedProof,
			action: T::SwapAction,
			duration: Minutes,
		) -> DispatchResult {
			let source = ensure_signed(origin)?;

            let blocktime: BlockNumber = MINUTES * (duration as BlockNumber);

			ensure!(
				!PendingSwaps::<T>::contains_key(&target, (currency, hashed_proof)),
				Error::<T>::AlreadyExist
			);

			action.reserve(currency, &source)?;

			let swap = PendingSwap {
				source,
                currency,
				action,
				end_block: frame_system::Pallet::<T>::block_number() + blocktime,
			};
			PendingSwaps::<T>::insert(target.clone(), (currency.clone(), hashed_proof.clone()), swap.clone());

			Self::deposit_event(Event::NewSwap { account: target, proof: hashed_proof, swap });

			Ok(())
		}

		/// Claim an atomic swap.
		///
		/// The dispatch origin for this call must be _Signed_.
		///
		/// - `proof`: Revealed proof of the claim.
		/// - `action`: Action defined in the swap, it must match the entry in blockchain. Otherwise
		///   the operation fails. This is used for weight calculation.
		#[pallet::weight(
			T::DbWeight::get().reads_writes(1, 1)
				.saturating_add(40_000_000)
				.saturating_add((proof.len() as Weight).saturating_mul(100))
				.saturating_add(action.weight())
		)]
		pub fn claim_swap(
			origin: OriginFor<T>,
			currency: CurrencyIdOf<T>,
			proof: Vec<u8>,
			action: T::SwapAction,
		) -> DispatchResult {
			ensure!(proof.len() <= T::ProofLimit::get() as usize, Error::<T>::ProofTooLarge);

			let target = ensure_signed(origin)?;
			let hashed_proof = blake2_256(&proof);

			let swap =
				PendingSwaps::<T>::get(&target, (currency, hashed_proof)).ok_or(Error::<T>::InvalidProof)?;
			ensure!(swap.action == action, Error::<T>::ClaimActionMismatch);
			ensure!(swap.currency == currency, Error::<T>::ClaimCurrencyMismatch);

			let succeeded = swap.action.claim(&swap.source, &target);

			PendingSwaps::<T>::remove(target.clone(), (currency.clone(), hashed_proof.clone()));

			Self::deposit_event(Event::SwapClaimed {
				account: target,
				proof: hashed_proof,
                swap,
				success: succeeded,
			});

			Ok(())
		}

		/// Cancel an atomic swap. Only possible after the originally set duration has passed.
		///
		/// The dispatch origin for this call must be _Signed_.
		///
		/// - `target`: Target of the original atomic swap.
		/// - `hashed_proof`: Hashed proof of the original atomic swap.
		#[pallet::weight(T::DbWeight::get().reads_writes(1, 1).saturating_add(40_000_000))]
		pub fn cancel_swap(
			origin: OriginFor<T>,
			target: T::AccountId,
			currency: CurrencyIdOf<T>,
			hashed_proof: HashedProof,
		) -> DispatchResult {
			let source = ensure_signed(origin)?;

			let swap = PendingSwaps::<T>::get(&target, (currency, hashed_proof)).ok_or(Error::<T>::NotExist)?;
			ensure!(swap.source == source, Error::<T>::SourceMismatch);
            ensure!(swap.currency == currency, Error::<T>::ClaimCurrencyMismatch);
            ensure!(
				frame_system::Pallet::<T>::block_number() >= swap.end_block,
				Error::<T>::DurationNotPassed,
			);

			swap.action.cancel(&swap.source);
			PendingSwaps::<T>::remove(&target, (currency.clone(), hashed_proof.clone()));

			Self::deposit_event(Event::SwapCancelled { account: target, swap, proof: hashed_proof });

			Ok(())
		}
	}
}
