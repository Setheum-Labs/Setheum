// This file is part of Setheum.

// Copyright (C) 2020-2021 Setheum Labs.
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

use codec::{Decode, Encode, FullCodec, HasCompact};
use frame_support::pallet_prelude::{DispatchClass, Pays, Weight};
use sp_core::H160;
use sp_runtime::{
	traits::{AtLeast32BitUnsigned, MaybeSerializeDeserialize},
	transaction_validity::TransactionValidityError,
	DispatchError, DispatchResult, FixedU128, RuntimeDebug,
};
use sp_std::{
	cmp::{Eq, PartialEq},
	fmt::Debug,
	prelude::*,
};

pub type Price = FixedU128;
pub type FiatCurrencyId: Parameter + Member + Copy + MaybeSerializeDeserialize + Ord;
pub type ExchangeRate = FixedU128;
pub type Ratio = FixedU128;
pub type Rate = FixedU128;

pub trait StandardManager<AccountId, CurrencyId, Balance, Balance> {
	fn check_position_valid(
		currency_id: CurrencyId,
		reserve_balance: Balance,
		standard_balance: Balance,
	) -> DispatchResult;
}

impl<AccountId, CurrencyId, Balance: Default, Balance> StandardManager<AccountId, CurrencyId, Balance, Balance>
	for ()
{
	fn check_position_valid(
		_currency_id: CurrencyId,
		_reserve_balance: Balance,
		_standard_balance: Balance,
	) -> DispatchResult {
		Ok(())
	}
}

pub trait SetheumDexManager<AccountId, CurrencyId, Balance> {
	fn get_liquidity_pool(currency_id_a: CurrencyId, currency_id_b: CurrencyId) -> (Balance, Balance);

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
}

impl<AccountId, CurrencyId, Balance> SetheumDexManager<AccountId, CurrencyId, Balance> for ()
where
	Balance: Default,
{
	fn get_liquidity_pool(_currency_id_a: CurrencyId, _currency_id_b: CurrencyId) -> (Balance, Balance) {
		Default::default()
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
}

/// An abstraction of serp treasury for the SERP (Setheum Elastic Reserve Protocol).
pub trait SerpTreasury<AccountId> {
	type Balance;
	type CurrencyId;

	/// get surplus amount of serp treasury
	fn get_surplus_pool() -> Self::Balance;

	/// get standard amount of serp treasury
	fn get_standard_pool() -> Self::Balance;

	/// get serpup amount of serp treasury
	fn get_surpup_pool() -> Self::Balance;

	/// get standard assets amount of serp treasury
	fn get_total_standard(id: Self::CurrencyId) -> Self::Balance;

	/// get reserve asset amount of serp treasury
	fn get_total_reserve(id: Self::CurrencyId) -> Self::Balance;

	/// calculate the proportion of specific standard amount for the whole system
	fn get_standard_proportion(amount: Self::Balance) -> Ratio;

	/// issue standard for serp treasury
	fn on_system_standard(id: Self::CurrencyId, amount: Self::Balance) -> DispatchResult;

	/// issue surplus(stable currencies) for serp treasury
	fn on_system_surplus(id: Self::CurrencyId, amount: Self::Balance) -> DispatchResult;

	/// issue standard to `who`
	fn issue_standard(who: &AccountId, standard: Self::Balance) -> DispatchResult;

	/// burn standard(stable currency) of `who`
	fn burn_standard(who: &AccountId, standard: Self::Balance) -> DispatchResult;

	/// deposit surplus(propperstable currency) to serp treasury by `from`
	fn deposit_surplus(from: &AccountId, surplus: Self::Balance) -> DispatchResult;

	/// deposit reserve assets to serp treasury by `who`
	fn deposit_reserve(from: &AccountId, currency_id: Self::CurrencyId, amount: Self::Balance) -> DispatchResult;

	/// withdraw reserve assets of serp treasury to `who`
	fn withdraw_reserve(to: &AccountId, currency_id: Self::CurrencyId, amount: Self::Balance) -> DispatchResult;
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
	fn get_fiat_price(fiat_id: FiatCurrencyId, currency_id: CurrencyId) -> Option<Price>;
	fn get_coin_to_peg_relative_price(currency_id: CurrencyId) -> Option<Price>;
	fn get_relative_price(base: CurrencyId, quote: CurrencyId) -> Option<Price>;
	fn get_peg_price(currency_id: CurrencyId) -> Option<Price>;
	fn get_price(currency_id: CurrencyId) -> Option<Price>;
	fn lock_price(currency_id: CurrencyId);
	fn unlock_price(currency_id: CurrencyId);
}

pub trait ExchangeRateProvider {
	fn get_exchange_rate() -> ExchangeRate;
}

pub trait DexIncentives<AccountId, CurrencyId, Balance> {
	fn do_deposit_dex_share(who: &AccountId, lp_currency_id: CurrencyId, amount: Balance) -> DispatchResult;
	fn do_withdraw_dex_share(who: &AccountId, lp_currency_id: CurrencyId, amount: Balance) -> DispatchResult;
}

impl<AccountId, CurrencyId, Balance> DexIncentives<AccountId, CurrencyId, Balance> for () {
	fn do_deposit_dex_share(_: &AccountId, _: CurrencyId, _: Balance) -> DispatchResult {
		Ok(())
	}

	fn do_withdraw_dex_share(_: &AccountId, _: CurrencyId, _: Balance) -> DispatchResult {
		Ok(())
	}
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
