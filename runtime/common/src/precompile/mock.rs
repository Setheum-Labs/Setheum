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

#![cfg(test)]

use crate::{AllPrecompiles, Ratio, BlockWeights, SystemContractsFilter, Weight};
use codec::{Decode, Encode, MaxEncodedLen};
use frame_support::{
	assert_ok, ord_parameter_types, parameter_types,
	traits::{GenesisBuild, InstanceFilter, OnFinalize, OnInitialize, SortedMembers},
	weights::IdentityFee,
	PalletId, RuntimeDebug,
};
use frame_system::{EnsureRoot, EnsureSignedBy};
use module_support::{
	mocks::MockAddressMapping, AddressMapping as AddressMappingT, SerpTreasury,
};
use orml_traits::parameter_type_with_key;
pub use primitives::{
	evm::EvmAddress, Amount, BlockNumber, CurrencyId, DexShare, Header, Nonce, ReserveIdentifier, TokenSymbol,
	TradingPair,
};
use sp_core::{bytes::from_hex, Bytes, crypto::AccountId32, H160, H256};
use sp_runtime::{
	traits::{BlakeTwo256, Convert, IdentityLookup, One as OneT},
	DispatchResult, FixedPointNumber, FixedU128, Perbill,
};
use sp_std::{collections::btree_map::BTreeMap, convert::TryFrom, str::FromStr};

pub type AccountId = AccountId32;
type Key = CurrencyId;
pub type Price = FixedU128;
type Balance = u128;

parameter_types! {
	pub const BlockHashCount: u32 = 250;
}
impl frame_system::Config for Test {
	type BaseCallFilter = ();
	type BlockWeights = BlockWeights;
	type BlockLength = ();
	type Origin = Origin;
	type Call = Call;
	type Index = u32;
	type BlockNumber = BlockNumber;
	type Hash = H256;
	type Hashing = BlakeTwo256;
	type AccountId = AccountId;
	type Lookup = IdentityLookup<Self::AccountId>;
	type Header = Header;
	type Event = Event;
	type BlockHashCount = BlockHashCount;
	type DbWeight = ();
	type Version = ();
	type PalletInfo = PalletInfo;
	type AccountData = pallet_balances::AccountData<Balance>;
	type OnNewAccount = ();
	type OnKilledAccount = ();
	type SystemWeightInfo = ();
	type SS58Prefix = ();
	type OnSetCode = ();
}

pub const DOLLARS: Balance = 1_000_000_000_000_000_000; // 18 DECIMALS

parameter_types! {
	pub const MaxNativeTokenExistentialDeposit: Balance = DOLLARS * 100; // 100 SETM
}

parameter_types! {
	pub const MinimumCount: u32 = 1;
	pub const ExpiresIn: u32 = 600;
	pub const RootOperatorAccountId: AccountId = ALICE;
	pub static OracleMembers: Vec<AccountId> = vec![ALICE, BOB, EVA];
	pub const MaxHasDispatchedSize: u32 = 40;
}

pub struct Members;

impl SortedMembers<AccountId> for Members {
	fn sorted_members() -> Vec<AccountId> {
		OracleMembers::get()
	}
}

impl orml_oracle::Config for Test {
	type Event = Event;
	type OnNewData = ();
	type CombineData = orml_oracle::DefaultCombineData<Self, MinimumCount, ExpiresIn>;
	type Time = Timestamp;
	type OracleKey = Key;
	type OracleValue = Price;
	type RootOperatorAccountId = RootOperatorAccountId;
	type Members = Members;
	type MaxHasDispatchedSize = MaxHasDispatchedSize;
	type WeightInfo = ();
}

impl pallet_timestamp::Config for Test {
	type Moment = u64;
	type OnTimestampSet = ();
	type MinimumPeriod = ();
	type WeightInfo = ();
}

parameter_type_with_key! {
	pub ExistentialDeposits: |_currency_id: CurrencyId| -> Balance {
		Default::default()
	};
}

impl orml_tokens::Config for Test {
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
	pub const MaxReserves: u32 = 50;
}

impl pallet_balances::Config for Test {
	type Balance = Balance;
	type DustRemoval = ();
	type Event = Event;
	type ExistentialDeposit = ExistentialDeposit;
	type AccountStore = System;
	type MaxLocks = ();
	type MaxReserves = MaxReserves;
	type ReserveIdentifier = ReserveIdentifier;
	type WeightInfo = ();
}

