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

#![cfg_attr(not(feature = "std"), no_std)]
#![allow(clippy::upper_case_acronyms)]

use codec::{Decode, Encode, FullCodec};
use frame_support::pallet_prelude::{DispatchClass, Pays, Weight};
use primitives::{
	Balance as AsBalance,
	CampaignId, CurrencyId,
	evm::{CallInfo, EvmAddress},
	task::TaskResult
};
use scale_info::TypeInfo;
use sp_core::H160;
use sp_runtime::{
	traits::{AtLeast32BitUnsigned, CheckedDiv, MaybeSerializeDeserialize},
	transaction_validity::TransactionValidityError,
	DispatchError, DispatchResult, FixedU128, RuntimeDebug,
};
use sp_std::{
	cmp::{Eq, PartialEq},
	fmt::Debug,
	prelude::*,
};

pub mod mocks;

pub type Price = FixedU128;
pub type ExchangeRate = FixedU128;
pub type Ratio = FixedU128;
pub type Rate = FixedU128;

pub trait RiskManager<AccountId, CurrencyId, Balance, DebitBalance> {
	fn get_debit_value(currency_id: CurrencyId, debit_balance: DebitBalance) -> Balance;

	fn check_position_valid(
		currency_id: CurrencyId,
		collateral_balance: Balance,
		debit_balance: DebitBalance,
		check_required_ratio: bool,
	) -> DispatchResult;

	fn check_debit_cap(currency_id: CurrencyId, total_debit_balance: DebitBalance) -> DispatchResult;
}

#[cfg(feature = "std")]
impl<AccountId, CurrencyId, Balance: Default, DebitBalance> RiskManager<AccountId, CurrencyId, Balance, DebitBalance>
	for ()
{
	fn get_debit_value(_currency_id: CurrencyId, _debit_balance: DebitBalance) -> Balance {
		Default::default()
	}

	fn check_position_valid(
		_currency_id: CurrencyId,
		_collateral_balance: Balance,
		_debit_balance: DebitBalance,
		_check_required_ratio: bool,
	) -> DispatchResult {
		Ok(())
	}

	fn check_debit_cap(_currency_id: CurrencyId, _total_debit_balance: DebitBalance) -> DispatchResult {
		Ok(())
	}
}

pub trait AuctionManager<AccountId> {
	type CurrencyId;
	type Balance;
	type AuctionId: FullCodec + Debug + Clone + Eq + PartialEq;

	fn new_collateral_auction(
		refund_recipient: &AccountId,
		currency_id: Self::CurrencyId,
		amount: Self::Balance,
		target: Self::Balance,
	) -> DispatchResult;
	fn cancel_auction(id: Self::AuctionId) -> DispatchResult;
	fn get_total_collateral_in_auction(id: Self::CurrencyId) -> Self::Balance;
	fn get_total_target_in_auction() -> Self::Balance;
}

/// The Structure of a Campaign info.
#[cfg_attr(feature = "std", derive(PartialEq, Eq, Encode, Decode, Debug, Clone))]
pub struct CampaignInfo<AccountId, Balance, BlockNumber> {
	/// The Campaign Id
	pub id: CampaignId,
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
	/// Get all proposals
	fn all_proposals() -> Vec<CampaignInfo<AccountId, AsBalance, BlockNumber>>;
	/// The Campaign Proposal info of `id`
	fn proposal_info(id: CampaignId) -> Option<CampaignInfo<AccountId, AsBalance, BlockNumber>>;
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
		token_price: AsBalance,
		crowd_allocation: AsBalance,
		goal: AsBalance,
		period: BlockNumber,
	) -> DispatchResult;
    /// Approve Proposal by `id` at `now`.
    fn on_approve_proposal(id: CampaignId) -> sp_std::result::Result<(), DispatchError>;
	/// Reject Proposal by `id` and update storage
	fn on_reject_proposal(id: CampaignId) -> sp_std::result::Result<(), DispatchError>;
	/// Remove Proposal by `id` from storage
	fn remove_proposal(id: CampaignId) -> sp_std::result::Result<(), DispatchError>;
}

