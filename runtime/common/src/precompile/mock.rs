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

#![cfg(test)]

use crate::{AllPrecompiles, Ratio, RuntimeBlockWeights, SystemContractsFilter, Weight};
use setheum_service::chain_spec::evm_genesis;
use codec::{Decode, Encode};
use frame_support::{
	assert_ok, ord_parameter_types, parameter_types,
	traits::{GenesisBuild, InstanceFilter, MaxEncodedLen, OnFinalize, OnInitialize, SortedMembers},
	weights::IdentityFee,
	PalletId, RuntimeDebug,
};
use frame_system::{EnsureRoot, EnsureSignedBy};
use setheum_support::{
	mocks::MockAddressMapping, AddressMapping as AddressMappingT, ExchangeRate, ExchangeRateProvider,
};
use orml_traits::{parameter_type_with_key, MultiReservableCurrency};
pub use primitives::{
	evm::EvmAddress, Amount, BlockNumber, CurrencyId, DexShare, Header, Nonce, ReserveIdentifier, TokenSymbol,
	TradingPair,
};
use sp_core::{crypto::AccountId32, H160, H256};
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
	type BlockWeights = RuntimeBlockWeights;
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

// Currencies constants - CurrencyId/TokenSymbol
pub const DNAR: CurrencyId = CurrencyId::Token(TokenSymbol::DNAR);
pub const DRAM: CurrencyId = CurrencyId::Token(TokenSymbol::DRAM);
pub const SETR: CurrencyId = CurrencyId::Token(TokenSymbol::SETR);
pub const SETUSD: CurrencyId = CurrencyId::Token(TokenSymbol::SETUSD);
pub const SETEUR: CurrencyId = CurrencyId::Token(TokenSymbol::SETEUR);
pub const SETGBP: CurrencyId = CurrencyId::Token(TokenSymbol::SETGBP);
pub const SETCHF: CurrencyId = CurrencyId::Token(TokenSymbol::SETCHF);
pub const SETSAR: CurrencyId = CurrencyId::Token(TokenSymbol::SETSAR);
pub const BTC: CurrencyId = CurrencyId::Token(TokenSymbol::RENBTC);

pub const RENBTC: CurrencyId = CurrencyId::Token(TokenSymbol::RENBTC);
pub const LP_DNAR_SETUSD: CurrencyId =
	CurrencyId::DexShare(DexShare::Token(TokenSymbol::DNAR), DexShare::Token(TokenSymbol::SETUSD));

// Currencies constants - FiatCurrencyIds (CurrencyId/TokenSymbol)
pub const CHF: CurrencyId = CurrencyId::Token(TokenSymbol::CHF);
pub const EUR: CurrencyId = CurrencyId::Token(TokenSymbol::EUR);
pub const GBP: CurrencyId = CurrencyId::Token(TokenSymbol::GBP);
pub const SAR: CurrencyId = CurrencyId::Token(TokenSymbol::SAR);
pub const USD: CurrencyId = CurrencyId::Token(TokenSymbol::USD);
pub const KWD: CurrencyId = CurrencyId::Token(TokenSymbol::KWD);
pub const JOD: CurrencyId = CurrencyId::Token(TokenSymbol::JOD);
pub const BHD: CurrencyId = CurrencyId::Token(TokenSymbol::BHD);
pub const KYD: CurrencyId = CurrencyId::Token(TokenSymbol::KYD);
pub const OMR: CurrencyId = CurrencyId::Token(TokenSymbol::OMR);
pub const GIP: CurrencyId = CurrencyId::Token(TokenSymbol::GIP);

parameter_types! {
	pub const GetNativeCurrencyId: CurrencyId = DNAR;
}

// TODO: Update!
impl setheum_currencies::Config for Test {
	type Event = Event;
	type MultiCurrency = Tokens;
	type NativeCurrency = AdaptedBasicCurrency;
	type GetNativeCurrencyId = GetNativeCurrencyId;
	type WeightInfo = ();
	type AddressMapping = MockAddressMapping;
	type EVMBridge = EVMBridge;
}

impl setheum_evm_bridge::Config for Test {
	type EVM = ModuleEVM;
}

impl setheum_evm_manager::Config for Test {
	type Currency = Balances;
	type EVMBridge = EVMBridge;
}

