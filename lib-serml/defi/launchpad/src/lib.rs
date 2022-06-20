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

//! # Launchpad Crowdsales Pallet
//!
//! ## Overview
//!
//! This module creates a crowdsales launchpad
//! for teams to raise funds and sell their tokens to the public -
//! governance is done from an update origin. 

#![cfg_attr(not(feature = "std"), no_std)]
// Disable the following two lints since they originate from an external macro (namely decl_storage)
#![allow(clippy::string_lit_as_bytes)]
#![allow(clippy::unused_unit)]

use frame_support::{
	pallet_prelude::*, transactional, PalletId, traits::Get, ensure
};
use frame_system::{pallet_prelude::*, ensure_signed};
use orml_traits::{GetByKey, MultiCurrency, MultiLockableCurrency, LockIdentifier};
use primitives::{Balance, CampaignId, CurrencyId};
use support::{
	CampaignInfo, CampaignManager, Proposal,
};

use sp_std::{
	vec::Vec,
};
use sp_runtime::{traits::{AccountIdConversion, Zero}, DispatchResult};

mod mock;
mod tests;

pub use module::*;

pub(crate) type BalanceOf<T> = <<T as Config>::MultiCurrency as MultiCurrency<<T as frame_system::Config>::AccountId>>::Balance;
pub(crate) type CurrencyIdOf<T> =
	<<T as Config>::MultiCurrency as MultiCurrency<<T as frame_system::Config>::AccountId>>::CurrencyId;
	pub(crate) type CampaignInfoOf<T> =
	CampaignInfo<<T as frame_system::Config>::AccountId, BalanceOf<T>, <T as frame_system::Config>::BlockNumber>;

pub const LAUNCHPAD_LOCK_ID: LockIdentifier = *b"set/lpad";
#[frame_support::pallet]
pub mod module {
	use super::*;

	#[pallet::config]
	pub trait Config: frame_system::Config {
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;

		/// The Currency for managing assets related to the SERP (Setheum Elastic Reserve Protocol).
		type MultiCurrency: MultiLockableCurrency<Self::AccountId, CurrencyId = CurrencyId, Balance = Balance>;

		#[pallet::constant]
		/// Native currency_id.
		/// 
		type GetNativeCurrencyId: Get<CurrencyIdOf<Self>>;

		/// The Campaign Commission rate taken from successful campaigns
		/// The Treasury Commission is transferred to the Network's Treasury account.
		/// The first item of the tuple is the numerator of the commission rate, second
		/// item is the denominator, fee_rate = numerator / denominator,
		/// use (u32, u32) over another type to minimize internal division operation.
		#[pallet::constant]
		type GetCommission: Get<(u32, u32)>;

		/// The amount to be held on deposit by the owner of a crowdfund
		/// - in HighEnd LaunchPad (HELP) currency id. (LaunchPad Token)  
		type SubmissionDeposit: Get<BalanceOf<Self>>;

		/// The minimum amount that must be raised in a crowdsales campaign.
        /// Campaign Goal must be at least this amount.
		/// If this amount is not met, the proposal can be updated by the proposer or will be rejected.
		type MinRaise: GetByKey<CurrencyIdOf<Self>, BalanceOf<Self>>;

		/// The minimum amount that may be contributed into a crowdfund - by currency_id.
		/// Should almost certainly be at least ExistentialDeposit.
		type MinContribution: GetByKey<CurrencyIdOf<Self>, BalanceOf<Self>>;

		/// The maximum number of proposals that could be running at any given time.
		/// If set to 0, proposals are disabled and the Module will panic if a proposal is made.
		type MaxProposalsCount: Get<u32>;

		/// The maximum number of campaigns that could be running at any given time.
		/// If set to 0, campaigns are disabled and the Module will panic if a campaign is made.
		type MaxCampaignsCount: Get<u32>;

		/// The maximum period of time (in blocks) that a crowdfund campaign clould be active.
		/// If set to 0, active period is disabled and the Module will panic if a campaign is activated.
		type MaxActivePeriod: Get<Self::BlockNumber>;

		/// The period of time (number of blocks) a campaign is delayed after being Approved by governance.
		type CampaignStartDelay: Get<Self::BlockNumber>;

		/// The period of time (in blocks) after an unsuccessful crowdfund ending during which
		/// contributors are able to withdraw their funds. After this period, their funds are lost.
		type CampaignRetirementPeriod: Get<Self::BlockNumber>;

		/// The period of time (in blocks) after a rejected crowdfund proposal during which
		/// proposal creators's locked deposits are unlocked and the proposal is set to `is_rejected`.
		/// After this period, their proposal is lost.
		type ProposalRetirementPeriod: Get<Self::BlockNumber>;

