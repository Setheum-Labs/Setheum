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
use primitives::TokenSymbol;
use sp_core::H256;
use sp_runtime::{
	testing::Header,
	traits::{AccountIdConversion, IdentityLookup},
};
use support::StandardValidator;

pub type AccountId = u128;
pub type BlockNumber = u64;

pub const ALICE: AccountId = 1;
pub const BOB: AccountId = 2;
pub const CHARITY_FUND: AccountId = 3;

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
	pub const SetterCurrencyId: CurrencyId = SETT;  // Setter  currency ticker is SETT/
	pub const GetSettUSDCurrencyId: CurrencyId = USDJ;  // Setter  currency ticker is USDJ/
	pub const DirhamCurrencyId: CurrencyId = DRAM; // SettinDEX currency ticker is DRAM/

	pub const SerpTreasuryPalletId: PalletId = PalletId(*b"set/serp");
	pub const SettPayTreasuryPalletId: PalletId = PalletId(*b"set/stpy");
	pub CharutyFundAcc: AccountId = CHARITY_FUND;

	pub SerpTesSchedule: BlockNumber = 60; // Triggers SERP-TES for serping after Every 60 blocks
	pub MaxSlippageSwapWithDEX: Ratio = Ratio::one();
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
	type GetSettUSDCurrencyId = GetSettUSDCurrencyId;
	type DirhamCurrencyId = DirhamCurrencyId;
	type SerpTesSchedule = SerpTesSchedule;
	type SettPayTreasuryAcc = SettPayTreasuryPalletId;
	type CharityFundAcc = CharutyFundAcc;
	type Dex = SetheumDEX;
	type MaxSlippageSwapWithDEX = MaxSlippageSwapWithDEX;
	type PriceSource = MockPriceSource;
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

// mock standard validator (checks if a standard is still valid or not)
pub struct MockStandardValidator;
impl StandardValidator<AccountId, CurrencyId, Balance, Balance> for MockStandardValidator {
	fn check_position_valid(
		currency_id: CurrencyId,
		_reserve_balance: Balance,
		_standard_balance: Balance,
	) -> DispatchResult {
		match currency_id {
			SETT => Err(sp_runtime::DispatchError::Other("mock error")),
			SETT => Ok(()),
			_ => Err(sp_runtime::DispatchError::Other("mock error")),
		}
	}
}

parameter_types! {
	pub StandardCurrencyIds: Vec<CurrencyId> = vec![
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
	pub const GetReserveCurrencyId: CurrencyId = SETT;
	pub const SettmintManagerPalletId: PalletId = PalletId(*b"set/mint");

}

impl Config for Runtime {
	type Event = Event;
	type Convert = MockConvert;
	type Currency = Currencies;
	type StandardCurrencyIds = StandardCurrencyIds;
	type GetReserveCurrencyId = GetReserveCurrencyId;
	type StandardValidator = MockStandardValidator;
	type SerpTreasury = SerpTreasuryModule;
	type PalletId = SettmintManagerPalletId;
	type OnUpdateSettMint = ();
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
		SerpTreasuryModule: serp_treasury::{Pallet, Storage, Call, Event<T>},
	}
);

pub struct ExtBuilder {
	endowed_accounts: Vec<(AccountId, CurrencyId, Balance)>,
}

impl Default for ExtBuilder {
	fn default() -> Self {
		Self {
			endowed_accounts: vec![
				(ALICE, SETT, 1000),
				(BOB, SETT, 1000),
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