pub struct MockSerpTreasury;
impl SerpTreasury<AccountId> for MockSerpTreasury {
	type Balance = Balance;
	type CurrencyId = CurrencyId;

	fn calculate_supply_change(
		_numerator: Balance,
		_denominator: Balance,
		_supply: Balance
	) -> Self::Balance{
		unimplemented!()
	}

	fn serp_tes_now() -> DispatchResult {
		unimplemented!()
	}

	/// Deliver System StableCurrency Inflation
	fn issue_stablecurrency_inflation() -> DispatchResult {
		unimplemented!()
	}

	/// SerpUp ratio for BuyBack Swaps to burn Dinar
	fn get_buyback_serpup(
		_amount: Balance,
		_currency_id: CurrencyId,
	) -> DispatchResult {
		unimplemented!()
	}

	/// Add CashDrop to the pool
	fn add_cashdrop_to_pool(
		_currency_id: Self::CurrencyId,
		_amount: Self::Balance
	) -> DispatchResult {
		unimplemented!()
	}

	/// Issue CashDrop from the pool to the claimant account
	fn issue_cashdrop_from_pool(
		_claimant_id: &AccountId,
		_currency_id: Self::CurrencyId,
		_amount: Self::Balance
	) -> DispatchResult {
		unimplemented!()
	}

	/// SerpUp ratio for SetPay Cashdrops
	fn get_cashdrop_serpup(
		_amount: Balance,
		_currency_id: CurrencyId
	) -> DispatchResult {
		unimplemented!()
	}

	/// SerpUp ratio for BuyBack Swaps to burn Dinar
	fn get_buyback_serplus(
		_amount: Balance,
		_currency_id: CurrencyId,
	) -> DispatchResult {
		unimplemented!()
	}

	fn get_cashdrop_serplus(
		_amount: Balance, 
		_currency_id: CurrencyId
	) -> DispatchResult {
		unimplemented!()
	}

	/// issue serpup surplus(stable currencies) to their destinations according to the serpup_ratio.
	fn on_serplus(
		_currency_id: CurrencyId,
		_amount: Balance,
	) -> DispatchResult {
		unimplemented!()
	}

	/// issue serpup surplus(stable currencies) to their destinations according to the serpup_ratio.
	fn on_serpup(
		_currency_id: CurrencyId,
		_amount: Balance,
	) -> DispatchResult {
		unimplemented!()
	}

	/// buy back and burn surplus(stable currencies) with swap by DEX.
	fn on_serpdown(
		_currency_id: CurrencyId,
		_amount: Balance,
	) -> DispatchResult {
		unimplemented!()
	}

	/// get the minimum supply of a setcurrency - by key
	fn get_minimum_supply(
		_currency_id: CurrencyId
	) -> Balance {
		unimplemented!()
	}

	/// issue standard to `who`
	fn issue_standard(
		_currency_id: CurrencyId,
		_who: &AccountId,
		_standard: Balance
	) -> DispatchResult {
		unimplemented!()
	}

	/// burn standard(stable currency) of `who`
	fn burn_standard(
		_currency_id: CurrencyId,
		_who: &AccountId,
		_standard: Balance
	) -> DispatchResult {
		unimplemented!()
	}

	/// issue setter of amount setter to `who`
	fn issue_setter(
		_who: &AccountId,
		_setter: Balance
	) -> DispatchResult {
		unimplemented!()
	}

	/// burn setter of `who`
	fn burn_setter(
		_who: &AccountId,
		_setter: Balance
	) -> DispatchResult {
		unimplemented!()
	}

	/// deposit reserve asset (Setter (SETR)) to serp treasury by `who`
	fn deposit_setter(
		_from: &AccountId,
		_amount: Balance
	) -> DispatchResult {
		unimplemented!()
	}

	/// claim cashdrop of `currency_id` relative to `transfer_amount` for `who`
	fn claim_cashdrop(
		_currency_id: CurrencyId,
		_who: &AccountId,
		_transfer_amount: Balance
	) -> DispatchResult {
		unimplemented!()
	}
}

pub const SETM: CurrencyId = CurrencyId::Token(TokenSymbol::SETM);
pub const SERP: CurrencyId = CurrencyId::Token(TokenSymbol::SERP);
pub const SETUSD: CurrencyId = CurrencyId::Token(TokenSymbol::SETUSD);
pub const SETR: CurrencyId = CurrencyId::Token(TokenSymbol::SETR);
pub const LP_SETM_SETUSD: CurrencyId =
	CurrencyId::DexShare(DexShare::Token(TokenSymbol::SETM), DexShare::Token(TokenSymbol::SETUSD));

