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

//! Mocks for the SERP Treasury module.

#![cfg(test)]

use super::*;
use frame_support::{construct_runtime, ord_parameter_types, parameter_types, PalletId};
use frame_system::EnsureSignedBy;
use orml_traits::parameter_type_with_key;
use primitives::{Amount, TokenSymbol, TradingPair};
use sp_core::H256;
use sp_runtime::{Permill, testing::Header, traits::IdentityLookup};
use sp_std::cell::RefCell;

pub type AccountId = u128;
pub type BlockNumber = u64;
pub type Amount = i64;
pub type AuctionId = u32;

pub const ALICE: AccountId = 0;
pub const BOB: AccountId = 1;
pub const CHARITY_FUND: AccountId = 2;

// Currencies constants - CurrencyId/TokenSymbol
pub const DNAR: CurrencyId = CurrencyId::Token(TokenSymbol::DNAR);
pub const SDEX: CurrencyId = CurrencyId::Token(TokenSymbol::SDEX);
pub const SETT: CurrencyId = CurrencyId::Token(TokenSymbol::SETT);
pub const AEDJ: CurrencyId = CurrencyId::Token(TokenSymbol::AEDJ);
pub const ARSJ: CurrencyId = CurrencyId::Token(TokenSymbol::ARSJ);
pub const AUDJ: CurrencyId = CurrencyId::Token(TokenSymbol::AUDJ);
pub const BRLJ: CurrencyId = CurrencyId::Token(TokenSymbol::BRLJ);
pub const CADJ: CurrencyId = CurrencyId::Token(TokenSymbol::CADJ);
pub const CHFJ: CurrencyId = CurrencyId::Token(TokenSymbol::CHFJ);
pub const CLPJ: CurrencyId = CurrencyId::Token(TokenSymbol::CLPJ);
pub const CNYJ: CurrencyId = CurrencyId::Token(TokenSymbol::CNYJ);
pub const COPJ: CurrencyId = CurrencyId::Token(TokenSymbol::COPJ);
pub const EURJ: CurrencyId = CurrencyId::Token(TokenSymbol::EURJ);
pub const GBPJ: CurrencyId = CurrencyId::Token(TokenSymbol::GBPJ);
pub const HKDJ: CurrencyId = CurrencyId::Token(TokenSymbol::HKDJ);
pub const HUFJ: CurrencyId = CurrencyId::Token(TokenSymbol::HUFJ);
pub const IDRJ: CurrencyId = CurrencyId::Token(TokenSymbol::IDRJ);
pub const JPYJ: CurrencyId = CurrencyId::Token(TokenSymbol::JPYJ);
pub const KESJ: CurrencyId = CurrencyId::Token(TokenSymbol::KESJ);
pub const KRWJ: CurrencyId = CurrencyId::Token(TokenSymbol::KRWJ);
pub const KZTJ: CurrencyId = CurrencyId::Token(TokenSymbol::KZTJ);
pub const MXNJ: CurrencyId = CurrencyId::Token(TokenSymbol::MXNJ);
pub const MYRJ: CurrencyId = CurrencyId::Token(TokenSymbol::MYRJ);
pub const NGNJ: CurrencyId = CurrencyId::Token(TokenSymbol::NGNJ);
pub const NOKJ: CurrencyId = CurrencyId::Token(TokenSymbol::NOKJ);
pub const NZDJ: CurrencyId = CurrencyId::Token(TokenSymbol::NZDJ);
pub const PENJ: CurrencyId = CurrencyId::Token(TokenSymbol::PENJ);
pub const PHPJ: CurrencyId = CurrencyId::Token(TokenSymbol::PHPJ);
pub const PKRJ: CurrencyId = CurrencyId::Token(TokenSymbol::PKRJ);
pub const PLNJ: CurrencyId = CurrencyId::Token(TokenSymbol::PLNJ);
pub const QARJ: CurrencyId = CurrencyId::Token(TokenSymbol::QARJ);
pub const RONJ: CurrencyId = CurrencyId::Token(TokenSymbol::RONJ);
pub const RUBJ: CurrencyId = CurrencyId::Token(TokenSymbol::RUBJ);
pub const SARJ: CurrencyId = CurrencyId::Token(TokenSymbol::SARJ);
pub const SEKJ: CurrencyId = CurrencyId::Token(TokenSymbol::SEKJ);
pub const SGDJ: CurrencyId = CurrencyId::Token(TokenSymbol::SGDJ);
pub const THBJ: CurrencyId = CurrencyId::Token(TokenSymbol::THBJ);
pub const TRYJ: CurrencyId = CurrencyId::Token(TokenSymbol::TRYJ);
pub const TWDJ: CurrencyId = CurrencyId::Token(TokenSymbol::TWDJ);
pub const TZSJ: CurrencyId = CurrencyId::Token(TokenSymbol::TZSJ);
pub const UAHJ: CurrencyId = CurrencyId::Token(TokenSymbol::UAHJ);
pub const USDJ: CurrencyId = CurrencyId::Token(TokenSymbol::USDJ);
pub const ZARJ: CurrencyId = CurrencyId::Token(TokenSymbol::ZARJ);

