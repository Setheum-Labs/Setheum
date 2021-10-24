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

#![cfg_attr(not(feature = "std"), no_std)]
// `construct_runtime!` does a lot of recursion and requires us to increase the limit to 256.
#![recursion_limit="256"]

// Make the WASM binary available.
#[cfg(feature = "std")]
include!(concat!(env!("OUT_DIR"), "/wasm_binary.rs"));

use codec::Encode;

use sp_std::prelude::*;
use sp_core::{
	crypto::KeyTypeId,
	u32_trait::{_1, _2, _3, _4},
	H160, OpaqueMetadata, Decode,
};
use sp_runtime::{
	ApplyExtrinsicResult, generic, create_runtime_str, impl_opaque_keys,
	transaction_validity::{TransactionValidity, TransactionSource, TransactionPriority},
	curve::PiecewiseLinear,
	FixedPointNumber,
};
use sp_runtime::traits::{
	BlakeTwo256,
	Block as BlockT,
	NumberFor,
	Zero,
	SaturatedConversion,
	StaticLookup,
	BadOrigin,
	OpaqueKeys,
};
pub use sp_runtime::{
	Perbill, Percent, Permill, Perquintill,
	DispatchResult,
};
use sp_api::impl_runtime_apis;
use pallet_grandpa::{AuthorityId as GrandpaId, AuthorityList as GrandpaAuthorityList};
use pallet_grandpa::fg_primitives;
pub use pallet_im_online::sr25519::AuthorityId as ImOnlineId;
pub use sp_authority_discovery::AuthorityId as AuthorityDiscoveryId;


use sp_version::RuntimeVersion;
#[cfg(feature = "std")]
use sp_version::NativeVersion;

// A few exports that help ease life for downstream crates.
#[cfg(any(feature = "std", test))]
pub use pallet_timestamp::Call as TimestampCall;
pub use pallet_balances::Call as BalancesCall;
pub use frame_support::{
	construct_runtime, parameter_types, debug,
	StorageValue,
	traits::{
		WithdrawReasons,
		KeyOwnerProofSystem, Randomness, EnsureOrigin, OriginTrait, U128CurrencyToVote,
		schedule::Priority,
	},
	weights::{
		Weight, IdentityFee,
		constants::{BlockExecutionWeight, ExtrinsicBaseWeight, RocksDbWeight, WEIGHT_PER_SECOND},
	},
};
pub use frame_system::{ensure_root, EnsureOneOf, EnsureRoot, RawOrigin};

use static_assertions::const_assert;

use orml_tokens::CurrencyAdapter;
use orml_traits::{create_median_value_data_provider, parameter_type_with_key, DataFeeder, DataProviderExtended};
use orml_authority::EnsureDelayed;

use module_evm::{CallInfo, CreateInfo};
use module_evm_accounts::EvmAddressMapping;
use module_currencies::{BasicCurrencyAdapter, Currency};
use module_transaction_payment::{Multiplier, TargetedFeeAdjustment};

// re-exports

pub use pallet_staking::StakerStatus;
pub use primitives::{
	evm::EstimateResourcesRequest, AuthoritysOriginId,
	AccountId, AccountIndex, Amount, Balance, BlockNumber,
	currency::{TokenInfo, SETM, SERP, DNAR, SETR, SETUSD, RENBTC},
	CurrencyId, EraIndex, Hash, Moment, Nonce, Signature, TokenSymbol,
	PRECOMPILE_ADDRESS_START, PREDEPLOY_ADDRESS_START, SYSTEM_CONTRACT_ADDRESS_PREFIX,
};
pub use module_support::{Contains, ExchangeRate, PrecompileCallerFilter, Price, Rate, Ratio};

pub use runtime_common::{};

pub use primitives::{currency::*, time::*};

mod weights;
mod benchmarking;

pub type TimeStampedPrice = orml_oracle::TimestampedValue<Price, primitives::Moment>;

// Priority of unsigned transactions
parameter_types! {
	// Operational is 3/4 of TransactionPriority::max_value().
	// Ensure Inherent -> Operational tx -> Unsigned tx -> Signed normal tx
	pub const RenvmBridgeUnsignedPriority: TransactionPriority = TransactionPriority::max_value() / 2;
	pub const CdpEngineUnsignedPriority: TransactionPriority = TransactionPriority::max_value();
	pub const AuctionManagerUnsignedPriority: TransactionPriority = TransactionPriority::max_value() - 1;
}

/// Check if the given `address` is a system contract.
///
/// It's system contract if the address starts with SYSTEM_CONTRACT_ADDRESS_PREFIX.
pub fn is_system_contract(address: H160) -> bool {
	address.as_bytes().starts_with(&SYSTEM_CONTRACT_ADDRESS_PREFIX)
}

pub fn is_core_precompile(address: H160) -> bool {
	address >= H160::from_low_u64_be(PRECOMPILE_ADDRESS_START)
		&& address < H160::from_low_u64_be(PREDEPLOY_ADDRESS_START)
}

/// The call is allowed only if caller is a system contract.
pub struct SystemContractsFilter;
impl PrecompileCallerFilter for SystemContractsFilter {
	fn is_allowed(caller: H160) -> bool {
		is_system_contract(caller)
	}
}

/// Convert gas to weight
pub struct GasToWeight;
impl Convert<u64, Weight> for GasToWeight {
	fn convert(a: u64) -> u64 {
		// TODO: estimate this
		a as Weight
	}
}

pub const AVERAGE_ON_INITIALIZE_RATIO: Perbill = Perbill::from_perthousand(25);
/// We allow `Normal` extrinsics to fill up the block up to 75%, the rest can be
/// used by  Operational  extrinsics.
const NORMAL_DISPATCH_RATIO: Perbill = Perbill::from_percent(75);
/// We allow for 2 seconds of compute with a 5 second average block time.
pub const MAXIMUM_BLOCK_WEIGHT: Weight = 500 * WEIGHT_PER_MILLIS;

const_assert!(NORMAL_DISPATCH_RATIO.deconstruct() >= AVERAGE_ON_INITIALIZE_RATIO.deconstruct());

parameter_types! {
	/// Maximum length of block. Up to 5MB.
	pub BlockLength: limits::BlockLength =
		limits::BlockLength::max_with_normal_ratio(5 * 1024 * 1024, NORMAL_DISPATCH_RATIO);
	/// Block weights base values and limits.
	pub BlockWeights: limits::BlockWeights = limits::BlockWeights::builder()
		.base_block(BlockExecutionWeight::get())
		.for_class(DispatchClass::all(), |weights| {
			weights.base_extrinsic = ExtrinsicBaseWeight::get();
		})
		.for_class(DispatchClass::Normal, |weights| {
			weights.max_total = Some(NORMAL_DISPATCH_RATIO * MAXIMUM_BLOCK_WEIGHT);
		})
		.for_class(DispatchClass::Operational, |weights| {
			weights.max_total = Some(MAXIMUM_BLOCK_WEIGHT);
			// Operational transactions have an extra reserved space, so that they
			// are included even if block reached `MAXIMUM_BLOCK_WEIGHT`.
			weights.reserved = Some(
				MAXIMUM_BLOCK_WEIGHT - NORMAL_DISPATCH_RATIO * MAXIMUM_BLOCK_WEIGHT,
			);
		})
		.avg_block_initialization(AVERAGE_ON_INITIALIZE_RATIO)
		.build_or_panic();
}

parameter_types! {
	/// A limit for off-chain phragmen unsigned solution submission.
	///
	/// We want to keep it as high as possible, but can't risk having it reject,
	/// so we always subtract the base block execution weight.
	pub OffchainSolutionWeightLimit: Weight = BlockWeights::get()
		.get(DispatchClass::Normal)
		.max_extrinsic
		.expect("Normal extrinsics have weight limit configured by default; qed")
		.saturating_sub(BlockExecutionWeight::get());
}

pub struct DummyNomineeFilter;
impl<AccountId> Contains<AccountId> for DummyNomineeFilter {
	fn contains(_: &AccountId) -> bool {
		true
	}
}

// TODO: make those const fn
pub fn dollar(currency_id: CurrencyId) -> Balance {
	10u128.saturating_pow(currency_id.decimals().expect("Not support Erc20 decimals").into())
}

pub fn cent(currency_id: CurrencyId) -> Balance {
	dollar(currency_id) / 100
}

pub fn millicent(currency_id: CurrencyId) -> Balance {
	cent(currency_id) / 1000
}

pub fn microcent(currency_id: CurrencyId) -> Balance {
	millicent(currency_id) / 1000
}

//
// formerly authority.rs
//
parameter_types! {
	pub const SevenDays: BlockNumber = 7 * DAYS;
}

// Module accounts of runtime
parameter_types! {
	pub const TreasuryModuleId: ModuleId = ModuleId(*b"set/trsy");
	pub const PublicFundModuleId: ModuleId = ModuleId(*b"set/fund");
	pub const SerpTreasuryModuleId: ModuleId = ModuleId(*b"set/serp");
	pub const CDPTreasuryModuleId: ModuleId = ModuleId(*b"set/cdpt");
	pub const LoansModuleId: ModuleId = ModuleId(*b"set/loan");
	pub const DEXModuleId: ModuleId = ModuleId(*b"set/sdex");
	pub const NftModuleId: ModuleId = ModuleId(*b"set/sNFT");
}

pub fn get_all_module_accounts() -> Vec<AccountId> {
	vec![
		TreasuryModuleId::get().into_account(),
		PublicFundModuleId::get().into_account(),
		SerpTreasuryModuleId::get().into_account(),
		CDPTreasuryModuleId::get().into_account(),
		LoansModuleId::get().into_account(),
		DEXModuleId::get().into_account(),
		ZeroAccountId::get(),		 	// ACCOUNT 0
		BuyBackPoolAccountId::get(), 	// ACCOUNT 1
		CashDropPoolAccountId::get(), 	// ACCOUNT 2
		ThreeAccountId::get(),			// ACCOUNT 3
	]
}

pub struct AuthorityConfigImpl;
impl orml_authority::AuthorityConfig<Origin, OriginCaller, BlockNumber> for AuthorityConfigImpl {
	fn check_schedule_dispatch(origin: Origin, _priority: Priority) -> DispatchResult {
		EnsureRoot::<AccountId>::try_origin(origin)
		.or_else(|o| EnsureRootOrHalfShuraCouncil::try_origin(o).map(|_| ()))
		.or_else(|o| EnsureRootOrHalfFinancialCouncil::try_origin(o).map(|_| ()))
		.or_else(|o| EnsureRootOrHalfPublicFundCouncil::try_origin(o).map(|_| ()))
		.map_or_else(|_| Err(BadOrigin.into()), |_| Ok(()))
	}

	fn check_fast_track_schedule(
		origin: Origin,
		_initial_origin: &OriginCaller,
		_new_delay: BlockNumber,
	) -> DispatchResult {
		ensure_root(origin.clone()).or_else(|_| {
			if new_delay / HOURS < 12 {
				EnsureRootOrTwoThirdsTechnicalCommittee::ensure_origin(origin)
					.map_or_else(|e| Err(e.into()), |_| Ok(()))
			} else {
				EnsureRootOrOneThirdsTechnicalCommittee::ensure_origin(origin)
					.map_or_else(|e| Err(e.into()), |_| Ok(()))
			}
		})
	}

	fn check_delay_schedule(origin: Origin, _initial_origin: &OriginCaller) -> DispatchResult {
		ensure_root(origin.clone()).or_else(|_| {
			EnsureRootOrOneThirdsTechnicalCommittee::ensure_origin(origin).map_or_else(|e| Err(e.into()), |_| Ok(()))
		})
	}

	fn check_cancel_schedule(origin: Origin, initial_origin: &OriginCaller) -> DispatchResult {
		ensure_root(origin.clone()).or_else(|_| {
			if origin.caller() == initial_origin
				|| EnsureRootOrThreeFourthsShuraCouncil::ensure_origin(origin).is_ok()
			{
				Ok(())
			} else {
				Err(BadOrigin.into())
			}
		})
	}
}

impl orml_authority::AsOriginId<Origin, OriginCaller> for AuthoritysOriginId {
	fn into_origin(self) -> OriginCaller {
		match self {
			AuthoritysOriginId::Root => Origin::root().caller().clone(),
			AuthoritysOriginId::Treasury => Origin::signed(TreasuryModuleId::get().into_account())
				.caller()
				.clone(),
			AuthoritysOriginId::PublicFund => Origin::signed(PublicFundModuleId::get().into_account())
				.caller()
				.clone(),
		}
	}