parameter_types! {
	pub const GetNativeCurrencyId: CurrencyId = SETM;
	pub StableCurrencyIds: Vec<CurrencyId> = vec![
		SETR,
		SETUSD,
	];
}

impl module_currencies::Config for Test {
	type Event = Event;
	type MultiCurrency = Tokens;
	type NativeCurrency = AdaptedBasicCurrency;
	type GetNativeCurrencyId = GetNativeCurrencyId;
	type StableCurrencyIds = StableCurrencyIds;
	type SerpTreasury = MockSerpTreasury;
	type WeightInfo = ();
	type AddressMapping = MockAddressMapping;
	type EVMBridge = EVMBridge;
	type SweepOrigin = EnsureSignedBy<CouncilAccount, AccountId>;
	type OnDust = ();
}

impl module_evm_bridge::Config for Test {
	type EVM = ModuleEVM;
}

impl module_evm_manager::Config for Test {
	type Currency = Balances;
	type EVMBridge = EVMBridge;
}

parameter_types! {
	pub const CreateClassDeposit: Balance = 200;
	pub const CreateTokenDeposit: Balance = 100;
	pub const DataDepositPerByte: Balance = 10;
	pub const NftPalletId: PalletId = PalletId(*b"set/sNFT");
	pub MaxAttributesBytes: u32 = 2048;
}
impl module_nft::Config for Test {
	type Event = Event;
	type Currency = Balances;
	type CreateClassDeposit = CreateClassDeposit;
	type CreateTokenDeposit = CreateTokenDeposit;
	type DataDepositPerByte = DataDepositPerByte;
	type PalletId = NftPalletId;
	type MaxAttributesBytes = MaxAttributesBytes;
	type WeightInfo = ();
}

parameter_types! {
	pub MaxClassMetadata: u32 = 1024;
	pub MaxTokenMetadata: u32 = 1024;
}

impl orml_nft::Config for Test {
	type ClassId = u32;
	type TokenId = u64;
	type ClassData = module_nft::ClassData<Balance>;
	type TokenData = module_nft::TokenData<Balance>;
	type MaxClassMetadata = MaxClassMetadata;
	type MaxTokenMetadata = MaxTokenMetadata;
}

parameter_types! {
	pub const TransactionByteFee: Balance = 10;
		pub DefaultFeeSwapPathList: Vec<Vec<CurrencyId>> = vec![vec![CurrencyId::Token(TokenSymbol::SETUSD), CurrencyId::Token(TokenSymbol::SETM)]];
	pub MaxSwapSlippageCompareToOracle: Ratio = Ratio::one();
}

impl module_transaction_payment::Config for Test {
	type NativeCurrencyId = GetNativeCurrencyId;
	type DefaultFeeSwapPathList = DefaultFeeSwapPathList;
	type Currency = Balances;
	type MultiCurrency = Currencies;
	type OnTransactionPayment = ();
	type TransactionByteFee = TransactionByteFee;
	type WeightToFee = IdentityFee<Balance>;
	type FeeMultiplierUpdate = ();
	type DEX = ();
	type MaxSwapSlippageCompareToOracle = MaxSwapSlippageCompareToOracle;
	type TradingPathLimit = TradingPathLimit;
	type PriceSource = module_prices::RealTimePriceProvider<Test>;
	type WeightInfo = ();
}
pub type ChargeTransactionPayment = module_transaction_payment::ChargeTransactionPayment<Test>;

parameter_types! {
	pub const ProxyDepositBase: u64 = 1;
	pub const ProxyDepositFactor: u64 = 1;
	pub const MaxProxies: u16 = 4;
	pub const MaxPending: u32 = 2;
	pub const AnnouncementDepositBase: u64 = 1;
	pub const AnnouncementDepositFactor: u64 = 1;
}

#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Encode, Decode, RuntimeDebug, MaxEncodedLen)]
pub enum ProxyType {
	Any,
	JustTransfer,
	JustUtility,
}
impl Default for ProxyType {
	fn default() -> Self {
		Self::Any
	}
}
impl InstanceFilter<Call> for ProxyType {
	fn filter(&self, c: &Call) -> bool {
		match self {
			ProxyType::Any => true,
			ProxyType::JustTransfer => matches!(c, Call::Balances(pallet_balances::Call::transfer(..))),
			ProxyType::JustUtility => matches!(c, Call::Utility(..)),
		}
	}
	fn is_superset(&self, o: &Self) -> bool {
		self == &ProxyType::Any || self == o
	}
}

