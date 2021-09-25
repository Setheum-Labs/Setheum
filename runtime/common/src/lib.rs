//! Common runtime code

#![cfg_attr(not(feature = "std"), no_std)]

use codec::{Decode, Encode};
use frame_support::{
	parameter_types,
	weights::{
		constants::{BlockExecutionWeight, ExtrinsicBaseWeight, WEIGHT_PER_MILLIS},
		DispatchClass, Weight,
	},
};
use frame_system::{limits, EnsureOneOf, EnsureRoot};
pub use module_support::{Contains, ExchangeRate, PrecompileCallerFilter, Price, Rate, Ratio};
use primitives::{
	Balance, BlockNumber, CurrencyId, PRECOMPILE_ADDRESS_START, PREDEPLOY_ADDRESS_START, SYSTEM_CONTRACT_ADDRESS_PREFIX,
};
use sp_core::{
	u32_trait::{_1, _2, _3, _4},
	H160,
};
use sp_runtime::{
	traits::Convert,
	transaction_validity::TransactionPriority,
	Perbill,
};
use static_assertions::const_assert;

pub mod precompile;
pub use precompile::{
	AllPrecompiles, DexPrecompile, MultiCurrencyPrecompile, NFTPrecompile, OraclePrecompile, ScheduleCallPrecompile,
	StateRentPrecompile,
};
pub use primitives::{
	currency::{TokenInfo, SETHEUM, DNAR, SETR, SETEUR, SETUSD, RENBTC},
	AccountId,
};

pub type TimeStampedPrice = orml_oracle::TimestampedValue<Price, primitives::Moment>;

// Priority of unsigned transactions
parameter_types! {
	// Operational is 3/4 of TransactionPriority::max_value().
	// Ensure Inherent -> Operational tx -> Unsigned tx -> Signed normal tx
	pub const RenvmBridgeUnsignedPriority: TransactionPriority = TransactionPriority::max_value() / 10;   // 10%
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

pub type ShuraCouncilInstance = pallet_collective::Instance1;
pub type GeneralCouncilInstance = pallet_collective::Instance2;
pub type FinancialCouncilInstance = pallet_collective::Instance3;
pub type TechnicalCommitteeInstance = pallet_collective::Instance4;

pub type ShuraCouncilMembershipInstance = pallet_membership::Instance1;
pub type GeneralCouncilMembershipInstance = pallet_membership::Instance2;
pub type FinancialCouncilMembershipInstance = pallet_membership::Instance3;
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

// General Council
pub type EnsureRootOrAllGeneralCouncil = EnsureOneOf<
	AccountId,
	EnsureRoot<AccountId>,
	pallet_collective::EnsureProportionAtLeast<_1, _1, AccountId, GeneralCouncilInstance>,
>;

pub type EnsureRootOrHalfGeneralCouncil = EnsureOneOf<
	AccountId,
	EnsureRoot<AccountId>,
	pallet_collective::EnsureProportionAtLeast<_1, _2, AccountId, GeneralCouncilInstance>,
>;

pub type EnsureRootOrOneThirdsGeneralCouncil = EnsureOneOf<
	AccountId,
	EnsureRoot<AccountId>,
	pallet_collective::EnsureProportionAtLeast<_1, _3, AccountId, GeneralCouncilInstance>,
>;

pub type EnsureRootOrTwoThirdsGeneralCouncil = EnsureOneOf<
	AccountId,
	EnsureRoot<AccountId>,
	pallet_collective::EnsureProportionAtLeast<_2, _3, AccountId, GeneralCouncilInstance>,
>;

pub type EnsureRootOrThreeFourthsGeneralCouncil = EnsureOneOf<
	AccountId,
	EnsureRoot<AccountId>,
	pallet_collective::EnsureProportionAtLeast<_3, _4, AccountId, GeneralCouncilInstance>,
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

/// The type used to represent the kinds of proxying allowed.
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Encode, Decode, RuntimeDebug)]
pub enum ProxyType {
	Any,
	CancelProxy,
	Governance,
	Swap,
}
impl Default for ProxyType {
	fn default() -> Self {
		Self::Any
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn system_contracts_filter_works() {
		assert!(SystemContractsFilter::is_allowed(H160::from_low_u64_be(1)));

		let mut max_allowed_addr = [0u8; 20];
		max_allowed_addr[SYSTEM_CONTRACT_LEADING_ZERO_BYTES] = 127u8;
		assert!(SystemContractsFilter::is_allowed(max_allowed_addr.into()));

		let mut min_blocked_addr = [0u8; 20];
		min_blocked_addr[SYSTEM_CONTRACT_LEADING_ZERO_BYTES - 1] = 1u8;
		assert!(!SystemContractsFilter::is_allowed(min_blocked_addr.into()));
	}

	#[test]
	fn is_system_contract_works() {
		assert!(is_system_contract(H160::from_low_u64_be(0)));
		assert!(is_system_contract(H160::from_low_u64_be(u64::max_value())));

		let mut bytes = [0u8; 20];
		bytes[SYSTEM_CONTRACT_LEADING_ZERO_BYTES - 1] = 1u8;

		assert!(!is_system_contract(bytes.into()));

		bytes = [0u8; 20];
		bytes[0] = 1u8;

		assert!(!is_system_contract(bytes.into()));
	}

	#[test]
	fn is_core_precompile_works() {
		assert!(!is_core_precompile(H160::from_low_u64_be(0)));
		assert!(!is_core_precompile(H160::from_low_u64_be(
			PRECOMPILE_ADDRESS_START - 1
		)));
		assert!(is_core_precompile(H160::from_low_u64_be(PRECOMPILE_ADDRESS_START)));
		assert!(is_core_precompile(H160::from_low_u64_be(PREDEPLOY_ADDRESS_START - 1)));
		assert!(!is_core_precompile(H160::from_low_u64_be(PREDEPLOY_ADDRESS_START)));
		assert!(!is_core_precompile([1u8; 20].into()));
	}
}
