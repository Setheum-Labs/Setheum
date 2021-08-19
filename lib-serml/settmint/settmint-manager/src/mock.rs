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

//! Mocks for the settmint-manager module.

#![cfg(test)]

use super::*;
use frame_support::{construct_runtime, ord_parameter_types, parameter_types, PalletId};
use frame_system::EnsureSignedBy;
use orml_traits::parameter_type_with_key;
use primitives::{Amount, ReserveIdentifier, TokenSymbol, TradingPair};
use sp_core::H256;
use sp_runtime::{
	testing::Header,
	traits::{IdentityLookup, One as OneT},
};
use support::{Price, PriceProvider, Ratio};
use sp_std::cell::RefCell;

pub type AccountId = u128;
pub type BlockNumber = u64;

pub const ALICE: AccountId = 1;
pub const BOB: AccountId = 2;
pub const CHARITY_FUND: AccountId = 3;

// Currencies constants - CurrencyId/TokenSymbol
pub const DNAR: CurrencyId = CurrencyId::Token(TokenSymbol::DNAR);
pub const DRAM: CurrencyId = CurrencyId::Token(TokenSymbol::DRAM);
pub const SETR: CurrencyId = CurrencyId::Token(TokenSymbol::SETR);
pub const AUDJ: CurrencyId = CurrencyId::Token(TokenSymbol::AUDJ);
pub const CADJ: CurrencyId = CurrencyId::Token(TokenSymbol::CADJ);
pub const CHFJ: CurrencyId = CurrencyId::Token(TokenSymbol::CHFJ);
pub const EURJ: CurrencyId = CurrencyId::Token(TokenSymbol::EURJ);
pub const GBPJ: CurrencyId = CurrencyId::Token(TokenSymbol::GBPJ);
pub const JPYJ: CurrencyId = CurrencyId::Token(TokenSymbol::JPYJ);
pub const SARJ: CurrencyId = CurrencyId::Token(TokenSymbol::SARJ);
pub const SEKJ: CurrencyId = CurrencyId::Token(TokenSymbol::SEKJ);
pub const SGDJ: CurrencyId = CurrencyId::Token(TokenSymbol::SGDJ);
pub const USDJ: CurrencyId = CurrencyId::Token(TokenSymbol::USDJ);


mod settmint_manager {
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

parameter_types! {
	pub const ExistentialDeposit: Balance = 1;
	pub const MaxReserves: u32 = 50;
}

impl pallet_balances::Config for Runtime {
	type Balance = Balance;
	type DustRemoval = ();
	type Event = Event;
	type ExistentialDeposit = ExistentialDeposit;
	type AccountStore = frame_system::Pallet<Runtime>;
	type MaxLocks = ();
	type MaxReserves = MaxReserves;
	type ReserveIdentifier = ReserveIdentifier;
	type WeightInfo = ();
}

parameter_types! {
	pub const GetNativeCurrencyId: CurrencyId = DNAR;
}

impl orml_currencies::Config for Runtime {
	type Event = Event;
	type MultiCurrency = Tokens;
	type NativeCurrency = AdaptedBasicCurrency;
	type GetNativeCurrencyId = GetNativeCurrencyId;
	type WeightInfo = ();
}
pub type AdaptedBasicCurrency = orml_currencies::BasicCurrencyAdapter<Runtime, PalletBalances, Amount, BlockNumber>;

parameter_types! {
	pub const GetExchangeFee: (u32, u32) = (1, 100);
	pub const DexPalletId: PalletId = PalletId(*b"set/sdex");
	pub const TradingPathLimit: u32 = 3;
	pub EnabledTradingPairs: Vec<TradingPair> = vec![
		TradingPair::from_currency_ids(DNAR, SETR).unwrap(),
		TradingPair::from_currency_ids(AUDJ, SETR).unwrap(),
		TradingPair::from_currency_ids(CADJ, SETR).unwrap(),
		TradingPair::from_currency_ids(CHFJ, SETR).unwrap(),
		TradingPair::from_currency_ids(EURJ, SETR).unwrap(),
		TradingPair::from_currency_ids(GBPJ, SETR).unwrap(),
		TradingPair::from_currency_ids(JPYJ, SETR).unwrap(),
		TradingPair::from_currency_ids(SARJ, SETR).unwrap(),
		TradingPair::from_currency_ids(SEKJ, SETR).unwrap(),
		TradingPair::from_currency_ids(SGDJ, SETR).unwrap(),
		TradingPair::from_currency_ids(USDJ, SETR).unwrap(),
		TradingPair::from_currency_ids(AUDJ, DNAR).unwrap(),
		TradingPair::from_currency_ids(CADJ, DNAR).unwrap(),
		TradingPair::from_currency_ids(CHFJ, DNAR).unwrap(),
		TradingPair::from_currency_ids(EURJ, DNAR).unwrap(),
		TradingPair::from_currency_ids(GBPJ, DNAR).unwrap(),
		TradingPair::from_currency_ids(JPYJ, DNAR).unwrap(),
		TradingPair::from_currency_ids(SARJ, DNAR).unwrap(),
		TradingPair::from_currency_ids(SEKJ, DNAR).unwrap(),
		TradingPair::from_currency_ids(SGDJ, DNAR).unwrap(),
		TradingPair::from_currency_ids(USDJ, DNAR).unwrap(),
	];
}

impl setheum_dex::Config for Runtime {
	type Event = Event;
	type Currency = Currencies;
	type GetExchangeFee = GetExchangeFee;
	type TradingPathLimit = TradingPathLimit;
	type PalletId = DexPalletId;
	type CurrencyIdMapping = ();
	type WeightInfo = ();
	type ListingOrigin = EnsureSignedBy<One, AccountId>;
}

thread_local! {
	static RELATIVE_PRICE: RefCell<Option<Price>> = RefCell::new(Some(Price::one()));
}

pub struct MockPriceSource;
impl MockPriceSource {
	pub fn set_relative_price(price: Option<Price>) {
		RELATIVE_PRICE.with(|v| *v.borrow_mut() = price);
	}
}
impl PriceProvider<CurrencyId> for MockPriceSource {

