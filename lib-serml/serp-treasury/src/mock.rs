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
use frame_support::{construct_runtime, ord_parameter_types, parameter_types};
use frame_system::EnsureSignedBy;
use orml_traits::parameter_type_with_key;
use primitives::{Amount, TokenSymbol};
use sp_core::{H160, H256};
use sp_runtime::{
	testing::Header, ModuleId,
	traits::IdentityLookup,
};
use sp_std::cell::RefCell;
use support::{Price, PriceProvider};

pub type AccountId = u128;
pub type BlockNumber = u64;

pub const ALICE: AccountId = 0;
pub const BOB: AccountId = 1;
pub const CHARITY_FUND: AccountId = 2;
pub const TREASURY: AccountId = 3;
pub const CDP_TREASURY: AccountId = 5;
pub const SETRPAY: AccountId = 9;
pub const VAULT: AccountId = 10;
pub const ROOT: AccountId = 11;

// Currencies constants - CurrencyId/TokenSymbol
pub const SETM: CurrencyId = CurrencyId::Token(TokenSymbol::SETM);
pub const SERP: CurrencyId = CurrencyId::Token(TokenSymbol::SERP);
pub const DNAR: CurrencyId = CurrencyId::Token(TokenSymbol::DNAR);
pub const SETR: CurrencyId = CurrencyId::Token(TokenSymbol::SETR);
pub const SETUSD: CurrencyId = CurrencyId::Token(TokenSymbol::SETUSD);

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
	pub StableCurrencyIds: Vec<CurrencyId> = vec![
		SETR,
		SETUSD,
	];
	pub const GetSerpCurrencyId: CurrencyId = SERP;
	pub const GetDinarCurrencyId: CurrencyId = DNAR;
	pub const SetterCurrencyId: CurrencyId = SETR;  // Setter  currency ticker is SETR/
	pub const GetSetUSDCurrencyId: CurrencyId = SETUSD;  // Setter  currency ticker is SETUSD/

	pub const SerpTreasuryModuleId: ModuleId = ModuleId(*b"set/serp");
	pub const PublicFundAccountId: AccountId = CHARITY_FUND;
	pub const CDPTreasuryAccountId: AccountId = CDP_TREASURY;
	pub const SetheumTreasuryAccountId: AccountId = TREASURY;
	pub const CashDropPoolAccountId: AccountId = VAULT;
}

pub struct MockDEX;
impl DEXManager<AccountId, CurrencyId, Balance> for MockDEX {
	fn get_liquidity_pool(currency_id_a: CurrencyId, currency_id_b: CurrencyId) -> (Balance, Balance) {
		match (currency_id_a, currency_id_b) {
			(SETUSD, DNAR) => (10000, 200),
			_ => (0, 0),
		}
	}

	fn get_liquidity_token_address(_currency_id_a: CurrencyId, _currency_id_b: CurrencyId) -> Option<H160> {
		unimplemented!()
	}

	fn get_swap_target_amount(
		_path: &[CurrencyId],
		_supply_amount: Balance,
	) -> Option<Balance> {
		unimplemented!()
	}

	fn get_swap_supply_amount(
		_path: &[CurrencyId],
		_target_amount: Balance,
	) -> Option<Balance> {
		unimplemented!()
	}

	fn swap_with_exact_supply(
		_who: &AccountId,
		_path: &[CurrencyId],
		_supply_amount: Balance,
		_min_target_amount: Balance,
	) -> sp_std::result::Result<Balance, DispatchError> {
		unimplemented!()
	}

	fn buyback_swap_with_exact_supply(
		_who: &AccountId,
		_path: &[CurrencyId],
		_supply_amount: Balance,
	) -> sp_std::result::Result<Balance, DispatchError> {
		unimplemented!()
	}

	fn swap_with_exact_target(
		_who: &AccountId,
		_path: &[CurrencyId],
		_target_amount: Balance,
		_max_supply_amount: Balance,
	) -> sp_std::result::Result<Balance, DispatchError> {
		unimplemented!()
	}

