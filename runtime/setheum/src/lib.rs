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

//! The Setheum runtime. This can be compiled with `#[no_std]`, ready for Wasm.

#![cfg_attr(not(feature = "std"), no_std)]
// `construct_runtime!` does a lot of recursion and requires us to increase the limit to 256.
#![recursion_limit = "256"]
// The `large_enum_variant` warning originates from `construct_runtime` macro.
#![allow(clippy::large_enum_variant)]
#![allow(clippy::unnecessary_mut_passed)]
#![allow(clippy::or_fun_call)]
#![allow(clippy::from_over_into)]
#![allow(clippy::upper_case_acronyms)]

// Make the WASM binary available.
#[cfg(feature = "std")]
include!(concat!(env!("OUT_DIR"), "/wasm_binary.rs"));

use codec::Encode;
pub use frame_support::{
	construct_runtime, debug, parameter_types,
	traits::{
		Contains, ContainsLengthBound, EnsureOrigin, Filter, Get, IsType, KeyOwnerProofSystem, LockIdentifier,
		Randomness, SortedMembers, U128CurrencyToVote, WithdrawReasons,
	},
	weights::{
		constants::{
			BlockExecutionWeight,
			ExtrinsicBaseWeight,
			RocksDbWeight,
			WEIGHT_PER_SECOND
		},
		IdentityFee, Weight
	},
	PalletId, StorageValue,
};

use hex_literal::hex;
use sp_api::impl_runtime_apis;
use sp_core::{
	crypto::KeyTypeId,
	u32_trait::{_1, _2, _3, _4},
	OpaqueMetadata, H160,
};
use sp_runtime::traits::{
	BadOrigin, BlakeTwo256, Block as BlockT, NumberFor, OpaqueKeys, SaturatedConversion, StaticLookup, Zero,
};
use sp_runtime::{
	create_runtime_str,
	curve::PiecewiseLinear,
	generic, impl_opaque_keys,
	traits::{AccountIdConversion, Zero, Convert, Identity},
	transaction_validity::{TransactionSource, TransactionValidity},
	ApplyExtrinsicResult, DispatchResult, FixedPointNumber,
};
use sp_std::prelude::*;
#[cfg(feature = "std")]
use sp_version::NativeVersion;
use sp_version::RuntimeVersion;

use frame_system::{EnsureOneOf, EnsureRoot, RawOrigin};
use setheum_currencies::{BasicCurrencyAdapter, Currency};
use setheum_evm::{CallInfo, CreateInfo};
use setheum_evm_accounts::EvmAddressMapping;
use setheum_evm_manager::EvmCurrencyIdMapping;
use setheum_support::CurrencyIdMapping;
use setheum_transaction_payment::{Multiplier, TargetedFeeAdjustment};
use orml_tokens::CurrencyAdapter;
use orml_traits::{create_median_value_data_provider, parameter_type_with_key, DataFeeder, DataProviderExtended};
use pallet_grandpa::fg_primitives;
use pallet_grandpa::{AuthorityId as GrandpaId, AuthorityList as GrandpaAuthorityList};
use pallet_session::historical as pallet_session_historical;

/// Weights for pallets used in the runtime.
mod weights;

pub use pallet_staking::StakerStatus;
pub use pallet_timestamp::Call as TimestampCall;
#[cfg(any(feature = "std", test))]
pub use sp_runtime::BuildStorage;
pub use sp_runtime::{Perbill, Percent, Permill, Perquintill};

pub use authority::AuthorityConfigImpl;
pub use constants::{fee::*, time::*};
pub use primitives::{
	evm::EstimateResourcesRequest, AccountId, AccountIndex, Amount, AuctionId,
	AuthoritysOriginId, Balance, BlockNumber, CurrencyId, DataProviderId,
	EraIndex, Hash, Moment, Nonce, Share, Signature, TokenSymbol, TradingPair,
};
pub use runtime_common::{
	cent, deposit, dollar, microcent, millicent, BlockLength, BlockWeights,
	ExchangeRate, GasToWeight, OffchainSolutionWeightLimit, Price, Rate, Ratio,
	RuntimeBlockLength, RuntimeBlockWeights,SystemContractsFilter, TimeStampedPrice, 
	DNAR, SDEX, SETT, USDJ, EURJ, JPYJ, GBPJ, AUDJ, CADJ, CHFJ, SGDJ, BRLJ, SARJ RENBTC,
	USD, EUR, JPY, GBP, AUD, CAD, CHF, SGD, BRL, SAR, KWD, JOD, BHD, KYD, OMR, GIP
};
mod authority;
mod benchmarking;
mod constants;

/// This runtime version.
pub const VERSION: RuntimeVersion = RuntimeVersion {
	spec_name: create_runtime_str!("newrome"),
	impl_name: create_runtime_str!("newrome"),
	authoring_version: 1,
	spec_version: 1000,
	impl_version: 0,
	apis: RUNTIME_API_VERSIONS,
	transaction_version: 1,
};

/// The version infromation used to identify this runtime when compiled
/// natively.
#[cfg(feature = "std")]
pub fn native_version() -> NativeVersion {
	NativeVersion {
		runtime_version: VERSION,
		can_author_with: Default::default(),
	}
}

impl_opaque_keys! {
	pub struct SessionKeys {
		pub grandpa: Grandpa,
		pub babe: Babe,
	}
}

// Module accounts of runtime
parameter_types! {
	pub const TreasuryPalletId: PalletId = PalletId(*b"set/trsy");
	pub const SettmintManagerPalletId: PalletId = PalletId(*b"set/mint");
	pub const DexPalletId: PalletId = PalletId(*b"set/dexm");
	pub const SerpTreasuryPalletId: PalletId = PalletId(*b"set/serp");
	pub const SettPayTreasuryPalletId: PalletId = PalletId(*b"set/stpy");
	pub const WellfareTreasuryPalletId: PalletId = PalletId(*b"set/welf");
	pub const IncentivesPalletId: PalletId = PalletId(*b"set/inct");
	pub const NftPalletId: PalletId = PalletId(*b"set/sNFT");
}

pub fn get_all_module_accounts() -> Vec<AccountId> {
	vec![
		TreasuryPalletId::get().into_account(),
		SettmintManagerPalletId::get().into_account(),
		DexPalletId::get().into_account(),
		SerpTreasuryPalletId::get().into_account(),
		SettPayTreasuryPalletId::get().into_account(),
		WellfareTreasuryPalletId::get().into_account(),
		IncentivesPalletId::get().into_account(),
		ZeroAccountId::get(),
	]
}

parameter_types! {
	pub const BlockHashCount: BlockNumber = 1200; // mortal tx can be valid up to 4 hour after signing
	pub const Version: RuntimeVersion = VERSION;
	pub const SS58Prefix: u16 = 258; // Ss58AddressFormat::SetheumAccount
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
			Call::Authority(_) | Call::GeneralCouncil(_) | Call::GeneralCouncilMembership(_) |
			Call::SetheumJury(_) | Call::SetheumJuryMembership(_) |
			Call::FinancialCouncil(_) | Call::FinancialCouncilMembership(_) |
			Call::TechnicalCommittee(_) | Call::TechnicalCommitteeMembership(_) |
			// Oracle
			Call::SetheumOracle(_) | Call::OperatorMembershipSetheum(_)
		)
	}
}

impl frame_system::Config for Runtime {
	type AccountId = AccountId;
	type Call = Call;
	type Lookup = Indices;
	type Index = Nonce;
	type BlockNumber = BlockNumber;
	type Hash = Hash;
	type Hashing = BlakeTwo256;
	type Header = generic::Header<BlockNumber, BlakeTwo256>;
	type Event = Event;
	type Origin = Origin;
	type BlockHashCount = BlockHashCount;
	type BlockWeights = BlockWeights;
	type BlockLength = BlockLength;
	type Version = Version;
	type PalletInfo = PalletInfo;
	type AccountData = pallet_balances::AccountData<Balance>;
	type OnNewAccount = ();
	type OnKilledAccount = (
		setheum_evm::CallKillAccount<Runtime>,
		setheum_evm_accounts::CallKillAccount<Runtime>,
	);
	type DbWeight = RocksDbWeight;
	type BaseCallFilter = ();
	type SystemWeightInfo = ();
	type SS58Prefix = SS58Prefix;
	type OnSetCode = ();
}

parameter_types! {
	pub const EpochDuration: u64 = EPOCH_DURATION_IN_SLOTS;
	pub const ExpectedBlockTime: Moment = MILLISECS_PER_BLOCK;
	pub const ReportLongevity: u64 =
		BondingDuration::get() as u64 * SessionsPerEra::get() as u64 * EpochDuration::get();
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
	type HandleEquivocation = pallet_babe::EquivocationHandler<Self::KeyOwnerIdentification, (), ReportLongevity>; // Offences
	type WeightInfo = ();
}

impl pallet_grandpa::Config for Runtime {
	type Event = Event;
	type Call = Call;

	type KeyOwnerProofSystem = Historical;

	type KeyOwnerProof = <Self::KeyOwnerProofSystem as KeyOwnerProofSystem<(KeyTypeId, GrandpaId)>>::Proof;