	fn check_dispatch_from(&self, origin: Origin) -> DispatchResult {
		ensure_root(origin.clone()).or_else(|_| {
			match self {
				AuthoritysOriginId::Root => <EnsureDelayed<
					SevenDays,
					EnsureRootOrThreeFourthsShuraCouncil,
					BlockNumber,
					OriginCaller,
				> as EnsureOrigin<Origin>>::ensure_origin(origin)
				.map_or_else(|_| Err(BadOrigin.into()), |_| Ok(())),
				AuthoritysOriginId::Treasury => {
					<EnsureDelayed<OneDay, EnsureRootOrHalfShuraCouncil, BlockNumber, OriginCaller> as EnsureOrigin<
						Origin,
					>>::ensure_origin(origin)
					.map_or_else(|_| Err(BadOrigin.into()), |_| Ok(()))
				}
				AuthoritysOriginId::PublicFund => <EnsureDelayed<
					OneDay,
					EnsureRootOrHalfPublicFundCouncil,
					BlockNumber,
					OriginCaller,
				> as EnsureOrigin<Origin>>::ensure_origin(origin)
				.map_or_else(|_| Err(BadOrigin.into()), |_| Ok(()))
			}
		})
	}
}

// end authority.rs

/// Opaque types. These are used by the CLI to instantiate machinery that don't need to know
/// the specifics of the runtime. They can then be made to be agnostic over specific formats
/// of data like extrinsics, allowing for them to continue syncing the network through upgrades
/// to even the core data structures.
pub mod opaque {
	use super::*;

	pub use sp_runtime::OpaqueExtrinsic as UncheckedExtrinsic;

	/// Opaque block header type.
	pub type Header = generic::Header<BlockNumber, BlakeTwo256>;
	/// Opaque block type.
	pub type Block = generic::Block<Header, UncheckedExtrinsic>;
	/// Opaque block identifier type.
	pub type BlockId = generic::BlockId<Block>;

	impl_opaque_keys! {
		pub struct SessionKeys {
			pub babe: Babe,
			pub grandpa: Grandpa,
			pub im_online: ImOnline,
			pub authority_discovery: AuthorityDiscovery,
		}
	}
}

/// Fee-related
pub mod fee {
	use primitives::Balance;
	use frame_support::weights::{
		constants::ExtrinsicBaseWeight, WeightToFeeCoefficient,
		WeightToFeeCoefficients, WeightToFeePolynomial,
	};
	use smallvec::smallvec;
	use sp_runtime::Perbill;

	/// The block saturation level. Fees will be updates based on this value.
	pub const TARGET_BLOCK_FULLNESS: Perbill = Perbill::from_percent(25);

	fn base_tx_in_setm() -> Balance {
		cent(SETM) / 10
	}

	/// Handles converting a weight scalar to a fee value, based on the scale
	/// and granularity of the node's balance type.
	///
	/// This should typically create a mapping between the following ranges:
	///   - [0, system::MaximumBlockWeight]
	///   - [Balance::min, Balance::max]
	///
	/// Yet, it can be used for any other sort of change to weight-fee. Some
	/// examples being:
	///   - Setting it to `0` will essentially disable the weight fee.
	///   - Setting it to `1` will cause the literal `#[weight = x]` values to
	///     be charged.
	pub struct WeightToFee;
	impl WeightToFeePolynomial for WeightToFee {
		type Balance = Balance;
		fn polynomial() -> WeightToFeeCoefficients<Self::Balance> {
			// in Setheum, extrinsic base weight (smallest non-zero weight) is mapped to 1/10 CENT:
			let p = base_tx_in_setm();
			let q = Balance::from(ExtrinsicBaseWeight::get()); // 125_000_000
			smallvec![WeightToFeeCoefficient {
				degree: 1,
				negative: false,
				coeff_frac: Perbill::from_rational_approximation(p % q, q),
				coeff_integer: p / q,
			}]
		}
	}
}

pub const VERSION: RuntimeVersion = RuntimeVersion {
	spec_name: create_runtime_str!("setheum"),
	impl_name: create_runtime_str!("setheum"),
	authoring_version: 1,
	spec_version: 100,
	impl_version: 0,
	apis: RUNTIME_API_VERSIONS,
	transaction_version: 1,
};

/// The version information used to identify this runtime when compiled natively.
#[cfg(feature = "std")]
pub fn native_version() -> NativeVersion {
	NativeVersion {
		runtime_version: VERSION,
		can_author_with: Default::default(),
	}
}

parameter_types! {
	pub const Version: RuntimeVersion = VERSION;
	pub const BlockHashCount: BlockNumber = 1200; // mortal tx can be valid up to 1 hour after signing
	pub const SS58Prefix: u8 = 258;
}

pub struct BaseCallFilter;
impl Filter<Call> for BaseCallFilter {
	fn filter(call: &Call) -> bool {
		matches!(
			call,
			// Core
			Call::System(_) | Call::Timestamp(_) |
			// Utility
			Call::Scheduler(_) | Call::Utility(_) | Call::Multisig(_) |
			// Sudo
			Call::Sudo(_) |
			// PoA
			Call::Authority(_) | Call::ShuraCouncil(_) | Call::ShuraCouncilMembership(_) |
			Call::FinancialCouncil(_) | Call::FinancialCouncilMembership(_) |
			Call::PublicFundCouncil(_) | Call::PublicFundCouncilMembership(_) |
			Call::TechnicalCommittee(_) | Call::TechnicalCommitteeMembership(_) |
			// Oracle
			Call::SetheumOracle(_) | Call::OperatorMembershipSetheum(_)
		)
	}
}

impl frame_system::Config for Runtime {
	/// The basic call filter to use in dispatchable.
	type BaseCallFilter = BaseCallFilter;
	/// Block & extrinsics weights: base values and limits.
	type BlockWeights = BlockWeights;
	/// The maximum length of a block (in bytes).
	type BlockLength = BlockLength;
	/// The identifier used to distinguish between accounts.
	type AccountId = AccountId;
	/// The aggregated dispatch type that is available for extrinsics.
	type Call = Call;
	/// The lookup mechanism to get account ID from whatever is passed in dispatchers.
	type Lookup = (Indices, EvmAccounts);
	/// The index type for storing how many extrinsics an account has signed.
	type Index = Nonce;
	/// The index type for blocks.
	type BlockNumber = BlockNumber;
	/// The type for hashing blocks and tries.
	type Hash = Hash;
	/// The hashing algorithm used.
	type Hashing = BlakeTwo256;
	/// The header type.
	type Header = generic::Header<BlockNumber, BlakeTwo256>;
	/// The ubiquitous event type.
	type Event = Event;
	/// The ubiquitous origin type.
	type Origin = Origin;
	/// Maximum number of block number to block hash mappings to keep (oldest pruned first).
	type BlockHashCount = BlockHashCount;
	/// Maximum weight of each block.
	type DbWeight = RocksDbWeight;
	/// Version of the runtime.
	type Version = Version;
	/// This type is being generated by `construct_runtime!`.
	type PalletInfo = PalletInfo;
	/// What to do if a new account is created.
	type OnNewAccount = ();
	/// What to do if an account is fully reaped from the system.
	type OnKilledAccount = (
		module_evm::CallKillAccount<Runtime>,
		module_evm_accounts::CallKillAccount<Runtime>,
	);
	/// The data to be stored in an account.
	type AccountData = pallet_balances::AccountData<Balance>;
	/// Weight information for the extrinsics of this pallet.
	type SystemWeightInfo = ();
	/// This is used as an identifier of the chain. 42 is the generic substrate prefix.
	type SS58Prefix = SS58Prefix;
}


parameter_types! {
	pub const DisabledValidatorsThreshold: Perbill = Perbill::from_percent(17);
}

impl pallet_session::Config for Runtime {
	type Event = Event;
	type ValidatorId = <Self as frame_system::Config>::AccountId;
	type ValidatorIdOf = pallet_staking::StashOf<Self>;
	type ShouldEndSession = Babe;
	type NextSessionRotation = Babe;
	type SessionManager = pallet_session::historical::NoteHistoricalRoot<Self, Staking>;
	type SessionHandler = <opaque::SessionKeys as OpaqueKeys>::KeyTypeIdProviders;
	type Keys = opaque::SessionKeys;
	type DisabledValidatorsThreshold = DisabledValidatorsThreshold;
	type WeightInfo = ();
}

parameter_types! {
	pub const EpochDuration: u64 = EPOCH_DURATION_IN_SLOTS;
	pub const ExpectedBlockTime: Moment = MILLISECS_PER_BLOCK;
	pub const ReportLongevity: u64 =
		BondingDuration::get() as u64 * SessionsPerEra::get() as u64 * EpochDuration::get();
}

impl pallet_session::historical::Config for Runtime {
	type FullIdentification = pallet_staking::Exposure<AccountId, Balance>;
	type FullIdentificationOf = pallet_staking::ExposureOf<Runtime>;
}

pallet_staking_reward_curve::build! {
	// 2.58% min, 25.8% max, 50% ideal stake
	const REWARD_CURVE: PiecewiseLinear<'static> = curve!(
		min_inflation: 0_025_800,
		max_inflation: 0_258_000,
		ideal_stake: 0_500_000,
		falloff: 0_050_000,
		max_piece_count: 40,
		test_precision: 0_005_500,
	);
}

parameter_types! {
	pub const SessionsPerEra: sp_staking::SessionIndex = 2; // 2 hours
	pub const BondingDuration: pallet_staking::EraIndex = 4; // 8 hours
	pub const SlashDeferDuration: pallet_staking::EraIndex = 2; // 4 hours
	pub const RewardCurve: &'static PiecewiseLinear<'static> = &REWARD_CURVE;
	// only top N nominators get paid for each validator
	pub const MaxNominatorRewardedPerValidator: u32 = 258;
	pub const ElectionLookahead: BlockNumber = EPOCH_DURATION_IN_BLOCKS / 2;
	pub const MaxIterations: u32 = 5;
	// 0.05%. The higher the value, the more strict solution acceptance becomes.
	pub MinSolutionScoreBump: Perbill = Perbill::from_rational_approximation(5 as u32, 10_000);
	// offchain tx signing
	pub const StakingUnsignedPriority: TransactionPriority = TransactionPriority::max_value() / 2;
}

impl pallet_staking::Config for Runtime {
	type Currency = Balances;
	type UnixTime = Timestamp;
	type CurrencyToVote = U128CurrencyToVote;
	type RewardRemainder = SetheumTreasury;
	type Event = Event;
	type Slash = SetheumTreasury; // send the slashed funds to the Setheum treasury.
	type Reward = (); // rewards are minted from the void
	type SessionsPerEra = SessionsPerEra;
	type BondingDuration = BondingDuration;
	type SlashDeferDuration = SlashDeferDuration;
	type SlashCancelOrigin = EnsureRootOrThreeFourthsTechnicalCommittee;
	type SessionInterface = Self;
	type RewardCurve = RewardCurve;
	type NextNewSession = Session;
	type ElectionLookahead = ElectionLookahead;
	type Call = Call;
	type MaxIterations = MaxIterations;
	type MinSolutionScoreBump = MinSolutionScoreBump;
	type MaxNominatorRewardedPerValidator = MaxNominatorRewardedPerValidator;
	type UnsignedPriority = StakingUnsignedPriority;
	type WeightInfo = ();
	type OffchainSolutionWeightLimit = OffchainSolutionWeightLimit;
}

impl pallet_babe::Config for Runtime {
	type EpochDuration = EpochDuration;
	type ExpectedBlockTime = ExpectedBlockTime;
	type EpochChangeTrigger = pallet_babe::ExternalTrigger;
	type KeyOwnerProofSystem = Historical;
	type KeyOwnerProof =
		<Self::KeyOwnerProofSystem as KeyOwnerProofSystem<(KeyTypeId, pallet_babe::AuthorityId)>>::Proof;
	type KeyOwnerIdentification =
		<Self::KeyOwnerProofSystem as KeyOwnerProofSystem<(KeyTypeId, pallet_babe::AuthorityId)>>::IdentificationTuple;
	type HandleEquivocation =
		pallet_babe::EquivocationHandler<Self::KeyOwnerIdentification, Offences, ReportLongevity>;
	type WeightInfo = ();
}

impl pallet_grandpa::Config for Runtime {
	type Event = Event;
	type Call = Call;
	type KeyOwnerProofSystem = Historical;
	type KeyOwnerProof =
		<Self::KeyOwnerProofSystem as KeyOwnerProofSystem<(KeyTypeId, GrandpaId)>>::Proof;
	type KeyOwnerIdentification =
		<Self::KeyOwnerProofSystem as KeyOwnerProofSystem<(KeyTypeId, GrandpaId)>>::IdentificationTuple;
	type HandleEquivocation =
		pallet_grandpa::EquivocationHandler<Self::KeyOwnerIdentification, Offences, ReportLongevity>;
	type WeightInfo = ();
}