/// Abstraction over the Launchpad Campaign system.
pub trait CampaignManager<AccountId, BlockNumber> {
	/// The Campaign info of `id`
	fn campaign_info(id: CampaignId) -> Option<CampaignInfo<AccountId, AsBalance, BlockNumber>>;
	/// Get all proposals
	fn all_campaigns() -> Vec<CampaignInfo<AccountId, AsBalance, BlockNumber>>;
	/// Called when a contribution is received.
	fn on_contribution(
		who: AccountId,
		id: CampaignId,
		amount: AsBalance,
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
	/// Activate a campaign by `id`
	fn activate_campaign(id: CampaignId) -> DispatchResult;
	/// Ensure campaign is Valid and Successfully Ended
	fn ensure_successfully_ended_campaign(id: CampaignId) -> DispatchResult;
	/// Record Successful Campaign by `id`
	fn on_successful_campaign(now: BlockNumber, id: CampaignId) -> DispatchResult ;
	/// Record Failed Campaign by `id`
	fn on_failed_campaign(now: BlockNumber, id: CampaignId) -> DispatchResult ;
	/// Called when pool is retired
	fn on_retire(id: CampaignId)-> DispatchResult;
	/// Get amount of contributors in a campaign
	fn get_contributors_count(id: CampaignId) -> u32;
	/// Get the total amounts raised in protocol
	fn get_total_amounts_raised() -> Vec<(CurrencyId, AsBalance)>;
}

#[derive(RuntimeDebug, Encode, Decode, Clone, Copy, PartialEq, TypeInfo)]
pub enum SwapLimit<Balance> {
	/// use exact amount supply amount to swap. (exact_supply_amount, minimum_target_amount)
	ExactSupply(Balance, Balance),
	/// swap to get exact amount target. (maximum_supply_amount, exact_target_amount)
	ExactTarget(Balance, Balance),
}

// #[derive(RuntimeDebug, Encode, Decode, Clone, Copy, PartialEq, TypeInfo)]
// pub enum SerpingStatus<BlockNumber> {
// 	/// Enable/Activate serping of setcurrencies (period).
// 	Active(BlockNumber),
// 	/// Disable/Deactivate serping of setcurrencies.
// 	Inactive,
// }

pub trait DEXManager<AccountId, CurrencyId, Balance> {
	fn get_liquidity_pool(currency_id_a: CurrencyId, currency_id_b: CurrencyId) -> (Balance, Balance);

	fn get_liquidity_token_address(currency_id_a: CurrencyId, currency_id_b: CurrencyId) -> Option<H160>;

	fn get_swap_amount(path: &[CurrencyId], limit: SwapLimit<Balance>) -> Option<(Balance, Balance)>;

	fn get_best_price_swap_path(
		supply_currency_id: CurrencyId,
		target_currency_id: CurrencyId,
		limit: SwapLimit<Balance>,
		alternative_path_joint_list: Vec<Vec<CurrencyId>>,
	) -> Option<Vec<CurrencyId>>;

	fn swap_with_specific_path(
		who: &AccountId,
		path: &[CurrencyId],
		limit: SwapLimit<Balance>,
	) -> sp_std::result::Result<(Balance, Balance), DispatchError>;

	fn buyback_swap_with_specific_path(
		who: &AccountId,
		path: &[CurrencyId],
		limit: SwapLimit<Balance>,
	) -> sp_std::result::Result<(Balance, Balance), DispatchError>;

	fn swap_with_exact_target(
		who: &AccountId,
		path: &[CurrencyId],
		exact_target_amount: Balance,
		max_supply_amount: Balance,
	) -> DispatchResult;

	fn add_liquidity(
		who: &AccountId,
		currency_id_a: CurrencyId,
		currency_id_b: CurrencyId,
		max_amount_a: Balance,
		max_amount_b: Balance,
		min_share_increment: Balance,
	) -> sp_std::result::Result<(Balance, Balance, Balance), DispatchError>;

