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
use primitives::{CurrencyId, TokenSymbol};
use sp_core::{H160, H256};
use sp_runtime::{testing::Header, traits::IdentityLookup};
use sp_std::cell::RefCell;
pub use support::{SerpTreasury, DEXManager, Price, Rate, Ratio};

pub type AccountId = u128;
pub type BlockNumber = u64;

pub const ALICE: AccountId = 1;
pub const BOB: AccountId = 2;

// Currencies constants - CurrencyId/TokenSymbol
pub const DNAR: CurrencyId = CurrencyId::Token(TokenSymbol::DNAR);
pub const DRAM: CurrencyId = CurrencyId::Token(TokenSymbol::DRAM); //  Setheum Dirham
pub const SETT: CurrencyId = CurrencyId::Token(TokenSymbol::SETT); // Setter   -  The Defacto stablecoin & settmint reserve asset
pub const USDJ: CurrencyId = CurrencyId::Token(TokenSymbol::USDJ); // Setheum USD (US Dollar stablecoin)
pub const GBPJ: CurrencyId = CurrencyId::Token(TokenSymbol::GBPJ); // Setheum GBP (Pound Sterling stablecoin)
pub const EURJ: CurrencyId = CurrencyId::Token(TokenSymbol::EURJ); // Setheum EUR (Euro stablecoin)
pub const KWDJ: CurrencyId = CurrencyId::Token(TokenSymbol::KWDJ); // Setheum KWD (Kuwaiti Dinar stablecoin)
pub const JODJ: CurrencyId = CurrencyId::Token(TokenSymbol::JODJ); // Setheum JOD (Jordanian Dinar stablecoin)
pub const BHDJ: CurrencyId = CurrencyId::Token(TokenSymbol::BHDJ); // Setheum BHD (Bahraini Dirham stablecoin)
pub const KYDJ: CurrencyId = CurrencyId::Token(TokenSymbol::KYDJ); // Setheum KYD (Cayman Islands Dollar stablecoin)
pub const OMRJ: CurrencyId = CurrencyId::Token(TokenSymbol::OMRJ); // Setheum OMR (Omani Riyal stablecoin)
pub const CHFJ: CurrencyId = CurrencyId::Token(TokenSymbol::CHFJ); // Setheum CHF (Swiss Franc stablecoin)
pub const GIPJ: CurrencyId = CurrencyId::Token(TokenSymbol::GIPJ); // Setheum GIP (Gibraltar Pound stablecoin)

// LP tokens constants - CurrencyId/TokenSymbol : Dex Shares
pub const CHFJ_USDJ_LP: CurrencyId = CurrencyId::DexShare(TokenSymbol::CHFJ, TokenSymbol::USDJ);
pub const CHFJ_SETT_LP: CurrencyId = CurrencyId::DexShare(TokenSymbol::CHFJ, TokenSymbol::USDJ);
pub const DNAR_USDJ_LP: CurrencyId = CurrencyId::DexShare(TokenSymbol::DNAR, TokenSymbol::USDJ);
pub const DNAR_SETT_LP: CurrencyId = CurrencyId::DexShare(TokenSymbol::DNAR, TokenSymbol::USDJ);

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
	type Amount = Amount;
	type Balance = Balance;
	type CurrencyId = CurrencyId;
	type BlockNumber = BlockNumber;

	fn get_adjustment_frequency() -> BlockNumber {
		unimplemented!()
	}

	fn get_total_setter() -> Balance {
		unimplemented!()
	}

	fn get_propper_proportion(_: Balance, _: CurrencyId) -> Ratio {
		unimplemented!()
	}

	fn get_serplus_serpup(_: Balance, _: CurrencyId) -> DispatchResult {
		unimplemented!()
	}

	fn get_settpay_serpup(_: Balance, _: CurrencyId) -> DispatchResult {
		unimplemented!()
	}

	fn get_treasury_serpup(_: Balance, _: CurrencyId) -> DispatchResult {
		unimplemented!()
	}

	fn get_sif_serpup(_: Balance, _: CurrencyId) -> DispatchResult {
		unimplemented!()
	}

	fn get_charity_fund_serpup(_: Balance, _: CurrencyId) -> DispatchResult {
		unimplemented!()
	}

	fn on_serpup(_: CurrencyId, _: Amount) -> DispatchResult {
		unimplemented!()
	}

	fn get_minimum_supply(_: CurrencyId) -> Balance {
		unimplemented!()
	}

	fn on_serpdown(_: CurrencyId, _: Amount) -> DispatchResult {
		unimplemented!()
	}

	fn on_serp_tes() -> DispatchRsult {
		unimplemented!()
	}

	fn serp_tes(_: CurrencyId) -> DispatchResult {
		unimplemented!()
	}

	fn issue_standard(_: CurrencyId, _: AccountId, _: Balance) -> DispatchResult {
		unimplemented!()
	}

	fn burn_standard(_: CurrencyId, _: AccountId, _: Balance) -> DispatchResult {
		unimplemented!()
	}

	fn issue_propper(_: CurrencyId, _: AccountId, _: Balance) -> DispatchResult {
		unimplemented!()
	}

	fn burn_propper(_: CurrencyId, _: AccountId, _: Balance) -> DispatchResult {
		unimplemented!()
	}
	fn issue_setter(_: AccountId, _: Balance) -> DispatchResult {
		unimplemented!()
	}

	fn burn_setter(_: AccountId, _:Balance) -> DispatchResult {
		unimplemented!()
	}

	fn issue_dexer(_: AccountId, _:Balance) -> DispatchResult {
		unimplemented!()
	}

	fn burn_dexer(_: AccountId, _:Balance) -> DispatchResult {
		unimplemented!()
	}

	fn deposit_serplus(_: CurrencyId, _: AccountId, _: Balance) -> DispatchResult {
		unimplemented!()
	}

	fn deposit_setter(_: AccountId, _:Balance) -> DispatchResult {
		unimplemented!()
	}

	fn burn_setter(_: AccountId, _:Balance) -> DispatchResult {
		unimplemented!()
	}
}