parameter_types! {
	pub const MinimumPeriod: u64 = SLOT_DURATION / 2;
}

impl pallet_timestamp::Config for Runtime {
	/// A timestamp: milliseconds since the unix epoch.
	type Moment = u64;
	type OnTimestampSet = Babe;
	type MinimumPeriod = MinimumPeriod;
	type WeightInfo = ();
}

parameter_types! {
	pub const UncleGenerations: BlockNumber = 5;
}

impl pallet_authorship::Config for Runtime {
	type FindAuthor = pallet_session::FindAccountFromAuthorIndex<Self, Babe>;
	type UncleGenerations = UncleGenerations;
	type FilterUncle = ();
	type EventHandler = (Staking, ImOnline);
}

parameter_types! {
	pub OffencesWeightSoftLimit: Weight = Perbill::from_percent(20) * BlockWeights::get().max_block;
}

impl pallet_offences::Config for Runtime {
	type Event = Event;
	type IdentificationTuple = pallet_session::historical::IdentificationTuple<Self>;
	type OnOffenceHandler = Staking;
	type WeightSoftLimit = OffencesWeightSoftLimit;
}

impl pallet_authority_discovery::Config for Runtime {}

parameter_types! {
	pub const ImOnlineUnsignedPriority: TransactionPriority = TransactionPriority::max_value();
}

parameter_types! {
	pub SessionDuration: BlockNumber = 1 * primitives::time::HOURS;
}

impl pallet_im_online::Config for Runtime {
	type AuthorityId = ImOnlineId;
	type Event = Event;
	type ValidatorSet = Historical;
	type SessionDuration = SessionDuration;
	type ReportUnresponsiveness = Offences;
	type UnsignedPriority = ImOnlineUnsignedPriority;
	type WeightInfo = ();
}

parameter_types! {
	pub const BasicDeposit: Balance =      100 * SETM;
	pub const FieldDeposit: Balance =        1 * SETM;
	pub const SubAccountDeposit: Balance =  20 * SETM;
	pub const MaxSubAccounts: u32 = 100;
	pub const MaxAdditionalFields: u32 = 100;
	pub const MaxRegistrars: u32 = 25;
}

impl pallet_identity::Config for Runtime {
	type Event = Event;
	type Currency = Balances;
	type BasicDeposit = BasicDeposit;
	type FieldDeposit = FieldDeposit;
	type SubAccountDeposit = SubAccountDeposit;
	type MaxSubAccounts = MaxSubAccounts;
	type MaxAdditionalFields = MaxAdditionalFields;
	type MaxRegistrars = MaxRegistrars;
	type Slashed = SetheumTreasury;
	type ForceOrigin = EnsureRootOrTwoThridsTechnicalCommittee;
	type RegistrarOrigin = EnsureRootOrTwoThridsTechnicalCommittee;
	type WeightInfo = ();
}


parameter_types! {
	pub const IndexDeposit: Balance = 1 * SETM;
}

impl pallet_indices::Config for Runtime {
	type AccountIndex = AccountIndex;
	type Event = Event;
	type Currency = Balances;
	type Deposit = IndexDeposit;
	type WeightInfo = ();
}

parameter_types! {
	pub const GetNativeCurrencyId: CurrencyId = CurrencyId::Token(TokenSymbol::SETM);
	pub const GetSerpCurrencyId: CurrencyId = CurrencyId::Token(TokenSymbol::SERP);
	pub const GetDinarCurrencyId: CurrencyId = CurrencyId::Token(TokenSymbol::DNAR);
	pub const SetterCurrencyId: CurrencyId = CurrencyId::Token(TokenSymbol::SETR);
	pub const GetSetUSDCurrencyId: CurrencyId = CurrencyId::Token(TokenSymbol::SETUSD);
	// All currency types except for native currency, Sort by fee charge order
	pub AllNonNativeCurrencyIds: Vec<CurrencyId> = vec![
		SETUSD,
		SETR,
		SERP,
		DNAR,
		RENBTC
	];
	pub StableCurrencyIds: Vec<CurrencyId> = vec![
		SETR,
		SETUSD,
	];
	pub CollateralCurrencyIds: Vec<CurrencyId> = vec![
		SETM,
		SERP,
		DNAR,
		SETR,
		RENBTC
	];
}

parameter_type_with_key! {
	pub ExistentialDeposits: |_currency_id: CurrencyId| -> Balance {
		match currency_id {
			CurrencyId::Token(symbol) => match symbol {
				TokenSymbol::SETUSD => cent(*currency_id),
				TokenSymbol::SETR => 100 * millicent(*currency_id),
				TokenSymbol::SERP => 10 * millicent(*currency_id),
				TokenSymbol::DNAR => 10 * millicent(*currency_id),

				TokenSymbol::SETM |
				TokenSymbol::RENBTC => Balance::max_value() // unsupported
			},
			CurrencyId::DexShare(_, _) => {
				let dec = <EvmCurrencyIdMapping<Runtime> as CurrencyIdMapping>::decimals(*currency_id);
				if let Some(dec) = dec {
					// TODO: verify if this make sense
					10u128.saturating_pow(dec as u32)
				} else {
					// TODO: update this before we enable ERC20 in DEX
					Balance::max_value() // unsupported
				}
			},
			CurrencyId::Erc20(_) => Balance::max_value(), // not handled by orml-tokens
		}
	};
}

parameter_types! {
	pub SetheumTreasuryAccount: AccountId = TreasuryModuleId::get().into_account();
}

impl orml_tokens::Config for Runtime {
	type Event = Event;
	type Balance = Balance;
	type Amount = Amount;
	type CurrencyId = CurrencyId;
	type WeightInfo = ();
	type ExistentialDeposits = ExistentialDeposits;
	type OnDust = orml_tokens::TransferDust<Runtime, SetheumTreasuryAccount>;
}

impl module_currencies::Config for Runtime {
	type Event = Event;
	type MultiCurrency = Tokens;
	type NativeCurrency = BasicCurrencyAdapter<Runtime, Balances, Amount, BlockNumber>;
	type GetNativeCurrencyId = GetNativeCurrencyId;
	type StableCurrencyIds = StableCurrencyIds;
	type SerpTreasury = SerpTreasury;
	type WeightInfo = weights::module_currencies::WeightInfo<Runtime>;
	type AddressMapping = EvmAddressMapping<Runtime>;
	type EVMBridge = EVMBridge;
}

parameter_types! {
	pub SetUSDFixedPrice: Price = Price::saturating_from_rational(1, 1); // $1
	pub SetterFixedPrice: Price = Price::saturating_from_rational(2, 1); // $2
}

impl module_prices::Config for Runtime {
	type Event = Event;
	type Source = AggregatedDataProvider;
	type GetSetUSDCurrencyId = GetSetUSDCurrencyId;
	type SetterCurrencyId = GetSetUSDCurrencyId;
	type SetUSDFixedPrice = SetUSDFixedPrice;
	type SetterFixedPrice = SetterFixedPrice;
	type LockOrigin = EnsureRootOrTwoThirdsGeneralCouncil;
	type DEX = Dex;
	type Currency = Currencies;
	type CurrencyIdMapping = EvmCurrencyIdMapping<Runtime>;
	type WeightInfo = weights::module_prices::WeightInfo<Runtime>;
}

parameter_types! {
	pub const TransactionByteFee: Balance = 10 * millicent(SETM);
	pub const TargetBlockFullness: Perquintill = Perquintill::from_percent(25);
	pub AdjustmentVariable: Multiplier = Multiplier::saturating_from_rational(1, 100_000);
	pub MinimumMultiplier:  Multiplier = Multiplier::saturating_from_rational(1, 1_000_000_000 as u128);
}

impl module_transaction_payment::Config for Runtime {
	type AllNonNativeCurrencyIds = AllNonNativeCurrencyIds;
	type NativeCurrencyId = GetNativeCurrencyId;
	type SetUSDCurrencyId = GetSetUSDCurrencyId;
	type Currency = Balances;
	type MultiCurrency = Currencies;
	type OnTransactionPayment = Treasury;
	type TransactionByteFee = TransactionByteFee;
	type WeightToFee = fee::WeightToFee;
	type FeeMultiplierUpdate = TargetedFeeAdjustment<Self, TargetBlockFullness, AdjustmentVariable, MinimumMultiplier>;
	type DEX = DEX;
	type WeightInfo = weights::transaction_payment::WeightInfo<Runtime>;
}

parameter_types! {
	pub SetheumFoundationAccounts: Vec<AccountId> = vec![
		hex_literal::hex![""].into(),	// TODO: Update Acc1
		hex_literal::hex![""].into(),	// TODO: Update Acc1
	];
}

pub struct EnsureSetheumFoundation;
impl EnsureOrigin<Origin> for EnsureSetheumFoundation {
	type Success = AccountId;

	fn try_origin(o: Origin) -> Result<Self::Success, Origin> {
		Into::<Result<RawOrigin<AccountId>, Origin>>::into(o).and_then(|o| match o {
			RawOrigin::Signed(caller) => {
				if SetheumFoundationAccounts::get().contains(&caller) {
					Ok(caller)
				} else {
					Err(Origin::from(Some(caller)))
				}
			}
			r => Err(Origin::from(r)),
		})
	}

	#[cfg(feature = "runtime-benchmarks")]
	fn successful_origin() -> Origin {
		Origin::from(RawOrigin::Signed(Default::default()))
	}
}

pub struct EvmAccountsOnClaimHandler;
impl module_evm_accounts::Handler<AccountId> for EvmAccountsOnClaimHandler {
	fn handle(who: &AccountId) -> DispatchResult {
		if System::providers(who) == 0 {
			// no provider. i.e. no native tokens
			// ensure there are some native tokens, which will add provider
			TransactionPayment::ensure_can_charge_fee(
				who,
				NativeTokenExistentialDeposit::get(),
				WithdrawReasons::TRANSACTION_PAYMENT,
			);
		}
		Ok(())
	}
}

impl module_evm_accounts::Config for Runtime {
	type Event = Event;
	type Currency = Balances;
	type AddressMapping = EvmAddressMapping<Runtime>;
	type MergeAccount = Currencies;
	type OnClaim = EvmAccountsOnClaimHandler;
	type WeightInfo = weights::module_evm_accounts::WeightInfo<Runtime>;
}

#[cfg(feature = "with-ethereum-compatibility")]
static ISTANBUL_CONFIG: evm::Config = evm::Config::istanbul();

parameter_types! {
	pub const ChainId: u64 = 258;
	// 10 SETM minimum storage deposit
	pub const NewContractExtraBytes: u32 = 10_000;
	pub StorageDepositPerByte: Balance = 1 * millicent(SETM);
	pub MaxCodeSize: u32 = 60 * 1024;
	pub NetworkContractSource: H160 = H160::from_low_u64_be(0);
	pub DeveloperDeposit: Balance = 100 * dollar(SETM);
	pub DeploymentFee: Balance = 1_000 * dollar(SETM);
}

pub type MultiCurrencyPrecompile = runtime_common::MultiCurrencyPrecompile<
	AccountId,
	EvmAddressMapping<Runtime>,
	EvmCurrencyIdMapping<Runtime>,
	Currencies
	>;

pub type NFTPrecompile =
	runtime_common::NFTPrecompile<
		AccountId,
		EvmAddressMapping<Runtime>,
		EvmCurrencyIdMapping<Runtime>,
		NFT
	>;
pub type StateRentPrecompile = 
	runtime_common::StateRentPrecompile<
		AccountId,
		EvmAddressMapping<Runtime>,
		EvmCurrencyIdMapping<Runtime>,
		EVM
	>;
pub type ScheduleCallPrecompile = runtime_common::ScheduleCallPrecompile<
	AccountId,
	EvmAddressMapping<Runtime>,
	EvmCurrencyIdMapping<Runtime>,
	Scheduler,
	module_transaction_payment::ChargeTransactionPayment<Runtime>,
	Call,
	Origin,
	OriginCaller,
	Runtime,
>;
pub type DexPrecompile =
	runtime_common::DexPrecompile<
	AccountId,
	EvmAddressMapping<Runtime>,
	EvmCurrencyIdMapping<Runtime>,
	Dex
>;