impl pallet_proxy::Config for Test {
	type Event = Event;
	type Call = Call;
	type Currency = Balances;
	type ProxyType = ProxyType;
	type ProxyDepositBase = ProxyDepositBase;
	type ProxyDepositFactor = ProxyDepositFactor;
	type MaxProxies = MaxProxies;
	type WeightInfo = ();
	type CallHasher = BlakeTwo256;
	type MaxPending = MaxPending;
	type AnnouncementDepositBase = AnnouncementDepositBase;
	type AnnouncementDepositFactor = AnnouncementDepositFactor;
}

impl pallet_utility::Config for Test {
	type Event = Event;
	type Call = Call;
	type WeightInfo = ();
}

parameter_types! {
	pub MaximumSchedulerWeight: Weight = Perbill::from_percent(10) * BlockWeights::get().max_block;
	pub const MaxScheduledPerBlock: u32 = 50;
}

impl pallet_scheduler::Config for Test {
	type Event = Event;
	type Origin = Origin;
	type PalletsOrigin = OriginCaller;
	type Call = Call;
	type MaximumWeight = MaximumSchedulerWeight;
	type ScheduleOrigin = EnsureRoot<AccountId>;
	type MaxScheduledPerBlock = MaxScheduledPerBlock;
	type WeightInfo = ();
}

ord_parameter_types! {
	pub const ListingOrigin: AccountId = ALICE;
}

parameter_types! {
	pub const GetExchangeFee: (u32, u32) = (1, 100);
	pub const GetStableCurrencyExchangeFee: (u32, u32) = (0, 100);
	pub const TradingPathLimit: u32 = 3;
	pub const DEXPalletId: PalletId = PalletId(*b"set/sdex");
}

impl module_dex::Config for Test {
	type Event = Event;
	type Currency = Tokens;
	type StableCurrencyIds = StableCurrencyIds;
	type GetExchangeFee = GetExchangeFee;
	type GetStableCurrencyExchangeFee = GetStableCurrencyExchangeFee;
	type TradingPathLimit = TradingPathLimit;
	type PalletId = DEXPalletId;
	type CurrencyIdMapping = EvmCurrencyIdMapping;
	type WeightInfo = ();
	type ListingOrigin = EnsureSignedBy<ListingOrigin, AccountId>;
}

pub type AdaptedBasicCurrency = module_currencies::BasicCurrencyAdapter<Test, Balances, Amount, BlockNumber>;

pub type EvmCurrencyIdMapping = module_evm_manager::EvmCurrencyIdMapping<Test>;
pub type MultiCurrencyPrecompile =
	crate::MultiCurrencyPrecompile<AccountId, MockAddressMapping, EvmCurrencyIdMapping, Currencies>;

pub type NFTPrecompile = crate::NFTPrecompile<AccountId, MockAddressMapping, EvmCurrencyIdMapping, NFTModule>;
pub type StateRentPrecompile =
	crate::StateRentPrecompile<AccountId, MockAddressMapping, EvmCurrencyIdMapping, ModuleEVM>;
pub type OraclePrecompile = crate::OraclePrecompile<
	AccountId,
	MockAddressMapping,
	EvmCurrencyIdMapping,
	module_prices::PriorityLockedPriceProvider<Test>,
>;
pub type ScheduleCallPrecompile = crate::ScheduleCallPrecompile<
	AccountId,
	MockAddressMapping,
	EvmCurrencyIdMapping,
	Scheduler,
	ChargeTransactionPayment,
	Call,
	Origin,
	OriginCaller,
	Test,
>;
pub type DexPrecompile = crate::DexPrecompile<AccountId, MockAddressMapping, EvmCurrencyIdMapping, DexModule>;

parameter_types! {
	pub NetworkContractSource: H160 = alice_evm_addr();
}

ord_parameter_types! {
	pub const CouncilAccount: AccountId32 = AccountId32::from([1u8; 32]);
	pub const TreasuryAccount: AccountId32 = AccountId32::from([2u8; 32]);
	pub const NetworkContractAccount: AccountId32 = AccountId32::from([0u8; 32]);
	pub const NewContractExtraBytes: u32 = 100;
	pub const StorageDepositPerByte: u64 = 10;
	pub const DeveloperDeposit: u64 = 1000;
	pub const DeploymentFee: u64 = 200;
	pub const ChainId: u64 = 1;
}

