// بِسْمِ اللَّهِ الرَّحْمَنِ الرَّحِيم

// This file is part of Setheum.

// Copyright (C) 2019-Present Setheum Labs.
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

//! Mocks for the transaction payment module.

#![cfg(test)]

use super::*;
pub use crate as transaction_payment;
use frame_support::{
	construct_runtime, derive_impl, ord_parameter_types, parameter_types,
	traits::{ConstU128, ConstU32, ConstU64, Nothing},
	weights::{WeightToFee as WeightToFeeT, WeightToFeeCoefficients, WeightToFeePolynomial},
	PalletId,
};
use frame_system::EnsureSignedBy;
use module_support::{
	mocks::MockAddressMapping,
	Price, SpecificJointsSwap,
};
use orml_traits::parameter_type_with_key;
use primitives::{Amount, ReserveIdentifier, TokenSymbol, TradingPair};
use smallvec::smallvec;
use sp_core::{crypto::AccountId32, H160};
use sp_runtime::{
	traits::{AccountIdConversion, IdentityLookup, One},
	BuildStorage, Perbill,
};
use sp_std::cell::RefCell;

pub type AccountId = AccountId32;
pub type BlockNumber = u64;

pub const ALICE: AccountId = AccountId::new([1u8; 32]);
pub const BOB: AccountId = AccountId::new([2u8; 32]);
pub const CHARLIE: AccountId = AccountId::new([3u8; 32]);
pub const DAVE: AccountId = AccountId::new([4u8; 32]);
pub const SEE: CurrencyId = CurrencyId::Token(TokenSymbol::SEE);
pub const SETR: CurrencyId = CurrencyId::Token(TokenSymbol::SETR);
pub const USSD: CurrencyId = CurrencyId::Token(TokenSymbol::USSD);
pub const EDF: CurrencyId = CurrencyId::Token(TokenSymbol::EDF);
pub const LSEE: CurrencyId = CurrencyId::Token(TokenSymbol::LSEE);

parameter_types! {
	static ExtrinsicBaseWeight: Weight = Weight::zero();
}

pub struct BlockWeights;
impl Get<frame_system::limits::BlockWeights> for BlockWeights {
	fn get() -> frame_system::limits::BlockWeights {
		frame_system::limits::BlockWeights::builder()
			.base_block(Weight::zero())
			.for_class(DispatchClass::all(), |weights| {
				weights.base_extrinsic = ExtrinsicBaseWeight::get().into();
			})
			.for_class(DispatchClass::non_mandatory(), |weights| {
				weights.max_total = Weight::from_parts(1024, 0).set_proof_size(u64::MAX).into();
			})
			.build_or_panic()
	}
}

#[derive_impl(frame_system::config_preludes::TestDefaultConfig as frame_system::DefaultConfig)]
impl frame_system::Config for Runtime {
	type AccountId = AccountId;
	type Lookup = IdentityLookup<Self::AccountId>;
	type Block = Block;
	type BlockWeights = BlockWeights;
	type AccountData = pallet_balances::AccountData<Balance>;
}

parameter_type_with_key! {
	pub ExistentialDeposits: |currency_id: CurrencyId| -> Balance {
		match *currency_id {
			USSD => 100,
			SETR => 100,
			EDF | LSEE => 1,
			_ => Default::default(),
		}
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

impl pallet_balances::Config for Runtime {
	type Balance = Balance;
	type DustRemoval = ();
	type RuntimeEvent = RuntimeEvent;
	type ExistentialDeposit = ConstU128<10>;
	type AccountStore = System;
	type MaxLocks = ();
	type MaxReserves = ConstU32<50>;
	type ReserveIdentifier = ReserveIdentifier;
	type WeightInfo = ();
	type RuntimeHoldReason = RuntimeHoldReason;
	type RuntimeFreezeReason = RuntimeFreezeReason;
	type FreezeIdentifier = ();
	type MaxHolds = ();
	type MaxFreezes = ();
}

pub type AdaptedBasicCurrency = module_currencies::BasicCurrencyAdapter<Runtime, PalletBalances, Amount, BlockNumber>;

parameter_types! {
	pub const GetNativeCurrencyId: CurrencyId = SEE;
	pub Erc20HoldingAccount: H160 = H160::from_low_u64_be(1);
}

impl module_currencies::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type MultiCurrency = Tokens;
	type NativeCurrency = AdaptedBasicCurrency;
	type GetNativeCurrencyId = GetNativeCurrencyId;
	type Erc20HoldingAccount = Erc20HoldingAccount;
	type WeightInfo = ();
	type AddressMapping = MockAddressMapping;
	type EVMBridge = ();
	type GasToWeight = ();
	type SweepOrigin = EnsureSignedBy<Zero, AccountId>;
	type OnDust = ();
}

thread_local! {
	static IS_SHUTDOWN: RefCell<bool> = RefCell::new(false);
}

ord_parameter_types! {
	pub const Zero: AccountId = AccountId::new([0u8; 32]);
}

parameter_types! {
	pub const DEXPalletId: PalletId = PalletId(*b"set/edfis");
	pub const GetExchangeFee: (u32, u32) = (0, 100);
	pub EnabledTradingPairs: Vec<TradingPair> = vec![
		TradingPair::from_currency_ids(USSD, SEE).unwrap(),
		TradingPair::from_currency_ids(USSD, SETR).unwrap(),
		TradingPair::from_currency_ids(SETR, SEE).unwrap(),
		TradingPair::from_currency_ids(USSD, EDF).unwrap(),
		TradingPair::from_currency_ids(SEE, LSEE).unwrap(),
	];
	pub const TradingPathLimit: u32 = 4;
}

impl edfis_swap_legacy_module::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type Currency = Currencies;
	type GetExchangeFee = GetExchangeFee;
	type TradingPathLimit = TradingPathLimit;
	type PalletId = DEXPalletId;
	type Erc20InfoMapping = ();
	type SwapDexIncentives = ();
	type WeightInfo = ();
	type ListingOrigin = EnsureSignedBy<Zero, AccountId>;
	type ExtendedProvisioningBlocks = ConstU64<0>;
	type OnLiquidityPoolUpdated = ();
}

