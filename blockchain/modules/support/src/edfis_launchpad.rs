// بِسْمِ اللَّهِ الرَّحْمَنِ الرَّحِيم

// This file is part of Setheum.

// Copyright (C) 2019-Present Setheum Labs.
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

//! Traits for the Launchpad Crowdsales Pallet.

use codec::{Decode, Encode};
use sp_runtime::{
	DispatchError, DispatchResult,
};
use sp_std::{
	cmp::{Eq, PartialEq},
};

/// Abstraction over th Launchpad Proposal system.
pub trait Proposal<AccountId, BlockNumber> {
	type CurrencyId;

	/// The Campaign Proposal info of `id`
	fn proposal_info(id: Self::CurrencyId) -> Option<CampaignInfo<AccountId, Balance, BlockNumber>>;
	/// Get all proposals
	fn all_proposals() -> Vec<CampaignInfo<AccountId, Balance, BlockNumber>>;
	/// Create new Campaign Proposal with specific `CampaignInfo`, return the `id` of the Campaign
	fn new_proposal(
		origin: AccountId,
		beneficiary: AccountId,
		raise_currency: CurrencyId,
		sale_token: CurrencyId,
		token_price: Balance,
		crowd_allocation: Balance,
		goal: Balance,
		period: BlockNumber,
	) -> DispatchResult;
    /// Approve Proposal by `id` at `now`.
    fn on_approve_proposal(id: Self::CurrencyId) -> sp_std::result::Result<(), DispatchError>;
	/// Reject Proposal by `id` and update storage
	fn on_reject_proposal(id: Self::CurrencyId) -> sp_std::result::Result<(), DispatchError>;
	/// Remove Proposal by `id` from storage
	fn remove_proposal(id: Self::CurrencyId) -> sp_std::result::Result<(), DispatchError>;
}

/// Abstraction over the Launchpad Campaign system.
pub trait CampaignManager<AccountId, BlockNumber> {
	type CurrencyId;

	/// The Campaign info of `id`
	fn campaign_info(id: Self::CurrencyId) -> Option<CampaignInfo<AccountId, Balance, BlockNumber>>;
	/// Get all proposals
	fn all_campaigns() -> Vec<CampaignInfo<AccountId, Balance, BlockNumber>>;
	/// Called when a contribution is received.
	fn on_contribution(
		who: AccountId,
		id: Self::CurrencyId,
		amount: Balance,
	) -> DispatchResult;
	/// Called when a contribution allocation is claimed
	fn on_claim_allocation(
		who: AccountId,
		id: Self::CurrencyId,
	) -> DispatchResult;
	/// Called when a campaign's raised fund is claimed
	fn on_claim_campaign(
		who: AccountId,
		id: Self::CurrencyId,
	) -> DispatchResult;
	/// Called when a failed campaign is claimed by the proposer
	fn on_claim_failed_campaign(
		who: AccountId,
		id: Self::CurrencyId,
	) -> DispatchResult;
	/// Activate a campaign by `id`
	fn activate_campaign(id: Self::CurrencyId) -> DispatchResult;
	/// Ensure campaign is Valid and Successfully Ended
	fn ensure_successfully_ended_campaign(id: Self::CurrencyId) -> DispatchResult;
	/// Record Successful Campaign by `id`
	fn on_successful_campaign(now: BlockNumber, id: Self::CurrencyId) -> DispatchResult ;
	/// Record Failed Campaign by `id`
	fn on_failed_campaign(now: BlockNumber, id: Self::CurrencyId) -> DispatchResult ;
	/// Called when pool is retired
	fn on_retire(id: Self::CurrencyId)-> DispatchResult;
	/// Get amount of contributors in a campaign
	fn get_contributors_count(id: Self::CurrencyId) -> u32;
	/// Get the total amounts raised in protocol
	fn get_total_amounts_raised() -> Vec<(CurrencyId, Balance)>;
}
