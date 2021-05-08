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

//! The Dev runtime. This can be compiled with `#[no_std]`, ready for Wasm.

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
use hex_literal::hex;
use sp_api::impl_runtime_apis;
use sp_core::{
	crypto::KeyTypeId,
	u32_trait::{_1, _2, _3, _4},
	OpaqueMetadata, H160,
};
use sp_runtime::{
	create_runtime_str, generic, impl_opaque_keys,
	traits::{AccountIdConversion, BadOrigin, BlakeTwo256, Block as BlockT, SaturatedConversion, StaticLookup, Zero},
	transaction_validity::{TransactionSource, TransactionValidity},
	ApplyExtrinsicResult, DispatchResult, FixedPointNumber, ModuleId,
};
use sp_std::prelude::*;
#[cfg(feature = "std")]
use sp_version::NativeVersion;
use sp_version::RuntimeVersion;

use frame_system::{EnsureOneOf, EnsureRoot, RawOrigin};
use setheum_currencies::{BasicCurrencyAdapter, Currency};
use evm::{CallInfo, CreateInfo};
use evm_accounts::EvmAddressMapping;
use module_transaction_payment::{Multiplier, TargetedFeeAdjustment};
use orml_tokens::CurrencyAdapter;
use orml_traits::{create_median_value_data_provider, parameter_type_with_key, DataFeeder, DataProviderExtended};
use pallet_transaction_payment::{FeeDetails, RuntimeDispatchInfo};

#[cfg(any(feature = "std", test))]
pub use pallet_staking::StakerStatus;

#[cfg(feature = "standalone")]
use standalone_use::*;
#[cfg(feature = "standalone")]
mod standalone_use {
	pub use pallet_grandpa::{fg_primitives, AuthorityId as GrandpaId, AuthorityList as GrandpaAuthorityList};
	pub use pallet_session::historical as pallet_session_historical;
	pub use pallet_staking::StakerStatus;
	pub use sp_runtime::{
		curve::PiecewiseLinear,
		traits::{NumberFor, OpaqueKeys},
		transaction_validity::TransactionPriority,
	};
}

#[cfg(not(feature = "standalone"))]
use parachain_use::*;
#[cfg(not(feature = "standalone"))]
mod parachain_use {
	pub use orml_xcm_support::{
		CurrencyIdConverter, IsConcreteWithGeneralKey, MultiCurrencyAdapter, NativePalletAssetOr,
		XcmHandler as XcmHandlerT,
	};
	pub use polkadot_parachain::primitives::Sibling;
	pub use sp_runtime::traits::{Convert, Identity};
	pub use sp_std::collections::btree_set::BTreeSet;
	pub use xcm::v0::{Junction, MultiLocation, NetworkId, Xcm};
	pub use xcm_builder::{
		AccountId32Aliases, LocationInverter, ParentIsDefault, RelayChainAsNative, SiblingParachainAsNative,
		SiblingParachainConvertsVia, SignedAccountId32AsNative, SovereignSignedViaLocation,
	};
	pub use xcm_executor::{Config, XcmExecutor};
}

/// Weights for pallets used in the runtime.
mod weights;

pub use frame_support::{
	construct_runtime, log, parameter_types,
	traits::{
		Contains, ContainsLengthBound, EnsureOrigin, Filter, Get, IsType, KeyOwnerProofSystem, LockIdentifier,
		Randomness, U128CurrencyToVote,
	},
	weights::{
		constants::{BlockExecutionWeight, ExtrinsicBaseWeight, RocksDbWeight, WEIGHT_PER_SECOND},
		DispatchClass, IdentityFee, Weight,
	},
	StorageValue,
};

// pub use pallet_staking::StakerStatus;
pub use pallet_timestamp::Call as TimestampCall;
#[cfg(any(feature = "std", test))]
pub use sp_runtime::BuildStorage;
pub use sp_runtime::{Perbill, Percent, Permill, Perquintill};

pub use authority::AuthorityConfigImpl;
pub use constants::{fee::*, time::*};
pub use primitives::{
	AccountId, AccountIndex, AirDropCurrencyId, Amount, AuthoritysOriginId, Balance, BlockNumber,
	CurrencyId, DataProviderId, EraIndex, Hash, Moment, Nonce, Share, Signature, TokenSymbol, TradingPair,
};
pub use runtime_common::{
	cent, deposit, dollar, microcent, millicent, ExchangeRate, GasToWeight, OffchainSolutionWeightLimit,
	Price, Rate, Ratio, RuntimeBlockLength, RuntimeBlockWeights, SystemContractsFilter, TimeStampedPrice, 
	DNAR, JUSD, JEUR, JGBP, NEOM, JSAR, JCHF, JNGN, SETN, HALAL, DOT, KSM,
};

mod authority;
mod benchmarking;
mod constants;