parameter_types! {
	pub MaxSwapSlippageComparedToOracle: Ratio = Ratio::saturating_from_rational(1, 2);
	pub static TransactionByteFee: u128 = 1;
	pub static TipPerWeightStep: u128 = 1;
	pub DefaultFeeTokens: Vec<CurrencyId> = vec![USSD, SETR];
	pub USSDFeeSwapPath: Vec<CurrencyId> = vec![USSD, SEE];
	pub SETRFeeSwapPath: Vec<CurrencyId> = vec![SETR, SEE];
	pub EDFFeeSwapPath: Vec<CurrencyId> = vec![EDF, SETR, SEE];
}

thread_local! {
	pub static TIP_UNBALANCED_AMOUNT: RefCell<u128> = RefCell::new(0);
	pub static FEE_UNBALANCED_AMOUNT: RefCell<u128> = RefCell::new(0);
}

pub struct DealWithFees;
impl OnUnbalanced<pallet_balances::NegativeImbalance<Runtime>> for DealWithFees {
	fn on_unbalanceds<B>(mut fees_then_tips: impl Iterator<Item = pallet_balances::NegativeImbalance<Runtime>>) {
		if let Some(fees) = fees_then_tips.next() {
			FEE_UNBALANCED_AMOUNT.with(|a| *a.borrow_mut() += fees.peek());
			if let Some(tips) = fees_then_tips.next() {
				TIP_UNBALANCED_AMOUNT.with(|a| *a.borrow_mut() += tips.peek());
			}
		}
	}
}

thread_local! {
	static RELATIVE_PRICE: RefCell<Option<Price>> = RefCell::new(Some(Price::one()));
}

pub struct MockPriceSource;
impl MockPriceSource {
	pub fn set_relative_price(price: Option<Price>) {
		RELATIVE_PRICE.with(|v| *v.borrow_mut() = price);
	}
}
impl PriceProvider<CurrencyId> for MockPriceSource {
	fn get_relative_price(_base: CurrencyId, _quote: CurrencyId) -> Option<Price> {
		RELATIVE_PRICE.with(|v| *v.borrow_mut())
	}

	fn get_price(_currency_id: CurrencyId) -> Option<Price> {
		unimplemented!()
	}
}

parameter_types! {
	// DO NOT CHANGE THIS VALUE, AS IT EFFECT THE TESTCASES.
	pub const FeePoolSize: Balance = 10_000;
	pub const LowerSwapThreshold: Balance = 20;
	pub const MiddSwapThreshold: Balance = 5000;
	pub const HigerSwapThreshold: Balance = 9500;
	pub const TransactionPaymentPalletId: PalletId = PalletId(*b"set/fees");
	pub const TreasuryPalletId: PalletId = PalletId(*b"set/trsy");
	pub TreasuryAccount: AccountId = TreasuryPalletId::get().into_account_truncating();
	pub AlternativeSwapPathJointList: Vec<Vec<CurrencyId>> = vec![
		vec![SETR, USSD],
	];
}
ord_parameter_types! {
	pub const ListingOrigin: AccountId = ALICE;
	pub const CustomFeeSurplus: Percent = Percent::from_percent(50);
	pub const AlternativeFeeSurplus: Percent = Percent::from_percent(25);
}

impl WeightToFeeT for TransactionByteFee {
	type Balance = Balance;

	fn weight_to_fee(weight: &Weight) -> Self::Balance {
		Self::Balance::saturated_from(weight.ref_time()).saturating_mul(TRANSACTION_BYTE_FEE.with(|v| *v.borrow()))
	}
}