pub struct GasToWeight;
impl Convert<u64, Weight> for GasToWeight {
	fn convert(a: u64) -> u64 {
		a as Weight
	}
}

impl module_evm::Config for Test {
	type AddressMapping = MockAddressMapping;
	type Currency = Balances;
	type TransferAll = Currencies;
	type NewContractExtraBytes = NewContractExtraBytes;
	type StorageDepositPerByte = StorageDepositPerByte;
	type Event = Event;
	type Precompiles = AllPrecompiles<
		SystemContractsFilter,
		MultiCurrencyPrecompile,
		NFTPrecompile,
		StateRentPrecompile,
		OraclePrecompile,
		ScheduleCallPrecompile,
		DexPrecompile,
	>;
	type ChainId = ChainId;
	type GasToWeight = GasToWeight;
	type ChargeTransactionPayment = ChargeTransactionPayment;
	type NetworkContractOrigin = EnsureSignedBy<NetworkContractAccount, AccountId>;
	type NetworkContractSource = NetworkContractSource;
	type DeveloperDeposit = DeveloperDeposit;
	type DeploymentFee = DeploymentFee;
	type TreasuryAccount = TreasuryAccount;
	type FreeDeploymentOrigin = EnsureSignedBy<CouncilAccount, AccountId>;
	type Runner = module_evm::runner::stack::Runner<Self>;
	type FindAuthor = ();
	type WeightInfo = ();
}

parameter_types! {
	pub SetUSDFixedPrice: Price = Price::saturating_from_rational(1, 1); // $1
	pub SetterFixedPrice: Price = Price::saturating_from_rational(1, 10); // $0.1(10 cents)
	pub const GetSetUSDId: CurrencyId = SETUSD;
	pub const SetterCurrencyId: CurrencyId = SETR;
}

ord_parameter_types! {
	pub const One: AccountId = AccountId::new([1u8; 32]);
}

impl module_prices::Config for Test {
	type Event = Event;
	type Source = Oracle;
	type GetSetUSDId = GetSetUSDId;
	type SetterCurrencyId = SetterCurrencyId;
	type SetUSDFixedPrice = SetUSDFixedPrice;
	type SetterFixedPrice = SetterFixedPrice;
	type LockOrigin = EnsureSignedBy<One, AccountId>;
	type DEX = DexModule;
	type Currency = Currencies;
	type CurrencyIdMapping = EvmCurrencyIdMapping;
	type WeightInfo = ();
}

pub const ALICE: AccountId = AccountId::new([1u8; 32]);
pub const BOB: AccountId = AccountId::new([2u8; 32]);
pub const EVA: AccountId = AccountId::new([5u8; 32]);

pub fn alice() -> AccountId {
	<Test as module_evm::Config>::AddressMapping::get_account_id(&alice_evm_addr())
}

pub fn alice_evm_addr() -> EvmAddress {
	EvmAddress::from_str("1000000000000000000000000000000000000001").unwrap()
}

pub fn bob() -> AccountId {
	<Test as module_evm::Config>::AddressMapping::get_account_id(&bob_evm_addr())
}

pub fn bob_evm_addr() -> EvmAddress {
	EvmAddress::from_str("1000000000000000000000000000000000000002").unwrap()
}

pub fn setm_evm_address() -> EvmAddress {
	EvmAddress::try_from(SETM).unwrap()
}

pub fn setusd_evm_address() -> EvmAddress {
	EvmAddress::try_from(SETUSD).unwrap()
}

pub fn serp_evm_address() -> EvmAddress {
	EvmAddress::try_from(SERP).unwrap()
}

pub fn lp_setm_setusd_evm_address() -> EvmAddress {
	EvmAddress::try_from(LP_SETM_SETUSD).unwrap()
}

pub fn erc20_address_not_exists() -> EvmAddress {
	EvmAddress::from_str("0000000000000000000000000000000200000001").unwrap()
}

/// Predeployed contract addresses
pub fn evm_genesis() -> BTreeMap<H160, module_evm::GenesisAccount<Balance, Nonce>> {
	let contracts_json = &include_bytes!("../../../../lib-serml/sevm/predeploy-contracts/resources/bytecodes.json")[..];
	let contracts: Vec<(String, String, String)> = serde_json::from_slice(contracts_json).unwrap();
	let mut accounts = BTreeMap::new();
	for (_, address, code_string) in contracts {
		let account = module_evm::GenesisAccount {
			nonce: 0,
			balance: 0u128,
			storage: Default::default(),
			code: Bytes::from_str(&code_string).unwrap().0,
		};
		let addr = H160::from_slice(
			from_hex(address.as_str())
				.expect("predeploy-contracts must specify address")
				.as_slice(),
		);
		accounts.insert(addr, account);
	}
	accounts
}


