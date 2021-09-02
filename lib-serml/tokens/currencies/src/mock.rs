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

//! Mocks for the currencies module.

#![cfg(test)]

use frame_support::{assert_ok, ord_parameter_types, parameter_types, traits::GenesisBuild, PalletId};
use orml_traits::parameter_type_with_key;
use primitives::{CurrencyId, ReserveIdentifier, TokenSymbol, TradingPair};
use sp_core::H256;
use sp_runtime::{
	testing::Header, FixedPointNumber,
	traits::{AccountIdConversion, IdentityLookup, One as OneT},
	AccountId32, Perbill,
};
use sp_std::cell::RefCell;
use support::{AddressMapping, mocks::MockAddressMapping, Price, PriceProvider, Ratio};

use super::*;
use frame_system::EnsureSignedBy;
use sp_core::{bytes::from_hex, H160};
use sp_std::str::FromStr;

pub use crate as currencies;

pub type BlockNumber = u64;

// Currencies constants - CurrencyId/TokenSymbol
pub const DNAR: CurrencyId = CurrencyId::Token(TokenSymbol::DNAR);
pub const DRAM: CurrencyId = CurrencyId::Token(TokenSymbol::DRAM);
pub const SETR: CurrencyId = CurrencyId::Token(TokenSymbol::SETR);
pub const SETUSD: CurrencyId = CurrencyId::Token(TokenSymbol::SETUSD);
pub const SETEUR: CurrencyId = CurrencyId::Token(TokenSymbol::SETEUR);
pub const SETGBP: CurrencyId = CurrencyId::Token(TokenSymbol::SETGBP);
pub const SETCHF: CurrencyId = CurrencyId::Token(TokenSymbol::SETCHF);
pub const SETSAR: CurrencyId = CurrencyId::Token(TokenSymbol::SETSAR);


parameter_types! {
	pub const BlockHashCount: u64 = 250;
	pub const MaximumBlockWeight: u32 = 1024;
	pub const MaximumBlockLength: u32 = 2 * 1024;
	pub const AvailableBlockRatio: Perbill = Perbill::one();
}