impl module_evm::Config for Runtime {
	type AddressMapping = EvmAddressMapping<Runtime>;
	type Currency = Balances;
	type MergeAccount = Currencies;
	type NewContractExtraBytes = NewContractExtraBytes;
	type StorageDepositPerByte = StorageDepositPerByte;
	type MaxCodeSize = MaxCodeSize;
	type Event = Event;
	type Precompiles = runtime_common::AllPrecompiles<
		SystemContractsFilter,
		MultiCurrencyPrecompile,
		StateRentPrecompile,
		ScheduleCallPrecompile,
	>;
	type ChainId = ChainId;
	type GasToWeight = GasToWeight;
	type ChargeTransactionPayment = module_transaction_payment::ChargeTransactionPayment<Runtime>;
	type NetworkContractOrigin = EnsureRootOrTwoThridsTechnicalCommittee;
	type NetworkContractSource = NetworkContractSource;
	type DeveloperDeposit = DeveloperDeposit;
	type DeploymentFee = DeploymentFee;
	type FreeDeploymentOrigin = EnsureRootOrHalfShuraCouncil;
	type WeightInfo = weights::module_evm::WeightInfo<Runtime>;

	#[cfg(feature = "with-ethereum-compatibility")]
	fn config() -> &'static evm::Config {
		&ISTANBUL_CONFIG
	}
}

impl module_evm_bridge::Config for Runtime {
	type EVM = Evm;
}

impl module_evm_manager::Config for Runtime {
	type Currency = Balances;
	type EVMBridge = EVMBridge;
}

parameter_types! {
	// note: if we add other native tokens (SETUSD) we have to set native
	// existential deposit to 0 or check for other tokens on account pruning
	pub const NativeTokenExistentialDeposit: Balance = 10 * millicent(SETM);
	pub const MaxNativeTokenExistentialDeposit: Balance = 258 * millicent(SETM);
	pub const MaxLocks: u32 = 50;
}

impl pallet_balances::Config for Runtime {
	type Balance = Balance;
	type DustRemoval = SetheumTreasury;
	type Event = Event;
	type ExistentialDeposit = NativeTokenExistentialDeposit;
	type AccountStore = frame_system::Module<Runtime>;
	type MaxLocks = MaxLocks;
	type WeightInfo = pallet_balances::weights::SubstrateWeight<Runtime>;
}

parameter_types! {
	pub MinimumIncrementSize: Rate = Rate::saturating_from_rational(2, 100);
	pub const AuctionTimeToClose: BlockNumber = 15 * MINUTES;
	pub const AuctionDurationSoftCap: BlockNumber = 2 * HOURS;
}

impl auction_manager::Config for Runtime {
	type Event = Event;
	type Currency = Currencies;
	type Auction = Auction;
	type MinimumIncrementSize = MinimumIncrementSize;
	type AuctionTimeToClose = AuctionTimeToClose;
	type AuctionDurationSoftCap = AuctionDurationSoftCap;
	type GetSetUSDCurrencyId = GetSetUSDCurrencyId;
	type CDPTreasury = CdpTreasury;
	type DEX = Dex;
	type PriceSource = Prices;
	type UnsignedPriority = runtime_common::AuctionManagerUnsignedPriority;
	type EmergencyShutdown = EmergencyShutdown;
	type WeightInfo = weights::auction_manager::WeightInfo<Runtime>;
}

impl module_loans::Config for Runtime {
	type Event = Event;
	type Convert = cdp_engine::DebitExchangeRateConvertor<Runtime>;
	type Currency = Currencies;
	type RiskManager = CdpEngine;
	type CDPTreasury = CdpTreasury;
	type ModuleId = LoansModuleId;
	type OnUpdateLoan = ();
}

impl<LocalCall> frame_system::offchain::CreateSignedTransaction<LocalCall> for Runtime
where
	Call: From<LocalCall>,
{
	fn create_transaction<C: frame_system::offchain::AppCrypto<Self::Public, Self::Signature>>(
		call: Call,
		public: <Signature as sp_runtime::traits::Verify>::Signer,
		account: AccountId,
		nonce: Nonce,
	) -> Option<(
		Call,
		<UncheckedExtrinsic as sp_runtime::traits::Extrinsic>::SignaturePayload,
	)> {
		// take the biggest period possible.
		let period = BlockHashCount::get()
			.checked_next_power_of_two()
			.map(|c| c / 2)
			.unwrap_or(2) as u64;
		let current_block = System::block_number()
			.saturated_into::<u64>()
			// The `System::block_number` is initialized with `n+1`,
			// so the actual block number is `n`.
			.saturating_sub(1);
		let tip = 0;
		let extra: SignedExtra = (
			frame_system::CheckSpecVersion::<Runtime>::new(),
			frame_system::CheckTxVersion::<Runtime>::new(),
			frame_system::CheckGenesis::<Runtime>::new(),
			frame_system::CheckEra::<Runtime>::from(generic::Era::mortal(period, current_block)),
			frame_system::CheckNonce::<Runtime>::from(nonce),
			frame_system::CheckWeight::<Runtime>::new(),
			module_transaction_payment::ChargeTransactionPayment::<Runtime>::from(tip),
			module_evm::SetEvmOrigin::<Runtime>::new(),
		);
		let raw_payload = SignedPayload::new(call, extra)
			.map_err(|e| {
				debug::warn!("Unable to create signed payload: {:?}", e);
			})
			.ok()?;
		let signature = raw_payload.using_encoded(|payload| C::sign(payload, public))?;
		let address = AccountIdLookup::unlookup(account);
		let (call, extra, _) = raw_payload.deconstruct();
		Some((call, (address, signature, extra)))
	}
}

impl frame_system::offchain::SigningTypes for Runtime {
	type Public = <Signature as sp_runtime::traits::Verify>::Signer;
	type Signature = Signature;
}

impl<C> frame_system::offchain::SendTransactionTypes<C> for Runtime
where
	Call: From<C>,
{
	type OverarchingCall = Call;
	type Extrinsic = UncheckedExtrinsic;
}

parameter_types! {
	pub DefaultLiquidationRatio: Ratio = Ratio::saturating_from_rational(130, 100);
	pub DefaultDebitExchangeRate: ExchangeRate = ExchangeRate::saturating_from_rational(1, 10);
	pub DefaultLiquidationPenalty: Rate = Rate::saturating_from_rational(8, 100);
	pub MinimumDebitValue: Balance = dollar(SETUSD);
	// pub MaxSlippageSwapWithDEX: Ratio = Ratio::saturating_from_rational(15, 100);
}

impl cdp_engine::Config for Runtime {
	type Event = Event;
	type PriceSource = Prices;
	type CollateralCurrencyIds = CollateralCurrencyIds;
	type GetSetUSDCurrencyId = GetSetUSDCurrencyId;
	type DefaultLiquidationRatio = DefaultLiquidationRatio;
	type DefaultDebitExchangeRate = DefaultDebitExchangeRate;
	type DefaultLiquidationPenalty = DefaultLiquidationPenalty;
	type MinimumDebitValue = MinimumDebitValue;
	type CDPTreasury = CdpTreasury;
	type UnsignedPriority = runtime_common::CdpEngineUnsignedPriority;
	type EmergencyShutdown = EmergencyShutdown;
	type UpdateOrigin = EnsureRootOrHalfFinancialCouncil;
	type WeightInfo = weights::cdp_engine::WeightInfo<Runtime>;
}

parameter_types! {
	pub const MaxAuctionsCount: u32 = 100;
}

impl cdp_treasury::Config for Runtime {
	type Event = Event;
	type Currency = Currencies;
	type GetSetUSDCurrencyId = GetSetUSDCurrencyId;
	type AuctionManagerHandler = AuctionManager;
	type DEX = Dex;
	type MaxAuctionsCount = MaxAuctionsCount;
	type SerpTreasury = SerpTreasury;
	type UpdateOrigin = EnsureRootOrHalfFinancialCouncil;
	type WeightInfo = weights::cdp_treasury::WeightInfo<Runtime>;
	type ModuleId = CDPTreasuryModuleId;
}

parameter_types! {
	pub MaxSwapSlippageCompareToOracle: Ratio = Ratio::saturating_from_rational(5, 100);
	pub DefaultSwapPathList: Vec<Vec<CurrencyId>> = 
		vec![
			vec![DNAR, SETUSD, SETR], 	// swap_dinar_to_exact_setter();
			vec![SETR, SETUSD], 		// swap_setter_to_exact_setcurrency();
			vec![SETR, SETUSD, DNAR],	// swap_exact_setter_to_dinar();
			vec![SETR, SETUSD, DNAR], 	// swap_exact_setcurrency_to_dinar()1;
			vec![SETUSD, DNAR], 		// swap_exact_setcurrency_to_dinar()2;
			vec![SETUSD, SETR] 			// swap_exact_setcurrency_to_setter();
			vec![SETUSD, SETR] 			// serplus_swap_exact_setcurrency_to_setter(();
			vec![SETR, SETUSD, SETM] 	// swap_exact_setcurrency_to_setheum()1;
			vec![SETUSD, SETM] 			// swap_exact_setcurrency_to_setheum()2;
			vec![SETR, SETUSD, SERP] 	// swap_exact_setcurrency_to_serp()1;
			vec![SETUSD, SERP] 			// swap_exact_setcurrency_to_serp()2;
		];
	pub const TradingPathLimit: u32 = 3;
	pub StableCurrencyInflationPeriod: u64 = 300; // 15 minutes (3 seconds BLOCKTIME) - [34,560 periods per annum]
	pub SetterMinimumClaimableTransferAmounts: Balance = 1 * dollar(SETR); 				// $2
	pub SetterMaximumClaimableTransferAmounts: Balance = 200_000 * dollar(SETR);		// $400_000
	pub SetDollarMinimumClaimableTransferAmounts: Balance = 10 * dollar(SETR); 			// $10
	pub SetDollarMaximumClaimableTransferAmounts: Balance = 20_000_000 * dollar(SETR);	// $20_000_000
}

parameter_type_with_key! {
	pub GetStableCurrencyMinimumSupply: |currency_id: CurrencyId| -> Balance {
		match currency_id {
			&SETR => 10_000_000_000* dollar(SETR),
			&SETUSD => 10_000_000_000* dollar(SETUSD),
			_ => 0,
		}
	};
}

parameter_types! {
	pub const CashDropPoolAccount: AccountId = AccountId::from([2u8; 32]); // ACCOUNT 2
	pub const PublicFundAccount: AccountId = PublicFundModuleId::get().into_account();
	pub const CDPTreasuryAccount: AccountId = CDPTreasuryModuleId::get().into_account();
}

impl serp_treasury::Config for Runtime {
	type Event = Event;
	type Currency = Currencies;
	type StableCurrencyIds = StableCurrencyIds;
	type StableCurrencyInflationPeriod = StableCurrencyInflationPeriod;
	type GetStableCurrencyMinimumSupply = GetStableCurrencyMinimumSupply;
	type GetNativeCurrencyId = GetNativeCurrencyId;
	type GetSerpCurrencyId = GetSerpCurrencyId;
	type GetDinarCurrencyId = GetDinarCurrencyId;
	type SetterCurrencyId = SetterCurrencyId;
	type GetSetUSDCurrencyId = GetSetUSDCurrencyId;
	type CashDropPoolAccountId = CashDropPoolAccount;
	type PublicFundAccountId = PublicFundAccount;
	type CDPTreasuryAccountId = CDPTreasuryAccount;
	type SetheumTreasuryAccountId = SetheumTreasuryAccount;
	type DefaultSwapPathList = DefaultSwapPathList;
	type Dex = Dex;
	type MaxSwapSlippageCompareToOracle = MaxSwapSlippageCompareToOracle;
	type TradingPathLimit = TradingPathLimit;
	type PriceSource = Prices;
	type SetterMinimumClaimableTransferAmounts = SetterMinimumClaimableTransferAmounts;
	type SetterMaximumClaimableTransferAmounts = SetterMaximumClaimableTransferAmounts;
	type SetDollarMinimumClaimableTransferAmounts = SetDollarMinimumClaimableTransferAmounts;
	type SetDollarMaximumClaimableTransferAmounts = SetDollarMaximumClaimableTransferAmounts;
	type UpdateOrigin = EnsureRootOrHalfFinancialCouncil;
	type ModuleId = SerpTreasuryModuleId;
	type WeightInfo = weights::serp_treasury::WeightInfo<Runtime>;
}

parameter_types! {
	pub DepositPerAuthorization: Balance = deposit(1, 64);
}

impl serp_setmint::Config for Runtime {
	type Event = Event;
	type Currency = Balances;
	type DepositPerAuthorization = DepositPerAuthorization;
	type WeightInfo = weights::serp_setmint::WeightInfo<Runtime>;
}

impl emergency_shutdown::Config for Runtime {
	type Event = Event;
	type CollateralCurrencyIds = CollateralCurrencyIds;
	type PriceSource = Prices;
	type CDPTreasury = CdpTreasury;
	type AuctionManagerHandler = AuctionManager;
	type ShutdownOrigin = EnsureRootOrThreeFourthsFinancialCouncil;
	type WeightInfo = weights::emergency_shutdown::WeightInfo<Runtime>;
}

