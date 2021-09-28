#![cfg(test)]

use crate::{AllPrecompiles, Ratio, RuntimeBlockWeights, SystemContractsFilter, Weight};
use setheum_node::chain_spec::evm_genesis;
use codec::{Decode, Encode};
use frame_support::{
	assert_ok, ord_parameter_types, parameter_types,
	traits::{GenesisBuild, InstanceFilter, OnFinalize, OnInitialize, SortedMembers},
	weights::IdentityFee, RuntimeDebug,
};
use frame_system::{EnsureRoot, EnsureSignedBy};
use module_support::{
	mocks::MockAddressMapping, AddressMapping as AddressMappingT,
};
use orml_traits::{parameter_type_with_key, MultiReservableCurrency};
pub use primitives::{
	evm::EvmAddress, TradingPair, DexShare, TokenSymbol,
	Amount, BlockNumber, CurrencyId, Header, Nonce,
};
use sha3::{Digest, Keccak256};
use sp_core::{crypto::AccountId32, H160, H256};
use sp_runtime::{
	traits::{BlakeTwo256, Convert, IdentityLookup, One as OneT},
	DispatchResult, FixedPointNumber, FixedU128, Perbill, ModuleId,
};
use sp_std::{
	collections::btree_map::BTreeMap,
	convert::{TryFrom, TryInto},
	str::FromStr,
};

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
	type Index = u64;
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
}

impl pallet_balances::Config for Test {
	type Balance = Balance;
	type DustRemoval = ();
	type Event = Event;
	type ExistentialDeposit = ExistentialDeposit;
	type AccountStore = System;
	type WeightInfo = ();
	type MaxLocks = ();
}

pub const SETHEUM: CurrencyId = CurrencyId::Token(TokenSymbol::SETHEUM);
pub const DNAR: CurrencyId = CurrencyId::Token(TokenSymbol::DNAR);
pub const SETR: CurrencyId = CurrencyId::Token(TokenSymbol::SETR);
pub const SETEUR: CurrencyId = CurrencyId::Token(TokenSymbol::SETEUR);
pub const SETUSD: CurrencyId = CurrencyId::Token(TokenSymbol::SETUSD);
pub const RENBTC: CurrencyId = CurrencyId::Token(TokenSymbol::RENBTC);
pub const LP_SETHEUM_SETUSD: CurrencyId =
	CurrencyId::DexShare(DexShare::Token(TokenSymbol::SETHEUM), DexShare::Token(TokenSymbol::SETUSD));
pub const LP_DNAR_SETUSD: CurrencyId =
	CurrencyId::DexShare(DexShare::Token(TokenSymbol::SETHEUM), DexShare::Token(TokenSymbol::SETUSD));
pub const LP_SETR_SETUSD: CurrencyId =
	CurrencyId::DexShare(DexShare::Token(TokenSymbol::SETHEUM), DexShare::Token(TokenSymbol::SETUSD));
pub const LP_SETEUR_SETUSD: CurrencyId =
	CurrencyId::DexShare(DexShare::Token(TokenSymbol::SETHEUM), DexShare::Token(TokenSymbol::SETUSD));
pub const LP_RENBTC_SETUSD: CurrencyId =
	CurrencyId::DexShare(DexShare::Token(TokenSymbol::SETHEUM), DexShare::Token(TokenSymbol::SETUSD));

parameter_types! {
	pub const GetNativeCurrencyId: CurrencyId = SETHEUM;
	pub AirdropMinimum: u32 = 2;
	pub AirdropMaximum: u32 = 3;
}

impl module_currencies::Config for Test {
	type Event = Event;
	type MultiCurrency = Tokens;
	type NativeCurrency = AdaptedBasicCurrency;
	type GetNativeCurrencyId = GetNativeCurrencyId;
    type SerpTreasury = ();
	type AirdropAccountId = ();
	type AirdropMinimum = AirdropMinimum;
	type AirdropMaximum = AirdropMaximum;
	type AirdropOrigin = ();
	type WeightInfo = ();
	type AddressMapping = MockAddressMapping;
	type EVMBridge = EVMBridge;
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
	pub const NftModuleId: ModuleId = ModuleId(*b"set/aNFT");
}
impl module_nft::Config for Test {
	type Event = Event;
	type CreateClassDeposit = CreateClassDeposit;
	type CreateTokenDeposit = CreateTokenDeposit;
	type ModuleId = NftModuleId;
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
	pub const GetStableCurrencyId: CurrencyId = CurrencyId::Token(TokenSymbol::SETUSD);
	pub MaxSwapSlippageCompareToOracle: Ratio = Ratio::one();
	pub DefaultFeeSwapPathList: Vec<Vec<CurrencyId>> = 
		vec![
			vec![SETUSD, SETHEUM],
			vec![DNAR, SETUSD, SETHEUM]
			vec![SETEUR, SETUSD, SETHEUM]
			vec![RENBTC, SETUSD, SETHEUM]
		];
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
	type PriceSource = MockPriceSource;
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

#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Encode, Decode, RuntimeDebug)]
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
	pub StableCurrencyIds: Vec<CurrencyId> = vec![
		SETR,
		SETUSD,
		SETEUR,
	];
	pub const GetExchangeFee: (u32, u32) = (1, 100); // 1%
	pub const TradingPathLimit: u32 = 3;
	pub const GetStableCurrencyExchangeFee: (u32, u32) = (1, 200); // 0.5%
	pub const BuyBackPoolAccountId: AccountId = BUYBACK_POOL;
	pub const DEXModuleId: ModuleId = ModuleId(*b"set/sdex");
}

