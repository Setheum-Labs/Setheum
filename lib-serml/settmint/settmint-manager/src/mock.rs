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
	testing::Header, FixedPointNumber,
	traits::{IdentityLookup, One as OneT},
};
use support::{Price, PriceProvider, Ratio};
use sp_std::cell::RefCell;

pub type AccountId = u128;
pub type BlockNumber = u64;

pub const ALICE: AccountId = 1;
pub const BOB: AccountId = 2;
pub const CHARITY_FUND: AccountId = 3;
pub const SETRPAY: AccountId = 9;
pub const VAULT: AccountId = 10;
pub const ROOT: AccountId = 11;

// Currencies constants - CurrencyId/TokenSymbol
pub const DNAR: CurrencyId = CurrencyId::Token(TokenSymbol::DNAR);
pub const DRAM: CurrencyId = CurrencyId::Token(TokenSymbol::DRAM);
pub const SETR: CurrencyId = CurrencyId::Token(TokenSymbol::SETR);
pub const SETUSD: CurrencyId = CurrencyId::Token(TokenSymbol::SETUSD);
pub const SETEUR: CurrencyId = CurrencyId::Token(TokenSymbol::SETEUR);
pub const SETGBP: CurrencyId = CurrencyId::Token(TokenSymbol::SETGBP);
pub const SETCHF: CurrencyId = CurrencyId::Token(TokenSymbol::SETCHF);
pub const SETSAR: CurrencyId = CurrencyId::Token(TokenSymbol::SETSAR);


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
	type DustRemovalWhitelist = ();
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
		TradingPair::from_currency_ids(SETCHF, SETR).unwrap(),
		TradingPair::from_currency_ids(SETEUR, SETR).unwrap(),
		TradingPair::from_currency_ids(SETGBP, SETR).unwrap(),
		TradingPair::from_currency_ids(SETSAR, SETR).unwrap(),
		TradingPair::from_currency_ids(SETUSD, SETR).unwrap(),
		TradingPair::from_currency_ids(SETCHF, DNAR).unwrap(),
		TradingPair::from_currency_ids(SETEUR, DNAR).unwrap(),
		TradingPair::from_currency_ids(SETGBP, DNAR).unwrap(),
		TradingPair::from_currency_ids(SETSAR, DNAR).unwrap(),
		TradingPair::from_currency_ids(SETUSD, DNAR).unwrap(),
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
	pub fn _set_relative_price(price: Option<Price>) {
		RELATIVE_PRICE.with(|v| *v.borrow_mut() = price);
	}
}
impl PriceProvider<CurrencyId> for MockPriceSource {

	fn get_relative_price(_base: CurrencyId, _quota: CurrencyId) -> Option<Price> {
		RELATIVE_PRICE.with(|v| *v.borrow_mut())
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
		SETCHF,
		SETEUR,
		SETGBP,
 		SETSAR,
		SETUSD,
	];
	pub const SetterCurrencyId: CurrencyId = SETR;  // Setter  currency ticker is SETR/
	pub const GetSetUSDCurrencyId: CurrencyId = SETUSD;  // Setter  currency ticker is SETUSD/
	pub const DirhamCurrencyId: CurrencyId = DRAM; // SettinDEX currency ticker is DRAM/

	pub const SerpTreasuryPalletId: PalletId = PalletId(*b"set/serp");
	pub const CharityFundAccountId: AccountId = CHARITY_FUND;
	pub const SettPayTreasuryAccountId: AccountId = SETRPAY;
	pub const CashDropVaultAccountId: AccountId = VAULT;

	pub CashDropPeriod: BlockNumber = 120; // Triggers SERP-TES for serping after Every 60 blocks
	pub MaxSlippageSwapWithDEX: Ratio = Ratio::one();
}

parameter_type_with_key! {
	pub MinimumClaimableTransferAmounts: |currency_id: CurrencyId| -> Balance {
		match currency_id {
			&SETR => 2,
			&SETCHF => 2,
			&SETEUR => 2,
			&SETGBP => 2,
			&SETUSD => 2,
			&SETSAR => 2,
			_ => 0,
		}
	};
}

parameter_type_with_key! {
	pub GetStableCurrencyMinimumSupply: |currency_id: CurrencyId| -> Balance {
		match currency_id {
			&SETR => 10_000,
			&SETCHF => 10_000,
			&SETEUR => 10_000,
			&SETGBP => 10_000,
			&SETUSD => 10_000,
			&SETSAR => 10_000,
			_ => 0,
		}
	};
}

parameter_types! {
	pub MaxSwapSlippageCompareToOracle: Ratio = Ratio::saturating_from_rational(1, 2);
	pub DefaultFeeSwapPathList: Vec<Vec<CurrencyId>> = vec![vec![SETR, DNAR], vec![SETUSD, SETR, DNAR]];
}

ord_parameter_types! {
	pub const Root: AccountId = ROOT;
}

impl serp_treasury::Config for Runtime {
	type Event = Event;
	type Currency = Currencies;
	type StableCurrencyIds = StableCurrencyIds;
	type GetStableCurrencyMinimumSupply = GetStableCurrencyMinimumSupply;
	type GetNativeCurrencyId = GetNativeCurrencyId;
	type SetterCurrencyId = SetterCurrencyId;
	type GetSetUSDCurrencyId = GetSetUSDCurrencyId;
	type DirhamCurrencyId = DirhamCurrencyId;
	type CashDropPeriod = CashDropPeriod;
	type SettPayTreasuryAccountId = SettPayTreasuryAccountId;
	type CashDropVaultAccountId = CashDropVaultAccountId;
	type CharityFundAccountId = CharityFundAccountId;
	type DefaultSwapPathList = DefaultFeeSwapPathList;
	type Dex = SetheumDEX;
	type MaxSwapSlippageCompareToOracle = MaxSwapSlippageCompareToOracle;
	type TradingPathLimit = TradingPathLimit;
	type PriceSource = MockPriceSource;
	type MinimumClaimableTransferAmounts = MinimumClaimableTransferAmounts;
	type UpdateOrigin = EnsureSignedBy<Root, AccountId>;
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
		SETCHF,
		SETEUR,
		SETGBP,
		SETSAR,
		SETUSD,
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
				(ALICE, SETUSD, 1000),
				(ALICE, SETEUR, 1000),
				(ALICE, SETCHF, 1000),
				(BOB, SETR, 1000),
				(BOB, SETUSD, 1000),
				(BOB, SETEUR, 1000),
				(BOB, SETCHF, 1000),
				(SETRPAY, SETR, 1000),
				(SETRPAY, SETUSD, 1000),
				(SETRPAY, SETEUR, 1000),
				(SETRPAY, SETCHF, 1000),
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