mod serp_treasury {
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
}

impl pallet_balances::Config for Runtime {
	type Balance = Balance;
	type DustRemoval = ();
	type Event = Event;
	type ExistentialDeposit = ExistentialDeposit;
	type AccountStore = frame_system::Module<Runtime>;
	type MaxLocks = ();
	type WeightInfo = ();
}
pub type AdaptedBasicCurrency = orml_currencies::BasicCurrencyAdapter<Runtime, PalletBalances, Amount, BlockNumber>;

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

parameter_types! {
	pub const DexPalletId: PalletId = PalletId(*b"set/dexm");
	pub const GetExchangeFee: (u32, u32) = (0, 100);
	pub const TradingPathLimit: u32 = 3;
	pub EnabledTradingPairs: Vec<TradingPair> = vec![TradingPair::new(USDJ, SETT)];
}

impl dex::Config for Runtime {
	type Event = Event;
	type Currency = Currencies;
	type GetExchangeFee = GetExchangeFee;
	type TradingPathLimit = TradingPathLimit;
	type PalletId = DexPalletId;
	type DEXIncentives = ();
	type WeightInfo = ();
	type ListingOrigin = EnsureSignedBy<One, AccountId>;
}

thread_local! {
	pub static TOTAL_SETTER_IN_AUCTION: RefCell<u32> = RefCell::new(0);
	pub static TOTAL_RESERVE_IN_AUCTION: RefCell<Balance> = RefCell::new(0);
	pub static TOTAL_DIAMOND_IN_AUCTION: RefCell<u32> = RefCell::new(0);
	pub static TOTAL_SERPLUS_IN_AUCTION: RefCell<u32> = RefCell::new(0);
}

pub struct MockSerpAuctionManager;
impl SerpAuctionManager<AccountId> for MockSerpAuctionManager {
	type CurrencyId = CurrencyId;
	type Balance = Balance;
	type AuctionId = AuctionId;

	fn new_diamond_auction(_amount: Self::Balance, _fix: Self::Balance) -> DispatchResult {
		TOTAL_SETTER_IN_AUCTION.with(|v| *v.borrow_mut() += 1);
		Ok(())
	}

	fn new_setter_auction(_amount: Self::Balance, _fix: Self::Balance, _currency: Self::CurrencyId) -> DispatchResult {
		TOTAL_SETT_CURRENCY_IN_AUCTION.with(|v| *v.borrow_mut() += 1); // TotalSettCurrencyInAuction
		Ok(())
	}

	fn new_serplus_auction(_amount: Self::Balance, _currency: Self::CurrencyId) -> DispatchResult {
		TOTAL_SERPLUS_IN_AUCTION.with(|v| *v.borrow_mut() += 1);
		Ok(())
	}

	fn cancel_auction(_id: Self::AuctionId) -> DispatchResult {
		Ok(())
	}

	fn get_total_serplus_in_auction(_currency: Self::CurrencyId) -> Self::Balance {
		Default::default()
	}

	fn get_total_settcurrency_in_auction(_currency: Self::CurrencyId) -> Self::Balance {
		Default::default()
	}

	fn get_total_setter_in_auction(_currency: Self::CurrencyId) -> Self::Balance {
		Default::default()
	}
}

ord_parameter_types! {
	pub const One: AccountId = 1;
	pub const MaxAuctionsCount: u32 = 5;
}