parameter_types! {
	pub GetExchangeFee: (u32, u32) = (3, 1000);	// 0.3%
	pub GetStableCurrencyExchangeFee: (u32, u32) = (1, 1000);	// 0.1%
	pub BuyBackPoolAccountId: AccountId = AccountId::from([1u8; 32]);
}

impl module_dex::Config for Runtime {
	type Event = Event;
	type Currency = Currencies;
	type StableCurrencyIds = StableCurrencyIds;
	type GetExchangeFee = GetExchangeFee;
	type GetStableCurrencyExchangeFee = GetStableCurrencyExchangeFee;
	type BuyBackPoolAccountId = BuyBackPoolAccountId;
	type TradingPathLimit = TradingPathLimit;
	type ModuleId = DEXModuleId;
	type CurrencyIdMapping = EvmCurrencyIdMapping<Runtime>;
	type WeightInfo = weights::module_dex::WeightInfo<Runtime>;
	type ListingOrigin = EnsureRootOrHalfFinancialCouncil;
}

parameter_types! {
	pub CreateClassDeposit: Balance = 50 * dollar(SETM);
	pub CreateTokenDeposit: Balance = 20 * cent(SETM);
	pub MaxAttributesBytes: u32 = 2048;
}

impl module_nft::Config for Runtime {
	type Event = Event;
	type Currency = Balances;
	type CreateClassDeposit = CreateClassDeposit;
	type CreateTokenDeposit = CreateTokenDeposit;
	type DataDepositPerByte = DataDepositPerByte;
	type ModuleId = NftModuleId;
	type MaxAttributesBytes = MaxAttributesBytes;
	type WeightInfo = weights::module_nft::WeightInfo<Runtime>;
}

// TODO: Update NFT module then impl.
// parameter_types! {
// 	pub MaxClassMetadata: u32 = 1024;
// 	pub MaxTokenMetadata: u32 = 1024;
// }

impl orml_nft::Config for Runtime {
	type ClassId = u32;
	type TokenId = u64;
	type ClassData = module_nft::ClassData<Balance>;
	type TokenData = module_nft::TokenData<Balance>;
}

parameter_types! {
	// One storage item; key size 32, value size 8; .
	pub ProxyDepositBase: Balance = deposit(1, 8);
	// Additional storage item size of 33 bytes.
	pub ProxyDepositFactor: Balance = deposit(0, 33);
	pub const MaxProxies: u16 = 32;
	pub AnnouncementDepositBase: Balance = deposit(1, 8);
	pub AnnouncementDepositFactor: Balance = deposit(0, 66);
	pub const MaxPending: u16 = 32;
}

impl InstanceFilter<Call> for ProxyType {
	fn filter(&self, c: &Call) -> bool {
		match self {
			// Always allowed Call::Utility no matter type.
			// Only transactions allowed by Proxy.filter can be executed,
			// otherwise `BadOrigin` will be returned in Call::Utility.
			_ if matches!(c, Call::Utility(..)) => true,
			ProxyType::Any => true,
			ProxyType::CancelProxy => matches!(c, Call::Proxy(pallet_proxy::Call::reject_announcement(..))),
			ProxyType::Governance => {
				matches!(
					c,
					Call::Authority(..)
						| Call::Democracy(..) | Call::ShuraCouncil(..)
						| Call::FinancialCouncil(..)
						| Call::PublicFundCouncil(..) | Call::TechnicalCommittee(..)
						| Call::Treasury(..) | Call::Bounties(..)
						| Call::Tips(..)
				)
			}
			ProxyType::Auction => {
				matches!(c, Call::Auction(orml_auction::Call::bid(..)))
			}
			ProxyType::Swap => {
				matches!(
					c,
					Call::Dex(module_dex::Call::swap_with_exact_supply(..))
						| Call::Dex(module_dex::Call::swap_with_exact_target(..))
				)
			}
			ProxyType::Loan => {
				matches!(
					c,
					Call::SetMint(serp_setmint::Call::adjust_loan(..))
						| Call::SetMint(serp_setmint::Call::close_loan_has_debit_by_dex(..))
				)
			}
		}
	}
	fn is_superset(&self, o: &Self) -> bool {
		match (self, o) {
			(x, y) if x == y => true,
			(ProxyType::Any, _) => true,
			(_, ProxyType::Any) => false,
			_ => false,
		}
	}
}

impl pallet_proxy::Config for Runtime {
	type Event = Event;
	type Call = Call;
	type Currency = Balances;
	type ProxyType = ProxyType;
	type ProxyDepositBase = ProxyDepositBase;
	type ProxyDepositFactor = ProxyDepositFactor;
	type MaxProxies = MaxProxies;
	type WeightInfo = ();
	type MaxPending = MaxPending;
	type CallHasher = BlakeTwo256;
	type AnnouncementDepositBase = AnnouncementDepositBase;
	type AnnouncementDepositFactor = AnnouncementDepositFactor;
}

parameter_types! {
	pub MaximumSchedulerWeight: Weight = Perbill::from_percent(10) * BlockWeights::get().max_block;
	pub const MaxScheduledPerBlock: u32 = 30;
}

impl pallet_scheduler::Config for Runtime {
	type Event = Event;
	type Origin = Origin;
	type PalletsOrigin = OriginCaller;
	type Call = Call;
	type MaximumWeight = MaximumSchedulerWeight;
	type ScheduleOrigin = EnsureRoot<AccountId>;
	type MaxScheduledPerBlock = MaxScheduledPerBlock;
	type WeightInfo = ();
}

impl orml_authority::Config for Runtime {
	type Event = Event;
	type Origin = Origin;
	type PalletsOrigin = OriginCaller;
	type Call = Call;
	type Scheduler = Scheduler;
	type AsOriginId = AuthoritysOriginId;
	type AuthorityConfig = AuthorityConfigImpl;
	type WeightInfo = ();
}


impl pallet_sudo::Config for Runtime {
	type Event = Event;
	type Call = Call;
}



pub type ShuraCouncilInstance = pallet_collective::Instance1;
pub type FinancialCouncilInstance = pallet_collective::Instance3;
pub type PublicFundCouncilInstance = pallet_collective::Instance2;
pub type TechnicalCommitteeInstance = pallet_collective::Instance4;

pub type ShuraCouncilMembershipInstance = pallet_membership::Instance1;
pub type FinancialCouncilMembershipInstance = pallet_membership::Instance3;
pub type PublicFundCouncilMembershipInstance = pallet_membership::Instance2;
pub type TechnicalCommitteeMembershipInstance = pallet_membership::Instance4;
pub type OperatorMembershipInstanceSetheum = pallet_membership::Instance5;

// Shura Council
pub type EnsureRootOrAllShuraCouncil = EnsureOneOf<
	AccountId,
	EnsureRoot<AccountId>,
	pallet_collective::EnsureProportionAtLeast<_1, _1, AccountId, ShuraCouncilInstance>,
>;

pub type EnsureRootOrHalfShuraCouncil = EnsureOneOf<
	AccountId,
	EnsureRoot<AccountId>,
	pallet_collective::EnsureProportionAtLeast<_1, _2, AccountId, ShuraCouncilInstance>,
>;

pub type EnsureRootOrOneThirdsShuraCouncil = EnsureOneOf<
	AccountId,
	EnsureRoot<AccountId>,
	pallet_collective::EnsureProportionAtLeast<_1, _3, AccountId, ShuraCouncilInstance>,
>;

pub type EnsureRootOrTwoThirdsShuraCouncil = EnsureOneOf<
	AccountId,
	EnsureRoot<AccountId>,
	pallet_collective::EnsureProportionAtLeast<_2, _3, AccountId, ShuraCouncilInstance>,
>;

pub type EnsureRootOrThreeFourthsShuraCouncil = EnsureOneOf<
	AccountId,
	EnsureRoot<AccountId>,
	pallet_collective::EnsureProportionAtLeast<_3, _4, AccountId, ShuraCouncilInstance>,
>;

// Financial Council
pub type EnsureRootOrAllFinancialCouncil = EnsureOneOf<
	AccountId,
	EnsureRoot<AccountId>,
	pallet_collective::EnsureProportionAtLeast<_1, _1, AccountId, FinancialCouncilInstance>,
>;

pub type EnsureRootOrHalfFinancialCouncil = EnsureOneOf<
	AccountId,
	EnsureRoot<AccountId>,
	pallet_collective::EnsureProportionAtLeast<_1, _2, AccountId, FinancialCouncilInstance>,
>;

pub type EnsureRootOrOneThirdsFinancialCouncil = EnsureOneOf<
	AccountId,
	EnsureRoot<AccountId>,
	pallet_collective::EnsureProportionAtLeast<_1, _3, AccountId, FinancialCouncilInstance>,
>;

pub type EnsureRootOrTwoThirdsFinancialCouncil = EnsureOneOf<
	AccountId,
	EnsureRoot<AccountId>,
	pallet_collective::EnsureProportionAtLeast<_2, _3, AccountId, FinancialCouncilInstance>,
>;

pub type EnsureRootOrThreeFourthsFinancialCouncil = EnsureOneOf<
	AccountId,
	EnsureRoot<AccountId>,
	pallet_collective::EnsureProportionAtLeast<_3, _4, AccountId, FinancialCouncilInstance>,
>;

// SPF (Public Fund) Council
pub type EnsureRootOrAllPublicFundCouncil = EnsureOneOf<
	AccountId,
	EnsureRoot<AccountId>,
	pallet_collective::EnsureProportionAtLeast<_1, _1, AccountId, PublicFundCouncilInstance>,
>;

pub type EnsureRootOrHalfPublicFundCouncil = EnsureOneOf<
	AccountId,
	EnsureRoot<AccountId>,
	pallet_collective::EnsureProportionAtLeast<_1, _2, AccountId, PublicFundCouncilInstance>,
>;

pub type EnsureRootOrOneThirdsPublicFundCouncil = EnsureOneOf<
	AccountId,
	EnsureRoot<AccountId>,
	pallet_collective::EnsureProportionAtLeast<_1, _3, AccountId, PublicFundCouncilInstance>,
>;

pub type EnsureRootOrTwoThirdsPublicFundCouncil = EnsureOneOf<
	AccountId,
	EnsureRoot<AccountId>,
	pallet_collective::EnsureProportionAtLeast<_2, _3, AccountId, PublicFundCouncilInstance>,
>;

pub type EnsureRootOrThreeFourthsPublicFundCouncil = EnsureOneOf<
	AccountId,
	EnsureRoot<AccountId>,
	pallet_collective::EnsureProportionAtLeast<_3, _4, AccountId, PublicFundCouncilInstance>,
>;

// Technical Committee Council
pub type EnsureRootOrAllTechnicalCommittee = EnsureOneOf<
	AccountId,
	EnsureRoot<AccountId>,
	pallet_collective::EnsureProportionAtLeast<_1, _1, AccountId, TechnicalCommitteeInstance>,
>;

pub type EnsureRootOrHalfTechnicalCommittee = EnsureOneOf<
	AccountId,
	EnsureRoot<AccountId>,
	pallet_collective::EnsureProportionAtLeast<_1, _2, AccountId, TechnicalCommitteeInstance>,
>;

pub type EnsureRootOrOneThirdsTechnicalCommittee = EnsureOneOf<
	AccountId,
	EnsureRoot<AccountId>,
	pallet_collective::EnsureProportionAtLeast<_1, _3, AccountId, TechnicalCommitteeInstance>,
>;

pub type EnsureRootOrTwoThirdsTechnicalCommittee = EnsureOneOf<
	AccountId,
	EnsureRoot<AccountId>,
	pallet_collective::EnsureProportionAtLeast<_2, _3, AccountId, TechnicalCommitteeInstance>,
>;

pub type EnsureRootOrThreeFourthsTechnicalCommittee = EnsureOneOf<
	AccountId,
	EnsureRoot<AccountId>,
	pallet_collective::EnsureProportionAtLeast<_3, _4, AccountId, TechnicalCommitteeInstance>,
>;



parameter_types! {
	pub const ShuraCouncilMotionDuration: BlockNumber = 3 * DAYS;
	pub const ShuraCouncilMaxProposals: u32 = 50;
	pub const ShuraCouncilMaxMembers: u32 = 50;
}

type ShuraCouncilInstance = pallet_collective::Instance1;
impl pallet_collective::Config<ShuraCouncilInstance> for Runtime {
	type Origin = Origin;
	type Proposal = Call;
	type Event = Event;
	type MotionDuration = ShuraCouncilMotionDuration;
	type MaxProposals = ShuraCouncilMaxProposals;
	type MaxMembers = ShuraCouncilMaxMembers;
	type DefaultVote = pallet_collective::PrimeDefaultVote;
	type WeightInfo = ();
}

