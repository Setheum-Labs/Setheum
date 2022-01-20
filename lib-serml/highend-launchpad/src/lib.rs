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
	pallet_prelude::*, transactional, PalletId, traits::Get, codec::{Decode, Encode}, ensure, storage::child,
};
use frame_system::{pallet_prelude::*, ensure_signed};
use orml_traits::{GetByKey, Get, MultiCurrency, MultiCurrencyExtended, MultiReservableCurrency};
use primitives::{AccountId, Balance, CampaignId, CurrencyId};
use sp_std::vec::Vec;
use sp_runtime::{traits::{AccountIdConversion, Saturating, Zero, Hash}};
mod mock;

pub use module::*;

/// Simple index for identifying a fund.
pub type FundIndex = u32;
pub type ProposalIndex = u32;
pub type CampaignIndex = u32;

type AccountIdOf<T> = <T as frame_system::Config>::AccountId;
type BalanceOf<T> = <<T as Config>::MultiCurrency as MultiCurrency<<T as frame_system::Config>::AccountId>>::Balance;
type CurrencyIdOf<T> =
	<<T as Config>::MultiCurrency as MultiCurrency<<T as frame_system::Config>::AccountId>>::CurrencyId;
type CampaignInfoOf<T> =
	CampaignInfo<AccountIdOf<T>, BalanceOf<T>, <T as frame_system::Config>::BlockNumber>;