	type KeyOwnerIdentification =
		<Self::KeyOwnerProofSystem as KeyOwnerProofSystem<(KeyTypeId, GrandpaId)>>::IdentificationTuple;

	type HandleEquivocation = pallet_grandpa::EquivocationHandler<Self::KeyOwnerIdentification, (), ReportLongevity>; // Offences

	type WeightInfo = ();
}

parameter_types! {
	pub IndexDeposit: Balance = dollar(DNAR);
}

impl pallet_indices::Config for Runtime {
	type AccountIndex = AccountIndex;
	type Event = Event;
	type Currency = Balances;
	type Deposit = IndexDeposit;
	type WeightInfo = ();
}

parameter_types! {
	pub const MinimumPeriod: u64 = SLOT_DURATION / 2;
}

impl pallet_timestamp::Config for Runtime {
	/// A timestamp: milliseconds since the unix epoch.
	type Moment = Moment;
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
	type EventHandler = (Staking, ()); // ImOnline
}

parameter_types! {
	pub const NativeTokenExistentialDeposit: Balance = 0;
	// For weight estimation, we assume that the most locks on an individual account will be 50.
	// This number may need to be adjusted in the future if this assumption no longer holds true.
	pub const MaxLocks: u32 = 50;
	pub const MaxReserves: u32 = 50;
}

impl pallet_balances::Config for Runtime {
	type Balance = Balance;
	type DustRemoval = Treasury;
	type Event = Event;
	type ExistentialDeposit = NativeTokenExistentialDeposit;
	type AccountStore = frame_system::Pallet<Runtime>;
	type MaxLocks = MaxLocks;
	type MaxReserves = MaxReserves;
	type ReserveIdentifier = ReserveIdentifier;
	type WeightInfo = ();
}

parameter_types! {
	pub TransactionByteFee: Balance = millicent(DNAR);
	/// The portion of the `NORMAL_DISPATCH_RATIO` that we adjust the fees with. Blocks filled less
	/// than this will decrease the weight and more will increase.
	pub const TargetBlockFullness: Perquintill = Perquintill::from_percent(25);
	/// The adjustment variable of the runtime. Higher values will cause `TargetBlockFullness` to
	/// change the fees more rapidly.
	pub AdjustmentVariable: Multiplier = Multiplier::saturating_from_rational(3, 100_000);
	/// Minimum amount of the multiplier. This value cannot be too low. A test case should ensure
	/// that combined with `AdjustmentVariable`, we can recover from the minimum.
	/// See `multiplier_can_grow_from_zero`.
	pub MinimumMultiplier: Multiplier = Multiplier::saturating_from_rational(1, 1_000_000_000u128);
}

impl pallet_sudo::Config for Runtime {
	type Event = Event;
	type Call = Call;
}

type EnsureRootOrAllGeneralCouncil = EnsureOneOf<
	AccountId,
	EnsureRoot<AccountId>,
	pallet_collective::EnsureProportionMoreThan<_1, _1, AccountId, GeneralCouncilInstance>,
>;

type EnsureRootOrAllSetheumJury = EnsureOneOf<
	AccountId,
	EnsureRoot<AccountId>,
	pallet_collective::EnsureProportionMoreThan<_1, _1, AccountId, SetheumJuryInstance>,
>;

type EnsureAllSetheumJuryOrAllGeneralCouncil = EnsureOneOf<
	AccountId,
	pallet_collective::EnsureProportionMoreThan<_1, _1, AccountId, SetheumJuryInstance>,
	pallet_collective::EnsureProportionMoreThan<_1, _1, AccountId, GeneralCouncilInstance>,
>;

type EnsureRootOrOneThirdSetheumJury = EnsureOneOf<
	AccountId,
	EnsureRoot<AccountId>,
	pallet_collective::EnsureProportionMoreThan<_1, _3, AccountId, SetheumJuryInstance>,
>;

type EnsureRootOrTwoThirdsSetheumJury = EnsureOneOf<
	AccountId,
	EnsureRoot<AccountId>,
	pallet_collective::EnsureProportionMoreThan<_2, _3, AccountId, SetheumJuryInstance>,
>;

type EnsureRootOrHalfSetheumJury = EnsureOneOf<
	AccountId,
	EnsureRoot<AccountId>,
	pallet_collective::EnsureProportionMoreThan<_1, _2, AccountId, SetheumJuryInstance>,
>;

type EnsureQuarterSetheumJuryOrHalfGeneralCouncil = EnsureOneOf<
	AccountId,
	pallet_collective::EnsureProportionMoreThan<_1, _4, AccountId, SetheumJuryInstance>,
	pallet_collective::EnsureProportionMoreThan<_1, _2, AccountId, GeneralCouncilInstance>,
>;

type EnsureOneThirdSetheumJuryOrHalfGeneralCouncil = EnsureOneOf<
	AccountId,
	pallet_collective::EnsureProportionMoreThan<_1, _3, AccountId, SetheumJuryInstance>,
	pallet_collective::EnsureProportionMoreThan<_1, _2, AccountId, GeneralCouncilInstance>,
>;

type EnsureHalfSetheumJuryOrHalfGeneralCouncil = EnsureOneOf<
	AccountId,
	pallet_collective::EnsureProportionMoreThan<_1, _2, AccountId, SetheumJuryInstance>,
	pallet_collective::EnsureProportionMoreThan<_1, _2, AccountId, GeneralCouncilInstance>,
>;

type EnsureRootOrHalfGeneralCouncil = EnsureOneOf<
	AccountId,
	EnsureRoot<AccountId>,
	pallet_collective::EnsureProportionMoreThan<_1, _2, AccountId, GeneralCouncilInstance>,
>;

type EnsureRootOrHalfFinancialCouncil = EnsureOneOf<
	AccountId,
	EnsureRoot<AccountId>,
	pallet_collective::EnsureProportionMoreThan<_1, _2, AccountId, FinancialCouncilInstance>,
>;

type EnsureHalfSetheumJuryOrHalfFinancialCouncil = EnsureOneOf<
	AccountId,
	pallet_collective::EnsureProportionMoreThan<_1, _2, AccountId, SetheumJuryInstance>,
	pallet_collective::EnsureProportionMoreThan<_1, _2, AccountId, FinancialCouncilInstance>,
>;

type EnsureRootOrTwoThirdsGeneralCouncil = EnsureOneOf<
	AccountId,
	EnsureRoot<AccountId>,
	pallet_collective::EnsureProportionMoreThan<_2, _3, AccountId, GeneralCouncilInstance>,
>;

type EnsureTwoThirdsSetheumJuryOrTwoThirdsGeneralCouncil = EnsureOneOf<
	AccountId,
	pallet_collective::EnsureProportionMoreThan<_2, _3, AccountId, SetheumJuryInstance>,
	pallet_collective::EnsureProportionMoreThan<_2, _3, AccountId, GeneralCouncilInstance>,
>;

type EnsureRootOrThreeFourthsGeneralCouncil = EnsureOneOf<
	AccountId,
	EnsureRoot<AccountId>,
	pallet_collective::EnsureProportionMoreThan<_3, _4, AccountId, GeneralCouncilInstance>,
>;

type EnsureRootOrThreeFourthsSetheumJury = EnsureOneOf<
	AccountId,
	EnsureRoot<AccountId>,
	pallet_collective::EnsureProportionMoreThan<_3, _4, AccountId, SetheumJuryInstance>,
>;

type EnsureThreeFourthsSetheumJuryOrThreeFourthsGeneralCouncil = EnsureOneOf<
	AccountId,
	pallet_collective::EnsureProportionMoreThan<_3, _4, AccountId, SetheumJuryInstance>,
	pallet_collective::EnsureProportionMoreThan<_3, _4, AccountId, GeneralCouncilInstance>,
>;

type EnsureRootOrOneThirdsTechnicalCommittee = EnsureOneOf<
	AccountId,
	EnsureRoot<AccountId>,
	pallet_collective::EnsureProportionMoreThan<_1, _3, AccountId, TechnicalCommitteeInstance>,
>;

type EnsureOneThirdsSetheumJuryOrOneThirdsTechnicalCommittee = EnsureOneOf<
	AccountId,
	pallet_collective::EnsureProportionMoreThan<_1, _3, AccountId, SetheumJuryInstance>,
	pallet_collective::EnsureProportionMoreThan<_1, _3, AccountId, TechnicalCommitteeInstance>,
>;

type EnsureTwoThirdsSetheumJuryOrTwoThirdsTechnicalCommittee = EnsureOneOf<
	AccountId,
	EnsureRoot<AccountId>,
	pallet_collective::EnsureProportionMoreThan<_2, _3, AccountId, TechnicalCommitteeInstance>,
>;

type EnsureRootOrTwoThirdsTechnicalCommittee = EnsureOneOf<
	AccountId,
	pallet_collective::EnsureProportionMoreThan<_2, _3, AccountId, SetheumJuryInstance>,
	pallet_collective::EnsureProportionMoreThan<_2, _3, AccountId, TechnicalCommitteeInstance>,
>;

parameter_types! {
	pub const GeneralCouncilMotionDuration: BlockNumber = 3 * DAYS;
	pub const GeneralCouncilMaxProposals: u32 = 50;
	pub const GeneralCouncilMaxMembers: u32 = 50;
}

