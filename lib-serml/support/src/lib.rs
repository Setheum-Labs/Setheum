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

use codec::{Decode, Encode};
use frame_support::pallet_prelude::{DispatchClass, Pays, Weight};
use primitives::{
	evm::{CallInfo, EvmAddress},
	CurrencyId,
};
use sp_core::H160;
use sp_runtime::{
	traits::{AtLeast32BitUnsigned, MaybeSerializeDeserialize},
	transaction_validity::TransactionValidityError,
	DispatchError, DispatchResult, FixedU128, RuntimeDebug,
};
use sp_std::{
	cmp::{Eq, PartialEq},
	prelude::*,
};

pub mod mocks;

pub type BlockNumber = u32;
pub type CashDropRate = FixedU128;
pub type CashDropClaim = bool;
pub type ExchangeRate = FixedU128;
pub type Price = FixedU128;
pub type Rate = FixedU128;
pub type Ratio = FixedU128;

pub trait StandardValidator<AccountId, CurrencyId, Balance, StandardBalance> {
	fn check_position_valid(
		currency_id: CurrencyId,
		reserve_balance: Balance,
		standard_balance: StandardBalance,
	) -> DispatchResult;
}

impl<AccountId, CurrencyId, Balance: Default, StandardBalance> StandardValidator<AccountId, CurrencyId, Balance, StandardBalance>
	for ()
{
	fn check_position_valid(
		_currency_id: CurrencyId,
		_reserve_balance: Balance,
		_standard_balance: StandardBalance,
	) -> DispatchResult {
		Ok(())
	}
}

pub trait DEXManager<AccountId, CurrencyId, Balance> {
	fn get_liquidity_pool(currency_id_a: CurrencyId, currency_id_b: CurrencyId) -> (Balance, Balance);

	fn get_liquidity_token_address(currency_id_a: CurrencyId, currency_id_b: CurrencyId) -> Option<H160>;

	fn get_swap_target_amount(
		path: &[CurrencyId],
		supply_amount: Balance,
		price_impact_limit: Option<Ratio>,
	) -> Option<Balance>;

	fn get_swap_supply_amount(
		path: &[CurrencyId],
		target_amount: Balance,
		price_impact_limit: Option<Ratio>,
	) -> Option<Balance>;

	fn swap_with_exact_supply(
		who: &AccountId,
		path: &[CurrencyId],
		supply_amount: Balance,
		min_target_amount: Balance,
		price_impact_limit: Option<Ratio>,
	) -> sp_std::result::Result<Balance, DispatchError>;

	fn swap_with_exact_target(
		who: &AccountId,
		path: &[CurrencyId],
		target_amount: Balance,
		max_supply_amount: Balance,
		price_impact_limit: Option<Ratio>,
	) -> sp_std::result::Result<Balance, DispatchError>;

	fn add_liquidity(
		who: &AccountId,
		currency_id_a: CurrencyId,
		currency_id_b: CurrencyId,
		max_amount_a: Balance,
		max_amount_b: Balance,
		min_share_increment: Balance,
		stake_increment_share: bool,
	) -> DispatchResult;

	fn remove_liquidity(
		who: &AccountId,
		currency_id_a: CurrencyId,
		currency_id_b: CurrencyId,
		remove_share: Balance,
		min_withdrawn_a: Balance,
		min_withdrawn_b: Balance,
		by_unstake: bool,
	) -> DispatchResult;
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

	fn get_swap_target_amount(
		_path: &[CurrencyId],
		_supply_amount: Balance,
		_price_impact_limit: Option<Ratio>,
	) -> Option<Balance> {
		Some(Default::default())
	}

	fn get_swap_supply_amount(
		_path: &[CurrencyId],
		_target_amount: Balance,
		_price_impact_limit: Option<Ratio>,
	) -> Option<Balance> {
		Some(Default::default())
	}

	fn swap_with_exact_supply(
		_who: &AccountId,
		_path: &[CurrencyId],
		_supply_amount: Balance,
		_min_target_amount: Balance,
		_price_impact_limit: Option<Ratio>,
	) -> sp_std::result::Result<Balance, DispatchError> {
		Ok(Default::default())
	}

	fn swap_with_exact_target(
		_who: &AccountId,
		_path: &[CurrencyId],
		_target_amount: Balance,
		_max_supply_amount: Balance,
		_price_impact_limit: Option<Ratio>,
	) -> sp_std::result::Result<Balance, DispatchError> {
		Ok(Default::default())
	}

