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

#![cfg_attr(not(feature = "std"), no_std)]
#![allow(clippy::upper_case_acronyms)]
#![allow(clippy::from_over_into)]
#![allow(clippy::type_complexity)]

use frame_support::pallet_prelude::{DispatchClass, Pays, Weight};
use primitives::{task::TaskResult, Balance, CurrencyId, Fees, Multiplier, Nonce, ReserveIdentifier};
use sp_runtime::{
	traits::CheckedDiv, transaction_validity::TransactionValidityError, DispatchError, DispatchResult, FixedU128,
};
use sp_std::{prelude::*, result::Result};
use xcm::prelude::*;

pub mod bounded;
pub mod ecdp;
pub mod edfis_launchpad;
pub mod edfis_mining;
pub mod edfis_swap;
pub mod edfis_swap_legacy;
pub mod evm;
pub mod liquid_staking;
pub mod mocks;

pub use crate::bounded::*;
pub use crate::ecdp::*;
pub use crate::edfis_launchpad::*;
pub use crate::edfis_mining::*;
pub use crate::edfis_swap::*;
pub use crate::edfis_swap_legacy::*;
pub use crate::evm::*;
pub use crate::liquid_staking::*;

pub type Price = FixedU128;
pub type ExchangeRate = FixedU128;
pub type Ratio = FixedU128;
pub type Rate = FixedU128;

/// Implement this StoredMap to replace https://github.com/paritytech/substrate/blob/569aae5341ea0c1d10426fa1ec13a36c0b64393b/frame/system/src/lib.rs#L1679
/// NOTE: If use module-evm, need regards existed `frame_system::Account` also exists
/// `pallet_balances::Account`, even if it's AccountData is default. (This kind of account is
/// usually created by inc_provider), so that `repatriate_reserved` can transfer reserved balance to
/// contract account, which is created by `inc_provider`.
pub struct SystemAccountStore<T>(sp_std::marker::PhantomData<T>);
impl<T: frame_system::Config> frame_support::traits::StoredMap<T::AccountId, T::AccountData> for SystemAccountStore<T> {
	fn get(k: &T::AccountId) -> T::AccountData {
		frame_system::Account::<T>::get(k).data
	}

	fn try_mutate_exists<R, E: From<DispatchError>>(
		k: &T::AccountId,
		f: impl FnOnce(&mut Option<T::AccountData>) -> Result<R, E>,
	) -> Result<R, E> {
		let account = frame_system::Account::<T>::get(k);
		let is_default = account.data == T::AccountData::default();

		// if System Account exists, act its Balances Account also exists.
		let mut some_data = if is_default && !frame_system::Pallet::<T>::account_exists(k) {
			None
		} else {
			Some(account.data)
		};

		let result = f(&mut some_data)?;
		if frame_system::Pallet::<T>::providers(k) > 0 || frame_system::Pallet::<T>::sufficients(k) > 0 {
			frame_system::Account::<T>::mutate(k, |a| a.data = some_data.unwrap_or_default());
		} else {
			frame_system::Account::<T>::remove(k)
		}
		Ok(result)
	}
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

pub trait SwapPriceProvider<CurrencyId> {
	fn get_relative_price(base: CurrencyId, quote: CurrencyId) -> Option<ExchangeRate>;
}

pub trait LockablePrice<CurrencyId> {
	fn lock_price(currency_id: CurrencyId) -> DispatchResult;
	fn unlock_price(currency_id: CurrencyId) -> DispatchResult;
}

pub trait ExchangeRateProvider {
	fn get_exchange_rate() -> ExchangeRate;
}

pub trait TransactionPayment<AccountId, Balance, NegativeImbalance> {
	fn reserve_fee(who: &AccountId, fee: Balance, named: Option<ReserveIdentifier>) -> Result<Balance, DispatchError>;
	fn unreserve_fee(who: &AccountId, fee: Balance, named: Option<ReserveIdentifier>) -> Balance;
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
	fn weight_to_fee(weight: Weight) -> Balance;
	fn apply_multiplier_to_fee(fee: Balance, multiplier: Option<Multiplier>) -> Balance;
}

/// Dispatchable tasks
pub trait DispatchableTask {
	fn dispatch(self, weight: Weight) -> TaskResult;
}

#[cfg(feature = "std")]
impl DispatchableTask for () {
	fn dispatch(self, _weight: Weight) -> TaskResult {
		unimplemented!()
	}
}

#[impl_trait_for_tuples::impl_for_tuples(30)]
pub trait OnNewEra<EraIndex> {
	fn on_new_era(era: EraIndex);
}

pub trait NomineesProvider<AccountId> {
	fn nominees() -> Vec<AccountId>;
}

pub trait LiquidateCollateral<AccountId> {
	fn liquidate(
		who: &AccountId,
		currency_id: CurrencyId,
		amount: Balance,
		target_ussd_amount: Balance,
	) -> DispatchResult;
}

#[impl_trait_for_tuples::impl_for_tuples(30)]
impl<AccountId> LiquidateCollateral<AccountId> for Tuple {
	fn liquidate(
		who: &AccountId,
		currency_id: CurrencyId,
		amount: Balance,
		target_ussd_amount: Balance,
	) -> DispatchResult {
		let mut last_error = None;
		for_tuples!( #(
			match Tuple::liquidate(who, currency_id, amount, target_ussd_amount) {
				Ok(_) => return Ok(()),
				Err(e) => { last_error = Some(e) }
			}
		)* );
		let last_error = last_error.unwrap_or(DispatchError::Other("No liquidation impl."));
		Err(last_error)
	}
}

