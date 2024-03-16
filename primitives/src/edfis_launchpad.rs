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

//! Primitives for the Edfis Launchpad module.

use codec::{Decode, Encode};
use sp_runtime::{
	DispatchError, DispatchResult, RuntimeDebug,
};
use sp_std::{
	cmp::{Eq, PartialEq},
};

/// Launchpad Campaign ID
pub type CampaignId = u32;

/// The Structure of a Campaign info.
// #[derive(Encode, Decode, Clone, RuntimeDebug, PartialEq, Eq)]
#[derive(Encode, Decode, Clone, Eq, PartialEq, RuntimeDebug)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub struct CampaignInfo<AccountId, Balance, BlockNumber> {
	/// The Campaign Id
	pub id: CurrencyId,
	/// Campaign Creator
	pub origin: AccountId,
	/// Campaign Beneficiary
	pub beneficiary: AccountId,
	/// Campaign Pool AccountId
	pub pool: AccountId,
	/// Currency type for the fundraise
	pub raise_currency: CurrencyId,
	/// Currency type (Token) for crowdsale
	pub sale_token: CurrencyId,
	/// Crowdsale Token Price - Amount of raise_currency per sale_token
	pub token_price: Balance,
	/// Crowdsale Token amount for sale
	pub crowd_allocation: Balance,
	/// The Fundraise Goal - HardCap
	pub goal: Balance,
	/// The Fundraise Amount raised - HardCap
	pub raised: Balance,
	/// The number of contributors to the campaign
	pub contributors_count: u32,
	/// The Campaign contributions
	/// account_id, contribution, allocation, bool:claimed_allocation
	pub contributions: Vec<(AccountId, Balance, Balance, bool)>,
	/// The period that the campaign runs for.
	pub period: BlockNumber,
	/// The time when the campaign starts.
	pub campaign_start: BlockNumber,
	/// The time when the campaign ends.
	pub campaign_end: BlockNumber,
	/// The time when the campaign fund retires.
	pub campaign_retirement_period: BlockNumber,
	/// The time when a rejected proposal is removed from storage.
	pub proposal_retirement_period: BlockNumber,
	/// Is the campaign approved?
	pub is_approved: bool,
	/// Is the proposal rejected?
	pub is_rejected: bool,
	/// Is the campaign in waiting period?
	pub is_waiting: bool,
	/// Is the campaign active?
	pub is_active: bool,
	/// Is the campaign Successful?
	pub is_successful: bool,
	/// Is the campaign Failed?
	pub is_failed: bool,
	/// Is the campaign Ended?
	pub is_ended: bool,
	/// Is the campaign funds raised claimed
	pub is_claimed: bool,
}
