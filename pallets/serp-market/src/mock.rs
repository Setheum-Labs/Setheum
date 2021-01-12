use crate::{Module, Trait};
use frame_support::{impl_outer_origin, parameter_types, weights::Weight};
use frame_system as system;
use sp_core::H256;
use sp_runtime::{
    testing::Header,
    traits::{BlakeTwo256, IdentityLookup},
    Perbill,
};

impl_outer_origin! {
    pub enum Origin for Test {}
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

#[derive(Clone, Eq, PartialEq)]
pub struct Test;
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

type AccountId = u32;
type BlockNumber = u64;

impl system::Trait for Test {
    type BaseCallFilter = ();
    type Origin = Origin;
    type Call = ();
    type Index = u64;
    type BlockNumber = u64;
    type Hash = H256;
    type Hashing = BlakeTwo256;
    type AccountId = u64;
    type Lookup = IdentityLookup<Self::AccountId>;
    type Header = Header;
    type Event = ();
    type BlockHashCount = BlockHashCount;
    type MaximumBlockWeight = MaximumBlockWeight;
    type DbWeight = ();
    type BlockExecutionWeight = ();
    type ExtrinsicBaseWeight = ();
    type MaximumExtrinsicWeight = MaximumBlockWeight;
    type MaximumBlockLength = MaximumBlockLength;
    type AvailableBlockRatio = AvailableBlockRatio;
    type Version = ();
    type PalletInfo = ();
    type AccountData = ();
    type OnNewAccount = ();
    type OnKilledAccount = ();
    type SystemWeightInfo = ();
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

pub type System = system::Module<Test>;
pub type Stp258 = Module<Test>;

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
	let shareholders: Vec<(AccountId, u64)> = shareholders.into_iter().zip(iter::repeat(1)).collect();
	// make sure to run our storage build function to check config
	let _ = GenesisConfig::<Test> { shareholders }.assimilate_storage(&mut storage);
	storage.into()
}

// ------------------------------------------------------------
// utils
pub type DinarT = Dinar<AccountId>;
// Implementation that we will instantiate.
pub type Transient =
	BoundedDeque<DinarT, <SettCurrency as Store>::DinarRange, <SettCurrency as Store>::Dinar, DinarIndex>;

pub fn add_dinar(dinar: DinarT) {
	let mut dinar = Transient::new();
	dinar.push_back(dinar);
	dinar.commit();
}