pub trait BuyWeightRate {
	fn calculate_rate(location: MultiLocation) -> Option<Ratio>;
}

// TODO:[src/lib.rs:0] - Use this as reference to upgrade the existing implementation of the Launchpad
// /// The Structure of a Campaign info.
// #[cfg_attr(feature = "std", derive(PartialEq, Eq, Encode, Decode, Debug, Clone))]
// pub struct CampaignInfo<AccountId, Balance, BlockNumber> {
// 	/// The Campaign Id
// 	pub id: CampaignId,
// 	/// Campaign Creator
// 	pub origin: AccountId,
// 	/// Project Name
// 	pub project_name: Vec<u8>,
// 	/// Project Logo
// 	pub project_logo: Vec<u8>,
// 	/// Project Description
// 	pub project_description: Vec<u8>,
// 	/// Project Website
// 	pub project_website: Vec<u8>,
// 	/// Campaign Beneficiary
// 	pub beneficiary: AccountId,
// 	/// Campaign Pool AccountId
// 	pub pool: AccountId,
// 	/// Currency type for the fundraise
// 	pub raise_currency: CurrencyId,
// 	/// Currency type (Token) for crowdsale
// 	pub sale_token: CurrencyId,
// 	/// Crowdsale Token Price - Amount of raise_currency per sale_token
// 	pub token_price: Balance,
// 	/// Crowdsale Token amount for sale
// 	pub crowd_allocation: Balance,
// 	/// The Fundraise Goal - HardCap
// 	pub goal: Balance,
// 	/// The Fundraise Amount raised - HardCap
// 	pub raised: Balance,
// 	/// The number of contributors to the campaign
// 	pub contributors_count: u32,
// 	/// The Campaign contributions
// 	/// account_id, contribution, allocation, bool:claimed_allocation
// 	pub contributions: Vec<(AccountId, Balance, Balance, bool)>,
// 	/// The period that the campaign runs for.
// 	pub period: BlockNumber,
// 	/// The time when the campaign starts.
// 	pub campaign_start: BlockNumber,
// 	/// The time when the campaign ends.
// 	pub campaign_end: BlockNumber,
// 	/// The time when the campaign fund retires.
// 	pub campaign_retirement_period: BlockNumber,
// 	/// The time when a rejected proposal is removed from storage.
// 	pub proposal_retirement_period: BlockNumber,
// 	/// Is the campaign approved?
// 	pub is_approved: bool,
// 	/// Is the proposal rejected?
// 	pub is_rejected: bool,
// 	/// Is the campaign in waiting period?
// 	pub is_waiting: bool,
// 	/// Is the campaign active?
// 	pub is_active: bool,
// 	/// Is the campaign Successful?
// 	pub is_successful: bool,
// 	/// Is the campaign Failed?
// 	pub is_failed: bool,
// 	/// Is the campaign Ended?
// 	pub is_ended: bool,
// 	/// Is the campaign funds raised claimed
// 	pub is_claimed: bool,
// }