	fn get_relative_price(_base: CurrencyId, _quota: CurrencyId) -> Option<Price> {
		RELATIVE_PRICE.with(|v| *v.borrow_mut())
	}

	fn get_market_price(_currency_id: CurrencyId) -> Option<Price> {
		Some(Price::one())
	}

	fn get_peg_price(_currency_id: CurrencyId) -> Option<Price> {
		Some(Price::one())
	}

	fn get_setter_price() -> Option<Price> {
		Some(Price::one())
	}

	fn get_price(_currency_id: CurrencyId) -> Option<Price> {
		None
	}

	fn lock_price(_currency_id: CurrencyId) {}

	fn unlock_price(_currency_id: CurrencyId) {}
}

ord_parameter_types! {
	pub const One: AccountId = 1;
}

parameter_types! {
	pub StableCurrencyIds: Vec<CurrencyId> = vec![
		SETR,
 		AUDJ,
		CADJ,
		CHFJ,
		EURJ,
		GBPJ,
		JPYJ,
 		SARJ,
 		SEKJ,
 		SGDJ,
		USDJ,
	];
	pub const SetterCurrencyId: CurrencyId = SETR;  // Setter  currency ticker is SETR/
	pub const GetSettUSDCurrencyId: CurrencyId = USDJ;  // Setter  currency ticker is USDJ/
	pub const DirhamCurrencyId: CurrencyId = DRAM; // SettinDEX currency ticker is DRAM/

	pub const SerpTreasuryPalletId: PalletId = PalletId(*b"set/serp");
	pub const CharityFundAccountId: AccountId = CHARITY_FUND;
	pub const SettPayTreasuryAccountId: AccountId = SETRPAY;
	pub const CashDropVaultAccountId: AccountId = VAULT;

	pub SerpTesSchedule: BlockNumber = 60; // Triggers SERP-TES for serping after Every 60 blocks
	pub CashDropPeriod: BlockNumber = 120; // Triggers SERP-TES for serping after Every 60 blocks
	pub MaxSlippageSwapWithDEX: Ratio = Ratio::one();

	pub RewardableCurrencyIds: Vec<CurrencyId> = vec![
		DNAR,
		DRAM,
		SETR,
 		AUDJ,
		CADJ,
		CHFJ,
		EURJ,
		GBPJ,
		JPYJ,
 		SARJ,
 		SEKJ,
 		SGDJ,
		USDJ,
	];
	pub NonStableDropCurrencyIds: Vec<CurrencyId> = vec![DNAR, DRAM];
	pub SettCurrencyDropCurrencyIds: Vec<CurrencyId> = vec![
 		AUDJ,
		CADJ,
		CHFJ,
		EURJ,
		GBPJ,
		JPYJ,
 		SARJ,
 		SEKJ,
 		SGDJ,
		USDJ,
	];
}

parameter_type_with_key! {
	pub MinimumClaimableTransferAmounts: |currency_id: CurrencyId| -> Balance {
		match currency_id {
			&SETR => 2,
			&AUDJ => 2,
			&CHFJ => 2,
			&EURJ => 2,
			&GBPJ => 2,
			&JPYJ => 2,
			&USDJ => 2,
			_ => 0,
		}
	};
}

parameter_type_with_key! {
	pub GetStableCurrencyMinimumSupply: |currency_id: CurrencyId| -> Balance {
		match currency_id {
			&SETR => 10_000,
			&AUDJ => 10_000,
			&CHFJ => 10_000,
			&EURJ => 10_000,
			&GBPJ => 10_000,
			&JPYJ => 10_000,
			&USDJ => 10_000,
			_ => 0,
		}
	};
}

impl serp_treasury::Config for Runtime {
	type Event = Event;
	type Currency = Currencies;
	type StableCurrencyIds = StableCurrencyIds;
	type GetStableCurrencyMinimumSupply = GetStableCurrencyMinimumSupply;
	type GetNativeCurrencyId = GetNativeCurrencyId;
	type SetterCurrencyId = SetterCurrencyId;
	type GetSettUSDCurrencyId = GetSettUSDCurrencyId;
	type DirhamCurrencyId = DirhamCurrencyId;
	type SerpTesSchedule = SerpTesSchedule;
	type CashDropPeriod = CashDropPeriod;
	type SettPayTreasuryAccountId = SettPayTreasuryAccountId;
	type CashDropVaultAccountId = CashDropVaultAccountId;
	type CharityFundAccountId = CharityFundAccountId;
	type Dex = SetheumDEX;
	type MaxSlippageSwapWithDEX = MaxSlippageSwapWithDEX;
	type PriceSource = MockPriceSource;
	type RewardableCurrencyIds = RewardableCurrencyIds;
	type NonStableDropCurrencyIds = StableCurrencyIds;
	type SettCurrencyDropCurrencyIds = SettCurrencyDropCurrencyIds;
	type MinimumClaimableTransferAmounts = MinimumClaimableTransferAmounts;
	type PalletId = SerpTreasuryPalletId;
	type WeightInfo = ();
}

// mock convert
pub struct MockConvert;
impl Convert<(CurrencyId, Balance), Balance> for MockConvert {
	fn convert(a: (CurrencyId, Balance)) -> Balance {
		(a.1 / Balance::from(2u64)).into()
	}
}

parameter_types! {
	pub StandardCurrencyIds: Vec<CurrencyId> = vec![
		AUDJ,
		CADJ,
		CHFJ,
		EURJ,
		GBPJ,
		JPYJ,
		SARJ,
		SEKJ,
		SGDJ,
		USDJ,
	];
	pub const GetReserveCurrencyId: CurrencyId = SETR;
	pub const SettmintManagerPalletId: PalletId = PalletId(*b"set/mint");

}

impl Config for Runtime {
	type Event = Event;
	type Convert = MockConvert;
	type Currency = Currencies;
	type StandardCurrencyIds = StandardCurrencyIds;
	type GetReserveCurrencyId = GetReserveCurrencyId;
	type SerpTreasury = SerpTreasuryModule;
	type PalletId = SettmintManagerPalletId;
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
		SettmintManagerModule: settmint_manager::{Pallet, Storage, Call, Event<T>},
		Tokens: orml_tokens::{Pallet, Storage, Event<T>, Config<T>},
		PalletBalances: pallet_balances::{Pallet, Call, Storage, Event<T>},
		Currencies: orml_currencies::{Pallet, Call, Event<T>},
		SerpTreasuryModule: serp_treasury::{Pallet, Storage, Event<T>},
		SetheumDEX: setheum_dex::{Pallet, Storage, Call, Event<T>, Config<T>},
	}
);

pub struct ExtBuilder {
	balances: Vec<(AccountId, CurrencyId, Balance)>,
}

impl Default for ExtBuilder {
	fn default() -> Self {
		Self {
			balances: vec![
				(ALICE, SETR, 1000),
				(ALICE, USDJ, 1000),
				(ALICE, EURJ, 1000),
				(ALICE, CHFJ, 1000),
				(BOB, SETR, 1000),
				(BOB, USDJ, 1000),
				(BOB, EURJ, 1000),
				(BOB, CHFJ, 1000),
			],
		}
	}
}

impl ExtBuilder {
	pub fn build(self) -> sp_io::TestExternalities {
		let mut t = frame_system::GenesisConfig::default()
			.build_storage::<Runtime>()
			.unwrap();
		orml_tokens::GenesisConfig::<Runtime> {
			balances: self.balances,
		}
		.assimilate_storage(&mut t)
		.unwrap();
		t.into()
	}
}