	fn remove_liquidity(
		who: &AccountId,
		currency_id_a: CurrencyId,
		currency_id_b: CurrencyId,
		remove_share: Balance,
		min_withdrawn_a: Balance,
		min_withdrawn_b: Balance,
	) -> sp_std::result::Result<(Balance, Balance), DispatchError>;
}

impl<AccountId, CurrencyId, Balance> DEXManager<AccountId, CurrencyId, Balance> for ()
where
	Balance: Default,
{
	fn get_liquidity_pool(_currency_id_a: CurrencyId, _currency_id_b: CurrencyId) -> (Balance, Balance) {
		Default::default()
	}

	fn get_liquidity_token_address(_currency_id_a: CurrencyId, _currency_id_b: CurrencyId) -> Option<H160> {
		Some(Default::default())
	}

	fn get_swap_amount(_path: &[CurrencyId], _limit: SwapLimit<Balance>) -> Option<(Balance, Balance)> {
		Some(Default::default())
	}

	fn get_best_price_swap_path(
		_supply_currency_id: CurrencyId,
		_target_currency_id: CurrencyId,
		_limit: SwapLimit<Balance>,
		_alternative_path_joint_list: Vec<Vec<CurrencyId>>,
	) -> Option<Vec<CurrencyId>> {
		Some(Default::default())
	}

	fn swap_with_specific_path(
		_who: &AccountId,
		_path: &[CurrencyId],
		_limit: SwapLimit<Balance>,
	) -> sp_std::result::Result<(Balance, Balance), DispatchError> {
		Ok(Default::default())
	}

	fn buyback_swap_with_specific_path(
		_who: &AccountId,
		_path: &[CurrencyId],
		_limit: SwapLimit<Balance>,
	) -> sp_std::result::Result<(Balance, Balance), DispatchError> {
		Ok(Default::default())
	}

	fn swap_with_exact_target(
		_who: &AccountId,
		_path: &[CurrencyId],
		_exact_target_amount: Balance,
		_max_supply_amount: Balance,
	) -> DispatchResult {
		Ok(Default::default())
	}

	fn add_liquidity(
		_who: &AccountId,
		_currency_id_a: CurrencyId,
		_currency_id_b: CurrencyId,
		_max_amount_a: Balance,
		_max_amount_b: Balance,
		_min_share_increment: Balance,
	) -> sp_std::result::Result<(Balance, Balance, Balance), DispatchError> {
		Ok(Default::default())
	}

	fn remove_liquidity(
		_who: &AccountId,
		_currency_id_a: CurrencyId,
		_currency_id_b: CurrencyId,
		_remove_share: Balance,
		_min_withdrawn_a: Balance,
		_min_withdrawn_b: Balance,
	) -> sp_std::result::Result<(Balance, Balance), DispatchError> {
		Ok(Default::default())
	}
}

/// An abstraction of serp treasury for the SERP (Setheum Elastic Reserve Protocol).
pub trait SerpTreasury<AccountId> {
	type Balance;
	type CurrencyId;

	fn calculate_supply_change(numerator: Self::Balance, denominator: Self::Balance, supply: Self::Balance) -> Self::Balance;

	fn serp_tes_now() -> DispatchResult;

	/// Deliver System StableCurrency Inflation
	fn issue_stablecurrency_inflation() -> DispatchResult;

	/// SerpUp ratio for BuyBack Swaps to burn Dinar or Setter
	fn get_buyback_serpup(amount: Self::Balance, currency_id: Self::CurrencyId) -> DispatchResult;

	/// Add CashDrop to the pool
	fn add_cashdrop_to_pool(currency_id: Self::CurrencyId, amount: Self::Balance) -> DispatchResult;

	/// Issue CashDrop from the pool to the claimant account
	fn issue_cashdrop_from_pool(
		claimant_id: &AccountId,
		currency_id: Self::CurrencyId,
		amount: Self::Balance
	) -> DispatchResult;

	/// SerpUp ratio for SetPay Cashdrops
	fn get_cashdrop_serpup(amount: Self::Balance, currency_id: Self::CurrencyId) -> DispatchResult;

	/// Serplus ratio for BuyBack Swaps to burn Setter
	fn get_buyback_serplus(amount: Self::Balance, currency_id: Self::CurrencyId) -> DispatchResult;

	fn get_cashdrop_serplus(amount: Self::Balance, currency_id: Self::CurrencyId) -> DispatchResult;

	/// issue system surplus(stable currencies) to their destinations according to the serpup_ratio.
	fn on_serplus(currency_id: Self::CurrencyId, amount: Self::Balance) -> DispatchResult;

	/// issue serpup surplus(stable currencies) to their destinations according to the serpup_ratio.
	fn on_serpup(currency_id: Self::CurrencyId, amount: Self::Balance) -> DispatchResult;

