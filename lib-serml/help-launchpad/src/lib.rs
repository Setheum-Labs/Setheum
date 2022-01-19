// بِسْمِ اللَّهِ الرَّحْمَنِ الرَّحِيم
// ٱلَّذِينَ يَأْكُلُونَ ٱلرِّبَوٰا۟ لَا يَقُومُونَ إِلَّا كَمَا يَقُومُ ٱلَّذِى يَتَخَبَّطُهُ ٱلشَّيْطَـٰنُ مِنَ ٱلْمَسِّ ۚ ذَٰلِكَ بِأَنَّهُمْ قَالُوٓا۟ إِنَّمَا ٱلْبَيْعُ مِثْلُ ٱلرِّبَوٰا۟ ۗ وَأَحَلَّ ٱللَّهُ ٱلْبَيْعَ وَحَرَّمَ ٱلرِّبَوٰا۟ ۚ فَمَن جَآءَهُۥ مَوْعِظَةٌ مِّن رَّبِّهِۦ فَٱنتَهَىٰ فَلَهُۥ مَا سَلَفَ وَأَمْرُهُۥٓ إِلَى ٱللَّهِ ۖ وَمَنْ عَادَ فَأُو۟لَـٰٓئِكَ أَصْحَـٰبُ ٱلنَّارِ ۖ هُمْ فِيهَا خَـٰلِدُونَ

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

//! # Airdrop Module
//!
//! ## Overview
//!
//! This module creates a crowdfunding launchpad for teams to raise funds
//! and bootstrap their tokens on the SetSwap -
//! governance is done from an update origin. 
//! The module for the Setheum LaunchPad Protocol.

#![cfg_attr(not(feature = "std"), no_std)]
#![allow(clippy::unused_unit)]

use frame_support::{
	pallet_prelude::*, transactional, PalletId, traits::Get, codec::{Decode, Encode}, storage::child,
};
use frame_system::pallet_prelude::*;
use orml_traits::{GetByKey, MultiCurrency, MultiCurrencyExtended, MultiReservableCurrency};
use primitives::{AccountId, Balance, CurrencyId};
use sp_std::vec::Vec;
use sp_runtime::{traits::{AccountIdConversion, Saturating, Zero, Hash}};
mod mock;

pub use module::*;

/// Simple index for identifying a fund.
pub type FundIndex = u32;
pub type ProjectIndex = u32;
pub type RoundIndex = u32;

type AccountIdOf<T> = <T as frame_system::Config>::AccountId;
type AllocationOf<T> = Allocation<AccountIdOf<T>, BalanceOf<T>, <T as frame_system::Config>::BlockNumber>;
type BalanceOf<T> = <<T as Config>::MultiCurrency as MultiCurrency<<T as frame_system::Config>::AccountId>>::Balance;
type CampaignOf<T> = Campaign<AccountIdOf<T>, BalanceOf<T>, <T as frame_system::Config>::BlockNumber>;
type CurrencyIdOf<T> =
	<<T as Config>::MultiCurrency as MultiCurrency<<T as frame_system::Config>::AccountId>>::CurrencyId;
type FundOf<T> = Fund<AccountIdOf<T>, BalanceOf<T>, <T as frame_system::Config>::BlockNumber>;
type ProjectOf<T> = Project<AccountIdOf<T>, <T as frame_system::Config>::BlockNumber>;
type FundInfoOf<T> =
		FundInfo<AccountIdOf<T>, BalanceOf<T>, <T as frame_system::Config>::BlockNumber>;

#[derive(Encode, Decode, Default, PartialEq, Eq)]
#[cfg_attr(feature = "std", derive(Debug))]
pub struct FundInfo<AccountId, Balance, BlockNumber> {
	/// The account that will recieve the funds if the campaign is successful
	beneficiary: AccountId,
	/// The currency of raise for the campaign
	currency: CurrencyId,
	/// The amount of deposit placed
	deposit: Balance,
	/// The total amount raised
	raised: Balance,
	/// Block number after which funding must have succeeded
	end: BlockNumber,
	/// Upper bound on `raised`
	goal: Balance,
}

/// Project struct
#[derive(Encode, Decode, Default, PartialEq, Eq, Clone, Debug)]
#[cfg_attr(feature = "std", derive(serde::Serialize))]
pub struct Project<AccountId, BlockNumber> {
	name: Vec<u8>,
	logo: Vec<u8>,
	description: Vec<u8>,
	website: Vec<u8>,
	/// The account that will receive the funds if the campaign is successful
	owner: AccountId,
	create_block_number: BlockNumber,
}