type GeneralCouncilInstance = pallet_collective::Instance1;
impl pallet_collective::Config<GeneralCouncilInstance> for Runtime {
	type Origin = Origin;
	type Proposal = Call;
	type Event = Event;
	type MotionDuration = GeneralCouncilMotionDuration;
	type MaxProposals = GeneralCouncilMaxProposals;
	type MaxMembers = GeneralCouncilMaxMembers;
	type DefaultVote = pallet_collective::PrimeDefaultVote;
	type WeightInfo = ();
}

type GeneralCouncilMembershipInstance = pallet_membership::Instance1;
impl pallet_membership::Config<GeneralCouncilMembershipInstance> for Runtime {
	type Event = Event;
	type AddOrigin = EnsureRootOrThreeFourthsGeneralCouncil;
	type RemoveOrigin = EnsureRootOrThreeFourthsGeneralCouncil;
	type SwapOrigin = EnsureRootOrThreeFourthsGeneralCouncil;
	type ResetOrigin = EnsureRootOrThreeFourthsGeneralCouncil;
	type PrimeOrigin = EnsureRootOrThreeFourthsGeneralCouncil;
	type MembershipInitialized = GeneralCouncil;
	type MembershipChanged = GeneralCouncil;
	type MaxMembers = GeneralCouncilMaxMembers;
	type WeightInfo = ();
}

parameter_types! {
	pub const SetheumJuryMotionDuration: BlockNumber = 3 * DAYS;
	pub const SetheumJuryMaxProposals: u32 = 50;
	pub const SetheumJuryMaxMembers: u32 = 50;
}

type SetheumJuryInstance = pallet_collective::Instance1;
impl pallet_collective::Config<SetheumJuryInstance> for Runtime {
	type Origin = Origin;
	type Proposal = Call;
	type Event = Event;
	type MotionDuration = SetheumJuryMotionDuration;
	type MaxProposals = SetheumJuryMaxProposals;
	type MaxMembers = SetheumJuryMaxMembers;
	type DefaultVote = pallet_collective::PrimeDefaultVote;
	type WeightInfo = ();
}

type SetheumJuryMembershipInstance = pallet_membership::Instance1;
impl pallet_membership::Config<SetheumJuryMembershipInstance> for Runtime {
	type Event = Event;
	type AddOrigin = EnsureRootOrThreeFourthsSetheumJury;
	type RemoveOrigin = EnsureRootOrThreeFourthsSetheumJury;
	type SwapOrigin = EnsureRootOrThreeFourthsSetheumJury;
	type ResetOrigin = EnsureRootOrThreeFourthsSetheumJury;
	type PrimeOrigin = EnsureRootOrThreeFourthsSetheumJury;
	type MembershipInitialized = SetheumJury;
	type MembershipChanged = SetheumJury;
	type MaxMembers = SetheumJuryMaxMembers;
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
	type AddOrigin = EnsureRootOrTwoThirdsGeneralCouncil;
	type RemoveOrigin = EnsureRootOrTwoThirdsGeneralCouncil;
	type SwapOrigin = EnsureRootOrTwoThirdsGeneralCouncil;
	type ResetOrigin = EnsureRootOrTwoThirdsGeneralCouncil;
	type PrimeOrigin = EnsureRootOrTwoThirdsGeneralCouncil;
	type MembershipInitialized = FinancialCouncil;
	type MembershipChanged = FinancialCouncil;
	type MaxMembers = FinancialCouncilMaxMembers;
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
	type AddOrigin = EnsureRootOrTwoThirdsGeneralCouncil;
	type RemoveOrigin = EnsureRootOrTwoThirdsGeneralCouncil;
	type SwapOrigin = EnsureRootOrTwoThirdsGeneralCouncil;
	type ResetOrigin = EnsureRootOrTwoThirdsGeneralCouncil;
	type PrimeOrigin = EnsureRootOrTwoThirdsGeneralCouncil;
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
	type AddOrigin = EnsureRootOrTwoThirdsGeneralCouncil;
	type RemoveOrigin = EnsureRootOrTwoThirdsGeneralCouncil;
	type SwapOrigin = EnsureRootOrTwoThirdsGeneralCouncil;
	type ResetOrigin = EnsureRootOrTwoThirdsGeneralCouncil;
	type PrimeOrigin = EnsureRootOrTwoThirdsGeneralCouncil;
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

pub struct GeneralCouncilProvider;
impl Contains<AccountId> for GeneralCouncilProvider {
	fn contains(who: &AccountId) -> bool {
		GeneralCouncil::is_member(who)
	}

	fn sorted_members() -> Vec<AccountId> {
		GeneralCouncil::members()
	}

	#[cfg(feature = "runtime-benchmarks")]
	fn add(_: &AccountId) {
		todo!()
	}
}

impl ContainsLengthBound for GeneralCouncilProvider {
	fn max_len() -> usize {
		100
	}
	fn min_len() -> usize {
		0
	}
}

parameter_types! {
	pub const ProposalBond: Permill = Permill::from_percent(5);
	pub ProposalBondMinimum: Balance = 2 * dollar(DNAR);
	pub const SpendPeriod: BlockNumber = 7 * DAYS;
	pub const Burn: Permill = Permill::from_percent(0);

	pub const TipCountdown: BlockNumber = DAYS;
	pub const TipFindersFee: Percent = Percent::from_percent(10);
	pub TipReportDepositBase: Balance = deposit(1, 0);
	pub BountyDepositBase: Balance = deposit(1, 0);
	pub const BountyDepositPayoutDelay: BlockNumber = 3 * DAYS;
	pub const BountyUpdatePeriod: BlockNumber = 30 * DAYS;
	pub const BountyCuratorDeposit: Permill = Permill::from_percent(50);
	pub BountyValueMinimum: Balance = 5 * dollar(DNAR);
	pub DataDepositPerByte: Balance = deposit(0, 1);
	pub const MaximumReasonLength: u32 = 16384;
	pub const MaxApprovals: u32 = 100;
}

impl pallet_treasury::Config for Runtime {
	type PalletId = TreasuryPalletId;
	type Currency = Balances;
	type ApproveOrigin = EnsureRootOrHalfGeneralCouncil;
	type RejectOrigin = EnsureRootOrHalfGeneralCouncil; // TODO: When root is removed, change to `EnsureHalfSetheumJuryOrHalfGeneralCouncil`.
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

// TODO: Update to `setheum-staking` and it's allied implementations
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
	type SessionHandler = <SessionKeys as OpaqueKeys>::KeyTypeIdProviders;
	type Keys = SessionKeys;
	type DisabledValidatorsThreshold = DisabledValidatorsThreshold;
	type WeightInfo = ();
}

impl pallet_session::historical::Config for Runtime {
	type FullIdentification = pallet_staking::Exposure<AccountId, Balance>;
	type FullIdentificationOf = pallet_staking::ExposureOf<Runtime>;
}

pallet_staking_reward_curve::build! {
	const REWARD_CURVE: PiecewiseLinear<'static> = curve!(
		min_inflation: 0_025_000,
		max_inflation: 0_100_000,
		ideal_stake: 0_500_000,
		falloff: 0_050_000,
		max_piece_count: 40,
		test_precision: 0_005_000,
	);
}

parameter_types! {
	pub const SessionsPerEra: sp_staking::SessionIndex = 3; // 3 hours
	pub const BondingDuration: pallet_staking::EraIndex = 0; // No bonding duration. can actively immediate bond/unbond anytime
	pub const SlashDeferDuration: pallet_staking::EraIndex = 2; // 6 hours
	pub const RewardCurve: &'static PiecewiseLinear<'static> = &REWARD_CURVE;
	pub const MaxNominatorRewardedPerValidator: u32 = 64;
	pub const ElectionLookahead: BlockNumber = EPOCH_DURATION_IN_BLOCKS / 4;
	pub const MaxIterations: u32 = 5;
	// 0.05%. The higher the value, the more strict solution acceptance becomes.
	pub MinSolutionScoreBump: Perbill = Perbill::from_rational_approximation(5u32, 10_000);
}

impl pallet_staking::Config for Runtime {
	type Currency = Balances;
	type UnixTime = Timestamp;
	type CurrencyToVote = U128CurrencyToVote;
	type RewardRemainder = Treasury;
	type Event = Event;
	type Slash = Treasury; // send the slashed funds to the pallet treasury.
	type Reward = (); // rewards are minted from the void
	type SessionsPerEra = SessionsPerEra;
	type BondingDuration = BondingDuration;
	type SlashDeferDuration = SlashDeferDuration;
	/// A super-majority of the council can cancel the slash.
	type SlashCancelOrigin = EnsureRootOrThreeFourthsGeneralCouncil;
	type SessionInterface = Self;
	type RewardCurve = RewardCurve;
	type NextNewSession = Session;
	type ElectionLookahead = ElectionLookahead;
	type Call = Call;
	type MaxIterations = MaxIterations;
	type MinSolutionScoreBump = MinSolutionScoreBump;
	type MaxNominatorRewardedPerValidator = MaxNominatorRewardedPerValidator;
	type UnsignedPriority = runtime_common::StakingUnsignedPriority;
	type WeightInfo = ();
	type OffchainSolutionWeightLimit = OffchainSolutionWeightLimit;
}

