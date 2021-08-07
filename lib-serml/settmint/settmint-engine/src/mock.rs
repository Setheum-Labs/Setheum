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

//! Mocks for the settmint engine module.

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

pub type AccountId = u128;
pub type BlockNumber = u64;
pub type AuctionId = u32;

pub const ALICE: AccountId = 1;
pub const BOB: AccountId = 2;
pub const CAROL: AccountId = 3;
pub const CHARITY_FUND: AccountId = 4;

// Currencies constants - CurrencyId/TokenSymbol
pub const DNAR: CurrencyId = CurrencyId::Token(TokenSymbol::DNAR);
pub const DRAM: CurrencyId = CurrencyId::Token(TokenSymbol::DRAM);
pub const SETT: CurrencyId = CurrencyId::Token(TokenSymbol::SETT);
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


mod settmint_engine {
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
	pub const SettmintManagerPalletId: PalletId = PalletId(*b"set/mint");
}

impl settmint_manager::Config for Runtime {
	type Event = Event;
	type Convert = StandardExchangeRateConvertor<Runtime>;
	type Currency = Currencies;
	type StandardValidator = SettmintEngineModule;
	type SerpTreasury = SerpTreasuryModule;
	type PalletId = SettmintManagerPalletId;
	type OnUpdateSettMint = ();
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
	fn get_peg_currency_by_currency_id(_currency_id: CurrencyId) -> CurrencyId {
		Default::default()
	}

	fn get_peg_price(_currency_id: CurrencyId) -> Option<Price> {
		Some(Price::one())
	}

	fn get_fiat_price(_currency_id: CurrencyId) -> Option<Price> {
		Some(Price::one())
	}

	fn get_fiat_usd_fixed_price() -> Option<Price> {
		Some(Price::one())
	}

	fn get_settusd_fixed_price() -> Option<Price> {
		Some(Price::one())
	}

	fn get_stablecoin_fixed_price(_currency_id: CurrencyId) -> Option<Price> {
		Some(Price::one())
	}

	fn get_stablecoin_market_price(_currency_id: CurrencyId) -> Option<Price> {
		Some(Price::one())
	}

	fn get_relative_price(base: CurrencyId, quote: CurrencyId) -> Option<Price> {
		match (base, quote) {
			(USDJ, SETT) => RELATIVE_PRICE.with(|v| *v.borrow_mut()),
			(SETT, USDJ) => RELATIVE_PRICE.with(|v| *v.borrow_mut()),
			_ => None,
		}
	}

	fn get_coin_to_peg_relative_price(_currency_id: CurrencyId) -> Option<Price> {
		Some(Price::one())
	}

	fn get_setter_basket_peg_price() -> Option<Price> {
		Some(Price::one())
	}

	fn get_setter_fixed_price() -> Option<Price> {
		Some(Price::one())
	}

	fn get_market_price(_currency_id: CurrencyId) -> Option<Price> {
		Some(Price::one())
	}

	fn get_price(_currency_id: CurrencyId) -> Option<Price> {
		Some(Price::one())
	}

	fn lock_price(_currency_id: CurrencyId) {}

	fn unlock_price(_currency_id: CurrencyId) {}
}

pub struct MockSerpAuctionManager;
impl SerpAuctionManager<AccountId> for MockSerpAuctionManager {
	type Balance = Balance;
	type CurrencyId = CurrencyId;
	type AuctionId = AuctionId;

	fn new_diamond_auction(_amount: Self::Balance, _fix: Self::Balance) -> DispatchResult {
		Ok(())
	}

	fn new_setter_auction(_amount: Self::Balance, _fix: Self::Balance, _id: Self::CurrencyId) -> DispatchResult {
		Ok(())
	}

	fn new_serplus_auction(_amount: Self::Balance, _id: Self::CurrencyId) -> DispatchResult {
		Ok(())
	}

	fn cancel_auction(_id: Self::AuctionId) -> DispatchResult {
		Ok(())
	}

	fn get_total_serplus_in_auction(_id: Self::CurrencyId) -> Self::Balance {
		Default::default()
	}

	fn get_total_settcurrency_in_auction(_id: Self::CurrencyId) -> Self::Balance {
		Default::default()
	}

	fn get_total_setter_in_auction() -> Self::Balance {
		Default::default()
	}
}

parameter_types! {
	
	pub StableCurrencyIds: Vec<CurrencyId> = vec![
		SETT, AUDJ, CADJ, CHFJ, EURJ, GBPJ,
		JPYJ, SARJ, SEKJ, SGDJ, USDJ,
	];
	pub const SetterCurrencyId: CurrencyId = SETT;  // Setter  currency ticker is SETT/NSETT
	pub const DirhamCurrencyId: CurrencyId = DRAM; // SettinDEX currency ticker is DRAM/MENA
	pub const GetDexerMaxSupply: Balance = 200_000; // SettinDEX currency ticker is DRAM/MENA

	pub const SerpTreasuryPalletId: PalletId = PalletId(*b"set/serp");
	pub const TreasuryPalletId: PalletId = PalletId(*b"set/trsy");
	pub const SettPayTreasuryPalletId: PalletId = PalletId(*b"set/stpy");
	
	pub SerpTesSchedule: BlockNumber = 60; // Triggers SERP-TES for serping after Every 60 blocks
	pub BuybackSerpupRatio: Permill = Permill::from_percent(10); // 10% of SerpUp to buy back & burn NativeCurrency.
	pub SettPaySerpupRatio: Permill = Permill::from_percent(60); // 60% of SerpUp to SettPay as Cashdrops.
	pub SetheumTreasurySerpupRatio: Permill = Permill::from_percent(10); // 10% of SerpUp to network Treasury.
	pub CharityFundSerpupRatio: Permill = Permill::from_percent(20); // 20% of SerpUp to Setheum Foundation's Charity Fund.
}

