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
	traits::{
		AccountIdConversion, BadOrigin, BlakeTwo256, Block as BlockT, Convert, Identity, SaturatedConversion,
		StaticLookup, Zero,
	},
	transaction_validity::{TransactionSource, TransactionValidity},
	ApplyExtrinsicResult, DispatchResult, FixedPointNumber, ModuleId,
};
use sp_std::prelude::*;
#[cfg(feature = "std")]
use sp_version::NativeVersion;
use sp_version::RuntimeVersion;

use frame_system::{EnsureOneOf, EnsureRoot, RawOrigin};
use module_currencies::{BasicCurrencyAdapter, Currency};
use sevm::{CallInfo, CreateInfo};
use sevm_accounts::EvmAddressMapping;
use module_transaction_payment::{Multiplier, TargetedFeeAdjustment};
use orml_tokens::CurrencyAdapter;
use orml_traits::{create_median_value_data_provider, parameter_type_with_key, DataFeeder, DataProviderExtended};
use pallet_transaction_payment::RuntimeDispatchInfo;

use orml_xcm_support::{
	CurrencyIdConverter, IsConcreteWithGeneralKey, MultiCurrencyAdapter, XcmHandler as XcmHandlerT,
};
use polkadot_parachain::primitives::Sibling;
use xcm::v0::{Junction, MultiLocation, NetworkId, Xcm};
use xcm_builder::{
	AccountId32Aliases, LocationInverter, ParentIsDefault, RelayChainAsNative, SiblingParachainAsNative,
	SiblingParachainConvertsVia, SignedAccountId32AsNative, SovereignSignedViaLocation,
};
use xcm_executor::{traits::NativeAsset, Config, XcmExecutor};

/// Weights for pallets used in the runtime.
mod weights;

pub use frame_support::{
	construct_runtime, log, parameter_types,
	traits::{
		Contains, ContainsLengthBound, EnsureOrigin, Filter, Get, IsType, KeyOwnerProofSystem, LockIdentifier,
		Randomness, U128CurrencyToVote,
	},
	weights::{constants::RocksDbWeight, IdentityFee, Weight},
	StorageValue,
};

pub use pallet_timestamp::Call as TimestampCall;
#[cfg(any(feature = "std", test))]
pub use sp_runtime::BuildStorage;
pub use sp_runtime::{Perbill, Percent, Permill, Perquintill};

pub use authority::AuthorityConfigImpl;
pub use constants::{fee::*, time::*};
pub use primitives::{
	AccountId, AccountIndex, Amount, AuthoritysOriginId, Balance, BlockNumber, CurrencyId, DataProviderId,
	EraIndex, Hash, Moment, Nonce, Share, Signature, TokenSymbol, TradingPair,
};
pub use runtime_common::{
	cent, deposit, dollar, microcent, millicent, CurveFeeModel, ExchangeRate, GasToWeight, OffchainSolutionWeightLimit,
	Price, Rate, Ratio, RuntimeBlockLength, RuntimeBlockWeights, SystemContractsFilter, TimeStampedPrice, DNAR, JUSD,
	DOT, LDOT, PHA, POLKABTC, RENBTC, SDN, XBTC,
};

mod authority;
mod constants;

/// This runtime version.
pub const VERSION: RuntimeVersion = RuntimeVersion {
	spec_name: create_runtime_str!("setheum"),
	impl_name: create_runtime_str!("setheum"),
	authoring_version: 1,
	spec_version: 100,
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
	pub struct SessionKeys {}
}

// Pallet accounts of runtime
parameter_types! {
	pub const SetheumTreasuryModuleId: ModuleId = ModuleId(*b"dnr/trsy");
	pub const SettinDEXModuleId: ModuleId = ModuleId(*b"dnr/sdex");
	pub const CDPTreasuryModuleId: ModuleId = ModuleId(*b"dnr/cdpt");
	pub const StakingPoolModuleId: ModuleId = ModuleId(*b"dnr/stkp");
	pub const AbhaTreasuryModuleId: ModuleId = ModuleId(*b"dnr/hmtr");
	// Decentralized Sovereign Wealth Fund
	pub const DSWFModuleId: ModuleId = ModuleId(*b"dnr/dswf");
	pub const ElectionsPhragmenModuleId: LockIdentifier = *b"dnr/phre";
	pub const NftModuleId: ModuleId = ModuleId(*b"dnr/sNFT");
}

