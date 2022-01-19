//! Mocks for the vesting module.

#![cfg(test)]

use super::*;
use frame_support::{
	construct_runtime, parameter_types,
	traits::{EnsureOrigin, Everything},
};
use frame_system::RawOrigin;
use orml_traits::parameter_type_with_key;
use primitives::{Amount, TokenSymbol};
use sp_core::H256;
use sp_runtime::{testing::Header, traits::IdentityLookup};

use crate as vesting;

parameter_types! {
	pub const BlockHashCount: u64 = 250;
}

pub type AccountId = u128;

pub const SETR: CurrencyId = CurrencyId::Token(TokenSymbol::SETR);
pub const SETUSD: CurrencyId = CurrencyId::Token(TokenSymbol::SETUSD);
pub const SETM: CurrencyId = CurrencyId::Token(TokenSymbol::SETM);
pub const SERP: CurrencyId = CurrencyId::Token(TokenSymbol::SERP);
pub const DNAR: CurrencyId = CurrencyId::Token(TokenSymbol::DNAR);
pub const HELP: CurrencyId = CurrencyId::Token(TokenSymbol::HELP);

impl frame_system::Config for Runtime {
	type Origin = Origin;
	type Call = Call;
	type Index = u64;
	type BlockNumber = u64;
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
	type AccountData = pallet_balances::AccountData<u64>;
	type OnNewAccount = ();
	type OnKilledAccount = ();
	type DbWeight = ();
	type BaseCallFilter = Everything;
	type SystemWeightInfo = ();
	type SS58Prefix = ();
	type OnSetCode = ();
}

type Balance = u64;

parameter_types! {
	pub const ExistentialDeposit: u64 = 1;
	pub const MaxLocks: u32 = 100;
}

impl pallet_balances::Config for Runtime {
	type Balance = Balance;
	type DustRemoval = ();
	type Event = Event;
	type ExistentialDeposit = ExistentialDeposit;
	type AccountStore = frame_system::Pallet<Runtime>;
	type MaxLocks = MaxLocks;
	type MaxReserves = ();
	type ReserveIdentifier = [u8; 8];
	type WeightInfo = ();
}

pub struct EnsureAliceOrBob;
impl EnsureOrigin<Origin> for EnsureAliceOrBob {
	type Success = AccountId;

	fn try_origin(o: Origin) -> Result<Self::Success, Origin> {
		Into::<Result<RawOrigin<AccountId>, Origin>>::into(o).and_then(|o| match o {
			RawOrigin::Signed(ALICE) => Ok(ALICE),
			RawOrigin::Signed(BOB) => Ok(BOB),
			r => Err(Origin::from(r)),
		})
	}

	#[cfg(feature = "runtime-benchmarks")]
	fn successful_origin() -> Origin {
		Origin::from(RawOrigin::Signed(Default::default()))
	}
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
	type MaxLocks = MaxLocks;
	type DustRemovalWhitelist = ();
}

parameter_types! {
	pub StableCurrencyIds: Vec<CurrencyId> = vec![
		SETR,
		SETUSD,
	];
	pub const SetterCurrencyId: CurrencyId = SETR;  	// Setter  currency ticker is SETR/
	pub const GetSetUSDId: CurrencyId = SETUSD;  		// SetDollar currency ticker is SETUSD/
	pub const GetNativeCurrencyId: CurrencyId = SETM;  	// Setheum native currency ticker is SETM/
	pub const GetSerpCurrencyId: CurrencyId = SERP;  	// Serp currency ticker is SERP/
	pub const GetDinarCurrencyId: CurrencyId = DNAR;  	// The Dinar currency ticker is DNAR/
	pub const GetHelpCurrencyId: CurrencyId = HELP;  	// HighEnd LaunchPad currency ticker is HELP/
	pub static MockBlockNumberProvider: u64 = 0;
	pub const TreasuryAccount: AccountId = TREASURY;
}

impl BlockNumberProvider for MockBlockNumberProvider {
	type BlockNumber = u64;

	fn current_block_number() -> Self::BlockNumber {
		Self::get()
	}
}

parameter_types! {
	pub const MaxNativeVestingSchedules: u32 = 2;
	pub const MaxSerpVestingSchedules: u32 = 2;
	pub const MaxDinarVestingSchedules: u32 = 2;
	pub const MaxHelpVestingSchedules: u32 = 2;
	pub const MaxSetterVestingSchedules: u32 = 2;
	pub const MaxSetUSDVestingSchedules: u32 = 2;
	pub const MinVestedTransfer: u64 = 5;
}

impl Config for Runtime {
	type Event = Event;
	type MultiCurrency = Tokens;
	type GetNativeCurrencyId = GetNativeCurrencyId;
	type GetSerpCurrencyId = GetSerpCurrencyId;
	type GetDinarCurrencyId = GetDinarCurrencyId;
	type GetHelpCurrencyId = GetHelpCurrencyId;
	type SetterCurrencyId = SetterCurrencyId;
	type GetSetUSDId = GetSetUSDId;
	type MinVestedTransfer = MinVestedTransfer;
	type TreasuryAccount = TreasuryAccount;
	type UpdateOrigin = EnsureAliceOrBob;
	type WeightInfo = ();
	type MaxNativeVestingSchedules = MaxNativeVestingSchedules;
	type MaxSerpVestingSchedules = MaxSerpVestingSchedules;
	type MaxDinarVestingSchedules = MaxDinarVestingSchedules;
	type MaxHelpVestingSchedules = MaxHelpVestingSchedules;
	type MaxSetterVestingSchedules = MaxSetterVestingSchedules;
	type MaxSetUSDVestingSchedules = MaxSetUSDVestingSchedules;
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
		Vesting: vesting::{Pallet, Storage, Call, Event<T>, Config<T>},
		PalletBalances: pallet_balances::{Pallet, Call, Storage, Config<T>, Event<T>},
		Tokens: orml_tokens::{Pallet, Storage, Event<T>, Config<T>}
	}
);

pub const ALICE: AccountId = 1;
pub const BOB: AccountId = 2;
pub const CHARLIE: AccountId = 3;
pub const TREASURY: AccountId = 4;

pub struct ExtBuilder {
	balances: Vec<(AccountId, CurrencyId, Balance)>,
}

impl Default for ExtBuilder {
	fn default() -> Self {
		Self {
			balances: vec![
				(ALICE, SETM, 10000),
				(ALICE, SERP, 1000),
				(ALICE, DNAR, 1000),
				(ALICE, HELP, 1000),
				(ALICE, SETR, 1000),
				(ALICE, SETUSD, 1000),
				(CHARLIE, SETM, 10000),
				(CHARLIE, SERP, 1000),
				(CHARLIE, DNAR, 1000),
				(CHARLIE, HELP, 1000),
				(CHARLIE, SETR, 1000),
				(CHARLIE, SETUSD, 1000),
				(TREASURY, SETM, 10000),
				(TREASURY, SERP, 1000),
				(TREASURY, DNAR, 1000),
				(TREASURY, HELP, 1000),
				(TREASURY, SETR, 1000),
				(TREASURY, SETUSD, 1000)
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
			balances: self.balances,
		}
		.assimilate_storage(&mut t)
		.unwrap();

		vesting::GenesisConfig::<Runtime> {
			// who, start, period, period_count, per_period
			vesting: vec![(CHARLIE, SETM, 2, 3, 4, 5)],
		}
		.assimilate_storage(&mut t)
		.unwrap();

		t.into()
	}
}