type ShuraCouncilMembershipInstance = pallet_membership::Instance1;
impl pallet_membership::Config<ShuraCouncilMembershipInstance> for Runtime {
	type Event = Event;
	type AddOrigin = EnsureRootOrThreeFourthsShuraCouncil;
	type RemoveOrigin = EnsureRootOrThreeFourthsShuraCouncil;
	type SwapOrigin = EnsureRootOrThreeFourthsShuraCouncil;
	type ResetOrigin = EnsureRootOrThreeFourthsShuraCouncil;
	type PrimeOrigin = EnsureRootOrThreeFourthsShuraCouncil;
	type MembershipInitialized = ShuraCouncil;
	type MembershipChanged = ShuraCouncil;
	type MaxMembers = ShuraCouncilMaxMembers;
	type WeightInfo = ();
}

parameter_types! {
	pub const FinancialCouncilMotionDuration: BlockNumber = 3 * DAYS;
	pub const FinancialCouncilMaxProposals: u32 = 50;
	pub const FinancialCouncilMaxMembers: u32 = 50;
}

type FinancialCouncilInstance = pallet_collective::Instance2;
impl pallet_collective::Config<FinancialCouncilInstance> for Runtime {
	type Origin = Origin;
	type Proposal = Call;
	type Event = Event;
	type MotionDuration = FinancialCouncilMotionDuration;
	type MaxProposals = FinancialCouncilMaxProposals;
	type MaxMembers = FinancialCouncilMaxMembers;
	type DefaultVote = pallet_collective::PrimeDefaultVote;
	type WeightInfo = ();
}

type FinancialCouncilMembershipInstance = pallet_membership::Instance2;
impl pallet_membership::Config<FinancialCouncilMembershipInstance> for Runtime {
	type Event = Event;
	type AddOrigin = EnsureRootOrTwoThirdsShuraCouncil;
	type RemoveOrigin = EnsureRootOrTwoThirdsShuraCouncil;
	type SwapOrigin = EnsureRootOrTwoThirdsShuraCouncil;
	type ResetOrigin = EnsureRootOrTwoThirdsShuraCouncil;
	type PrimeOrigin = EnsureRootOrTwoThirdsShuraCouncil;
	type MembershipInitialized = FinancialCouncil;
	type MembershipChanged = FinancialCouncil;
	type MaxMembers = FinancialCouncilMaxMembers;
	type WeightInfo = ();
}

parameter_types! {
	pub const PublicFundCouncilMotionDuration: BlockNumber = 3 * DAYS;
	pub const PublicFundCouncilMaxProposals: u32 = 50;
	pub const PublicFundCouncilMaxMembers: u32 = 50;
}

type PublicFundCouncilInstance = pallet_collective::Instance3;
impl pallet_collective::Config<PublicFundCouncilInstance> for Runtime {
	type Origin = Origin;
	type Proposal = Call;
	type Event = Event;
	type MotionDuration = PublicFundCouncilMotionDuration;
	type MaxProposals = PublicFundCouncilMaxProposals;
	type MaxMembers = PublicFundCouncilMaxMembers;
	type DefaultVote = pallet_collective::PrimeDefaultVote;
	type WeightInfo = ();
}

type PublicFundCouncilMembershipInstance = pallet_membership::Instance3;
impl pallet_membership::Config<PublicFundCouncilMembershipInstance> for Runtime {
	type Event = Event;
	type AddOrigin = EnsureRootOrTwoThirdsShuraCouncil;
	type RemoveOrigin = EnsureRootOrTwoThirdsShuraCouncil;
	type SwapOrigin = EnsureRootOrTwoThirdsShuraCouncil;
	type ResetOrigin = EnsureRootOrTwoThirdsShuraCouncil;
	type PrimeOrigin = EnsureRootOrTwoThirdsShuraCouncil;
	type MembershipInitialized = PublicFundCouncil;
	type MembershipChanged = PublicFundCouncil;
	type MaxMembers = PublicFundCouncilMaxMembers;
	type WeightInfo = ();
}

parameter_types! {
	pub const TechnicalCommitteeMotionDuration: BlockNumber = 3 * DAYS;
	pub const TechnicalCommitteeMaxProposals: u32 = 50;
	pub const TechnicalCouncilMaxMembers: u32 = 50;
}

type TechnicalCommitteeInstance = pallet_collective::Instance4;
impl pallet_collective::Config<TechnicalCommitteeInstance> for Runtime {
	type Origin = Origin;
	type Proposal = Call;
	type Event = Event;
	type MotionDuration = TechnicalCommitteeMotionDuration;
	type MaxProposals = TechnicalCommitteeMaxProposals;
	type MaxMembers = TechnicalCouncilMaxMembers;
	type DefaultVote = pallet_collective::PrimeDefaultVote;
	type WeightInfo = ();
}

type TechnicalCommitteeMembershipInstance = pallet_membership::Instance4;
impl pallet_membership::Config<TechnicalCommitteeMembershipInstance> for Runtime {
	type Event = Event;
	type AddOrigin = EnsureRootOrTwoThirdsShuraCouncil;
	type RemoveOrigin = EnsureRootOrTwoThirdsShuraCouncil;
	type SwapOrigin = EnsureRootOrTwoThirdsShuraCouncil;
	type ResetOrigin = EnsureRootOrTwoThirdsShuraCouncil;
	type PrimeOrigin = EnsureRootOrTwoThirdsShuraCouncil;
	type MembershipInitialized = TechnicalCommittee;
	type MembershipChanged = TechnicalCommittee;
	type MaxMembers = TechnicalCouncilMaxMembers;
	type WeightInfo = ();
}

parameter_types! {
	pub const OracleMaxMembers: u32 = 50;
}

type OperatorMembershipInstanceSetheum = pallet_membership::Instance5;
impl pallet_membership::Config<OperatorMembershipInstanceSetheum> for Runtime {
	type Event = Event;
	type AddOrigin = EnsureRootOrTwoThirdsShuraCouncil;
	type RemoveOrigin = EnsureRootOrTwoThirdsShuraCouncil;
	type SwapOrigin = EnsureRootOrTwoThirdsShuraCouncil;
	type ResetOrigin = EnsureRootOrTwoThirdsShuraCouncil;
	type PrimeOrigin = EnsureRootOrTwoThirdsShuraCouncil;
	type MembershipInitialized = ();
	type MembershipChanged = SetheumOracle;
	type MaxMembers = OracleMaxMembers;
	type WeightInfo = ();
}

impl pallet_utility::Config for Runtime {
	type Event = Event;
	type Call = Call;
	type WeightInfo = ();
}

parameter_types! {
	pub MultisigDepositBase: Balance = deposit(1, 88);
	pub MultisigDepositFactor: Balance = deposit(0, 32);
	pub const MaxSignatories: u16 = 100;
}

impl pallet_multisig::Config for Runtime {
	type Event = Event;
	type Call = Call;
	type Currency = Balances;
	type DepositBase = MultisigDepositBase;
	type DepositFactor = MultisigDepositFactor;
	type MaxSignatories = MaxSignatories;
	type WeightInfo = ();
}

pub struct ShuraCouncilProvider;
impl SortedMembers<AccountId> for ShuraCouncilProvider {
	fn contains(who: &AccountId) -> bool {
		ShuraCouncil::is_member(who)
	}

	fn sorted_members() -> Vec<AccountId> {
		ShuraCouncil::members()
	}

	#[cfg(feature = "runtime-benchmarks")]
	fn add(_: &AccountId) {
		unimplemented!()
	}
}

impl ContainsLengthBound for ShuraCouncilProvider {
	fn max_len() -> usize {
		ShuraCouncilMaxMembers::get() as usize
	}
	fn min_len() -> usize {
		0
	}
}

parameter_types! {
	pub const ProposalBond: Permill = Permill::from_percent(5);
	pub ProposalBondMinimum: Balance = 2 * dollar(KAR);
	pub const SpendPeriod: BlockNumber = 7 * DAYS;
	pub const Burn: Permill = Permill::from_percent(0);

	pub const TipCountdown: BlockNumber = DAYS;
	pub const TipFindersFee: Percent = Percent::from_percent(5);
	pub TipReportDepositBase: Balance = deposit(1, 0);
	pub BountyDepositBase: Balance = deposit(1, 0);
	pub const BountyDepositPayoutDelay: BlockNumber = 4 * DAYS;
	pub const BountyUpdatePeriod: BlockNumber = 35 * DAYS;
	pub const BountyCuratorDeposit: Permill = Permill::from_percent(50);
	pub BountyValueMinimum: Balance = 5 * dollar(KAR);
	pub DataDepositPerByte: Balance = deposit(0, 1);
	pub const MaximumReasonLength: u32 = 16384;
	pub const MaxApprovals: u32 = 100;
}

type SetheumTreasuryInstance = pallet_treasury::Instance1;
impl pallet_treasury::Config<SetheumTreasuryInstance> for Runtime {
	type ModuleId = TreasuryModuleId;
	type Currency = Balances;
	type ApproveOrigin = EnsureRootOrHalfShuraCouncil;
	type RejectOrigin = EnsureRootOrHalfShuraCouncil;
	type Event = Event;
	type OnSlash = ();
	type ProposalBond = ProposalBond;
	type ProposalBondMinimum = ProposalBondMinimum;
	type SpendPeriod = SpendPeriod;
	type Burn = Burn;
	type BurnDestination = ();
	type SpendFunds = Bounties;
	type WeightInfo = ();
	type MaxApprovals = MaxApprovals;
}

type PublicFundTreasuryInstance = pallet_treasury::Instance2;
impl pallet_treasury::Config<PublicFundTreasuryInstance> for Runtime {
	type ModuleId = TreasuryModuleId;
	type Currency = Balances;
	type ApproveOrigin = EnsureRootOrHalfPublicFundCouncil;
	type RejectOrigin = EnsureRootOrHalfPublicFundCouncil;
	type Event = Event;
	type OnSlash = ();
	type ProposalBond = ProposalBond;
	type ProposalBondMinimum = ProposalBondMinimum;
	type SpendPeriod = SpendPeriod;
	type Burn = Burn;
	type BurnDestination = ();
	type SpendFunds = Bounties;
	type WeightInfo = ();
	type MaxApprovals = MaxApprovals;
}

impl pallet_bounties::Config for Runtime {
	type Event = Event;
	type BountyDepositBase = BountyDepositBase;
	type BountyDepositPayoutDelay = BountyDepositPayoutDelay;
	type BountyUpdatePeriod = BountyUpdatePeriod;
	type BountyCuratorDeposit = BountyCuratorDeposit;
	type BountyValueMinimum = BountyValueMinimum;
	type DataDepositPerByte = DataDepositPerByte;
	type MaximumReasonLength = MaximumReasonLength;
	type WeightInfo = ();
}

impl pallet_tips::Config for Runtime {
	type Event = Event;
	type DataDepositPerByte = DataDepositPerByte;
	type MaximumReasonLength = MaximumReasonLength;
	type Tippers = GeneralCouncilProvider;
	type TipCountdown = TipCountdown;
	type TipFindersFee = TipFindersFee;
	type TipReportDepositBase = TipReportDepositBase;
	type WeightInfo = ();
}

impl orml_auction::Config for Runtime {
	type Event = Event;
	type Balance = Balance;
	type AuctionId = AuctionId;
	type Handler = AuctionManager;
	type WeightInfo = weights::orml_auction::WeightInfo<Runtime>;
}

impl orml_authority::Config for Runtime {
	type Event = Event;
	type Origin = Origin;
	type PalletsOrigin = OriginCaller;
	type Call = Call;
	type Scheduler = Scheduler;
	type AsOriginId = AuthoritysOriginId;
	type AuthorityConfig = AuthorityConfigImpl;
	type WeightInfo = weights::orml_authority::WeightInfo<Runtime>;
}

parameter_types! {
	pub const MinimumCount: u32 = 3;
	pub const ExpiresIn: Moment = 1000 * 60 * 60 * 2; // 2 hours
	pub ZeroAccountId: AccountId = AccountId::from([0u8; 32]);
}

type SetheumDataProvider = orml_oracle::Instance1;
impl orml_oracle::Config<SetheumDataProvider> for Runtime {
	type Event = Event;
	type OnNewData = ();
	type CombineData = orml_oracle::DefaultCombineData<Runtime, MinimumCount, ExpiresIn, SetheumDataProvider>;
	type Time = Timestamp;
	type OracleKey = CurrencyId;
	type OracleValue = Price;
	type RootOperatorAccountId = ZeroAccountId;
	type Members = OperatorMembershipSetheum;
	type WeightInfo = ();
}

