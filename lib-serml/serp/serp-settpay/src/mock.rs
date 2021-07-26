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

//! Mocks for the serp_settpay module.

#![cfg(test)]

use super::*;
use frame_support::{construct_runtime, ord_parameter_types, parameter_types, PalletId};
use frame_system::{offchain::SendTransactionTypes, EnsureSignedBy};
use orml_traits::{parameter_type_with_key, DataFeeder};
use primitives::{Amount, Balance, Moment, TokenSymbol};
use sp_core::{H160, H256};
use sp_runtime::{
	testing::{Header, TestXt},
	traits::{AccountIdConversion, IdentityLookup, One as OneT},
	DispatchError, FixedPointNumber,
};
use sp_std::cell::RefCell;
use support::{ExchangeRate, MockCurrencyIdMapping, CashDropRate, Price, PriceProvider, Rate, Ratio,  SerpAuctionManager};

mod serp_settpay {
	pub use super::super::*;
}

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


// LP tokens constants - CurrencyId/TokenSymbol : Dex Shares
pub const LP_CHFJ_USDJ: CurrencyId =
CurrencyId::DexShare(DexShare::Token(TokenSymbol::CHFJ), DexShare::Token(TokenSymbol::USDJ));
pub const LP_USDJ_DNAR: CurrencyId =
CurrencyId::DexShare(DexShare::Token(TokenSymbol::USDJ), DexShare::Token(TokenSymbol::DNAR));

// Currencies constants - FiatCurrencyIds (CurrencyId/TokenSymbol)
pub const BRL: CurrencyId = CurrencyId::Token(TokenSymbol::BRL);
pub const CAD: CurrencyId = CurrencyId::Token(TokenSymbol::CAD);
pub const CHF: CurrencyId = CurrencyId::Token(TokenSymbol::CHF);
pub const EUR: CurrencyId = CurrencyId::Token(TokenSymbol::EUR);
pub const GBP: CurrencyId = CurrencyId::Token(TokenSymbol::GBP);
pub const JPY: CurrencyId = CurrencyId::Token(TokenSymbol::JPY);
pub const QAR: CurrencyId = CurrencyId::Token(TokenSymbol::QAR);
pub const SAR: CurrencyId = CurrencyId::Token(TokenSymbol::SAR);
pub const SEK: CurrencyId = CurrencyId::Token(TokenSymbol::SEK);
pub const SGD: CurrencyId = CurrencyId::Token(TokenSymbol::SGD);
pub const USD: CurrencyId = CurrencyId::Token(TokenSymbol::USD);
pub const KWD: CurrencyId = CurrencyId::Token(TokenSymbol::KWD);
pub const JOD: CurrencyId = CurrencyId::Token(TokenSymbol::JOD);
pub const BHD: CurrencyId = CurrencyId::Token(TokenSymbol::BHD);
pub const KYD: CurrencyId = CurrencyId::Token(TokenSymbol::KYD);
pub const OMR: CurrencyId = CurrencyId::Token(TokenSymbol::OMR);
pub const GIP: CurrencyId = CurrencyId::Token(TokenSymbol::GIP);

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

pub struct MockDataProvider;
impl DataProvider<CurrencyId, Price> for MockDataProvider {
	fn get(currency_id: &CurrencyId) -> Option<Price> {
		match *currency_id {
			USDJ => Some(Price::saturating_from_rational(99, 100)),
			CHFJ => Some(Price::saturating_from_integer(50000)),
			DNAR => Some(Price::saturating_from_integer(100)),
			DNAR => Some(Price::zero()),
			_ => None,
		}
	}
}

impl DataFeeder<CurrencyId, Price, AccountId> for MockDataProvider {
	fn feed_value(_: AccountId, _: CurrencyId, _: Price) -> sp_runtime::DispatchResult {
		Ok(())
	}
}

pub struct MockDex;
impl DEXManager<AccountId, CurrencyId, Balance> for MockDex {
	fn get_liquidity_pool(currency_id_a: CurrencyId, currency_id_b: CurrencyId) -> (Balance, Balance) {
		match (currency_id_a, currency_id_b) {
			(USDJ, DNAR) => (10000, 200),
			_ => (0, 0),
		}
	}

	fn get_liquidity_token_address(_currency_id_a: CurrencyId, _currency_id_b: CurrencyId) -> Option<H160> {
		unimplemented!()
	}