parameter_types! {
	pub ConfigDepositBase: Balance = 10 * cent(DNAR);
	pub FriendDepositFactor: Balance = cent(DNAR);
	pub const MaxFriends: u16 = 9;
	pub RecoveryDeposit: Balance = 10 * cent(DNAR);
}

impl pallet_recovery::Config for Runtime {
	type Event = Event;
	type Call = Call;
	type Currency = Balances;
	type ConfigDepositBase = ConfigDepositBase;
	type FriendDepositFactor = FriendDepositFactor;
	type MaxFriends = MaxFriends;
	type RecoveryDeposit = RecoveryDeposit;
}

parameter_types! {
	pub const LaunchPeriod: BlockNumber = 28 * DAYS;
	pub const VotingPeriod: BlockNumber = 28 * DAYS;
	pub const FastTrackVotingPeriod: BlockNumber = 3 * HOURS;
	pub MinimumDeposit: Balance = 100 * dollar(DNAR);
	pub const EnactmentPeriod: BlockNumber = 28 * DAYS;
	pub const CooloffPeriod: BlockNumber = 7 * DAYS;
	pub PreimageByteDeposit: Balance = cent(DNAR);
	pub const InstantAllowed: bool = true;
	pub const MaxVotes: u32 = 100;
	pub const MaxProposals: u32 = 100;
}

impl pallet_democracy::Config for Runtime {
	type Proposal = Call;
	type Event = Event;
	type Currency = Balances;
	type EnactmentPeriod = EnactmentPeriod;
	type LaunchPeriod = LaunchPeriod;
	type VotingPeriod = VotingPeriod;
	type MinimumDeposit = MinimumDeposit;
	/// A straight majority of the council can decide what their next motion is.
	type ExternalOrigin = EnsureRootOrHalfGeneralCouncil;
	/// A majority can have the next scheduled referendum be a straight majority-carries vote.
	type ExternalMajorityOrigin = EnsureRootOrHalfGeneralCouncil;
	/// A unanimous council can have the next scheduled referendum be a straight default-carries
	/// (NTB) vote.
	type ExternalDefaultOrigin = EnsureRootOrAllGeneralCouncilOrAllSetheumJury;
	/// Two thirds of the technical committee can have an ExternalMajority/ExternalDefault vote
	/// be tabled immediately and with a shorter voting/enactment period.
	type FastTrackOrigin = EnsureRootOrTwoThirdsTechnicalCommittee;
	type InstantOrigin = EnsureRootOrAllTechnicalCommittee;
	type InstantAllowed = InstantAllowed;
	type FastTrackVotingPeriod = FastTrackVotingPeriod;
	// To cancel a proposal which has been passed, 2/3 of the council must agree to it.
	type CancellationOrigin = EnsureRootOrTwoThirdsGeneralCouncil; // TODO: When root is removed, change to `EnsureHalfSetheumJuryOrTwoThirdsGeneralCouncil`.
	type BlacklistOrigin = EnsureRootOrHalfSetheumJury;
	// To cancel a proposal before it has been passed, the technical committee must be unanimous or
	// Root must agree or half of SetheumJury must agree.
	type CancelProposalOrigin = EnsureRootOrAllTechnicalCommittee; // TODO: When root is removed, change to `EnsureHalfSetheumJuryOrAllTechnicalCommittee`.
	// Any single technical committee member may veto a coming council proposal, however they can
	// only do it once and it lasts only for the cooloff period.
	type VetoOrigin = pallet_collective::EnsureMember<AccountId, SetheumJuryInstance> | EnsureMember<AccountId, TechnicalCommitteeInstance>;
	type CooloffPeriod = CooloffPeriod;
	type PreimageByteDeposit = PreimageByteDeposit;
	type OperationalPreimageOrigin = pallet_collective::EnsureMember<AccountId, GeneralCouncilInstance>;
	type Slash = Treasury;
	type Scheduler = Scheduler;
	type PalletsOrigin = OriginCaller;
	type MaxVotes = MaxVotes;
	// TODO: might need to custom-weight for Setheum
	type WeightInfo = pallet_democracy::weights::SubstrateWeight<Runtime>;
	type MaxProposals = MaxProposals;
}