		/// The origin which may update, approve or reject campaign proposals.
		type UpdateOrigin: EnsureOrigin<Self::Origin>;

		#[pallet::constant]
		/// The Airdrop module pallet id, keeps airdrop funds.
		type PalletId: Get<PalletId>;                                                                                                                              
	}

	#[pallet::error]
	pub enum Error<T> {
		/// The campaign funds raised already claimed by campaign creator or beneficiary
		CampaignAlreadyClaimed,
		/// The crowdfund's contribution period has ended; no more contributions will be accepted.
		CampaignEnded,
		/// Campaign has failed
		CampaignFailed,
		/// Campaign is not approved
		CampaignNotApproved,
		/// Campaign is not active
		CampaignNotActive,
		/// Campaign is not in the list of campaigns.
		CampaignNotFound,
		/// Campaign has not started
		CampaignNotStarted,
		/// Campaign is still active
		CampaignStillActive,
		/// Contributors balance is not enough to contribute
		ContributionCurrencyNotEnough,
		/// Contribution failed to transfer
		ContributionFailedTransfer,
		/// Contribution is not in the list of contributions.
		ContributionNotFound,
		/// Must contribute at least the minimum amount of funds.
		ContributionTooSmall,
		/// Contribution has duplicate account
		DuplicateContribution,
		/// Must contribute at least the minimum amount of funds.
		GoalBelowMinimumRaise,
		/// The goal is not equal to allocation
		GoalNotAllignedWithAllocation,
		/// The Submission Deposit Funds are insufficient
		InsufficientBalance,
		/// Wrong Currency Type in use.
		InvalidCurrencyType,
		/// The fund index specified does not exist.
		InvalidIndex,
		/// The campaign is in waiting period
		InWaitingPeriod,
		/// Maximum number of simultaneous campaigns has been reached;
        /// no more campaigns can be approved until one is closed.
		MaxCampaignsExceeded,
		/// Crowdsale period has exceeded the maximum active period.
		MaxActivePeriodExceeded,
		/// Maximum number of simultaneous proposals has been exceeded;
		/// no more proposals can be made until one is approved or rejected.
		MaxProposalsExceeded,
		/// You cannot withdraw funds because you have not contributed any.
		NoContribution,
		/// Proposal is already approved.
		ProposalAlreadyApproved,
		/// Proposal is not in the list of proposals.
		ProposalNotFound,
		/// The origin is not correct
		WrongOrigin,
		/// Crowdfund period is too short.
		ZeroPeriod,
	}

	#[pallet::event]
	#[pallet::generate_deposit(pub(crate) fn deposit_event)]
	#[pallet::metadata(T::AccountId = "AccountId", BalanceOf<T> = "Balance", CurrencyId = "CurrencyId")]
	pub enum Event<T: Config> {
		/// Created Proposal \[campaign_id\]
		CreatedProposal(CampaignId, CampaignInfoOf<T>),
		/// Contributed to a campaign \[contributor, campaign_id, amount\]
		Contributed(T::AccountId, CampaignId, BalanceOf<T>),
		/// Claim contribution allocation \[contributor, campaign_id, amount\]
		ClaimedContributionAlloc(T::AccountId, CampaignId, BalanceOf<T>),
		/// Claimed Funds Raised \[claimant_account_id, campaign_id, amount_claimed\]
		ClaimedFundraise(T::AccountId, CampaignId, BalanceOf<T>),
		/// Rejected Proposal \[campaign_id\]
		RejectedProposal(CampaignId),
		/// Approved Proposal \[campaign_id\]
		ApprovedProposal(CampaignId),
		/// Campaign Started \[campaign_id\]
		StartedCampaign(CampaignId),
		/// Ended Campaign Successfully \[campaign_id, campaign_info\]
		EndedCampaignSuccessful(CampaignId),
		/// Ended Campaign Unsuccessfully \[campaign_id, campaign_info\]
		EndedCampaignUnsuccessful(CampaignId),
		/// Contributed to Campaign \[campaign_id, contribution_amount\]
		ContributedToCampaign(CampaignId, BalanceOf<T>),
		/// Claimed Contribution Allocation \[claimant_account_id, campaign_id, allocation_claimed\]
		ClaimedAllocation(T::AccountId, CampaignId, BalanceOf<T>),
		/// Dissolved Unclaimed Funds \[amount, campaign_id, now\]
		DissolvedFunds(BalanceOf<T>, CampaignId, <T as frame_system::Config>::BlockNumber),
		/// Dispensed Commissions \[amount, campaign_id, now\]
		DispensedCommissions(BalanceOf<T>, CampaignId, <T as frame_system::Config>::BlockNumber),
	}
	
