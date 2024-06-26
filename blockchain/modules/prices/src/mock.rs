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

//! Mocks for the prices module.

#![cfg(test)]

use super::*;
use frame_support::{
	construct_runtime, ord_parameter_types, parameter_types,
	traits::{ConstU64, Everything, Nothing},
};
use frame_system::EnsureSignedBy;
use module_support::{mocks::MockErc20InfoMapping, ExchangeRate, SwapLimit};
use orml_traits::{parameter_type_with_key, DataFeeder};
use primitives::{currency::DexShare, Amount, TokenSymbol};
use sp_core::{H160, H256};
use sp_runtime::{
	traits::{IdentityLookup, One as OneT, Zero},
	BuildStorage, DispatchError, FixedPointNumber,
};
use sp_std::cell::RefCell;

pub type AccountId = u128;
pub type BlockNumber = u64;

pub const SEE: CurrencyId = CurrencyId::Token(TokenSymbol::SEE);
pub const USSD: CurrencyId = CurrencyId::Token(TokenSymbol::USSD);
pub const EDF: CurrencyId = CurrencyId::Token(TokenSymbol::EDF);
pub const LP_USSD_SEE: CurrencyId =
	CurrencyId::DexShare(DexShare::Token(TokenSymbol::USSD), DexShare::Token(TokenSymbol::SEE));


mod prices {
	pub use super::super::*;
}

impl frame_system::Config for Runtime {
	type RuntimeOrigin = RuntimeOrigin;
	type Nonce = u64;
	type RuntimeCall = RuntimeCall;
	type Hash = H256;
	type Hashing = ::sp_runtime::traits::BlakeTwo256;
	type AccountId = AccountId;
	type Lookup = IdentityLookup<Self::AccountId>;
	type Block = Block;
	type RuntimeEvent = RuntimeEvent;
	type BlockHashCount = ConstU64<250>;
	type BlockWeights = ();
	type BlockLength = ();
	type Version = ();
	type PalletInfo = PalletInfo;
	type AccountData = ();
	type OnNewAccount = ();
	type OnKilledAccount = ();
	type DbWeight = ();
	type BaseCallFilter = Everything;
	type SystemWeightInfo = ();
	type SS58Prefix = ();
	type OnSetCode = ();
	type MaxConsumers = ConstU32<16>;
}

thread_local! {
	static CHANGED: RefCell<bool> = RefCell::new(false);
}

pub fn mock_oracle_update() {
	CHANGED.with(|v| *v.borrow_mut() = true)
}

pub struct MockDataProvider;
impl DataProvider<CurrencyId, Price> for MockDataProvider {
	fn get(currency_id: &CurrencyId) -> Option<Price> {
		if CHANGED.with(|v| *v.borrow_mut()) {
			match *currency_id {
				USSD => None,
				SEE => Some(Price::saturating_from_integer(10)),
				EDF => Some(Price::saturating_from_integer(200)),
				_ => None,
			}
		} else {
			match *currency_id {
				USSD => Some(Price::saturating_from_rational(99, 100)),
				SEE => Some(Price::saturating_from_integer(100)),
				EDF => None,
				_ => None,
			}
		}
	}
}

impl DataFeeder<CurrencyId, Price, AccountId> for MockDataProvider {
	fn feed_value(_: Option<AccountId>, _: CurrencyId, _: Price) -> sp_runtime::DispatchResult {
		Ok(())
	}
}

pub struct MockSwapManager;
impl SwapManager<AccountId, Balance, CurrencyId> for MockSwapManager {
	fn get_liquidity_pool(currency_id_a: CurrencyId, currency_id_b: CurrencyId) -> (Balance, Balance) {
		match (currency_id_a, currency_id_b) {
			(USSD, SEE) => (10000, 200),
			_ => (0, 0),
		}
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

parameter_type_with_key! {
	pub ExistentialDeposits: |_currency_id: CurrencyId| -> Balance {
		Default::default()
	};
}

impl orml_tokens::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type Balance = Balance;
	type Amount = Amount;
	type CurrencyId = CurrencyId;
	type WeightInfo = ();
	type ExistentialDeposits = ExistentialDeposits;
	type CurrencyHooks = ();
	type MaxLocks = ();
	type MaxReserves = ();
	type ReserveIdentifier = [u8; 8];
	type DustRemovalWhitelist = Nothing;
}

ord_parameter_types! {
	pub const One: AccountId = 1;
}

parameter_types! {
	pub const GetUSSDCurrencyId: CurrencyId = USSD;
	pub const GetSEECurrencyId: CurrencyId = SEE;
	pub USSDFixedPrice: Price = Price::one();
}

impl Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type Source = MockDataProvider;
	type GetUSSDCurrencyId = GetUSSDCurrencyId;
	type USSDFixedPrice = USSDFixedPrice;
	type GetSEECurrencyId = GetSEECurrencyId;
	type LockOrigin = EnsureSignedBy<One, AccountId>;
	type SwapManager = MockSwapManager;
	type Currency = Tokens;
	type Erc20InfoMapping = MockErc20InfoMapping;
	type PricingPegged = PricingPegged;
	type WeightInfo = ();
}

type Block = frame_system::mocking::MockBlock<Runtime>;

construct_runtime!(
	pub enum Runtime {
		System: frame_system,
		PricesModule: prices,
		Tokens: orml_tokens,
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
