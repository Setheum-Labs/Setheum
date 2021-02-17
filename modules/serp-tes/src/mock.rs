//! Mock for the sert-tes module.

use crate::{Module, Trait};
use frame_support::{impl_outer_origin, parameter_types, weights::Weight};
use serml_traits::*;
use frame_system as system;
use sp_core::H256;
use sp_runtime::{
	testing::Header,
	traits::{AccountIdConversion, IdentityLookup},
	AccountId32, ModuleId, Fixed64, Perbill,
};

use stp258;
use serp_market;

use super::*;
use itertools::Itertools;
use log;
use more_asserts::*;
use quickcheck::{QuickCheck, TestResult};
use rand::{thread_rng, Rng};
use std::sync::atomic::{AtomicU64, Ordering};

use sp_std::iter;
use system;

mod serp_tes {
	pub use crate::Event;
}

impl_outer_origin! {
	pub enum Origin for Runtime {}
}

const TEST_BASE_UNIT: u64 = 1000;
static LAST_PRICE: AtomicU64 = AtomicU64::new(TEST_BASE_UNIT);
pub struct RandomPrice;

impl FetchPrice<SettCurrency> for RandomPrice {
	fn fetch_price() -> SettCurrency {
		let prev = LAST_PRICE.load(Ordering::SeqCst);
		let random = thread_rng().gen_range(500, 1500);
		let ratio: Ratio<u64> = Ratio::new(random, 1000);
		let next = ratio
			.checked_mul(&prev.into())
			.map(|r| r.to_integer())
			.unwrap_or(prev);
		LAST_PRICE.store(next + 1, Ordering::SeqCst);
		prev
	}
}

// Configure a mock runtime to test the pallet.
// For testing the pallet, we construct most of a mock runtime. This means
// first constructing a configuration type (`Test`) which `impl`s each of the
// configuration traits of modules we want to use.

// Workaround for https://github.com/rust-lang/rust/issues/26925 . Remove when sorted.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Runtime;
parameter_types! {
    pub const BlockHashCount: u64 = 250;
    pub const MaximumBlockWeight: Weight = 1024;
    pub const MaximumBlockLength: u32 = 2 * 1024;
    pub const AvailableBlockRatio: Perbill = Perbill::from_percent(75);
    
    // allow few bids
	pub const MaximumBids: u64 = 10;
	// adjust supply every second block
	pub const ElastAdjustmentFrequency: u64 = 2;
	pub const BaseUnit: u64 = TEST_BASE_UNIT;
	pub const InitialSupply: u64 = 100 * BaseUnit::get();
	pub const MinimumSupply: u64 = BaseUnit::get();
	pub const MinimumDinarPrice: Perbill = Perbill::from_percent(10);
}

pub type AccountId = AccountId32;
pub type BlockNumber = u64;

impl system::Config for Runtime {
	type Origin = Origin;
	type Call = ();
	type Index = u64;
	type BlockNumber = u64;
	type Hash = H256;
	type Hashing = ::sp_runtime::traits::BlakeTwo256;
	type AccountId = AccountId;
	type Lookup = IdentityLookup<Self::AccountId>;
	type Header = Header;
	type Event = TestEvent;
	type BlockHashCount = BlockHashCount;
	type BlockWeights = ();
	type BlockLength = ();
	type Version = ();
	type PalletInfo = ();
	type AccountData = pallet_balances::AccountData<u64>;
	type OnNewAccount = ();
	type OnKilledAccount = ();
	type DbWeight = ();
	type BaseCallFilter = ();
	type SystemWeightInfo = ();
    type DbWeight = ();
    type BlockExecutionWeight = ();
    type ExtrinsicBaseWeight = ();
    type MaximumBlockWeight = MaximumBlockWeight;
    type MaximumExtrinsicWeight = MaximumBlockWeight;
    type MaximumBlockLength = MaximumBlockLength;
    type AvailableBlockRatio = AvailableBlockRatio;
}      

impl Trait for Test {
    type Event = ();
    type SettCurrencyPrice = RandomPrice;
	type MaximumBids = MaximumBids;
	type ElastAdjustmentFrequency = ElastAdjustmentFrequency;
	type BaseUnit = BaseUnit;
	type InitialSupply = InitialSupply;
	type MinimumSupply = MinimumSupply;
	type MinimumDinarPrice = MinimumDinarPrice;
}

pub type System = frame_system::Module<Runtime>;
pub type SerpTes = Module<Runtime>;

type CurrencyId = u32;
type Balance = u64;


parameter_types! {
	pub const ExistentialDeposit: u64 = 1;
}

impl pallet_balances::Config for Runtime {
	type Balance = Balance;
	type DustRemoval = ();
	type Event = TestEvent;
	type ExistentialDeposit = ExistentialDeposit;
	type AccountStore = frame_system::Module<Runtime>;
	type MaxLocks = ();
	type WeightInfo = ();
}
pub type PalletBalances = pallet_balances::Module<Runtime>;

