// بِسْمِ اللَّهِ الرَّحْمَنِ الرَّحِيم

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

//! Mocks for the cdp engine module.

#![cfg(test)]

use super::*;
use frame_support::{construct_runtime, ord_parameter_types, parameter_types, PalletId};
use frame_system::EnsureSignedBy;
use orml_traits::parameter_type_with_key;
use primitives::{DexShare, Moment, TokenSymbol, TradingPair};
use sp_core::H256;
use sp_runtime::{
	testing::{Header, TestXt},
	traits::{IdentityLookup, One as OneT},
};
use sp_std::cell::RefCell;
use support::{AuctionManager, EmergencyShutdown};

pub type AccountId = u128;
pub type BlockNumber = u64;
pub type AuctionId = u32;

pub const ALICE: AccountId = 1;
pub const BOB: AccountId = 2;
pub const CAROL: AccountId = 3;
pub const ETH: CurrencyId = CurrencyId::Token(TokenSymbol::ETH);
pub const SETM: CurrencyId = CurrencyId::Token(TokenSymbol::SETM);
pub const USDT: CurrencyId = CurrencyId::Token(TokenSymbol::USDT);
pub const USDI: CurrencyId = CurrencyId::Token(TokenSymbol::USDI);
pub const WBTC: CurrencyId = CurrencyId::Token(TokenSymbol::WBTC);

pub const LP_USDI_WBTC: CurrencyId =
	CurrencyId::DexShare(DexShare::Token(TokenSymbol::USDI), DexShare::Token(TokenSymbol::WBTC));
pub const LP_WBTC_ETH: CurrencyId =
	CurrencyId::DexShare(DexShare::Token(TokenSymbol::ETH), DexShare::Token(TokenSymbol::WBTC));

mod cdp_engine {
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
}

impl pallet_balances::Config for Runtime {
	type Balance = Balance;
	type DustRemoval = ();
	type Event = Event;
	type ExistentialDeposit = ExistentialDeposit;
	type AccountStore = frame_system::Pallet<Runtime>;
	type MaxLocks = ();
	type MaxReserves = ();
	type ReserveIdentifier = [u8; 8];
	type WeightInfo = ();
}
pub type AdaptedBasicCurrency = orml_currencies::BasicCurrencyAdapter<Runtime, PalletBalances, Amount, BlockNumber>;

parameter_types! {
	pub const GetNativeCurrencyId: CurrencyId = SETM;
}

impl orml_currencies::Config for Runtime {
	type Event = Event;
	type MultiCurrency = Tokens;
	type NativeCurrency = AdaptedBasicCurrency;
	type GetNativeCurrencyId = GetNativeCurrencyId;
	type WeightInfo = ();
}

parameter_types! {
	pub const LoansPalletId: PalletId = PalletId(*b"set/loan");
}

impl loans::Config for Runtime {
	type Event = Event;
	type Currency = Currencies;
	type RiskManager = CDPEngineModule;
	type CDPTreasury = CDPTreasuryModule;
	type PalletId = LoansPalletId;
}

thread_local! {
	static ETH_PRICE: RefCell<Option<Price>> = RefCell::new(Some(Price::one()));
	static WBTC_PRICE: RefCell<Option<Price>> = RefCell::new(Some(Price::one()));
	static LP_USDI_WBTC_PRICE: RefCell<Option<Price>> = RefCell::new(Some(Price::one()));
	static LP_WBTC_ETH_PRICE: RefCell<Option<Price>> = RefCell::new(Some(Price::one()));
}