	/// buy back and burn surplus(stable currencies) with swap by DEX.
	fn on_serpdown(currency_id: Self::CurrencyId, amount: Self::Balance) -> DispatchResult;

	/// get the minimum supply of a setcurrency - by key
	fn get_minimum_supply(currency_id: Self::CurrencyId) -> Self::Balance;

	/// issue standard to `who`
	fn issue_standard(currency_id: Self::CurrencyId, who: &AccountId, standard: Self::Balance) -> DispatchResult;

	/// burn standard(stable currency) of `who`
	fn burn_standard(currency_id: Self::CurrencyId, who: &AccountId, standard: Self::Balance) -> DispatchResult;

	/// issue setter of amount setter to `who`
	fn issue_setter(who: &AccountId, setter: Self::Balance) -> DispatchResult;

	/// burn setter of `who`
	fn burn_setter(who: &AccountId, setter: Self::Balance) -> DispatchResult;

	/// deposit reserve asset (Setter (SETR)) to serp treasury by `who`
	fn deposit_setter(from: &AccountId, amount: Self::Balance) -> DispatchResult;

	/// claim cashdrop of `currency_id` relative to `transfer_amount` for `who`
	fn claim_cashdrop(currency_id: Self::CurrencyId, who: &AccountId, transfer_amount: Self::Balance) -> DispatchResult;
}

pub trait SerpTreasuryExtended<AccountId>: SerpTreasury<AccountId> {
	/// When SetCurrency needs SerpDown
	fn buyback_swap_with_exact_supply(
		from_currency_id: Self::CurrencyId,
		to_currency_id: Self::CurrencyId,
		swap_limit: SwapLimit<Self::Balance>,
	) -> sp_std::result::Result<(Self::Balance, Self::Balance), DispatchError>;

	/// When SetCurrency needs SerpDown
	fn buyback_swap_with_exact_target(
		from_currency_id: Self::CurrencyId,
		to_currency_id: Self::CurrencyId,
		swap_limit: SwapLimit<Self::Balance>,
	) -> sp_std::result::Result<(Self::Balance, Self::Balance), DispatchError>;
}

/// An abstraction of cdp treasury for Setmint Protocol.
pub trait CDPTreasury<AccountId> {
	type Balance;
	type CurrencyId;

	/// get surplus amount of cdp treasury
	fn get_surplus_pool() -> Self::Balance;

	/// get debit amount of cdp treasury
	fn get_debit_pool() -> Self::Balance;

	/// get collateral assets amount of cdp treasury
	fn get_total_collaterals(id: Self::CurrencyId) -> Self::Balance;

	/// calculate the proportion of specific debit amount for the whole system
	fn get_debit_proportion(amount: Self::Balance) -> Ratio;

	/// issue debit for cdp treasury
	fn on_system_debit(amount: Self::Balance) -> DispatchResult;

	/// issue surplus(stable currency) for cdp treasury
	fn on_system_surplus(amount: Self::Balance) -> DispatchResult;

	/// issue debit to `who`
	/// if backed flag is true, means the debit to issue is backed on some
	/// assets, otherwise will increase same amount of debit to system debit.
	fn issue_debit(who: &AccountId, debit: Self::Balance, backed: bool) -> DispatchResult;

	/// burn debit(stable currency) of `who`
	fn burn_debit(who: &AccountId, debit: Self::Balance) -> DispatchResult;

	/// deposit surplus(stable currency) to cdp treasury by `from`
	fn deposit_surplus(from: &AccountId, surplus: Self::Balance) -> DispatchResult;

	/// deposit collateral assets to cdp treasury by `who`
	fn deposit_collateral(from: &AccountId, currency_id: Self::CurrencyId, amount: Self::Balance) -> DispatchResult;

	/// withdraw collateral assets of cdp treasury to `who`
	fn withdraw_collateral(to: &AccountId, currency_id: Self::CurrencyId, amount: Self::Balance) -> DispatchResult;
}

pub trait CDPTreasuryExtended<AccountId>: CDPTreasury<AccountId> {
	fn swap_collateral_to_stable(
		currency_id: Self::CurrencyId,
		limit: SwapLimit<Self::Balance>,
		collateral_in_auction: bool,
	) -> sp_std::result::Result<(Self::Balance, Self::Balance), DispatchError>;

