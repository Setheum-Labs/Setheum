//! Mocks for the vesting module.

#![cfg(test)]

use super::*;
use frame_support::{
	construct_runtime, parameter_types,
	traits::{ConstU32, ConstU64, EnsureOrigin, Everything, Nothing},
};
use frame_system::RawOrigin;
use orml_traits::parameter_type_with_key;
use primitives::{Amount, TokenSymbol};
use sp_core::H256;
use sp_runtime::{traits::IdentityLookup, BuildStorage};

mod vesting {
	pub use super::super::*;
}

pub type AccountId = u128;

impl frame_system::Config for Runtime {
	type RuntimeOrigin = RuntimeOrigin;
	type RuntimeCall = RuntimeCall;
	type Nonce = u64;
	type Hash = H256;
	type Hashing = ::sp_runtime::traits::BlakeTwo256;
	type AccountId = AccountId;
	type Lookup = IdentityLookup<Self::AccountId>;
	type Block = Block;
	type RuntimeEvent = RuntimeEvent;
	type BlockHashCount = ConstU64<250>;
	type BlockWeights = ();
	type BlockLength = ();
	type Version = ();
	type PalletInfo = PalletInfo;
	type AccountData = ();
	type OnNewAccount = ();
	type OnKilledAccount = ();
	type DbWeight = ();
	type BaseCallFilter = Everything;
	type SystemWeightInfo = ();
	type SS58Prefix = ();
	type OnSetCode = ();
	type MaxConsumers = ConstU32<16>;
}

type Balance = u64;

parameter_type_with_key! {
	pub ExistentialDeposits: |_currency_id: CurrencyId| -> Balance {
		Default::default()
	};
}

impl orml_tokens::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type Balance = Balance;
	type Amount = Amount;
	type CurrencyId = CurrencyId;
	type WeightInfo = ();
	type ExistentialDeposits = ExistentialDeposits;
	type CurrencyHooks = ();
	type MaxLocks = ();
	type MaxReserves = ();
	type ReserveIdentifier = [u8; 8];
	type DustRemovalWhitelist = Nothing;
}

pub struct EnsureAliceOrBob;
impl EnsureOrigin<RuntimeOrigin> for EnsureAliceOrBob {
	type Success = AccountId;

	fn try_origin(o: RuntimeOrigin) -> Result<Self::Success, RuntimeOrigin> {
		Into::<Result<RawOrigin<AccountId>, RuntimeOrigin>>::into(o).and_then(|o| match o {
			RawOrigin::Signed(ALICE) => Ok(ALICE),
			RawOrigin::Signed(BOB) => Ok(BOB),
			r => Err(RuntimeOrigin::from(r)),
		})
	}

	#[cfg(feature = "runtime-benchmarks")]
	fn try_successful_origin() -> Result<RuntimeOrigin, ()> {
		let zero_account_id = AccountId::decode(&mut sp_runtime::traits::TrailingZeroInput::zeroes())
			.expect("infinite length input; no invalid inputs for type; qed");
		Ok(RuntimeOrigin::from(RawOrigin::Signed(zero_account_id)))
	}
}

pub const SEE: CurrencyId = CurrencyId::Token(TokenSymbol::SEE);
pub const EDF: CurrencyId = CurrencyId::Token(TokenSymbol::edf);

parameter_types! {
	pub const GetNativeCurrencyId: CurrencyId = SEE;
	pub const GetDEFCurrencyId: CurrencyId = EDF;
}

impl Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type MultiCurrency = Tokens;
	type GetNativeCurrencyId = GetNativeCurrencyId;
	type GetEDFCurrencyId = GetEDFCurrencyId;
	type MinNativeVestedTransfer = ConstU64<5>;
	type MinEDFVestedTransfer = ConstU64<5>;
	type VestedTransferOrigin = EnsureAliceOrBob;
	type WeightInfo = ();
	type MaxNativeVestingSchedules = ConstU32<2>;
	type MaxEDFVestingSchedules = ConstU32<2>;
}

type Block = frame_system::mocking::MockBlock<Runtime>;

construct_runtime!(
	pub enum Runtime {
		System: frame_system,
		Vesting: vesting,
		Tokens: orml_tokens,
	}
);

pub const ALICE: AccountId = 1;
pub const BOB: AccountId = 2;
pub const CHARLIE: AccountId = 3;

#[derive(Default)]
pub struct ExtBuilder {
	balances: Vec<(AccountId, CurrencyId, Balance)>,
}

impl Default for ExtBuilder {
	fn default() -> Self {
		Self {
			balances: vec![
				(ALICE, SEE, 100),
				(ALICE, EDF, 100),
				(CHARLIE, SEE, 50)
				(CHARLIE, EDF, 50)
			],
		}
	}
}

impl ExtBuilder {
	pub fn build() -> sp_io::TestExternalities {
		let mut t = frame_system::GenesisConfig::<Runtime>::default()
			.build_storage()
			.unwrap();

		orml_tokens::GenesisConfig::<Runtime> {
			balances: self.balances,
		}
		.assimilate_storage(&mut t)
		.unwrap();

		vesting::GenesisConfig::<Runtime> {
			vesting: vec![
				// who, start, period, period_count, per_period
				(CHARLIE, SEE, 2, 3, 1, 5),
				(CHARLIE, SEE, 2 + 3, 3, 3, 5),
				(CHARLIE, EDF, 2, 3, 1, 5),
				(CHARLIE, EDF, 2 + 3, 3, 3, 5),
			],
		}
		.assimilate_storage(&mut t)
		.unwrap();

		t.into()
	}
}
