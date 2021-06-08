// This file is part of Setheum.

// Copyright (C) 2020-2021 Setheum Labs.
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

//! Mocks for the setheum_dex module.

#![cfg(test)]

use super::*;
use frame_support::{construct_runtime, ord_parameter_types, parameter_types};
use frame_system::EnsureSignedBy;
use orml_traits::{parameter_type_with_key, MultiReservableCurrency};
use primitives::{Amount, TokenSymbol};
use sp_core::H256;
use sp_runtime::{testing::Header, traits::IdentityLookup};

pub type BlockNumber = u64;
pub type AccountId = u128;

pub const ALICE: AccountId = 1;
pub const BOB: AccountId = 2;
pub const USDJ: CurrencyId = CurrencyId::Token(TokenSymbol::USDJ);
pub const JCHF: CurrencyId = CurrencyId::Token(TokenSymbol::JCHF);
pub const DNAR: CurrencyId = CurrencyId::Token(TokenSymbol::DNAR);
pub const USDJ_JCHF_PAIR: TradingPair = TradingPair(USDJ, JCHF);
pub const USDJ_DNAR_PAIR: TradingPair = TradingPair(USDJ, DNAR);
pub const DNAR_JCHF_PAIR: TradingPair = TradingPair(DNAR, JCHF);

mod dex {
	pub use super::super::*;
}

parameter_types! {
	pub const BlockHashCount: BlockNumber = 250;
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

pub struct MockDexIncentives;
impl DexIncentives<AccountId, CurrencyId, Balance> for MockDexIncentives {
	fn do_deposit_dex_share(who: &AccountId, lp_currency_id: CurrencyId, amount: Balance) -> DispatchResult {
		Tokens::reserve(lp_currency_id, who, amount)
	}

	fn do_withdraw_dex_share(who: &AccountId, lp_currency_id: CurrencyId, amount: Balance) -> DispatchResult {
		let _ = Tokens::unreserve(lp_currency_id, who, amount);
		Ok(())
	}
}

ord_parameter_types! {
	pub const ListingOrigin: AccountId = 3;
}

parameter_types! {
	pub const GetExchangeFee: (u32, u32) = (1, 100);
	pub const TradingPathLimit: u32 = 3;
	pub const SetheumDexPalletId: PalletId = PalletId(*b"dnr/sdex");
}

impl Config for Runtime {
	type Event = Event;
	type Currency = Tokens;
	type GetExchangeFee = GetExchangeFee;
	type TradingPathLimit = TradingPathLimit;
	type PalletId = SetheumDexPalletId;
	type WeightInfo = ();
	type DexIncentives = MockDexIncentives;
	type ListingOrigin = EnsureSignedBy<ListingOrigin, AccountId>;
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
		DEX = setheum_dex::{Pallet, Storage, Call, Event<T>, Config<T>},
		Tokens: orml_tokens::{Pallet, Storage, Event<T>, Config<T>},
	}
);

pub struct ExtBuilder {
	endowed_accounts: Vec<(AccountId, CurrencyId, Balance)>,
	initial_listing_trading_pairs: Vec<(TradingPair, (Balance, Balance), (Balance, Balance), BlockNumber)>,
	initial_enabled_trading_pairs: Vec<TradingPair>,
	initial_added_liquidity_pools: Vec<(AccountId, Vec<(TradingPair, (Balance, Balance))>)>,
}

impl Default for ExtBuilder {
	fn default() -> Self {
		Self {
			endowed_accounts: vec![
				(ALICE, USDJ, 1_000_000_000_000_000_000u128),
				(BOB, USDJ, 1_000_000_000_000_000_000u128),
				(ALICE, JCHF, 1_000_000_000_000_000_000u128),
				(BOB, JCHF, 1_000_000_000_000_000_000u128),
				(ALICE, DNAR, 1_000_000_000_000_000_000u128),
				(BOB, DNAR, 1_000_000_000_000_000_000u128),
			],
			initial_listing_trading_pairs: vec![],
			initial_enabled_trading_pairs: vec![],
			initial_added_liquidity_pools: vec![],
		}
	}
}

impl ExtBuilder {
	pub fn initialize_listing_trading_pairs(mut self) -> Self {
		self.initial_listing_trading_pairs = vec![
			(
				USDJ_DNAR_PAIR,
				(5_000_000_000_000u128, 1_000_000_000_000u128),
				(5_000_000_000_000_000u128, 1_000_000_000_000_000u128),
				10,
			),
			(
				USDJ_JCHF_PAIR,
				(20_000_000_000_000u128, 1_000_000_000u128),
				(20_000_000_000_000_000u128, 1_000_000_000_000u128),
				10,
			),
			(
				DNAR_JCHF_PAIR,
				(4_000_000_000_000u128, 1_000_000_000u128),
				(4_000_000_000_000_000u128, 1_000_000_000_000u128),
				20,
			),
		];
		self
	}

	pub fn initialize_enabled_trading_pairs(mut self) -> Self {
		self.initial_enabled_trading_pairs = vec![USDJ_DNAR_PAIR, USDJ_JCHF_PAIR, DNAR_JCHF_PAIR];
		self
	}

	pub fn initialize_added_liquidity_pools(mut self, who: AccountId) -> Self {
		self.initial_added_liquidity_pools = vec![(
			who,
			vec![
				(USDJ_DNAR_PAIR, (1_000_000u128, 2_000_000u128)),
				(USDJ_JCHF_PAIR, (1_000_000u128, 2_000_000u128)),
				(DNAR_JCHF_PAIR, (1_000_000u128, 2_000_000u128)),
			],
		)];
		self
	}

	pub fn build(self) -> sp_io::TestExternalities {
		let mut t = frame_system::GenesisConfig::default()
			.build_storage::<Runtime>()
			.unwrap();

		orml_tokens::GenesisConfig::<Runtime> {
			endowed_accounts: self.endowed_accounts,
		}
		.assimilate_storage(&mut t)
		.unwrap();

		setheum_dex::GenesisConfig::<Runtime> {
			initial_listing_trading_pairs: self.initial_listing_trading_pairs,
			initial_enabled_trading_pairs: self.initial_enabled_trading_pairs,
			initial_added_liquidity_pools: self.initial_added_liquidity_pools,
		}
		.assimilate_storage(&mut t)
		.unwrap();

		t.into()
	}
}