parameter_types! {
	pub const CreateClassDeposit: Balance = 200;
	pub const CreateTokenDeposit: Balance = 100;
	pub const DataDepositPerByte: Balance = 10;
	pub const NftPalletId: PalletId = PalletId(*b"set/sNFT");
}
impl setheum_nft::Config for Test {
	type Event = Event;
	type Currency = Balances;
	type CreateClassDeposit = CreateClassDeposit;
	type CreateTokenDeposit = CreateTokenDeposit;
	type DataDepositPerByte = DataDepositPerByte;
	type PalletId = NftPalletId;
	type WeightInfo = ();
}

parameter_types! {
	pub MaxClassMetadata: u32 = 1024;
	pub MaxTokenMetadata: u32 = 1024;
}

impl orml_nft::Config for Test {
	type ClassId = u32;
	type TokenId = u64;
	type ClassData = setheum_nft::ClassData<Balance>;
	type TokenData = setheum_nft::TokenData<Balance>;
	type MaxClassMetadata = MaxClassMetadata;
	type MaxTokenMetadata = MaxTokenMetadata;
}

parameter_types! {
	pub const TransactionByteFee: Balance = 10;
	pub const SetterCurrencyId: CurrencyId = CurrencyId::Token(TokenSymbol::SETR);
	pub AllNonNativeCurrencyIds: Vec<CurrencyId> = vec![CurrencyId::Token(TokenSymbol::SETR)];
	pub MaxSlippageSwapWithDEX: Ratio = Ratio::one();
}

impl setheum_transaction_payment::Config for Test {
	type AllNonNativeCurrencyIds = AllNonNativeCurrencyIds;
	type NativeCurrencyId = GetNativeCurrencyId;
	type SetterCurrencyId = SetterCurrencyId;
	type Currency = Balances;
	type MultiCurrency = Currencies;
	type OnTransactionPayment = ();
	type TransactionByteFee = TransactionByteFee;
	type WeightToFee = IdentityFee<Balance>;
	type FeeMultiplierUpdate = ();
	type DEX = ();
	type MaxSlippageSwapWithDEX = MaxSlippageSwapWithDEX;
	type WeightInfo = ();
}
pub type ChargeTransactionPayment = setheum_transaction_payment::ChargeTransactionPayment<Test>;

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
	pub MaximumSchedulerWeight: Weight = Perbill::from_percent(10) * RuntimeBlockWeights::get().max_block;
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
	pub const TradingPathLimit: u32 = 3;
	pub const DEXPalletId: PalletId = PalletId(*b"set/sdex");
}

impl setheum_dex::Config for Test {
	type Event = Event;
	type Currency = Tokens;
	type GetExchangeFee = GetExchangeFee;
	type TradingPathLimit = TradingPathLimit;
	type PalletId = DEXPalletId;
	type CurrencyIdMapping = EvmCurrencyIdMapping;
	type WeightInfo = ();
	type ListingOrigin = EnsureSignedBy<ListingOrigin, AccountId>;
}

pub type AdaptedBasicCurrency = setheum_currencies::BasicCurrencyAdapter<Test, Balances, Amount, BlockNumber>;

pub type EvmCurrencyIdMapping = setheum_evm_manager::EvmCurrencyIdMapping<Test>;
pub type MultiCurrencyPrecompile =
	crate::MultiCurrencyPrecompile<AccountId, MockAddressMapping, EvmCurrencyIdMapping, Currencies>;

pub type NFTPrecompile = crate::NFTPrecompile<AccountId, MockAddressMapping, EvmCurrencyIdMapping, NFTModule>;
pub type StateRentPrecompile =
	crate::StateRentPrecompile<AccountId, MockAddressMapping, EvmCurrencyIdMapping, ModuleEVM>;
pub type OraclePrecompile = crate::OraclePrecompile<AccountId, MockAddressMapping, EvmCurrencyIdMapping, Prices>;
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
	pub const MaxCodeSize: u32 = 60 * 1024;
	pub const ChainId: u64 = 1;
}

pub struct GasToWeight;
impl Convert<u64, Weight> for GasToWeight {
	fn convert(a: u64) -> u64 {
		a as Weight
	}
}

