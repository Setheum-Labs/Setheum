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
use orml_traits::parameter_type_with_key;
use primitives::{Balance, Moment, TokenSymbol};
use sp_core::H256;
use sp_runtime::{
	testing::{Header, TestXt},
	traits::{AccountIdConversion, IdentityLookup, One as OneT},
	FixedPointNumber,
};
use sp_std::cell::RefCell;
use support::{SerpAuctionManager, CashDropRate, Price, PriceProvider, Rate, Ratio};

mod serp_settpay {
	pub use super::super::*;
}

pub type AccountId = u128;
pub type BlockNumber = u64;
pub type AuctionId = u32;

pub const ALICE: AccountId = 1;
pub const BOB: AccountId = 2;
pub const CAROL: AccountId = 3;

// Currencies constants - CurrencyId/TokenSymbol
pub const DNAR: CurrencyId = CurrencyId::Token(TokenSymbol::DNAR); // Setheum Dinar
pub const DRAM: CurrencyId = CurrencyId::Token(TokenSymbol::DRAM); // Setheum Dirham
pub const SETT: CurrencyId = CurrencyId::Token(TokenSymbol::SETT); // Setter   -  The Defacto stablecoin & settmint reserve asset
pub const USDJ: CurrencyId = CurrencyId::Token(TokenSymbol::USDJ); // Setheum USD (US Dollar stablecoin)

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
	fn get_relative_price(_base: CurrencyId, _quote: CurrencyId) -> Option<Price> {
		Some(Price::one())
	}

	fn get_price(_currency_id: CurrencyId) -> Option<Price> {
		Some(Price::one())
	}

	fn lock_price(_currency_id: CurrencyId) {}

	fn unlock_price(_currency_id: CurrencyId) {}
}
// TODO: Update
pub struct MockSerpAuctionManager;
impl SerpAuctionManager<AccountId> for MockSerpAuctionManager {
	type Balance = Balance;
	type CurrencyId = CurrencyId;
	type AuctionId = AuctionId;

	fn new_setter_auction(
		_refund_recipient: &AccountId,
		_currency_id: Self::CurrencyId,
		_amount: Self::Balance,
		_target: Self::Balance,
	) -> DispatchResult {
		Ok(())
	}

	fn new_diamond_auction(_amount: Self::Balance, _fix: Self::Balance) -> DispatchResult {
		Ok(())
	}

	fn new_serplus_auction(_amount: Self::Balance) -> DispatchResult {
		Ok(())
	}

	fn cancel_auction(_id: Self::AuctionId) -> DispatchResult {
		Ok(())
	}

	fn get_total_standard_in_auction() -> Self::Balance {
		Default::default()
	}

	fn get_total_target_in_auction() -> Self::Balance {
		Default::default()
	}

	fn get_total_setter_in_auction(_id: Self::CurrencyId) -> Self::Balance {
		Default::default()
	}

	fn get_total_diamond_in_auction() -> Self::Balance {
		Default::default()
	}
}

ord_parameter_types! {
	pub const One: AccountId = 1;
}

parameter_types! {
	pub StableCurrencyIds: Vec<CurrencyId> = vec![SETT, USDJ];
	pub const GetSetterCurrencyId: CurrencyId = SETT;  // Setter  currency ticker is SETT
	pub const GetDexerCurrencyId: CurrencyId = DRAM; // SettinDEX currency ticker is DRAM

	pub const MaxAuctionsCount: u32 = 10_000;
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
	pub CashDropCurrencyIds: Vec<CurrencyId> = vec![SETT, USDJ];
	pub RewardableCurrencyIds: Vec<CurrencyId> = vec![DNAR, DRAM, SETT, USDJ];
	pub NonStableDropCurrencyIds: Vec<CurrencyId> = vec![DNAR, DRAM];
	pub SetCurrencyDropCurrencyIds: Vec<CurrencyId> = vec![SETT, USDJ];
	pub const DefaultCashDropRate: CashDropRate = CashDropRate::::saturating_from_rational(2 : 100); // 2% cashdrop
	pub const DefaultMinimumClaimableTransfer: Balance = 10;
	pub const SettPayPalletId: PalletId = PalletId(*b"set/tpay");
}

impl Config for Runtime {
	type Event = Event;
	type Currency = Currencies;
	type GetSetterCurrencyId = GetSetterCurrencyId;
	type StableCurrencyIds = StableCurrencyIds;
	type CashDropCurrencyIds = CashDropCurrencyIds;
	type RewardableCurrencyIds = RewardableCurrencyIds;
	type NonStableDropCurrencyIds = NonStableDropCurrencyIds;
	type SetCurrencyDropCurrencyIds = SetCurrencyDropCurrencyIds;
	type DefaultCashDropRate = DefaultCashDropRate;
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
		SerpTreasuryModule: serp_treasury::{Pallet, Storage, Call, Event<T>},
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