	fn get_swap_target_amount(
		_path: &[CurrencyId],
		_supply_amount: Balance,
		_price_impact_limit: Option<Ratio>,
	) -> Option<Balance> {
		unimplemented!()
	}

	fn get_swap_supply_amount(
		_path: &[CurrencyId],
		_target_amount: Balance,
		_price_impact_limit: Option<Ratio>,
	) -> Option<Balance> {
		unimplemented!()
	}

	fn swap_with_exact_supply(
		_who: &AccountId,
		_path: &[CurrencyId],
		_supply_amount: Balance,
		_min_target_amount: Balance,
		_price_impact_limit: Option<Ratio>,
	) -> sp_std::result::Result<Balance, DispatchError> {
		unimplemented!()
	}

	fn swap_with_exact_target(
		_who: &AccountId,
		_path: &[CurrencyId],
		_target_amount: Balance,
		_max_supply_amount: Balance,
		_price_impact_limit: Option<Ratio>,
	) -> sp_std::result::Result<Balance, DispatchError> {
		unimplemented!()
	}

	fn add_liquidity(
		_who: &AccountId,
		_currency_id_a: CurrencyId,
		_currency_id_b: CurrencyId,
		_max_amount_a: Balance,
		_max_amount_b: Balance,
		_min_share_increment: Balance,
		_deposit_increment_share: bool,
	) -> DispatchResult {
		unimplemented!()
	}

	fn remove_liquidity(
		_who: &AccountId,
		_currency_id_a: CurrencyId,
		_currency_id_b: CurrencyId,
		_remove_share: Balance,
		_min_withdrawn_a: Balance,
		_min_withdrawn_b: Balance,
		_by_withdraw: bool,
	) -> DispatchResult {
		unimplemented!()
	}
}

parameter_type_with_key! {
	pub ExistentialDeposits: |_currency_id: CurrencyId| -> Balance {
		Default::default()
	};
}

