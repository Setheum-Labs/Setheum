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

//! Mocks for the serp auction module.

#![cfg(test)]

use super::*;
use frame_support::{construct_runtime, ord_parameter_types, parameter_types, PalletId};
use frame_system::EnsureSignedBy;
use orml_traits::parameter_type_with_key;
use primitives::{TokenSymbol, TradingPair};
use sp_core::H256;
use sp_runtime::{
	testing::{Header, TestXt},
	traits::IdentityLookup,
};
use sp_std::cell::RefCell;
pub use support::Price;

pub type AccountId = u128;
pub type BlockNumber = u64;
pub type AuctionId = u32;
pub type Amount = i64;

pub const ALICE: AccountId = 1;
pub const BOB: AccountId = 2;
pub const CAROL: AccountId = 3;

// Currencies constants - CurrencyId/TokenSymbol
pub const DNAR: CurrencyId = CurrencyId::Token(TokenSymbol::DNAR);
pub const DRAM: CurrencyId = CurrencyId::Token(TokenSymbol::DRAM);
pub const SETT: CurrencyId = CurrencyId::Token(TokenSymbol::SETT);
pub const AEDJ: CurrencyId = CurrencyId::Token(TokenSymbol::AEDJ);
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
pub const USDJ: CurrencyId = CurrencyId::Token(TokenSymbol::USDJ);
pub const ZARJ: CurrencyId = CurrencyId::Token(TokenSymbol::ZARJ);

mod serp_auction {
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

impl orml_auction::Config for Runtime {
	type Event = Event;
	type Balance = Balance;
	type AuctionId = AuctionId;
	type Handler = SerpAuctionManagerModule;
	type WeightInfo = ();
}

ord_parameter_types! {
	pub const One: AccountId = 1;
}

parameter_types! {
	pub StableCurrencyIds: Vec<CurrencyId> = vec![
		SETT,
		AEDJ,
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
		USDJ,
		ZARJ,
	];
	pub const GetSetterCurrencyId: CurrencyId = SETT;  // Setter  currency ticker is SETT
	pub const GetDexerCurrencyId: CurrencyId = DRAM; // SettinDEX currency ticker is DRAM

	pub const SerpTreasuryPalletId: PalletId = PalletId(*b"set/serp");
	pub SerpTesSchedule: BlockNumber = 60; // Triggers SERP-TES for serping after Every 60 blocks
	pub SerplusSerpupRatio: Permill = Permill::from_percent(10); // 10% of SerpUp to buy back & burn NativeCurrency.
	pub SettPaySerpupRatio: Permill = Permill::from_percent(60); // 60% of SerpUp to SettPay as Cashdrops.
	pub SetheumTreasurySerpupRatio: Permill = Permill::from_percent(10); // 10% of SerpUp to network Treasury.
	pub CharityFundSerpupRatio: Permill = Permill::from_percent(20); // 20% of SerpUp to Setheum Foundation's Charity Fund.
}

impl serp_treasury::Config for Runtime {
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
	type SerpAuctionManagerHandler = MockSerpAuctionManager;
	type UpdateOrigin = EnsureSignedBy<One, AccountId>;
	type Dex = SetheumDEX;
	type MaxAuctionsCount = MaxAuctionsCount;
	type PalletId = SerpTreasuryPalletId;
	type WeightInfo = ();
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

	fn get_price(_currency_id: CurrencyId) -> Option<Price> {
		None
	}

	fn lock_price(_currency_id: CurrencyId) {}

