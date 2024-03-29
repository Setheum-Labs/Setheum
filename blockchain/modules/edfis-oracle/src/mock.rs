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

//! Mocks for the dex-oracle module.

#![cfg(test)]

use super::*;
use frame_support::{construct_runtime, derive_impl, ord_parameter_types, parameter_types, traits::ConstU64};
use frame_system::EnsureSignedBy;
use module_support::SwapLimit;
use primitives::{DexShare, Moment, TokenSymbol};
use sp_core::H160;
use sp_runtime::{
	traits::{IdentityLookup, Zero},
	BuildStorage, DispatchError,
};
use sp_std::cell::RefCell;

pub type AccountId = u128;

pub const SEE: CurrencyId = CurrencyId::Token(TokenSymbol::SEE);
pub const USSD: CurrencyId = CurrencyId::Token(TokenSymbol::USSD);
pub const EDF: CurrencyId = CurrencyId::Token(TokenSymbol::EDF);
pub const LP_USSD_EDF: CurrencyId =
	CurrencyId::DexShare(DexShare::Token(TokenSymbol::USSD), DexShare::Token(TokenSymbol::EDF));

mod dex_oracle {
	pub use super::super::*;
}

parameter_types! {
	pub static USSDEDFPair: TradingPair = TradingPair::from_currency_ids(USSD, EDF).unwrap();
	pub static SEEEDFPair: TradingPair = TradingPair::from_currency_ids(SEE, EDF).unwrap();
}

#[derive_impl(frame_system::config_preludes::TestDefaultConfig as frame_system::DefaultConfig)]
impl frame_system::Config for Runtime {
	type AccountId = AccountId;
	type Lookup = IdentityLookup<Self::AccountId>;
	type Block = Block;
	type AccountData = ();
}

impl pallet_timestamp::Config for Runtime {
	type Moment = Moment;
	type OnTimestampSet = ();
	type MinimumPeriod = ConstU64<1000>;
	type WeightInfo = ();
}

thread_local! {
	static USSD_EDF_POOL: RefCell<(Balance, Balance)> = RefCell::new((Zero::zero(), Zero::zero()));
	static SEE_EDF_POOL: RefCell<(Balance, Balance)> = RefCell::new((Zero::zero(), Zero::zero()));
}

pub fn set_pool(trading_pair: &TradingPair, pool_0: Balance, pool_1: Balance) {
	if *trading_pair == USSDEDFPair::get() {
		USSD_EDF_POOL.with(|v| *v.borrow_mut() = (pool_0, pool_1));
	} else if *trading_pair == SEEEDFPair::get() {
		SEE_EDF_POOL.with(|v| *v.borrow_mut() = (pool_0, pool_1));
	}
}

pub struct MockDEX;
impl SwapManager<AccountId, Balance, CurrencyId> for MockDEX {
	fn get_liquidity_pool(currency_id_0: CurrencyId, currency_id_1: CurrencyId) -> (Balance, Balance) {
		TradingPair::from_currency_ids(currency_id_0, currency_id_1)
			.map(|trading_pair| {
				if trading_pair == USSDEDFPair::get() {
					USSD_EDF_POOL.with(|v| *v.borrow())
				} else if trading_pair == SEEEDFPair::get() {
					SEE_EDF_POOL.with(|v| *v.borrow())
				} else {
					(0, 0)
				}
			})
			.unwrap_or_else(|| (0, 0))
	}

	fn get_liquidity_token_address(_currency_id_a: CurrencyId, _currency_id_b: CurrencyId) -> Option<H160> {
		unimplemented!()
	}

	fn get_swap_amount(_: &[CurrencyId], _: SwapLimit<Balance>) -> Option<(Balance, Balance)> {
		unimplemented!()
	}

	fn get_best_price_swap_path(
		_: CurrencyId,
		_: CurrencyId,
		_: SwapLimit<Balance>,
		_: Vec<Vec<CurrencyId>>,
	) -> Option<(Vec<CurrencyId>, Balance, Balance)> {
		unimplemented!()
	}

	fn swap_with_specific_path(
		_: &AccountId,
		_: &[CurrencyId],
		_: SwapLimit<Balance>,
	) -> sp_std::result::Result<(Balance, Balance), DispatchError> {
		unimplemented!()
	}

	fn add_liquidity(
		_who: &AccountId,
		_currency_id_a: CurrencyId,
		_currency_id_b: CurrencyId,
		_max_amount_a: Balance,
		_max_amount_b: Balance,
		_min_share_increment: Balance,
		_stake_increment_share: bool,
	) -> sp_std::result::Result<(Balance, Balance, Balance), DispatchError> {
		unimplemented!()
	}

	fn remove_liquidity(
		_who: &AccountId,
		_currency_id_a: CurrencyId,
		_currency_id_b: CurrencyId,
		_remove_share: Balance,
		_min_withdrawn_a: Balance,
		_min_withdrawn_b: Balance,
		_by_unstake: bool,
	) -> sp_std::result::Result<(Balance, Balance), DispatchError> {
		unimplemented!()
	}
}

ord_parameter_types! {
	pub const One: AccountId = 1;
}

impl Config for Runtime {
	type DEX = MockDEX;
	type Time = Timestamp;
	type UpdateOrigin = EnsureSignedBy<One, AccountId>;
	type WeightInfo = ();
}

type Block = frame_system::mocking::MockBlock<Runtime>;

construct_runtime!(
	pub enum Runtime {
		System: frame_system,
		Timestamp: pallet_timestamp,
		EdfisOracle: edfis_oracle,
	}
);

pub struct ExtBuilder;

impl Default for ExtBuilder {
	fn default() -> Self {
		ExtBuilder
	}
}

impl ExtBuilder {
	pub fn build(self) -> sp_io::TestExternalities {
		let t = frame_system::GenesisConfig::<Runtime>::default()
			.build_storage()
			.unwrap();

		t.into()
	}
}