parameter_types! {
	pub StableCurrencyIds: Vec<CurrencyId> = vec![
		SETT,
		AEDJ,
		ARSJ,
 		AUDJ,
		BRLJ,
		CADJ,
		CHFJ,
		CLPJ,
		CNYJ,
		COPJ,
		EURJ,
		GBPJ,
		HKDJ,
		HUFJ,
		IDRJ,
		JPYJ,
 		KESJ,
 		KRWJ,
 		KZTJ,
		MXNJ,
		MYRJ,
 		NGNJ,
		NOKJ,
		NZDJ,
		PENJ,
		PHPJ,
 		PKRJ,
		PLNJ,
		QARJ,
		RONJ,
		RUBJ,
 		SARJ,
 		SEKJ,
 		SGDJ,
		THBJ,
		TRYJ,
		TWDJ,
		TZSJ,
		UAHJ,
		USDJ,
		ZARJ,
	];
	pub const GetSetterCurrencyId: CurrencyId = SETT;  // Setter  currency ticker is SETT
	pub const GetDexerCurrencyId: CurrencyId = SDEX; // SettinDEX currency ticker is SDEX

	pub const SerpTreasuryPalletId: PalletId = PalletId(*b"set/settmintt");
	pub const SetheumTreasuryPalletId: PalletId = PalletId(*b"set/treasury");
	pub const SettPayTreasuryPalletId: PalletId = PalletId(*b"set/settpay");
	
	pub SerpTesSchedule: BlockNumber = 60; // Triggers SERP-TES for serping after Every 60 blocks
	pub SerplusSerpupRatio: Permill = Permill::from_percent(10); // 10% of SerpUp to buy back & burn NativeCurrency.
	pub SettPaySerpupRatio: Permill = Permill::from_percent(60); // 60% of SerpUp to SettPay as Cashdrops.
	pub SetheumTreasurySerpupRatio: Permill = Permill::from_percent(10); // 10% of SerpUp to network Treasury.
	pub CharityFundSerpupRatio: Permill = Permill::from_percent(20); // 20% of SerpUp to Setheum Foundation's Charity Fund.
}

impl Config for Runtime {
	type Event = Event;
	type Currency = Currencies;
	type StableCurrencyIds = StableCurrencyIds;
	type GetSetterCurrencyId = GetSetterCurrencyId;
	type GetDexerCurrencyId = GetDexerCurrencyId;
	type SerpTesSchedule = SerpTesSchedule;
	type SerplusSerpupRatio = SerplusSerpupRatio;
	type SettPaySerpupRatio = SettPaySerpupRatio;
	type SetheumTreasurySerpupRatio = SetheumTreasurySerpupRatio;
	type CharityFundSerpupRatio = CharityFundSerpupRatio;
	type SettPayTreasuryAcc = SettPayTreasuryPalletId;
	type SetheumTreasuryAcc = SetheumTreasuryPalletId;
	type CharityFundAcc = CHARITY_FUND;
	type SerpAuctionManagerHandler = MockSerpAuctionManager;
	type UpdateOrigin = EnsureSignedBy<One, AccountId>;
	type Dex = SetheumDEX;
	type MaxAuctionsCount = MaxAuctionsCount;
	type PalletId = SerpTreasuryPalletId;
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
		System: frame_system::{Module, Call, Storage, Config, Event<T>},
		SerpTreasuryModule: serp_treasury::{Module, Storage, Call, Config, Event<T>},
		Currencies: orml_currencies::{Module, Call, Event<T>},
		Tokens: orml_tokens::{Module, Storage, Event<T>, Config<T>},
		PalletBalances: pallet_balances::{Module, Call, Storage, Event<T>},
		SetheumDEX: dex::{Module, Storage, Call, Event<T>, Config<T>},
	}
);

pub struct ExtBuilder {
	endowed_accounts: Vec<(AccountId, CurrencyId, Balance)>,
}

impl Default for ExtBuilder {
	fn default() -> Self {
		Self {
			endowed_accounts: vec![
				(ALICE, USDJ, 1000),
				(ALICE, SETT, 1000),
				(BOB, USDJ, 1000),
				(BOB, SETT, 1000),
				(CHARITY_FUND, USDJ, 1000),
				(CHARITY_FUND, SETT, 1000),
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
			endowed_accounts: self.endowed_accounts,
		}
		.assimilate_storage(&mut t)
		.unwrap();

		dex::GenesisConfig::<Runtime> {
			initial_listing_trading_pairs: vec![],
			initial_enabled_trading_pairs: EnabledTradingPairs::get(),
			initial_added_liquidity_pools: vec![],
		}
		.assimilate_storage(&mut t)
		.unwrap();

		t.into()
	}
}