#[frame_support::pallet]
pub mod module {
	use super::*;

	#[pallet::config]
	pub trait Config: frame_system::Config {
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;

		/// The Currency for managing assets related to the SERP (Setheum Elastic Reserve Protocol).
		type MultiCurrency: MultiCurrencyExtended<Self::AccountId, CurrencyId = CurrencyId, Balance = Balance>;

		#[pallet::constant]
		/// Setter (SETR) currency id
		/// 
		type SetterCurrencyId: Get<CurrencyId>;

		#[pallet::constant]
		/// The SetDollar (SETUSD) currency id
		type GetSetUSDId: Get<CurrencyId>;

		#[pallet::constant]
		/// Native Setheum (SETM) currency id. [P]Pronounced "set M" or "setem"
		/// 
		type GetNativeCurrencyId: Get<CurrencyId>;

		#[pallet::constant]
		/// Serp (SERP) currency id.
		/// 
		type GetSerpCurrencyId: Get<CurrencyId>;

		#[pallet::constant]
		/// The Dinar (DNAR) currency id.
		/// 
		type GetDinarCurrencyId: Get<CurrencyId>;

		#[pallet::constant]
		/// High-End LaunchPad (HELP) currency id. (LaunchPad Token)
		/// 
		type GetHelpCurrencyId: Get<CurrencyId>;

		/// The amount to be held on deposit by the owner of a crowdfund
		type SubmissionDeposit: Get<BalanceOf<Self>>;

		/// The minimum Setheum Currency amount to be deposited in a bootstrap pair
		/// by the owner of a crowdfund Bootstrap, get by currency_id.
		type MinBootstrapDeposit: GetByKey<CurrencyId, Balance>;

		/// The minimum amount that may be contributed into a crowdfund. Should almost certainly be at
		/// least ExistentialDeposit.
		type MinContribution: Get<BalanceOf<Self>>;

		/// The period of time (in blocks) after an unsuccessful crowdfund ending during which
		/// contributors are able to withdraw their funds. After this period, their funds are lost.
		type RetirementPeriod: Get<Self::BlockNumber>;

		#[pallet::constant]
		/// The Airdrop module pallet id, keeps airdrop funds.
		type UpdateOrigin: Get<Self::AccountId>;
		
		#[pallet::constant]
		/// The Airdrop module pallet id, keeps airdrop funds.
		type PalletId: Get<PalletId>;

	}

	#[pallet::error]
	pub enum Error<T> {
		/// Crowdfund must end after it starts
		EndTooEarly,
		/// Must contribute at least the minimum amount of funds
		ContributionTooSmall,
		/// The fund index specified does not exist
		InvalidIndex,
		/// The crowdfund's contribution period has ended; no more contributions will be accepted
		ContributionPeriodOver,
		/// You may not withdraw or dispense funds while the fund is still active
		FundStillActive,
		/// You cannot withdraw funds because you have not contributed any
		NoContribution,
		/// You cannot dissolve a fund that has not yet completed its retirement period
		FundNotRetired,
		/// Cannot dispense funds from an unsuccessful fund
		UnsuccessfulFund,
	}

	#[pallet::event]
	#[pallet::generate_deposit(pub(crate) fn deposit_event)]
	#[pallet::metadata(T::AccountId = "AccountId", BalanceOf<T> = "Balance", AirDropCurrencyId = "AirDropCurrencyId")]
	pub enum Event<T: Config> {
		/// Drop Airdrop \[currency_id\]
		Created(FundIndex, <T as frame_system::Config>::BlockNumber),
		Contributed(<T as frame_system::Config>::AccountId, FundIndex, BalanceOf<T>, <T as frame_system::Config>::BlockNumber),
		Withdrew(<T as frame_system::Config>::AccountId, FundIndex, BalanceOf<T>, <T as frame_system::Config>::BlockNumber),
		Retiring(FundIndex, <T as frame_system::Config>::BlockNumber),
		Dissolved(FundIndex, <T as frame_system::Config>::BlockNumber, <T as frame_system::Config>::AccountId),
		Dispensed(FundIndex, <T as frame_system::Config>::BlockNumber, <T as frame_system::Config>::AccountId),
	}

	/// Info on all of the funds.
	///
	/// map CurrencyId => FundInfoOf<T>
	#[pallet::storage]
	#[pallet::getter(fn funds)]
	pub type Funds<T: Config> = StorageMap<_, Blake2_128Concat, FundIndex, FundInfoOf<T>, OptionQuery>;
	