	fn add_liquidity(
		_who: &AccountId,
		_currency_id_a: CurrencyId,
		_currency_id_b: CurrencyId,
		_max_amount_a: Balance,
		_max_amount_b: Balance,
		_min_share_increment: Balance,
		_stake_increment_share: bool,
	) -> DispatchResult {
		Ok(())
	}

	fn remove_liquidity(
		_who: &AccountId,
		_currency_id_a: CurrencyId,
		_currency_id_b: CurrencyId,
		_remove_share: Balance,
		_min_withdrawn_a: Balance,
		_min_withdrawn_b: Balance,
		_by_unstake: bool,
	) -> DispatchResult {
		Ok(())
	}
}

/// An abstraction of serp treasury for the SERP (Setheum Elastic Reserve Protocol).
pub trait SerpTreasury<AccountId> {
	type Amount;
	type Balance;
	type CurrencyId;
	type BlockNumber;

	fn get_adjustment_frequency() -> Self::BlockNumber;

	/// get reserve asset amount of serp treasury
	fn get_total_setter() -> Self::Balance;

	/// calculate the proportion of specific currency amount for the whole system
	fn get_propper_proportion(amount: Self::Balance, currency_id: Self::CurrencyId) -> Ratio;

	/// SerpUp ratio for Serplus Auctions / Swaps
	fn get_serplus_serpup(amount: Self::Balance, currency_id: Self::CurrencyId) -> DispatchResult;

	/// SerpUp ratio for SettPay Cashdrops
	fn get_settpay_serpup(amount: Self::Balance, currency_id: Self::CurrencyId) -> DispatchResult;

	/// SerpUp ratio for Setheum Treasury
	fn get_treasury_serpup(amount: Self::Balance, currency_id: Self::CurrencyId) -> DispatchResult;

	/// SerpUp ratio for Setheum Foundation's Charity Fund
	fn get_charity_fund_serpup(amount: Self::Balance, currency_id: Self::CurrencyId) -> DispatchResult;

	/// issue serpup surplus(stable currencies) to their destinations according to the serpup_ratio.
	fn on_surpup(currency_id: Self::CurrencyId, amount: Self::Balance) -> DispatchResult;

	/// buy back and burn surplus(stable currencies) with auction.
	fn on_serpdown(currency_id: Self::CurrencyId, amount: Self::Balance) -> DispatchResult;

	/// get the minimum supply of a settcurrency - by key
	fn get_minimum_supply(currency_id: Self::CurrencyId) -> Self::Balance;

	/// buy back and burn surplus(stable currencies) with auction
	/// Create the necessary serp down parameters and starts new auction.
	fn on_surpdown(currency_id: Self::CurrencyId, amount: Self::Balance) -> DispatchResult;

	/// Determines whether to SerpUp or SerpDown based on price swing (+/-)).
	/// positive means "Serp Up", negative means "Serp Down".
	/// Then it calls the necessary option to serp the currency supply (up/down).
	fn serp_tes(currency_id: Self::CurrencyId) -> DispatchResult;

	/// Trigger SERP-TES for all stablecoins
	/// Check all stablecoins stability and elasticity
	/// and calls the serp to stabilise the unstable one(s)
	/// on SERP-TES.
	fn on_serp_tes() -> DispatchResult;

	/// issue standard to `who`
	fn issue_standard(currency_id: Self::CurrencyId, who: &AccountId, standard: Self::Balance) -> DispatchResult;

	/// burn standard(stable currency) of `who`
	fn burn_standard(currency_id: Self::CurrencyId, who: &AccountId, standard: Self::Balance) -> DispatchResult;

	/// issue propper(stable currency) of amount propper to `who`
	fn issue_propper(currency_id: Self::CurrencyId, who: &AccountId, propper: Self::Balance) -> DispatchResult;

	/// burn propper(stable currency) of `who`
	fn burn_propper(currency_id: Self::CurrencyId, who: &AccountId, propper: Self::Balance) -> DispatchResult;

	/// issue setter of amount setter to `who`
	fn issue_setter(who: &AccountId, setter: Self::Balance) -> DispatchResult;

	/// burn setter of `who`
	fn burn_setter(who: &AccountId, setter: Self::Balance) -> DispatchResult;

	/// Get the Maximum supply of the Dexer (`DRAM` in Setheum).
	fn get_dexer_max_supply() -> Self::Balance;
	