impl module_dex::Config for Test {
	type Event = Event;
	type Currency = Tokens;
	type StableCurrencyIds = StableCurrencyIds;
	type GetExchangeFee = GetExchangeFee;
	type GetStableCurrencyExchangeFee = GetStableCurrencyExchangeFee;
	type BuyBackPoolAccountId = BuyBackPoolAccountId;
	type TradingPathLimit = TradingPathLimit;
	type ModuleId = DEXModuleId;
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

impl module_evm::Config for Test {
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

parameter_types! {
	pub const GetSetUSDCurrencyId: CurrencyId = SETUSD; // Setter currency ticker is SETUSD.
	pub const SetterCurrencyId: CurrencyId = SETR; // Setter currency ticker is SETR.
	pub const SetterCurrencyId: CurrencyId = SETR; // Setter currency ticker is SETR.
	pub SetUSDFixedPrice: Price = Price::saturating_from_rational(1, 1);
	pub SetterFixedPrice: Price = Price::saturating_from_rational(1, 1);
}

ord_parameter_types! {
	pub const One: AccountId = AccountId::new([1u8; 32]);
}

impl module_prices::Config for Test {
	type Event = Event;
	type Source = Oracle;
	type GetSetUSDCurrencyId = GetSetUSDCurrencyId;
	type SetterCurrencyId = GetSetUSDCurrencyId;
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

pub fn setheum_evm_address() -> EvmAddress {
	EvmAddress::try_from(SETHEUM).unwrap()
}

pub fn setusd_evm_address() -> EvmAddress {
	EvmAddress::try_from(SETUSD).unwrap()
}

pub fn renbtc_evm_address() -> EvmAddress {
	EvmAddress::try_from(RENBTC).unwrap()
}

pub fn lp_setheum_setusd_evm_address() -> EvmAddress {
	EvmAddress::try_from(LP_SETHEUM_SETUSD).unwrap()
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
		System: frame_system::{Module, Call, Storage, Config, Event<T>},
		Oracle: orml_oracle::{Pallet, Storage, Call, Event<T>},
		Timestamp: pallet_timestamp::{Module, Call, Storage, Inherent},
		Tokens: orml_tokens::{Module, Storage, Event<T>, Config<T>},
		Balances: pallet_balances::{Module, Call, Storage, Config<T>, Event<T>},
		Currencies: module_currencies::{Module, Call, Event<T>},
		EVMBridge: module_evm_bridge::{Module},
		EVMManager: module_evm_manager::{Pallet, Storage},
		TransactionPayment: module_transaction_payment::{Module, Call, Storage},
		Prices: module_prices::{Pallet, Storage, Call, Event<T>},
		Proxy: pallet_proxy::{Module, Call, Storage, Event<T>},
		Utility: pallet_utility::{Module, Call, Event},
		Scheduler: pallet_scheduler::{Module, Call, Storage, Event<T>},
		DexModule: module_dex::{Pallet, Storage, Call, Event<T>, Config<T>},
		ModuleEVM: module_evm::{Module, Config<T>, Call, Storage, Event<T>},
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
		alice(),
		module_evm::GenesisAccount {
			nonce: 1,
			balance: INITIAL_BALANCE,
			storage: Default::default(),
			code: Default::default(),
		},
	);
	accounts.insert(
		bob(),
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
			SETHEUM,
			1_000_000_000_000
		));
		assert_ok!(Currencies::update_balance(Origin::root(), ALICE, SETUSD, 1_000_000_000));

		assert_ok!(Currencies::update_balance(
			Origin::root(),
			MockAddressMapping::get_account_id(&alice()),
			SETUSD,
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

pub fn get_function_selector(s: &str) -> [u8; 4] {
	// create a SHA3-256 object
	let mut hasher = Keccak256::new();
	// write input message
	hasher.update(s);
	// read hash digest
	let result = hasher.finalize();
	result[..4].try_into().unwrap()
}
