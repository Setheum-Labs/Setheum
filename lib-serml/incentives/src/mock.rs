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

//! Mocks for the incentive module.

#![cfg(test)]

use super::*;
use frame_support::{
	construct_runtime,
	dispatch::{DispatchError, DispatchResult},
	ord_parameter_types, parameter_types,
};
use frame_system::EnsureSignedBy;
use orml_traits::parameter_type_with_key;
use primitives::{DexShare, TokenSymbol};
use sp_core::{H160, H256};
use sp_runtime::{testing::Header, traits::IdentityLookup};
use sp_std::cell::RefCell;
pub use support::{SerpTreasury, DEXManager, Price, Ratio};

pub type AccountId = u128;
pub type BlockNumber = u64;

pub const ALICE: AccountId = 1;
pub const BOB: AccountId = 2;
pub const VAULT: AccountId = 10;
pub const VALIDATOR: AccountId = 3;
pub const DNAR: CurrencyId = CurrencyId::Token(TokenSymbol::DNAR);
pub const SETT: CurrencyId = CurrencyId::Token(TokenSymbol::SETT);
pub const USDJ: CurrencyId = CurrencyId::Token(TokenSymbol::USDJ);
pub const CHFJ_USDJ_LP: CurrencyId =
	CurrencyId::DexShare(DexShare::Token(TokenSymbol::CHFJ), DexShare::Token(TokenSymbol::USDJ));
pub const DNAR_USDJ_LP: CurrencyId =
	CurrencyId::DexShare(DexShare::Token(TokenSymbol::DNAR), DexShare::Token(TokenSymbol::USDJ));

mod incentives {
	pub use super::super::*;
}

parameter_types! {
	pub const BlockHashCount: u64 = 250;
}

impl frame_system::Config for Runtime {
	type Origin = Origin;
	type Index = u64;
	type BlockNumber = BlockNumber;
	type Call = Call;
	type Hash = H256;
	type Hashing = ::sp_runtime::traits::BlakeTwo256;
	type AccountId = AccountId;
	type Lookup = IdentityLookup<Self::AccountId>;
	type Header = Header;
	type Event = Event;
	type BlockHashCount = BlockHashCount;
	type BlockWeights = ();
	type BlockLength = ();
	type Version = ();
	type PalletInfo = PalletInfo;
	type AccountData = pallet_balances::AccountData<Balance>;
	type OnNewAccount = ();
	type OnKilledAccount = ();
	type DbWeight = ();
	type BaseCallFilter = ();
	type SystemWeightInfo = ();
	type SS58Prefix = ();
	type OnSetCode = ();
}

parameter_type_with_key! {
	pub ExistentialDeposits: |_currency_id: CurrencyId| -> Balance {
		Default::default()
	};
}

impl orml_tokens::Config for Runtime {
	type Event = Event;
	type Balance = Balance;
	type Amount = Amount;
	type CurrencyId = CurrencyId;
	type WeightInfo = ();
	type ExistentialDeposits = ExistentialDeposits;
	type OnDust = ();
	type MaxLocks = ();
}

pub struct MockSerpTreasury;
impl SerpTreasury<AccountId> for MockSerpTreasury {
	type Balance = Balance;
	type CurrencyId = CurrencyId;

	fn get_serpluspool() -> Balance {
		unimplemented!()
	}

	fn get_standard_pool() -> Balance {
		unimplemented!()
	}

	fn get_total_reserves(_: CurrencyId) -> Balance {
		unimplemented!()
	}

	fn get_standard_proportion(_: Balance) -> Ratio {
		unimplemented!()
	}

	fn on_system_standard(_: Balance) -> DispatchResult {
		unimplemented!()
	}

	fn on_system_serplus(_: Balance) -> DispatchResult {
		unimplemented!()
	}

	fn issue_standard(who: &AccountId, standard: Balance) -> DispatchResult {
		TokensModule::deposit(AUSD, who, standard)
	}

	fn burn_standard(_: &AccountId, _: Balance) -> DispatchResult {
		unimplemented!()
	}

	fn issue_dexer(who: &AccountId, dexer: Balance) -> DispatchResult {
		TokensModule::deposit(AUSD, who, dexer)
	}

	fn burn_dexer(_: &AccountId, _: Balance) -> DispatchResult {
		unimplemented!()
	}

	fn deposit_serplus(_: &AccountId, _: Balance) -> DispatchResult {
		unimplemented!()
	}

	fn deposit_reserve(_: &AccountId, _: CurrencyId, _: Balance) -> DispatchResult {
		unimplemented!()
	}

	fn withdraw_reserve(_: &AccountId, _: CurrencyId, _: Balance) -> DispatchResult {
		unimplemented!()
	}
}