// /// Abstraction over th Launchpad Proposal system.
// pub trait Proposal<AccountId, BlockNumber> {
// 	/// Get all proposals
// 	fn all_proposals() -> Vec<CampaignInfo<AccountId, AsBalance, BlockNumber>>;
// 	/// The Campaign Proposal info of `id`
// 	fn proposal_info(id: CampaignId) -> Option<CampaignInfo<AccountId, AsBalance, BlockNumber>>;
// 	/// Create new Campaign Proposal with specific `CampaignInfo`, return the `id` of the Campaign
// 	fn new_proposal(
// 		origin: AccountId,
// 		project_name: Vec<u8>,
// 		project_logo: Vec<u8>,
// 		project_description: Vec<u8>,
// 		project_website: Vec<u8>,
// 		beneficiary: AccountId,
// 		raise_currency: CurrencyId,
// 		sale_token: CurrencyId,
// 		token_price: AsBalance,
// 		crowd_allocation: AsBalance,
// 		goal: AsBalance,
// 		period: BlockNumber,
// 	) -> DispatchResult;
//     /// Approve Proposal by `id` at `now`.
//     fn on_approve_proposal(id: CampaignId) -> sp_std::result::Result<(), DispatchError>;
// 	/// Reject Proposal by `id` and update storage
// 	fn on_reject_proposal(id: CampaignId) -> sp_std::result::Result<(), DispatchError>;
// 	/// Remove Proposal by `id` from storage
// 	fn remove_proposal(id: CampaignId) -> sp_std::result::Result<(), DispatchError>;
// }

// /// Abstraction over the Launchpad Campaign system.
// pub trait CampaignManager<AccountId, BlockNumber> {
// 	/// The Campaign info of `id`
// 	fn campaign_info(id: CampaignId) -> Option<CampaignInfo<AccountId, AsBalance, BlockNumber>>;
// 	/// Get all proposals
// 	fn all_campaigns() -> Vec<CampaignInfo<AccountId, AsBalance, BlockNumber>>;
// 	/// Called when a contribution is received.
// 	fn on_contribution(
// 		who: AccountId,
// 		id: CampaignId,
// 		amount: AsBalance,
// 	) -> DispatchResult;
// 	/// Called when a contribution allocation is claimed
// 	fn on_claim_allocation(
// 		who: AccountId,
// 		id: CampaignId,
// 	) -> DispatchResult;
// 	/// Called when a campaign's raised fund is claimed
// 	fn on_claim_campaign(
// 		who: AccountId,
// 		id: CampaignId,
// 	) -> DispatchResult;
// 	/// Called when a failed campaign is claimed by the proposer
// 	fn on_claim_failed_campaign(
// 		who: AccountId,
// 		id: CampaignId,
// 	) -> DispatchResult;
// 	/// Activate a campaign by `id`
// 	fn activate_campaign(id: CampaignId) -> DispatchResult;
// 	/// Ensure campaign is Valid and Successfully Ended
// 	fn ensure_successfully_ended_campaign(id: CampaignId) -> DispatchResult;
// 	/// Record Successful Campaign by `id`
// 	fn on_successful_campaign(now: BlockNumber, id: CampaignId) -> DispatchResult ;
// 	/// Record Failed Campaign by `id`
// 	fn on_failed_campaign(now: BlockNumber, id: CampaignId) -> DispatchResult ;
// 	/// Called when pool is retired
// 	fn on_retire(id: CampaignId)-> DispatchResult;
// 	/// Get amount of contributors in a campaign
// 	fn get_contributors_count(id: CampaignId) -> u32;
// 	/// Get the total amounts raised in protocol
// 	fn get_total_amounts_raised() -> Vec<(CurrencyId, AsBalance)>;
// }