	fn create_collateral_auctions(
		currency_id: Self::CurrencyId,
		amount: Self::Balance,
		target: Self::Balance,
		refund_receiver: AccountId,
		splited: bool,
	) -> DispatchResult;

	fn remove_liquidity_for_lp_collateral(
		currency_id: Self::CurrencyId,
		amount: Self::Balance,
	) -> sp_std::result::Result<(Self::Balance, Self::Balance), DispatchError>;

	fn max_auction() -> u32;
}

pub trait PriceProvider<CurrencyId> {
	fn get_price(currency_id: CurrencyId) -> Option<Price>;
	fn get_relative_price(base: CurrencyId, quote: CurrencyId) -> Option<Price> {
		if let (Some(base_price), Some(quote_price)) = (Self::get_price(base), Self::get_price(quote)) {
			base_price.checked_div(&quote_price)
		} else {
			None
		}
	}
}

pub trait DEXPriceProvider<CurrencyId> {
	fn get_relative_price(base: CurrencyId, quote: CurrencyId) -> Option<ExchangeRate>;
}

pub trait LockablePrice<CurrencyId> {
	fn lock_price(currency_id: CurrencyId) -> DispatchResult;
	fn unlock_price(currency_id: CurrencyId) -> DispatchResult;
}

pub trait ExchangeRateProvider {
	fn get_exchange_rate() -> ExchangeRate;
}

/// Dispatchable tasks
pub trait DispatchableTask {
	fn dispatch(self, weight: Weight) -> TaskResult;
}

/// Idle scheduler trait
pub trait IdleScheduler<Task> {
	fn schedule(task: Task) -> DispatchResult;
}

#[cfg(feature = "std")]
impl DispatchableTask for () {
	fn dispatch(self, _weight: Weight) -> TaskResult {
		unimplemented!()
	}
}

#[cfg(feature = "std")]
impl<Task> IdleScheduler<Task> for () {
	fn schedule(_task: Task) -> DispatchResult {
		unimplemented!()
	}
}

pub trait EmergencyShutdown {
	fn is_shutdown() -> bool;
}

/// Return true if the call of EVM precompile contract is allowed.
pub trait PrecompileCallerFilter {
	fn is_allowed(caller: H160) -> bool;
}

/// An abstraction of EVM for EVMBridge
pub trait EVM<AccountId> {
	type Balance: AtLeast32BitUnsigned + Copy + MaybeSerializeDeserialize + Default;

	fn execute(
		context: InvokeContext,
		input: Vec<u8>,
		value: Self::Balance,
		gas_limit: u64,
		storage_limit: u32,
		mode: ExecutionMode,
	) -> Result<CallInfo, sp_runtime::DispatchError>;