impl orml_auction::Config for Runtime {
	type Event = Event;
	type Balance = Balance;
	type AuctionId = AuctionId;
	type Handler = SerpAuctionManager;
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
	pub const MinimumCount: u32 = 1;
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
	type WeightInfo = weights::orml_oracle::WeightInfo<Runtime>;
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

parameter_type_with_key! {
	pub ExistentialDeposits: |_currency_id: CurrencyId| -> Balance {
		match currency_id {
			CurrencyId::Token(symbol) => match symbol {
				TokenSymbol::DNAR => Balance::max_value(), // unsupported
				TokenSymbol::SDEX => Balance::max_value(*currency_id), // unsupported
				
				TokenSymbol::SETT => cent(*currency_id),
				TokenSymbol::USDJ => cent(*currency_id),
				TokenSymbol::EURJ => cent(*currency_id),
				TokenSymbol::JPYJ => cent(*currency_id),
				TokenSymbol::GBPJ => cent(*currency_id),
				TokenSymbol::AUDJ => cent(*currency_id),
				TokenSymbol::CADJ => cent(*currency_id),
				TokenSymbol::CHFJ => cent(*currency_id),
				TokenSymbol::SGDJ => cent(*currency_id),
				TokenSymbol::BRLJ => cent(*currency_id),
				TokenSymbol::SARJ => cent(*currency_id)

				TokenSymbol::RENBTC |
			},
			CurrencyId::DexShare(_, _) => {
				let dec = <EvmCurrencyIdMapping<Runtime> as CurrencyIdMapping>::decimals(*currency_id);
				if let Some(dec) = dec {
					// TODO: verify if this makes sense
					10u128.saturating_pow(dec as u32)
				} else {
					// TODO: update this before we enable ERC20 in DEX
					Balance::max_value() // unsupported
				}
			},
			CurrencyId::Erc20(_) => Balance::max_value(), // not handled by orml-tokens
			CurrencyId::ChainBridge(_) => Balance::max_value(), // TODO: update this before we enable ChainBridge
		}
	};
}

parameter_types! {
	pub TreasuryAccount: AccountId = TreasuryPalletId::get().into_account();
}

impl orml_tokens::Config for Runtime {
	type Event = Event;
	type Balance = Balance;
	type Amount = Amount;
	type CurrencyId = CurrencyId;
	type WeightInfo = weights::orml_tokens::WeightInfo<Runtime>;
	type ExistentialDeposits = ExistentialDeposits;
	type OnDust = orml_tokens::TransferDust<Runtime, TreasuryAccount>;
}

parameter_types! {
	pub PegCurrencyIds: |_currency_id: CurrencyId| -> CurrencyId {
		match currency_id {
			&USDJ => &USD,
			&EURJ => &EUR,
			&JPYJ => &JPY,
			&GBPJ => &GBP,
			&AUDJ => &AUD,
			&CADJ => &CAD,
			&CHFJ => &CHF,
			&SGDJ => &SGD,
			&BRLJ => &BRL,
			&SARJ => &SAR,
			_ => None,
		}
	};

	pub FiatUsdFixedPrice: Price = Price::saturating_from_rational(1, 1);

	pub const GetSetterPegOneCurrencyId: CurrencyId = GBP; // Fiat pegs of the Setter (SETT).
	pub const GetSetterPegTwoCurrencyId: CurrencyId = EUR; // Fiat pegs of the Setter (SETT).
	pub const GetSetterPegThreeCurrencyId: CurrencyId = KWD; // Fiat pegs of the Setter (SETT).
	pub const GetSetterPegFourCurrencyId: CurrencyId = JOD; // Fiat pegs of the Setter (SETT).
	pub const GetSetterPegFiveCurrencyId: CurrencyId = BHD; // Fiat pegs of the Setter (SETT).
	pub const GetSetterPegSixCurrencyId: CurrencyId = KYD; // Fiat pegs of the Setter (SETT).
	pub const GetSetterPegSevenCurrencyId: CurrencyId = OMR; // Fiat pegs of the Setter (SETT).
	pub const GetSetterPegEightCurrencyId: CurrencyId = CHF; // Fiat pegs of the Setter (SETT).
	pub const GetSetterPegNineCurrencyId: CurrencyId = GIP; // Fiat pegs of the Setter (SETT).
	pub const GetSetterPegTenCurrencyId: CurrencyId = USD; // Fiat pegs of the Setter (SETT).
	
	pub StableCurrencyIds: Vec<CurrencyId> = vec![
		SETT, USDJ, EURJ, JPYJ, GBPJ, AUDJ, CADJ, CHFJ, SGDJ, BRLJ, SARJ
	];
	pub FiatCurrencyIds: Vec<CurrencyId> = vec![
		USD, EUR, JPY, GBP, AUD, CAD, CHF, SGD, BRL, SAR, KWD, JOD, BHD, KYD, OMR, GIP
	];
}

impl setheum_prices::Config for Runtime {
	type Event = Event;
	type Source = AggregatedDataProvider;
	type GetSetterCurrencyId = GetSetterCurrencyId;
	type GetSettUSDCurrencyId = GetSettUSDCurrencyId;
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
	type StableCurrencyIds = StableCurrencyIds;
	type PegCurrencyIds = PegCurrencyIds;
	type FiatCurrencyIds = FiatCurrencyIds;
	type LockOrigin = EnsureRootOrTwoThirdsGeneralCouncil;
	type Dex = Dex;
	type Currency = Currencies;
	type WeightInfo = weights::setheum_prices::WeightInfo<Runtime>;
}

// TODO: Remove Other incentives except for DexIncentive 
// TODO - and update DexPremium to the new Incentive model
// TODO - based on trading volume, this new method will be 
// TODO - implemented from `setheum_dex` module.
parameter_types! {
	pub const GetNativeCurrencyId: CurrencyId = DNAR;
	pub const GetSetterCurrencyId: CurrencyId = SETT;
	pub const GetDexerCurrencyId: CurrencyId = SDEX;
	pub const GetSettUSDCurrencyId: CurrencyId = USDJ;
	pub const GetFiatUSDCurrencyId: CurrencyId = USD;
	pub const GetIncentiveCurrencyId: CurrencyId = SDEX;
	pub const GetPremiumCurrencyId: CurrencyId = SETT; // TODO: Update
	pub const GetPlusCurrencyId: CurrencyId = SETT; // TODO: Update and remove
	pub const GetBonusCurrencyId: CurrencyId = USDJ; // TODO: Update and remove
	pub const GetExtraCurrencyId: CurrencyId = EURJ; // TODO: Update and remove
	pub const GetDexCurrencyId: CurrencyId = SDEX; // TODO: Update and remove
}

impl setheum_currencies::Config for Runtime {
	type Event = Event;
	type MultiCurrency = Tokens;
	type NativeCurrency = BasicCurrencyAdapter<Runtime, Balances, Amount, BlockNumber>;
	type GetNativeCurrencyId = GetNativeCurrencyId;
	type WeightInfo = weights::setheum_currencies::WeightInfo<Runtime>;
	type AddressMapping = EvmAddressMapping<Runtime>;
	type EVMBridge = EVMBridge;
}

parameter_types! {
	pub SetheumFoundationAccounts: Vec<AccountId> = vec![
		// Charity Fund Account : "5DhvNsZdYTtWUYdHvREWhsHWt1StP9bA21vsC1Wp6UksjNAh"
		hex_literal::hex!["0x489e7647f3a94725e0178fc1da16ef671175837089ebe83e6d1f0a5c8b682e56"].into(),	// "5DhvNsZdYTtWUYdHvREWhsHWt1StP9bA21vsC1Wp6UksjNAh"
		// TODO: Add second foundation account `hex_literal::hex!["0x489e7647f3a94725e0178fc1da16ef671175837089ebe83e6d1f0a5c8b682e56"].into(),	// "5DhvNsZdYTtWUYdHvREWhsHWt1StP9bA21vsC1Wp6UksjNAh"`
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

parameter_types! {
	pub MinVestedTransfer: Balance = 0;
	pub const MaxVestingSchedules: u32 = 258;
}

impl orml_vesting::Config for Runtime {
	type Event = Event;
	type Currency = pallet_balances::Pallet<Runtime>;
	type MinVestedTransfer = MinVestedTransfer;
	type VestedTransferOrigin = EnsureSetheumFoundation;
	type WeightInfo = weights::orml_vesting::WeightInfo<Runtime>;
	type MaxVestingSchedules = MaxVestingSchedules;
	type BlockNumberProvider = RelaychainBlockNumberProvider<Runtime>;
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

parameter_types! {
	pub DiamondAuctionMinimumIncrementSize: Rate = Rate::saturating_from_rational(3 : 100); // 3% increment
	pub SetterAuctionMinimumIncrementSize: Rate = Rate::saturating_from_rational(2 : 100); // 2% increment
	pub SerplusAuctionMinimumIncrementSize: Rate = Rate::saturating_from_rational(1, 100); // 1% increment
	pub const AuctionTimeToClose: BlockNumber = 15 * MINUTES;
	pub const AuctionDurationSoftCap: BlockNumber = 2 * HOURS;
}

impl serp_auction::Config for Runtime {
	type Event = Event;
	type Currency = Currencies;
	type Auction = Auction;
	type DiamondAuctionMinimumIncrementSize = DiamondAuctionMinimumIncrementSize;
	type SetterAuctionMinimumIncrementSize = SetterAuctionMinimumIncrementSize;
	type SerplusAuctionMinimumIncrementSize = SerplusAuctionMinimumIncrementSize;
	type AuctionTimeToClose = AuctionTimeToClose;
	type AuctionDurationSoftCap = AuctionDurationSoftCap;
	type StableCurrencyIds = StableCurrencyIds;
	type GetNativeCurrencyId = GetNativeCurrencyId;
	type GetSetterCurrencyId = GetSetterCurrencyId;
	type SerpTreasury = SerpTreasury;
	type Dex = Dex;
	type PriceSource = Prices;
	type UnsignedPriority = runtime_common::SerpAuctionUnsignedPriority;
	type WeightInfo = weights::serp_auction::WeightInfo<Runtime>;
}

parameter_types! {
	pub StandardCurrencyIds: Vec<CurrencyId> = vec![
		USDJ, EURJ, JPYJ, GBPJ, AUDJ, CADJ, CHFJ, SGDJ, BRLJ, SARJ
	];
	pub const GetReserveCurrencyId: CurrencyId = SETT;
}
impl settmint_manager::Config for Runtime {
	type Event = Event;
	type Convert = settmint_engine::StandardExchangeRateConvertor<Runtime>;
	type Currency = Currencies;
	type StandardCurrencyIds = StandardCurrencyIds;
	type GetReserveCurrencyId = GetReserveCurrencyId;
	type StandardValidator = SettmintEngine;
	type SerpTreasury = SerpTreasury;
	type PalletId = SettmintManagerPalletId;
	type OnUpdateSettMint = setheum_incentives::OnUpdateSettMint<Runtime>;
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
			setheum_transaction_payment::ChargeTransactionPayment::<Runtime>::from(tip),
			setheum_evm::SetEvmOrigin::<Runtime>::new(),
		);
		let raw_payload = SignedPayload::new(call, extra)
			.map_err(|e| {
				log::warn!("Unable to create signed payload: {:?}", e);
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

parameter_types! {
	pub GetReserveCurrencyId: CurrencyId = SETT;
	pub DefaultStandardExchangeRate: ExchangeRate = ExchangeRate::saturating_from_rational(1, 10);
	pub MinimumStandardValue: Balance = dollar(SETT);
}

impl settmint_engine::Config for Runtime {
	type Event = Event;
	type PriceSource = Prices;
	type GetReserveCurrencyId = GetReserveCurrencyId;
	type StandardCurrencyIds = StandardCurrencyIds;
	type DefaultStandardExchangeRate = DefaultStandardExchangeRate;
	type MinimumStandardValue = MinimumStandardValue;
	type SerpTreasury = SerpTreasury;
	type UpdateOrigin = EnsureRootOrHalfFinancialCouncil;
	type Dex = Dex;
	type UnsignedPriority = runtime_common::SettmintEngineUnsignedPriority;
	type WeightInfo = weights::settmint_engine::WeightInfo<Runtime>;
}

parameter_types! {
	pub DepositPerAuthorization: Balance = deposit(1, 64);
}

impl settmint_gateway::Config for Runtime {
	type Event = Event;
	type Currency = Balances;
	type DepositPerAuthorization = DepositPerAuthorization;
	type WeightInfo = weights::settmint_gateway::WeightInfo<Runtime>;
}

parameter_types! {
	// TODO: Update `GetExchangeFee` to get from storage map and let-vvv
	// TODO: --- ^^^ - the Exchange Council update it through GOVERNANCE
	pub const GetExchangeFee: (u32, u32) = (1, 1000);	// 0.1%
	pub const TradingPathLimit: u32 = 3;
	pub EnabledTradingPairs: Vec<TradingPair> = vec![
		TradingPair::new(SETT, DNAR),
		TradingPair::new(SETT, SDEX),

		TradingPair::new(SETT, USDJ),
		TradingPair::new(SETT, EURJ),
		TradingPair::new(SETT, JPYJ),
		TradingPair::new(SETT, GBPJ),
		TradingPair::new(SETT, AUDJ),
		TradingPair::new(SETT, CADJ),
		TradingPair::new(SETT, CHFJ),
		TradingPair::new(SETT, SGDJ),
		TradingPair::new(SETT, BRLJ),
		TradingPair::new(SETT, SARJ),
		
		TradingPair::new(SETT, RENBTC),
	];
}

impl dex::Config for Runtime {
	type Event = Event;
	type Currency = Currencies;
	type GetExchangeFee = GetExchangeFee;
	type TradingPathLimit = TradingPathLimit;
	type PalletId = DexPalletId;
	type CurrencyIdMapping = EvmCurrencyIdMapping<Runtime>;
	type DEXIncentives = Incentives;
	type WeightInfo = weights::dex::WeightInfo<Runtime>;
	type ListingOrigin = EnsureRootOrHalfGeneralCouncil; // TODO: When root is removed, change to `EnsureHalfSetheumJuryOrHalfGeneralCouncil`.
}

parameter_types! {
	pub const MaxAuctionsCount: u32 = 100;
	// Charity Fund Account : "5DhvNsZdYTtWUYdHvREWhsHWt1StP9bA21vsC1Wp6UksjNAh"
	pub const CharityFundAccount: AccountId = hex!["0x489e7647f3a94725e0178fc1da16ef671175837089ebe83e6d1f0a5c8b682e56"].into();

	pub SettPayTreasuryAccount: AccountId = SettPayTreasuryPalletId::get().into_account()
	// TODO: Update SerpTesSchedule to an updatable param in the storage map, under financial council
	pub SerpTesSchedule: BlockNumber = 12 * MINUTES; // Triggers SERP-TES for serping Every 12 minutes.
	pub SerplusSerpupRatio: Permill = Permill::from_percent(10); // 10% of SerpUp to buy back & burn NativeCurrency.
	pub SettPaySerpupRatio: Permill = Permill::from_percent(60); // 60% of SerpUp to SettPay as Cashdrops.
	pub TreasurySerpupRatio: Permill = Permill::from_percent(10); // 10% of SerpUp to network Treasury.
	pub CharityFundSerpupRatio: Permill = Permill::from_percent(20); // 20% of SerpUp to Setheum Foundation's Charity Fund.
}

impl serp_treasury::Config for Runtime {
	type Event = Event;
	type Currency = Currencies;
	type StableCurrencyIds = StableCurrencyIds;
	type GetSetterCurrencyId = GetSetterCurrencyId;
	type GetDexerCurrencyId = GetDexerCurrencyId;
	type SerpTesSchedule = SerpTesSchedule;
	type SerplusSerpupRatio = SerplusSerpupRatio;
	type SettPaySerpupRatio = SettPaySerpupRatio;
	type TreasurySerpupRatio = TreasurySerpupRatio;
	type CharityFundSerpupRatio = CharityFundSerpupRatio;
	type SettPayTreasuryAcc = SettPayTreasuryAccount;
	type TreasuryAcc = TreasuryAccount;
	type CharityFundAcc = CharityFundAccount;
	type SerpAuctionManagerHandler = MockSerpAuctionManager;
	type UpdateOrigin = EnsureSignedBy<One, AccountId>;
	type Dex = SetheumDEX;
	type MaxAuctionsCount = MaxAuctionsCount;
	type PalletId = SerpTreasuryPalletId;
	type WeightInfo = weights::serp_treasury::WeightInfo<Runtime>;
}

parameter_types! {
	// All currency types except for native currency, Sort by fee charge order
	pub AllNonNativeCurrencyIds: Vec<CurrencyId> = vec![
		SDEX, SETT, USDJ, EURJ, JPYJ, GBPJ, AUDJ, CADJ, CHFJ, SGDJ, BRLJ, SARJ, RENBTC
	];
	pub MaxSlippageSwapWithDex: Ratio = Ratio::saturating_from_rational(5, 100);
}

impl setheum_transaction_payment::Config for Runtime {
	type AllNonNativeCurrencyIds = AllNonNativeCurrencyIds;
	type NativeCurrencyId = GetNativeCurrencyId;
	type GetSetterCurrencyId = GetSetterCurrencyId;
	type Currency = Balances;
	type MultiCurrency = Currencies;
	type OnTransactionPayment = Treasury;
	type TransactionByteFee = TransactionByteFee;
	type WeightToFee = WeightToFee;
	type FeeMultiplierUpdate = TargetedFeeAdjustment<Self, TargetBlockFullness, AdjustmentVariable, MinimumMultiplier>;
	type Dex = Dex;
	type MaxSlippageSwapWithDex = MaxSlippageSwapWithDex;
	type WeightInfo = weights::setheum_transaction_payment::WeightInfo<Runtime>;
}

pub struct EvmAccountsOnClaimHandler;
impl Handler<AccountId> for EvmAccountsOnClaimHandler {
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

impl setheum_evm_accounts::Config for Runtime {
	type Event = Event;
	type Currency = Balances;
	type AddressMapping = EvmAddressMapping<Runtime>;
	type TransferAll = Currencies;
	type OnClaim = EvmAccountsOnClaimHandler;
	type WeightInfo = weights::setheum_evm_accounts::WeightInfo<Runtime>;
}

impl setheum_evm_manager::Config for Runtime {
	type Currency = Balances;
	type EVMBridge = EVMBridge;
}

impl orml_rewards::Config for Runtime {
	type Share = Balance;
	type Balance = Balance;
	type PoolId = setheum_incentives::PoolId<AccountId>;
	type Handler = Incentives;
}

// TODO: Remove Other incentives except for DexIncentive 
// TODO - and update ExchangeFeeWaiver (DexFeeWaiverIncentive) to the new Incentive model
// TODO - based on trading volume, this new method will be 
// TODO - implemented from `setheum_dex` module.
parameter_types! {
	// TODO: Remove Other incentives except for DexIncentive 
	// TODO - and update ExchangeFeeWaiver (DexFeeWaiverIncentive) to the new Incentive model
	// TODO - based on trading volume, this new method will be 
	// TODO - implemented from `setheum_dex` module.
	pub const AccumulatePeriod: BlockNumber = MINUTES;
}

impl setheum_incentives::Config for Runtime {
	type Event = Event;
	type DexIncentivePool = ZeroAccountId;
	type DexPremiumPool = ZeroAccountId;
	type DexPlusPool = ZeroAccountId;
	type DexBonusPool = ZeroAccountId;
	type DexExtraPool = ZeroAccountId;
	type AccumulatePeriod = AccumulatePeriod;
	type IncentiveCurrencyId = GetIncentiveCurrencyId;
	type PremiumCurrencyId = GetPremiumCurrencyId;
	type PlusCurrencyId = GetPlusCurrencyId;
	type BonusCurrencyId = GetBonusCurrencyId
	type DexCurrencyId = GetDexCurrencyId;
	type ExtraCurrencyId = GetExtraCurrencyId
	type NativeCurrencyId = GetNativeCurrencyId;
	type StableCurrencyIds = StableCurrencyIds;
	type UpdateOrigin = EnsureRootOrHalfFinancialCouncil;
	type SerpTreasury = SerpTreasury;
	type Currency = Currencies;
	type Dex = Dex;
	type PalletId = IncentivesPalletId;
	type WeightInfo = weights::setheum_incentives::WeightInfo<Runtime>;
}

parameter_types! {
	pub CreateClassDeposit: Balance = 20 * millicent(DNAR);
	pub CreateTokenDeposit: Balance = 2 * millicent(DNAR);
}

impl setheum_nft::Config for Runtime {
	type Event = Event;
	type CreateClassDeposit = CreateClassDeposit;
	type CreateTokenDeposit = CreateTokenDeposit;
	type PalletId = NftPalletId;
	type WeightInfo = weights::setheum_nft::WeightInfo<Runtime>;
}

parameter_types! {
	pub MaxClassMetadata: u32 = 1024;
	pub MaxTokenMetadata: u32 = 1024;
}

impl orml_nft::Config for Runtime {
	type ClassId = u32;
	type TokenId = u64;
	type ClassData = setheum_nft::ClassData<Balance>;
	type TokenData = setheum_nft::TokenData<Balance>;
	type MaxClassMetadata = MaxClassMetadata;
	type MaxTokenMetadata = MaxTokenMetadata;
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

impl pallet_proxy::Config for Runtime {
	type Event = Event;
	type Call = Call;
	type Currency = Balances;
	type ProxyType = ();
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
	pub const RENBTCCurrencyId: CurrencyId = RENBTC;
	pub const RENBTCIdentifier: [u8; 32] = hex!["f6b5b360905f856404bd4cf39021b82209908faa44159e68ea207ab8a5e13197"];
}

impl setheum_renvm_bridge::Config for Runtime {
	type Event = Event;
	type Currency = Balances;
	type BridgedTokenCurrency = Currency<Runtime, RENBTCCurrencyId>;
	type CurrencyIdentifier = RENBTCIdentifier;
	type UnsignedPriority = runtime_common::RenvmBridgeUnsignedPriority;
	type ChargeTransactionPayment = setheum_transaction_payment::ChargeTransactionPayment<Runtime>;
}

parameter_types! {
	// TODO: update
	pub const ChainId: u64 = 258;
	pub const NewContractExtraBytes: u32 = 10_000;
	pub StorageDepositPerByte: Balance = deposit(0, 1);
	// https://eips.ethereum.org/EIPS/eip-170
	pub const MaxCodeSize: u32 = 0x6000;
	pub NetworkContractSource: H160 = H160::from_low_u64_be(0);
	pub DeveloperDeposit: Balance = 100 * dollar(DNAR);
	pub DeploymentFee: Balance = 10000 * dollar(DNAR);
}

pub type MultiCurrencyPrecompile = runtime_common::MultiCurrencyPrecompile<
	AccountId,
	EvmAddressMapping<Runtime>,
	EvmCurrencyIdMapping<Runtime>,
	Currencies,
>;

pub type NFTPrecompile =
	runtime_common::NFTPrecompile<AccountId, EvmAddressMapping<Runtime>, EvmCurrencyIdMapping<Runtime>, NFT>;
pub type StateRentPrecompile =
	runtime_common::StateRentPrecompile<AccountId, EvmAddressMapping<Runtime>, EvmCurrencyIdMapping<Runtime>, EVM>;
pub type OraclePrecompile =
	runtime_common::OraclePrecompile<AccountId, EvmAddressMapping<Runtime>, EvmCurrencyIdMapping<Runtime>, Prices>;
pub type ScheduleCallPrecompile = runtime_common::ScheduleCallPrecompile<
	AccountId,
	EvmAddressMapping<Runtime>,
	EvmCurrencyIdMapping<Runtime>,
	Scheduler,
	setheum_transaction_payment::ChargeTransactionPayment<Runtime>,
	Call,
	Origin,
	OriginCaller,
	Runtime,
>;

pub type DexPrecompile =
	runtime_common::DexPrecompile<AccountId, EvmAddressMapping<Runtime>, EvmCurrencyIdMapping<Runtime>, Dex>;

impl setheum_evm::Config for Runtime {
	type AddressMapping = EvmAddressMapping<Runtime>;
	type Currency = Balances;
	type TransferAll = Currencies;
	type NewContractExtraBytes = NewContractExtraBytes;
	type StorageDepositPerByte = StorageDepositPerByte;
	type MaxCodeSize = MaxCodeSize;
	type Event = Event;
	type Precompiles = runtime_common::AllPrecompiles<
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
	type ChargeTransactionPayment = setheum_transaction_payment::ChargeTransactionPayment<Runtime>;
	type NetworkContractOrigin = EnsureRootOrTwoThirdsTechnicalCommittee;
	type NetworkContractSource = NetworkContractSource;
	type DeveloperDeposit = DeveloperDeposit;
	type DeploymentFee = DeploymentFee;
	type TreasuryAccount = TreasuryAccount;
	type FreeDeploymentOrigin = EnsureRootOrHalfGeneralCouncil; // TODO: When root is removed, change to `EnsureHalfSetheumJuryOrHalfGeneralCouncil`.
	type WeightInfo = weights::setheum_evm::WeightInfo<Runtime>;
}

impl setheum_evm_bridge::Config for Runtime {
	type EVM = EVM;
}

#[allow(clippy::large_enum_variant)]
construct_runtime!(
	pub enum Runtime where
		Block = Block,
		NodeBlock = primitives::Block,
		UncheckedExtrinsic = UncheckedExtrinsic
	{
		// Core
		System: frame_system::{Pallet, Call, Storage, Config, Event<T>} = 0,
		Timestamp: pallet_timestamp::{Pallet, Call, Storage, Inherent} = 1,
		Scheduler: pallet_scheduler::{Pallet, Call, Storage, Event<T>} = 2,
		RandomnessCollectiveFlip: pallet_randomness_collective_flip::{Module, Call, Storage} = 3,

		// Tokens & Related
		Balances: pallet_balances::{Module, Call, Storage, Config<T>, Event<T>} = 4,
		Tokens: orml_tokens::{Module, Storage, Event<T>, Config<T>} = 5,
		Currencies: setheum_currencies::{Module, Call, Event<T>} = 6,
		NFT: setheum_nft::{Module, Call, Event<T>} = 7,
		Vesting: orml_vesting::{Module, Storage, Call, Event<T>, Config<T>} = 8,
		TransactionPayment: setheum_transaction_payment::{Module, Call, Storage} = 9,

		// Treasury
		Treasury: pallet_treasury::{Module, Call, Storage, Config, Event<T>} = 10,
		Bounties: pallet_bounties::{Module, Call, Storage, Event<T>} = 11,
		Tips: pallet_tips::{Module, Call, Storage, Event<T>} = 12,

		// Utility
		Utility: pallet_utility::{Module, Call, Event} = 13,
		Multisig: pallet_multisig::{Module, Call, Storage, Event<T>} = 14,
		Recovery: pallet_recovery::{Module, Call, Storage, Event<T>} = 15,
		Proxy: pallet_proxy::{Module, Call, Storage, Event<T>} = 16 ,
		Indices: pallet_indices::{Module, Call, Storage, Config<T>, Event<T>} = 17,

		// Consensus & Staking
		Authorship: pallet_authorship::{Module, Call, Storage, Inherent} = 18,
		Babe: pallet_babe::{Module, Call, Storage, Config, Inherent, ValidateUnsigned} = 19,
		Grandpa: pallet_grandpa::{Module, Call, Storage, Config, Event, ValidateUnsigned} = 20,
		Staking: pallet_staking::{Module, Call, Config<T>, Storage, Event<T>} = 21,
		Session: pallet_session::{Module, Call, Storage, Event, Config<T>} = 22,
		Historical: pallet_session_historical::{Module} = 23,

		// Governance
		GeneralCouncil: pallet_collective::<Instance1>::{Module, Call, Storage, Origin<T>, Event<T>, Config<T>} = 24,
		GeneralCouncilMembership: pallet_membership::<Instance1>::{Module, Call, Storage, Event<T>, Config<T>} = 25,
		FinancialCouncil: pallet_collective::<Instance2>::{Module, Call, Storage, Origin<T>, Event<T>, Config<T>} = 26,
		FinancialCouncilMembership: pallet_membership::<Instance2>::{Module, Call, Storage, Event<T>, Config<T>} = 27,
		TechnicalCommittee: pallet_collective::<Instance4>::{Module, Call, Storage, Origin<T>, Event<T>, Config<T>} = 28,
		TechnicalCommitteeMembership: pallet_membership::<Instance4>::{Module, Call, Storage, Event<T>, Config<T>} = 29,
		SetheumJury: pallet_collective::<Instance1>::{Module, Call, Storage, Origin<T>, Event<T>, Config<T>} = 30,
		SetheumJuryMembership: pallet_membership::<Instance1>::{Module, Call, Storage, Event<T>, Config<T>} = 31,
		Authority: orml_authority::{Module, Call, Event<T>, Origin<T>} = 32,
		
		// Oracle
		//
		// NOTE: OperatorMembership must be placed after Oracle or else will have race condition on initialization
		SetheumOracle: orml_oracle::<Instance1>::{Module, Storage, Call, Config<T>, Event<T>} = 33,
		// OperatorMembership must be placed after Oracle or else will have race condition on initialization
		OperatorMembershipSetheum: pallet_membership::<Instance5>::{Module, Call, Storage, Event<T>, Config<T>} = 34,

		// ORML Core
		Auction: orml_auction::{Module, Storage, Call, Event<T>} = 35,
		Rewards: orml_rewards::{Module, Storage, Call} = 36,
		OrmlNFT: orml_nft::{Module, Storage, Config<T>} = 37,

		// Setheum Core
		Prices: setheum_prices::{Module, Storage, Call, Event<T>} = 38,
		SerpAuctionManager: serp_auction::{Module, Storage, Call, Event<T>, ValidateUnsigned} 39,
		SerpTreasury: serp_treasury::{Module, Storage, Call, Config, Event<T>} = 40,

		// Dex
		Dex: dex::{Module, Storage, Call, Event<T>, Config<T>} = 41,
		Incentives: setheum_incentives::{Module, Storage, Call, Event<T>} = 42,

		// Settmint
		SettmintEngine: settmint_engine::{Module, Storage, Call, Event<T>, Config, ValidateUnsigned} = 41,
		SettmintGateway: settmint_gateway::{Module, Storage, Call, Event<T>} = 43,
		SettmintManager: settmint_manager::{Module, Storage, Call, Event<T>} = 44,

		// Smart contracts
		// Setheum EVM (SEVM)
		EVM: setheum_evm::{Pallet, Config<T>, Call, Storage, Event<T>} = 45,
		EVMBridge: setheum_evm_bridge::{Pallet} = 46,
		EvmAccounts: setheum_evm_accounts::{Pallet, Call, Storage, Event<T>} = 47,
		EvmManager: setheum_evm_manager::{Pallet, Storage} = 48,

		// Dev
		Sudo: pallet_sudo::{Module, Call, Config<T>, Storage, Event<T>} = 255,
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
	setheum_transaction_payment::ChargeTransactionPayment<Runtime>,
);
/// Unchecked extrinsic type as expected by this runtime.
pub type UncheckedExtrinsic = generic::UncheckedExtrinsic<Address, Call, Signature, SignedExtra>;
/// The payload being signed in transactions.
pub type SignedPayload = generic::SignedPayload<Call, SignedExtra>;
/// Extrinsic type that has already been checked.
pub type CheckedExtrinsic = generic::CheckedExtrinsic<AccountId, Call, SignedExtra>;
/// Executive: handles dispatch to the various modules.
pub type Executive =
	frame_executive::Executive<Runtime, Block, frame_system::ChainContext<Runtime>, Runtime, AllModules, ()>;

#[cfg(not(feature = "disable-runtime-api"))]
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

	impl sp_session::SessionKeys<Block> for Runtime {
		fn generate_session_keys(seed: Option<Vec<u8>>) -> Vec<u8> {
			SessionKeys::generate(seed)
		}

		fn decode_session_keys(
			encoded: Vec<u8>,
		) -> Option<Vec<(Vec<u8>, KeyTypeId)>> {
			SessionKeys::decode_into_raw_public_keys(&encoded)
		}
	}

	impl fg_primitives::GrandpaApi<Block> for Runtime {
		fn grandpa_authorities() -> GrandpaAuthorityList {
			Grandpa::grandpa_authorities()
		}

		fn submit_report_equivocation_unsigned_extrinsic(
			equivocation_proof: fg_primitives::EquivocationProof<
				<Block as BlockT>::Hash,
				NumberFor<Block>,
			>,
			key_owner_proof: fg_primitives::OpaqueKeyOwnershipProof,
		) -> Option<()> {
			let key_owner_proof = key_owner_proof.decode()?;

			Grandpa::submit_unsigned_equivocation_report(
				equivocation_proof,
				key_owner_proof,
			)
		}

		fn generate_key_ownership_proof(
			_set_id: fg_primitives::SetId,
			authority_id: GrandpaId,
		) -> Option<fg_primitives::OpaqueKeyOwnershipProof> {
			use codec::Encode;

			Historical::prove((fg_primitives::KEY_TYPE, authority_id))
				.map(|p| p.encode())
				.map(fg_primitives::OpaqueKeyOwnershipProof::new)
		}
	}

	impl frame_system_rpc_runtime_api::AccountNonceApi<Block, AccountId, Nonce> for Runtime {
		fn account_nonce(account: AccountId) -> Nonce {
			System::account_nonce(account)
		}
	}

	impl pallet_transaction_payment_rpc_runtime_api::TransactionPaymentApi<
		Block,
		Balance,
	> for Runtime {
		fn query_info(uxt: <Block as BlockT>::Extrinsic, len: u32) -> pallet_transaction_payment_rpc_runtime_api::RuntimeDispatchInfo<Balance> {
			TransactionPayment::query_info(uxt, len)
		}

		fn query_fee_details(uxt: <Block as BlockT>::Extrinsic, len: u32) -> pallet_transaction_payment_rpc_runtime_api::FeeDetails<Balance> {
			TransactionPayment::query_fee_details(uxt, len)
		}
	}

	impl orml_oracle_rpc_runtime_api::OracleApi<
		Block,
		DataProviderId,
		CurrencyId,
		TimeStampedPrice,
	> for Runtime {
		fn get_value(provider_id: DataProviderId ,key: CurrencyId) -> Option<TimeStampedPrice> {
			match provider_id {
				DataProviderId::Setheum => SetheumOracle::get_no_op(&key),
				DataProviderId::Aggregated => <AggregatedDataProvider as DataProviderExtended<_, _>>::get_no_op(&key)
			}
		}

		fn get_all_values(provider_id: DataProviderId) -> Vec<(CurrencyId, Option<TimeStampedPrice>)> {
			match provider_id {
				DataProviderId::Setheum => SetheumOracle::get_all_values(),
				DataProviderId::Aggregated => <AggregatedDataProvider as DataProviderExtended<_, _>>::get_all_values()
			}
		}
	}

	impl setheum_evm_rpc_runtime_api::EVMRuntimeRPCApi<Block, Balance> for Runtime {
		fn call(
			from: H160,
			to: H160,
			data: Vec<u8>,
			value: Balance,
			gas_limit: u64,
			storage_limit: u32,
			estimate: bool,
		) -> Result<CallInfo, sp_runtime::DispatchError> {
			let config = if estimate {
				let mut config = <Runtime as setheum_evm::Config>::config().clone();
				config.estimate = true;
				Some(config)
			} else {
				None
			};

			setheum_evm::Runner::<Runtime>::call(
				from,
				from,
				to,
				data,
				value,
				gas_limit,
				storage_limit,
				config.as_ref().unwrap_or(<Runtime as setheum_evm::Config>::config()),
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
			let config = if estimate {
				let mut config = <Runtime as setheum_evm::Config>::config().clone();
				config.estimate = true;
				Some(config)
			} else {
				None
			};

			setheum_evm::Runner::<Runtime>::create(
				from,
				data,
				value,
				gas_limit,
				storage_limit,
				config.as_ref().unwrap_or(<Runtime as setheum_evm::Config>::config()),
			)
		}

		fn get_estimate_resources_request(extrinsic: Vec<u8>) -> Result<EstimateResourcesRequest, sp_runtime::DispatchError> {
			let utx = UncheckedExtrinsic::decode(&mut &*extrinsic)
				.map_err(|_| sp_runtime::DispatchError::Other("Invalid parameter extrinsic, decode failed"))?;

			let request = match utx.function {
				Call::EVM(setheum_evm::Call::call(to, data, value, gas_limit, storage_limit)) => {
					Some(EstimateResourcesRequest {
						from: None,
						to: Some(to),
						gas_limit: Some(gas_limit),
						storage_limit: Some(storage_limit),
						value: Some(value),
						data: Some(data),
					})
				}
				Call::EVM(setheum_evm::Call::create(data, value, gas_limit, storage_limit)) => {
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

	#[cfg(feature = "try-runtime")]
	impl frame_try_runtime::TryRuntime<Block> for Runtime {
		fn on_runtime_upgrade() -> Result<(Weight, Weight), sp_runtime::RuntimeString> {
			let weight = Executive::try_runtime_upgrade()?;
			Ok((weight, RuntimeBlockWeights::get().max_block))
		}
	}

	// benchmarks for setheum modules
	#[cfg(feature = "runtime-benchmarks")]
	impl frame_benchmarking::Benchmark<Block> for Runtime {
		fn dispatch_benchmark(
			config: frame_benchmarking::BenchmarkConfig
		) -> Result<Vec<frame_benchmarking::BenchmarkBatch>, sp_runtime::RuntimeString> {
			use frame_benchmarking::{Benchmarking, BenchmarkBatch, add_benchmark, TrackedStorageKey};
			use orml_benchmarking::{add_benchmark as orml_add_benchmark};

			use setheum_nft_benchmarking::Module as NftBench;
			impl setheum_nft_benchmarking::Config for Runtime {}

			// TODO: Update
			let whitelist: Vec<TrackedStorageKey> = vec![
				// Block Number
				// frame_system::Number::<Runtime>::hashed_key().to_vec(),
				hex_literal::hex!("26aa394eea5630e07c48ae0c9558cef702a5c1b19ab7a04f536c519set4983ac").to_vec().into(),
				// Total Issuance
				hex_literal::hex!("c2261276cc9d1f8598ea4b6a74b15c2f57c875e4cff74148e4628f264b974c80").to_vec().into(),
				// Execution Phase
				hex_literal::hex!("26aa394eea5630e07c48ae0c9558cef7ff553b5a9862a516939d82b3d3d8661a").to_vec().into(),
				// Event Count
				hex_literal::hex!("26aa394eea5630e07c48ae0c9558cef70a98fdbe9ce6c55837576c60c7af3850").to_vec().into(),
				// System Events
				hex_literal::hex!("26aa394eea5630e07c48ae0c9558cef780d41e5e16056765bc8461851072c9d7").to_vec().into(),
				// Caller 0 Account
				hex_literal::hex!("26aa394eea5630e07c48ae0c9558cef7b99d880ec681799c0cf30e8886371da946c154ffd9992e395af90b5b13cc6f295c77033fce8a9045824a6690bbf99c6db269502f0a8d1d2a008542d5690a0749").to_vec().into(),
				// Treasury Account
				hex_literal::hex!("26aa394eea5630e07c48ae0c9558cef7b99d880ec681799c0cf30e8886371da95ecffd7b6c0f78751baa9d281e0bfa3a6d6f646c70792f74727372790000000000000000000000000000000000000000").to_vec().into(),
			];
			let mut batches = Vec::<BenchmarkBatch>::new();
			let params = (&config, &whitelist);

			// TODO: Update!
			orml_add_benchmark!(params, batches, orml_auction, benchmarking::auction);
			orml_add_benchmark!(params, batches, serp_auction, benchmarking::serp_auction);
			orml_add_benchmark!(params, batches, orml_authority, benchmarking::authority);
			orml_add_benchmark!(params, batches, orml_currencies, benchmarking::currencies);
			orml_add_benchmark!(params, batches, dex, benchmarking::dex);
			orml_add_benchmark!(params, batches, setheum_evm_accounts, benchmarking::evm_accounts);
			orml_add_benchmark!(params, batches, setheum_evm, benchmarking::evm);
			orml_add_benchmark!(params, batches, orml_rewards, benchmarking::rewards);
			orml_add_benchmark!(params, batches, incentives, benchmarking::incentives);
			orml_add_benchmark!(params, batches, orml_oracle, benchmarking::oracle);
			orml_add_benchmark!(params, batches, prices, benchmarking::prices);
			orml_add_benchmark!(params, batches, serp_treasury, benchmarking::serp_treasury);
			// orml_add_benchmark!(params, batches, settmint_engine, benchmarking::settmint_engine);
			orml_add_benchmark!(params, batches, settmint_gateway, benchmarking::settmint_gateway);
			orml_add_benchmark!(params, batches, orml_tokens, benchmarking::tokens);
			orml_add_benchmark!(params, batches, transaction_payment, benchmarking::transaction_payment);
			orml_add_benchmark!(params, batches, orml_vesting, benchmarking::vesting);

			add_benchmark!(params, batches, nft, NftBench::<Runtime>);

			if batches.is_empty() { return Err("Benchmark not found for this module.".into()) }
			Ok(batches)
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use frame_system::offchain::CreateSignedTransaction;

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