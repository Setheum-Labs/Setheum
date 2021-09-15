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

//! Mocks for the transaction payment module.

#![cfg(test)]

use super::*;
use crate as transaction_payment;
use frame_support::{
	construct_runtime, ord_parameter_types, parameter_types, weights::WeightToFeeCoefficients, PalletId,
};
use orml_traits::parameter_type_with_key;
use primitives::{Amount, TokenSymbol, TradingPair};
use smallvec::smallvec;
use sp_core::{crypto::AccountId32, H160, H256};
use sp_runtime::{
	testing::Header,
	traits::{IdentityLookup, One},
	Perbill,
};
use sp_std::cell::RefCell;
use support::{mocks::MockAddressMapping, Ratio, DEXManager, Price, SerpTreasury};

pub type AccountId = AccountId32;
pub type BlockNumber = u64;

pub const ALICE: AccountId = AccountId::new([1u8; 32]);
pub const BOB: AccountId = AccountId::new([2u8; 32]);
pub const CHARLIE: AccountId = AccountId::new([3u8; 32]);
pub const SETHEUM: CurrencyId = CurrencyId::Token(TokenSymbol::SETHEUM);
pub const SETUSD: CurrencyId = CurrencyId::Token(TokenSymbol::SETUSD);
pub const DNAR: CurrencyId = CurrencyId::Token(TokenSymbol::DNAR);

parameter_types! {
	pub const BlockHashCount: u64 = 250;
	pub static ExtrinsicBaseWeight: u64 = 0;
}

pub struct BlockWeights;
impl Get<frame_system::limits::BlockWeights> for BlockWeights {
	fn get() -> frame_system::limits::BlockWeights {
		frame_system::limits::BlockWeights::builder()
			.base_block(0)
			.for_class(DispatchClass::all(), |weights| {
				weights.base_extrinsic = EXTRINSIC_BASE_WEIGHT.with(|v| *v.borrow()).into();
			})
			.for_class(DispatchClass::non_mandatory(), |weights| {
				weights.max_total = 1024.into();
			})
			.build_or_panic()
	}
}

impl frame_system::Config for Runtime {
	type Origin = Origin;
	type Index = u64;
	type BlockNumber = BlockNumber;
	type Call = Call;
	type Hash = H256;
	type Hashing = ::sp_runtime::traits::BlakeTwo256;
	type AccountId = AccountId;
	type Lookup = IdentityLookup<Self::AccountId>;
	type Header = Header;
	type Event = Event;
	type BlockHashCount = BlockHashCount;
	type BlockWeights = BlockWeights;
	type BlockLength = ();
	type Version = ();
	type PalletInfo = PalletInfo;
	type AccountData = pallet_balances::AccountData<Balance>;
	type OnNewAccount = ();
	type OnKilledAccount = ();
	type DbWeight = ();
	type BaseCallFilter = ();
	type SystemWeightInfo = ();
	type SS58Prefix = ();
	type OnSetCode = ();
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
	type MaxLocks = ();
}

parameter_types! {
	pub const NativeTokenExistentialDeposit: Balance = 0;
}

impl pallet_balances::Config for Runtime {
	type Balance = Balance;
	type DustRemoval = ();
	type Event = Event;
	type ExistentialDeposit = NativeTokenExistentialDeposit;
	type AccountStore = System;
	type MaxLocks = ();
	type WeightInfo = ();
}

pub type AdaptedBasicCurrency = module_currencies::BasicCurrencyAdapter<Runtime, PalletBalances, Amount, BlockNumber>;

parameter_types! {
	pub const GetNativeCurrencyId: CurrencyId = SETHEUM;
}

pub struct MockSerpTreasury;
impl SerpTreasury<AccountId> for MockSerpTreasury {
	type Balance = Balance;
	type CurrencyId = CurrencyId;

	/// Deliver System StableCurrency Inflation
	fn issue_stablecurrency_inflation() -> DispatchResult {
		unimplemented!()
	}

	/// SerpUp ratio for BuyBack Swaps to burn Dinar
	fn get_buyback_serpup(
		_amount: Balance,
		_currency_id: CurrencyId,
	) -> DispatchResult {
		unimplemented!()
	}

	/// SerpUp ratio for Setheum Foundation's Charity Fund
	fn get_charity_fund_serpup(
		_amount: Balance,
		_currency_id: CurrencyId
	) -> DispatchResult {
		unimplemented!()
	}
	
	/// SerpUp ratio for SetPay Cashdrops
	fn get_cashdrop_serpup(
		_amount: Balance,
		_currency_id: CurrencyId
	) -> DispatchResult {
		unimplemented!()
	}