	/// Info on all of the proposed campaigns.
	///
	/// map CampaignId => CampaignInfo
	#[pallet::storage]
	#[pallet::getter(fn proposals)]
	pub type Proposals<T: Config> = StorageMap<_, Blake2_128Concat, CampaignId, CampaignInfoOf<T>, OptionQuery>;
	
	/// Info on all of the approved campaigns.
	///
	/// map CampaignId => CampaignInfo
	#[pallet::storage]
	#[pallet::getter(fn campaigns)]
	pub type Campaigns<T: Config> = StorageMap<_, Blake2_128Concat, CampaignId, CampaignInfoOf<T>, OptionQuery>;

	// Track the next campaign id to be used.
	#[pallet::storage]
	#[pallet::getter(fn campaign_index)]
	pub type CampaignsIndex<T: Config> = StorageValue<_, CampaignId, ValueQuery>;

	// Track the number of simultaneous Active Campaigns - ActiveCampaignsCount

	#[pallet::storage]
	#[pallet::getter(fn active_campaigns_count)]
	pub type ActiveCampaignsCount<T: Config> = StorageValue<_, CampaignId, ValueQuery>;

	// Track the number of successful campaigns the protocol has achieved.
	#[pallet::storage]
	#[pallet::getter(fn successful_campaign_index)]
	pub type SuccessfulCampaignsCount<T: Config> = StorageValue<_, CampaignId, ValueQuery>;


	/// Record of the total amount of funds raised in the protocol
	///  under a specific currency_id. currency_id => total_raised
	///
	/// TotalAmountRaised: map CurrencyIdOf<T> => BalanceOf<T>
	#[pallet::storage]
	#[pallet::getter(fn total_amount_raised)]
	pub type TotalAmountRaised<T: Config> = StorageMap<_, Twox64Concat, CurrencyIdOf<T>, BalanceOf<T>, ValueQuery>;

	#[pallet::pallet]
	pub struct Pallet<T>(PhantomData<T>);

	#[pallet::hooks]
	impl<T: Config> Hooks<T::BlockNumber> for Pallet<T> {
		// Call at the start of the block to eventuiate on_proposals and on_campaigns
		// on_initialize is called at the start of the block.
		fn on_initialize(now: T::BlockNumber) -> Weight {

			// Calls to eventuate proposals and campaigns.
			
			let mut count: Weight = 0;
			count += 1;

			// Eventuate proposals and campaigns

			// If there are proposals, check if to remove rejected and retired proposals.
			// Iterate over the proposals
			for (campaign_id, campaign_info) in Proposals::<T>::iter() {
				// If the proposal is rejected, check if to remove it
				if campaign_info.is_rejected && now >= campaign_info.proposal_retirement_period {
					// Remove the proposal
					Self::remove_proposal(campaign_id).unwrap();
					count += 1;
				}
				break;
			}
			// If there are campaigns, check if to start or end them
			// Iterate over the campaigns
			for (campaign_id, campaign_info) in Campaigns::<T>::iter() {
				// If the campaign is waiting, check if to start it
				if campaign_info.is_waiting && campaign_info.campaign_start <= now {
					// Activate Campaign
					Self::activate_campaign(campaign_id).unwrap();
					count += 1;
				}
				// If the campaign is active, check if to end it
				if campaign_info.is_active && !campaign_info.is_ended {
					// If campaign is successfull, call on successful campaign
					if campaign_info.raised >= campaign_info.goal {
						Self::on_successful_campaign(now, campaign_id).unwrap();
						count += 1;
					} else if campaign_info.campaign_end <= now && campaign_info.raised < campaign_info.goal {
						// If campaign is failed, call on failed campaign
						Self::on_failed_campaign(now, campaign_id).unwrap();
						count += 1;
					}
				}
				// If the campaign reaches retirement period, call on retirement
				if campaign_info.is_ended && &campaign_info.campaign_retirement_period <= &now {
					Self::on_retire(campaign_id).unwrap();
					count += 1;
				}
				break;
			}
			count
		}
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// Make a new proposal
		#[pallet::weight((100_000_000 as Weight, DispatchClass::Operational))]
		#[transactional]
		pub fn make_proposal(
			origin: OriginFor<T>,
			project_name: Vec<u8>,
			project_logo: Vec<u8>,
			project_description: Vec<u8>,
			project_website: Vec<u8>,
			beneficiary: T::AccountId,
			raise_currency: CurrencyIdOf<T>,
			sale_token: CurrencyIdOf<T>,
			token_price: BalanceOf<T>,
			crowd_allocation: BalanceOf<T>,
			goal: BalanceOf<T>,
			period: T::BlockNumber,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;

			// Ensure that the period is not zero
			ensure!(period > T::BlockNumber::zero(), Error::<T>::ZeroPeriod);
			// Ensure that the period is not too long
			ensure!(period <= T::MaxActivePeriod::get(), Error::<T>::MaxActivePeriodExceeded);
			// Ensure that the goal is not less than the Minimum Raise
			ensure!(goal > T::MinRaise::get(&raise_currency), Error::<T>::GoalBelowMinimumRaise);

			// Create proposal and add campaign_id.
			Self::new_proposal(
				who.clone(),
				project_name,
				project_logo,
				project_description,
				project_website,
				beneficiary,
				raise_currency,
				sale_token,
				token_price,
				crowd_allocation,
				goal,
				period,
			)?;
			Ok(())
		}