	/// The total number of funds that have so far been allocated.
	#[pallet::storage]
	#[pallet::getter(fn fund_count)]
	pub type FundCount<T: Config> = StorageValue<_, FundIndex, ValueQuery>;
	
	#[pallet::pallet]
	pub struct Pallet<T>(PhantomData<T>);

	#[pallet::hooks]
	impl<T: Config> Hooks<T::BlockNumber> for Pallet<T> {}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// Create a new fund
		#[pallet::weight(10_000)]
		pub fn create(
			origin: OriginFor<T>,
			currency_id: CurrencyIdOf<T>,
			beneficiary: AccountIdOf<T>,
			goal: BalanceOf<T>,
			end: T::BlockNumber,
		)-> DispatchResultWithPostInfo {
			let creator = ensure_signed(origin)?;
			let now = <frame_system::Pallet<T>>::block_number();
				ensure!(end > now, Error::<T>::EndTooEarly);
				let deposit = T::SubmissionDeposit::get();
			let imb = T::MultiCurrency::transfer(
				currency_id,
				&creator,
				deposit,
			)?;
				
			let index = <FundCount<T>>::get();
			// not protected against overflow, see safemath section
			<FundCount<T>>::put(index + 1);
			// No fees are paid here if we need to create this account; that's why we don't just
			// use the stock `transfer`.
			T::MultiCurrency::resolve_creating(&Self::fund_account_id(index), imb);

			<Funds<T>>::insert(index, FundInfo{
				beneficiary,
				deposit,
				raised: Zero::zero(),
				end,
				goal,
			});

			Self::deposit_event(Event::Created(index, now));
			Ok(().into())
		}

		/// Contribute funds to an existing fund
		#[pallet::weight(10_000)]
		pub fn contribute(
			origin: OriginFor<T>, 
			index: FundIndex, 
			value: BalanceOf<T>) -> DispatchResultWithPostInfo {

			let who = ensure_signed(origin)?;

			ensure!(value >= T::MinContribution::get(), Error::<T>::ContributionTooSmall);
			let mut fund = Self::funds(index).ok_or(Error::<T>::InvalidIndex)?;

			// Make sure crowdfund has not ended
			let now = <frame_system::Pallet<T>>::block_number();
			ensure!(fund.end > now, Error::<T>::ContributionPeriodOver);

			// Add contribution to the fund
			T::Currency::transfer(
				&who,
				&Self::fund_account_id(index),
				value,
				ExistenceRequirement::AllowDeath
			)?;
			fund.raised += value;
			Funds::<T>::insert(index, &fund);

			let balance = Self::contribution_get(index, &who);
			let balance = balance.saturating_add(value);
			Self::contribution_put(index, &who, &balance);

			Self::deposit_event(Event::Contributed(who, index, balance, now));

			Ok(().into())
		}

		/// Withdraw full balance of a contributor to a fund
		#[pallet::weight(10_000)]
		pub fn withdraw(
			origin: OriginFor<T>,
			#[pallet::compact] index: FundIndex) -> DispatchResultWithPostInfo {
			let who = ensure_signed(origin)?;

			let mut fund = Self::funds(index).ok_or(Error::<T>::InvalidIndex)?;
			let now = <frame_system::Pallet<T>>::block_number();
			ensure!(fund.end < now, Error::<T>::FundStillActive);

			let balance = Self::contribution_get(index, &who);
			ensure!(balance > Zero::zero(), Error::<T>::NoContribution);

			// Return funds to caller without charging a transfer fee
			let _ = T::Currency::resolve_into_existing(&who, T::Currency::withdraw(
				&Self::fund_account_id(index),
				balance,
				WithdrawReasons::TRANSFER,
				ExistenceRequirement::AllowDeath
			)?);

			// Update storage
			Self::contribution_kill(index, &who);
			fund.raised = fund.raised.saturating_sub(balance);
			<Funds<T>>::insert(index, &fund);

			Self::deposit_event(Event::Withdrew(who, index, balance, now));

			Ok(().into())
		}