impl Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type RuntimeCall = RuntimeCall;
	type NativeCurrencyId = GetNativeCurrencyId;
	type Currency = PalletBalances;
	type MultiCurrency = Currencies;
	type OnTransactionPayment = DealWithFees;
	type OperationalFeeMultiplier = ConstU64<5>;
	type TipPerWeightStep = TipPerWeightStep;
	type MaxTipsOfPriority = ConstU128<1000>;
	type AlternativeFeeSwapDeposit = ConstU128<1000>;
	type WeightToFee = WeightToFee;
	type LengthToFee = TransactionByteFee;
	type FeeMultiplierUpdate = ();
	type Swap = SpecificJointsSwap<EdfisSwapLegacyModule, AlternativeSwapPathJointList>;
	type MaxSwapSlippageComparedToOracle = MaxSwapSlippageComparedToOracle;
	type TradingPathLimit = TradingPathLimit;
	type PriceSource = MockPriceSource;
	type WeightInfo = ();
	type PalletId = TransactionPaymentPalletId;
	type TreasuryAccount = TreasuryAccount;
	type CustomFeeSurplus = CustomFeeSurplus;
	type AlternativeFeeSurplus = AlternativeFeeSurplus;
	type DefaultFeeTokens = DefaultFeeTokens;
	type UpdateOrigin = EnsureSignedBy<ListingOrigin, AccountId>;
}

thread_local! {
	static WEIGHT_TO_FEE: RefCell<u128> = RefCell::new(1);
}

pub struct WeightToFee;
impl WeightToFeePolynomial for WeightToFee {
	type Balance = u128;

	fn polynomial() -> WeightToFeeCoefficients<Self::Balance> {
		smallvec![frame_support::weights::WeightToFeeCoefficient {
			degree: 1,
			coeff_frac: Perbill::zero(),
			coeff_integer: WEIGHT_TO_FEE.with(|v| *v.borrow()),
			negative: false,
		}]
	}
}

type Block = frame_system::mocking::MockBlock<Runtime>;

construct_runtime!(
	pub enum Runtime {
		System: frame_system,
		TransactionPayment: transaction_payment,
		PalletBalances: pallet_balances,
		Tokens: orml_tokens,
		Currencies: module_currencies,
		EdfisSwapLegacyModule: edfis_swap_legacy_module,
	}
);

pub struct ExtBuilder {
	balances: Vec<(AccountId, CurrencyId, Balance)>,
	base_weight: Weight,
	byte_fee: u128,
	weight_to_fee: u128,
	tip_per_weight_step: u128,
	native_balances: Vec<(AccountId, Balance)>,
}

impl Default for ExtBuilder {
	fn default() -> Self {
		Self {
			balances: vec![
				(ALICE, USSD, 10000),
				(ALICE, SETR, 10000),
				(ALICE, EDF, 1000),
				(ALICE, LSEE, 1000)
			],
			base_weight: Weight::zero(),
			byte_fee: 2,
			weight_to_fee: 1,
			tip_per_weight_step: 1,
			native_balances: vec![],
		}
	}
}

impl ExtBuilder {
	pub fn base_weight(mut self, base_weight: Weight) -> Self {
		self.base_weight = base_weight;
		self
	}
	pub fn byte_fee(mut self, byte_fee: u128) -> Self {
		self.byte_fee = byte_fee;
		self
	}
	pub fn weight_fee(mut self, weight_to_fee: u128) -> Self {
		self.weight_to_fee = weight_to_fee;
		self
	}
	pub fn tip_per_weight_step(mut self, tip_per_weight_step: u128) -> Self {
		self.tip_per_weight_step = tip_per_weight_step;
		self
	}
	pub fn one_hundred_thousand_for_alice_n_charlie(mut self) -> Self {
		self.native_balances = vec![(ALICE, 100000), (CHARLIE, 100000)];
		self
	}
	fn set_constants(&self) {
		ExtrinsicBaseWeight::mutate(|v| *v = self.base_weight);
		TRANSACTION_BYTE_FEE.with(|v| *v.borrow_mut() = self.byte_fee);
		WEIGHT_TO_FEE.with(|v| *v.borrow_mut() = self.weight_to_fee);
		TIP_PER_WEIGHT_STEP.with(|v| *v.borrow_mut() = self.tip_per_weight_step);
	}
	pub fn build(self) -> sp_io::TestExternalities {
		self.set_constants();
		let mut t = frame_system::GenesisConfig::<Runtime>::default()
			.build_storage()
			.unwrap();

		pallet_balances::GenesisConfig::<Runtime> {
			balances: self.native_balances,
		}
		.assimilate_storage(&mut t)
		.unwrap();

		orml_tokens::GenesisConfig::<Runtime> {
			balances: self.balances,
		}
		.assimilate_storage(&mut t)
		.unwrap();

		edfis_swap_legacy_module::GenesisConfig::<Runtime> {
			initial_listing_trading_pairs: vec![],
			initial_enabled_trading_pairs: EnabledTradingPairs::get(),
			initial_added_liquidity_pools: vec![],
		}
		.assimilate_storage(&mut t)
		.unwrap();

		let mut ext = sp_io::TestExternalities::new(t);
		ext.execute_with(|| System::set_block_number(1));
		ext
	}
}