		// Make a contribution to an active campaign
		#[pallet::weight((100_000_000 as Weight, DispatchClass::Operational))]
		#[transactional]
		pub fn contribute(
			origin: OriginFor<T>,
			campaign_id: CampaignId,
			contribution_amount: BalanceOf<T>,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;

			Self::on_contribution(
				who.clone(),
				campaign_id,
				contribution_amount
			)?;
			Self::deposit_event(Event::Contributed(who, campaign_id, contribution_amount));
			Ok(())
		}

		// Claim a contribution allocation
		#[pallet::weight((100_000_000 as Weight, DispatchClass::Operational))]
		#[transactional]
		pub fn claim_contribution_allocation(
			origin: OriginFor<T>,
			campaign_id: CampaignId,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;

			Self::on_claim_allocation(
				who.clone(),
				campaign_id,
			)?;
			Ok(())
		}
		
		// Claim a campaign's raised funds
		#[pallet::weight((100_000_000 as Weight, DispatchClass::Operational))]
		#[transactional]
		pub fn claim_campaign_fundraise(
			origin: OriginFor<T>,
			campaign_id: CampaignId,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;

			Self::on_claim_campaign(
				who.clone(),
				campaign_id,
			)?;

			let campaign = Self::campaigns(campaign_id).ok_or(Error::<T>::CampaignNotFound)?;
			Self::deposit_event(Event::ClaimedFundraise(who, campaign_id, campaign.raised));
			Ok(())
		}
		
		// Approve a proposal - origin must be `UpdateOrigin`
		#[pallet::weight((100_000_000 as Weight, DispatchClass::Operational))]
		#[transactional]
		pub fn approve_proposal(
			origin: OriginFor<T>,
			campaign_id: CampaignId,
		) -> DispatchResult {
			T::UpdateOrigin::ensure_origin(origin)?;

			Self::on_approve_proposal(
				campaign_id,
			)?;
			
			Self::deposit_event(Event::ApprovedProposal(campaign_id));
			Ok(())
		}
		
		// Reject a proposal - origin must be `UpdateOrigin`
		#[pallet::weight((100_000_000 as Weight, DispatchClass::Operational))]
		#[transactional]
		pub fn reject_proposal(
			origin: OriginFor<T>,
			campaign_id: CampaignId,
		) -> DispatchResult {
			T::UpdateOrigin::ensure_origin(origin)?;

			Self::on_reject_proposal(
				campaign_id,
			)?;

			Self::deposit_event(Event::RejectedProposal(campaign_id));
			Ok(())
		}
	}
}


impl<T: Config> Pallet<T> {
	/// Get the Launchpad's Treasury  Account.
	pub fn launchpad_treasury() -> T::AccountId {
		T::PalletId::get().into_account()
	}

	/// The account ID of the fund pot.
	///
	pub fn campaign_pool(id: CampaignId) -> T::AccountId {
		T::PalletId::get().into_sub_account(id)
	}
}

impl<T: Config> Proposal<T::AccountId, T::BlockNumber> for Pallet<T> {
	/// The Campaign Proposal info of `id`
	fn proposal_info(id: CampaignId) -> Option<CampaignInfo<T::AccountId, Balance, T::BlockNumber>> {
		<Proposals<T>>::get(id)
	}

	/// Get all proposals
	fn all_proposals() -> Vec<CampaignInfo<T::AccountId, Balance, T::BlockNumber>> {
		let proposals = Proposals::<T>::iter().into_iter();
		let mut proposals_vec: Vec<CampaignInfo<T::AccountId, Balance, T::BlockNumber>> = Vec::new();
		for (_, proposal) in proposals {
			proposals_vec.push(proposal);
		}
		proposals_vec
	}