		/// Dissolve an entire crowdfund after its retirement period has expired.
		/// Anyone can call this function, and they are incentivized to do so because
		/// they inherit the deposit.
		#[pallet::weight(10_000)]
		pub fn dissolve(
			origin: OriginFor<T>, 
			index: FundIndex) -> DispatchResultWithPostInfo {
			let reporter = ensure_signed(origin)?;

			let fund = Self::funds(index).ok_or(Error::<T>::InvalidIndex)?;

			// Check that enough time has passed to remove from storage
			let now = <frame_system::Pallet<T>>::block_number();
			ensure!(now >= fund.end + T::RetirementPeriod::get(), Error::<T>::FundNotRetired);

			let account = Self::fund_account_id(index);

			// Dissolver collects the deposit and any remaining funds
			let _ = T::Currency::resolve_creating(&reporter, T::Currency::withdraw(
				&account,
				fund.deposit + fund.raised,
				WithdrawReasons::TRANSFER,
				ExistenceRequirement::AllowDeath,
			)?);

			// Remove the fund info from storage
			<Funds<T>>::remove(index);
			// Remove all the contributor info from storage in a single write.
			// This is possible thanks to the use of a child tree.
			Self::crowdfund_kill(index);

			Self::deposit_event(Event::Dissolved(index, now, reporter));

			Ok(().into())
		}

		/// Dispense a payment to the beneficiary of a successful crowdfund.
		/// The beneficiary receives the contributed funds and the caller receives
		/// the deposit as a reward to incentivize clearing settled crowdfunds out of storage.
		#[pallet::weight(10_000)]
		pub fn dispense(
			origin: OriginFor<T>, 
			index: FundIndex) -> DispatchResultWithPostInfo {
			let caller = ensure_signed(origin)?;

			let fund = Self::funds(index).ok_or(Error::<T>::InvalidIndex)?;

			// Check that enough time has passed to remove from storage
			let now = <frame_system::MoPalletdule<T>>::block_number();

			ensure!(now >= fund.end, Error::<T>::FundStillActive);

			// Check that the fund was actually successful
			ensure!(fund.raised >= fund.goal, Error::<T>::UnsuccessfulFund);

			let account = Self::fund_account_id(index);

			// Beneficiary collects the contributed funds
			let _ = T::Currency::resolve_creating(&fund.beneficiary, T::Currency::withdraw(
				&account,
				fund.raised,
				WithdrawReasons::TRANSFER,
				ExistenceRequirement::AllowDeath,
			)?);

			// Caller collects the deposit
			let _ = T::Currency::resolve_creating(&caller, T::Currency::withdraw(
				&account,
				fund.deposit,
				WithdrawReasons::TRANSFER,
				ExistenceRequirement::AllowDeath,
			)?);

			// Remove the fund info from storage
			<Funds<T>>::remove(index);
			// Remove all the contributor info from storage in a single write.
			// This is possible thanks to the use of a child tree.
			Self::crowdfund_kill(index);

			Self::deposit_event(Event::Dispensed(index, now, caller));
			Ok(().into())
		}
	}
}

impl<T: Config> Pallet<T> {
	/// Get account of SERP Treasury module.
	pub fn account_id() -> T::AccountId {
		T::PalletId::get().into_account()
	}


	/// Find the ID associated with the fund
	///
	/// Each fund stores information about its contributors and their contributions in a child trie
	/// This helper function calculates the id of the associated child trie.
	pub fn id_from_index(index: FundIndex) -> child::ChildInfo {
		let mut buf = Vec::new();
		buf.extend_from_slice(b"crowdfnd");
		buf.extend_from_slice(&index.to_le_bytes()[..]);

		child::ChildInfo::new_default(T::Hashing::hash(&buf[..]).as_ref())
	}

	/// Record a contribution in the associated child trie.
	pub fn contribution_put(index: FundIndex, who: &T::AccountId, balance: &BalanceOf<T>) {
		let id = Self::id_from_index(index);
		who.using_encoded(|b| child::put(&id, b, &balance));
	}

	/// Lookup a contribution in the associated child trie.
	pub fn contribution_get(index: FundIndex, who: &T::AccountId) -> BalanceOf<T> {
		let id = Self::id_from_index(index);
		who.using_encoded(|b| child::get_or_default::<BalanceOf<T>>(&id, b))
	}

	/// Remove a contribution from an associated child trie.
	pub fn contribution_kill(index: FundIndex, who: &T::AccountId) {
		let id = Self::id_from_index(index);
		who.using_encoded(|b| child::kill(&id, b));
	}

	/// Remove the entire record of contributions in the associated child trie in a single
	/// storage write.
	pub fn crowdfund_kill(index: FundIndex) {
		let id = Self::id_from_index(index);
		// The None here means we aren't setting a limit to how many keys to delete.
		// Limiting can be useful, but is beyond the scope of this recipe. For more info, see
		// https://crates.parity.io/frame_support/storage/child/fn.kill_storage.html
		child::kill_storage(&id, None);
	}
}