create_median_value_data_provider!(
	AggregatedDataProvider,
	CurrencyId,
	Price,
	TimeStampedPrice,
	[SetheumOracle]
);
// Aggregated data provider cannot feed.
impl DataFeeder<CurrencyId, Price, AccountId> for AggregatedDataProvider {
	fn feed_value(_: AccountId, _: CurrencyId, _: Price) -> DispatchResult {
		Err("Not supported".into())
	}
}


// Create the runtime by composing the FRAME pallets that were previously configured.

// workaround for a weird bug in macro
use pallet_session::historical as pallet_session_historical;

// TODO: Implementation of `From` is preferred since it gives you `Into<_>` for free where the reverse isn't true.
// After this TODO will be resolved, remove the suppresion of `from-over-into` warnings in the Makefile.
construct_runtime!(
	pub enum Runtime where
		Block = Block,
		NodeBlock = opaque::Block,
		UncheckedExtrinsic = UncheckedExtrinsic
	{
		// Core
		System: frame_system::{Module, Call, Config, Storage, Event<T>} = 0,
		RandomnessCollectiveFlip: pallet_randomness_collective_flip::{Module, Call, Storage} = 1,
		Timestamp: pallet_timestamp::{Module, Call, Storage, Inherent} = 2,
		Sudo: pallet_sudo::{Module, Call, Config<T>, Storage, Event<T>} = 3,
		Scheduler: pallet_scheduler::{Module, Call, Storage, Event<T>} = 4,
		TransactionPayment: module_transaction_payment::{Module, Call, Storage} = 9,
		Prices: module_prices::{Pallet, Storage, Call, Event<T>} = 90,
		Dex: module_dex::{Pallet, Storage, Call, Event<T>, Config<T>} = 91,

		// Identity
		Identity: pallet_identity::{Module, Call, Storage, Event<T>} = 40,

		// Account lookup
		Indices: pallet_indices::{Module, Call, Storage, Config<T>, Event<T>} = 5,

		// Account recovery
		Recovery: module_recovery::{Module, Call, Storage, Event<T>} = 5,

		// Treasury
		Treasury: pallet_treasury::<Instance1>::{Module, Call, Storage, Config, Event<T>} = 20,
		PublicFund: pallet_treasury::<Instance2>::{Module, Call, Storage, Config, Event<T>} = 20,
		Bounties: pallet_bounties::{Module, Call, Storage, Event<T>} = 21,
		Tips: pallet_tips::{Module, Call, Storage, Event<T>} = 22,

		// ORML Core
		Auction: orml_auction::{Pallet, Storage, Call, Event<T>} = 80,
		OrmlNFT: orml_nft::{Pallet, Storage, Config<T>} = 82,

		// Tokens
		Balances: pallet_balances::{Module, Call, Storage, Config<T>, Event<T>} = 6,
		Currencies: module_currencies::{Module, Call, Event<T>} = 7,
		Tokens: orml_tokens::{Module, Storage, Event<T>, Config<T>} = 8,
		NFT: module_nft::{Module, Call, Event<T>} = 121,

		// Smart contracts - SEVM
		EvmAccounts: module_evm_accounts::{Module, Call, Storage, Event<T>} = 20,
		Evm: module_evm::{Module, Config<T>, Call, Storage, Event<T>} = 21,
		EVMBridge: module_evm_bridge::{Module} = 22,
		EvmManager: module_evm_manager::{Module, Storage} = 133,

		// SERP & SetMint
		AuctionManager: auction_manager::{Module, Storage, Call, Event<T>, ValidateUnsigned} = 100,
		Loans: module_loans::{Module, Storage, Call, Event<T>} = 101,
		SetMint: serp_setmint::{Module, Storage, Call, Event<T>} = 102,
		CdpTreasury: cdp_treasury::{Module, Storage, Call, Config, Event<T>} = 103,
		SerpTreasury: serp_treasury::{Module, Storage, Call, Config, Event<T>} = 103,
		SerpOcw: serp_ocw::{Module, Storage, Call, Config, Event<T>} = 103,
		CdpEngine: cdp_engine::{Module, Storage, Call, Event<T>, Config, ValidateUnsigned} = 104,
		EmergencyShutdown: emergency_shutdown::{Module, Storage, Call, Event<T>} = 105,

		// Consensus
		Authorship: pallet_authorship::{Module, Call, Storage, Inherent} = 30,
		Babe: pallet_babe::{Module, Call, Storage, Config, Inherent, ValidateUnsigned} = 31,
		Grandpa: pallet_grandpa::{Module, Call, Storage, Config, Event, ValidateUnsigned} = 32,
		Staking: pallet_staking::{Module, Call, Config<T>, Storage, Event<T>} = 33,
		Session: pallet_session::{Module, Call, Storage, Event, Config<T>} = 34,
		Historical: pallet_session_historical::{Module} = 35,
		Offences: pallet_offences::{Module, Call, Storage, Event} = 36,
		ImOnline: pallet_im_online::{Module, Call, Storage, Event<T>, ValidateUnsigned, Config<T>} = 37,
		AuthorityDiscovery: pallet_authority_discovery::{Module, Call, Config} = 38,

		// Governance
		Authority: orml_authority::{Module, Call, Event<T>, Origin<T>} = 60,
		ShuraCouncil: pallet_collective::<Instance1>::{Module, Call, Storage, Origin<T>, Event<T>, Config<T>} = 61,
		ShuraCouncilMembership: pallet_membership::<Instance1>::{Module, Call, Storage, Event<T>, Config<T>} = 62,
		FinancialCouncil: pallet_collective::<Instance2>::{Module, Call, Storage, Origin<T>, Event<T>, Config<T>} = 63,
		FinancialCouncilMembership: pallet_membership::<Instance2>::{Module, Call, Storage, Event<T>, Config<T>} = 64,
		PublicFundCouncil: pallet_collective::<Instance3>::{Module, Call, Storage, Origin<T>, Event<T>, Config<T>} = 65,
		PublicFundCouncilMembership: pallet_membership::<Instance3>::{Module, Call, Storage, Event<T>, Config<T>} = 66,
		TechnicalCommittee: pallet_collective::<Instance4>::{Module, Call, Storage, Origin<T>, Event<T>, Config<T>} = 67,
		TechnicalCommitteeMembership: pallet_membership::<Instance4>::{Module, Call, Storage, Event<T>, Config<T>} = 68,
		Democracy: pallet_democracy::{Module, Call, Storage, Config<T>, Event<T>} = 69,

		// Oracle
		//
		// NOTE: OperatorMembership must be placed after Oracle or else will have race condition on initialization
		SetheumOracle: orml_oracle::<Instance1>::{Module, Storage, Call, Event<T>} = 70,
		OperatorMembershipSetheum: pallet_membership::<Instance5>::{Module, Call, Storage, Event<T>, Config<T>} = 71,
	}
);

/// The address format for describing accounts.
pub type Address = sp_runtime::MultiAddress<AccountId, AccountIndex>;
/// Block header type as expected by this runtime.
pub type Header = generic::Header<BlockNumber, BlakeTwo256>;
/// Block type as expected by this runtime.
pub type Block = generic::Block<Header, UncheckedExtrinsic>;
/// A Block signed with a Justification
pub type SignedBlock = generic::SignedBlock<Block>;
/// BlockId type as expected by this runtime.
pub type BlockId = generic::BlockId<Block>;
/// The SignedExtension to the basic transaction logic.
pub type SignedExtra = (
	frame_system::CheckSpecVersion<Runtime>,
	frame_system::CheckTxVersion<Runtime>,
	frame_system::CheckGenesis<Runtime>,
	frame_system::CheckEra<Runtime>,
	frame_system::CheckNonce<Runtime>,
	frame_system::CheckWeight<Runtime>,
	module_transaction_payment::ChargeTransactionPayment<Runtime>,
	module_evm::SetEvmOrigin<Runtime>,
);
/// Unchecked extrinsic type as expected by this runtime.
pub type UncheckedExtrinsic = generic::UncheckedExtrinsic<Address, Call, Signature, SignedExtra>;
/// The payload being signed in transactions.
pub type SignedPayload = generic::SignedPayload<Call, SignedExtra>;
/// Extrinsic type that has already been checked.
pub type CheckedExtrinsic = generic::CheckedExtrinsic<AccountId, Call, SignedExtra>;
/// Executive: handles dispatch to the various modules.
pub type Executive = frame_executive::Executive<
	Runtime,
	Block,
	frame_system::ChainContext<Runtime>,
	Runtime,
	AllModules,
>;

impl<LocalCall> frame_system::offchain::CreateSignedTransaction<LocalCall> for Runtime
where
	Call: From<LocalCall>,
{
	fn create_transaction<C: frame_system::offchain::AppCrypto<Self::Public, Self::Signature>>(
		call: Call,
		public: <Signature as sp_runtime::traits::Verify>::Signer,
		account: AccountId,
		nonce: Nonce,
	) -> Option<(
		Call,
		<UncheckedExtrinsic as sp_runtime::traits::Extrinsic>::SignaturePayload,
	)> {
		// take the biggest period possible.
		let period = BlockHashCount::get()
			.checked_next_power_of_two()
			.map(|c| c / 2)
			.unwrap_or(2) as u64;
		let current_block = System::block_number()
			.saturated_into::<u64>()
			// The `System::block_number` is initialized with `n+1`,
			// so the actual block number is `n`.
			.saturating_sub(1);
		let tip = 0;
		let extra: SignedExtra = (
			frame_system::CheckSpecVersion::<Runtime>::new(),
			frame_system::CheckTxVersion::<Runtime>::new(),
			frame_system::CheckGenesis::<Runtime>::new(),
			frame_system::CheckEra::<Runtime>::from(generic::Era::mortal(period, current_block)),
			frame_system::CheckNonce::<Runtime>::from(nonce),
			frame_system::CheckWeight::<Runtime>::new(),
			module_transaction_payment::ChargeTransactionPayment::<Runtime>::from(tip),
			module_evm::SetEvmOrigin::<Runtime>::new(),
		);
		let raw_payload = SignedPayload::new(call, extra)
			.map_err(|e| {
				debug::warn!("Unable to create signed payload: {:?}", e);
			})
			.ok()?;
		let signature = raw_payload.using_encoded(|payload| C::sign(payload, public))?;
		let address = Indices::unlookup(account);
		let (call, extra, _) = raw_payload.deconstruct();
		Some((call, (address, signature, extra)))
	}
}

impl frame_system::offchain::SigningTypes for Runtime {
	type Public = <Signature as sp_runtime::traits::Verify>::Signer;
	type Signature = Signature;
}

impl<C> frame_system::offchain::SendTransactionTypes<C> for Runtime
where
	Call: From<C>,
{
	type OverarchingCall = Call;
	type Extrinsic = UncheckedExtrinsic;
}