pub type AccountId = AccountId32;
impl frame_system::Config for Runtime {
	type Origin = Origin;
	type Call = Call;
	type Index = u64;
	type BlockNumber = u64;
	type Hash = H256;
	type Hashing = sp_runtime::traits::BlakeTwo256;
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

type Balance = u128;

parameter_type_with_key! {
	pub ExistentialDeposits: |_currency_id: CurrencyId| -> Balance {
		Default::default()
	};
}

parameter_types! {
	pub DustAccount: AccountId = PalletId(*b"set/dust").into_account();
	pub const MaxLocks: u32 = 100;
}

impl tokens::Config for Runtime {
	type Event = Event;
	type Balance = Balance;
	type Amount = i64;
	type CurrencyId = CurrencyId;
	type ExistentialDeposits = ExistentialDeposits;
	type OnDust = tokens::TransferDust<Runtime, DustAccount>;
	type WeightInfo = ();
	type MaxLocks = MaxLocks;
	type DustRemovalWhitelist = ();
}

pub const NATIVE_CURRENCY_ID: CurrencyId = CurrencyId::Token(TokenSymbol::DNAR);
pub const X_TOKEN_ID: CurrencyId = CurrencyId::Token(TokenSymbol::SETUSD);

parameter_types! {
	pub const GetNativeCurrencyId: CurrencyId = NATIVE_CURRENCY_ID;
}

parameter_types! {
	pub const ExistentialDeposit: u64 = 1;
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

pub type PalletBalances = pallet_balances::Pallet<Runtime>;

parameter_types! {
	pub const MinimumPeriod: u64 = 1000;
}
impl pallet_timestamp::Config for Runtime {
	type Moment = u64;
	type OnTimestampSet = ();
	type MinimumPeriod = MinimumPeriod;
	type WeightInfo = ();
}

parameter_types! {
	pub const NewContractExtraBytes: u32 = 1;
	pub NetworkContractSource: H160 = alice_evm_addr();
}

ord_parameter_types! {
	pub const CharityFundAccountId:  AccountId32 = AccountId32::from([5u8; 32]);
	pub const SettPayTreasuryAccountId:  AccountId32 = AccountId32::from([6u8; 32]);
	pub const CashDropVaultAccountId:  AccountId32 = AccountId32::from([10u8; 32]);
	pub const Root:  AccountId32 = AccountId32::from([11u8; 32]);
	
	pub const CouncilAccount: AccountId32 = AccountId32::from([1u8; 32]);
	pub const TreasuryAccount: AccountId32 = AccountId32::from([2u8; 32]);
	pub const NetworkContractAccount: AccountId32 = AccountId32::from([0u8; 32]);
	pub const StorageDepositPerByte: u128 = 10;
	pub const MaxCodeSize: u32 = 60 * 1024;
	pub const DeveloperDeposit: u64 = 1000;
	pub const DeploymentFee: u64 = 200;
}

impl setheum_evm::Config for Runtime {
	type AddressMapping = MockAddressMapping;
	type Currency = PalletBalances;
	type TransferAll = ();
	type NewContractExtraBytes = NewContractExtraBytes;
	type StorageDepositPerByte = StorageDepositPerByte;
	type MaxCodeSize = MaxCodeSize;

	type Event = Event;
	type Precompiles = ();
	type ChainId = ();
	type GasToWeight = ();
	type ChargeTransactionPayment = ();
	type NetworkContractOrigin = EnsureSignedBy<NetworkContractAccount, AccountId>;
	type NetworkContractSource = NetworkContractSource;

	type DeveloperDeposit = DeveloperDeposit;
	type DeploymentFee = DeploymentFee;
	type TreasuryAccount = TreasuryAccount;
	type FreeDeploymentOrigin = EnsureSignedBy<CouncilAccount, AccountId32>;

	type WeightInfo = ();
}

impl setheum_evm_bridge::Config for Runtime {
	type EVM = EVM;
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
		SETCHF,
		SETEUR,
		SETGBP,
 		SETSAR,
		SETUSD,
	];
	pub const SetterCurrencyId: CurrencyId = SETR;  // Setter  currency ticker is SETR/
	pub const GetSetUSDCurrencyId: CurrencyId = SETUSD;  // Setter  currency ticker is SETUSD/
	pub const DirhamCurrencyId: CurrencyId = DRAM; // SettinDEX currency ticker is DRAM/

	pub const SerpTreasuryPalletId: PalletId = PalletId(*b"set/serp");

	pub CashDropPeriod: BlockNumber = 120; // Triggers SERP-TES for serping after Every 60 blocks
}

parameter_types! {
	pub const GetExchangeFee: (u32, u32) = (1, 100);
	pub const GetStableCurrencyExchangeFee: (u32, u32) = (1, 200);
	pub const DexPalletId: PalletId = PalletId(*b"set/sdex");
	pub const TradingPathLimit: u32 = 3;
	pub EnabledTradingPairs: Vec<TradingPair> = vec![
		TradingPair::from_currency_ids(DNAR, SETR).unwrap(),
		TradingPair::from_currency_ids(SETCHF, SETR).unwrap(),
		TradingPair::from_currency_ids(SETEUR, SETR).unwrap(),
		TradingPair::from_currency_ids(SETGBP, SETR).unwrap(),
		TradingPair::from_currency_ids(SETSAR, SETR).unwrap(),
		TradingPair::from_currency_ids(SETUSD, SETR).unwrap(),
		TradingPair::from_currency_ids(SETCHF, DNAR).unwrap(),
		TradingPair::from_currency_ids(SETEUR, DNAR).unwrap(),
		TradingPair::from_currency_ids(SETGBP, DNAR).unwrap(),
		TradingPair::from_currency_ids(SETSAR, DNAR).unwrap(),
		TradingPair::from_currency_ids(SETUSD, DNAR).unwrap(),
	];
}

impl setheum_dex::Config for Runtime {
	type Event = Event;
	type Currency = OrmlCurrencies;
	type StableCurrencyIds = StableCurrencyIds;
	type GetExchangeFee = GetExchangeFee;
	type GetStableCurrencyExchangeFee = GetStableCurrencyExchangeFee;
	type TradingPathLimit = TradingPathLimit;
	type PalletId = DexPalletId;
	type CurrencyIdMapping = ();
	type WeightInfo = ();
	type ListingOrigin = EnsureSignedBy<One, AccountId>;
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
	pub const One: AccountId32 = AccountId32::from([11u8; 32]);
}

parameter_type_with_key! {
	pub MinimumClaimableTransferAmounts: |currency_id: CurrencyId| -> Balance {
		match currency_id {
			&SETR => 2,
			&SETCHF => 2,
			&SETEUR => 2,
			&SETGBP => 2,
			&SETUSD => 2,
			&SETSAR => 2,
			_ => 0,
		}
	};
}


parameter_type_with_key! {
	pub GetStableCurrencyMinimumSupply: |currency_id: CurrencyId| -> Balance {
		match currency_id {
			&SETR => 10_000,
			&SETCHF => 10_000,
			&SETEUR => 10_000,
			&SETGBP => 10_000,
			&SETUSD => 10_000,
			&SETSAR => 10_000,
			_ => 0,
		}
	};
}

parameter_types! {
	pub MaxSwapSlippageCompareToOracle: Ratio = Ratio::saturating_from_rational(1, 2);
	pub DefaultFeeSwapPathList: Vec<Vec<CurrencyId>> = vec![vec![SETR, DNAR], vec![SETUSD, SETR, DNAR]];
}

impl serp_treasury::Config for Runtime {
	type Event = Event;
	type Currency = OrmlCurrencies;
	type StableCurrencyIds = StableCurrencyIds;
	type GetStableCurrencyMinimumSupply = GetStableCurrencyMinimumSupply;
	type GetNativeCurrencyId = GetNativeCurrencyId;
	type SetterCurrencyId = SetterCurrencyId;
	type GetSetUSDCurrencyId = GetSetUSDCurrencyId;
	type DirhamCurrencyId = DirhamCurrencyId;
	type CashDropPeriod = CashDropPeriod;
	type SettPayTreasuryAccountId = SettPayTreasuryAccountId;
	type CashDropVaultAccountId = CashDropVaultAccountId;
	type CharityFundAccountId = CharityFundAccountId;
	type DefaultSwapPathList = DefaultFeeSwapPathList;
	type Dex = SetheumDEX;
	type MaxSwapSlippageCompareToOracle = MaxSwapSlippageCompareToOracle;
	type TradingPathLimit = TradingPathLimit;
	type PriceSource = MockPriceSource;
	type MinimumClaimableTransferAmounts = MinimumClaimableTransferAmounts;
	type UpdateOrigin = EnsureSignedBy<Root, AccountId>;
	type PalletId = SerpTreasuryPalletId;
	type WeightInfo = ();
}

impl Config for Runtime {
	type Event = Event;
	type MultiCurrency = Tokens;
	type NativeCurrency = AdaptedBasicCurrency;
	type GetNativeCurrencyId = GetNativeCurrencyId;
	type SerpTreasury = SerpTreasuryModule;
	type WeightInfo = ();
	type AddressMapping = MockAddressMapping;
	type EVMBridge = EVMBridge;
}

pub type NativeCurrency = Currency<Runtime, GetNativeCurrencyId>;
pub type AdaptedBasicCurrency = BasicCurrencyAdapter<Runtime, PalletBalances, i64, u64>;

pub type SignedExtra = setheum_evm::SetEvmOrigin<Runtime>;

pub type Block = sp_runtime::generic::Block<Header, UncheckedExtrinsic>;
pub type UncheckedExtrinsic = sp_runtime::generic::UncheckedExtrinsic<u32, Call, u32, SignedExtra>;

frame_support::construct_runtime!(
	pub enum Runtime where
		Block = Block,
	NodeBlock = Block,
	UncheckedExtrinsic = UncheckedExtrinsic
	{
		System: frame_system::{Pallet, Call, Config, Storage, Event<T>},
		Balances: pallet_balances::{Pallet, Call, Storage, Config<T>, Event<T>},
		Tokens: tokens::{Pallet, Storage, Event<T>, Config<T>},
		EVM: setheum_evm::{Pallet, Config<T>, Call, Storage, Event<T>},
		EVMBridge: setheum_evm_bridge::{Pallet},
		SerpTreasuryModule: serp_treasury::{Pallet, Storage, Event<T>},
		SetheumDEX: setheum_dex::{Pallet, Storage, Call, Event<T>, Config<T>},
		OrmlCurrencies: orml_currencies::{Pallet, Call, Event<T>},
		Currencies: currencies::{Pallet, Call, Event<T>},
	}
);

pub fn alice() -> AccountId {
	<Runtime as Config>::AddressMapping::get_account_id(&alice_evm_addr())
}

pub fn alice_evm_addr() -> EvmAddress {
	EvmAddress::from_str("1000000000000000000000000000000000000001").unwrap()
}

pub fn bob() -> AccountId {
	<Runtime as Config>::AddressMapping::get_account_id(&bob_evm_addr())
}

pub fn bob_evm_addr() -> EvmAddress {
	EvmAddress::from_str("1000000000000000000000000000000000000002").unwrap()
}

pub fn eva() -> AccountId {
	<Runtime as Config>::AddressMapping::get_account_id(&eva_evm_addr())
}

pub fn eva_evm_addr() -> EvmAddress {
	EvmAddress::from_str("1000000000000000000000000000000000000005").unwrap()
}

pub const ID_1: LockIdentifier = *b"1       ";

pub fn erc20_address() -> EvmAddress {
	EvmAddress::from_str("0000000000000000000000000000000002000000").unwrap()
}

pub fn deploy_contracts() {
	let code = from_hex(include!("../../../evm/evm-bridge/src/erc20_demo_contract")).unwrap();
	assert_ok!(EVM::create_network_contract(
		Origin::signed(NetworkContractAccount::get()),
		code,
		0,
		2100_000,
		10000
	));

	let event = Event::EVM(setheum_evm::Event::Created(erc20_address()));
	assert_eq!(System::events().iter().last().unwrap().event, event);

	assert_ok!(EVM::deploy_free(Origin::signed(CouncilAccount::get()), erc20_address()));
}

pub struct ExtBuilder {
	balances: Vec<(AccountId, CurrencyId, Balance)>,
}

impl Default for ExtBuilder {
	fn default() -> Self {
		Self { balances: vec![] }
	}
}

impl ExtBuilder {
	pub fn balances(mut self, balances: Vec<(AccountId, CurrencyId, Balance)>) -> Self {
		self.balances = balances;
		self
	}