/// This runtime version.
pub const VERSION: RuntimeVersion = RuntimeVersion {
	spec_name: create_runtime_str!("newrome"),
	impl_name: create_runtime_str!("newrome"),
	authoring_version: 1,
	spec_version: 730,
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

#[cfg(not(feature = "standalone"))]
impl_opaque_keys! {
	pub struct SessionKeys {}
}

// Pallet accounts of runtime
parameter_types! {
	pub const SetheumTreasuryModuleId: ModuleId = ModuleId(*b"dnr/trsy");
	pub const SetheumDexModuleId: ModuleId = ModuleId(*b"dnr/sdex");
	// Setheum Investment Fund
	pub const SIFModuleId: ModuleId = ModuleId(*b"dnr/sSIF");
	pub const ElectionsPhragmenModuleId: LockIdentifier = *b"dnr/phre";
	pub const NftModuleId: ModuleId = ModuleId(*b"dnr/sNFT");
}

pub fn get_all_module_accounts() -> Vec<AccountId> {
	vec![
		SetheumTreasuryModuleId::get().into_account(),
		SetheumDexModuleId::get().into_account(),
		SIFModuleId::get().into_account(),
		ZeroAccountId::get(),
	]
}

parameter_types! {
	pub const BlockHashCount: BlockNumber = 900; // mortal tx can be valid up to 1 hour after signing
	pub const Version: RuntimeVersion = VERSION;
	pub const SS58Prefix: u8 = 42; // Ss58AddressFormat::SubstrateAccount
}

impl frame_system::Config for Runtime {
	type AccountId = AccountId;
	type Call = Call;
	type Lookup = (Indices, SevmAccounts);
	type Index = Nonce;
	type BlockNumber = BlockNumber;
	type Hash = Hash;
	type Hashing = BlakeTwo256;
	type Header = generic::Header<BlockNumber, BlakeTwo256>;
	type Event = Event;
	type Origin = Origin;
	type BlockHashCount = BlockHashCount;
	type BlockWeights = RuntimeBlockWeights;
	type BlockLength = RuntimeBlockLength;
	type Version = Version;
	type PalletInfo = PalletInfo;
	type AccountData = pallet_balances::AccountData<Balance>;
	type OnNewAccount = ();
	type OnKilledAccount = (
		evm::CallKillAccount<Runtime>,
		evm_accounts::CallKillAccount<Runtime>,
	);
	type DbWeight = RocksDbWeight;
	type BaseCallFilter = ();
	type SystemWeightInfo = ();
	type SS58Prefix = SS58Prefix;
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
	type OnTimestampSet = ();
	// type OnTimestampSet = Babe;
	type MinimumPeriod = MinimumPeriod;
	type WeightInfo = ();
}

parameter_types! {
	pub const NativeTokenExistentialDeposit: Balance = 0;
	// For weight estimation, we assume that the most locks on an individual account will be 50.
	// This number may need to be adjusted in the future if this assumption no longer holds true.
	pub const MaxLocks: u32 = 50;
}

impl pallet_balances::Config for Runtime {
	type Balance = Balance;
	type DustRemoval = SetheumTreasury;
	type Event = Event;
	type ExistentialDeposit = NativeTokenExistentialDeposit;
	type AccountStore = frame_system::Pallet<Runtime>;
	type MaxLocks = MaxLocks;
	type WeightInfo = ();
}

parameter_types! {
	pub TransactionByteFee: Balance = 10 * millicent(DNAR);
	pub const TargetBlockFullness: Perquintill = Perquintill::from_percent(25);
	pub AdjustmentVariable: Multiplier = Multiplier::saturating_from_rational(1, 100_000);
	pub MinimumMultiplier: Multiplier = Multiplier::saturating_from_rational(1, 1_000_000_000u128);
}

impl pallet_sudo::Config for Runtime {
	type Event = Event;
	type Call = Call;
}

type EnsureRootOrHalfGeneralCouncil = EnsureOneOf<
	AccountId,
	EnsureRoot<AccountId>,
	pallet_collective::EnsureProportionMoreThan<_1, _2, AccountId, GeneralCouncilInstance>,
>;

type EnsureRootOrTwoThirdsGeneralCouncil = EnsureOneOf<
	AccountId,
	EnsureRoot<AccountId>,
	pallet_collective::EnsureProportionMoreThan<_2, _3, AccountId, GeneralCouncilInstance>,
>;

type EnsureRootOrThreeFourthsGeneralCouncil = EnsureOneOf<
	AccountId,
	EnsureRoot<AccountId>,
	pallet_collective::EnsureProportionMoreThan<_3, _4, AccountId, GeneralCouncilInstance>,
>;

type EnsureRootOrOneThirdsTechnicalCommittee = EnsureOneOf<
	AccountId,
	EnsureRoot<AccountId>,
	pallet_collective::EnsureProportionMoreThan<_1, _3, AccountId, TechnicalCommitteeInstance>,
>;

type EnsureRootOrTwoThirdsTechnicalCommittee = EnsureOneOf<
	AccountId,
	EnsureRoot<AccountId>,
	pallet_collective::EnsureProportionMoreThan<_2, _3, AccountId, TechnicalCommitteeInstance>,
>;

parameter_types! {
	pub const GeneralCouncilMotionDuration: BlockNumber = 7 * DAYS;
	pub const GeneralCouncilMaxProposals: u32 = 100;
	pub const GeneralCouncilMaxMembers: u32 = 100;
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
}

parameter_types! {
	pub const TechnicalCommitteeMotionDuration: BlockNumber = 7 * DAYS;
	pub const TechnicalCommitteeMaxProposals: u32 = 100;
	pub const TechnicalCouncilMaxMembers: u32 = 100;
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
}

type OperatorMembershipInstancesetheum = pallet_membership::Instance5;
impl pallet_membership::Config<OperatorMembershipInstancesetheum> for Runtime {
	type Event = Event;
	type AddOrigin = EnsureRootOrTwoThirdsGeneralCouncil;
	type RemoveOrigin = EnsureRootOrTwoThirdsGeneralCouncil;
	type SwapOrigin = EnsureRootOrTwoThirdsGeneralCouncil;
	type ResetOrigin = EnsureRootOrTwoThirdsGeneralCouncil;
	type PrimeOrigin = EnsureRootOrTwoThirdsGeneralCouncil;
	type MembershipInitialized = SetheumOracle;
	type MembershipChanged = SetheumOracle;
}

type OperatorMembershipInstanceBand = pallet_membership::Instance6;
impl pallet_membership::Config<OperatorMembershipInstanceBand> for Runtime {
	type Event = Event;
	type AddOrigin = EnsureRootOrTwoThirdsGeneralCouncil;
	type RemoveOrigin = EnsureRootOrTwoThirdsGeneralCouncil;
	type SwapOrigin = EnsureRootOrTwoThirdsGeneralCouncil;
	type ResetOrigin = EnsureRootOrTwoThirdsGeneralCouncil;
	type PrimeOrigin = EnsureRootOrTwoThirdsGeneralCouncil;
	type MembershipInitialized = BandOracle;
	type MembershipChanged = BandOracle;
}

impl pallet_utility::Config for Runtime {
	type Event = Event;
	type Call = Call;
	type WeightInfo = ();
}

parameter_types! {
	pub MultisigDepositBase: Balance = 500 * millicent(DNAR);
	pub MultisigDepositFactor: Balance = 100 * millicent(DNAR);
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
	pub ProposalBondMinimum: Balance = dollar(DNAR);
	pub const SpendPeriod: BlockNumber = DAYS;
	pub const Burn: Permill = Permill::from_percent(0);
	pub const TipCountdown: BlockNumber = DAYS;
	pub const TipFindersFee: Percent = Percent::from_percent(10);
	pub TipReportDepositBase: Balance = dollar(DNAR);
	pub const SevenDays: BlockNumber = 7 * DAYS;
	pub const ZeroDay: BlockNumber = 0;
	pub const OneDay: BlockNumber = DAYS;
	pub BountyDepositBase: Balance = dollar(DNAR);
	pub const BountyDepositPayoutDelay: BlockNumber = DAYS;
	pub const BountyUpdatePeriod: BlockNumber = 14 * DAYS;
	pub const BountyCuratorDeposit: Permill = Permill::from_percent(50);
	pub BountyValueMinimum: Balance = 5 * dollar(DNAR);
	pub DataDepositPerByte: Balance = cent(DNAR);
	pub const MaximumReasonLength: u32 = 16384;
}

impl pallet_treasury::Config for Runtime {
	type ModuleId = SetheumTreasuryModuleId;
	type Currency = Balances;
	type ApproveOrigin = EnsureRootOrHalfGeneralCouncil;
	type RejectOrigin = EnsureRootOrHalfGeneralCouncil;
	type Event = Event;
	type OnSlash = ();
	type ProposalBond = ProposalBond;
	type ProposalBondMinimum = ProposalBondMinimum;
	type SpendPeriod = SpendPeriod;
	type Burn = Burn;
	type BurnDestination = ();
	type SpendFunds = Bounties;
	type WeightInfo = ();
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
	pub const ExpiresIn: Moment = 1000 * 60 * 60; // 60 mins
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
	type WeightInfo = weights::orml_oracle::WeightInfo<Runtime>;
}

type BandDataProvider = orml_oracle::Instance2;
impl orml_oracle::Config<BandDataProvider> for Runtime {
	type Event = Event;
	type OnNewData = ();
	type CombineData = orml_oracle::DefaultCombineData<Runtime, MinimumCount, ExpiresIn, BandDataProvider>;
	type Time = Timestamp;
	type OracleKey = CurrencyId;
	type OracleValue = Price;
	type RootOperatorAccountId = ZeroAccountId;
	type WeightInfo = weights::orml_oracle::WeightInfo<Runtime>;
}

create_median_value_data_provider!(
	AggregatedDataProvider,
	CurrencyId,
	Price,
	TimeStampedPrice,
	[SetheumOracle, BandOracle]
);
// Aggregated data provider cannot feed.
impl DataFeeder<CurrencyId, Price, AccountId> for AggregatedDataProvider {
	fn feed_value(_: AccountId, _: CurrencyId, _: Price) -> DispatchResult {
		Err("Not supported".into())
	}
}

parameter_type_with_key! {
	pub ExistentialDeposits: |_currency_id: CurrencyId| -> Balance {
		Zero::zero()
	};
}

parameter_types! {
	pub TreasuryModuleAccount: AccountId = SetheumTreasuryModuleId::get().into_account();
}

impl orml_tokens::Config for Runtime {
	type Event = Event;
	type Balance = Balance;
	type Amount = Amount;
	type CurrencyId = CurrencyId;
	type WeightInfo = weights::orml_tokens::WeightInfo<Runtime>;
	type ExistentialDeposits = ExistentialDeposits;
	type OnDust = orml_tokens::TransferDust<Runtime, TreasuryModuleAccount>;
}

parameter_types! {
	pub StableCurrencyFixedPrice: Price = Price::saturating_from_rational(1, 1);
}

impl setheum_prices::Config for Runtime {
	type Event = Event;
	type Source = AggregatedDataProvider;
	type GetStableCurrencyId = GetStableCurrencyId;
	type StableCurrencyFixedPrice = StableCurrencyFixedPrice;
	type LockOrigin = EnsureRootOrTwoThirdsGeneralCouncil;
	type DEX = SetheumDex;
	type Currency = Currencies;
	type WeightInfo = weights::setheum_prices::WeightInfo<Runtime>;
}

parameter_types! {
	pub const GetNativeCurrencyId: CurrencyId = DNAR;
	pub const GetStableCurrencyId: CurrencyId = JUSD;
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

pub struct EnsureRootOrSetheumTreasury;
impl EnsureOrigin<Origin> for EnsureRootOrSetheumTreasury {
	type Success = AccountId;

	fn try_origin(o: Origin) -> Result<Self::Success, Origin> {
		Into::<Result<RawOrigin<AccountId>, Origin>>::into(o).and_then(|o| match o {
			RawOrigin::Root => Ok(SetheumTreasuryModuleId::get().into_account()),
			RawOrigin::Signed(caller) => {
				if caller == SetheumTreasuryModuleId::get().into_account() {
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
	pub MinVestedTransfer: Balance = 100 * dollar(DNAR);
}

impl orml_vesting::Config for Runtime {
	type Event = Event;
	type Currency = pallet_balances::Pallet<Runtime>;
	type MinVestedTransfer = MinVestedTransfer;
	type VestedTransferOrigin = EnsureRootOrsetheumTreasury;
	type WeightInfo = weights::orml_vesting::WeightInfo<Runtime>;
}

parameter_types! {
	pub MaximumSchedulerWeight: Weight = Perbill::from_percent(10) * RuntimeBlockWeights::get().max_block;
	pub const MaxScheduledPerBlock: u32 = 50;
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
	pub const UpdateFrequency: BlockNumber = 10;
}

impl orml_gradually_update::Config for Runtime {
	type Event = Event;
	type UpdateFrequency = UpdateFrequency;
	type DispatchOrigin = EnsureRoot<AccountId>;
	type WeightInfo = weights::orml_gradually_update::WeightInfo<Runtime>;
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
			evm::SetEvmOrigin::<Runtime>::new(),
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
	pub const GetExchangeFee: (u32, u32) = (1, 1000);	// 0.1%
	pub const TradingPathLimit: u32 = 3;
	pub EnabledTradingPairs: Vec<TradingPair> = vec![
		TradingPair::new(JUSD, DNAR),
		TradingPair::new(JUSD, DOT),
		TradingPair::new(JUSD, JEUR),
	];
}

impl setheum_dex::Config for Runtime {
	type Event = Event;
	type Currency = Currencies;
	type GetExchangeFee = GetExchangeFee;
	type TradingPathLimit = TradingPathLimit;
	type ModuleId = SetheumDexModuleId;
	type DexIncentives = Incentives;
	type WeightInfo = weights::setheum_dex::WeightInfo<Runtime>;
	type ListingOrigin = EnsureRootOrHalfGeneralCouncil;
}

parameter_types! {
	// All currency types except for native currency, Sort by fee charge order
	pub AllNonNativeCurrencyIds: Vec<CurrencyId> = vec![JUSD, JEUR, JGBP, JSAR, JCHF, JNGN, SETN, HALAL, DOT, KSM];
}

impl module_transaction_payment::Config for Runtime {
	type AllNonNativeCurrencyIds = AllNonNativeCurrencyIds;
	type NativeCurrencyId = GetNativeCurrencyId;
	type StableCurrencyId = GetStableCurrencyId;
	type Currency = Balances;
	type MultiCurrency = Currencies;
	type OnTransactionPayment = SetheumTreasury;
	type TransactionByteFee = TransactionByteFee;
	type WeightToFee = WeightToFee;
	type FeeMultiplierUpdate = TargetedFeeAdjustment<Self, TargetBlockFullness, AdjustmentVariable, MinimumMultiplier>;
	type DEX = SetheumDex;
	type MaxSlippageSwapWithDEX = MaxSlippageSwapWithDEX;
	type WeightInfo = weights::module_transaction_payment::WeightInfo<Runtime>;
}

impl evm_accounts::Config for Runtime {
	type Event = Event;
	type Currency = Balances;
	type AddressMapping = EvmAddressMapping<Runtime>;
	type MergeAccount = Currencies;
	type WeightInfo = weights::evm_accounts::WeightInfo<Runtime>;
}

impl orml_rewards::Config for Runtime {
	type Share = Balance;
	type Balance = Balance;
	type PoolId = setheum_incentives::PoolId<AccountId>;
	type Handler = Incentives;
}

impl module_airdrop::Config for Runtime {
	type Event = Event;
}

parameter_types! {
	pub CreateClassDeposit: Balance = 500 * millicent(DNAR);
	pub CreateTokenDeposit: Balance = 100 * millicent(DNAR);
}

impl setheum_nft::Config for Runtime {
	type Event = Event;
	type CreateClassDeposit = CreateClassDeposit;
	type CreateTokenDeposit = CreateTokenDeposit;
	type ModuleId = NftModuleId;
	type Currency = Currency<Runtime, GetNativeCurrencyId>;
	type WeightInfo = weights::setheum_nft::WeightInfo<Runtime>;
}

impl orml_nft::Config for Runtime {
	type ClassId = u32;
	type TokenId = u64;
	type ClassData = setheum_nft::ClassData;
	type TokenData = setheum_nft::TokenData;
}

parameter_types! {
	// One storage item; key size 32, value size 8; .
	pub ProxyDepositBase: Balance = deposit(1, 8, DNAR);
	// Additional storage item size of 33 bytes.
	pub ProxyDepositFactor: Balance = deposit(0, 33, DNAR);
	pub const MaxProxies: u16 = 32;
	pub AnnouncementDepositBase: Balance = deposit(1, 8, DNAR);
	pub AnnouncementDepositFactor: Balance = deposit(0, 66, DNAR);
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
	pub const ChainId: u64 = 595;
	pub NetworkContractSource: H160 = H160::from_low_u64_be(0);
}

#[cfg(feature = "with-ethereum-compatibility")]
parameter_types! {
	pub const NewContractExtraBytes: u32 = 0;
	pub const StorageDepositPerByte: Balance = 0;
	pub const MaxCodeSize: u32 = 0x6000;
	pub const DeveloperDeposit: Balance = 0;
	pub const DeploymentFee: Balance = 0;
}

#[cfg(not(feature = "with-ethereum-compatibility"))]
parameter_types! {
	pub const NewContractExtraBytes: u32 = 10_000;
	pub StorageDepositPerByte: Balance = microcent(DNAR);
	pub const MaxCodeSize: u32 = 60 * 1024;
	pub DeveloperDeposit: Balance = dollar(DNAR);
	pub DeploymentFee: Balance = dollar(DNAR);
}

pub type MultiCurrencyPrecompile =
	runtime_common::MultiCurrencyPrecompile<AccountId, EvmAddressMapping<Runtime>, Currencies>;

pub type NFTPrecompile = runtime_common::NFTPrecompile<AccountId, EvmAddressMapping<Runtime>, NFT>;
pub type StateRentPrecompile = runtime_common::StateRentPrecompile<AccountId, EvmAddressMapping<Runtime>, EVM>;
pub type OraclePrecompile = runtime_common::OraclePrecompile<AccountId, EvmAddressMapping<Runtime>, Prices>;
pub type ScheduleCallPrecompile = runtime_common::ScheduleCallPrecompile<
	AccountId,
	EvmAddressMapping<Runtime>,
	Scheduler,
	module_transaction_payment::ChargeTransactionPayment<Runtime>,
	Call,
	Origin,
	OriginCaller,
	Runtime,
>;
pub type DexPrecompile = runtime_common::DexPrecompile<AccountId, EvmAddressMapping<Runtime>, Dex>;

#[cfg(feature = "with-ethereum-compatibility")]
static ISTANBUL_CONFIG: evm::Config = evm::Config::istanbul();

impl evm::Config for Runtime {
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
		NFTPrecompile,
		StateRentPrecompile,
		OraclePrecompile,
		ScheduleCallPrecompile,
		DexPrecompile,
	>;
	type ChainId = ChainId;
	type GasToWeight = GasToWeight;
	type ChargeTransactionPayment = module_transaction_payment::ChargeTransactionPayment<Runtime>;
	type NetworkContractOrigin = EnsureRootOrTwoThirdsTechnicalCommittee;
	type NetworkContractSource = NetworkContractSource;
	type DeveloperDeposit = DeveloperDeposit;
	type DeploymentFee = DeploymentFee;
	type TreasuryAccount = TreasuryModuleAccount;
	type FreeDeploymentOrigin = EnsureRootOrHalfGeneralCouncil;
	type WeightInfo = weights::evm::WeightInfo<Runtime>;

	#[cfg(feature = "with-ethereum-compatibility")]
	fn config() -> &'static evm::Config {
		&ISTANBUL_CONFIG
	}
}

impl setheum_evm_bridge::Config for Runtime {
	type EVM = EVM;
}

#[cfg(feature = "standalone")]
pub use standalone_impl::*;

#[cfg(feature = "standalone")]
mod standalone_impl {
	use super::*;

	/// The BABE epoch configuration at genesis.
	pub const BABE_GENESIS_EPOCH_CONFIG: sp_consensus_babe::BabeEpochConfiguration =
		sp_consensus_babe::BabeEpochConfiguration {
			c: PRIMARY_PROBABILITY,
			allowed_slots: sp_consensus_babe::AllowedSlots::PrimaryAndSecondaryPlainSlots,
		};

	impl_opaque_keys! {
		pub struct SessionKeys {
			pub babe: Babe,
			pub grandpa: Grandpa,
		}
	}

	parameter_types! {
		pub const EpochDuration: u64 = EPOCH_DURATION_IN_SLOTS;
		pub const ExpectedBlockTime: Moment = MILLISECS_PER_BLOCK;
		pub const ReportLongevity: u64 =
			BondingDuration::get() as u64 * SessionsPerEra::get() as u64 *
	EpochDuration::get(); }

	impl pallet_babe::Config for Runtime {
		type EpochDuration = EpochDuration;
		type ExpectedBlockTime = ExpectedBlockTime;
		type EpochChangeTrigger = pallet_babe::ExternalTrigger;
		type KeyOwnerProofSystem = Historical;
		type KeyOwnerProof =
			<Self::KeyOwnerProofSystem as KeyOwnerProofSystem<(KeyTypeId, pallet_babe::AuthorityId)>>::Proof;
		type KeyOwnerIdentification = <Self::KeyOwnerProofSystem as KeyOwnerProofSystem<(
			KeyTypeId,
			pallet_babe::AuthorityId,
		)>>::IdentificationTuple;
		type HandleEquivocation = pallet_babe::EquivocationHandler<Self::KeyOwnerIdentification, (), ReportLongevity>;
		type WeightInfo = ();
	}

	impl pallet_grandpa::Config for Runtime {
		type Event = Event;
		type Call = Call;

		type KeyOwnerProofSystem = Historical;

		type KeyOwnerProof = <Self::KeyOwnerProofSystem as KeyOwnerProofSystem<(KeyTypeId, GrandpaId)>>::Proof;

		type KeyOwnerIdentification =
			<Self::KeyOwnerProofSystem as KeyOwnerProofSystem<(KeyTypeId, GrandpaId)>>::IdentificationTuple;

		type HandleEquivocation =
			pallet_grandpa::EquivocationHandler<Self::KeyOwnerIdentification, (), ReportLongevity>; // Offences

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
		pub const BondingDuration: pallet_staking::EraIndex = 4; // 12 hours
		pub const SlashDeferDuration: pallet_staking::EraIndex = 2; // 6 hours
		pub const RewardCurve: &'static PiecewiseLinear<'static> = &REWARD_CURVE;
		pub const MaxNominatorRewardedPerValidator: u32 = 64;
		pub const ElectionLookahead: BlockNumber = EPOCH_DURATION_IN_BLOCKS / 4;
		pub const MaxIterations: u32 = 5;
		// 0.05%. The higher the value, the more strict solution acceptance becomes.
		pub MinSolutionScoreBump: Perbill = Perbill::from_rational(5u32, 10_000);
	}

	impl pallet_staking::Config for Runtime {
		type Currency = Balances;
		type UnixTime = Timestamp;
		type CurrencyToVote = U128CurrencyToVote;
		type RewardRemainder = SetheumTreasury;
		type Event = Event;
		type Slash = SetheumTreasury; // send the slashed funds to the pallet treasury.
		type Reward = (); // rewards are minted from the void
		type SessionsPerEra = SessionsPerEra;
		type BondingDuration = BondingDuration;
		type SlashDeferDuration = SlashDeferDuration;
		/// A super-majority of the council can cancel the slash.
		type SlashCancelOrigin = EnsureRootOrThreeFourthsGeneralCouncil;
		type SessionInterface = Self;
		type EraPayout = pallet_staking::ConvertCurve<RewardCurve>;
		type NextNewSession = Session;
		type MaxNominatorRewardedPerValidator = MaxNominatorRewardedPerValidator;
		type WeightInfo = ();
		type ElectionProvider = ElectionProviderMultiPhase;
	}

	parameter_types! {
		pub const SessionDuration: BlockNumber = EPOCH_DURATION_IN_SLOTS as _;
		pub const ImOnlineUnsignedPriority: TransactionPriority = TransactionPriority::max_value();
		/// We prioritize im-online heartbeats over election solution submission.
		pub const StakingUnsignedPriority: TransactionPriority = TransactionPriority::max_value() / 2;
	}

	parameter_types! {
		// phase durations. 1/4 of the last session for each.
		pub const SignedPhase: u32 = EPOCH_DURATION_IN_BLOCKS / 4;
		pub const UnsignedPhase: u32 = EPOCH_DURATION_IN_BLOCKS / 4;

		// fallback: no need to do on-chain phragmen initially.
		pub const Fallback: pallet_election_provider_multi_phase::FallbackStrategy =
			pallet_election_provider_multi_phase::FallbackStrategy::Nothing;

		pub SolutionImprovementThreshold: Perbill = Perbill::from_rational(1u32, 10_000);

		// miner configs
		pub const MultiPhaseUnsignedPriority: TransactionPriority = StakingUnsignedPriority::get() - 1u64;
		pub const MinerMaxIterations: u32 = 10;
		pub MinerMaxWeight: Weight = RuntimeBlockWeights::get()
			.get(DispatchClass::Normal)
			.max_extrinsic.expect("Normal extrinsics have a weight limit configured; qed")
			.saturating_sub(BlockExecutionWeight::get());
	}

	impl pallet_election_provider_multi_phase::Config for Runtime {
		type Event = Event;
		type Currency = Balances;
		type SignedPhase = SignedPhase;
		type UnsignedPhase = UnsignedPhase;
		type SolutionImprovementThreshold = MinSolutionScoreBump;
		type MinerMaxIterations = MinerMaxIterations;
		type MinerMaxWeight = MinerMaxWeight;
		type MinerTxPriority = MultiPhaseUnsignedPriority;
		type DataProvider = Staking;
		type OnChainAccuracy = Perbill;
		type CompactSolution = pallet_staking::CompactAssignments;
		type Fallback = Fallback;
		type WeightInfo = pallet_election_provider_multi_phase::weights::SubstrateWeight<Runtime>;
		type BenchmarkingConfig = ();
	}
}

#[cfg(not(feature = "standalone"))]
pub use parachain_impl::*;
#[cfg(not(feature = "standalone"))]
mod parachain_impl {
	use super::*;

	impl cumulus_pallet_parachain_system::Config for Runtime {
		type Event = Event;
		type OnValidationData = ();
		type SelfParaId = parachain_info::Pallet<Runtime>;
		type DownwardMessageHandlers = XcmHandler;
		type HrmpMessageHandlers = XcmHandler;
	}

	impl parachain_info::Config for Runtime {}

	parameter_types! {
		pub const PolkadotNetworkId: NetworkId = NetworkId::Polkadot;
	}

	pub struct AccountId32Convert;
	impl Convert<AccountId, [u8; 32]> for AccountId32Convert {
		fn convert(account_id: AccountId) -> [u8; 32] {
			account_id.into()
		}
	}

	parameter_types! {
		pub SetheumNetwork: NetworkId = NetworkId::Named("setheum".into());
		pub RelayChainOrigin: Origin = cumulus_pallet_xcm_handler::Origin::Relay.into();
		pub Ancestry: MultiLocation = MultiLocation::X1(Junction::Parachain {
			id: ParachainInfo::get().into(),
		});
		pub const RelayChainCurrencyId: CurrencyId = CurrencyId::Token(TokenSymbol::DOT);
	}

	pub type LocationConverter = (
		ParentIsDefault<AccountId>,
		SiblingParachainConvertsVia<Sibling, AccountId>,
		AccountId32Aliases<SetheumNetwork, AccountId>,
	);

	pub type LocalAssetTransactor = MultiCurrencyAdapter<
		Currencies,
		UnknownTokens,
		IsConcreteWithGeneralKey<CurrencyId, Identity>,
		LocationConverter,
		AccountId,
		CurrencyIdConverter<CurrencyId, RelayChainCurrencyId>,
		CurrencyId,
	>;

	pub type LocalOriginConverter = (
		SovereignSignedViaLocation<LocationConverter, Origin>,
		RelayChainAsNative<RelayChainOrigin, Origin>,
		SiblingParachainAsNative<cumulus_pallet_xcm_handler::Origin, Origin>,
		SignedAccountId32AsNative<SetheumNetwork, Origin>,
	);

	parameter_types! {
		pub NativeOrmlTokens: BTreeSet<(Vec<u8>, MultiLocation)> = {
			let mut t = BTreeSet::new();
			//TODO: might need to add other assets based on orml-tokens

			// Plasm
			t.insert(("SDN".into(), (Junction::Parent, Junction::Parachain { id: 5000 }).into()));
			// Plasm
			t.insert(("PLM".into(), (Junction::Parent, Junction::Parachain { id: 5000 }).into()));

			// Hydrate
			t.insert(("HDT".into(), (Junction::Parent, Junction::Parachain { id: 82406 }).into()));

			// KILT
			t.insert(("KILT".into(), (Junction::Parent, Junction::Parachain { id: 12623 }).into()));

			t
		};
	}

	pub struct XcmConfig;
	impl Config for XcmConfig {
		type Call = Call;
		type XcmSender = XcmHandler;
		type AssetTransactor = LocalAssetTransactor;
		type OriginConverter = LocalOriginConverter;
		//TODO: might need to add other assets based on orml-tokens
		type IsReserve = NativePalletAssetOr<NativeOrmlTokens>;
		type IsTeleporter = ();
		type LocationInverter = LocationInverter<Ancestry>;
	}

	impl cumulus_pallet_xcm_handler::Config for Runtime {
		type Event = Event;
		type XcmExecutor = XcmExecutor<XcmConfig>;
		type UpwardMessageSender = ParachainSystem;
		type HrmpMessageSender = ParachainSystem;
		type SendXcmOrigin = EnsureRoot<AccountId>;
		type AccountIdConverter = LocationConverter;
	}

	pub struct HandleXcm;
	impl XcmHandlerT<AccountId> for HandleXcm {
		fn execute_xcm(origin: AccountId, xcm: Xcm) -> DispatchResult {
			XcmHandler::execute_xcm(origin, xcm)
		}
	}

	impl orml_xtokens::Config for Runtime {
		type Event = Event;
		type Balance = Balance;
		type ToRelayChainBalance = Identity;
		type AccountId32Convert = AccountId32Convert;
		//TODO: change network id if kusama
		type RelayChainNetworkId = PolkadotNetworkId;
		type ParaId = ParachainInfo;
		type XcmHandler = HandleXcm;
	}

	impl orml_unknown_tokens::Config for Runtime {
		type Event = Event;
	}
}

macro_rules! construct_newrome_runtime {
	($( $modules:tt )*) => {
		#[allow(clippy::large_enum_variant)]
		construct_runtime! {
			pub enum Runtime where
				Block = Block,
				NodeBlock = primitives::Block,
				UncheckedExtrinsic = UncheckedExtrinsic
			{
				// Core
				System: frame_system::{Pallet, Call, Storage, Config, Event<T>} = 0,
				Timestamp: pallet_timestamp::{Pallet, Call, Storage, Inherent} = 1,
				RandomnessCollectiveFlip: pallet_randomness_collective_flip::{Pallet, Call, Storage} = 2,

				// Tokens & Related
				Balances: pallet_balances::{Pallet, Call, Storage, Config<T>, Event<T>} = 3,

				TransactionPayment: module_transaction_payment::{Pallet, Call, Storage} = 4,
				SevmAccounts: evm_accounts::{Pallet, Call, Storage, Event<T>} = 5,
				Currencies: setheum_currencies::{Pallet, Call, Event<T>} = 6,
				Tokens: orml_tokens::{Pallet, Storage, Event<T>, Config<T>} = 7,
				Vesting: orml_vesting::{Pallet, Storage, Call, Event<T>, Config<T>} = 8,

				SetheumTreasury: pallet_treasury::{Pallet, Call, Storage, Config, Event<T>} = 9,
				Bounties: pallet_bounties::{Pallet, Call, Storage, Event<T>} = 10,
				Tips: pallet_tips::{Pallet, Call, Storage, Event<T>} = 11,

				// Utility
				Utility: pallet_utility::{Pallet, Call, Event} = 12,
				Multisig: pallet_multisig::{Pallet, Call, Storage, Event<T>} = 13,
				Recovery: pallet_recovery::{Pallet, Call, Storage, Event<T>} = 14,
				Proxy: pallet_proxy::{Pallet, Call, Storage, Event<T>} = 15,
				Scheduler: pallet_scheduler::{Pallet, Call, Storage, Event<T>} = 16,

				Indices: pallet_indices::{Pallet, Call, Storage, Config<T>, Event<T>} = 17,
				GraduallyUpdate: orml_gradually_update::{Pallet, Storage, Call, Event<T>} = 18,

				// Governance
				GeneralCouncil: pallet_collective::<Instance1>::{Pallet, Call, Storage, Origin<T>, Event<T>, Config<T>} = 19,
				GeneralCouncilMembership: pallet_membership::<Instance1>::{Pallet, Call, Storage, Event<T>, Config<T>} = 20,
				TechnicalCommittee: pallet_collective::<Instance4>::{Pallet, Call, Storage, Origin<T>, Event<T>, Config<T>} = 25,
				TechnicalCommitteeMembership: pallet_membership::<Instance4>::{Pallet, Call, Storage, Event<T>, Config<T>} = 26,

				Authority: orml_authority::{Pallet, Call, Event<T>, Origin<T>} = 27,
				ElectionsPhragmen: pallet_elections_phragmen::{Pallet, Call, Storage, Event<T>} = 28,

				// Oracle
				SetheumOracle: orml_oracle::<Instance1>::{Pallet, Storage, Call, Config<T>, Event<T>} = 29,
				BandOracle: orml_oracle::<Instance2>::{Pallet, Storage, Call, Config<T>, Event<T>} = 30,
				// OperatorMembership must be placed after Oracle or else will have race condition on initialization
				OperatorMembershipsetheum: pallet_membership::<Instance5>::{Pallet, Call, Storage, Event<T>, Config<T>} = 31,
				OperatorMembershipBand: pallet_membership::<Instance6>::{Pallet, Call, Storage, Event<T>, Config<T>} = 32,

				// ORML Core
				Rewards: orml_rewards::{Pallet, Storage, Call} = 34,
				OrmlNFT: orml_nft::{Pallet, Storage, Config<T>} = 35,

				// setheum Core
				Prices: setheum_prices::{Pallet, Storage, Call, Event<T>} = 36,

				// SetheumDex
				DEX = setheum_dex::{Pallet, Storage, Call, Event<T>, Config<T>} = 37,

				// Setheum Other
				Incentives: setheum_incentives::{Pallet, Storage, Call, Event<T>} = 49,
				AirDrop: module_airdrop::{Pallet, Call, Storage, Event<T>, Config<T>} = 50,
				NFT: setheum_nft::{Pallet, Call, Event<T>} = 51,

				// Smart contracts
				EVM: evm::{Pallet, Config<T>, Call, Storage, Event<T>} = 53,
				EVMBridge: setheum_setheum_evm_bridge::{Pallet} = 54,

				// Dev
				Sudo: pallet_sudo::{Pallet, Call, Config<T>, Storage, Event<T>} = 55,

				$($modules)*
			}
		}
	}
}

#[cfg(feature = "standalone")]
construct_newrome_runtime! {
	// Consensus & Staking
	Authorship: pallet_authorship::{Pallet, Call, Storage, Inherent} = 56,
	Babe: pallet_babe::{Pallet, Call, Storage, Config, ValidateUnsigned} = 57,
	Grandpa: pallet_grandpa::{Pallet, Call, Storage, Config, Event, ValidateUnsigned} = 58,
	ElectionProviderMultiPhase: pallet_election_provider_multi_phase::{Pallet, Call, Storage, Event<T>, ValidateUnsigned} = 59,
	Staking: pallet_staking::{Pallet, Call, Config<T>, Storage, Event<T>} = 60,
	Session: pallet_session::{Pallet, Call, Storage, Event, Config<T>} = 61,
	Historical: pallet_session_historical::{Pallet} = 62,
}

#[cfg(not(feature = "standalone"))]
construct_newrome_runtime! {
	// Parachain
	ParachainSystem: cumulus_pallet_parachain_system::{Pallet, Call, Storage, Inherent, Event} = 56,
	ParachainInfo: parachain_info::{Pallet, Storage, Config} = 57,
	XcmHandler: cumulus_pallet_xcm_handler::{Pallet, Call, Event<T>, Origin} = 58,
	XTokens: orml_xtokens::{Pallet, Storage, Call, Event<T>} = 59,
	UnknownTokens: orml_unknown_tokens::{Pallet, Storage, Event} = 60,
}

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
	evm::SetEvmOrigin<Runtime>,
);
/// Unchecked extrinsic type as expected by this runtime.
pub type UncheckedExtrinsic = generic::UncheckedExtrinsic<Address, Call, Signature, SignedExtra>;
/// The payload being signed in transactions.
pub type SignedPayload = generic::SignedPayload<Call, SignedExtra>;
/// Extrinsic type that has already been checked.
pub type CheckedExtrinsic = generic::CheckedExtrinsic<AccountId, Call, SignedExtra>;
/// Executive: handles dispatch to the various modules.
pub type Executive =
	frame_executive::Executive<Runtime, Block, frame_system::ChainContext<Runtime>, Runtime, AllPallets>;

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
			RandomnessCollectiveFlip::random_seed().0
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

	#[cfg(feature = "standalone")]
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

	#[cfg(feature = "standalone")]
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
		fn query_info(uxt: <Block as BlockT>::Extrinsic, len: u32) -> RuntimeDispatchInfo<Balance> {
			TransactionPayment::query_info(uxt, len)
		}
		fn query_fee_details(uxt: <Block as BlockT>::Extrinsic, len: u32) -> FeeDetails<Balance> {
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
				DataProviderId::setheum => SetheumOracle::get_no_op(&key),
				DataProviderId::Band => BandOracle::get_no_op(&key),
				DataProviderId::Aggregated => <AggregatedDataProvider as DataProviderExtended<_, _>>::get_no_op(&key)
			}
		}

		fn get_all_values(provider_id: DataProviderId) -> Vec<(CurrencyId, Option<TimeStampedPrice>)> {
			match provider_id {
				DataProviderId::setheum => SetheumOracle::get_all_values(),
				DataProviderId::Band => BandOracle::get_all_values(),
				DataProviderId::Aggregated => <AggregatedDataProvider as DataProviderExtended<_, _>>::get_all_values()
			}
		}
	}

	impl evm_rpc_runtime_api::EVMRuntimeRPCApi<Block, Balance> for Runtime {
		fn call(
			from: H160,
			to: H160,
			data: Vec<u8>,
			value: Balance,
			gas_limit: u32,
			storage_limit: u32,
			estimate: bool,
		) -> Result<CallInfo, sp_runtime::DispatchError> {
			let config = if estimate {
				let mut config = <Runtime as evm::Config>::config().clone();
				config.estimate = true;
				Some(config)
			} else {
				None
			};

			evm::Runner::<Runtime>::call(
				from,
				from,
				to,
				data,
				value,
				gas_limit.into(),
				storage_limit,
				config.as_ref().unwrap_or(<Runtime as evm::Config>::config()),
			)
		}

		fn create(
			from: H160,
			data: Vec<u8>,
			value: Balance,
			gas_limit: u32,
			storage_limit: u32,
			estimate: bool,
		) -> Result<CreateInfo, sp_runtime::DispatchError> {
			let config = if estimate {
				let mut config = <Runtime as evm::Config>::config().clone();
				config.estimate = true;
				Some(config)
			} else {
				None
			};

			evm::Runner::<Runtime>::create(
				from,
				data,
				value,
				gas_limit.into(),
				storage_limit,
				config.as_ref().unwrap_or(<Runtime as evm::Config>::config()),
			)
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

			use setheum_nft::benchmarking::Pallet as NftBench;
			impl setheum_nft::benchmarking::Config for Runtime {}

			let whitelist: Vec<TrackedStorageKey> = vec![
				// Block Number
				// frame_system::Number::<Runtime>::hashed_key().to_vec(),
				hex_literal::hex!("26aa394eea5630e07c48ae0c9558cef702a5c1b19ab7a04f536c519DNAR4983ac").to_vec().into(),
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

			add_benchmark!(params, batches, setheum_nft, NftBench::<Runtime>);
			orml_add_benchmark!(params, batches, setheum_dex, benchmarking::dex);
			orml_add_benchmark!(params, batches, evm, benchmarking::evm);
			orml_add_benchmark!(params, batches, module_transaction_payment, benchmarking::transaction_payment);
			orml_add_benchmark!(params, batches, setheum_incentives, benchmarking::incentives);
			orml_add_benchmark!(params, batches, setheum_prices, benchmarking::prices);
			orml_add_benchmark!(params, batches, evm_accounts, benchmarking::evm_accounts);
			orml_add_benchmark!(params, batches, setheum_currencies, benchmarking::currencies);

			orml_add_benchmark!(params, batches, orml_tokens, benchmarking::tokens);
			orml_add_benchmark!(params, batches, orml_vesting, benchmarking::vesting);

			orml_add_benchmark!(params, batches, orml_authority, benchmarking::authority);
			orml_add_benchmark!(params, batches, orml_gradually_update, benchmarking::gradually_update);
			orml_add_benchmark!(params, batches, orml_oracle, benchmarking::oracle);

			if batches.is_empty() { return Err("Benchmark not found for this module.".into()) }
			Ok(batches)
		}
	}
}

#[cfg(not(feature = "standalone"))]
cumulus_pallet_parachain_system::register_validate_block!(Runtime, Executive);

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
}

#[test]
fn transfer() {
	let t = Call::System(frame_system::Call::remark(vec![1, 2, 3])).encode();
	println!("t: {:?}", t);
}