parameter_type_with_key! {
	pub PegCurrencyIds: |_currency_id: CurrencyId| -> CurrencyId {
		match currency_id {
			&AUDJ => &AUD,
			&CADJ => &CAD,
			&CHFJ => &CHF,
			&EURJ => &EUR,
			&GBPJ => &GBP,
			&JPYJ => &JPY,
			&SARJ => &SAR,
			&SEKJ => &SEK,
			&SGDJ => &SGD,
			&USDJ => &USD,
			_ => None,
		}
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

ord_parameter_types! {
	pub const One: AccountId = 1;
}

parameter_types! {
	pub const SetterCurrencyId: CurrencyId = SETT; // Setter currency ticker is SETT.
	pub const GetSettUSDCurrencyId: CurrencyId = USDJ; // SettUSD currency ticker is USDJ.
	pub const GetFiatUSDCurrencyId: CurrencyId = USD; // The USD Fiat currency denomination.
	pub FiatUsdFixedPrice: Price = Price::one(); // Fixed 1 USD Fiat denomination for pricing.

	pub const GetSetterPegOneCurrencyId: CurrencyId = GBP; // Fiat pegs of the Setter (SETT).
	pub const GetSetterPegTwoCurrencyId: CurrencyId = EUR; // Fiat pegs of the Setter (SETT).
	pub const GetSetterPegThreeCurrencyId: CurrencyId = KWD; // Fiat pegs of the Setter (SETT).
	pub const GetSetterPegFourCurrencyId: CurrencyId = JOD; // Fiat pegs of the Setter (SETT).
	pub const GetSetterPegFiveCurrencyId: CurrencyId = BHD; // Fiat pegs of the Setter (SETT).
	pub const GetSetterPegSixCurrencyId: CurrencyId = KYD; // Fiat pegs of the Setter (SETT).
	pub const GetSetterPegSevenCurrencyId: CurrencyId = OMR; // Fiat pegs of the Setter (SETT).
	pub const GetSetterPegEightCurrencyId: CurrencyId = CHF; // Fiat pegs of the Setter (SETT).
	pub const GetSetterPegNineCurrencyId: CurrencyId = GIP; // Fiat pegs of the Setter (SETT).
	pub const GetSetterPegTenCurrencyId: CurrencyId = USD; // Fiat pegs of the Setter (SETT).
	
	
	pub StableCurrencyIds: Vec<CurrencyId> = vec![
		SETT, AUDJ, CADJ, CHFJ, EURJ, GBPJ,
		JPYJ, SARJ, SEKJ, SGDJ, USDJ,
	];
	pub FiatCurrencyIds: Vec<CurrencyId> = vec![
		AUD, CAD, CHF, EUR, GBP, JPY, QAR, SAR,
		SEK, SGD, USD, JOD, BHD, KYD, OMR, GIP
	];
}

impl prices::Config for Runtime {
	type Event = Event;
	type Source = MockDataProvider;
	type SetterCurrencyId = SetterCurrencyId;
	type GetSettUSDCurrencyId = GetSettUSDCurrencyId;
	type GetFiatUSDCurrencyId = GetFiatUSDCurrencyId;
	type FiatUsdFixedPrice = FiatUsdFixedPrice;
	type GetSetterPegOneCurrencyId = GetSetterPegOneCurrencyId;
	type GetSetterPegTwoCurrencyId = GetSetterPegTwoCurrencyId;
	type GetSetterPegThreeCurrencyId = GetSetterPegThreeCurrencyId;
	type GetSetterPegFourCurrencyId = GetSetterPegFourCurrencyId;
	type GetSetterPegFiveCurrencyId = GetSetterPegFiveCurrencyId;
	type GetSetterPegSixCurrencyId = GetSetterPegSixCurrencyId;
	type GetSetterPegSevenCurrencyId = GetSetterPegSevenCurrencyId;
	type GetSetterPegEightCurrencyId = GetSetterPegEightCurrencyId;
	type GetSetterPegNineCurrencyId = GetSetterPegNineCurrencyId;
	type GetSetterPegTenCurrencyId = GetSetterPegTenCurrencyId;
	type StableCurrencyIds = StableCurrencyIds;
	type PegCurrencyIds = PegCurrencyIds;
	type FiatCurrencyIds = FiatCurrencyIds;
	type LockOrigin = EnsureSignedBy<One, AccountId>;
	type DEX = MockDEX;
	type Currency = Tokens;
	type CurrencyIdMapping = MockCurrencyIdMapping;
	type WeightInfo = ();
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

pub struct MockPriceSource;
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

	fn get_relative_price(_base: CurrencyId, _quote: CurrencyId) -> Option<Price> {
		Some(Price::one())
	}

	fn get_market_relative_price(_base: CurrencyId, _quote: CurrencyId) -> Option<Price> {
		Some(Price::one())
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

	fn new_setter_auction(_amount: Self::Balance, _fix: Self::Balance, _currency_id: Self::CurrencyId) -> DispatchResult {
		Ok(())
	}

	fn new_serplus_auction(_amount: Self::Balance, _currency_id: Self::CurrencyId) -> DispatchResult {
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

ord_parameter_types! {
	pub const One: AccountId = 1;
}

parameter_types! {
	pub StableCurrencyIds: Vec<CurrencyId> = vec![SETT, USDJ];
	pub const SetterCurrencyId: CurrencyId = SETT;  // Setter  currency ticker is SETT/NSETT
	pub const DexerCurrencyId: CurrencyId = DRAM; // SettinDEX currency ticker is DRAM/MENA
	pub const GetDexerMaxSupply: Balance = 200_000; // SettinDEX currency ticker is DRAM/MENA

	pub const SerpTreasuryPalletId: PalletId = PalletId(*b"set/serp");
	pub const TreasuryPalletId: PalletId = PalletId(*b"set/trsy");
	pub const SettPayTreasuryPalletId: PalletId = PalletId(*b"set/stpy");
	
	pub SerpTesSchedule: BlockNumber = 60; // Triggers SERP-TES for serping after Every 60 blocks
	pub SerplusSerpupRatio: Permill = Permill::from_percent(10); // 10% of SerpUp to buy back & burn NativeCurrency.
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
	type DexerCurrencyId = DexerCurrencyId;
	type GetDexerMaxSupply = GetDexerMaxSupply;
	type SerpTesSchedule = SerpTesSchedule;
	type SerplusSerpupRatio = SerplusSerpupRatio;
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
	pub const MinimumPeriod: Moment = 1000;
}
impl pallet_timestamp::Config for Runtime {
	type Moment = Moment;
	type OnTimestampSet = ();
	type MinimumPeriod = MinimumPeriod;
	type WeightInfo = ();
}

type UncheckedExtrinsic = frame_system::mocking::MockUncheckedExtrinsic<Runtime>;
type Block = frame_system::mocking::MockBlock<Runtime>;

parameter_types! {
	pub RewardableCurrencyIds: Vec<CurrencyId> = vec![DNAR, DRAM, SETT, USDJ];
	pub NonStableDropCurrencyIds: Vec<CurrencyId> = vec![DNAR, DRAM];
	pub SetCurrencyDropCurrencyIds: Vec<CurrencyId> = vec![SETT, USDJ];
	pub const DefaultCashDropRate: CashDropRate = CashDropRate::::saturating_from_rational(2 : 100); // 2% cashdrop
	pub const DefaultMinimumClaimableTransfer: Balance = 10;
	pub const SettPayPalletId: PalletId = PalletId(*b"set/tpay");
}

parameter_type_with_key! {
	pub GetCashDropRates: |currency_id: CurrencyId| -> (Balance, {
		match currency_id {
			&DNAR => (5, 100), // 5% cashdrop.
			&DRAM => (5, 100), // 5% cashdrop.
			&SETT => (5, 100), // 5% cashdrop.
			&AUDJ => (5, 100), // 5% cashdrop.
			&CADJ => (5, 100), // 5% cashdrop.
			&CHFJ => (5, 100), // 5% cashdrop.
			&EURJ => (5, 100), // 5% cashdrop.
			&GBPJ => (5, 100), // 5% cashdrop.
			&JPYJ => (5, 100), // 5% cashdrop.
			&SARJ => (5, 100), // 5% cashdrop.
			&SEKJ => (5, 100), // 5% cashdrop.
			&SGDJ => (5, 100), // 5% cashdrop.
			&USDJ => (5, 100), // 5% cashdrop.
			_ => 0,
		}
	};
}

impl Config for Runtime {
	type Event = Event;
	type Currency = Currencies;
	type SetterCurrencyId = SetterCurrencyId;
	type StableCurrencyIds = StableCurrencyIds;
	type RewardableCurrencyIds = RewardableCurrencyIds;
	type NonStableDropCurrencyIds = NonStableDropCurrencyIds;
	type SetCurrencyDropCurrencyIds = SetCurrencyDropCurrencyIds;
	type DefaultCashDropRate = DefaultCashDropRate;
	type GetCashDropRates = GetCashDropRates;
	type DefaultMinimumClaimableTransfer = DefaultMinimumClaimableTransfer;
	type SerpTreasury = SerpTreasuryModule;
	type UpdateOrigin = EnsureSignedBy<One, AccountId>;
	type PalletId = SettPayPalletId;
	type WeightInfo = ();
}

construct_runtime!(
	pub enum Runtime where
		Block = Block,
		NodeBlock = Block,
		UncheckedExtrinsic = UncheckedExtrinsic,
	{
		System: frame_system::{Pallet, Call, Storage, Config, Event<T>},
		Tokens: orml_tokens::{Pallet, Storage, Event<T>, Config<T>},
		PalletBalances: pallet_balances::{Pallet, Call, Storage, Event<T>},
		Currencies: orml_currencies::{Pallet, Call, Event<T>},
		Tokens: orml_tokens::{Pallet, Call, Storage, Event<T>},
		SerpTreasuryModule: serp_treasury::{Pallet, Storage, Call, Event<T>},
		SerpPrices: prices::{Pallet, Storage, Call, Event<T>},
		SettPay: serp_settpay::{Pallet, Storage, Call, Event<T>},
	}
);

pub struct ExtBuilder {
	endowed_accounts: Vec<(AccountId, CurrencyId, Balance)>,
}

impl Default for ExtBuilder {
	fn default() -> Self {
		Self {
			endowed_accounts: vec![
				(ALICE, DNAR, 100_000),
				(BOB, DNAR, 100_000),
				(CAROL, DNAR, 100_000),
				(ALICE, DRAM, 100_000),
				(BOB, DRAM, 100_000),
				(CAROL, DRAM, 100_000),
				(ALICE, SETT, 100_000),
				(BOB, SETT, 100_000),
				(CAROL, SETT, 100_000),
				(ALICE, USDJ, 100_000),
				(BOB, USDJ, 100_000),
				(CAROL, USDJ, 100_000),
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
		t.into()
	}
}
