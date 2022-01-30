//! Traits for the Launchpad Crowdsales Pallet.

use codec::{Decode, Encode};
use sp_runtime::{
	DispatchError, DispatchResult,
};
use sp_std::{
	cmp::{Eq, PartialEq},
};

pub type CurrencyId = u32;
pub type CampaignId = u32;
pub type Balance = u32;

/// The Structure of a Campaign info.
#[cfg_attr(feature = "std", derive(PartialEq, Eq, Encode, Decode))]
pub struct CampaignInfo<AccountId, Balance, BlockNumber> {
	/// Campaign Creator
	pub origin: AccountId,
	/// Project Name
	pub project_name: Vec<u8>,
	/// Project Logo
	pub project_logo: Vec<u8>,
	/// Project Description
	pub project_description: Vec<u8>,
	/// Project Website
	pub project_website: Vec<u8>,
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
	/// The time when the proposal was made.
	pub proposal_time: BlockNumber,
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

/// Abstraction over th Launchpad Proposal system.
pub trait Proposal<AccountId, BlockNumber> {
	/// The Campaign Proposal info of `id`
	fn proposal_info(id: CampaignId) -> Option<CampaignInfo<AccountId, Balance, BlockNumber>>;
	/// Create new Campaign Proposal with specific `CampaignInfo`, return the `id` of the Campaign
	fn new_proposal(
		origin: AccountId,
		project_name: Vec<u8>,
		project_logo: Vec<u8>,
		project_description: Vec<u8>,
		project_website: Vec<u8>,
		beneficiary: AccountId,
		raise_currency: CurrencyId,
		sale_token: CurrencyId,
		token_price: Balance,
		crowd_allocation: Balance,
		goal: Balance,
		period: BlockNumber,
	) -> DispatchResult;
	/// Ensure proposal is valid
	fn ensure_valid_proposal(id: CampaignId) -> sp_std::result::Result<(), DispatchError>;
    /// Approve Proposal by `id` at `now`.
    fn approve_proposal(id: CampaignId) -> sp_std::result::Result<(), DispatchError>;
	/// Reject Proposal by `id` and update storage
	fn reject_proposal(id: CampaignId) -> sp_std::result::Result<(), DispatchError>;
	/// Remove Proposal by `id` and remove from storage
	fn remove_proposal(id: CampaignId) -> sp_std::result::Result<(), DispatchError>;
}

/// Abstraction over the Launchpad Campaign system.
pub trait CampaignManager<AccountId, BlockNumber> {
	/// The Campaign info of `id`
	fn campaign_info(id: CampaignId) -> Option<CampaignInfo<AccountId, Balance, BlockNumber>>;
	/// Called when a contribution is received.
	fn on_contribution(
		who: AccountId,
		id: CampaignId,
		amount: Balance,
	) -> DispatchResult;
	/// Called when a contribution allocation is claimed
	fn on_claim_allocation(
		who: AccountId,
		id: CampaignId,
	) -> DispatchResult;
	/// Called when a campaign's raised fund is claimed
	fn on_claim_campaign(
		who: AccountId,
		id: CampaignId,
	) -> DispatchResult;
	/// Called when a failed campaign is claimed by the proposer
	fn on_claim_failed_campaign(
		who: AccountId,
		id: CampaignId,
	) -> DispatchResult;
	/// Ensure campaign is Valid and Running
	fn ensure_valid_running_campaign(id: CampaignId) -> DispatchResult;
	/// Ensure campaign is Valid and Ended
	fn ensure_ended_campaign(id: CampaignId) -> DispatchResult;
	/// Ensure campaign is Valid and Successfully Ended
	fn ensure_successfully_ended_campaign(id: CampaignId) -> DispatchResult;
	/// Record Successful Campaign by `id`
	fn on_successful_campaign(id: CampaignId) -> DispatchResult ;
	/// Record Failed Campaign by `id`
	fn on_failed_campaign(id: CampaignId) -> DispatchResult ;
	/// Called when pool is retired
	fn on_retire(id: CampaignId)-> DispatchResult;
}