type CampaignInfoOf<T> =
	CampaignInfo<AccountIdOf<T>, BalanceOf<T>, <T as frame_system::Config>::BlockNumber>;

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
		/// HighEnd LaunchPad (HELP) currency id. (LaunchPad Token)
		/// 
		type GetHelpCurrencyId: Get<CurrencyId>;

		/// The Campaign Commission rate taken from successful campaigns.
		/// The Buyback Commission is used to buy back and burn the HELP tokens.
		/// The first item of the tuple is the numerator of the commission rate, second
		/// item is the denominator, fee_rate = numerator / denominator,
		/// use (u32, u32) over `Rate` type to minimize internal division
		/// operation.
		#[pallet::constant]
		type GetBuyBackCommission: Get<(u32, u32)>;

		/// The Campaign Commission rate taken from successful campaigns
		/// The QMA-FlashLoans Commission is transferred to the Flashloans QMATreasury account.
		/// The first item of the tuple is the numerator of the commission rate, second
		/// item is the denominator, fee_rate = numerator / denominator,
		/// use (u32, u32) over `Rate` type to minimize internal division
		/// operation.
		#[pallet::constant]
		type GetQMACommission: Get<(u32, u32)>;

		/// The Campaign Commission rate taken from successful campaigns
		/// The Bootstrap Commission is transferred to the LaunchPool account
		/// for the Liquidity Providers (LPs) to claim.
		/// The first item of the tuple is the numerator of the commission rate, second
		/// item is the denominator, fee_rate = numerator / denominator,
		/// use (u32, u32) over `Rate` type to minimize internal division
		/// operation.
		#[pallet::constant]
		type GetLaunchPoolCommission: Get<(u32, u32)>;

		/// The amount to be held on deposit by the owner of a crowdfund
		/// - in HighEnd LaunchPad (HELP) currency id. (LaunchPad Token)  
		type SubmissionDeposit: Get<BalanceOf<Self>>;

		/// The Currency amount to be deposited in a bootstrap pair from the LaunchPool get by currency_id.
		type BootstrapDeposit: GetByKey<CurrencyId, Balance>;

		/// The minimum amount that may be contributed into a crowdfund. Should almost certainly be at
		/// least ExistentialDeposit.
		type MinContribution: Get<BalanceOf<Self>>;

		/// The maximum number of proposals that could be running at any given time.
		type MaxProposals: Get<Self::BlockNumber>;

		/// The maximum number of campaigns that could be running at any given time.
		type MaxCampaigns: Get<Self::BlockNumber>;

		/// The maximum number of campaigns that could be waiting at any given time.
		type MaxWaitingCampaigns: Get<Self::BlockNumber>;

		/// The maximum period of time (in blocks) that a crowdfund campaign clould be active.
		type MaxCampaignPeriod: Get<Self::BlockNumber>;

		/// The period of time (in blocks) after an unsuccessful crowdfund ending during which
		/// contributors are able to withdraw their funds. After this period, their funds are lost.
		type CampaignRetirementPeriod: Get<Self::BlockNumber>;

		/// The origin which may update, approve or reject campaign proposals.
		type UpdateOrigin: EnsureOrigin<Self::Origin>;

		#[pallet::constant]
		/// The Airdrop module pallet id, keeps airdrop funds.
		type PalletId: Get<PalletId>;                                                                                                                              

	}

	#[pallet::error]
	pub enum Error<T> {
		/// Crowdfund period is too long.
		PeriodTooLong,
		/// Crowdfund period is too short.
		ZeroPeriod,
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
		/// The Currency is not Correct
		InvalidCurrencyType,
		/// The EVM Contract Address is not Correct
		InvalidEvmContractAddress,
	}

	#[pallet::event]
	#[pallet::generate_deposit(pub(crate) fn deposit_event)]
	#[pallet::metadata(T::AccountId = "AccountId", BalanceOf<T> = "Balance", AirDropCurrencyId = "AirDropCurrencyId")]
	pub enum Event<T: Config> {
		/// Drop Airdrop \[currency_id\]
		Created(CampaignIndex, <T as frame_system::Config>::BlockNumber),
		Contributed(<T as frame_system::Config>::AccountId, CampaignIndex, BalanceOf<T>, <T as frame_system::Config>::BlockNumber),
		Withdrew(<T as frame_system::Config>::AccountId, CampaignIndex, BalanceOf<T>, <T as frame_system::Config>::BlockNumber),
		Retiring(CampaignIndex, <T as frame_system::Config>::BlockNumber),
		Dissolved(CampaignIndex, <T as frame_system::Config>::BlockNumber, <T as frame_system::Config>::AccountId),
		Dispensed(CampaignIndex, <T as frame_system::Config>::BlockNumber, <T as frame_system::Config>::AccountId),
	}
	#[derive(Encode, Decode, Default, PartialEq, Eq)]
	#[cfg_attr(feature = "std", derive(Debug))]
	pub struct CampaignInfo<AccountId, Balance, BlockNumber> {
		/// The name of the Project that will recieve the funds if the campaign is successful
		project_name: Vec<u8>,
		/// The account that will recieve the funds if the campaign is successful
		project_logo: Vec<u8>,
		/// The account that will recieve the funds if the campaign is successful
		project_description: Vec<u8>,
		/// The account that will recieve the funds if the campaign is successful
		project_website: Vec<u8>,
		/// The account that will recieve the funds if the campaign is successful
		beneficiary: AccountId,
		/// The currency to raise for the campaign
		raise_currency: CurrencyId,
		/// The projects ERC20 token contract address
		erc20_contract: CurrencyId,
		/// The amount of project tokens (ERC20)
		/// allocated to the crowdfund campaign contributors
		crowd_allocation: Balance,
		/// The amount of project tokens (ERC20)
		/// allocated to the DEX for bootstrap
		bootstrap_allocation: Balance,
		/// The amount of deposit placed
		submission_deposit: Balance,
		/// The total amount raised
		raised: Balance,
		/// Success bound on `raised` - Soft Cap for the campaign.
		soft_goal: Balance,
		/// Upper bound on `raised` - Hard Cap for the campaign.
		hard_goal: Balance,
		/// The number of blocks that the campaign will last.
		period: BlockNumber,
	}
	/// Info on all of the proposed campaigns.
	///
	/// map ProposalIndex => CampaignInfoOf<T>
	#[pallet::storage]
	#[pallet::getter(fn proposals)]
	pub type Proposals<T: Config> = StorageMap<_, Blake2_128Concat, ProposalIndex, CampaignInfoOf<T>, OptionQuery>;
	
	/// Info on all of the approved campaigns.
	///
	/// map CampaignIndex => CampaignInfoOf<T>
	#[pallet::storage]
	#[pallet::getter(fn campaigns)]
	pub type Campaigns<T: Config> = StorageMap<_, Blake2_128Concat, CampaignIndex, CampaignInfoOf<T>, OptionQuery>;
	
	/// The total number of proposals that have so far been allocated.
	#[pallet::storage]
	#[pallet::getter(fn proposal_count)]
	pub type ProposalCount<T: Config> = StorageValue<_, ProposalIndex, ValueQuery>;
	
	/// The total number of campaigns that have so far been allocated.
	#[pallet::storage]
	#[pallet::getter(fn fund_count)]
	pub type FundCount<T: Config> = StorageValue<_, CampaignIndex, ValueQuery>;
	
	#[pallet::pallet]
	pub struct Pallet<T>(PhantomData<T>);

	#[pallet::hooks]
	impl<T: Config> Hooks<T::BlockNumber> for Pallet<T> {}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// Create a new Campaign Proposal
		#[pallet::weight(10_000)]
		pub fn propose_campaign(
			origin: OriginFor<T>,
			project_name: Vec<u8>,
			project_logo: Vec<u8>,
			project_description: Vec<u8>,
			project_website: Vec<u8>,
			beneficiary: AccountIdOf<T>,
			raise_currency: CurrencyIdOf<T>,
			erc20_contract: CurrencyIdOf<T>,
			crowd_allocation: BalanceOf<T>,
			bootstrap_allocation: BalanceOf<T>,
			soft_goal: BalanceOf<T>,
			hard_goal: BalanceOf<T>,
			period: T::BlockNumber,
		)-> DispatchResultWithPostInfo {
			let creator = ensure_signed(origin)?;

			// Ensure that the period is not zero
			ensure!(period > T::BlockNumber::zero(), Error::<T>::ZeroPeriod);
			// Ensure that the period is not too long
			ensure!(period <= T::MaxCampaignPeriod::get(), Error::<T>::PeriodTooLong);

			// Tag the time of the proposal
			let now = <frame_system::Pallet<T>>::block_number();

			// Ensure that the erc20 contract is a valid ERC20 token contract
			ensure!(
				erc20_contract == CurrencyId::Erc20(contract),
				Error::<T>::InvalidEvmContractAddress
			);
			if erc20_contract == CurrencyId::Erc20(contract) {
				// Transfer the crowd allocation to the CrowdfundPool account
				T::MultiCurrency::transfer(
					&erc20_contract,
					&creator,
					&Self::account_id(),
					crowd_allocation,
				)?;
				// Transfer the bootstrap allocation to the BootstrapPool account
				T::MultiCurrency::transfer(
					&erc20_contract,
					&creator,
					&Self::account_id(),
					bootstrap_allocation,
				)?;
			};
			
			// Transfer the submission deposit to the CrowdfundTreasury account
			let submission_deposit = T::SubmissionDeposit::get();
			T::MultiCurrency::transfer(
				T::GetHelpCurrencyId::get(),
				&creator,
				&Self::account_id(),
				submission_deposit,
			)?;

			let index = <FundCount<T>>::get();
			// not protected against overflow, see safemath section
			<FundCount<T>>::put(index + 1);

			// Create the campaign info and add it to the proposals storage
			<Proposals<T>>::insert(index, CampaignInfo{
				project_name,
				project_logo,
				project_description,
				project_website,
				beneficiary,
				raise_currency,
				erc20_contract,
				crowd_allocation,
				bootstrap_allocation,
				submission_deposit,
				raised: Zero::zero(),
				soft_goal,
				hard_goal,
				period,
			});

			Self::deposit_event(Event::Created(index, now));
			Ok(().into())
		}

		/// Approve a campaign proposal
		#[pallet::weight(10_000)]
		pub fn approve_campaign(
			origin: OriginFor<T>,
			index: ProposalIndex,
		)-> DispatchResultWithPostInfo {
			// Ensure that the origin is the UpdateOrigin (i.e. the Launchpad Council)
			T::UpdateOrigin::ensure_origin(origin)?;

			// Ensure that the proposal exists
			let proposal = Self::proposals(index).ok_or(Error::<T>::InvalidIndex)?;

			// Tag the time of approval
			let now = <frame_system::Pallet<T>>::block_number();

			// Remove the proposal from the proposals storage
			<Proposals<T>>::take(index).ok_or(Error::<T>::InvalidIndex)?;
			// Create the Approved campaign info and add it to the `Campaigns` storage
			<Campaigns<T>>::insert(index, CampaignInfo{
				project_name: proposal.project_name,
				project_logo: proposal.project_logo,
				project_description: proposal.project_description,
				project_website: proposal.project_website,
				beneficiary: proposal.beneficiary,
				raise_currency: proposal.raise_currency,
				erc20_contract: proposal.erc20_contract,
				crowd_allocation: proposal.crowd_allocation,
				bootstrap_allocation: proposal.bootstrap_allocation,
				submission_deposit: proposal.submission_deposit,
				raised: Zero::zero(),
				soft_goal: proposal.soft_goal,
				hard_goal: proposal.hard_goal,
				period: proposal.period,
			});

			Self::deposit_event(Event::Created(index, now));
			Ok(().into())
		}

		/// Approve a campaign proposal
		#[pallet::weight(10_000)]
		pub fn reject_campaign(
			origin: OriginFor<T>,
			index: ProposalIndex,
		)-> DispatchResultWithPostInfo {
			T::UpdateOrigin::ensure_origin(origin)?;

			// Create the campaign info and add it to the proposals storage
			<Campaigns<T>>::take(index).ok_or(Error::<T>::InvalidIndex)?;

			// Transfer the crowd allocation to the CrowdfundPool account
			T::MultiCurrency::transfer(
				&erc20_contract,
				&creator,
				&Self::account_id(),
				crowd_allocation,
			)?;
			// Transfer the bootstrap allocation to the BootstrapPool account
			T::MultiCurrency::transfer(
				&erc20_contract,
				&creator,
				&Self::account_id(),
				bootstrap_allocation,
			)?;
			
			// Transfer the submission deposit to the CrowdfundTreasury account
			let submission_deposit = T::SubmissionDeposit::get();
			T::MultiCurrency::transfer(
				T::GetHelpCurrencyId::get(),
				&creator,
				&Self::account_id(),
				submission_deposit,
			)?;

			let index = <FundCount<T>>::get();
			// not protected against overflow, see safemath section
			<FundCount<T>>::put(index + 1);

			Self::deposit_event(Event::Created(index, now));
			Ok(().into())
		}

		/// Contribute funds to an existing fund
		#[pallet::weight(10_000)]
		pub fn contribute(
			origin: OriginFor<T>, 
			index: CampaignIndex,
			currency_id: CurrencyIdOf<T>,
			value: BalanceOf<T>
		) -> DispatchResultWithPostInfo {

			let who = ensure_signed(origin)?;

			ensure!(value >= T::MinContribution::get(), Error::<T>::ContributionTooSmall);
			let mut campaign = Self::campaigns(index).ok_or(Error::<T>::InvalidIndex)?;
			ensure!(currency_id == campaign.raise_currency, Error::<T>::InvalidCurrencyType);

			// Make sure crowdfund has not ended
			let now = <frame_system::Pallet<T>>::block_number();
			ensure!(campaign.period > now, Error::<T>::ContributionPeriodOver);

			// Add contribution to the campaign
			T::MultiCurrency::transfer(
				&who,
				&Self::fund_account_id(index),
				value,
				ExistenceRequirement::AllowDeath
			)?;
			campaign.raised += value;
			Campaigns::<T>::insert(index, &campaign);

			let balance = Self::contribution_get(index, &who);
			let balance = balance.saturating_add(value);
			Self::contribution_put(index, &who, &balance);

			Self::deposit_event(Event::Contributed(who, index, balance, now));

			Ok(().into())
		}

		/// Withdraw full balance of a contributor to a campaign
		/// TODO: Transfer instead of resolve into existence
		#[pallet::weight(10_000)]
		pub fn withdraw(
			origin: OriginFor<T>,
			#[pallet::compact] index: CampaignIndex) -> DispatchResultWithPostInfo {
			let who = ensure_signed(origin)?;

			let mut campaign = Self::campaigns(index).ok_or(Error::<T>::InvalidIndex)?;
			let now = <frame_system::Pallet<T>>::block_number();
			ensure!(campaign.period < now, Error::<T>::FundStillActive);

			let balance = Self::contribution_get(index, &who);
			ensure!(balance > Zero::zero(), Error::<T>::NoContribution);

			// Return funds to caller without charging a transfer fee
			let _ = T::MultiCurrency::resolve_into_existing(&who, T::MultiCurrency::withdraw(
				campaign.raise_currency,
				&Self::fund_account_id(index),
				balance,
			)?);

			// Update storage
			Self::contribution_kill(index, &who);
			campaign.raised = campaign.raised.saturating_sub(balance);
			<Campaigns<T>>::insert(index, &campaign);

			Self::deposit_event(Event::Withdrew(who, index, balance, now));

			Ok(().into())
		}

		/// Dissolve an entire crowdfund after its retirement period has expired.
		/// Anyone can call this function, and they are incentivized to do so because
		/// they inherit the deposit.
		#[pallet::weight(10_000)]
		pub fn dissolve(
			origin: OriginFor<T>, 
			index: CampaignIndex) -> DispatchResultWithPostInfo {
			let reporter = ensure_signed(origin)?;

			let campaign = Self::campaigns(index).ok_or(Error::<T>::InvalidIndex)?;

			// Check that enough time has passed to remove from storage
			let now = <frame_system::Pallet<T>>::block_number();
			ensure!(now >= campaign.period + T::CampaignRetirementPeriod::get(), Error::<T>::FundNotRetired);

			let account = Self::fund_account_id(index);

			// Dissolver collects the deposit and any remaining funds
			let _ = T::MultiCurrency::resolve_creating(&reporter, T::MultiCurrency::withdraw(
				campaign.raise_currency,
				&account,
				campaign.deposit + campaign.raised,
			)?);

			// Remove the campaign info from storage
			<Campaigns<T>>::remove(index);
			// Remove all the contributor info from storage in a single write.
			// This is possible thanks to the use of a child tree.
			Self::crowdfund_kill(index);

			Self::deposit_event(Event::Dissolved(index, now, reporter));

			Ok(().into())
		}

		/// Dispense a payment to the beneficiary of a successful crowdfund.
		/// The beneficiary receives the contributed funds and the caller receives
		/// the deposit as a reward to incentivize clearing settled crowdfunds out of storage.
		/// TODO: Transfer instead of resolve into creating
		#[pallet::weight(10_000)]
		pub fn dispense(
			origin: OriginFor<T>, 
			index: CampaignIndex) -> DispatchResultWithPostInfo {
			let caller = ensure_signed(origin)?;

			let campaign = Self::campaigns(index).ok_or(Error::<T>::InvalidIndex)?;

			// Check that enough time has passed to remove from storage
			let now = <frame_system::MoPalletdule<T>>::block_number();

			ensure!(now >= campaign.period, Error::<T>::FundStillActive);

			// Check that the campaign was actually successful
			ensure!(campaign.raised >= campaign.goal, Error::<T>::UnsuccessfulFund);

			let account = Self::fund_account_id(index);

			// Beneficiary collects the contributed funds
			let _ = T::MultiCurrency::resolve_creating(&campaign.beneficiary, T::MultiCurrency::withdraw(
				campaign.raise_currency,
				&account,
				campaign.raised,
			)?);

			// Caller collects the deposit
			let _ = T::MultiCurrency::resolve_creating(&caller, T::MultiCurrency::withdraw(
				campaign.raise_currency,
				&account,
				campaign.deposit,
			)?);

			// Remove the campaign info from storage
			<Campaigns<T>>::remove(index);
			// Remove all the contributor info from storage in a single write.
			// This is possible thanks to the use of a child tree.
			Self::crowdfund_kill(index);

			Self::deposit_event(Event::Dispensed(index, now, caller));
			Ok(().into())
		}
	}
}