impl setheum_evm::Config for Test {
	type AddressMapping = MockAddressMapping;
	type Currency = Balances;
	type TransferAll = Currencies;
	type NewContractExtraBytes = NewContractExtraBytes;
	type StorageDepositPerByte = StorageDepositPerByte;
	type MaxCodeSize = MaxCodeSize;
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
	type WeightInfo = ();
}

pub struct MockLiquidStakingExchangeProvider;
impl ExchangeRateProvider for MockLiquidStakingExchangeProvider {
	fn get_exchange_rate() -> ExchangeRate {
		ExchangeRate::saturating_from_rational(1, 2)
	}
}

parameter_types! {
	pub const SetterCurrencyId: CurrencyId = SETR; // Setter currency ticker is SETR.
	pub const GetSetUSDCurrencyId: CurrencyId = SETUSD; // SetUSD currency ticker is SETUSD.
	pub const GetFiatCHFCurrencyId: CurrencyId = CHF; // The CHF Fiat currency denomination.
	pub const GetFiatEURCurrencyId: CurrencyId = EUR; // The EUR Fiat currency denomination.
	pub const GetFiatGBPCurrencyId: CurrencyId = GBP; // The GBP Fiat currency denomination.
	pub const GetFiatSARCurrencyId: CurrencyId = SAR; // The SAR Fiat currency denomination.
	pub const GetFiatUSDCurrencyId: CurrencyId = USD; // The USD Fiat currency denomination.
	pub FiatUsdFixedPrice: Price = Price::saturating_from_rational(1, 1);
	
	pub const GetSetterPegOneCurrencyId: CurrencyId = GBP; // Fiat pegs of the Setter (SETR).
	pub const GetSetterPegTwoCurrencyId: CurrencyId = EUR; // Fiat pegs of the Setter (SETR).
	pub const GetSetterPegThreeCurrencyId: CurrencyId = KWD; // Fiat pegs of the Setter (SETR).
	pub const GetSetterPegFourCurrencyId: CurrencyId = JOD; // Fiat pegs of the Setter (SETR).
	pub const GetSetterPegFiveCurrencyId: CurrencyId = BHD; // Fiat pegs of the Setter (SETR).
	pub const GetSetterPegSixCurrencyId: CurrencyId = KYD; // Fiat pegs of the Setter (SETR).
	pub const GetSetterPegSevenCurrencyId: CurrencyId = OMR; // Fiat pegs of the Setter (SETR).
	pub const GetSetterPegEightCurrencyId: CurrencyId = CHF; // Fiat pegs of the Setter (SETR).
	pub const GetSetterPegNineCurrencyId: CurrencyId = GIP; // Fiat pegs of the Setter (SETR).
	pub const GetSetterPegTenCurrencyId: CurrencyId = USD; // Fiat pegs of the Setter (SETR).
	
	pub StableCurrencyIds: Vec<CurrencyId> = vec![SETR, SETCHF, SETEUR, SETGBP, SETSAR, SETUSD];
	pub FiatCurrencyIds: Vec<CurrencyId> = vec![CHF, EUR, GBP, QAR, SAR, USD, JOD, BHD, KYD, OMR, GIP];
}

ord_parameter_types! {
	pub const One: AccountId = AccountId::new([1u8; 32]);
}

impl setheum_prices::Config for Test {
	type Event = Event;
	type Source = Oracle;
	type SetterCurrencyId = SetterCurrencyId;
	type GetSetUSDCurrencyId = GetSetUSDCurrencyId;
	type GetFiatCHFCurrencyId = GetFiatCHFCurrencyId;
	type GetFiatEURCurrencyId = GetFiatEURCurrencyId;
	type GetFiatGBPCurrencyId = GetFiatGBPCurrencyId;
	type GetFiatSARCurrencyId = GetFiatSARCurrencyId;
	type GetFiatUSDCurrencyId = GetFiatUSDCurrencyId;
	type FiatUsdFixedPrice = FiatUsdFixedPrice;
	type GetSetterPegOneCurrencyId = GetSetterPegOneCurrencyId;
	type GetSetterPegTwoCurrencyId = GetSetterPegTwoCurrencyId;
	type GetSetterPegThreeCurrencyId = GetSetterPegThreeCurrencyId;
	type GetSetterPegFourCurrencyId = GetSetterPegFourCurrencyId;
	type GetSetterPegFiveCurrencyId = GetSetterPegFiveCurrencyId;
	type GetSetterPegSixCurrencyId = GetSetterPegSixCurrencyId;
	type GetSetterPegSevenCurrencyId = GetSetterPegSevenCurrencyId;
	type GetSetterPegEightCurrencyId = GetSetterPegEightCurrencyId;
	type GetSetterPegNineCurrencyId = GetSetterPegNineCurrencyId;
	type GetSetterPegTenCurrencyId = GetSetterPegTenCurrencyId;
	type LockOrigin = EnsureSignedBy<One, AccountId>;
	type LiquidStakingExchangeRateProvider = MockLiquidStakingExchangeProvider;
	type DEX = DexModule;
	type Currency = Currencies;
	type CurrencyIdMapping = EvmCurrencyIdMapping;
	type WeightInfo = ();
}