	/// Get the real origin account and charge storage rent from the origin.
	fn get_origin() -> Option<AccountId>;
	/// Provide a method to set origin for `on_initialize`
	fn set_origin(origin: AccountId);
}

#[derive(Encode, Decode, Eq, PartialEq, Copy, Clone, RuntimeDebug)]
pub enum ExecutionMode {
	Execute,
	/// Discard any state changes
	View,
	/// Also discard any state changes and use estimate gas mode for evm config
	EstimateGas,
}

#[derive(Encode, Decode, Eq, PartialEq, Copy, Clone, RuntimeDebug)]
pub struct InvokeContext {
	pub contract: EvmAddress,
	/// similar to msg.sender
	pub sender: EvmAddress,
	/// similar to tx.origin
	pub origin: EvmAddress,
}

/// An abstraction of EVMBridge
pub trait EVMBridge<AccountId, Balance> {
	/// Execute ERC20.name() to read token name from ERC20 contract
	fn name(context: InvokeContext) -> Result<Vec<u8>, DispatchError>;
	/// Execute ERC20.symbol() to read token symbol from ERC20 contract
	fn symbol(context: InvokeContext) -> Result<Vec<u8>, DispatchError>;
	/// Execute ERC20.decimals() to read token decimals from ERC20 contract
	fn decimals(context: InvokeContext) -> Result<u8, DispatchError>;
	/// Execute ERC20.totalSupply() to read total supply from ERC20 contract
	fn total_supply(context: InvokeContext) -> Result<Balance, DispatchError>;
	/// Execute ERC20.balanceOf(address) to read balance of address from ERC20
	/// contract
	fn balance_of(context: InvokeContext, address: EvmAddress) -> Result<Balance, DispatchError>;
	/// Execute ERC20.transfer(address, uint256) to transfer value to `to`
	fn transfer(context: InvokeContext, to: EvmAddress, value: Balance) -> DispatchResult;
	/// Get the real origin account and charge storage rent from the origin.
	fn get_origin() -> Option<AccountId>;
	/// Provide a method to set origin for `on_initialize`
	fn set_origin(origin: AccountId);
}

#[cfg(feature = "std")]
impl<AccountId, Balance: Default> EVMBridge<AccountId, Balance> for () {
	fn name(_context: InvokeContext) -> Result<Vec<u8>, DispatchError> {
		Err(DispatchError::Other("unimplemented evm bridge"))
	}
	fn symbol(_context: InvokeContext) -> Result<Vec<u8>, DispatchError> {
		Err(DispatchError::Other("unimplemented evm bridge"))
	}
	fn decimals(_context: InvokeContext) -> Result<u8, DispatchError> {
		Err(DispatchError::Other("unimplemented evm bridge"))
	}
	fn total_supply(_context: InvokeContext) -> Result<Balance, DispatchError> {
		Err(DispatchError::Other("unimplemented evm bridge"))
	}
	fn balance_of(_context: InvokeContext, _address: EvmAddress) -> Result<Balance, DispatchError> {
		Err(DispatchError::Other("unimplemented evm bridge"))
	}
	fn transfer(_context: InvokeContext, _to: EvmAddress, _value: Balance) -> DispatchResult {
		Err(DispatchError::Other("unimplemented evm bridge"))
	}
	fn get_origin() -> Option<AccountId> {
		None
	}
	fn set_origin(_origin: AccountId) {}
}

/// An abstraction of EVMStateRentTrait
pub trait EVMStateRentTrait<AccountId, Balance> {
	/// Query the constants `NewContractExtraBytes` value from evm module.
	fn query_new_contract_extra_bytes() -> u32;
	/// Query the constants `StorageDepositPerByte` value from evm module.
	fn query_storage_deposit_per_byte() -> Balance;
	/// Query the maintainer address from the ERC20 contract.
	fn query_maintainer(contract: H160) -> Result<H160, DispatchError>;
	/// Query the constants `DeveloperDeposit` value from evm module.
	fn query_developer_deposit() -> Balance;
	/// Query the constants `DeploymentFee` value from evm module.
	fn query_deployment_fee() -> Balance;
	/// Transfer the maintainer of the contract address.
	fn transfer_maintainer(from: AccountId, contract: H160, new_maintainer: H160) -> DispatchResult;
}

pub trait TransactionPayment<AccountId, Balance, NegativeImbalance> {
	fn reserve_fee(who: &AccountId, weight: Weight) -> Result<Balance, DispatchError>;
	fn unreserve_fee(who: &AccountId, fee: Balance);
	fn unreserve_and_charge_fee(
		who: &AccountId,
		weight: Weight,
	) -> Result<(Balance, NegativeImbalance), TransactionValidityError>;
	fn refund_fee(who: &AccountId, weight: Weight, payed: NegativeImbalance) -> Result<(), TransactionValidityError>;
	fn charge_fee(
		who: &AccountId,
		len: u32,
		weight: Weight,
		tip: Balance,
		pays_fee: Pays,
		class: DispatchClass,
	) -> Result<(), TransactionValidityError>;
}

#[cfg(feature = "std")]
use frame_support::traits::Imbalance;
#[cfg(feature = "std")]
impl<AccountId, Balance: Default + Copy, NegativeImbalance: Imbalance<Balance>>
	TransactionPayment<AccountId, Balance, NegativeImbalance> for ()
{
	fn reserve_fee(_who: &AccountId, _weight: Weight) -> Result<Balance, DispatchError> {
		Ok(Default::default())
	}

	fn unreserve_fee(_who: &AccountId, _fee: Balance) {}

	fn unreserve_and_charge_fee(
		_who: &AccountId,
		_weight: Weight,
	) -> Result<(Balance, NegativeImbalance), TransactionValidityError> {
		Ok((Default::default(), Imbalance::zero()))
	}

	fn refund_fee(
		_who: &AccountId,
		_weight: Weight,
		_payed: NegativeImbalance,
	) -> Result<(), TransactionValidityError> {
		Ok(())
	}

	fn charge_fee(
		_who: &AccountId,
		_len: u32,
		_weight: Weight,
		_tip: Balance,
		_pays_fee: Pays,
		_class: DispatchClass,
	) -> Result<(), TransactionValidityError> {
		Ok(())
	}
}

pub trait Contains<T> {
	fn contains(t: &T) -> bool;
}

/// A mapping between `AccountId` and `EvmAddress`.
pub trait AddressMapping<AccountId> {
	/// Returns the AccountId used go generate the given EvmAddress.
	fn get_account_id(evm: &EvmAddress) -> AccountId;
	/// Returns the EvmAddress associated with a given AccountId or the
	/// underlying EvmAddress of the AccountId.
	/// Returns None if there is no EvmAddress associated with the AccountId
	/// and there is no underlying EvmAddress in the AccountId.
	fn get_evm_address(account_id: &AccountId) -> Option<EvmAddress>;
	/// Returns the EVM address associated with an account ID and generates an
	/// account mapping if no association exists.
	fn get_or_create_evm_address(account_id: &AccountId) -> EvmAddress;
	/// Returns the default EVM address associated with an account ID.
	fn get_default_evm_address(account_id: &AccountId) -> EvmAddress;
	/// Returns true if a given AccountId is associated with a given EvmAddress
	/// and false if is not.
	fn is_linked(account_id: &AccountId, evm: &EvmAddress) -> bool;
}

/// A mapping between u32 and Erc20 address.
/// provide a way to encode/decode for CurrencyId;
pub trait CurrencyIdMapping {
	/// Use first 4 non-zero bytes as u32 to the mapping between u32 and evm
	/// address.
	fn set_erc20_mapping(address: EvmAddress) -> DispatchResult;
	/// Returns the EvmAddress associated with a given u32.
	fn get_evm_address(currency_id: u32) -> Option<EvmAddress>;
	/// Returns the name associated with a given CurrencyId.
	/// If CurrencyId is CurrencyId::DexShare and contain DexShare::Erc20,
	/// the EvmAddress must have been mapped.
	fn name(currency_id: CurrencyId) -> Option<Vec<u8>>;
	/// Returns the symbol associated with a given CurrencyId.
	/// If CurrencyId is CurrencyId::DexShare and contain DexShare::Erc20,
	/// the EvmAddress must have been mapped.
	fn symbol(currency_id: CurrencyId) -> Option<Vec<u8>>;
	/// Returns the decimals associated with a given CurrencyId.
	/// If CurrencyId is CurrencyId::DexShare and contain DexShare::Erc20,
	/// the EvmAddress must have been mapped.
	fn decimals(currency_id: CurrencyId) -> Option<u8>;
	/// Encode the CurrencyId to EvmAddress.
	/// If is CurrencyId::DexShare and contain DexShare::Erc20,
	/// will use the u32 to get the DexShare::Erc20 from the mapping.
	fn encode_evm_address(v: CurrencyId) -> Option<EvmAddress>;
	/// Decode the CurrencyId from EvmAddress.
	/// If is CurrencyId::DexShare and contain DexShare::Erc20,
	/// will use the u32 to get the DexShare::Erc20 from the mapping.
	fn decode_evm_address(v: EvmAddress) -> Option<CurrencyId>;
}

#[cfg(feature = "std")]
impl CurrencyIdMapping for () {
	fn set_erc20_mapping(_address: EvmAddress) -> DispatchResult {
		Err(DispatchError::Other("unimplemented CurrencyIdMapping"))
	}

	fn get_evm_address(_currency_id: u32) -> Option<EvmAddress> {
		None
	}

	fn name(_currency_id: CurrencyId) -> Option<Vec<u8>> {
		None
	}

	fn symbol(_currency_id: CurrencyId) -> Option<Vec<u8>> {
		None
	}

	fn decimals(_currency_id: CurrencyId) -> Option<u8> {
		None
	}

	fn encode_evm_address(_v: CurrencyId) -> Option<EvmAddress> {
		None
	}

	fn decode_evm_address(_v: EvmAddress) -> Option<CurrencyId> {
		None
	}
}
