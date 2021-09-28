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

//! Mocks for the loans module.

#![cfg(test)]

use super::*;
use frame_support::{construct_runtime, ord_parameter_types, parameter_types};
use frame_system::EnsureSignedBy;
use orml_traits::parameter_type_with_key;
use primitives::TokenSymbol;
use sp_core::H256;
use sp_runtime::{
	testing::Header,
	traits::{AccountIdConversion, IdentityLookup},
};
use support::{AuctionManager, RiskManager};

pub type AccountId = u128;
pub type AuctionId = u32;
pub type BlockNumber = u64;

pub const ALICE: AccountId = 1;
pub const BOB: AccountId = 2;
pub const SETHEUM: CurrencyId = CurrencyId::Token(TokenSymbol::SETHEUM);
pub const SETUSD: CurrencyId = CurrencyId::Token(TokenSymbol::SETUSD);
pub const DNAR: CurrencyId = CurrencyId::Token(TokenSymbol::DNAR);
pub const BTC: CurrencyId = CurrencyId::Token(TokenSymbol::RENBTC);

mod loans {
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
		100
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
	type AccountStore = frame_system::Pallet<Runtime>;
	type MaxLocks = ();
	type MaxReserves = ();
	type ReserveIdentifier = [u8; 8];
	type WeightInfo = ();
}

parameter_types! {
	pub const GetNativeCurrencyId: CurrencyId = SETHEUM;
}

impl orml_currencies::Config for Runtime {
	type Event = Event;
	type MultiCurrency = Tokens;
	type NativeCurrency = AdaptedBasicCurrency;
	type GetNativeCurrencyId = GetNativeCurrencyId;
	type WeightInfo = ();
}
pub type AdaptedBasicCurrency = orml_currencies::BasicCurrencyAdapter<Runtime, PalletBalances, Amount, BlockNumber>;

pub struct MockAuctionManager;
impl AuctionManager<AccountId> for MockAuctionManager {
	type CurrencyId = CurrencyId;
	type Balance = Balance;
	type AuctionId = AuctionId;

	fn new_collateral_auction(
		_refund_recipient: &AccountId,
		_currency_id: Self::CurrencyId,
		_amount: Self::Balance,
		_target: Self::Balance,
	) -> DispatchResult {
		Ok(())
	}

	fn cancel_auction(_id: Self::AuctionId) -> DispatchResult {
		Ok(())
	}

	fn get_total_target_in_auction() -> Self::Balance {
		Default::default()
	}

	fn get_total_collateral_in_auction(_id: Self::CurrencyId) -> Self::Balance {
		Default::default()
	}
}

ord_parameter_types! {
	pub const One: AccountId = 1;
}

parameter_types! {
	pub const GetStableCurrencyId: CurrencyId = SETUSD;
	pub const MaxAuctionsCount: u32 = 10_000;
	pub const CDPTreasuryModuleId: ModuleId = ModuleId(*b"set/cdpt");
	pub TreasuryAccount: AccountId = ModuleId(*b"set/smtr").into_account();
}

impl cdp_treasury::Config for Runtime {
	type Event = Event;
	type Currency = Currencies;
	type GetStableCurrencyId = GetStableCurrencyId;
	type AuctionManagerHandler = MockAuctionManager;
	type UpdateOrigin = EnsureSignedBy<One, AccountId>;
	type DEX = ();
	type MaxAuctionsCount = MaxAuctionsCount;
	type ModuleId = CDPTreasuryModuleId;
	type TreasuryAccount = TreasuryAccount;
	type WeightInfo = ();
}

// mock convert
pub struct MockConvert;
impl Convert<(CurrencyId, Balance), Balance> for MockConvert {
	fn convert(a: (CurrencyId, Balance)) -> Balance {
		(a.1 / Balance::from(2u64)).into()
	}
}

// mock risk manager
pub struct MockRiskManager;
impl RiskManager<AccountId, CurrencyId, Balance, Balance> for MockRiskManager {
	fn get_bad_debt_value(currency_id: CurrencyId, debit_balance: Balance) -> Balance {
		MockConvert::convert((currency_id, debit_balance))
	}

	fn check_position_valid(
		currency_id: CurrencyId,
		_collateral_balance: Balance,
		_debit_balance: Balance,
	) -> DispatchResult {
		match currency_id {
			DNAR => Err(sp_runtime::DispatchError::Other("mock invalid position error")),
			BTC => Ok(()),
			_ => Err(sp_runtime::DispatchError::Other("mock invalid position error")),
		}
	}

	fn check_debit_cap(currency_id: CurrencyId, total_debit_balance: Balance) -> DispatchResult {
		match (currency_id, total_debit_balance) {
			(DNAR, 1000) => Err(sp_runtime::DispatchError::Other("mock exceed debit value cap error")),
			(BTC, 1000) => Err(sp_runtime::DispatchError::Other("mock exceed debit value cap error")),
			(_, _) => Ok(()),
		}
	}
}

parameter_types! {
	pub const LoansModuleId: ModuleId = ModuleId(*b"set/loan");
}

impl Config for Runtime {
	type Event = Event;
	type Convert = MockConvert;
	type Currency = Currencies;
	type RiskManager = MockRiskManager;
	type CDPTreasury = CDPTreasuryModule;
	type ModuleId = LoansModuleId;
	type OnUpdateLoan = ();
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
		LoansModule: loans::{Pallet, Storage, Call, Event<T>},
		Tokens: orml_tokens::{Pallet, Storage, Event<T>, Config<T>},
		PalletBalances: pallet_balances::{Pallet, Call, Storage, Event<T>},
		Currencies: orml_currencies::{Pallet, Call, Event<T>},
		CDPTreasuryModule: cdp_treasury::{Pallet, Storage, Call, Event<T>},
	}
);

pub struct ExtBuilder {
	balances: Vec<(AccountId, CurrencyId, Balance)>,
}

impl Default for ExtBuilder {
	fn default() -> Self {
		Self {
			balances: vec![
				(ALICE, DNAR, 1000),
				(ALICE, BTC, 1000),
				(BOB, DNAR, 1000),
				(BOB, BTC, 1000),
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