	pub fn one_hundred_for_alice_n_bob(self) -> Self {
		self.balances(vec![
			(alice(), NATIVE_CURRENCY_ID, 100),
			(bob(), NATIVE_CURRENCY_ID, 100),
			(alice(), X_TOKEN_ID, 100),
			(bob(), X_TOKEN_ID, 100),
		])
	}

	pub fn build(self) -> sp_io::TestExternalities {
		let mut t = frame_system::GenesisConfig::default()
			.build_storage::<Runtime>()
			.unwrap();

		pallet_balances::GenesisConfig::<Runtime> {
			balances: self
				.balances
				.clone()
				.into_iter()
				.filter(|(_, currency_id, _)| *currency_id == NATIVE_CURRENCY_ID)
				.map(|(account_id, _, initial_balance)| (account_id, initial_balance))
				.collect::<Vec<_>>(),
		}
		.assimilate_storage(&mut t)
		.unwrap();

		tokens::GenesisConfig::<Runtime> {
			balances: self
				.balances
				.into_iter()
				.filter(|(_, currency_id, _)| *currency_id != NATIVE_CURRENCY_ID)
				.collect::<Vec<_>>(),
		}
		.assimilate_storage(&mut t)
		.unwrap();

		setheum_evm::GenesisConfig::<Runtime>::default()
			.assimilate_storage(&mut t)
			.unwrap();

		let mut ext = sp_io::TestExternalities::new(t);
		ext.execute_with(|| System::set_block_number(1));
		ext
	}
}