	/// Create new Campaign Proposal with specific `CampaignInfo`, return the `id` of the Campaign
	fn new_proposal(
		origin: T::AccountId,
		project_name: Vec<u8>,
		project_logo: Vec<u8>,
		project_description: Vec<u8>,
		project_website: Vec<u8>,
		beneficiary: T::AccountId,
		raise_currency: CurrencyIdOf<T>,
		sale_token: CurrencyIdOf<T>,
		token_price: BalanceOf<T>,
		crowd_allocation: BalanceOf<T>,
		goal: BalanceOf<T>,
		period: T::BlockNumber,
	) -> DispatchResult {

		ensure!(token_price * crowd_allocation == goal, Error::<T>::GoalNotAllignedWithAllocation);

		// Generate campaign_id - overflow not managed
		let campaign_id = <CampaignsIndex<T>>::get() + 1;
		<CampaignsIndex<T>>::put(campaign_id);

		// Generate the CampaignInfo structure
		let proposal = CampaignInfo {
			id: campaign_id,
			origin: origin.clone(),
			project_name: project_name,
			project_logo: project_logo,
			project_description: project_description,
			project_website: project_website,
			beneficiary: beneficiary,
			pool: Self::campaign_pool(campaign_id),
			raise_currency: raise_currency,
			sale_token: sale_token,
			token_price: token_price,
			crowd_allocation: crowd_allocation,
			goal: goal,
			raised: Zero::zero(),
			contributors_count: Zero::zero(),
			contributions: Vec::new(),
			period: period,
			campaign_start: Zero::zero(),
			campaign_end: Zero::zero(),
			campaign_retirement_period: Zero::zero(),
			proposal_retirement_period: Zero::zero(),
			is_approved: false,
			is_rejected: false,
			is_waiting: false,
			is_active: false,
			is_successful: false,
			is_failed: false,
			is_ended: false,
			is_claimed: false,
		};

		// try checks
		let try_set_lock = T::MultiCurrency::set_lock(LAUNCHPAD_LOCK_ID, T::GetNativeCurrencyId::get(), &origin, T::SubmissionDeposit::get()).is_ok();
		let try_make_transfer = T::MultiCurrency::transfer(sale_token, &origin, &Self::campaign_pool(campaign_id), crowd_allocation).is_ok() ;

		if T::MultiCurrency::free_balance(T::GetNativeCurrencyId::get(), &origin) >= T::SubmissionDeposit::get() &&
			T::MultiCurrency::free_balance(sale_token, &origin) >= crowd_allocation {
				if try_set_lock && try_make_transfer {
					// set lock
					T::MultiCurrency::set_lock(LAUNCHPAD_LOCK_ID, T::GetNativeCurrencyId::get(), &origin, T::SubmissionDeposit::get()).unwrap();
					// make transfer
					T::MultiCurrency::transfer(sale_token, &origin, &Self::campaign_pool(campaign_id), crowd_allocation).unwrap();
					// insert proposal
					<Proposals<T>>::insert(campaign_id, proposal.clone());
				}
		} else {
			return Err(Error::<T>::InsufficientBalance.into());
		}
		
		Self::deposit_event(Event::CreatedProposal(campaign_id, proposal.clone()));
		Ok(())
	}

    /// Approve Proposal by `id` at `now`.
    fn on_approve_proposal(id: CampaignId)-> sp_std::result::Result<(), DispatchError> {
		// Tag the proposal and ensure it is not already approved.
		let mut proposal = Self::proposals(id).ok_or(Error::<T>::ProposalNotFound)?;
		ensure!(!proposal.is_approved, Error::<T>::ProposalAlreadyApproved);

		// Approve the proposal in CampaignInfo and set it to waiting
		proposal.is_approved = true;
		proposal.is_waiting = true;

		// Set campaign start time
		proposal.campaign_start = <frame_system::Pallet<T>>::block_number() + T::CampaignStartDelay::get();

		// Set campaign end time
		proposal.campaign_end = <frame_system::Pallet<T>>::block_number() + T::CampaignStartDelay::get() + proposal.period;

		// Remove from proposals and add to campaigns
		<Proposals<T>>::remove(id);
		<Campaigns<T>>::insert(id, proposal);
		// Active Campaigns count - overflow not managed
		<ActiveCampaignsCount<T>>::put(<ActiveCampaignsCount<T>>::get() + 1);
		Ok(())
	}
	
	/// Reject Proposal by `id` and remove from storage.
	fn on_reject_proposal(id: CampaignId)-> sp_std::result::Result<(), DispatchError> {
		// Check that the Proposal exists and tag it
		let mut proposal = Self::proposals(id).ok_or(Error::<T>::ProposalNotFound)?;
		// Ensure that the proposal is not already approved
		ensure!(!proposal.is_approved, Error::<T>::ProposalAlreadyApproved);

		// Set the proposal to rejected
		proposal.is_rejected = true;
		proposal.proposal_retirement_period = <frame_system::Pallet<T>>::block_number() + T::ProposalRetirementPeriod::get();
		// Update proposal storage
		<Proposals<T>>::insert(id, proposal);
		Ok(())
	}