pub const ALICE: AccountId = AccountId::new([1u8; 32]);
pub const BOB: AccountId = AccountId::new([2u8; 32]);
pub const EVA: AccountId = AccountId::new([5u8; 32]);

pub fn alice() -> AccountId {
	<Test as setheum_evm::Config>::AddressMapping::get_account_id(&alice_evm_addr())
}

pub fn alice_evm_addr() -> EvmAddress {
	EvmAddress::from_str("1000000000000000000000000000000000000001").unwrap()
}

pub fn bob() -> AccountId {
	<Test as setheum_evm::Config>::AddressMapping::get_account_id(&bob_evm_addr())
}

pub fn bob_evm_addr() -> EvmAddress {
	EvmAddress::from_str("1000000000000000000000000000000000000002").unwrap()
}

pub fn dnar_evm_address() -> EvmAddress {
	EvmAddress::try_from(DNAR).unwrap()
}

pub fn usdj_evm_address() -> EvmAddress {
	EvmAddress::try_from(SETUSD).unwrap()
}

pub fn renbtc_evm_address() -> EvmAddress {
	EvmAddress::try_from(RENBTC).unwrap()
}

pub fn lp_dnar_usdj_evm_address() -> EvmAddress {
	EvmAddress::try_from(LP_DNAR_SETUSD).unwrap()
}

pub fn erc20_address_not_exists() -> EvmAddress {
	EvmAddress::from_str("0000000000000000000000000000000200000001").unwrap()
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
		Currencies: setheum_currencies::{Pallet, Call, Event<T>},
		EVMBridge: setheum_evm_bridge::{Pallet},
		EVMManager: setheum_evm_manager::{Pallet, Storage},
		NFTModule: setheum_nft::{Pallet, Call, Event<T>},
		TransactionPayment: setheum_transaction_payment::{Pallet, Call, Storage},
		Prices: setheum_prices::{Pallet, Storage, Call, Event<T>},
		Proxy: pallet_proxy::{Pallet, Call, Storage, Event<T>},
		Utility: pallet_utility::{Pallet, Call, Event},
		Scheduler: pallet_scheduler::{Pallet, Call, Storage, Event<T>},
		DexModule: setheum_dex::{Pallet, Storage, Call, Event<T>, Config<T>},
		ModuleEVM: setheum_evm::{Pallet, Config<T>, Call, Storage, Event<T>},
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
		setheum_evm::GenesisAccount {
			nonce: 1,
			balance: INITIAL_BALANCE,
			storage: Default::default(),
			code: Default::default(),
		},
	);
	accounts.insert(
		bob_evm_addr(),
		setheum_evm::GenesisAccount {
			nonce: 1,
			balance: INITIAL_BALANCE,
			storage: Default::default(),
			code: Default::default(),
		},
	);

	pallet_balances::GenesisConfig::<Test>::default()
		.assimilate_storage(&mut storage)
		.unwrap();
	setheum_evm::GenesisConfig::<Test> {
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
			RENBTC,
			1_000_000_000_000
		));
		assert_ok!(Currencies::update_balance(Origin::root(), ALICE, SETUSD, 1_000_000_000));

		assert_ok!(Currencies::update_balance(
			Origin::root(),
			MockAddressMapping::get_account_id(&alice_evm_addr()),
			RENBTC,
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
	num[..].copy_from_slice(&output[32 - 4..32]);
	let task_id_len: u32 = u32::from_be_bytes(num);
	return output[32..32 + task_id_len as usize].to_vec();
}