pub struct MockSetheumDex;
impl DEXManager<AccountId, CurrencyId, Balance> for MockSetheumDex {
	fn get_liquidity_pool(currency_id_a: CurrencyId, currency_id_b: CurrencyId) -> (Balance, Balance) {
		match (currency_id_a, currency_id_b) {
			(SETT, CHFJ) => (500, 100),
			(SETT, DNAR) => (400, 100),
			(CHFJ, SETT) => (100, 500),
			(DNAR, SETT) => (100, 400),
			_ => (0, 0),
		}
	}

	fn get_liquidity_token_address(_currency_id_a: CurrencyId, _currency_id_b: CurrencyId) -> Option<H160> {
		unimplemented!()
	}

	fn get_swap_target_amount(_: &[CurrencyId], _: Balance, _: Option<Ratio>) -> Option<Balance> {
		unimplemented!()
	}

	fn get_swap_supply_amount(_: &[CurrencyId], _: Balance, _: Option<Ratio>) -> Option<Balance> {
		unimplemented!()
	}

	fn swap_with_exact_supply(
		_: &AccountId,
		_: &[CurrencyId],
		_: Balance,
		_: Balance,
		_: Option<Ratio>,
	) -> sp_std::result::Result<Balance, DispatchError> {
		unimplemented!()
	}

	fn swap_with_exact_target(
		_: &AccountId,
		_: &[CurrencyId],
		_: Balance,
		_: Balance,
		_: Option<Ratio>,
	) -> sp_std::result::Result<Balance, DispatchError> {
		unimplemented!()
	}

	fn add_liquidity(_: &AccountId, _: CurrencyId, _: CurrencyId, _: Balance, _: Balance, _: bool) -> DispatchResult {
		unimplemented!()
	}

	fn remove_liquidity(_: &AccountId, _: CurrencyId, _: CurrencyId, _: Balance, _: bool) -> DispatchResult {
		unimplemented!()
	}
}

impl orml_rewards::Config for Runtime {
	type Share = Balance;
	type Balance = Balance;
	type PoolId = PoolId<AccountId>;
	type Handler = IncentivesModule;
}

parameter_types! {
	pub const RewardsVaultAccountId: AccountId = VAULT;
	pub const DexIncentivePool: AccountId = 11;
	pub const AccumulatePeriod: BlockNumber = 10;
	pub const DexCurrencyId: CurrencyId = SDEX;
	pub const StableCurrencyId: CurrencyId = SETT;
	pub const IncentivesPalletId: PalletId = PalletId(*b"dnr/inct");
}

ord_parameter_types! {
	pub const Four: AccountId = 4;
}

impl Config for Runtime {
	type Event = Event;
	type SettersIncentivePool = SettersIncentivePool;
	type RewardsVaultAccountId = RewardsVaultAccountId;
	type DexIncentivePool = DexIncentivePool;
	type AccumulatePeriod = AccumulatePeriod;
	type DexCurrencyId = DexCurrencyId;
	type StableCurrencyId = StableCurrencyId;
	type UpdateOrigin = EnsureSignedBy<Four, AccountId>;
	type SerpTreasury = MockSerpTreasury;
	type Currency = TokensModule;
	type DEX = MockDEX;
	type PalletId = IncentivesPalletId;
	type WeightInfo = ();
}

type UncheckedExtrinsic = frame_system::mocking::MockUncheckedExtrinsic<Runtime>;
type Block = frame_system::mocking::MockBlock<Runtime>;

construct_runtime!(
	pub enum Runtime where
		Block = Block,
		NodeBlock = Block,
		UncheckedExtrinsic = UncheckedExtrinsic,
	{
		System: frame_system::{Pallet, Call, Storage, Config, Event<T>},
		IncentivesModule: setheum_incentives::{Pallet, Storage, Call, Event<T>},
		TokensModule: orml_tokens::{Pallet, Storage, Event<T>},
		RewardsModule: orml_rewards::{Pallet, Storage, Call},
	}
);

pub struct ExtBuilder {
	endowed_accounts: Vec<(AccountId, CurrencyId, Balance)>,
}

impl Default for ExtBuilder {
	fn default() -> Self {
		Self {
			endowed_accounts: vec![(DNAR, 10_000)],
		}
	}
}

impl ExtBuilder {
	pub fn build(self) -> sp_io::TestExternalities {
		let mut t = frame_system::GenesisConfig::default()
			.build_storage::<Runtime>()
			.unwrap();
		orml_tokens::GenesisConfig::<Runtime> {
			endowed_accounts: self.endowed_accounts,
		}
		.assimilate_storage(&mut t)
		.unwrap();
		t.into()
	}
}