	/// Remove proposal from storage by `id`
	fn remove_proposal(id: CampaignId)-> sp_std::result::Result<(), DispatchError> {
		// Check that the Proposal exists and tag it
		let proposal = Self::proposals(id).ok_or(Error::<T>::ProposalNotFound)?;
		// Ensure that the proposal is not already approved
		ensure!(!proposal.is_approved, Error::<T>::ProposalAlreadyApproved);
		ensure!(proposal.is_rejected, Error::<T>::ProposalAlreadyApproved);

		let try_remove_lock = T::MultiCurrency::remove_lock(LAUNCHPAD_LOCK_ID, T::GetNativeCurrencyId::get(), &proposal.origin).is_ok();
		let try_refund_transfer = T::MultiCurrency::transfer( proposal.sale_token, &proposal.pool, &proposal.origin, proposal.crowd_allocation).is_ok();
		// Unlock balances and remove the Proposal from the storage.
		if try_remove_lock && try_refund_transfer {
			// remove lock and refund proposal
			T::MultiCurrency::remove_lock(LAUNCHPAD_LOCK_ID, T::GetNativeCurrencyId::get(), &proposal.origin).unwrap();
			T::MultiCurrency::transfer( proposal.sale_token, &proposal.pool, &proposal.origin, proposal.crowd_allocation).unwrap();
			// Remove from proposals
			<Proposals<T>>::remove(id);
		};
		Ok(())
	}
}

impl<T: Config> CampaignManager<T::AccountId, T::BlockNumber> for Pallet<T> {
	/// The Campaign info of `id`
	fn campaign_info(id: CampaignId) -> Option<CampaignInfo<T::AccountId, Balance, T::BlockNumber>> {
		<Campaigns<T>>::get(id)
	}

	/// Get all campaigns
	fn all_campaigns() -> Vec<CampaignInfo<T::AccountId, Balance, T::BlockNumber>> {
		let campaigns = Campaigns::<T>::iter().into_iter();
		let mut campaigns_vec: Vec<CampaignInfo<T::AccountId, Balance, T::BlockNumber>> = Vec::new();
		for (_, proposal) in campaigns {
			campaigns_vec.push(proposal);
		}
		campaigns_vec
	}

	/// Called when a contribution is received.
	fn on_contribution(
		who: T::AccountId,
		id: CampaignId,
		amount: BalanceOf<T>,
	) -> DispatchResult {
		let mut campaign = Self::campaigns(id).ok_or(Error::<T>::CampaignNotFound)?;

		// Ensure campaign is valid
		ensure!(!campaign.is_failed, Error::<T>::CampaignFailed);
		ensure!(!campaign.is_ended, Error::<T>::CampaignEnded);
		ensure!(campaign.is_approved, Error::<T>::CampaignNotApproved);
		ensure!(campaign.is_active, Error::<T>::CampaignNotActive);

		// Make assurances - minimum contribution & free balance
		ensure!(amount >= T::MinContribution::get(&campaign.raise_currency), Error::<T>::ContributionTooSmall);
		ensure!(T::MultiCurrency::free_balance(campaign.raise_currency, &who) >= amount, Error::<T>::ContributionCurrencyNotEnough);
		
		// Initiate the Contribution
		let transfer_contribution = T::MultiCurrency::transfer(campaign.raise_currency, &who, &campaign.pool, amount).is_ok();
		if transfer_contribution {
			// Transfer contribution and tag allocation
			T::MultiCurrency::transfer(campaign.raise_currency, &who, &campaign.pool, amount).unwrap();
			let allocated = amount / campaign.token_price;

			// Check if contributor already exists in contributions list
			let mut found = false;
			// if campaign.contributions exists, check for who's contribution
			
			for (contributor, contribution, allocation, _) in campaign.contributions.iter_mut() {
				if contributor == &who {

					found = true;
					*contribution += amount;
					*allocation += allocated;
					campaign.raised += amount;
				}
				break;
			}
			if !found {
				campaign.contributions.push((who, amount, allocated, false));
				campaign.raised += amount;
			}

			// Tag contributors count
			campaign.contributors_count = campaign.contributions.len() as u32;

			// Put campaign in campaigns storage
			<Campaigns<T>>::insert(id, campaign);
		};
		Ok(())
	}