impl<T: Config> Pallet<T> {
	/// Get account of HELP Treasury module Account.
	pub fn treasury_account_id() -> T::AccountId {
		T::PalletId::get().into_account()
	}

	/// The account ID of the fund pot.
	///
	/// This actually does computation. If you need to keep using it, then make sure you cache the
	/// value and only call this once.
	pub fn campaign_account_id(index: CampaignIndex) -> T::AccountId {
		T::PalletId::get().into_sub_account(index)
	}

	/// Find the ID associated with the fund
	///
	/// Each fund stores information about its contributors and their contributions in a child trie
	/// This helper function calculates the id of the associated child trie.
	pub fn id_from_index(index: CampaignIndex) -> child::ChildInfo {
		let mut buf = Vec::new();
		buf.extend_from_slice(b"crowdfnd");
		buf.extend_from_slice(&index.to_le_bytes()[..]);

		child::ChildInfo::new_default(T::Hashing::hash(&buf[..]).as_ref())
	}

	/// Record a contribution in the associated child trie.
	pub fn contribution_put(index: CampaignIndex, who: &T::AccountId, balance: &BalanceOf<T>) {
		let id = Self::id_from_index(index);
		who.using_encoded(|b| child::put(&id, b, &balance));
	}

	/// Lookup a contribution in the associated child trie.
	pub fn contribution_get(index: CampaignIndex, who: &T::AccountId) -> BalanceOf<T> {
		let id = Self::id_from_index(index);
		who.using_encoded(|b| child::get_or_default::<BalanceOf<T>>(&id, b))
	}

	/// Remove a contribution from an associated child trie.
	pub fn contribution_kill(index: CampaignIndex, who: &T::AccountId) {
		let id = Self::id_from_index(index);
		who.using_encoded(|b| child::kill(&id, b));
	}

	/// Remove the entire record of contributions in the associated child trie in a single
	/// storage write.
	pub fn crowdfund_kill(index: CampaignIndex) {
		let id = Self::id_from_index(index);
		// The None here means we aren't setting a limit to how many keys to delete.
		// Limiting can be useful, but is beyond the scope of this recipe. For more info, see
		// https://crates.parity.io/frame_support/storage/child/fn.kill_storage.html
		child::kill_storage(&id, None);
	}
}