parameter_type_with_key! {
	pub GetStableCurrencyMinimumSupply: |currency_id: CurrencyId| -> Balance {
		match currency_id {
			&SETT => 10_000,
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
	type DirhamCurrencyId = DirhamCurrencyId;
	type GetDexerMaxSupply = GetDexerMaxSupply;
	type SerpTesSchedule = SerpTesSchedule;
	type BuybackSerpupRatio = BuybackSerpupRatio;
	type SettPaySerpupRatio = SettPaySerpupRatio;
	type SetheumTreasurySerpupRatio = SetheumTreasurySerpupRatio;
	type CharityFundSerpupRatio = CharityFundSerpupRatio;
	type SettPayTreasuryAcc = SettPayTreasuryPalletId;
	type SetheumTreasuryAcc = TreasuryPalletId;
	type CharityFundAcc = CHARITY_FUND;
	type SerpAuctionManagerHandler = MockSerpAuctionManager;
	type UpdateOrigin = EnsureSignedBy<One, AccountId>;
	type Dex = SetheumDEX;
	type MaxAuctionsCount = MaxAuctionsCount;
	type PalletId = SerpTreasuryPalletId;
	type WeightInfo = ();
}

parameter_types! {
	pub const DexPalletId: PalletId = PalletId(*b"set/sdex");
	pub const TradingPathLimit: u32 = 3;
	pub EnabledTradingPairs : Vec<TradingPair> = vec![TradingPair::new(USDJ, SETT), TradingPair::new(USDJ, EURJ)];
}

impl dex::Config for Runtime {
	type Event = Event;
	type Currency = Currencies;
	type TradingPathLimit = TradingPathLimit;
	type PalletId = DexPalletId;
	type DEXIncentives = ();
	type WeightInfo = ();
	type UpdateOrigin = EnsureSignedBy<One, AccountId>;
	type ListingOrigin = EnsureSignedBy<One, AccountId>;
}

ord_parameter_types! {
	pub const One: AccountId = 1;
}

parameter_types! {
	pub StandardCurrencyIds: Vec<CurrencyId> = vec![
		AEDJ, AUDJ, BRLJ, CADJ, CHFJ, CLPJ, CNYJ, COPJ, EURJ, GBPJ,
		HKDJ, HUFJ, IDRJ, JPYJ, KESJ, KRWJ, KZTJ, MXNJ, MYRJ, NGNJ,
		NOKJ, NZDJ, PENJ, PHPJ, PKRJ, PLNJ, QARJ, RONJ, RUBJ, SARJ, 
		SEKJ, SGDJ, THBJ, TRYJ, TWDJ, TZSJ, USDJ, ZARJ,
	];
	pub const GetReserveCurrencyId: CurrencyId = SETT;
	pub DefaultStandardExchangeRate: ExchangeRate = ExchangeRate::one();
	pub const MinimumStandardValue: Balance = 2;
	pub const UnsignedPriority: u64 = 1 << 20;
}

impl Config for Runtime {
	type Event = Event;
	type PriceSource = MockPriceSource;
	type GetReserveCurrencyId = GetReserveCurrencyId;
	type StandardCurrencyIds = StandardCurrencyIds;
	type DefaultStandardExchangeRate = DefaultStandardExchangeRate;
	type MinimumStandardValue = MinimumStandardValue;
	type GetStableCurrencyId = GetStableCurrencyId;
	type SerpTreasury = SerpTreasuryModule;
	type UpdateOrigin = EnsureSignedBy<One, AccountId>;
	type Dex = SetheumDEX;
	type UnsignedPriority = UnsignedPriority;
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
		SettmintEngineModule: settmint_engine::{Pallet, Storage, Call, Event<T>, Config, ValidateUnsigned},
		SerpTreasuryModule: serp_treasury::{Pallet, Storage, Call, Config, Event<T>},
		Currencies: orml_currencies::{Pallet, Call, Event<T>},
		Tokens: orml_tokens::{Pallet, Storage, Event<T>, Config<T>},
		SettmintManagerModule: settmint_manager::{Pallet, Storage, Call, Event<T>},
		PalletBalances: pallet_balances::{Pallet, Call, Storage, Event<T>},
		SetheumDEX: dex::{Pallet, Storage, Call, Event<T>, Config<T>},
	}
);

/// An extrinsic type used for tests.
pub type Extrinsic = TestXt<Call, ()>;

impl<LocalCall> SendTransactionTypes<LocalCall> for Runtime
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
				(ALICE, SETT, 1000),
				(BOB, SETT, 1000),
				(CAROL, SETT, 100),
				(ALICE, EURJ, 1000),
				(BOB, EURJ, 1000),
				(CAROL, USDJ, 1000),
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