	/// Called when a contribution allocation is claimed
	fn on_claim_allocation(
		who: T::AccountId,
		id: CampaignId,
	) -> DispatchResult {
		let mut campaign = Self::campaigns(id).ok_or(Error::<T>::CampaignNotFound)?;
		let  campaign_p = Self::campaigns(id).ok_or(Error::<T>::CampaignNotFound)?;

		// Check if the contributor exists in the contributions of the campaign, if not return error
		ensure!(campaign.contributions.iter().any(|(contributor, _, _, _)| *contributor == who), Error::<T>::ContributionNotFound);

		// Ensure campaign is successfully ended
		Self::ensure_successfully_ended_campaign(id)?;

		// Check if the contributor exists and transfer allocated from pool to contributor
		for (contributor, _, allocation, claimed) in campaign.contributions.iter_mut() {
			let transfer_allocation = T::MultiCurrency::transfer(campaign.sale_token, &campaign.pool, &who, *allocation).is_ok();

			if contributor == &who && *claimed == false && transfer_allocation {
				// set claimed to true - allocation claimed
				*claimed = true;
				// complete claim by adding campaign update to storage
				<Campaigns<T>>::insert(id, campaign);

				for (contributor_p, _, allocation_p, claimed_p) in campaign_p.contributions.iter() {
					if contributor_p == &who && *claimed_p == false {		
						// transfer allocation
						T::MultiCurrency::transfer(campaign_p.sale_token, &campaign_p.pool, &who, *allocation_p).unwrap();
						Self::deposit_event(Event::ClaimedContributionAlloc(who, id, *allocation_p));
					}

					break;
				}
			}

			break;
		}
		Ok(())
	}

	/// Called when a campaign's raised fund is claimed
	fn on_claim_campaign(
		who: T::AccountId,
		id: CampaignId,
	) -> DispatchResult {
		let mut campaign = Self::campaigns(id).ok_or(Error::<T>::CampaignNotFound)?;

		// Ensure origin is who created campaign proposal or beneficiary and its not claimed
		ensure!(campaign.origin == who || campaign.beneficiary == who, Error::<T>::WrongOrigin);
		ensure!(!campaign.is_claimed, Error::<T>::CampaignAlreadyClaimed);

		// Ensure campaign is valid
		ensure!(campaign.is_ended, Error::<T>::CampaignStillActive);

		// Claim the campaign raised funds and transfer to the beneficiary
		let transfer_claim = T::MultiCurrency::transfer(
			campaign.raise_currency,
			&campaign.pool,
			&campaign.beneficiary,
			campaign.raised
		)
		.is_ok();

		if campaign.is_successful && transfer_claim {
			T::MultiCurrency::transfer(
				campaign.raise_currency,
				&campaign.pool,
				&campaign.beneficiary,
				campaign.raised
			).unwrap();
			// Campaign is claimed, update storage
			campaign.is_claimed = true;
			<Campaigns<T>>::insert(id, campaign);
		}
		Ok(())
	}

	/// Called when a failed campaign is claimed by the proposer
	fn on_claim_failed_campaign(
		who: T::AccountId,
		id: CampaignId,
	) -> DispatchResult {
		let campaign = Self::campaigns(id).ok_or(Error::<T>::CampaignNotFound)?;

		// Ensure origin is who created campaign proposal
		ensure!(campaign.origin == who || campaign.beneficiary == who, Error::<T>::WrongOrigin);

		// Ensure campaign is valid and failed
		ensure!(campaign.is_failed, Error::<T>::CampaignFailed);
		ensure!(campaign.is_ended, Error::<T>::CampaignEnded);

		// Get the total amount of sale_token in the pool
		let total_sale_token = T::MultiCurrency::total_balance(campaign.sale_token, &campaign.pool);
		
		let remove_lock = T::MultiCurrency::remove_lock(LAUNCHPAD_LOCK_ID, T::GetNativeCurrencyId::get(), &campaign.origin).is_ok();
		let transfer_claim = T::MultiCurrency::transfer( campaign.sale_token, &campaign.pool, &who, total_sale_token).is_ok();
		// Unlock balances and remove the Proposal from the storage.
		if remove_lock && transfer_claim {
			T::MultiCurrency::remove_lock(LAUNCHPAD_LOCK_ID, T::GetNativeCurrencyId::get(), &campaign.origin).unwrap();
			T::MultiCurrency::transfer( campaign.sale_token, &campaign.pool, &who, total_sale_token).unwrap();
			// Update campaign in campaigns storage
			<Campaigns<T>>::insert(id, campaign);
		};
		Ok(())
	}

	/// Activate a campaign by `id`
	fn activate_campaign(id: CampaignId) -> DispatchResult {
		// Ensure campaign exists
		let mut campaign = Self::campaigns(id).ok_or(Error::<T>::CampaignNotFound)?;

		// Set campaign to active
		campaign.is_waiting = false;
		campaign.is_active = true;
		// Update campaign storage
		<Campaigns<T>>::insert(id, campaign);
		Ok(())
	}