pub const INITIAL_BALANCE: Balance = 1_000_000_000_000;

type UncheckedExtrinsic = frame_system::mocking::MockUncheckedExtrinsic<Test>;
type Block = frame_system::mocking::MockBlock<Test>;

frame_support::construct_runtime!(
	pub enum Test where
		Block = Block,
		NodeBlock = Block,
		UncheckedExtrinsic = UncheckedExtrinsic,
	{
		System: frame_system::{Pallet, Call, Storage, Config, Event<T>},
		Oracle: orml_oracle::{Pallet, Storage, Call, Event<T>},
		Timestamp: pallet_timestamp::{Pallet, Call, Storage, Inherent},
		Tokens: orml_tokens::{Pallet, Storage, Event<T>, Config<T>},
		Balances: pallet_balances::{Pallet, Call, Storage, Config<T>, Event<T>},
		Currencies: module_currencies::{Pallet, Call, Event<T>},
		EVMBridge: module_evm_bridge::{Pallet},
		EVMManager: module_evm_manager::{Pallet, Storage},
		NFTModule: module_nft::{Pallet, Call, Event<T>},
		TransactionPayment: module_transaction_payment::{Pallet, Call, Storage},
		Prices: module_prices::{Pallet, Storage, Call, Event<T>},
		Proxy: pallet_proxy::{Pallet, Call, Storage, Event<T>},
		Utility: pallet_utility::{Pallet, Call, Event},
		Scheduler: pallet_scheduler::{Pallet, Call, Storage, Event<T>},
		DexModule: module_dex::{Pallet, Storage, Call, Event<T>, Config<T>},
		ModuleEVM: module_evm::{Pallet, Config<T>, Call, Storage, Event<T>},
	}
);

// This function basically just builds a genesis storage key/value store
// according to our desired mockup.
pub fn new_test_ext() -> sp_io::TestExternalities {
	let mut storage = frame_system::GenesisConfig::default().build_storage::<Test>().unwrap();

	let mut accounts = BTreeMap::new();
	let mut evm_genesis_accounts = evm_genesis();
	accounts.append(&mut evm_genesis_accounts);

	accounts.insert(
		alice_evm_addr(),
		module_evm::GenesisAccount {
			nonce: 1,
			balance: INITIAL_BALANCE,
			storage: Default::default(),
			code: Default::default(),
		},
	);
	accounts.insert(
		bob_evm_addr(),
		module_evm::GenesisAccount {
			nonce: 1,
			balance: INITIAL_BALANCE,
			storage: Default::default(),
			code: Default::default(),
		},
	);

	pallet_balances::GenesisConfig::<Test>::default()
		.assimilate_storage(&mut storage)
		.unwrap();
	module_evm::GenesisConfig::<Test> {
		accounts,
		treasury: Default::default(),
	}
	.assimilate_storage(&mut storage)
	.unwrap();

	let mut ext = sp_io::TestExternalities::new(storage);
	ext.execute_with(|| {
		System::set_block_number(1);
		Timestamp::set_timestamp(1);

		assert_ok!(Currencies::update_balance(
			Origin::root(),
			ALICE,
			SERP,
			1_000_000_000_000
		));
		assert_ok!(Currencies::update_balance(Origin::root(), ALICE, SETUSD, 1_000_000_000));

		assert_ok!(Currencies::update_balance(
			Origin::root(),
			MockAddressMapping::get_account_id(&alice_evm_addr()),
			SERP,
			1_000
		));
	});
	ext
}

pub fn run_to_block(n: u32) {
	while System::block_number() < n {
		Scheduler::on_finalize(System::block_number());
		System::set_block_number(System::block_number() + 1);
		Scheduler::on_initialize(System::block_number());
	}
}
pub fn get_task_id(output: Vec<u8>) -> Vec<u8> {
	let mut num = [0u8; 4];
	num[..].copy_from_slice(&output[64 - 4..64]);
	let task_id_len: u32 = u32::from_be_bytes(num);
	output[64..64 + task_id_len as usize].to_vec()
}