parameter_type_with_key! {
	pub ExistentialDeposits: |currency_id: CurrencyId| -> Balance {
		Default::default()
	};
}

parameter_types! {
	pub DustAccount: AccountId = ModuleId(*b"orml/dst").into_account();
}

impl orml_tokens::Config for Runtime {
	type Event = TestEvent;
	type Balance = Balance;
	type Amount = i64;
	type CurrencyId = CurrencyId;
	type WeightInfo = ();
	type ExistentialDeposits = ExistentialDeposits;
	type OnDust = orml_tokens::TransferDust<Runtime, DustAccount>;
}
pub type Tokens = orml_tokens::Module<Runtime>;

pub const NATIVE_CURRENCY_ID: CurrencyId = CurrencyId::Token(TokenSymbol::DNAR);
pub const SETT_USD_ID: CurrencyId = CurrencyId::Token(TokenSymbol::JUSD);

parameter_types! {
	pub const GetNativeCurrencyId: CurrencyId = NATIVE_CURRENCY_ID;
}

impl Config for Runtime {
	type Event = TestEvent;
	type SettCurrency = Tokens;
	type NativeCurrency = AdaptedBasicCurrency;
	type GetNativeCurrencyId = GetNativeCurrencyId;
	type WeightInfo = ();
}
pub type Stp258 = Module<Runtime>;
pub type NativeCurrency = NativeCurrencyOf<Runtime>;
pub type AdaptedBasicCurrency = BasicCurrencyAdapter<Runtime, PalletBalances, i64, u64>;

pub const ALICE: AccountId = AccountId32::new([1u8; 32]);
pub const BOB: AccountId = AccountId32::new([2u8; 32]);
pub const EVA: AccountId = AccountId32::new([5u8; 32]);
pub const ID_1: LockIdentifier = *b"1       ";


// Build genesis storage according to the mock runtime.
pub fn new_test_ext() -> sp_io::TestExternalities {
    system::GenesisConfig::default()
        .build_storage::<Test>()
        .unwrap()
        .into()
}

pub fn new_test_ext() -> sp_io::TestExternalities {
	let mut storage = system::GenesisConfig::default().build_storage::<Test>().unwrap();
	let shareholders: Vec<(AccountId, u64)> = vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10]
		.into_iter()
		.zip(iter::repeat(1))
		.collect();
	// make sure to run our storage build function to check config
	let _ = GenesisConfig::<Test> { shareholders }.assimilate_storage(&mut storage);
	storage.into()
}

pub fn new_test_ext_with(shareholders: Vec<AccountId>) -> sp_io::TestExternalities {
	let mut storage = system::GenesisConfig::default().build_storage::<Test>().unwrap();
	let shareholders: Vec<(AccountId, u64)> = shareholders.into_iter().zip(iter::repeat(1)).collect::<Vec<PathBuf>>();
	// make sure to run our storage build function to check config
	let _ = GenesisConfig::<Test> { shareholders }.assimilate_storage(&mut storage);
	storage.into()
}

///-----------------------------------------------------------------------------------



pub struct ExtBuilder {
	endowed_accounts: Vec<(AccountId, CurrencyId, Balance)>,
}

impl Default for ExtBuilder {
	fn default() -> Self {
		Self {
			endowed_accounts: vec![],
		}
	}
}

impl ExtBuilder {
	pub fn balances(mut self, endowed_accounts: Vec<(AccountId, CurrencyId, Balance)>) -> Self {
		self.endowed_accounts = endowed_accounts;
		self
	}

	pub fn one_hundred_for_alice_n_bob(self) -> Self {
		self.balances(vec![
			(ALICE, NATIVE_CURRENCY_ID, 100),
			(BOB, NATIVE_CURRENCY_ID, 100),
			(ALICE, SETT_USD_ID, 100),
			(BOB, SETT_USD_ID, 100),
		])
	}

	pub fn build(self) -> sp_io::TestExternalities {
		let mut t = frame_system::GenesisConfig::default()
			.build_storage::<Runtime>()
			.unwrap();

		pallet_balances::GenesisConfig::<Runtime> {
			balances: self
				.endowed_accounts
				.clone()
				.into_iter()
				.filter(|(_, currency_id, _)| *currency_id == NATIVE_CURRENCY_ID)
				.map(|(account_id, _, initial_balance)| (account_id, initial_balance))
				.collect::<Vec<_>>(),
		}
		.assimilate_storage(&mut t)
		.unwrap();

		orml_tokens::GenesisConfig::<Runtime> {
			endowed_accounts: self
				.endowed_accounts
				.into_iter()
				.filter(|(_, currency_id, _)| *currency_id != NATIVE_CURRENCY_ID)
				.collect::<Vec<_>>(),
		}
		.assimilate_storage(&mut t)
		.unwrap();

		t.into()
	}
}