	/// Ensure campaign is Valid and Successfully Ended
	fn ensure_successfully_ended_campaign(id: CampaignId) -> DispatchResult {
		let campaign = Self::campaigns(id).ok_or(Error::<T>::CampaignNotFound)?;
		ensure!(!campaign.is_failed, Error::<T>::CampaignFailed);
		ensure!(campaign.is_successful, Error::<T>::CampaignFailed);
		ensure!(campaign.is_ended, Error::<T>::CampaignStillActive);
		ensure!(campaign.is_approved, Error::<T>::CampaignNotApproved);

		ensure!(campaign.campaign_start <= <frame_system::Pallet<T>>::block_number(), Error::<T>::CampaignNotStarted);
		Ok(())
	}

	/// Record Successful Campaign by `id`
	fn on_successful_campaign(now: T::BlockNumber, id: CampaignId) -> DispatchResult {
		let mut campaign = Self::campaigns(id).ok_or(Error::<T>::CampaignNotFound)?;
		
		// Set to successful and ended
		campaign.is_successful = true;
		campaign.is_ended = true;

		// Set retirement period
		campaign.campaign_retirement_period = now + T::CampaignRetirementPeriod::get();

		// Tag contributors count
		campaign.contributors_count = campaign.contributions.len() as u32;
		

		// Success count - overflow not managed
		// Add to total successful campaigns
		let success_count = <SuccessfulCampaignsCount<T>>::get() + 1;
		<SuccessfulCampaignsCount<T>>::put(success_count);

		// Add to `TotalAmountRaised` in protocol
		<TotalAmountRaised<T>>::mutate(campaign.raise_currency,  |total| *total += campaign.raised);
		
		// Update campaign storage
		<Campaigns<T>>::insert(id, campaign);
		Ok(())
	}

	/// Record Failed Campaign by `id`
	fn on_failed_campaign(now: T::BlockNumber, id: CampaignId) -> DispatchResult {
		let mut campaign = Self::campaigns(id).ok_or(Error::<T>::CampaignNotFound)?;
		
		// Set to failed and ended
		campaign.is_failed = true;
		campaign.is_ended = true;

		// Tag contributors count
		campaign.contributors_count = campaign.contributions.len() as u32;
		
		// Set retirement period
		campaign.campaign_retirement_period = now + T::CampaignRetirementPeriod::get();
		
		// Update campaign storage
		<Campaigns<T>>::insert(id, campaign);
		Ok(())
	}

	/// Called when pool is retired
	/// Only unsuccessful pools are retired
	fn on_retire(id: CampaignId) -> DispatchResult {
		// Get campaign in tag
		let campaign = Self::campaigns(id).ok_or(Error::<T>::CampaignNotFound)?;
		// Get accounts in tag
		let treasury = Self::launchpad_treasury();

		// Get the total amount of raise_currency in the pool
		let total_raise_currency = T::MultiCurrency::total_balance(campaign.raise_currency, &campaign.pool);
		// Get the total amount of sale_token in the pool
		let total_sale_token = T::MultiCurrency::total_balance(campaign.sale_token, &campaign.pool);
		
		let transfer_allocation = T::MultiCurrency::transfer(campaign.raise_currency, &campaign.pool, &treasury, total_raise_currency).is_ok();
		let transfer_raise = T::MultiCurrency::transfer(campaign.sale_token, &campaign.pool, &treasury, total_sale_token).is_ok();
		// Dissolve unclaimed Fundraise 
		if transfer_allocation && transfer_raise {
			T::MultiCurrency::transfer(campaign.raise_currency, &campaign.pool, &treasury, total_raise_currency).unwrap();	
			T::MultiCurrency::transfer(campaign.sale_token, &campaign.pool, &treasury, total_sale_token).unwrap();
			// Remove campaign from campaigns storage
			<Campaigns<T>>::remove(id);
		}
		Ok(())
	}

	/// Get amount of contributors/contributions in a campaign
	fn get_contributors_count(id: CampaignId) -> u32 {
		let campaign = Self::campaigns(id).unwrap();
		campaign.contributions.len() as u32
	}

	/// Get the total_amounts_raised for all currencies from `TotalAmountRaised`
	fn get_total_amounts_raised() -> Vec<(CurrencyId, BalanceOf<T>)> {
		let total_amounts_raised: Vec<(CurrencyId, BalanceOf<T>)> = <TotalAmountRaised<T>>::iter()
			.into_iter()
			.collect::<Vec<(CurrencyId, BalanceOf<T>)>>();
		total_amounts_raised
	}
}