impl_runtime_apis! {
	impl sp_api::Core<Block> for Runtime {
		fn version() -> RuntimeVersion {
			VERSION
		}

		fn execute_block(block: Block) {
			Executive::execute_block(block)
		}

		fn initialize_block(header: &<Block as BlockT>::Header) {
			Executive::initialize_block(header)
		}
	}

	impl sp_api::Metadata<Block> for Runtime {
		fn metadata() -> OpaqueMetadata {
			Runtime::metadata().into()
		}
	}

	impl sp_block_builder::BlockBuilder<Block> for Runtime {
		fn apply_extrinsic(extrinsic: <Block as BlockT>::Extrinsic) -> ApplyExtrinsicResult {
			Executive::apply_extrinsic(extrinsic)
		}

		fn finalize_block() -> <Block as BlockT>::Header {
			Executive::finalize_block()
		}

		fn inherent_extrinsics(data: sp_inherents::InherentData) -> Vec<<Block as BlockT>::Extrinsic> {
			data.create_extrinsics()
		}

		fn check_inherents(
			block: Block,
			data: sp_inherents::InherentData,
		) -> sp_inherents::CheckInherentsResult {
			data.check_extrinsics(&block)
		}

		fn random_seed() -> <Block as BlockT>::Hash {
			RandomnessCollectiveFlip::random_seed()
		}
	}

	impl sp_transaction_pool::runtime_api::TaggedTransactionQueue<Block> for Runtime {
		fn validate_transaction(
			source: TransactionSource,
			tx: <Block as BlockT>::Extrinsic,
		) -> TransactionValidity {
			Executive::validate_transaction(source, tx)
		}
	}

	impl sp_offchain::OffchainWorkerApi<Block> for Runtime {
		fn offchain_worker(header: &<Block as BlockT>::Header) {
			Executive::offchain_worker(header)
		}
	}

	impl sp_consensus_babe::BabeApi<Block> for Runtime {
		fn configuration() -> sp_consensus_babe::BabeGenesisConfiguration {
			sp_consensus_babe::BabeGenesisConfiguration {
				slot_duration: Babe::slot_duration(),
				epoch_length: EpochDuration::get(),
				c: PRIMARY_PROBABILITY,
				genesis_authorities: Babe::authorities(),
				randomness: Babe::randomness(),
				allowed_slots: sp_consensus_babe::AllowedSlots::PrimaryAndSecondaryPlainSlots,
			}
		}

		fn current_epoch_start() -> sp_consensus_babe::Slot {
			Babe::current_epoch_start()
		}

		fn current_epoch() -> sp_consensus_babe::Epoch {
			Babe::current_epoch()
		}

		fn next_epoch() -> sp_consensus_babe::Epoch {
			Babe::next_epoch()
		}

		fn generate_key_ownership_proof(
			_slot_number: sp_consensus_babe::Slot,
			authority_id: sp_consensus_babe::AuthorityId,
			) -> Option<sp_consensus_babe::OpaqueKeyOwnershipProof> {
			use codec::Encode;

			Historical::prove((sp_consensus_babe::KEY_TYPE, authority_id))
				.map(|p| p.encode())
				.map(sp_consensus_babe::OpaqueKeyOwnershipProof::new)
		}

		fn submit_report_equivocation_unsigned_extrinsic(
			equivocation_proof: sp_consensus_babe::EquivocationProof<<Block as BlockT>::Header>,
			key_owner_proof: sp_consensus_babe::OpaqueKeyOwnershipProof,
			) -> Option<()> {
			let key_owner_proof = key_owner_proof.decode()?;

			Babe::submit_unsigned_equivocation_report(
				equivocation_proof,
				key_owner_proof,
				)
		}
	}

	impl sp_authority_discovery::AuthorityDiscoveryApi<Block> for Runtime {
		fn authorities() -> Vec<AuthorityDiscoveryId> {
			AuthorityDiscovery::authorities()
		}
	}

	impl sp_session::SessionKeys<Block> for Runtime {
		fn generate_session_keys(seed: Option<Vec<u8>>) -> Vec<u8> {
			opaque::SessionKeys::generate(seed)
		}

		fn decode_session_keys(
			encoded: Vec<u8>,
		) -> Option<Vec<(Vec<u8>, KeyTypeId)>> {
			opaque::SessionKeys::decode_into_raw_public_keys(&encoded)
		}
	}

	impl fg_primitives::GrandpaApi<Block> for Runtime {
		fn grandpa_authorities() -> GrandpaAuthorityList {
			Grandpa::grandpa_authorities()
		}

		fn submit_report_equivocation_unsigned_extrinsic(
			_equivocation_proof: fg_primitives::EquivocationProof<
				<Block as BlockT>::Hash,
				NumberFor<Block>,
			>,
			_key_owner_proof: fg_primitives::OpaqueKeyOwnershipProof,
		) -> Option<()> {
			None
		}

		fn generate_key_ownership_proof(
			_set_id: fg_primitives::SetId,
			_authority_id: GrandpaId,
		) -> Option<fg_primitives::OpaqueKeyOwnershipProof> {
			// NOTE: this is the only implementation possible since we've
			// defined our key owner proof type as a bottom type (i.e. a type
			// with no values).
			None
		}
	}

	impl frame_system_rpc_runtime_api::AccountNonceApi<Block, AccountId, Nonce> for Runtime {
		fn account_nonce(account: AccountId) -> Nonce {
			System::account_nonce(account)
		}
	}

	impl pallet_transaction_payment_rpc_runtime_api::TransactionPaymentApi<Block, Balance> for Runtime {
		fn query_info(
			uxt: <Block as BlockT>::Extrinsic,
			len: u32,
		) -> pallet_transaction_payment_rpc_runtime_api::RuntimeDispatchInfo<Balance> {
			TransactionPayment::query_info(uxt, len)
		}
		fn query_fee_details(
			uxt: <Block as BlockT>::Extrinsic,
			len: u32,
		) -> pallet_transaction_payment::FeeDetails<Balance> {
			TransactionPayment::query_fee_details(uxt, len)
		}
	}

	impl module_evm_rpc_runtime_api::EVMRuntimeRPCApi<Block, Balance> for Runtime {
		fn call(
			from: H160,
			to: H160,
			data: Vec<u8>,
			value: Balance,
			gas_limit: u64,
			storage_limit: u32,
			estimate: bool,
		) -> Result<CallInfo, sp_runtime::DispatchError> {
			let mut config = <Runtime as module_evm::Config>::config().clone();
			if estimate {
				config.estimate = true;
			}
			module_evm::Runner::<Runtime>::call(
				from,
				from,
				to,
				data,
				value,
				gas_limit,
				storage_limit,
				&config,
			)
		}

		fn create(
			from: H160,
			data: Vec<u8>,
			value: Balance,
			gas_limit: u64,
			storage_limit: u32,
			estimate: bool,
		) -> Result<CreateInfo, sp_runtime::DispatchError> {
			let mut config = <Runtime as module_evm::Config>::config().clone();
			if estimate {
				config.estimate = true;
			}
			module_evm::Runner::<Runtime>::create(
				from,
				data,
				value,
				gas_limit,
				storage_limit,
				&config,
			)
		}

		fn get_estimate_resources_request(
			extrinsic: Vec<u8>,
		) -> Result<EstimateResourcesRequest, sp_runtime::DispatchError> {

			let utx = UncheckedExtrinsic::decode(&mut &*extrinsic)
				.map_err(|_| sp_runtime::DispatchError::Other("Invalid parameter extrinsic, decode failed"))?;

			let request = match utx.function {
				Call::Evm(module_evm::Call::call(to, data, value, gas_limit, storage_limit)) => {
					Some(EstimateResourcesRequest {
						from: None,
						to: Some(to),
						gas_limit: Some(gas_limit),
						storage_limit: Some(storage_limit),
						value: Some(value),
						data: Some(data),
					})
				}
				Call::Evm(module_evm::Call::create(data, value, gas_limit, storage_limit)) => {
					Some(EstimateResourcesRequest {
						from: None,
						to: None,
						gas_limit: Some(gas_limit),
						storage_limit: Some(storage_limit),
						value: Some(value),
						data: Some(data),
					})
				}
				_ => None,
			};

			request.ok_or(sp_runtime::DispatchError::Other("Invalid parameter extrinsic, not evm Call"))
		}

	}

	// TODO: Update!
	#[cfg(feature = "runtime-benchmarks")]
	impl frame_benchmarking::Benchmark<Block> for Runtime {
		fn dispatch_benchmark(
			config: frame_benchmarking::BenchmarkConfig
		) -> Result<Vec<frame_benchmarking::BenchmarkBatch>, sp_runtime::RuntimeString> {
			use frame_benchmarking::{Benchmarking, BenchmarkBatch, add_benchmark, TrackedStorageKey};
			use orml_benchmarking::{add_benchmark as orml_add_benchmark};

			use module_nft::benchmarking::Module as NftBench;

			use frame_system_benchmarking::Module as SystemBench;
			impl frame_system_benchmarking::Config for Runtime {}

			let whitelist: Vec<TrackedStorageKey> = vec![
				// Block Number
				hex_literal::hex!("26aa394eea5630e07c48ae0c9558cef702a5c1b19ab7a04f536c519aca4983ac").to_vec().into(),
				// Total Issuance
				hex_literal::hex!("c2261276cc9d1f8598ea4b6a74b15c2f57c875e4cff74148e4628f264b974c80").to_vec().into(),
				// Execution Phase
				hex_literal::hex!("26aa394eea5630e07c48ae0c9558cef7ff553b5a9862a516939d82b3d3d8661a").to_vec().into(),
				// Event Count
				hex_literal::hex!("26aa394eea5630e07c48ae0c9558cef70a98fdbe9ce6c55837576c60c7af3850").to_vec().into(),
				// System Events
				hex_literal::hex!("26aa394eea5630e07c48ae0c9558cef780d41e5e16056765bc8461851072c9d7").to_vec().into(),
				/// TODO: Update hexcode
				// Caller 0 Account
				hex_literal::hex!("26aa394eea5630e07c48ae0c9558cef7b99d880ec681799c0cf30e8886371da946c154ffd9992e395af90b5b13cc6f295c77033fce8a9045824a6690bbf99c6db269502f0a8d1d2a008542d5690a0749").to_vec().into(),
				// Caller 1 Account
				hex_literal::hex!("26aa394eea5630e07c48ae0c9558cef7b99d880ec681799c0cf30e8886371da946c154ffd9992e395af90b5b13cc6f295c77033fce8a9045824a6690bbf99c6db269502f0a8d1d2a008542d5690a0749").to_vec().into(),
				// Treasury Account
				hex_literal::hex!("26aa394eea5630e07c48ae0c9558cef7b99d880ec681799c0cf30e8886371da95ecffd7b6c0f78751baa9d281e0bfa3a6d6f646c70792f74727372790000000000000000000000000000000000000000").to_vec().into(),
			];

			let mut batches = Vec::<BenchmarkBatch>::new();
			let params = (&config, &whitelist);

			add_benchmark!(params, batches, frame_system, SystemBench::<Runtime>);
			add_benchmark!(params, batches, pallet_balances, Balances);
			add_benchmark!(params, batches, pallet_timestamp, Timestamp);

			add_benchmark!(params, batches, module_nft, NftBench::<Runtime>);

			orml_add_benchmark!(params, batches, module_dex, benchmarking::dex);
			orml_add_benchmark!(params, batches, auction_manager, benchmarking::auction_manager);
			orml_add_benchmark!(params, batches, cdp_engine, benchmarking::cdp_engine);
			orml_add_benchmark!(params, batches, emergency_shutdown, benchmarking::emergency_shutdown);
			orml_add_benchmark!(params, batches, module_evm, benchmarking::evm);
			orml_add_benchmark!(params, batches, serp_setmint, benchmarking::setmint);
			orml_add_benchmark!(params, batches, serp_treasury, benchmarking::serp_treasury);
			orml_add_benchmark!(params, batches, cdp_treasury, benchmarking::cdp_treasury);
			orml_add_benchmark!(params, batches, module_transaction_payment, benchmarking::transaction_payment);
			orml_add_benchmark!(params, batches, module_prices, benchmarking::prices);
			orml_add_benchmark!(params, batches, module_evm_accounts, benchmarking::evm_accounts);
			orml_add_benchmark!(params, batches, module_currencies, benchmarking::currencies);

			orml_add_benchmark!(params, batches, orml_tokens, benchmarking::tokens);
			orml_add_benchmark!(params, batches, orml_auction, benchmarking::auction);

			orml_add_benchmark!(params, batches, orml_authority, benchmarking::authority);
			orml_add_benchmark!(params, batches, orml_oracle, benchmarking::oracle);

			if batches.is_empty() { return Err("Benchmark not found for this pallet.".into()) }
			Ok(batches)
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use frame_support::weights::DispatchClass;
	use frame_system::offchain::CreateSignedTransaction;
	use sp_runtime::traits::Convert;

	fn run_with_system_weight<F>(w: Weight, mut assertions: F)
	where
		F: FnMut() -> (),
	{
		let mut t: sp_io::TestExternalities = frame_system::GenesisConfig::default()
			.build_storage::<Runtime>()
			.unwrap()
			.into();
		t.execute_with(|| {
			System::set_block_consumed_resources(w, 0);
			assertions()
		});
	}

	#[test]
	fn validate_transaction_submitter_bounds() {
		fn is_submit_signed_transaction<T>()
		where
			T: CreateSignedTransaction<Call>,
		{
		}

		is_submit_signed_transaction::<Runtime>();
	}

	#[test]
	fn multiplier_can_grow_from_zero() {
		let minimum_multiplier = MinimumMultiplier::get();
		let target =
			TargetBlockFullness::get() * RuntimeBlockWeights::get().get(DispatchClass::Normal).max_total.unwrap();
		// if the min is too small, then this will not change, and we are doomed forever.
		// the weight is 1/100th bigger than target.
		run_with_system_weight(target * 101 / 100, || {
			let next = SlowAdjustingFeeUpdate::<Runtime>::convert(minimum_multiplier);
			assert!(next > minimum_multiplier, "{:?} !>= {:?}", next, minimum_multiplier);
		})
	}

	#[test]
	fn ensure_can_create_contract() {
		// Ensure that the `ExistentialDeposit` for creating the contract >= account `ExistentialDeposit`.
		// Otherwise, the creation of the contract account will fail because it is less than
		// ExistentialDeposit.
		assert!(
			Balance::from(NewContractExtraBytes::get()) * StorageDepositPerByte::get()
				>= NativeTokenExistentialDeposit::get()
		);
	}
}