pub struct MockDex;
impl DEXManager<AccountId, CurrencyId, Balance> for MockDex {
	fn get_liquidity_pool(currency_id_a: CurrencyId, currency_id_b: CurrencyId) -> (Balance, Balance) {
		match (currency_id_a, currency_id_b) {
			(SETT, CHFJ) => (500, 100),
			(SETT, DNAR) => (400, 100),
			(CHFJ, SETT) => (100, 500),
			(DNAR, SETT) => (100, 400),
			_ => (0, 0),
		}
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
	type PoolId = PoolId;
	type Handler = IncentivesModule;
	type WeightInfo = ();
}

parameter_types! {
	pub const DexIncentivePool: AccountId = 10;
	pub const DexPremiumPool: AccountId = 11;
	pub const DexPremiumInflationRate: Balance = 200; // RATE PER ACCUMULATION PERIOD
	pub const SetterCurrencyId: CurrencyId = SETT;
	pub const DexerCurrencyId: CurrencyId = DRAM;
	pub const NativeCurrencyId: CurrencyId = DNAR;
	pub StableCurrencyIds: Vec<CurrencyId> = vec![
		SETT, // Setter   -  The Defacto stablecoin & settmint reserve asset
		USDJ, // Setheum USD (US Dollar stablecoin)
		GBPJ, // Setheum GBP (Pound Sterling stablecoin)
		EURJ, // Setheum EUR (Euro stablecoin)
		KWDJ, // Setheum KWD (Kuwaiti Dinar stablecoin)
		JODJ, // Setheum JOD (Jordanian Dinar stablecoin)
		BHDJ, // Setheum BHD (Bahraini Dirham stablecoin)
		KYDJ, // Setheum KYD (Cayman Islands Dollar stablecoin)
		OMRJ, // Setheum OMR (Omani Riyal stablecoin)
		CHFJ, // Setheum CHF (Swiss Franc stablecoin)
		GIPJ, // Setheum GIP (Gibraltar Pound stablecoin)
	];
	pub const IncentivesPalletId: PalletId = PalletId(*b"set/inct");
}

parameter_type_with_key! {
	pub DexPremiumRewardRates: |_currency_id: CurrencyId| -> (Rate, Rate) {
		match currency_id {
			&CHFJ_USDJ_LP => (10, 100),
			&CHFJ_SETT_LP => (20, 100),
			&DNAR_USDJ_LP => (20, 100),
			&DNAR_SETT_LP => (50, 100),
			_ => None,
		}
	};
}

ord_parameter_types! {
	pub const Four: AccountId = 4;
}

impl Config for Runtime {
	type Event = Event;
	type DexIncentivePool = DexIncentivePool;
	type DexPremiumPool = DexPremiumPool;
	type DexPremiumRewardRates = DexPremiumRewardRates;
	type DexPremiumInflationRate = DexPremiumInflationRate;
	type SetterCurrencyId = SetterCurrencyId;
	type DexerCurrencyId = DexerCurrencyId;
	type NativeCurrencyId = NativeCurrencyId;
	type StableCurrencyIds = StableCurrencyIds;
	type UpdateOrigin = EnsureSignedBy<Four, AccountId>;
	type AccumulatePeriodUpdateOrigin = EnsureSignedBy<Four, AccountId>;
	type SerpTreasury = MockSerpTreasury;
	type Currency = TokensModule;
	type Dex = MockDex;
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

#[derive(Default)]
pub struct ExtBuilder;

impl ExtBuilder {
	pub fn build(self) -> sp_io::TestExternalities {
		let t = frame_system::GenesisConfig::default()
			.build_storage::<Runtime>()
			.unwrap();
		t.into()
	}
}