pub fn get_all_module_accounts() -> Vec<AccountId> {
	vec![
		SetheumTreasuryModuleId::get().into_account(),
		SettinDEXModuleId::get().into_account(),
		CDPTreasuryModuleId::get().into_account(),
		StakingPoolModuleId::get().into_account(),
		AbhaTreasuryModuleId::get().into_account(),
		DSWFModuleId::get().into_account(),
		ZeroAccountId::get(),
	]
}

parameter_types! {
	pub const BlockHashCount: BlockNumber = 900; // mortal tx can be valid up to 1 hour after signing
	pub const Version: RuntimeVersion = VERSION;
	pub const SS58Prefix: u8 = 10; // Ss58AddressFormat::setheumAccount
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
	type BlockWeights = RuntimeBlockWeights;
	type BlockLength = RuntimeBlockLength;
	type Version = Version;
	type PalletInfo = PalletInfo;
	type AccountData = pallet_balances::AccountData<Balance>;
	type OnNewAccount = ();
	type OnKilledAccount = (
		sevm::CallKillAccount<Runtime>,
		sevm_accounts::CallKillAccount<Runtime>,
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

type EnsureRootOrHalfAbhaCouncil = EnsureOneOf<
	AccountId,
	EnsureRoot<AccountId>,
	pallet_collective::EnsureProportionMoreThan<_1, _2, AccountId, AbhaCouncilInstance>,
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
	pub const AbhaCouncilMotionDuration: BlockNumber = 7 * DAYS;
	pub const AbhaCouncilMaxProposals: u32 = 100;
	pub const AbhaCouncilMaxMembers: u32 = 100;
}

type AbhaCouncilInstance = pallet_collective::Instance3;
impl pallet_collective::Config<AbhaCouncilInstance> for Runtime {
	type Origin = Origin;
	type Proposal = Call;
	type Event = Event;
	type MotionDuration = AbhaCouncilMotionDuration;
	type MaxProposals = AbhaCouncilMaxProposals;
	type MaxMembers = AbhaCouncilMaxMembers;
	type DefaultVote = pallet_collective::PrimeDefaultVote;
	type WeightInfo = ();
}

type AbhaCouncilMembershipInstance = pallet_membership::Instance3;
impl pallet_membership::Config<AbhaCouncilMembershipInstance> for Runtime {
	type Event = Event;
	type AddOrigin = EnsureRootOrTwoThirdsGeneralCouncil;
	type RemoveOrigin = EnsureRootOrTwoThirdsGeneralCouncil;
	type SwapOrigin = EnsureRootOrTwoThirdsGeneralCouncil;
	type ResetOrigin = EnsureRootOrTwoThirdsGeneralCouncil;
	type PrimeOrigin = EnsureRootOrTwoThirdsGeneralCouncil;
	type MembershipInitialized = AbhaCouncil;
	type MembershipChanged = AbhaCouncil;
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
	type MembershipInitialized = setheumOracle;
	type MembershipChanged = setheumOracle;
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
	pub ConfigDepositBase: Balance =  10 * dollar(DNAR);
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
	pub CandidacyBond: Balance = 10 * dollar(LDOT);
	pub VotingBondBase: Balance = 2 * dollar(LDOT);
	pub VotingBondFactor: Balance = dollar(LDOT);
	pub const TermDuration: BlockNumber = 7 * DAYS;
	pub const DesiredMembers: u32 = 13;
	pub const DesiredRunnersUp: u32 = 7;
}

impl pallet_elections_phragmen::Config for Runtime {
	type ModuleId = ElectionsPhragmenModuleId;
	type Event = Event;
	type Currency = CurrencyAdapter<Runtime, GetLiquidCurrencyId>;
	type CurrencyToVote = U128CurrencyToVote;
	type ChangeMembers = AbhaCouncil;
	type InitializeMembers = AbhaCouncil;
	type CandidacyBond = CandidacyBond;
	type VotingBondBase = VotingBondBase;
	type VotingBondFactor = VotingBondFactor;
	type TermDuration = TermDuration;
	type DesiredMembers = DesiredMembers;
	type DesiredRunnersUp = DesiredRunnersUp;
	type LoserCandidate = ();
	type KickedMember = ();
	type WeightInfo = ();
}

parameter_types! {
	pub const MinimumCount: u32 = 1;
	pub const ExpiresIn: Moment = 1000 * 60 * 60; // 60 mins
	pub ZeroAccountId: AccountId = AccountId::from([0u8; 32]);
}

type setheumDataProvider = orml_oracle::Instance1;
impl orml_oracle::Config<setheumDataProvider> for Runtime {
	type Event = Event;
	type OnNewData = ();
	type CombineData = orml_oracle::DefaultCombineData<Runtime, MinimumCount, ExpiresIn, setheumDataProvider>;
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
	[setheumOracle, BandOracle]
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

impl module_prices::Config for Runtime {
	type Event = Event;
	type Source = AggregatedDataProvider;
	type GetStableCurrencyId = GetStableCurrencyId;
	type StableCurrencyFixedPrice = StableCurrencyFixedPrice;
	type GetStakingCurrencyId = GetStakingCurrencyId;
	type GetLiquidCurrencyId = GetLiquidCurrencyId;
	type LockOrigin = EnsureRootOrTwoThirdsGeneralCouncil;
	type LiquidStakingExchangeRateProvider = LiquidStakingExchangeRateProvider;
	type SettinDex = SettinDex;
	type Currency = Currencies;
	type WeightInfo = weights::module_prices::WeightInfo<Runtime>;
}

pub struct LiquidStakingExchangeRateProvider;
impl module_support::ExchangeRateProvider for LiquidStakingExchangeRateProvider {
	fn get_exchange_rate() -> ExchangeRate {
		StakingPool::liquid_exchange_rate()
	}
}

parameter_types! {
	pub const GetNativeCurrencyId: CurrencyId = DNAR;
	pub const GetStableCurrencyId: CurrencyId = JUSD;
}

impl module_currencies::Config for Runtime {
	type Event = Event;
	type MultiCurrency = Tokens;
	type NativeCurrency = BasicCurrencyAdapter<Runtime, Balances, Amount, BlockNumber>;
	type GetNativeCurrencyId = GetNativeCurrencyId;
	type WeightInfo = weights::module_currencies::WeightInfo<Runtime>;
	type AddressMapping = EvmAddressMapping<Runtime>;
	type SEVMBridge = SEVMBridge;
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
	pub MaximumSchedulerWeight: Weight = Perbill::from_percent(10) *
		RuntimeBlockWeights::get().max_block;
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
			sevm::SetEvmOrigin::<Runtime>::new(),
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
		TradingPair::new(JUSD, LDOT),
		TradingPair::new(JUSD, XBTC),
		TradingPair::new(JUSD, RENBTC),
		TradingPair::new(JUSD, POLKABTC),
		TradingPair::new(JUSD, PHA),
	];
}

impl settindex::Config for Runtime {
	type Event = Event;
	type Currency = Currencies;
	type GetExchangeFee = GetExchangeFee;
	type TradingPathLimit = TradingPathLimit;
	type ModuleId = SettinDEXModuleId;
	type SettinDexIncentives = Incentives;
	type WeightInfo = weights::settindex::WeightInfo<Runtime>;
	type ListingOrigin = EnsureRootOrHalfGeneralCouncil;
}

parameter_types! {
	// All currency types except for native currency, Sort by fee charge order
	pub AllNonNativeCurrencyIds: Vec<CurrencyId> = vec![JUSD, LDOT, DOT, XBTC, RENBTC, POLKABTC, PHA];
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
	type SettinDex = SettinDex;
	type MaxSlippageSwapWithDEX = MaxSlippageSwapWithDEX;
	type WeightInfo = weights::module_transaction_payment::WeightInfo<Runtime>;
}

impl sevm_accounts::Config for Runtime {
	type Event = Event;
	type Currency = Balances;
	type AddressMapping = EvmAddressMapping<Runtime>;
	type MergeAccount = Currencies;
	type WeightInfo = weights::sevm_accounts::WeightInfo<Runtime>;
}

impl orml_rewards::Config for Runtime {
	type Share = Balance;
	type Balance = Balance;
	type PoolId = module_incentives::PoolId<AccountId>;
	type Handler = Incentives;
}

parameter_types! {
	pub const AccumulatePeriod: BlockNumber = MINUTES;
}

impl module_incentives::Config for Runtime {
	type Event = Event;
	type RelaychainAccountId = AccountId;
	type RewardsVaultAccountId = ZeroAccountId;
	type NativeCurrencyId = GetNativeCurrencyId;
	type StableCurrencyId = GetStableCurrencyId;
	type LiquidCurrencyId = GetLiquidCurrencyId;
	type AccumulatePeriod = AccumulatePeriod;
	type UpdateOrigin = EnsureRootOrHalfSerpCouncil;
	type CDPTreasury = CdpTreasury;
	type Currency = Currencies;
	type SettinDex = SettinDex;
	type EmergencyShutdown = EmergencyShutdown;
	type ModuleId = IncentivesModuleId;
	type WeightInfo = weights::module_incentives::WeightInfo<Runtime>;
}

parameter_types! {
	pub const PolkadotBondingDuration: EraIndex = 7;
	pub const EraLength: BlockNumber = DAYS;
}

impl module_polkadot_bridge::Config for Runtime {
	type DOTCurrency = Currency<Runtime, GetStakingCurrencyId>;
	type OnNewEra = (NomineesElection, StakingPool);
	type BondingDuration = PolkadotBondingDuration;
	type EraLength = EraLength;
	type PolkadotAccountId = AccountId;
}

parameter_types! {
	pub const GetLiquidCurrencyId: CurrencyId = LDOT;
	pub const GetStakingCurrencyId: CurrencyId = DOT;
	pub DefaultExchangeRate: ExchangeRate = ExchangeRate::saturating_from_rational(10, 100);	// 1 : 10
	pub PoolAccountIndexes: Vec<u32> = vec![1, 2, 3, 4];
}

impl module_staking_pool::Config for Runtime {
	type Event = Event;
	type StakingCurrencyId = GetStakingCurrencyId;
	type LiquidCurrencyId = GetLiquidCurrencyId;
	type DefaultExchangeRate = DefaultExchangeRate;
	type ModuleId = StakingPoolModuleId;
	type PoolAccountIndexes = PoolAccountIndexes;
	type UpdateOrigin = EnsureRootOrHalfAbhaCouncil;
	type FeeModel = CurveFeeModel;
	type Nominees = NomineesElection;
	type Bridge = PolkadotBridge;
	type Currency = Currencies;
}

impl setheum_abha::Config for Runtime {
	type Homa = StakingPool;
	type WeightInfo = weights::setheum_abha::WeightInfo<Runtime>;
}

parameter_types! {
	pub MinCouncilBondThreshold: Balance = dollar(LDOT);
	pub const NominateesCount: u32 = 7;
	pub const MaxUnlockingChunks: u32 = 7;
	pub const NomineesElectionBondingDuration: EraIndex = 7;
}

impl module_nominees_election::Config for Runtime {
	type Currency = Currency<Runtime, GetLiquidCurrencyId>;
	type PolkadotAccountId = AccountId;
	type MinBondThreshold = MinCouncilBondThreshold;
	type BondingDuration = NomineesElectionBondingDuration;
	type NominateesCount = NominateesCount;
	type MaxUnlockingChunks = MaxUnlockingChunks;
	type RelaychainValidatorFilter = runtime_common::RelaychainValidatorFilter;
}

parameter_types! {
	pub MinGuaranteeAmount: Balance = dollar(LDOT);
	pub const ValidatorInsuranceThreshold: Balance = 0;
}

impl setheum_abha_validator_list::Config for Runtime {
	type Event = Event;
	type RelaychainAccountId = AccountId;
	type LiquidTokenCurrency = Currency<Runtime, GetLiquidCurrencyId>;
	type MinBondAmount = MinGuaranteeAmount;
	type BondingDuration = PolkadotBondingDuration;
	type ValidatorInsuranceThreshold = ValidatorInsuranceThreshold;
	type FreezeOrigin = EnsureRootOrHalfAbhaCouncil;
	type SlashOrigin = EnsureRootOrHalfAbhaCouncil;
	type OnSlash = module_staking_pool::OnSlash<Runtime>;
	type LiquidStakingExchangeRateProvider = LiquidStakingExchangeRateProvider;
	type WeightInfo = ();
	type OnIncreaseGuarantee = module_incentives::OnIncreaseGuarantee<Runtime>;
	type OnDecreaseGuarantee = module_incentives::OnDecreaseGuarantee<Runtime>;
}

parameter_types! {
	pub CreateClassDeposit: Balance = 500 * millicent(DNAR);
	pub CreateTokenDeposit: Balance = 100 * millicent(DNAR);
}

impl module_nft::Config for Runtime {
	type Event = Event;
	type CreateClassDeposit = CreateClassDeposit;
	type CreateTokenDeposit = CreateTokenDeposit;
	type ModuleId = NftModuleId;
	type Currency = Currency<Runtime, GetNativeCurrencyId>;
	type WeightInfo = weights::module_nft::WeightInfo<Runtime>;
}

impl orml_nft::Config for Runtime {
	type ClassId = u32;
	type TokenId = u64;
	type ClassData = module_nft::ClassData;
	type TokenData = module_nft::TokenData;
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
	// TODO: update
	pub const ChainId: u64 = 787;
	pub const NewContractExtraBytes: u32 = 10_000;
	pub StorageDepositPerByte: Balance = microcent(DNAR);
	pub const MaxCodeSize: u32 = 60 * 1024;
	pub NetworkContractSource: H160 = H160::from_low_u64_be(0);
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

pub type SettinDexPrecompile = runtime_common::SettinDexPrecompile<AccountId, EvmAddressMapping<Runtime>, Dex>;

impl sevm::Config for Runtime {
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
		SettinDexPrecompile,
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
	type WeightInfo = weights::sevm::WeightInfo<Runtime>;
}

impl sevm_evm_bridge::Config for Runtime {
	type EVM = EVM;
}

impl cumulus_pallet_parachain_system::Config for Runtime {
	type Event = Event;
	type OnValidationData = ();
	type SelfParaId = ParachainInfo;
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
	pub setheumNetwork: NetworkId = NetworkId::Named("setheum".into());
	pub RelayChainOrigin: Origin = cumulus_pallet_xcm_handler::Origin::Relay.into();
	pub Ancestry: MultiLocation = MultiLocation::X1(Junction::Parachain {
		id: ParachainInfo::get().into(),
	});
	pub const RelayChainCurrencyId: CurrencyId = CurrencyId::Token(TokenSymbol::DOT);
}

pub type LocationConverter = (
	ParentIsDefault<AccountId>,
	SiblingParachainConvertsVia<Sibling, AccountId>,
	AccountId32Aliases<setheumNetwork, AccountId>,
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
	SignedAccountId32AsNative<setheumNetwork, Origin>,
);

pub struct XcmConfig;
impl Config for XcmConfig {
	type Call = Call;
	type XcmSender = XcmHandler;
	type AssetTransactor = LocalAssetTransactor;
	type OriginConverter = LocalOriginConverter;
	//TODO: might need to add other assets based on orml-tokens
	type IsReserve = NativeAsset;
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
		RandomnessCollectiveFlip: pallet_randomness_collective_flip::{Pallet, Call, Storage} = 2,

		// Tokens & Related
		Balances: pallet_balances::{Pallet, Call, Storage, Config<T>, Event<T>} = 3,

		TransactionPayment: module_transaction_payment::{Pallet, Call, Storage} = 4,
		EvmAccounts: sevm_accounts::{Pallet, Call, Storage, Event<T>} = 5,
		Currencies: module_currencies::{Pallet, Call, Event<T>} = 6,
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
		AbhaCouncil: pallet_collective::<Instance3>::{Pallet, Call, Storage, Origin<T>, Event<T>, Config<T>} = 23,
		AbhaCouncilMembership: pallet_membership::<Instance3>::{Pallet, Call, Storage, Event<T>, Config<T>} = 24,
		TechnicalCommittee: pallet_collective::<Instance4>::{Pallet, Call, Storage, Origin<T>, Event<T>, Config<T>} = 25,
		TechnicalCommitteeMembership: pallet_membership::<Instance4>::{Pallet, Call, Storage, Event<T>, Config<T>} = 26,

		Authority: orml_authority::{Pallet, Call, Event<T>, Origin<T>} = 27,
		ElectionsPhragmen: pallet_elections_phragmen::{Pallet, Call, Storage, Event<T>} = 28,

		// Oracle
		setheumOracle: orml_oracle::<Instance1>::{Pallet, Storage, Call, Config<T>, Event<T>} = 29,
		BandOracle: orml_oracle::<Instance2>::{Pallet, Storage, Call, Config<T>, Event<T>} = 30,
		// OperatorMembership must be placed after Oracle or else will have race condition on initialization
		OperatorMembershipsetheum: pallet_membership::<Instance5>::{Pallet, Call, Storage, Event<T>, Config<T>} = 31,
		OperatorMembershipBand: pallet_membership::<Instance6>::{Pallet, Call, Storage, Event<T>, Config<T>} = 32,

		// ORML Core
		Rewards: orml_rewards::{Pallet, Storage, Call} = 34,
		OrmlNFT: orml_nft::{Pallet, Storage, Config<T>} = 35,

		// setheum Core
		Prices: module_prices::{Pallet, Storage, Call, Event<T>} = 36,

		// DEX
		SettinDex: settindex::{Pallet, Storage, Call, Event<T>, Config<T>} = 37,

		// Serp
		Serp: module_serp::{Pallet, Storage, Call, Event<T>} = 40,
		CdpTreasury: module_cdp_treasury::{Pallet, Storage, Call, Config, Event<T>} = 41,
		CdpEngine: module_cdp_engine::{Pallet, Storage, Call, Event<T>, Config, ValidateUnsigned} = 42,
		EmergencyShutdown: module_emergency_shutdown::{Pallet, Storage, Call, Event<T>} = 43,

		// Homa
		Homa: setheum_abha::{Pallet, Call} = 44,
		NomineesElection: module_nominees_election::{Pallet, Call, Storage} = 45,
		StakingPool: module_staking_pool::{Pallet, Call, Storage, Event<T>, Config} = 46,
		PolkadotBridge: module_polkadot_bridge::{Pallet, Call, Storage} = 47,
		HomaValidatorListModule: setheum_abha_validator_list::{Pallet, Call, Storage, Event<T>} = 48,

		// setheum Other
		Incentives: module_incentives::{Pallet, Storage, Call, Event<T>} = 49,
		NFT: module_nft::{Pallet, Call, Event<T>} = 50,

		// Smart contracts
		EVM: sevm::{Pallet, Config<T>, Call, Storage, Event<T>} = 52,
		SEVMBridge: sevm_evm_bridge::{Pallet} = 53,

		// Parachain
		ParachainSystem: cumulus_pallet_parachain_system::{Pallet, Call, Storage, Inherent, Event} = 54,
		ParachainInfo: parachain_info::{Pallet, Storage, Config} = 55,
		XcmHandler: cumulus_pallet_xcm_handler::{Pallet, Event<T>, Origin} = 56,
		XTokens: orml_xtokens::{Pallet, Storage, Call, Event<T>} = 57,
		UnknownTokens: orml_unknown_tokens::{Pallet, Storage, Event} = 58,

		// Dev
		Sudo: pallet_sudo::{Pallet, Call, Config<T>, Storage, Event<T>} = 59,
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
	sevm::SetEvmOrigin<Runtime>,
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
				DataProviderId::setheum => setheumOracle::get_no_op(&key),
				DataProviderId::Band => BandOracle::get_no_op(&key),
				DataProviderId::Aggregated => <AggregatedDataProvider as DataProviderExtended<_, _>>::get_no_op(&key)
			}
		}

		fn get_all_values(provider_id: DataProviderId) -> Vec<(CurrencyId, Option<TimeStampedPrice>)> {
			match provider_id {
				DataProviderId::setheum => setheumOracle::get_all_values(),
				DataProviderId::Band => BandOracle::get_all_values(),
				DataProviderId::Aggregated => <AggregatedDataProvider as DataProviderExtended<_, _>>::get_all_values()
			}
		}
	}

	impl module_staking_pool_rpc_runtime_api::StakingPoolApi<
		Block,
		AccountId,
		Balance,
	> for Runtime {
		fn get_available_unbonded(account: AccountId) -> module_staking_pool_rpc_runtime_api::BalanceInfo<Balance> {
			module_staking_pool_rpc_runtime_api::BalanceInfo {
				amount: StakingPool::get_available_unbonded(&account)
			}
		}

		fn get_liquid_staking_exchange_rate() -> ExchangeRate {
			StakingPool::liquid_exchange_rate()
		}
	}

	impl sevm_rpc_runtime_api::EVMRuntimeRPCApi<Block, Balance> for Runtime {
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
				let mut config = <Runtime as sevm::Config>::config().clone();
				config.estimate = true;
				Some(config)
			} else {
				None
			};

			sevm::Runner::<Runtime>::call(
				from,
				from,
				to,
				data,
				value,
				gas_limit.into(),
				storage_limit,
				config.as_ref().unwrap_or(<Runtime as sevm::Config>::config()),
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
				let mut config = <Runtime as sevm::Config>::config().clone();
				config.estimate = true;
				Some(config)
			} else {
				None
			};

			sevm::Runner::<Runtime>::create(
				from,
				data,
				value,
				gas_limit.into(),
				storage_limit,
				config.as_ref().unwrap_or(<Runtime as sevm::Config>::config()),
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

			add_benchmark!(params, batches, nft, NftBench::<Runtime>);
			// orml_add_benchmark!(params, batches, settindex, benchmarking::settindex);
			// orml_add_benchmark!(params, batches, transaction_payment, benchmarking::transaction_payment);
			// orml_add_benchmark!(params, batches, incentives, benchmarking::incentives);
			// orml_add_benchmark!(params, batches, prices, benchmarking::prices);

			// orml_add_benchmark!(params, batches, orml_tokens, benchmarking::tokens);
			// orml_add_benchmark!(params, batches, orml_vesting, benchmarking::vesting);
			// orml_add_benchmark!(params, batches, orml_currencies, benchmarking::currencies);

			// orml_add_benchmark!(params, batches, orml_authority, benchmarking::authority);
			// orml_add_benchmark!(params, batches, orml_gradually_update, benchmarking::gradually_update);
			// orml_add_benchmark!(params, batches, orml_oracle, benchmarking::oracle);

			if batches.is_empty() { return Err("Benchmark not found for this module.".into()) }
			Ok(batches)
		}
	}
}

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
