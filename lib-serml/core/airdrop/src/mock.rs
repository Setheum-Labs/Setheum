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

//! Mocks for the airdrop module.

#![cfg(test)]

use super::*;
use frame_support::{construct_runtime, ord_parameter_types, parameter_types};
use frame_system::EnsureSignedBy;
use orml_traits::parameter_type_with_key;
use primitives::{Amount, AccountId as AccId, TokenSymbol};
use sp_core::H256;
use sp_runtime::{
	testing::Header, AccountId32,
	traits::{IdentityLookup},
};

pub type AccountId = AccId;
pub type BlockNumber = u64;

pub const TREASURY: AccountId = AccountId32::new([0u8; 32]);
pub const ALICE: AccountId = AccountId32::new([2u8; 32]);
pub const BOB: AccountId = AccountId32::new([3u8; 32]);
pub const USDT: CurrencyId = CurrencyId::Token(TokenSymbol::USDT);
pub const USDI: CurrencyId = CurrencyId::Token(TokenSymbol::USDI);
pub const SETM: CurrencyId = CurrencyId::Token(TokenSymbol::SETM);
pub const BNB: CurrencyId = CurrencyId::Token(TokenSymbol::BNB);

mod airdrop {
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
	type AccountData = ();
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
	pub const GetUSDStablecoinId: CurrencyId = USDI;  // SetDollar currency ticker is USDI/
	pub const GetNativeCurrencyId: CurrencyId = SETM;  // Setheum native currency ticker is SETM/
	pub const AirdropPalletId: PalletId = PalletId(*b"set/drop");
	pub const MaxAirdropListSize: usize = 4;
}

ord_parameter_types! {
	pub const TreasuryAccount: AccountId = TREASURY;
	pub const One: AccountId = AccountId32::new([1u8; 32]);
}
impl Config for Runtime {
	type Event = Event;
	type MultiCurrency = Tokens;
	type MaxAirdropListSize = MaxAirdropListSize;
	type FundingOrigin = TreasuryAccount;
	type DropOrigin = EnsureSignedBy<One, AccountId>;
	type PalletId = AirdropPalletId;
}

pub type Block = sp_runtime::generic::Block<Header, UncheckedExtrinsic>;
pub type UncheckedExtrinsic = sp_runtime::generic::UncheckedExtrinsic<u32, Call, u32, ()>;

construct_runtime!(
	pub enum Runtime where
		Block = Block,
		NodeBlock = Block,
		UncheckedExtrinsic = UncheckedExtrinsic
	{
		System: frame_system::{Pallet, Storage, Call, Config, Event<T>},
		AirDrop: airdrop::{Pallet, Storage, Call, Event<T>},
		Tokens: orml_tokens::{Pallet, Storage, Call, Event<T>},
	}
);

pub struct ExtBuilder {
	_balances: Vec<(AccountId, CurrencyId, Balance)>,
}

impl Default for ExtBuilder {
	fn default() -> Self {
		Self {
			_balances: vec![
				(ALICE, USDT, 1000),
				(BOB, USDT, 1000),
				(TREASURY, USDT, 1000),
				(ALICE, USDI, 1000),
				(BOB, USDI, 1000),
				(TREASURY, USDI, 1000),
				(ALICE, SETM, 1000),
				(BOB, SETM, 1000),
				(TREASURY, SETM, 1000),
				(ALICE, ETH, 1000),
				(BOB, ETH, 1000),
				(TREASURY, ETH, 1000),
				(ALICE, WBTC, 1000),
				(BOB, WBTC, 1000),
				(TREASURY, WBTC, 1000),
				(ALICE, BNB, 1000),
				(BOB, BNB, 1000),
				(TREASURY, BNB, 1000),
			],
		}
	}
}