pub struct MockPriceSource;
impl MockPriceSource {
	pub fn set_price(currency_id: CurrencyId, price: Option<Price>) {
		match currency_id {
			ETH => ETH_PRICE.with(|v| *v.borrow_mut() = price),
			WBTC => WBTC_PRICE.with(|v| *v.borrow_mut() = price),
			LP_USDI_WBTC => LP_USDI_WBTC_PRICE.with(|v| *v.borrow_mut() = price),
			LP_WBTC_ETH => LP_WBTC_ETH_PRICE.with(|v| *v.borrow_mut() = price),
			_ => {}
		}
	}
}
impl PriceProvider<CurrencyId> for MockPriceSource {
	fn get_price(currency_id: CurrencyId) -> Option<Price> {
		match currency_id {
			ETH => ETH_PRICE.with(|v| *v.borrow()),
			WBTC => WBTC_PRICE.with(|v| *v.borrow()),
			USDI => Some(Price::one()),
			LP_USDI_WBTC => LP_USDI_WBTC_PRICE.with(|v| *v.borrow()),
			LP_WBTC_ETH => LP_WBTC_ETH_PRICE.with(|v| *v.borrow()),
			_ => None,
		}
	}
}

thread_local! {
	pub static AUCTION: RefCell<Option<(AccountId, CurrencyId, Balance, Balance)>> = RefCell::new(None);
}

pub struct MockAuctionManager;
impl AuctionManager<AccountId> for MockAuctionManager {
	type Balance = Balance;
	type CurrencyId = CurrencyId;
	type AuctionId = AuctionId;

	fn new_collateral_auction(
		refund_recipient: &AccountId,
		currency_id: Self::CurrencyId,
		amount: Self::Balance,
		target: Self::Balance,
	) -> DispatchResult {
		AUCTION.with(|v| *v.borrow_mut() = Some((refund_recipient.clone(), currency_id, amount, target)));
		Ok(())
	}

	fn cancel_auction(_id: Self::AuctionId) -> DispatchResult {
		AUCTION.with(|v| *v.borrow_mut() = None);
		Ok(())
	}

	fn get_total_target_in_auction() -> Self::Balance {
		AUCTION
			.with(|v| *v.borrow())
			.map(|auction| auction.3)
			.unwrap_or_default()
	}

	fn get_total_collateral_in_auction(_id: Self::CurrencyId) -> Self::Balance {
		AUCTION
			.with(|v| *v.borrow())
			.map(|auction| auction.2)
			.unwrap_or_default()
	}
}

parameter_types! {
	pub const GetSetUSDId: CurrencyId = USDI;
	pub const MaxAuctionsCount: u32 = 10_000;
	pub const CDPTreasuryPalletId: PalletId = PalletId(*b"set/cdpt");
	pub AlternativeSwapPathJointList: Vec<Vec<CurrencyId>> = vec![
		vec![ETH],
	];
}
impl cdp_treasury::Config for Runtime {
	type Event = Event;
	type Currency = Currencies;
	type GetSetUSDId = GetSetUSDId;
	type AuctionManagerHandler = MockAuctionManager;
	type UpdateOrigin = EnsureSignedBy<One, AccountId>;
	type DEX = DEXModule;
	type MaxAuctionsCount = MaxAuctionsCount;
	type PalletId = CDPTreasuryPalletId;
	type AlternativeSwapPathJointList = AlternativeSwapPathJointList;
	type WeightInfo = ();
}

parameter_types! {
	pub const DEXPalletId: PalletId = PalletId(*b"set/sdex");
	pub GetExchangeFee: (u32, u32) = (1, 100); // 1%
	pub const TradingPathLimit: u32 = 4;
	pub EnabledTradingPairs: Vec<TradingPair> = vec![
		TradingPair::from_currency_ids(USDI, ETH).unwrap(),
		TradingPair::from_currency_ids(USDI, WBTC).unwrap(),
		TradingPair::from_currency_ids(SETM, ETH).unwrap(),
		TradingPair::from_currency_ids(SETM, WBTC).unwrap(),
		TradingPair::from_currency_ids(SETM, USDI).unwrap(),
	];
}