	/// issue serpup surplus(stable currencies) to their destinations according to the serpup_ratio.
	fn on_serpup(
		_currency_id: CurrencyId,
		_amount: Balance,
	) -> DispatchResult {
		unimplemented!()
	}

	/// buy back and burn surplus(stable currencies) with swap by DEX.
	fn on_serpdown(
		_currency_id: CurrencyId,
		_amount: Balance,
	) -> DispatchResult {
		unimplemented!()
	}

	/// get the minimum supply of a setcurrency - by key
	fn get_minimum_supply(
		_currency_id: CurrencyId
	) -> Balance {
		unimplemented!()
	}

	/// issue standard to `who`
	fn issue_standard(
		_currency_id: CurrencyId,
		_who: &AccountId,
		_standard: Balance
	) -> DispatchResult {
		unimplemented!()
	}

	/// burn standard(stable currency) of `who`
	fn burn_standard(
		_currency_id: CurrencyId,
		_who: &AccountId,
		_standard: Balance
	) -> DispatchResult {
		unimplemented!()
	}

	/// issue setter of amount setter to `who`
	fn issue_setter(
		_who: &AccountId,
		_setter: Balance
	) -> DispatchResult {
		unimplemented!()
	}

	/// burn setter of `who`
	fn burn_setter(
		_who: &AccountId,
		_setter: Balance
	) -> DispatchResult {
		unimplemented!()
	}

	/// deposit reserve asset (Setter (SETUSD)) to serp treasury by `who`
	fn deposit_setter(
		_from: &AccountId,
		_amount: Balance
	) -> DispatchResult {
		unimplemented!()
	}

	/// claim cashdrop of `currency_id` relative to `transfer_amount` for `who`
	fn claim_cashdrop(
		_currency_id: CurrencyId,
		_who: &AccountId,
		_transfer_amount: Balance
	) -> DispatchResult {
		unimplemented!()
	}
}

impl module_currencies::Config for Runtime {
	type Event = Event;
    type SerpTreasury = MockSerpTreasury;
	type MultiCurrency = Tokens;
	type NativeCurrency = AdaptedBasicCurrency;
	type GetNativeCurrencyId = GetNativeCurrencyId;
	type WeightInfo = ();
	type AddressMapping = MockAddressMapping;
	type EVMBridge = ();
}

ord_parameter_types! {
	pub const Zero: AccountId = AccountId::new([0u8; 32]);
}

parameter_types! {
	pub const DEXPalletId: PalletId = PalletId(*b"set/sdex");
	pub const GetExchangeFee: (u32, u32) = (0, 100);
	pub const GetStableCurrencyExchangeFee: (u32, u32) = (0, 100);
	pub const TradingPathLimit: u32 = 3;
	pub EnabledTradingPairs: Vec<TradingPair> = vec![
		TradingPair::from_currency_ids(SETUSD, DNAR).unwrap(),
		TradingPair::from_currency_ids(SETUSD, SETHEUM).unwrap(),
	];
	pub StableCurrencyIds: Vec<CurrencyId> = vec![
		SETUSD,
		SETUSD,
	];
}

pub struct MockDEX;
impl DEXManager<AccountId, CurrencyId, Balance> for MockDEX {
	fn get_liquidity_pool(currency_id_a: CurrencyId, currency_id_b: CurrencyId) -> (Balance, Balance) {
		match (currency_id_a, currency_id_b) {
			(SETUSD, SETHEUM) => (10000, 200),
			(DNAR, SETUSD) => (200, 10000),
			_ => (0, 0),
		}
	}

	fn get_liquidity_token_address(_currency_id_a: CurrencyId, _currency_id_b: CurrencyId) -> Option<H160> {
		unimplemented!()
	}

	fn get_swap_target_amount(
		_path: &[CurrencyId],
		_supply_amount: Balance,
	) -> Option<Balance> {
		unimplemented!()
	}

	fn get_swap_supply_amount(
		_path: &[CurrencyId],
		_target_amount: Balance,
	) -> Option<Balance> {
		unimplemented!()
	}

	fn swap_with_exact_supply(
		_who: &AccountId,
		_path: &[CurrencyId],
		_supply_amount: Balance,
		_min_target_amount: Balance,
	) -> sp_std::result::Result<Balance, DispatchError> {
		unimplemented!()
	}

	fn buyback_swap_with_exact_supply(
		_who: &AccountId,
		_path: &[CurrencyId],
		_supply_amount: Balance,
	) -> sp_std::result::Result<Balance, DispatchError> {
		unimplemented!()
	}

	fn swap_with_exact_target(
		_who: &AccountId,
		_path: &[CurrencyId],
		_target_amount: Balance,
		_max_supply_amount: Balance,
	) -> sp_std::result::Result<Balance, DispatchError> {
		unimplemented!()
	}

