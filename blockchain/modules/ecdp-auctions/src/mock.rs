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

//! Mocks for the auction manager module.

#![cfg(test)]

use super::*;
use frame_support::{
	construct_runtime, derive_impl, ord_parameter_types, parameter_types,
	traits::{ConstU32, ConstU64, Nothing},
	PalletId,
};
use frame_system::EnsureSignedBy;
pub use module_support::Price;
use module_support::SpecificJointsSwap;
use orml_traits::parameter_type_with_key;
use primitives::{TokenSymbol, TradingPair};
use sp_runtime::{
	testing::{Header, TestXt},
	traits::{AccountIdConversion, IdentityLookup, One as OneT},
	BuildStorage,
};
use sp_std::cell::RefCell;

pub type AccountId = u128;
pub type BlockNumber = u64;
pub type AuctionId = u32;
pub type Amount = i64;

pub const ALICE: AccountId = 1;
pub const BOB: AccountId = 2;
pub const CAROL: AccountId = 3;
pub const USSD: CurrencyId = CurrencyId::Token(TokenSymbol::USSD);
pub const BTC: CurrencyId = CurrencyId::ForeignAsset(255);
pub const EDF: CurrencyId = CurrencyId::Token(TokenSymbol::EDF);

mod auction_manager {
	pub use super::super::*;
}

#[derive_impl(frame_system::config_preludes::TestDefaultConfig as frame_system::DefaultConfig)]
impl frame_system::Config for Runtime {
	type AccountId = AccountId;
	type Lookup = IdentityLookup<Self::AccountId>;
	type Block = Block;
	type AccountData = pallet_balances::AccountData<Balance>;
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

impl orml_auction::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type Balance = Balance;
	type AuctionId = AuctionId;
	type Handler = EcdpAuctionsManagerModule;
	type WeightInfo = ();
}

ord_parameter_types! {
	pub const One: AccountId = 1;
}

parameter_types! {
	pub const GetUSSDCurrencyId: CurrencyId = USSD;
	pub const MaxAuctionsCount: u32 = 10_000;
	pub const EcdpUssdTreasuryPalletId: PalletId = PalletId(*b"set/ussdtrsymod");
	pub TreasuryAccount: AccountId = PalletId(*b"set/ussdtrsyacc").into_account_truncating();
	pub AlternativeSwapPathJointList: Vec<Vec<CurrencyId>> = vec![
		vec![EDF],
	];
}

impl module_ecdp_ussd_treasury::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type Currency = Tokens;
	type GetUSSDCurrencyId = GetUSSDCurrencyId;
	type EcdpAuctionsManagerHandler = EcdpAuctionsManagerModule;
	type UpdateOrigin = EnsureSignedBy<One, AccountId>;
	type DEX = EdfisSwapModule;
	type Swap = SpecificJointsSwap<EdfisSwapModule, AlternativeSwapPathJointList>;
	type MaxAuctionsCount = MaxAuctionsCount;
	type PalletId = EcdpUssdTreasuryPalletId;
	type TreasuryAccount = TreasuryAccount;
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
	fn get_relative_price(_base: CurrencyId, _quote: CurrencyId) -> Option<Price> {
		RELATIVE_PRICE.with(|v| *v.borrow_mut())
	}

	fn get_price(_currency_id: CurrencyId) -> Option<Price> {
		None
	}
}

parameter_types! {
	pub const EdfisSwapPalletId: PalletId = PalletId(*b"set/edfis");
	pub const GetExchangeFee: (u32, u32) = (0, 100);
	pub EnabledTradingPairs: Vec<TradingPair> = vec![
		TradingPair::from_currency_ids(USSD, BTC).unwrap(),
		TradingPair::from_currency_ids(EDF, BTC).unwrap(),
		TradingPair::from_currency_ids(USSD, EDF).unwrap()
	];
}

impl module_edfis_swap_legacy::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type Currency = Tokens;
	type GetExchangeFee = GetExchangeFee;
	type TradingPathLimit = ConstU32<4>;
	type PalletId = EdfisSwapPalletId;
	type Erc20InfoMapping = ();
	type Incentives = ();
	type WeightInfo = ();
	type ListingOrigin = EnsureSignedBy<One, AccountId>;
	type ExtendedProvisioningBlocks = ConstU64<0>;
	type OnLiquidityPoolUpdated = ();
}

thread_local! {
	static IS_SHUTDOWN: RefCell<bool> = RefCell::new(false);
}

pub fn mock_shutdown() {
	IS_SHUTDOWN.with(|v| *v.borrow_mut() = true)
}

pub struct MockEcdpEmergencyShutdown;
impl EcdpEmergencyShutdown for MockEcdpEmergencyShutdown {
	fn is_shutdown() -> bool {
		IS_SHUTDOWN.with(|v| *v.borrow_mut())
	}
}

parameter_types! {
	pub MinimumIncrementSize: Rate = Rate::saturating_from_rational(1, 20);
}

impl Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type Currency = Tokens;
	type Auction = AuctionModule;
	type MinimumIncrementSize = MinimumIncrementSize;
	type AuctionTimeToClose = ConstU64<100>;
	type AuctionDurationSoftCap = ConstU64<2000>;
	type GetUSSDCurrencyId = GetUSSDCurrencyId;
	type EcdpUssdTreasury = EcdpUssdTreasuryModule;
	type PriceSource = MockPriceSource;
	type UnsignedPriority = ConstU64<1048576>; // 1 << 20
	type EcdpEmergencyShutdown = MockEcdpEmergencyShutdown;
	type WeightInfo = ();
}

pub type Block = sp_runtime::generic::Block<Header, UncheckedExtrinsic>;
pub type UncheckedExtrinsic = sp_runtime::generic::UncheckedExtrinsic<u32, RuntimeCall, u32, ()>;

construct_runtime!(
	pub enum Runtime {
		System: frame_system,
		EcdpAuctionsManagerModule: auction_manager,
		Tokens: orml_tokens,
		AuctionModule: orml_auction,
		EcdpUssdTreasuryModule: module_ecdp_ussd_treasury,
		EdfisSwapModule: module_edfis_swap_legacy,
	}
);

pub type Extrinsic = TestXt<RuntimeCall, ()>;

impl<LocalCall> frame_system::offchain::SendTransactionTypes<LocalCall> for Runtime
where
	RuntimeCall: From<LocalCall>,
{
	type OverarchingCall = RuntimeCall;
	type Extrinsic = Extrinsic;
}

pub struct ExtBuilder {
	balances: Vec<(AccountId, CurrencyId, Balance)>,
}

impl Default for ExtBuilder {
	fn default() -> Self {
		Self {
			balances: vec![
				(ALICE, USSD, 1000),
				(BOB, USSD, 1000),
				(CAROL, USSD, 1000),
				(ALICE, BTC, 1000),
				(BOB, BTC, 1000),
				(CAROL, BTC, 1000),
				(ALICE, EDF, 1000),
				(BOB, EDF, 1000),
				(CAROL, EDF, 1000),
			],
		}
	}
}

impl ExtBuilder {
	pub fn build(self) -> sp_io::TestExternalities {
		let mut t = frame_system::GenesisConfig::<Runtime>::default()
			.build_storage()
			.unwrap();

		orml_tokens::GenesisConfig::<Runtime> {
			balances: self.balances,
		}
		.assimilate_storage(&mut t)
		.unwrap();

		module_edfis_swap_legacy::GenesisConfig::<Runtime> {
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
			balances.push((account_id, BTC, 1000));
		}
		Self { balances }
	}
}