impl dex::Config for Runtime {
	type Event = Event;
	type Currency = Currencies;
	type GetExchangeFee = GetExchangeFee;
	type TradingPathLimit = TradingPathLimit;
	type PalletId = DEXPalletId;
	type CurrencyIdMapping = ();
	type WeightInfo = ();
	type ListingOrigin = EnsureSignedBy<One, AccountId>;
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

thread_local! {
	static IS_SHUTDOWN: RefCell<bool> = RefCell::new(false);
}

pub fn mock_shutdown() {
	IS_SHUTDOWN.with(|v| *v.borrow_mut() = true)
}

pub struct MockEmergencyShutdown;
impl EmergencyShutdown for MockEmergencyShutdown {
	fn is_shutdown() -> bool {
		IS_SHUTDOWN.with(|v| *v.borrow_mut())
	}
}

ord_parameter_types! {
	pub const One: AccountId = 1;
}

parameter_types! {
	pub DefaultLiquidationRatio: Ratio = Ratio::saturating_from_rational(3, 2);
	pub DefaultDebitExchangeRate: ExchangeRate = ExchangeRate::saturating_from_rational(1, 10);
	pub DefaultLiquidationPenalty: Rate = Rate::saturating_from_rational(10, 100);
	pub const MinimumDebitValue: Balance = 2;
	pub MaxSwapSlippageCompareToOracle: Ratio = Ratio::saturating_from_rational(50, 100);
	pub const UnsignedPriority: u64 = 1 << 20;
	pub CollateralCurrencyIds: Vec<CurrencyId> = vec![ETH, WBTC];
	pub DefaultSwapParitalPathList: Vec<Vec<CurrencyId>> = vec![
		vec![USDI],
		vec![SETM, USDI],
	];
}

impl Config for Runtime {
	type Event = Event;
	type PriceSource = MockPriceSource;
	type CollateralCurrencyIds = CollateralCurrencyIds;
	type DefaultLiquidationRatio = DefaultLiquidationRatio;
	type DefaultDebitExchangeRate = DefaultDebitExchangeRate;
	type DefaultLiquidationPenalty = DefaultLiquidationPenalty;
	type MinimumDebitValue = MinimumDebitValue;
	type GetSetUSDId = GetSetUSDId;
	type CDPTreasury = CDPTreasuryModule;
	type UpdateOrigin = EnsureSignedBy<One, AccountId>;
	type MaxSwapSlippageCompareToOracle = MaxSwapSlippageCompareToOracle;
	type UnsignedPriority = UnsignedPriority;
	type EmergencyShutdown = MockEmergencyShutdown;
	type Currency = Currencies;
	type AlternativeSwapPathJointList = AlternativeSwapPathJointList;
	type DEX = DEXModule;
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
		CDPEngineModule: cdp_engine::{Pallet, Storage, Call, Event<T>, Config, ValidateUnsigned},
		CDPTreasuryModule: cdp_treasury::{Pallet, Storage, Call, Config, Event<T>},
		Currencies: orml_currencies::{Pallet, Call, Event<T>},
		Tokens: orml_tokens::{Pallet, Storage, Event<T>, Config<T>},
		LoansModule: loans::{Pallet, Storage, Call, Event<T>},
		PalletBalances: pallet_balances::{Pallet, Call, Storage, Event<T>},
		DEXModule: dex::{Pallet, Storage, Call, Event<T>, Config<T>},
		Timestamp: pallet_timestamp::{Pallet, Call, Storage, Inherent},
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
	balances: Vec<(AccountId, CurrencyId, Balance)>,
}

impl Default for ExtBuilder {
	fn default() -> Self {
		Self {
			balances: vec![
				(ALICE, ETH, 1000),
				(BOB, ETH, 1000),
				(CAROL, ETH, 10000),
				(ALICE, WBTC, 1000),
				(BOB, WBTC, 1000),
				(CAROL, WBTC, 10000),
				(CAROL, USDI, 10000),
			],
		}
	}
}

impl ExtBuilder {
	pub fn build(self) -> sp_io::TestExternalities {
		let mut t = frame_system::GenesisConfig::default()
			.build_storage::<Runtime>()
			.unwrap();

		pallet_balances::GenesisConfig::<Runtime> {
			balances: vec![(CAROL, 10000)],
		}
		.assimilate_storage(&mut t)
		.unwrap();

		orml_tokens::GenesisConfig::<Runtime> {
			balances: self.balances,
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

	pub fn lots_of_accounts() -> Self {
		let mut balances = Vec::new();
		for i in 0..1001 {
			let account_id: AccountId = i;
			balances.push((account_id, ETH, 1000));
		}
		Self { balances }
	}
}