	/// Issue Dexer (`DRAM` in Setheum). `dexer` here just referring to the DEX token balance.
	fn issue_dexer(who: &AccountId, dexer: Self::Balance) -> DispatchResult;

	/// Burn Dexer (`DRAM` in Setheum). `dexer` here just referring to the DEX token balance.
	fn burn_dexer(who: &AccountId, dexer: Self::Balance) -> DispatchResult;

	/// deposit surplus(propperstable currency) to serp treasury by `from`
	fn deposit_surplus(currency_id: Self::CurrencyId, from: &AccountId, surplus: Self::Balance) -> DispatchResult;

	/// deposit reserve asset (Setter (SETT)) to serp treasury by `who`
	fn deposit_setter(from: &AccountId, amount: Self::Balance) -> DispatchResult;
}

/// An abstraction of settpay for the SERP (Setheum Elastic Reserve Protocol) for CashDrop.
pub trait CashDrop<AccountId> {
	type Balance;
	type CurrencyId;

	/// claim cashdrop of `currency_id` relative to `transfer_amount` for `who`
	fn claim_cashdrop(currency_id: Self::CurrencyId, who: &AccountId, transfer_amount: Self::Balance) -> DispatchResult;

	/// deposit cashdrop of `SETT` of `cashdrop_amount` to `who`
	fn deposit_setter_drop(who: &AccountId, cashdrop_amount: Self::Balance) -> DispatchResult;

	/// deposit cashdrop of `currency_id` relative to `cashdrop_amount` for `who`
	fn deposit_settcurrency_drop(currency_id: Self::CurrencyId, who: &AccountId, cashdrop_amount: Self::Balance) -> DispatchResult;
}

pub trait SerpTreasuryExtended<AccountId>: SerpTreasury<AccountId> {
	fn swap_exact_setter_in_auction_to_settcurrency(
		currency_id: Self::CurrencyId,
		supply_amount: Self::Balance,
		min_target_amount: Self::Balance,
		price_impact_limit: Option<Ratio>,
	) -> sp_std::result::Result<Self::Balance, DispatchError>;

	fn swap_setter_not_in_auction_with_exact_settcurrency(
		currency_id: Self::CurrencyId,
		target_amount: Self::Balance,
		max_supply_amount: Self::Balance,
		price_impact_limit: Option<Ratio>,
	) -> sp_std::result::Result<Self::Balance, DispatchError>;

	fn create_reserve_auctions(
		currency_id: Self::CurrencyId,
		amount: Self::Balance,
		target: Self::Balance,
		refund_receiver: AccountId,
		splited: bool,
	) -> DispatchResult;
}

pub trait PriceProvider<CurrencyId> {
	fn get_peg_currency_by_currency_id(currency_id: CurrencyId) -> CurrencyId;
	fn get_peg_price(currency_id: CurrencyId) -> Option<Price>;
	fn get_fiat_price(fiat_currency_id: CurrencyId) -> Option<Price>;
	fn get_fiat_usd_fixed_price() -> Option<Price>;
	fn get_settusd_fixed_price() -> Option<Price>;
	fn get_stablecoin_fixed_price(currency_id: CurrencyId) -> Option<Price>;
	fn get_stablecoin_market_price(currency_id: CurrencyId) -> Option<Price>;
	fn get_relative_price(base: CurrencyId, quote: CurrencyId) -> Option<Price>;
	fn get_market_relative_price(base: CurrencyId, quote: CurrencyId) -> Option<Price>;
	fn get_coin_to_peg_relative_price(currency_id: CurrencyId) -> Option<Price>;
	fn get_setter_basket_peg_price() -> Option<Price>;
	fn get_setter_fixed_price() -> Option<Price>;
	fn get_market_price(currency_id: CurrencyId) -> Option<Price>;
	fn get_price(currency_id: CurrencyId) -> Option<Price>;
	fn lock_price(currency_id: CurrencyId);
	fn unlock_price(currency_id: CurrencyId);
}

pub trait ExchangeRateProvider {
	fn get_exchange_rate() -> ExchangeRate;
}

pub trait DEXIncentives<AccountId, CurrencyId, Balance> {
	fn dex_premium_rewards(lp_currency_id: CurrencyId) -> Option<Balance>;
	fn do_deposit_dex_share(who: &AccountId, lp_currency_id: CurrencyId, amount: Balance) -> DispatchResult;
	fn do_withdraw_dex_share(who: &AccountId, lp_currency_id: CurrencyId, amount: Balance) -> DispatchResult;
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