	fn unlock_price(_currency_id: CurrencyId) {}
}

parameter_types! {
	pub const DexPalletId: PalletId = PalletId(*b"set/sdex");
	pub const TradingPathLimit: u32 = 3;
	pub EnabledTradingPairs : Vec<TradingPair> = vec![TradingPair::new(USDJ, CHFJ)];
}

impl dex::Config for Runtime {
	type Event = Event;
	type Currency = Tokens;
	type TradingPathLimit = TradingPathLimit;
	type PalletId = DexPalletId;
	type DEXIncentives = ();
	type WeightInfo = ();
	type UpdateOrigin = EnsureSignedBy<One, AccountId>;
	type ListingOrigin = EnsureSignedBy<One, AccountId>;
}

parameter_types! {
	pub DiamondAuctionMinimumIncrementSize: Rate = Rate::saturating_from_rational(3 : 100); // 3% increment
	pub SetterAuctionMinimumIncrementSize: Rate = Rate::saturating_from_rational(1 : 50); // 2% increment
	pub SerplusAuctionMinimumIncrementSize: Rate = Rate::saturating_from_rational(1, 100); // 1% increment
	pub const AuctionTimeToClose: u64 = 100;
	pub const AuctionDurationSoftCap: u64 = 2000;
	pub const MaxAuctionsCount: u32 = 10_000;

	pub const GetNativeCurrencyId: CurrencyId = DNAR;
	pub const GetSetterCurrencyId: CurrencyId = SETT;
}

impl Config for Runtime {
	type Event = Event;
	type Currency = Tokens;
	type Auction = AuctionModule;
	type DiamondAuctionMinimumIncrementSize = DiamondAuctionMinimumIncrementSize;
	type SetterAuctionMinimumIncrementSize = SetterAuctionMinimumIncrementSize;
	type SerplusAuctionMinimumIncrementSize = SerplusAuctionMinimumIncrementSize;
	type AuctionTimeToClose = AuctionTimeToClose;
	type AuctionDurationSoftCap = AuctionDurationSoftCap;
	type StableCurrencyIds = StableCurrencyIds;
	type GetNativeCurrencyId = GetNativeCurrencyId;
	type GetSetterCurrencyId = GetSetterCurrencyId;
	type SerpTreasury = SerpTreasuryModule;
	type Dex = SetheumDEX;
	type PriceSource = MockPriceSource;
	type UnsignedPriority = UnsignedPriority;
	type WeightInfo = ();
}

pub type Block = sp_runtime::generic::Block<Header, UncheckedExtrinsic>;
pub type UncheckedExtrinsic = sp_runtime::generic::UncheckedExtrinsic<u32, Call, u32, ()>;

construct_runtime!(
	pub enum Runtime where
		Block = Block,
		NodeBlock = Block,
		UncheckedExtrinsic = UncheckedExtrinsic,
	{
		System: frame_system::{Module, Call, Storage, Config, Event<T>},
		SerpAuctionManagerModule: serp_auction::{Module, Storage, Call, Event<T>, ValidateUnsigned},
		Tokens: orml_tokens::{Module, Storage, Event<T>, Config<T>},
		AuctionModule: orml_auction::{Module, Storage, Call, Event<T>},
		SerpTreasuryModule: serp_treasury::{Module, Storage, Call, Event<T>},
		SetheumDEX: dex::{Module, Storage, Call, Event<T>, Config<T>},
	}
);

pub type Extrinsic = TestXt<Call, ()>;

impl<LocalCall> frame_system::offchain::SendTransactionTypes<LocalCall> for Runtime
where
	Call: From<LocalCall>,
{
	type OverarchingCall = Call;
	type Extrinsic = Extrinsic;
}

pub struct ExtBuilder {
	endowed_accounts: Vec<(AccountId, CurrencyId, Balance)>,
}

impl Default for ExtBuilder {
	fn default() -> Self {
		Self {
			endowed_accounts: vec![
				(ALICE, USDJ, 1000),
				(BOB, USDJ, 1000),
				(CAROL, USDJ, 1000),
				(ALICE, CHFJ, 1000),
				(BOB, CHFJ, 1000),
				(CAROL, CHFJ, 1000),
				(ALICE, DNAR, 1000),
				(BOB, DNAR, 1000),
				(CAROL, DNAR, 1000),
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