	fn buyback_swap_with_exact_target(
		_who: &AccountId,
		_path: &[CurrencyId],
		_target_amount: Balance,
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
	) -> DispatchResult {
		unimplemented!()
	}
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

parameter_type_with_key! {
	pub GetStableCurrencyMinimumSupply: |currency_id: CurrencyId| -> Balance {
		match currency_id {
			&SETR => 10_000,
			&SETUSD => 10_000,
			_ => 0,
		}
	};
}

parameter_types! {
	pub MaxSwapSlippageCompareToOracle: Ratio = Ratio::saturating_from_rational(1, 2);
	pub DefaultSwapPathList: Vec<Vec<CurrencyId>> = vec![vec![SETR, DNAR], vec![SETUSD, SETR, DNAR]];
	pub const TradingPathLimit: u32 = 3;
	pub StableCurrencyInflationPeriod: u64 = 5;
	pub SetterMinimumClaimableTransferAmounts: Balance = 2;
	pub SetterMaximumClaimableTransferAmounts: Balance = 200;
	pub SetDollarMinimumClaimableTransferAmounts: Balance = 2;
	pub SetDollarMaximumClaimableTransferAmounts: Balance = 200;
}

ord_parameter_types! {
	pub const Root: AccountId = ROOT;
}

impl Config for Runtime {
	type Event = Event;
	type Currency = Currencies;
	type StableCurrencyIds = StableCurrencyIds;
	type StableCurrencyInflationPeriod = StableCurrencyInflationPeriod;
	type GetStableCurrencyMinimumSupply = GetStableCurrencyMinimumSupply;
	type GetNativeCurrencyId = GetNativeCurrencyId;
	type GetSerpCurrencyId = GetSerpCurrencyId;
	type GetDinarCurrencyId = GetDinarCurrencyId;
	type SetterCurrencyId = SetterCurrencyId;
	type GetSetUSDCurrencyId = GetSetUSDCurrencyId;
	type CashDropPoolAccountId = CashDropPoolAccountId;
	type PublicFundAccountId = PublicFundAccountId;
	type CDPTreasuryAccountId = CDPTreasuryAccountId;
	type SetheumTreasuryAccountId = SetheumTreasuryAccountId;
	type DefaultSwapPathList = DefaultSwapPathList;
	type Dex = MockDEX;
	type MaxSwapSlippageCompareToOracle = MaxSwapSlippageCompareToOracle;
	type TradingPathLimit = TradingPathLimit;
	type PriceSource = MockPriceSource;
	type SetterMinimumClaimableTransferAmounts = SetterMinimumClaimableTransferAmounts;
	type SetterMaximumClaimableTransferAmounts = SetterMaximumClaimableTransferAmounts;
	type SetDollarMinimumClaimableTransferAmounts = SetDollarMinimumClaimableTransferAmounts;
	type SetDollarMaximumClaimableTransferAmounts = SetDollarMaximumClaimableTransferAmounts;
	type UpdateOrigin = EnsureSignedBy<Root, AccountId>;
	type ModuleId = SerpTreasuryModuleId;
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
	}
);

pub struct ExtBuilder {
	endowed_accounts: Vec<(AccountId, CurrencyId, Balance)>,
}

impl Default for ExtBuilder {
	fn default() -> Self {
		Self {
			endowed_accounts: vec![
				(ALICE, SETUSD, 1000),
				(ALICE, SETR, 1000),
				(ALICE, DNAR, 1000),
				(ALICE, SETM, 1000),
				(BOB, SETUSD, 1000),
				(BOB, SETR, 1000),
				(BOB, DNAR, 1000),
				(BOB, SETM, 1000),
				(CHARITY_FUND, SETUSD, 1000),
				(CHARITY_FUND, SETR, 1000),
				(CHARITY_FUND, DNAR, 1000),
				(CHARITY_FUND, SETM, 1000),
				(TREASURY, SETUSD, 1000),
				(TREASURY, SETR, 1000),
				(TREASURY, DNAR, 1000),
				(TREASURY, SETM, 1000),
				(CDP_TREASURY, SETUSD, 1000),
				(CDP_TREASURY, SETR, 1000),
				(CDP_TREASURY, DNAR, 1000),
				(CDP_TREASURY, SETM, 1000),
				(SETRPAY, SETUSD, 1000),
				(SETRPAY, SETR, 1000),
				(VAULT, SETUSD, 1000),
				(VAULT, SETR, 1000),
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