	fn buyback_swap_with_exact_target(
		_who: &AccountId,
		_path: &[CurrencyId],
		_target_amount: Balance,
	) -> sp_std::result::Result<Balance, DispatchError> {
		unimplemented!()
	}

	fn add_liquidity(
		_who: &AccountId,
		_currency_id_a: CurrencyId,
		_currency_id_b: CurrencyId,
		_max_amount_a: Balance,
		_max_amount_b: Balance,
		_min_share_increment: Balance,
	) -> DispatchResult {
		unimplemented!()
	}

	fn remove_liquidity(
		_who: &AccountId,
		_currency_id_a: CurrencyId,
		_currency_id_b: CurrencyId,
		_remove_share: Balance,
		_min_withdrawn_a: Balance,
		_min_withdrawn_b: Balance,
	) -> DispatchResult {
		unimplemented!()
	}
}

parameter_types! {
	pub MaxSwapSlippageCompareToOracle: Ratio = Ratio::saturating_from_rational(1, 2);
	pub static TransactionByteFee: u128 = 1;
	pub DefaultFeeSwapPathList: Vec<Vec<CurrencyId>> = vec![vec![SETUSD, SETHEUM], vec![DNAR, SETUSD, SETHEUM]];
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
	
	fn lock_price(_currency_id: CurrencyId) {}

	fn unlock_price(_currency_id: CurrencyId) {}
}

impl Config for Runtime {
	type NativeCurrencyId = GetNativeCurrencyId;
	type DefaultFeeSwapPathList = DefaultFeeSwapPathList;
	type Currency = PalletBalances;
	type MultiCurrency = Currencies;
	type OnTransactionPayment = DealWithFees;
	type TransactionByteFee = TransactionByteFee;
	type WeightToFee = WeightToFee;
	type FeeMultiplierUpdate = ();
	type DEX = MockDEX;
	type MaxSwapSlippageCompareToOracle = MaxSwapSlippageCompareToOracle;
	type TradingPathLimit = TradingPathLimit;
	type PriceSource = MockPriceSource;
	type WeightInfo = ();
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

type UncheckedExtrinsic = frame_system::mocking::MockUncheckedExtrinsic<Runtime>;
type Block = frame_system::mocking::MockBlock<Runtime>;

construct_runtime!(
	pub enum Runtime where
		Block = Block,
		NodeBlock = Block,
		UncheckedExtrinsic = UncheckedExtrinsic
	{
		System: frame_system::{Pallet, Call, Config, Storage, Event<T>},
		TransactionPayment: transaction_payment::{Pallet, Call, Storage},
		PalletBalances: pallet_balances::{Pallet, Call, Storage, Config<T>, Event<T>},
		Tokens: orml_tokens::{Pallet, Storage, Event<T>, Config<T>},
		Currencies: module_currencies::{Pallet, Call, Event<T>},
	}
);

pub struct ExtBuilder {
	endowed_accounts: Vec<(AccountId, CurrencyId, Balance)>,
	base_weight: u64,
	byte_fee: u128,
	weight_to_fee: u128,
	native_balances: Vec<(AccountId, Balance)>,
}

impl Default for ExtBuilder {
	fn default() -> Self {
		Self {
			endowed_accounts: vec![(ALICE, SETUSD, 10000), (ALICE, DNAR, 1000)],
			base_weight: 0,
			byte_fee: 2,
			weight_to_fee: 1,
			native_balances: vec![],
		}
	}
}

impl ExtBuilder {
	pub fn base_weight(mut self, base_weight: u64) -> Self {
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
	pub fn one_hundred_thousand_for_alice_n_charlie(mut self) -> Self {
		self.native_balances = vec![(ALICE, 100000), (CHARLIE, 100000)];
		self
	}
	fn set_constants(&self) {
		EXTRINSIC_BASE_WEIGHT.with(|v| *v.borrow_mut() = self.base_weight);
		TRANSACTION_BYTE_FEE.with(|v| *v.borrow_mut() = self.byte_fee);
		WEIGHT_TO_FEE.with(|v| *v.borrow_mut() = self.weight_to_fee);
	}
	pub fn build(self) -> sp_io::TestExternalities {
		self.set_constants();
		let mut t = frame_system::GenesisConfig::default()
			.build_storage::<Runtime>()
			.unwrap();

		pallet_balances::GenesisConfig::<Runtime> {
			balances: self.native_balances,
		}
		.assimilate_storage(&mut t)
		.unwrap();

		orml_tokens::GenesisConfig::<Runtime> {
			endowed_accounts: self.endowed_accounts,
		}
		.assimilate_storage(&mut t)
		.unwrap();

		t.into()
	}
}
