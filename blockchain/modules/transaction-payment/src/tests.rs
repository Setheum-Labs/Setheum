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

//! Unit tests for the transaction payment module.

#![cfg(test)]

use super::*;
use crate::mock::{AlternativeFeeSurplus, AusdFeeSwapPath, CustomFeeSurplus, DotFeeSwapPath, PalletBalances};
use frame_support::{
	assert_noop, assert_ok,
	dispatch::{DispatchClass, DispatchInfo, Pays},
};
use mock::{
	AccountId, BlockWeights, Currencies, DEXModule, ExtBuilder, FeePoolSize, MockPriceSource, Runtime, RuntimeCall,
	RuntimeOrigin, System, TransactionPayment, ALICE, BOB, CHARLIE, DAVE, FEE_UNBALANCED_AMOUNT, TIP_UNBALANCED_AMOUNT,
	SEE, USSD, EDF, LSEE,
};
use module_support::{BuyWeightRate, DEXManager, Price, TransactionPayment as TransactionPaymentT};
use orml_traits::{MultiCurrency, MultiLockableCurrency};
use pallet_balances::ReserveData;
use primitives::currency::*;
use sp_io::TestExternalities;
use sp_runtime::{
	testing::TestXt,
	traits::{One, UniqueSaturatedInto},
};
use xcm::v3::prelude::*;

const CALL: <Runtime as frame_system::Config>::RuntimeCall =
	RuntimeCall::Currencies(module_currencies::Call::transfer {
		dest: BOB,
		currency_id: USSD,
		amount: 100,
	});

const CALL2: <Runtime as frame_system::Config>::RuntimeCall =
	RuntimeCall::Currencies(module_currencies::Call::transfer_native_currency { dest: BOB, amount: 12 });

const INFO: DispatchInfo = DispatchInfo {
	weight: Weight::from_parts(1000, 0),
	class: DispatchClass::Normal,
	pays_fee: Pays::Yes,
};

const INFO2: DispatchInfo = DispatchInfo {
	weight: Weight::from_parts(100, 0),
	class: DispatchClass::Normal,
	pays_fee: Pays::Yes,
};

const POST_INFO: PostDispatchInfo = PostDispatchInfo {
	actual_weight: Some(Weight::from_parts(800, 0)),
	pays_fee: Pays::Yes,
};

const POST_INFO2: PostDispatchInfo = PostDispatchInfo {
	actual_weight: Some(Weight::from_parts(80, 0)),
	pays_fee: Pays::Yes,
};

fn with_fee_path_call(fee_swap_path: Vec<CurrencyId>) -> <Runtime as Config>::RuntimeCall {
	let fee_call: <Runtime as Config>::RuntimeCall =
		RuntimeCall::TransactionPayment(crate::mock::transaction_payment::Call::with_fee_path {
			fee_swap_path,
			call: Box::new(CALL),
		});
	fee_call
}

fn with_fee_currency_call(currency_id: CurrencyId) -> <Runtime as Config>::RuntimeCall {
	let fee_call: <Runtime as Config>::RuntimeCall =
		RuntimeCall::TransactionPayment(crate::mock::transaction_payment::Call::with_fee_currency {
			currency_id,
			call: Box::new(CALL),
		});
	fee_call
}

fn with_fee_aggregated_path_by_call(
	fee_aggregated_path: Vec<AggregatedSwapPath<CurrencyId>>,
) -> <Runtime as Config>::RuntimeCall {
	let fee_call: <Runtime as Config>::RuntimeCall =
		RuntimeCall::TransactionPayment(crate::mock::transaction_payment::Call::with_fee_aggregated_path {
			fee_aggregated_path,
			call: Box::new(CALL),
		});
	fee_call
}

fn enable_dex_and_tx_fee_pool() {
	let treasury_account: AccountId = <Runtime as Config>::TreasuryAccount::get();
	let init_balance = FeePoolSize::get();
	assert_ok!(Currencies::update_balance(
		RuntimeOrigin::root(),
		treasury_account.clone(),
		SEE,
		(init_balance * 100).unique_saturated_into(),
	));
	vec![USSD, SETR, EDF, LSEE].iter().for_each(|token| {
		let ed = (<Currencies as MultiCurrency<AccountId>>::minimum_balance(token.clone())).unique_saturated_into();
		assert_ok!(Currencies::update_balance(
			RuntimeOrigin::root(),
			treasury_account.clone(),
			token.clone(),
			ed,
		));
	});

	let alice_balance = Currencies::free_balance(SEE, &ALICE);
	if alice_balance < 100000 {
		assert_ok!(Currencies::update_balance(
			RuntimeOrigin::root(),
			ALICE,
			SEE,
			100000.unique_saturated_into(),
		));
	}

	// enable dex
	assert_ok!(DEXModule::add_liquidity(
		RuntimeOrigin::signed(ALICE),
		SEE,
		USSD,
		10000,
		1000,
		0,
		false
	));
	assert_ok!(DEXModule::add_liquidity(
		RuntimeOrigin::signed(ALICE),
		EDF,
		USSD,
		100,
		1000,
		0,
		false
	));
	assert_ok!(DEXModule::add_liquidity(
		RuntimeOrigin::signed(ALICE),
		LSEE,
		SEE,
		100,
		1000,
		0,
		false
	));
	assert_eq!(DEXModule::get_liquidity_pool(SEE, USSD), (10000, 1000));
	assert_eq!(DEXModule::get_liquidity_pool(EDF, USSD), (100, 1000));
	assert_eq!(DEXModule::get_liquidity_pool(LSEE, SEE), (100, 1000));
	assert_eq!(DEXModule::get_liquidity_pool(EDF, SEE), (0, 0));

	// enable tx fee pool for USSD and EDF token.
	vec![USSD, EDF].iter().for_each(|token| {
		assert_ok!(Pallet::<Runtime>::enable_charge_fee_pool(
			RuntimeOrigin::signed(ALICE),
			*token,
			FeePoolSize::get(),
			crate::mock::LowerSwapThreshold::get()
		));
	});

	// validate tx fee pool works
	vec![USSD, EDF].iter().for_each(|token| {
		let ed = (<Currencies as MultiCurrency<AccountId>>::minimum_balance(token.clone())).unique_saturated_into();
		let sub_account: AccountId = <Runtime as Config>::PalletId::get().into_sub_account_truncating(token.clone());
		assert_eq!(Currencies::free_balance(token.clone(), &treasury_account), 0);
		assert_eq!(Currencies::free_balance(token.clone(), &sub_account), ed);
		assert_eq!(Currencies::free_balance(SEE, &sub_account), init_balance);
	});

	// manual set the exchange rate for simplify calculation
	TokenExchangeRate::<Runtime>::insert(USSD, Ratio::saturating_from_rational(10, 1));
	let edf_rate = TokenExchangeRate::<Runtime>::get(EDF).unwrap();
	assert_eq!(edf_rate, Ratio::saturating_from_rational(1, 10));
}

fn builder_with_dex_and_fee_pool(enable_pool: bool) -> TestExternalities {
	let mut builder = ExtBuilder::default().one_hundred_thousand_for_alice_n_charlie().build();
	if enable_pool {
		builder.execute_with(|| {
			enable_dex_and_tx_fee_pool();
		});
	}
	builder
}

#[test]
fn charges_fee_when_native_is_enough_but_cannot_keep_alive() {
	ExtBuilder::default().build().execute_with(|| {
		// balance set to fee, after charge fee, balance less than ED, cannot keep alive
		// fee = len(validate method parameter) * byte_fee(constant) + weight(in DispatchInfo)
		let fee = 5000 * 2 + 1000;
		assert_ok!(Currencies::update_balance(
			RuntimeOrigin::root(),
			ALICE,
			SEE,
			fee.unique_saturated_into(),
		));
		assert_eq!(Currencies::free_balance(SEE, &ALICE), fee);
		assert_noop!(
			ChargeTransactionPayment::<Runtime>::from(0).validate(&ALICE, &CALL, &INFO, 5000),
			TransactionValidityError::Invalid(InvalidTransaction::Payment)
		);

		// after charge fee, balance=fee-fee2=ED, equal to ED, keep alive
		let fee2 = 5000 * 2 + 990;
		let info = DispatchInfo {
			weight: Weight::from_parts(990, 0),
			class: DispatchClass::Normal,
			pays_fee: Pays::Yes,
		};
		let expect_priority = ChargeTransactionPayment::<Runtime>::get_priority(&info, 5000, fee2, fee2);
		assert_eq!(1000, expect_priority);
		assert_eq!(
			ChargeTransactionPayment::<Runtime>::from(0)
				.validate(&ALICE, &CALL, &info, 5000)
				.unwrap()
				.priority,
			1
		);
		assert_eq!(Currencies::free_balance(SEE, &ALICE), Currencies::minimum_balance(SEE));
	});
}

#[test]
fn charges_fee_when_validate_native_is_enough() {
	// Alice init 100000 SEE(native asset)
	builder_with_dex_and_fee_pool(false).execute_with(|| {
		let fee = 23 * 2 + 1000; // len * byte + weight
		assert_eq!(
			ChargeTransactionPayment::<Runtime>::from(0)
				.validate(&ALICE, &CALL, &INFO, 23)
				.unwrap()
				.priority,
			1
		);
		assert_eq!(Currencies::free_balance(SEE, &ALICE), 100000 - fee);

		let fee2 = 18 * 2 + 1000; // len * byte + weight
		assert_eq!(
			ChargeTransactionPayment::<Runtime>::from(0)
				.validate(&ALICE, &CALL2, &INFO, 18)
				.unwrap()
				.priority,
			1
		);
		assert_eq!(Currencies::free_balance(SEE, &ALICE), 100000 - fee - fee2);
	});
}

#[test]
fn charges_fee_when_locked_transfer_not_enough() {
	builder_with_dex_and_fee_pool(false).execute_with(|| {
		let fee = 12 * 2 + 1000; // len * byte + weight
		assert_ok!(Currencies::update_balance(RuntimeOrigin::root(), BOB, SEE, 2048,));

		// transferable=2048-1025 < fee=1024, native asset is not enough
		assert_ok!(<Currencies as MultiLockableCurrency<AccountId>>::set_lock(
			[0u8; 8], SEE, &BOB, 1025
		));
		assert_noop!(
			ChargeTransactionPayment::<Runtime>::from(0).validate(&BOB, &CALL, &INFO, 12),
			TransactionValidityError::Invalid(InvalidTransaction::Payment)
		);

		// after remove lock, transferable=2048 > fee
		assert_ok!(<Currencies as MultiLockableCurrency<AccountId>>::remove_lock(
			[0u8; 8], SEE, &BOB
		));
		assert_ok!(ChargeTransactionPayment::<Runtime>::from(0).validate(&BOB, &CALL, &INFO, 12));
		assert_eq!(Currencies::free_balance(SEE, &BOB), 2048 - fee);
	});
}

#[test]
fn pre_post_dispatch_and_refund_native_is_enough() {
	builder_with_dex_and_fee_pool(false).execute_with(|| {
		let fee = 23 * 2 + 1000; // len * byte + weight
		let pre = ChargeTransactionPayment::<Runtime>::from(0)
			.pre_dispatch(&ALICE, &CALL, &INFO, 23)
			.unwrap();
		assert_eq!(Currencies::free_balance(SEE, &ALICE), 100000 - fee);

		let actual_fee = TransactionPayment::compute_actual_fee(23, &INFO, &POST_INFO, 0);
		assert_eq!(actual_fee, 23 * 2 + 800);

		assert_ok!(ChargeTransactionPayment::<Runtime>::post_dispatch(
			Some(pre),
			&INFO,
			&POST_INFO,
			23,
			&Ok(())
		));

		let refund = 200; // 1000 - 800
		assert_eq!(Currencies::free_balance(SEE, &ALICE), 100000 - fee + refund);
		assert_eq!(FEE_UNBALANCED_AMOUNT.with(|a| *a.borrow()), fee - refund);
		assert_eq!(TIP_UNBALANCED_AMOUNT.with(|a| *a.borrow()), 0);

		System::assert_has_event(crate::mock::RuntimeEvent::TransactionPayment(
			crate::Event::TransactionFeePaid {
				who: ALICE,
				actual_fee,
				actual_tip: 0,
				actual_surplus: 0,
			},
		));

		// reset and test refund with tip
		FEE_UNBALANCED_AMOUNT.with(|a| *a.borrow_mut() = 0);

		let tip: Balance = 5;
		let pre = ChargeTransactionPayment::<Runtime>::from(tip)
			.pre_dispatch(&CHARLIE, &CALL, &INFO, 23)
			.unwrap();
		assert_eq!(Currencies::free_balance(SEE, &CHARLIE), 100000 - fee - tip);
		let actual_fee = TransactionPayment::compute_actual_fee(23, &INFO, &POST_INFO, tip);
		assert_eq!(actual_fee, 23 * 2 + 800 + 5);
		assert_ok!(ChargeTransactionPayment::<Runtime>::post_dispatch(
			Some(pre),
			&INFO,
			&POST_INFO,
			23,
			&Ok(())
		));
		assert_eq!(Currencies::free_balance(SEE, &CHARLIE), 100000 - fee - tip + refund);
		assert_eq!(FEE_UNBALANCED_AMOUNT.with(|a| *a.borrow()), fee - refund);
		assert_eq!(TIP_UNBALANCED_AMOUNT.with(|a| *a.borrow()), tip);

		System::assert_has_event(crate::mock::RuntimeEvent::TransactionPayment(
			crate::Event::TransactionFeePaid {
				who: CHARLIE,
				actual_fee,
				actual_tip: tip,
				actual_surplus: 0,
			},
		));
	});
}

#[test]
fn pre_post_dispatch_and_refund_with_fee_currency_call_default_fee_tokens_work() {
	// default fee token, and enabled by charge fee pool
	pre_post_dispatch_and_refund_with_fee_currency_call(
		USSD,
		AlternativeFeeSurplus::get(),
		Ratio::saturating_from_rational(10, 1),
	);
}

#[test]
fn pre_post_dispatch_and_refund_with_fee_currency_call_non_default_fee_tokens_work() {
	// non default fee token, and enabled by charge fee pool
	pre_post_dispatch_and_refund_with_fee_currency_call(
		EDF,
		CustomFeeSurplus::get(),
		Ratio::saturating_from_rational(1, 10),
	);
}

fn pre_post_dispatch_and_refund_with_fee_currency_call(token: CurrencyId, surplus_percent: Percent, token_rate: Ratio) {
	builder_with_dex_and_fee_pool(true).execute_with(|| {
		// with_fee_currency call will swap user's USSD/EDF out of SEE, then withdraw SEE as fee
		let token_subacc = Pallet::<Runtime>::sub_account_id(token);
		let fee: Balance = 500 * 2 + 1000; // len * byte + weight
		let surplus = surplus_percent.mul_ceil(fee);
		let fee_surplus = surplus + fee;

		assert_ok!(Currencies::update_balance(RuntimeOrigin::root(), ALICE, token, 20000));
		let see_init = Currencies::free_balance(SEE, &ALICE);
		let token_init = Currencies::free_balance(token, &ALICE);
		assert_eq!(89000, see_init);

		let pre = ChargeTransactionPayment::<Runtime>::from(0)
			.pre_dispatch(&ALICE, &with_fee_currency_call(token), &INFO, 500)
			.unwrap();
		assert_eq!(pre.2, Some(pallet_balances::NegativeImbalance::new(fee_surplus)));
		assert_eq!(pre.3, fee_surplus);

		// with_fee_currency will set OverrideChargeFeeMethod when pre_dispatch
		assert_eq!(
			OverrideChargeFeeMethod::<Runtime>::get(),
			Some(ChargeFeeMethod::FeeCurrency(token))
		);

		let token_transfer = token_rate.saturating_mul_int(fee_surplus);
		System::assert_has_event(crate::mock::RuntimeEvent::Tokens(orml_tokens::Event::Transfer {
			currency_id: token,
			from: ALICE,
			to: token_subacc.clone(),
			amount: token_transfer,
		}));
		System::assert_has_event(crate::mock::RuntimeEvent::PalletBalances(
			pallet_balances::Event::Transfer {
				from: token_subacc.clone(),
				to: ALICE,
				amount: fee_surplus,
			},
		));
		assert_eq!(Currencies::free_balance(SEE, &ALICE), see_init);
		assert_eq!(Currencies::free_balance(token, &ALICE), token_init - token_transfer);

		// the actual fee not include fee surplus
		let actual_fee = TransactionPayment::compute_actual_fee(500, &INFO, &POST_INFO, 0);
		assert_eq!(actual_fee, 500 * 2 + 800);

		assert_ok!(ChargeTransactionPayment::<Runtime>::post_dispatch(
			Some(pre),
			&INFO,
			&POST_INFO,
			500,
			&Ok(())
		));

		// always clear OverrideChargeFeeMethod when post_dispatch
		assert_eq!(OverrideChargeFeeMethod::<Runtime>::get(), None);

		let refund = 200; // 1000 - 800
		let refund_surplus = surplus_percent.mul_ceil(refund);
		let actual_surplus = surplus - refund_surplus;
		assert_eq!(
			Currencies::free_balance(SEE, &ALICE),
			see_init + refund + refund_surplus
		);
		assert_eq!(
			FEE_UNBALANCED_AMOUNT.with(|a| *a.borrow()),
			fee - refund + actual_surplus
		);
		assert_eq!(TIP_UNBALANCED_AMOUNT.with(|a| *a.borrow()), 0);

		System::assert_has_event(crate::mock::RuntimeEvent::TransactionPayment(
			crate::Event::TransactionFeePaid {
				who: ALICE,
				actual_fee,
				actual_tip: 0,
				actual_surplus,
			},
		));

		// reset and test refund with tip
		FEE_UNBALANCED_AMOUNT.with(|a| *a.borrow_mut() = 0);

		assert_ok!(Currencies::update_balance(
			RuntimeOrigin::root(),
			CHARLIE,
			token,
			28000.unique_saturated_into(),
		));
		let see_init = Currencies::free_balance(SEE, &CHARLIE);
		let token_init = Currencies::free_balance(token, &CHARLIE);
		let tip: Balance = 200;
		let surplus = surplus_percent.mul_ceil(fee + tip);
		let fee_surplus = surplus + fee + tip;
		let token_transfer = token_rate.saturating_mul_int(fee_surplus);

		let pre = ChargeTransactionPayment::<Runtime>::from(tip)
			.pre_dispatch(&CHARLIE, &with_fee_currency_call(token), &INFO, 500)
			.unwrap();
		assert_eq!(pre.2, Some(pallet_balances::NegativeImbalance::new(fee_surplus)));
		assert_eq!(pre.3, fee_surplus);

		// with_fee_currency will set OverrideChargeFeeMethod when pre_dispatch
		assert_eq!(
			OverrideChargeFeeMethod::<Runtime>::get(),
			Some(ChargeFeeMethod::FeeCurrency(token))
		);

		System::assert_has_event(crate::mock::RuntimeEvent::Tokens(orml_tokens::Event::Transfer {
			currency_id: token,
			from: CHARLIE,
			to: token_subacc.clone(),
			amount: token_transfer,
		}));
		System::assert_has_event(crate::mock::RuntimeEvent::PalletBalances(
			pallet_balances::Event::Transfer {
				from: token_subacc,
				to: CHARLIE,
				amount: fee_surplus,
			},
		));
		assert_eq!(Currencies::free_balance(SEE, &CHARLIE), see_init);
		assert_eq!(Currencies::free_balance(token, &CHARLIE), token_init - token_transfer);
		let actual_fee = TransactionPayment::compute_actual_fee(500, &INFO, &POST_INFO, tip);
		assert_eq!(actual_fee, 500 * 2 + 800 + 200);
		assert_ok!(ChargeTransactionPayment::<Runtime>::post_dispatch(
			Some(pre),
			&INFO,
			&POST_INFO,
			500,
			&Ok(())
		));

		// always clear OverrideChargeFeeMethod when post_dispatch
		assert_eq!(OverrideChargeFeeMethod::<Runtime>::get(), None);

		assert_eq!(
			Currencies::free_balance(SEE, &CHARLIE),
			see_init + refund + refund_surplus
		);
		assert_eq!(
			FEE_UNBALANCED_AMOUNT.with(|a| *a.borrow()),
			fee - refund + surplus - refund_surplus
		);
		assert_eq!(TIP_UNBALANCED_AMOUNT.with(|a| *a.borrow()), tip);

		System::assert_has_event(crate::mock::RuntimeEvent::TransactionPayment(
			crate::Event::TransactionFeePaid {
				who: CHARLIE,
				actual_fee,
				actual_tip: tip,
				actual_surplus: surplus - refund_surplus,
			},
		));
	});
}

#[test]
fn pre_post_dispatch_and_refund_with_fee_currency_call_use_dex() {
	pre_post_dispatch_and_refund_with_fee_call_use_dex(with_fee_currency_call(LSEE));
}

#[test]
fn pre_post_dispatch_and_refund_with_fee_path_call_use_dex() {
	pre_post_dispatch_and_refund_with_fee_call_use_dex(with_fee_path_call(vec![LSEE, SEE]));
}

fn pre_post_dispatch_and_refund_with_fee_call_use_dex(with_fee_call: <Runtime as Config>::RuntimeCall) {
	let (token, surplus_percent) = (LSEE, CustomFeeSurplus::get());
	builder_with_dex_and_fee_pool(true).execute_with(|| {
		// without tip
		let dex_acc: AccountId = PalletId(*b"set/edfis").into_account_truncating();
		let dex_see = Currencies::free_balance(SEE, &dex_acc);

		let fee: Balance = 50 * 2 + 100; // len * byte + weight
		let surplus = surplus_percent.mul_ceil(fee); // 200*50%=100
		let fee_surplus = surplus + fee; // 300

		assert_ok!(Currencies::update_balance(RuntimeOrigin::root(), ALICE, token, 500));
		let pre = ChargeTransactionPayment::<Runtime>::from(0)
			.pre_dispatch(&ALICE, &with_fee_call, &INFO2, 50)
			.unwrap();
		assert_eq!(pre.2, Some(pallet_balances::NegativeImbalance::new(fee_surplus)));
		assert_eq!(pre.3, fee_surplus);
		System::assert_has_event(crate::mock::RuntimeEvent::DEXModule(module_dex::Event::Swap {
			trader: ALICE,
			path: vec![LSEE, SEE],
			liquidity_changes: vec![43, 300],
		}));
		assert_eq!(dex_see - 300, Currencies::free_balance(SEE, &dex_acc));

		// the actual fee not include fee surplus
		let actual_fee = TransactionPayment::compute_actual_fee(50, &INFO2, &POST_INFO2, 0);
		assert_eq!(actual_fee, 50 * 2 + 80);

		assert_ok!(ChargeTransactionPayment::<Runtime>::post_dispatch(
			Some(pre),
			&INFO2,
			&POST_INFO2,
			50,
			&Ok(())
		));

		let refund = 20; // 100 - 80
		let refund_surplus = surplus_percent.mul_ceil(refund); // 20*50%=10
		let actual_surplus = surplus - refund_surplus; // 100-10=90
		let actual_surplus_direct = surplus_percent.mul_ceil(actual_fee);
		assert_eq!(actual_surplus, actual_surplus_direct);
		System::assert_has_event(crate::mock::RuntimeEvent::TransactionPayment(
			crate::Event::TransactionFeePaid {
				who: ALICE,
				actual_fee,
				actual_tip: 0,
				actual_surplus,
			},
		));

		// with tip
		assert_ok!(Currencies::update_balance(RuntimeOrigin::root(), CHARLIE, token, 500));
		let tip: Balance = 20;
		let surplus = surplus_percent.mul_ceil(fee + tip); // 220*50%=110
		let fee_surplus = surplus + fee + tip; // 200+20+110=330
		assert_eq!(fee_surplus, 330);

		let pre = ChargeTransactionPayment::<Runtime>::from(tip)
			.pre_dispatch(&CHARLIE, &with_fee_call, &INFO2, 50)
			.unwrap();
		assert_eq!(pre.2, Some(pallet_balances::NegativeImbalance::new(fee_surplus)));
		assert_eq!(pre.3, fee_surplus);

		assert_ok!(ChargeTransactionPayment::<Runtime>::post_dispatch(
			Some(pre),
			&INFO2,
			&POST_INFO2,
			50,
			&Ok(())
		));

		let actual_fee = TransactionPayment::compute_actual_fee(50, &INFO2, &POST_INFO2, tip);
		assert_eq!(actual_fee, 50 * 2 + 80 + 20);

		let refund = 30; // 110 - 80 = 30
		let refund_surplus = surplus_percent.mul_ceil(refund); // 30*50%=15
		let actual_surplus = surplus - refund_surplus; // 110-15=95
		let actual_surplus_direct = surplus_percent.mul_ceil(actual_fee);
		assert_ne!(actual_surplus, actual_surplus_direct);
		System::assert_has_event(crate::mock::RuntimeEvent::TransactionPayment(
			crate::Event::TransactionFeePaid {
				who: CHARLIE,
				actual_fee,      // 200
				actual_tip: tip, // 20
				actual_surplus: actual_surplus_direct,
			},
		));
	});
}

#[test]
fn charges_fee_when_pre_dispatch_and_native_currency_is_enough() {
	builder_with_dex_and_fee_pool(false).execute_with(|| {
		let fee = 23 * 2 + 1000; // len * byte + weight
		assert_ok!(ChargeTransactionPayment::<Runtime>::from(0).pre_dispatch(&ALICE, &CALL, &INFO, 23));
		assert_eq!(Currencies::free_balance(SEE, &ALICE), 100000 - fee);
	});
}

#[test]
fn refund_fee_according_to_actual_when_post_dispatch_and_native_currency_is_enough() {
	builder_with_dex_and_fee_pool(false).execute_with(|| {
		let fee = 23 * 2 + 1000; // len * byte + weight
		let pre = ChargeTransactionPayment::<Runtime>::from(0)
			.pre_dispatch(&ALICE, &CALL, &INFO, 23)
			.unwrap();
		assert_eq!(Currencies::free_balance(SEE, &ALICE), 100000 - fee);

		let refund = 200; // 1000 - 800
		assert_ok!(ChargeTransactionPayment::<Runtime>::post_dispatch(
			Some(pre),
			&INFO,
			&POST_INFO,
			23,
			&Ok(())
		));
		assert_eq!(Currencies::free_balance(SEE, &ALICE), 100000 - fee + refund);

		System::assert_has_event(crate::mock::RuntimeEvent::TransactionPayment(
			crate::Event::TransactionFeePaid {
				who: ALICE,
				actual_fee: fee - refund,
				actual_tip: 0,
				actual_surplus: 0,
			},
		));
	});
}

#[test]
fn refund_tip_according_to_actual_when_post_dispatch_and_native_currency_is_enough() {
	builder_with_dex_and_fee_pool(false).execute_with(|| {
		// tip = 0
		let fee = 23 * 2 + 1000; // len * byte + weight
		let pre = ChargeTransactionPayment::<Runtime>::from(0)
			.pre_dispatch(&ALICE, &CALL, &INFO, 23)
			.unwrap();
		assert_eq!(Currencies::free_balance(SEE, &ALICE), 100000 - fee);

		let refund = 200; // 1000 - 800
		assert_ok!(ChargeTransactionPayment::<Runtime>::post_dispatch(
			Some(pre),
			&INFO,
			&POST_INFO,
			23,
			&Ok(())
		));
		assert_eq!(Currencies::free_balance(SEE, &ALICE), 100000 - fee + refund);

		System::assert_has_event(crate::mock::RuntimeEvent::TransactionPayment(
			crate::Event::TransactionFeePaid {
				who: ALICE,
				actual_fee: fee - refund,
				actual_tip: 0,
				actual_surplus: 0,
			},
		));

		// tip = 1000
		let fee = 23 * 2 + 1000; // len * byte + weight
		let tip = 1000;
		let pre = ChargeTransactionPayment::<Runtime>::from(tip)
			.pre_dispatch(&CHARLIE, &CALL, &INFO, 23)
			.unwrap();
		assert_eq!(Currencies::free_balance(SEE, &CHARLIE), 100000 - fee - tip);

		let refund_fee = 200; // 1000 - 800
		let refund_tip = 200; // 1000 - 800
		assert_ok!(ChargeTransactionPayment::<Runtime>::post_dispatch(
			Some(pre),
			&INFO,
			&POST_INFO,
			23,
			&Ok(())
		));
		assert_eq!(
			Currencies::free_balance(SEE, &CHARLIE),
			100000 - fee - tip + refund_fee + refund_tip
		);

		System::assert_has_event(crate::mock::RuntimeEvent::TransactionPayment(
			crate::Event::TransactionFeePaid {
				who: CHARLIE,
				actual_fee: fee - refund_fee + tip,
				actual_tip: tip - refund_tip,
				actual_surplus: 0,
			},
		));
	});
}

#[test]
fn refund_should_not_works() {
	builder_with_dex_and_fee_pool(false).execute_with(|| {
		let tip = 1000;
		let fee = 23 * 2 + 1000; // len * byte + weight
		let pre = ChargeTransactionPayment::<Runtime>::from(tip)
			.pre_dispatch(&ALICE, &CALL, &INFO, 23)
			.unwrap();
		assert_eq!(Currencies::free_balance(SEE, &ALICE), 100000 - fee - tip);

		// actual_weight > weight
		const POST_INFO: PostDispatchInfo = PostDispatchInfo {
			actual_weight: Some(INFO.weight.add_ref_time(1)),
			pays_fee: Pays::Yes,
		};

		assert_ok!(ChargeTransactionPayment::<Runtime>::post_dispatch(
			Some(pre),
			&INFO,
			&POST_INFO,
			23,
			&Ok(())
		));
		assert_eq!(Currencies::free_balance(SEE, &ALICE), 100000 - fee - tip);

		System::assert_has_event(crate::mock::RuntimeEvent::TransactionPayment(
			crate::Event::TransactionFeePaid {
				who: ALICE,
				actual_fee: fee + tip,
				actual_tip: tip,
				actual_surplus: 0,
			},
		));
	});
}

#[test]
fn charges_fee_when_validate_with_fee_currency_call_use_swap() {
	charges_fee_when_validate_with_fee_call_use_swap(with_fee_currency_call(LSEE));
}

#[test]
fn charges_fee_when_validate_with_fee_path_call_use_swap() {
	charges_fee_when_validate_with_fee_call_use_swap(with_fee_path_call(vec![LSEE, SEE]));
}

fn charges_fee_when_validate_with_fee_call_use_swap(with_fee_call: <Runtime as Config>::RuntimeCall) {
	// Enable dex with Alice, and initialize tx charge fee pool
	builder_with_dex_and_fee_pool(true).execute_with(|| {
		let dex_acc: AccountId = PalletId(*b"set/edfis").into_account_truncating();
		let dex_see = Currencies::free_balance(SEE, &dex_acc);

		// first tx consider existential deposit.
		// LSEE is not enabled charge fee pool, so use dex swap.
		let fee: Balance = 50 * 2 + 100 + 10;
		let fee_surplus = fee + CustomFeeSurplus::get().mul_ceil(fee);
		assert_eq!(315, fee_surplus);
		assert_ok!(Currencies::update_balance(RuntimeOrigin::root(), BOB, LSEE, 1000));

		assert_ok!(ChargeTransactionPayment::<Runtime>::from(0).validate(&BOB, &with_fee_call, &INFO2, 50));
		System::assert_has_event(crate::mock::RuntimeEvent::DEXModule(module_dex::Event::Swap {
			trader: BOB,
			path: vec![LSEE, SEE],
			liquidity_changes: vec![46, 315],
		}));
		assert_eq!(1000 - 46, Currencies::free_balance(LSEE, &BOB));
		assert_eq!(10, Currencies::free_balance(SEE, &BOB));
		assert_eq!(dex_see - 315, Currencies::free_balance(SEE, &dex_acc));

		// second tx no need to consider existential deposit.
		let fee: Balance = 50 * 2 + 100;
		let fee_surplus2 = fee + CustomFeeSurplus::get().mul_ceil(fee);
		assert_eq!(300, fee_surplus2); // refund 200*1.5=300 SEE

		assert_ok!(ChargeTransactionPayment::<Runtime>::from(0).validate(&BOB, &with_fee_call, &INFO2, 50));
		System::assert_has_event(crate::mock::RuntimeEvent::DEXModule(module_dex::Event::Swap {
			trader: BOB,
			path: vec![LSEE, SEE],
			liquidity_changes: vec![114, 300],
		}));
		assert_eq!(1000 - 46 - 114, Currencies::free_balance(LSEE, &BOB));
		assert_eq!(10, Currencies::free_balance(SEE, &BOB));
		assert_eq!(dex_see - 315 - 300, Currencies::free_balance(SEE, &dex_acc));
	});
}

#[test]
fn charges_fee_when_validate_with_fee_currency_call_use_pool() {
	// Enable dex with Alice, and initialize tx charge fee pool
	builder_with_dex_and_fee_pool(true).execute_with(|| {
		let ausd_acc = Pallet::<Runtime>::sub_account_id(USSD);
		let edf_acc = Pallet::<Runtime>::sub_account_id(EDF);
		let sub_ausd_see = Currencies::free_balance(SEE, &ausd_acc);
		let sub_ausd_usd = Currencies::free_balance(USSD, &ausd_acc);
		let sub_edf_see = Currencies::free_balance(SEE, &edf_acc);
		let sub_edf_edf = Currencies::free_balance(EDF, &edf_acc);

		// first tx consider existential deposit.
		// USSD - SEE charge fee pool: 2630 USSD - 263 SEE
		let fee: Balance = 50 * 2 + 100 + 10;
		let fee_perc = AlternativeFeeSurplus::get(); // DefaultFeeTokens: 25%
		let surplus = fee_perc.mul_ceil(fee); // 53
		let fee_amount = fee + surplus; // 263 SEE

		assert_ok!(Currencies::update_balance(RuntimeOrigin::root(), BOB, USSD, 10000));
		assert_eq!(0, Currencies::free_balance(SEE, &BOB));
		assert_ok!(ChargeTransactionPayment::<Runtime>::from(0).validate(
			&BOB,
			&with_fee_currency_call(USSD),
			&INFO2,
			50
		));
		assert_eq!(10, Currencies::free_balance(SEE, &BOB)); // ED
		assert_eq!(7370, Currencies::free_balance(USSD, &BOB));
		System::assert_has_event(crate::mock::RuntimeEvent::Tokens(orml_tokens::Event::Transfer {
			currency_id: USSD,
			from: BOB,
			to: ausd_acc.clone(),
			amount: 2630,
		}));
		System::assert_has_event(crate::mock::RuntimeEvent::PalletBalances(
			pallet_balances::Event::Transfer {
				from: ausd_acc.clone(),
				to: BOB,
				amount: 263,
			},
		));

		assert_eq!(sub_ausd_see - fee_amount, Currencies::free_balance(SEE, &ausd_acc));
		assert_eq!(
			sub_ausd_usd + fee_amount * 10, // 1 SEE = 10 USSD
			Currencies::free_balance(USSD, &ausd_acc)
		);

		// second tx no need to consider existential deposit.
		// EDF - SEE charge fee pool: 2630 USSD - 263 SEE
		let fee: Balance = 50 * 2 + 100;
		let fee_perc = CustomFeeSurplus::get(); // none default fee tokens: 50%
		let surplus = fee_perc.mul_ceil(fee);
		let fee_amount = fee + surplus; // 300 SEE
		assert_eq!(fee_amount, 300);

		assert_ok!(Currencies::update_balance(RuntimeOrigin::root(), BOB, EDF, 10000));
		assert_eq!(10, Currencies::free_balance(SEE, &BOB)); // ED
		assert_ok!(ChargeTransactionPayment::<Runtime>::from(0).validate(
			&BOB,
			&with_fee_currency_call(EDF),
			&INFO2,
			50
		));
		assert_eq!(sub_edf_see - fee_amount, Currencies::free_balance(SEE, &edf_acc));
		assert_eq!(sub_edf_edf + fee_amount / 10, Currencies::free_balance(EDF, &edf_acc));
		// 1 EDF = 10
		// SEE
	});
}

#[test]
fn charges_fee_when_validate_and_native_is_not_enough() {
	// Enable dex with Alice, and initialize tx charge fee pool
	builder_with_dex_and_fee_pool(true).execute_with(|| {
		let sub_account = Pallet::<Runtime>::sub_account_id(USSD);
		let init_balance = FeePoolSize::get();
		let ausd_ed: Balance = <Currencies as MultiCurrency<AccountId>>::minimum_balance(USSD);
		let ed: Balance = <Currencies as MultiCurrency<AccountId>>::minimum_balance(SEE);
		let rate: u128 = 10;

		// transfer token to Bob, and use Bob as tx sender to test
		// Bob do not have enough native asset(SEE), but he has USSD
		assert_ok!(<Currencies as MultiCurrency<_>>::transfer(USSD, &ALICE, &BOB, 4000));
		assert_eq!(DEXModule::get_liquidity_pool(SEE, USSD), (10000, 1000));
		assert_eq!(Currencies::total_balance(SEE, &BOB), 0);
		assert_eq!(<Currencies as MultiCurrency<_>>::free_balance(SEE, &BOB), 0);
		assert_eq!(<Currencies as MultiCurrency<_>>::free_balance(USSD, &BOB), 4000);

		// native balance is lt ED, will swap fee and ED with foreign asset
		// none surplus: fee: 200, ed: 10, swap_out:200+10=210SEE, swap_in=260*10=2100USSD
		// have surplus: fee: 200, ed: 10, surplus=200*0.25=50, swap_out:200+10+50=260SEE,
		// swap_in=260*10=2600USSD
		let fee = 50 * 2 + 100; // len * byte + weight
		let surplus1 = AlternativeFeeSurplus::get().mul_ceil(fee);
		let expect_priority = ChargeTransactionPayment::<Runtime>::get_priority(&INFO2, 50, fee, fee);
		assert_eq!(expect_priority, 2010);
		assert_eq!(
			ChargeTransactionPayment::<Runtime>::from(0)
				.validate(&BOB, &CALL2, &INFO2, 50)
				.unwrap()
				.priority,
			10
		);

		assert_eq!(Currencies::total_balance(SEE, &BOB), ed);
		assert_eq!(Currencies::free_balance(SEE, &BOB), ed);
		// surplus=50SEE/500USSD, balance=4000, swap_in=2600, left=1400
		// surplus=0, balance=4000, swap_in=2100, left=1900
		assert_eq!(Currencies::free_balance(USSD, &BOB), 1900 - surplus1 * 10);
		assert_eq!(DEXModule::get_liquidity_pool(SEE, USSD), (10000, 1000));
		assert_eq!(
			Currencies::free_balance(SEE, &sub_account),
			init_balance - (fee + ed + surplus1)
		);
		assert_eq!(
			Currencies::free_balance(USSD, &sub_account),
			ausd_ed + (fee + ed + surplus1) * rate
		);

		// native balance is eq ED, cannot keep alive after charge, swap with foreign asset
		// fee: 112, ed: 10, surplus=110*0.25=28, swap_out:112+28=140SEE, swap_in=260*10=1400USSD
		let fee2 = 6 * 2 + 100; // len * byte + weight
		let surplus2 = AlternativeFeeSurplus::get().mul_ceil(fee2);
		assert_ok!(ChargeTransactionPayment::<Runtime>::from(0).validate(&BOB, &CALL2, &INFO2, 6));
		assert_eq!(Currencies::total_balance(SEE, &BOB), ed);
		assert_eq!(Currencies::free_balance(SEE, &BOB), ed);
		assert_eq!(
			Currencies::free_balance(USSD, &BOB),
			1900 - (surplus1 + fee2 + surplus2) * 10
		);
		assert_eq!(
			Currencies::free_balance(SEE, &sub_account),
			init_balance - (fee + ed + surplus1) - (fee2 + surplus2)
		);
		// two tx, first receive: (fee+ED+surplus)*10, second receive: (fee2+surplus)*10
		assert_eq!(
			Currencies::free_balance(USSD, &sub_account),
			ausd_ed + (fee + ed + surplus1 + fee2 + surplus2) * rate
		);

		// Bob only has ED of native asset, but has not enough USSD, validate failed.
		assert_noop!(
			ChargeTransactionPayment::<Runtime>::from(0).validate(&BOB, &CALL2, &INFO2, 1),
			TransactionValidityError::Invalid(InvalidTransaction::Payment)
		);
		assert_eq!(Currencies::total_balance(SEE, &BOB), 10);
		assert_eq!(Currencies::free_balance(SEE, &BOB), 10);
		assert_eq!(
			Currencies::free_balance(USSD, &BOB),
			1900 - (surplus1 + fee2 + surplus2) * 10
		);
	});
}

#[test]
fn payment_reserve_fee() {
	builder_with_dex_and_fee_pool(true).execute_with(|| {
		// Alice has enough native token: SEE
		let alice_see_init = 89000;
		assert_eq!(alice_see_init, Currencies::free_balance(SEE, &ALICE));
		let fee = <ChargeTransactionPayment<Runtime> as TransactionPaymentT<AccountId, Balance, _>>::reserve_fee(
			&ALICE, 100, None,
		);
		assert_eq!(100, fee.unwrap());
		assert_eq!(alice_see_init - 100, Currencies::free_balance(SEE, &ALICE));

		let reserves = crate::mock::PalletBalances::reserves(&ALICE);
		let reserve_data = ReserveData {
			id: ReserveIdentifier::TransactionPayment,
			amount: 100,
		};
		assert_eq!(reserve_data, *reserves.get(0).unwrap());

		// Bob has not enough native token, but have enough none native token
		assert_ok!(<Currencies as MultiCurrency<_>>::transfer(USSD, &ALICE, &BOB, 4000));
		let fee = <ChargeTransactionPayment<Runtime> as TransactionPaymentT<AccountId, Balance, _>>::reserve_fee(
			&BOB, 100, None,
		);
		assert_eq!(100, fee.unwrap());
		assert_eq!(35, Currencies::free_balance(SEE, &BOB));
		assert_eq!(135, Currencies::total_balance(SEE, &BOB));
		assert_eq!(2650, Currencies::free_balance(USSD, &BOB));

		// reserve fee not consider multiplier
		NextFeeMultiplier::<Runtime>::put(Multiplier::saturating_from_rational(3, 2));
		assert_ok!(<Currencies as MultiCurrency<_>>::transfer(USSD, &ALICE, &DAVE, 4000));
		let fee = <ChargeTransactionPayment<Runtime> as TransactionPaymentT<AccountId, Balance, _>>::reserve_fee(
			&DAVE, 100, None,
		);
		assert_eq!(100, fee.unwrap());

		let fee =
			<ChargeTransactionPayment<Runtime> as TransactionPaymentT<AccountId, Balance, _>>::apply_multiplier_to_fee(
				100, None,
			);
		assert_eq!(150, fee);
		let fee =
			<ChargeTransactionPayment<Runtime> as TransactionPaymentT<AccountId, Balance, _>>::apply_multiplier_to_fee(
				100,
				Some(Multiplier::saturating_from_rational(2, 1)),
			);
		assert_eq!(200, fee);
	});
}

#[test]
fn charges_fee_failed_by_slippage_limit() {
	builder_with_dex_and_fee_pool(true).execute_with(|| {
		assert_ok!(<Currencies as MultiCurrency<_>>::transfer(USSD, &ALICE, &BOB, 1000));

		assert_eq!(DEXModule::get_liquidity_pool(SEE, USSD), (10000, 1000));
		assert_eq!(Currencies::total_balance(SEE, &BOB), 0);
		assert_eq!(<Currencies as MultiCurrency<_>>::free_balance(SEE, &BOB), 0);
		assert_eq!(<Currencies as MultiCurrency<_>>::free_balance(USSD, &BOB), 1000);

		assert_eq!(
			DEXModule::get_swap_amount(&vec![USSD, SEE], SwapLimit::ExactTarget(Balance::MAX, 2010)),
			Some((252, 2010))
		);
		assert_eq!(
			DEXModule::get_swap_amount(&vec![USSD, SEE], SwapLimit::ExactSupply(1000, 0)),
			Some((1000, 5000))
		);

		// pool is enough, but slippage limit the swap
		MockPriceSource::set_relative_price(Some(Price::saturating_from_rational(252, 4020)));
		assert_eq!(
			DEXModule::get_swap_amount(&vec![USSD, SEE], SwapLimit::ExactTarget(Balance::MAX, 2010)),
			Some((252, 2010))
		);
		assert_eq!(
			DEXModule::get_swap_amount(&vec![USSD, SEE], SwapLimit::ExactSupply(1000, 0)),
			Some((1000, 5000))
		);
		assert_noop!(
			ChargeTransactionPayment::<Runtime>::from(0).validate(&BOB, &CALL2, &INFO, 500),
			TransactionValidityError::Invalid(InvalidTransaction::Payment)
		);
		assert_eq!(DEXModule::get_liquidity_pool(SEE, USSD), (10000, 1000));
	});
}

#[test]
fn set_alternative_fee_swap_path_work() {
	ExtBuilder::default()
		.one_hundred_thousand_for_alice_n_charlie()
		.build()
		.execute_with(|| {
			assert_eq!(TransactionPayment::alternative_fee_swap_path(&ALICE), None);
			assert_ok!(TransactionPayment::set_alternative_fee_swap_path(
				RuntimeOrigin::signed(ALICE),
				Some(vec![USSD, SEE])
			));
			assert_eq!(
				TransactionPayment::alternative_fee_swap_path(&ALICE).unwrap(),
				vec![USSD, SEE]
			);
			assert_ok!(TransactionPayment::set_alternative_fee_swap_path(
				RuntimeOrigin::signed(ALICE),
				None
			));
			assert_eq!(TransactionPayment::alternative_fee_swap_path(&ALICE), None);

			assert_noop!(
				TransactionPayment::set_alternative_fee_swap_path(RuntimeOrigin::signed(ALICE), Some(vec![SEE])),
				Error::<Runtime>::InvalidSwapPath
			);

			assert_noop!(
				TransactionPayment::set_alternative_fee_swap_path(RuntimeOrigin::signed(ALICE), Some(vec![USSD, EDF])),
				Error::<Runtime>::InvalidSwapPath
			);

			assert_noop!(
				TransactionPayment::set_alternative_fee_swap_path(RuntimeOrigin::signed(ALICE), Some(vec![SEE, SEE])),
				Error::<Runtime>::InvalidSwapPath
			);
		});
}

#[test]
fn charge_fee_by_alternative_swap_first_priority() {
	builder_with_dex_and_fee_pool(true).execute_with(|| {
		let sub_account = Pallet::<Runtime>::sub_account_id(EDF);
		let init_balance = FeePoolSize::get();
		let edf_ed = Currencies::minimum_balance(EDF);
		let ed = Currencies::minimum_balance(SEE);
		let alternative_fee_swap_deposit: u128 =
			<<Runtime as Config>::AlternativeFeeSwapDeposit as frame_support::traits::Get<u128>>::get();

		assert_eq!(DEXModule::get_liquidity_pool(SEE, USSD), (10000, 1000));
		assert_eq!(DEXModule::get_liquidity_pool(EDF, USSD), (100, 1000));
		assert_ok!(Currencies::update_balance(
			RuntimeOrigin::root(),
			BOB,
			SEE,
			(alternative_fee_swap_deposit + PalletBalances::minimum_balance())
				.try_into()
				.unwrap(),
		));

		assert_ok!(TransactionPayment::set_alternative_fee_swap_path(
			RuntimeOrigin::signed(BOB),
			Some(vec![EDF, USSD, SEE])
		));
		assert_eq!(
			TransactionPayment::alternative_fee_swap_path(&BOB).unwrap(),
			vec![EDF, USSD, SEE]
		);
		// the `AlternativeFeeSwapDeposit` amount balance is in user reserve balance,
		// user reserve balance is not consider when check native is enough or not.
		assert_eq!(
			alternative_fee_swap_deposit + PalletBalances::minimum_balance(),
			Currencies::total_balance(SEE, &BOB)
		);

		// charge fee token use `DefaultFeeTokens` as `AlternativeFeeSwapPath` condition is failed.
		assert_ok!(<Currencies as MultiCurrency<_>>::transfer(EDF, &ALICE, &BOB, 300));
		assert_eq!(
			<Currencies as MultiCurrency<_>>::free_balance(SEE, &BOB),
			PalletBalances::minimum_balance()
		);
		assert_eq!(<Currencies as MultiCurrency<_>>::free_balance(USSD, &BOB), 0);
		assert_eq!(<Currencies as MultiCurrency<_>>::free_balance(EDF, &BOB), 300);

		// use user's free_balance to check native is enough or not:
		// fee=500*2+1000=2000SEE, surplus=2000*0.25=500SEE, fee_amount=2500SEE
		let surplus: u128 = AlternativeFeeSurplus::get().mul_ceil(2000);
		let fee_surplus: u128 = 2000 + surplus;
		assert_eq!(
			ChargeTransactionPayment::<Runtime>::from(0)
				.validate(&BOB, &CALL2, &INFO, 500)
				.unwrap()
				.priority,
			1
		);
		System::assert_has_event(crate::mock::RuntimeEvent::DEXModule(module_dex::Event::Swap {
			trader: BOB,
			path: vec![EDF, USSD, SEE],
			liquidity_changes: vec![51, 334, fee_surplus],
		}));

		assert_eq!(Currencies::free_balance(SEE, &BOB), ed);
		assert_eq!(Currencies::free_balance(USSD, &BOB), 0);
		assert_eq!(Currencies::free_balance(EDF, &BOB), 249);
		assert_eq!(DEXModule::get_liquidity_pool(SEE, USSD), (7500, 1334));
		assert_eq!(DEXModule::get_liquidity_pool(EDF, USSD), (151, 666));
		assert_eq!(Currencies::free_balance(SEE, &sub_account), init_balance,);
		assert_eq!(Currencies::free_balance(EDF, &sub_account), edf_ed);
	});
}

#[test]
fn charge_fee_by_default_fee_tokens_second_priority() {
	builder_with_dex_and_fee_pool(true).execute_with(|| {
		let sub_account = Pallet::<Runtime>::sub_account_id(EDF);
		let init_balance = FeePoolSize::get();
		let edf_ed = Currencies::minimum_balance(EDF);
		let ed = Currencies::minimum_balance(SEE);
		let alternative_fee_swap_deposit: u128 =
			<<Runtime as Config>::AlternativeFeeSwapDeposit as frame_support::traits::Get<u128>>::get();

		assert_ok!(Currencies::update_balance(
			RuntimeOrigin::root(),
			BOB,
			SEE,
			(alternative_fee_swap_deposit + PalletBalances::minimum_balance())
				.try_into()
				.unwrap(),
		));

		assert_ok!(TransactionPayment::set_alternative_fee_swap_path(
			RuntimeOrigin::signed(BOB),
			Some(vec![EDF, USSD, SEE])
		));
		assert_eq!(
			TransactionPayment::alternative_fee_swap_path(&BOB).unwrap(),
			vec![EDF, USSD, SEE]
		);
		// the `AlternativeFeeSwapDeposit` amount balance is in user reserve balance,
		// user reserve balance is not consider when check native is enough or not.
		assert_eq!(
			alternative_fee_swap_deposit + PalletBalances::minimum_balance(),
			Currencies::total_balance(SEE, &BOB)
		);

		// charge fee token use `AlternativeFeeSwapPath`, although the swap path is invalid.
		assert_ok!(<Currencies as MultiCurrency<_>>::transfer(EDF, &ALICE, &BOB, 300));
		assert_eq!(
			<Currencies as MultiCurrency<_>>::free_balance(SEE, &BOB),
			PalletBalances::minimum_balance()
		);
		assert_eq!(<Currencies as MultiCurrency<_>>::free_balance(USSD, &BOB), 0);
		assert_eq!(<Currencies as MultiCurrency<_>>::free_balance(EDF, &BOB), 300);
		assert_eq!(Currencies::free_balance(SEE, &sub_account), init_balance,);
		assert_eq!(Currencies::free_balance(EDF, &sub_account), edf_ed);

		// use user's total_balance to check native is enough or not:
		// fee=500*2+1000=2000SEE, surplus=2000*0.25=500SEE, fee_amount=2500SEE
		let surplus: u128 = AlternativeFeeSurplus::get().mul_ceil(2000);
		let fee_surplus = 2000 + surplus;
		assert_eq!(
			ChargeTransactionPayment::<Runtime>::from(0)
				.validate(&BOB, &CALL2, &INFO, 500)
				.unwrap()
				.priority,
			1
		);
		// Alternative fee swap directly from dex, not from fee pool.
		System::assert_has_event(crate::mock::RuntimeEvent::DEXModule(module_dex::Event::Swap {
			trader: BOB,
			path: vec![EDF, USSD, SEE],
			liquidity_changes: vec![51, 334, fee_surplus],
		}));

		assert_eq!(Currencies::free_balance(SEE, &BOB), ed);
		assert_eq!(Currencies::free_balance(USSD, &BOB), 0);
		assert_eq!(Currencies::free_balance(EDF, &BOB), 249);
		assert_eq!(DEXModule::get_liquidity_pool(SEE, USSD), (7500, 1334));
		assert_eq!(DEXModule::get_liquidity_pool(EDF, USSD), (151, 666));
		// sub-account balance not changed, because not passing through sub-account.
		assert_eq!(Currencies::free_balance(SEE, &sub_account), init_balance,);
		assert_eq!(Currencies::free_balance(EDF, &sub_account), edf_ed);
	});
}

#[test]
fn query_info_works() {
	ExtBuilder::default()
		.base_weight(Weight::from_parts(5, 0))
		.byte_fee(1)
		.weight_fee(2)
		.build()
		.execute_with(|| {
			let call = RuntimeCall::PalletBalances(pallet_balances::Call::transfer_allow_death {
				dest: AccountId::new([2u8; 32]),
				value: 69,
			});
			let origin = 111111;
			let extra = ();
			let xt = TestXt::new(call, Some((origin, extra)));
			let info = xt.get_dispatch_info();
			let ext = xt.encode();
			let len = ext.len() as u32;

			// all fees should be x1.5
			NextFeeMultiplier::<Runtime>::put(Multiplier::saturating_from_rational(3, 2));

			assert_eq!(
				TransactionPayment::query_info(xt, len),
				RuntimeDispatchInfo {
					weight: info.weight,
					class: info.class,
					partial_fee: 5 * 2 /* base_weight * weight_fee */
						+ len as u128  /* len * byte_fee */
						+ info.weight.ref_time().min(BlockWeights::get().max_block.ref_time()) as u128 * 2 * 3 / 2 /* weight */
				},
			);
		});
}

#[test]
fn compute_fee_works_without_multiplier() {
	ExtBuilder::default()
		.base_weight(Weight::from_parts(100, 0))
		.byte_fee(10)
		.build()
		.execute_with(|| {
			// Next fee multiplier is zero
			assert_eq!(NextFeeMultiplier::<Runtime>::get(), Multiplier::one());

			// Tip only, no fees works
			let dispatch_info = DispatchInfo {
				weight: Weight::from_parts(0, 0),
				class: DispatchClass::Operational,
				pays_fee: Pays::No,
			};
			assert_eq!(Pallet::<Runtime>::compute_fee(0, &dispatch_info, 10), 10);
			// No tip, only base fee works
			let dispatch_info = DispatchInfo {
				weight: Weight::from_parts(0, 0),
				class: DispatchClass::Operational,
				pays_fee: Pays::Yes,
			};
			assert_eq!(Pallet::<Runtime>::compute_fee(0, &dispatch_info, 0), 100);
			// Tip + base fee works
			assert_eq!(Pallet::<Runtime>::compute_fee(0, &dispatch_info, 69), 169);
			// Len (byte fee) + base fee works
			assert_eq!(Pallet::<Runtime>::compute_fee(42, &dispatch_info, 0), 520);
			// Weight fee + base fee works
			let dispatch_info = DispatchInfo {
				weight: Weight::from_parts(1000, 0),
				class: DispatchClass::Operational,
				pays_fee: Pays::Yes,
			};
			assert_eq!(Pallet::<Runtime>::compute_fee(0, &dispatch_info, 0), 1100);
		});
}

#[test]
fn compute_fee_works_with_multiplier() {
	ExtBuilder::default()
		.base_weight(Weight::from_parts(100, 0))
		.byte_fee(10)
		.build()
		.execute_with(|| {
			// Add a next fee multiplier. Fees will be x3/2.
			NextFeeMultiplier::<Runtime>::put(Multiplier::saturating_from_rational(3, 2));
			// Base fee is unaffected by multiplier
			let dispatch_info = DispatchInfo {
				weight: Weight::from_parts(0, 0),
				class: DispatchClass::Operational,
				pays_fee: Pays::Yes,
			};
			assert_eq!(Pallet::<Runtime>::compute_fee(0, &dispatch_info, 0), 100);

			// Everything works together :)
			let dispatch_info = DispatchInfo {
				weight: Weight::from_parts(123, 0),
				class: DispatchClass::Operational,
				pays_fee: Pays::Yes,
			};
			// 123 weight, 456 length, 100 base
			assert_eq!(
				Pallet::<Runtime>::compute_fee(456, &dispatch_info, 789),
				100 + (3 * 123 / 2) + 4560 + 789,
			);
		});
}

#[test]
fn compute_fee_works_with_negative_multiplier() {
	ExtBuilder::default()
		.base_weight(Weight::from_parts(100, 0))
		.byte_fee(10)
		.build()
		.execute_with(|| {
			// Add a next fee multiplier. All fees will be x1/2.
			NextFeeMultiplier::<Runtime>::put(Multiplier::saturating_from_rational(1, 2));

			// Base fee is unaffected by multiplier.
			let dispatch_info = DispatchInfo {
				weight: Weight::from_parts(0, 0),
				class: DispatchClass::Operational,
				pays_fee: Pays::Yes,
			};
			assert_eq!(Pallet::<Runtime>::compute_fee(0, &dispatch_info, 0), 100);

			// Everything works together.
			let dispatch_info = DispatchInfo {
				weight: Weight::from_parts(123, 0),
				class: DispatchClass::Operational,
				pays_fee: Pays::Yes,
			};
			// 123 weight, 456 length, 100 base
			assert_eq!(
				Pallet::<Runtime>::compute_fee(456, &dispatch_info, 789),
				100 + (123 / 2) + 4560 + 789,
			);
		});
}

#[test]
fn compute_fee_does_not_overflow() {
	ExtBuilder::default()
		.base_weight(Weight::from_parts(100, 0))
		.byte_fee(10)
		.build()
		.execute_with(|| {
			// Overflow is handled
			let dispatch_info = DispatchInfo {
				weight: Weight::MAX,
				class: DispatchClass::Operational,
				pays_fee: Pays::Yes,
			};
			assert_eq!(
				Pallet::<Runtime>::compute_fee(<u32>::max_value(), &dispatch_info, <u128>::max_value()),
				<u128>::max_value()
			);
		});
}

#[test]
fn should_alter_operational_priority() {
	let tip = 5;
	let len = 10;

	ExtBuilder::default()
		.one_hundred_thousand_for_alice_n_charlie()
		.build()
		.execute_with(|| {
			let normal = DispatchInfo {
				weight: Weight::from_parts(100, 0),
				class: DispatchClass::Normal,
				pays_fee: Pays::Yes,
			};
			let priority = ChargeTransactionPayment::<Runtime>(tip)
				.validate(&ALICE, &CALL, &normal, len)
				.unwrap()
				.priority;

			assert_eq!(priority, 60);

			let priority = ChargeTransactionPayment::<Runtime>(2 * tip)
				.validate(&ALICE, &CALL, &normal, len)
				.unwrap()
				.priority;

			assert_eq!(priority, 110);
		});

	ExtBuilder::default()
		.one_hundred_thousand_for_alice_n_charlie()
		.build()
		.execute_with(|| {
			let op = DispatchInfo {
				weight: Weight::from_parts(100, 0),
				class: DispatchClass::Operational,
				pays_fee: Pays::Yes,
			};
			let priority = ChargeTransactionPayment::<Runtime>(tip)
				.validate(&ALICE, &CALL, &op, len)
				.unwrap()
				.priority;
			// final_fee = base_fee + len_fee + adjusted_weight_fee + tip = 0 + 20 + 100 + 5 = 125
			// priority = final_fee * fee_multiplier * max_tx_per_block + (tip + 1) * max_tx_per_block
			//          = 125 * 5 * 10 + 60 = 6310
			assert_eq!(priority, 6310);

			let priority = ChargeTransactionPayment::<Runtime>(2 * tip)
				.validate(&ALICE, &CALL, &op, len)
				.unwrap()
				.priority;
			// final_fee = base_fee + len_fee + adjusted_weight_fee + tip = 0 + 20 + 100 + 10 = 130
			// priority = final_fee * fee_multiplier * max_tx_per_block + (tip + 1) * max_tx_per_block
			//          = 130 * 5 * 10 + 110 = 6610
			assert_eq!(priority, 6610);
		});
}

#[test]
fn no_tip_has_some_priority() {
	let tip = 0;
	let len = 10;

	ExtBuilder::default()
		.one_hundred_thousand_for_alice_n_charlie()
		.build()
		.execute_with(|| {
			let normal = DispatchInfo {
				weight: Weight::from_parts(100, 0),
				class: DispatchClass::Normal,
				pays_fee: Pays::Yes,
			};
			let priority = ChargeTransactionPayment::<Runtime>(tip)
				.validate(&ALICE, &CALL, &normal, len)
				.unwrap()
				.priority;
			// max_tx_per_block = 10
			assert_eq!(priority, 10);
		});

	ExtBuilder::default()
		.one_hundred_thousand_for_alice_n_charlie()
		.build()
		.execute_with(|| {
			let op = DispatchInfo {
				weight: Weight::from_parts(100, 0),
				class: DispatchClass::Operational,
				pays_fee: Pays::Yes,
			};
			let priority = ChargeTransactionPayment::<Runtime>(tip)
				.validate(&ALICE, &CALL, &op, len)
				.unwrap()
				.priority;
			// final_fee = base_fee + len_fee + adjusted_weight_fee + tip = 0 + 20 + 100 + 0 = 120
			// priority = final_fee * fee_multiplier * max_tx_per_block + (tip + 1) * max_tx_per_block
			//          = 120 * 5 * 10 + 10 = 6010
			assert_eq!(priority, 6010);
		});
}

#[test]
fn min_tip_has_same_priority() {
	let tip = 100;
	let len = 10;

	ExtBuilder::default()
		.tip_per_weight_step(tip)
		.one_hundred_thousand_for_alice_n_charlie()
		.build()
		.execute_with(|| {
			let normal = DispatchInfo {
				weight: Weight::from_parts(100, 0),
				class: DispatchClass::Normal,
				pays_fee: Pays::Yes,
			};
			let priority = ChargeTransactionPayment::<Runtime>(0)
				.validate(&ALICE, &CALL, &normal, len)
				.unwrap()
				.priority;
			// max_tx_per_block = 10
			assert_eq!(priority, 0);

			let priority = ChargeTransactionPayment::<Runtime>(tip - 2)
				.validate(&ALICE, &CALL, &normal, len)
				.unwrap()
				.priority;
			// max_tx_per_block = 10
			assert_eq!(priority, 0);

			let priority = ChargeTransactionPayment::<Runtime>(tip - 1)
				.validate(&ALICE, &CALL, &normal, len)
				.unwrap()
				.priority;
			// max_tx_per_block = 10
			assert_eq!(priority, 10);

			let priority = ChargeTransactionPayment::<Runtime>(tip)
				.validate(&ALICE, &CALL, &normal, len)
				.unwrap()
				.priority;
			// max_tx_per_block = 10
			assert_eq!(priority, 10);

			let priority = ChargeTransactionPayment::<Runtime>(2 * tip - 2)
				.validate(&ALICE, &CALL, &normal, len)
				.unwrap()
				.priority;
			// max_tx_per_block = 10
			assert_eq!(priority, 10);

			let priority = ChargeTransactionPayment::<Runtime>(2 * tip - 1)
				.validate(&ALICE, &CALL, &normal, len)
				.unwrap()
				.priority;
			// max_tx_per_block = 10
			assert_eq!(priority, 20);

			let priority = ChargeTransactionPayment::<Runtime>(2 * tip)
				.validate(&ALICE, &CALL, &normal, len)
				.unwrap()
				.priority;
			// max_tx_per_block = 10
			assert_eq!(priority, 20);
		});
}

#[test]
fn max_tip_has_same_priority() {
	let tip = 1000;
	let len = 10;

	ExtBuilder::default()
		.one_hundred_thousand_for_alice_n_charlie()
		.build()
		.execute_with(|| {
			let normal = DispatchInfo {
				weight: Weight::from_parts(100, 0),
				class: DispatchClass::Normal,
				pays_fee: Pays::Yes,
			};
			let priority = ChargeTransactionPayment::<Runtime>(tip)
				.validate(&ALICE, &CALL, &normal, len)
				.unwrap()
				.priority;
			// max_tx_per_block = 10
			assert_eq!(priority, 10_000);

			let priority = ChargeTransactionPayment::<Runtime>(2 * tip)
				.validate(&ALICE, &CALL, &normal, len)
				.unwrap()
				.priority;
			// max_tx_per_block = 10
			assert_eq!(priority, 10_000);
		});
}

struct CurrencyIdConvert;
impl Convert<MultiLocation, Option<CurrencyId>> for CurrencyIdConvert {
	fn convert(location: MultiLocation) -> Option<CurrencyId> {
		use CurrencyId::Token;
		use TokenSymbol::*;

		if location == MultiLocation::parent() {
			return Some(Token(EDF));
		}

		match location {
			MultiLocation {
				interior: X1(GeneralKey { data, length }),
				..
			} => {
				let key = &data[..data.len().min(length as usize)];
				CurrencyId::decode(&mut &*key).ok()
			}
			_ => None,
		}
	}
}

#[test]
fn buy_weight_transaction_fee_pool_works() {
	builder_with_dex_and_fee_pool(true).execute_with(|| {
		// Location convert return None.
		let location = MultiLocation::new(1, X1(Junction::Parachain(2000)));
		let rate = <BuyWeightRateOfTransactionFeePool<Runtime, CurrencyIdConvert>>::calculate_rate(location);
		assert_eq!(rate, None);

		// Token not in charge fee pool
		let currency_id = CurrencyId::Token(TokenSymbol::LSEE);

		let location = MultiLocation::new(
			1,
			X1(Junction::from(BoundedVec::try_from(currency_id.encode()).unwrap())),
		);
		let rate = <BuyWeightRateOfTransactionFeePool<Runtime, CurrencyIdConvert>>::calculate_rate(location);
		assert_eq!(rate, None);

		// EDF Token is in charge fee pool.
		let location = MultiLocation::parent();
		let rate = <BuyWeightRateOfTransactionFeePool<Runtime, CurrencyIdConvert>>::calculate_rate(location);
		assert_eq!(rate, Some(Ratio::saturating_from_rational(1, 10)));
	});
}

#[test]
fn swap_from_pool_not_enough_currency() {
	builder_with_dex_and_fee_pool(true).execute_with(|| {
		let balance = 100 as u128;
		assert_ok!(Currencies::update_balance(
			RuntimeOrigin::root(),
			BOB,
			EDF,
			balance.unique_saturated_into(),
		));
		assert_ok!(Currencies::update_balance(
			RuntimeOrigin::root(),
			BOB,
			USSD,
			balance.unique_saturated_into(),
		));
		assert_eq!(Currencies::free_balance(EDF, &BOB), 100);
		assert_eq!(Currencies::free_balance(USSD, &BOB), 100);

		// 1100 SEE equals to 110 EDF, but Bob only has 100 EDF
		let result = Pallet::<Runtime>::swap_from_pool_or_dex(&BOB, 1100, EDF);
		assert!(result.is_err());
		// 11 SEE equals to 110 USSD, but Bob only has 100 USSD
		let result = Pallet::<Runtime>::swap_from_pool_or_dex(&BOB, 11, USSD);
		assert!(result.is_err());
	});
}

#[test]
fn swap_from_pool_with_enough_balance() {
	builder_with_dex_and_fee_pool(true).execute_with(|| {
		let pool_size = FeePoolSize::get();
		let edf_fee_account = Pallet::<Runtime>::sub_account_id(EDF);
		let usd_fee_account = Pallet::<Runtime>::sub_account_id(USSD);
		let edf_ed = <Currencies as MultiCurrency<AccountId>>::minimum_balance(EDF);
		let usd_ed = <Currencies as MultiCurrency<AccountId>>::minimum_balance(USSD);

		// 1 EDF = 10 SEE, swap 500 SEE with 50 EDF
		let balance = 500 as u128;
		assert_ok!(Currencies::update_balance(
			RuntimeOrigin::root(),
			BOB,
			EDF,
			balance.unique_saturated_into(),
		));
		let fee = balance; // 500 SEE
		let expect_treasury_edf = (balance / 10) as u128; // 50 EDF
		let expect_user_edf = balance - expect_treasury_edf; // 450 EDF
		let expect_treasury_see = (pool_size - fee) as u128; // 500 SEE
		let expect_user_see = fee; // 500 SEE

		assert_ok!(Pallet::<Runtime>::swap_from_pool_or_dex(&BOB, fee, EDF));
		assert_eq!(expect_user_edf, Currencies::free_balance(EDF, &BOB));
		assert_eq!(
			expect_treasury_edf,
			Currencies::free_balance(EDF, &edf_fee_account) - edf_ed
		);
		assert_eq!(expect_user_see, Currencies::free_balance(SEE, &BOB));
		assert_eq!(expect_treasury_see, Currencies::free_balance(SEE, &edf_fee_account));

		// 1 SEE = 10 USSD, swap 500 SEE with 5000 USSD
		let balance = 500 as u128;
		let ausd_balance = (balance * 11) as u128; // 5500 USSD
		assert_ok!(Currencies::update_balance(
			RuntimeOrigin::root(),
			BOB,
			USSD,
			ausd_balance.unique_saturated_into(),
		));
		assert_eq!(0, Currencies::free_balance(USSD, &usd_fee_account) - usd_ed);
		let fee = balance; // 500 SEE
		let expect_treasury_ausd = (balance * 10) as u128; // 5000 USSD
		let expect_user_ausd = balance; // (balance * 11) - (balance * 10) = balance = 500 USSD
		let expect_treasury_see = pool_size - fee; // 1000 SEE - 500 SEE
		let expect_user_see = expect_user_see + fee; // 500 SEE

		assert_ok!(Pallet::<Runtime>::swap_from_pool_or_dex(&BOB, fee, USSD));
		assert_eq!(expect_user_ausd, Currencies::free_balance(USSD, &BOB));
		assert_eq!(
			expect_treasury_ausd,
			Currencies::free_balance(USSD, &usd_fee_account) - usd_ed
		);
		assert_eq!(expect_user_see, Currencies::free_balance(SEE, &BOB));
		assert_eq!(expect_treasury_see, Currencies::free_balance(SEE, &usd_fee_account));
	});
}

#[test]
fn swap_from_pool_and_dex_with_higher_threshold() {
	builder_with_dex_and_fee_pool(true).execute_with(|| {
		let pool_size = FeePoolSize::get();
		let edf_fee_account = Pallet::<Runtime>::sub_account_id(EDF);
		let edf_ed = <Currencies as MultiCurrency<AccountId>>::minimum_balance(EDF);

		// Bob has 800 EDF, the fee is 800 SEE, equal to 80 EDF
		let balance = 800 as u128;
		let fee_edf = 80 as u128;
		assert_ok!(Currencies::update_balance(
			RuntimeOrigin::root(),
			BOB,
			EDF,
			balance.unique_saturated_into(),
		));

		// First transaction success get 800 SEE as fee from pool
		Pallet::<Runtime>::swap_from_pool_or_dex(&BOB, balance, EDF).unwrap();
		// Bob withdraw 80 EDF(remain 720), and deposit 800 SEE
		assert_eq!(balance - fee_edf, Currencies::free_balance(EDF, &BOB));
		assert_eq!(balance, Currencies::free_balance(SEE, &BOB));
		// sub account deposit 80 EDF, and withdraw 800 SEE(remain 9200)
		assert_eq!(fee_edf + edf_ed, Currencies::free_balance(EDF, &edf_fee_account));
		assert_eq!(pool_size - balance, Currencies::free_balance(SEE, &edf_fee_account));

		let old_exchange_rate = TokenExchangeRate::<Runtime>::get(EDF).unwrap();
		assert_eq!(old_exchange_rate, Ratio::saturating_from_rational(fee_edf, balance));

		// Set threshold(init-500) gt sub account balance(init-800), trigger swap from dex.
		SwapBalanceThreshold::<Runtime>::insert(EDF, crate::mock::HigerSwapThreshold::get());
		SwapBalanceThreshold::<Runtime>::insert(USSD, crate::mock::HigerSwapThreshold::get());

		// swap 80 EDF out 3074 SEE
		let trading_path = DotFeeSwapPath::get();
		let supply_amount = Currencies::free_balance(EDF, &edf_fee_account) - edf_ed;
		// here just get swap out amount, the swap not happened
		let (supply_in_amount, swap_out_native) =
			module_dex::Pallet::<Runtime>::get_swap_amount(&trading_path, SwapLimit::ExactSupply(supply_amount, 0))
				.unwrap();
		assert_eq!(3074, swap_out_native);
		assert_eq!(supply_in_amount, supply_amount);
		let new_pool_size =
			(swap_out_native + Currencies::free_balance(SEE, &edf_fee_account)).saturated_into::<u128>();

		// the swap also has it's own exchange rate by input_amount divide output_amount
		let swap_exchange_rate = Ratio::saturating_from_rational(supply_in_amount, swap_out_native);
		assert_eq!(swap_exchange_rate, Ratio::saturating_from_rational(80, 3074));

		// swap_rate=80/3074, threshold=9500, pool_size=10000, threshold_rate=0.95, old_rate=1/10
		// new_rate = 1/10 * 0.95 + 80/3074 * 0.05 = 0.095 + 0.001301236174365 = 0.096301236174365
		let new_exchange_rate_val =
			Ratio::saturating_from_rational(9_630_123_6174_365_647 as u128, 1_000_000_000_000_000_000 as u128);

		// the sub account has 9200 SEE, 80 EDF, use 80 EDF to swap out some SEE
		let balance2 = 300 as u128;
		assert_ok!(Pallet::<Runtime>::swap_from_pool_or_dex(&BOB, balance2, EDF));
		System::assert_has_event(crate::mock::RuntimeEvent::TransactionPayment(
			crate::Event::ChargeFeePoolSwapped {
				sub_account: edf_fee_account,
				supply_currency_id: EDF,
				old_exchange_rate,
				swap_exchange_rate,
				new_exchange_rate: new_exchange_rate_val,
				new_pool_size,
			},
		));

		let new_rate = TokenExchangeRate::<Runtime>::get(EDF).unwrap();
		assert_eq!(new_exchange_rate_val, new_rate);
		assert_eq!(PoolSize::<Runtime>::get(EDF), new_pool_size);
	});
}

#[test]
fn swap_from_pool_and_dex_with_midd_threshold() {
	builder_with_dex_and_fee_pool(true).execute_with(|| {
		let sub_account: AccountId = <Runtime as Config>::PalletId::get().into_sub_account_truncating(EDF);
		let edf_ed = <Currencies as MultiCurrency<AccountId>>::minimum_balance(EDF);
		let trading_path = vec![EDF, USSD, SEE];

		// the pool size has 10000 SEE, and set threshold to half of pool size: 5000 SEE
		let balance = 3000 as u128;
		assert_ok!(Currencies::update_balance(
			RuntimeOrigin::root(),
			BOB,
			EDF,
			balance.unique_saturated_into(),
		));

		SwapBalanceThreshold::<Runtime>::insert(EDF, crate::mock::MiddSwapThreshold::get());
		SwapBalanceThreshold::<Runtime>::insert(USSD, crate::mock::MiddSwapThreshold::get());

		// After tx#1, SEE balance of sub account is large than threshold(5000 SEE)
		Pallet::<Runtime>::swap_from_pool_or_dex(&BOB, balance, EDF).unwrap();
		assert_eq!(Currencies::free_balance(SEE, &sub_account), 7000);
		assert_eq!(Currencies::free_balance(EDF, &sub_account), 301);

		// After tx#2, SEE balance of sub account is less than threshold(5000 SEE)
		Pallet::<Runtime>::swap_from_pool_or_dex(&BOB, balance, EDF).unwrap();
		assert_eq!(Currencies::free_balance(SEE, &sub_account), 4000);
		assert_eq!(Currencies::free_balance(EDF, &sub_account), 601);

		let supply_amount = Currencies::free_balance(EDF, &sub_account) - edf_ed;
		// Given different EDF, get the swap out SEE
		// EDF | SEE  | SwapRate
		// 001 | 0089 | FixedU128(0.011235955056179775)
		// 050 | 2498 | FixedU128(0.020016012810248198)
		// 100 | 3333 | FixedU128(0.030003000300030003)
		// 200 | 3997 | FixedU128(0.050037528146109582)
		// 300 | 4285 | FixedU128(0.070011668611435239)
		// 500 | 4544 | FixedU128(0.110035211267605633)
		// 600 | 4614 | FixedU128(0.130039011703511053) <- this case hit here
		let (supply_in_amount, swap_out_native) =
			module_dex::Pallet::<Runtime>::get_swap_amount(&trading_path, SwapLimit::ExactSupply(supply_amount, 0))
				.unwrap();
		assert_eq!(600, supply_in_amount);
		assert_eq!(4614, swap_out_native);
		// new pool size = swap_out_native + SEE balance of sub account
		let new_pool_size = swap_out_native + 4000;

		// When execute tx#3, trigger swap from dex, but this tx still use old rate(1/10)
		Pallet::<Runtime>::swap_from_pool_or_dex(&BOB, balance, EDF).unwrap();
		assert_eq!(Currencies::free_balance(SEE, &sub_account), 5614); // 4000+4614-3000=5614
		assert_eq!(Currencies::free_balance(EDF, &sub_account), 301);

		let old_exchange_rate = Ratio::saturating_from_rational(1, 10);
		let swap_exchange_rate = Ratio::saturating_from_rational(supply_in_amount, swap_out_native);
		// (0.1 + 0.130039011703511053)/2 = 0.230039011703511053/2 = 0.115019505851755526
		let new_exchange_rate_val =
			Ratio::saturating_from_rational(115_019_505_851_755_526 as u128, 1_000_000_000_000_000_000 as u128);

		System::assert_has_event(crate::mock::RuntimeEvent::TransactionPayment(
			crate::Event::ChargeFeePoolSwapped {
				sub_account: sub_account.clone(),
				supply_currency_id: EDF,
				old_exchange_rate,
				swap_exchange_rate,
				new_exchange_rate: new_exchange_rate_val,
				new_pool_size,
			},
		));

		// tx#3 use new exchange rate
		Pallet::<Runtime>::swap_from_pool_or_dex(&BOB, balance, EDF).unwrap();
		assert_eq!(Currencies::free_balance(SEE, &sub_account), 2614);
		assert_eq!(Currencies::free_balance(EDF, &sub_account), 301 + 3 * 115);
	});
}

#[test]
#[should_panic(expected = "Swap tx fee pool should not fail!")]
fn charge_fee_failed_when_disable_dex() {
	use module_dex::TradingPairStatus;
	use primitives::TradingPair;

	ExtBuilder::default().build().execute_with(|| {
		let fee_account = Pallet::<Runtime>::sub_account_id(USSD);
		let pool_size = FeePoolSize::get();
		let swap_balance_threshold = (pool_size - 200) as u128;
		let ausd_ed = <Currencies as MultiCurrency<AccountId>>::minimum_balance(USSD);
		let ed = <Currencies as MultiCurrency<AccountId>>::minimum_balance(SEE);
		let trading_path = AusdFeeSwapPath::get();

		assert_ok!(Currencies::update_balance(
			RuntimeOrigin::root(),
			BOB,
			USSD,
			100000.unique_saturated_into(),
		));

		// tx failed because of dex not enabled even though user has enough USSD
		assert_noop!(
			ChargeTransactionPayment::<Runtime>::from(0).validate(&BOB, &CALL2, &INFO2, 50),
			TransactionValidityError::Invalid(InvalidTransaction::Payment)
		);

		enable_dex_and_tx_fee_pool();

		// after runtime upgrade, tx success because of dex enabled and has enough token balance
		// fee=50*2+100=200, ED=10, surplus=200*0.25=50, fee_amount=260, ausd_swap=260*10=2600
		let surplus = AlternativeFeeSurplus::get().mul_ceil(200);
		assert_ok!(ChargeTransactionPayment::<Runtime>::from(0).validate(&BOB, &CALL2, &INFO2, 50));
		assert_eq!(100000 - (210 + surplus) * 10, Currencies::free_balance(USSD, &BOB));

		// update threshold, next tx will trigger swap
		SwapBalanceThreshold::<Runtime>::insert(USSD, swap_balance_threshold);

		// trading pair is enabled
		let pair = TradingPair::from_currency_ids(USSD, SEE).unwrap();
		assert_eq!(
			module_dex::Pallet::<Runtime>::trading_pair_statuses(pair),
			TradingPairStatus::Enabled
		);
		// make sure swap is valid
		let swap_result = module_dex::Pallet::<Runtime>::get_swap_amount(&trading_path, SwapLimit::ExactSupply(1, 0));
		assert!(swap_result.is_some());
		assert_ok!(module_dex::Pallet::<Runtime>::swap_with_specific_path(
			&ALICE,
			&trading_path,
			SwapLimit::ExactSupply(100, 0)
		));

		// balance lt threshold, trigger swap from dex
		assert_eq!(
			ausd_ed + (210 + surplus) * 10,
			Currencies::free_balance(USSD, &fee_account)
		);
		assert_eq!(9790 - surplus, Currencies::free_balance(SEE, &fee_account));
		assert_ok!(ChargeTransactionPayment::<Runtime>::from(0).validate(&BOB, &CALL2, &INFO2, 50));
		// AlternativeFeeSurplus=25%, swap 2600 USSD with 6388 SEE, pool_size=9740+6388=16128
		// fee=50*2+100=200, surplus=200*0.25=50, fee_amount=250, ausd_swap=250*10=2500
		let fee_see = Currencies::free_balance(SEE, &fee_account);
		assert_eq!(
			ausd_ed + (200 + surplus) * 10,
			Currencies::free_balance(USSD, &fee_account)
		);
		if AlternativeFeeSurplus::get() == Percent::from_percent(25) {
			// pool_size=16128, one tx cost SEE=250(with surplus), result=16128-250=15878
			assert_eq!(15878, fee_see);
			System::assert_has_event(crate::mock::RuntimeEvent::TransactionPayment(
				crate::Event::ChargeFeePoolSwapped {
					sub_account: fee_account.clone(),
					supply_currency_id: USSD,
					old_exchange_rate: Ratio::saturating_from_rational(10, 1),
					swap_exchange_rate: Ratio::saturating_from_rational(
						407_013_149_655_604_257 as u128,
						1_000_000_000_000_000_000 as u128,
					),
					new_exchange_rate: Ratio::saturating_from_rational(
						9_808_140_262_993_112_085 as u128,
						1_000_000_000_000_000_000 as u128,
					),
					new_pool_size: 16128,
				},
			));
		} else if AlternativeFeeSurplus::get() == Percent::from_percent(0) {
			// pool_size=15755, one tx cost SEE=200(without surplus), result=15755-200=15555
			assert_eq!(15555, fee_see);
			System::assert_has_event(crate::mock::RuntimeEvent::TransactionPayment(
				crate::Event::ChargeFeePoolSwapped {
					sub_account: fee_account.clone(),
					supply_currency_id: USSD,
					old_exchange_rate: Ratio::saturating_from_rational(10, 1),
					swap_exchange_rate: Ratio::saturating_from_rational(
						352053646269907795 as u128,
						1_000_000_000_000_000_000 as u128,
					),
					new_exchange_rate: Ratio::saturating_from_rational(
						9807041072925398155 as u128,
						1_000_000_000_000_000_000 as u128,
					),
					new_pool_size: 15755,
				},
			));
		}

		// when trading pair disabled, the swap action will failed
		assert_ok!(module_dex::Pallet::<Runtime>::disable_trading_pair(
			RuntimeOrigin::signed(AccountId::new([0u8; 32])),
			USSD,
			SEE
		));
		assert_eq!(
			module_dex::Pallet::<Runtime>::trading_pair_statuses(pair),
			TradingPairStatus::Disabled
		);
		let res = module_dex::Pallet::<Runtime>::swap_with_specific_path(
			&ALICE,
			&trading_path,
			SwapLimit::ExactSupply(100, 0),
		);
		assert!(res.is_err());

		// but `swap_from_pool_or_dex` still can work, because tx fee pool is not disabled.
		// after swap, the balance gt threshold, tx still success because not trigger swap.
		// the rate is using new exchange rate, but swap native asset still keep 250 SEE.
		assert_ok!(ChargeTransactionPayment::<Runtime>::from(0).validate(&BOB, &CALL2, &INFO2, 50));

		let fee_balance = Currencies::free_balance(SEE, &fee_account);
		assert_eq!(fee_see - (200 + surplus), fee_balance);
		assert_eq!(fee_balance > swap_balance_threshold, true);
		let swap_balance_threshold = (fee_balance - 199) as u128;

		SwapBalanceThreshold::<Runtime>::insert(USSD, swap_balance_threshold);

		// this tx success because before execution, native_balance > threshold
		assert_ok!(ChargeTransactionPayment::<Runtime>::from(0).validate(&BOB, &CALL2, &INFO2, 50));
		// assert_eq!(15378, Currencies::free_balance(SEE, &fee_account));
		assert_eq!(
			fee_see - (200 + surplus) * 2,
			Currencies::free_balance(SEE, &fee_account)
		);
		assert_eq!(ed, Currencies::free_balance(SEE, &BOB));

		// this tx failed because when execute, native_balance < threshold, the dex swap failed
		assert_ok!(ChargeTransactionPayment::<Runtime>::from(0).validate(&BOB, &CALL2, &INFO2, 50));
	});
}

#[test]
fn charge_fee_pool_operation_works() {
	ExtBuilder::default().build().execute_with(|| {
		let alternative_fee_swap_deposit: u128 =
			<<Runtime as Config>::AlternativeFeeSwapDeposit as frame_support::traits::Get<u128>>::get();
		assert_ok!(Currencies::update_balance(
			RuntimeOrigin::root(),
			ALICE,
			SEE,
			alternative_fee_swap_deposit.try_into().unwrap(),
		));
		assert_ok!(TransactionPayment::set_alternative_fee_swap_path(
			RuntimeOrigin::signed(ALICE),
			Some(vec![USSD, SEE])
		));
		assert_eq!(
			TransactionPayment::alternative_fee_swap_path(&ALICE).unwrap(),
			vec![USSD, SEE]
		);

		assert_ok!(Currencies::update_balance(
			RuntimeOrigin::root(),
			ALICE,
			SEE,
			10000.unique_saturated_into(),
		));

		assert_ok!(DEXModule::add_liquidity(
			RuntimeOrigin::signed(ALICE),
			SEE,
			USSD,
			10000,
			1000,
			0,
			false
		));

		let treasury_account: AccountId = <Runtime as Config>::TreasuryAccount::get();
		let sub_account: AccountId = <Runtime as Config>::PalletId::get().into_sub_account_truncating(USSD);
		let usd_ed = <Currencies as MultiCurrency<AccountId>>::minimum_balance(USSD);
		let pool_size = FeePoolSize::get();
		let swap_threshold = crate::mock::MiddSwapThreshold::get();

		assert_ok!(Currencies::update_balance(
			RuntimeOrigin::root(),
			treasury_account.clone(),
			SEE,
			(pool_size * 2).unique_saturated_into(),
		));
		assert_ok!(Currencies::update_balance(
			RuntimeOrigin::root(),
			treasury_account.clone(),
			USSD,
			(usd_ed * 2).unique_saturated_into(),
		));

		assert_ok!(Pallet::<Runtime>::enable_charge_fee_pool(
			RuntimeOrigin::signed(ALICE),
			USSD,
			pool_size,
			swap_threshold
		));
		let rate = TokenExchangeRate::<Runtime>::get(USSD);
		assert_eq!(rate, Some(Ratio::saturating_from_rational(2, 10)));
		System::assert_has_event(crate::mock::RuntimeEvent::TransactionPayment(
			crate::Event::ChargeFeePoolEnabled {
				sub_account: sub_account.clone(),
				currency_id: USSD,
				exchange_rate: Ratio::saturating_from_rational(2, 10),
				pool_size,
				swap_threshold,
			},
		));

		assert_noop!(
			Pallet::<Runtime>::enable_charge_fee_pool(RuntimeOrigin::signed(ALICE), USSD, pool_size, swap_threshold),
			Error::<Runtime>::ChargeFeePoolAlreadyExisted
		);

		assert_noop!(
			Pallet::<Runtime>::enable_charge_fee_pool(RuntimeOrigin::signed(ALICE), LEDF, pool_size, swap_threshold),
			Error::<Runtime>::DexNotAvailable
		);
		assert_noop!(
			Pallet::<Runtime>::disable_charge_fee_pool(RuntimeOrigin::signed(ALICE), LEDF),
			Error::<Runtime>::InvalidToken
		);

		let ausd_amount1 = <Currencies as MultiCurrency<AccountId>>::free_balance(USSD, &sub_account);
		let see_amount1 = crate::mock::PalletBalances::free_balance(&sub_account);
		assert_ok!(Pallet::<Runtime>::disable_charge_fee_pool(
			RuntimeOrigin::signed(ALICE),
			USSD
		));
		assert_eq!(TokenExchangeRate::<Runtime>::get(USSD), None);
		System::assert_has_event(crate::mock::RuntimeEvent::TransactionPayment(
			crate::Event::ChargeFeePoolDisabled {
				currency_id: USSD,
				foreign_amount: ausd_amount1,
				native_amount: see_amount1,
			},
		));
		let ausd_amount2 = <Currencies as MultiCurrency<AccountId>>::free_balance(USSD, &sub_account);
		let see_amount2 = crate::mock::PalletBalances::free_balance(&sub_account);
		assert_eq!(see_amount2, 0);
		assert_eq!(ausd_amount2, 0);

		assert_ok!(Pallet::<Runtime>::enable_charge_fee_pool(
			RuntimeOrigin::signed(ALICE),
			USSD,
			pool_size,
			swap_threshold
		));
	});
}

#[test]
fn with_fee_call_validation_works() {
	ExtBuilder::default()
		.one_hundred_thousand_for_alice_n_charlie()
		.build()
		.execute_with(|| {
			assert_ok!(Currencies::update_balance(
				RuntimeOrigin::root(),
				CHARLIE,
				USSD,
				1000000,
			));
			assert_ok!(Currencies::update_balance(RuntimeOrigin::root(), CHARLIE, EDF, 1000000,));
			assert_ok!(Currencies::update_balance(
				RuntimeOrigin::root(),
				CHARLIE,
				LSEE,
				1000000,
			));
			assert_eq!(1000000, Currencies::free_balance(USSD, &CHARLIE));
			assert_eq!(1000000, Currencies::free_balance(EDF, &CHARLIE));
			assert_eq!(1000000, Currencies::free_balance(LSEE, &CHARLIE));

			assert_eq!(OverrideChargeFeeMethod::<Runtime>::get(), None);

			// CHARLIE has enough native token, default charge fee succeed
			assert_ok!(ChargeTransactionPayment::<Runtime>::from(0).pre_dispatch(&CHARLIE, &CALL, &INFO, 10));
			// default charge fee will not set OverrideChargeFeeMethod
			assert_eq!(OverrideChargeFeeMethod::<Runtime>::get(), None);

			// BOB has not enough native token, default charge fee fail
			assert_noop!(
				ChargeTransactionPayment::<Runtime>::from(0).pre_dispatch(&BOB, &CALL, &INFO, 10),
				TransactionValidityError::Invalid(InvalidTransaction::Payment)
			);

			// dex swap not enabled, validate failed.
			// with_fee_currency test
			for token in vec![EDF, USSD] {
				assert_noop!(
					ChargeTransactionPayment::<Runtime>::from(0).pre_dispatch(
						&CHARLIE,
						&with_fee_currency_call(token),
						&INFO,
						10
					),
					TransactionValidityError::Invalid(InvalidTransaction::Payment)
				);

				// ensure_can_charge_fee_with_call failed, edf not set OverrideChargeFeeMethod
				assert_eq!(OverrideChargeFeeMethod::<Runtime>::get(), None);
			}

			// test the wrapped call by with_fee_currency
			assert_ok!(TransactionPayment::with_fee_currency(
				RuntimeOrigin::signed(ALICE),
				EDF,
				Box::new(CALL),
			));
			assert_eq!(9900, Currencies::free_balance(USSD, &ALICE));
			assert_eq!(100, Currencies::free_balance(USSD, &BOB));

			// with_fee_path test
			for path in vec![vec![EDF, USSD, SEE], vec![USSD, SEE]] {
				assert_noop!(
					ChargeTransactionPayment::<Runtime>::from(0).pre_dispatch(
						&CHARLIE,
						&with_fee_path_call(path.clone()),
						&INFO,
						10
					),
					TransactionValidityError::Invalid(InvalidTransaction::Payment)
				);

				// ensure_can_charge_fee_with_call failed, edf not set OverrideChargeFeeMethod
				assert_eq!(OverrideChargeFeeMethod::<Runtime>::get(), None);
			}

			// test the wrapped call by with_fee_currency
			assert_ok!(TransactionPayment::with_fee_path(
				RuntimeOrigin::signed(ALICE),
				vec![EDF, USSD, SEE],
				Box::new(CALL),
			));
			assert_eq!(9800, Currencies::free_balance(USSD, &ALICE));
			assert_eq!(200, Currencies::free_balance(USSD, &BOB));

			// with_fee_aggregated_path test
			let aggregated_path = vec![AggregatedSwapPath::Dex(vec![EDF, USSD])];
			assert_noop!(
				ChargeTransactionPayment::<Runtime>::from(0).pre_dispatch(
					&ALICE,
					&with_fee_aggregated_path_by_call(aggregated_path.clone()),
					&INFO,
					10
				),
				TransactionValidityError::Invalid(InvalidTransaction::Payment)
			);

			let aggregated_path = vec![AggregatedSwapPath::Dex(vec![EDF, SEE])];
			assert_noop!(
				ChargeTransactionPayment::<Runtime>::from(0).pre_dispatch(
					&CHARLIE,
					&with_fee_aggregated_path_by_call(aggregated_path.clone()),
					&INFO,
					10
				),
				TransactionValidityError::Invalid(InvalidTransaction::Payment)
			);
			// ensure_can_charge_fee_with_call failed, edf not set OverrideChargeFeeMethod
			assert_eq!(OverrideChargeFeeMethod::<Runtime>::get(), None);

			let aggregated_path = vec![AggregatedSwapPath::Taiga(0, 0, 0)];
			assert_noop!(
				ChargeTransactionPayment::<Runtime>::from(0).pre_dispatch(
					&CHARLIE,
					&with_fee_aggregated_path_by_call(aggregated_path.clone()),
					&INFO,
					10
				),
				TransactionValidityError::Invalid(InvalidTransaction::Payment)
			);
			// ensure_can_charge_fee_with_call failed, edf not set OverrideChargeFeeMethod
			assert_eq!(OverrideChargeFeeMethod::<Runtime>::get(), None);

			// test the wrapped call by with_fee_aggregated_path
			assert_ok!(TransactionPayment::with_fee_aggregated_path(
				RuntimeOrigin::signed(ALICE),
				aggregated_path,
				Box::new(CALL),
			));
			assert_eq!(9700, Currencies::free_balance(USSD, &ALICE));
			assert_eq!(300, Currencies::free_balance(USSD, &BOB));

			// enable dex and enable USSD, EDF as fee pool
			enable_dex_and_tx_fee_pool();

			// with_fee_currency test
			for token in vec![EDF, USSD] {
				assert_ok!(ChargeTransactionPayment::<Runtime>::from(0).pre_dispatch(
					&CHARLIE,
					&with_fee_currency_call(token),
					&INFO,
					10
				));

				// OverrideChargeFeeMethod will be set
				assert_eq!(
					OverrideChargeFeeMethod::<Runtime>::get(),
					Some(ChargeFeeMethod::FeeCurrency(token))
				);
			}

			// LSEE is not enabled fee pool, cannot charge fee by with_fee_currency
			assert_noop!(
				ChargeTransactionPayment::<Runtime>::from(0).pre_dispatch(
					&CHARLIE,
					&with_fee_currency_call(LSEE),
					&INFO,
					10
				),
				TransactionValidityError::Invalid(InvalidTransaction::Payment)
			);

			for path in vec![vec![EDF, USSD, SEE], vec![USSD, SEE]] {
				assert_ok!(ChargeTransactionPayment::<Runtime>::from(0).pre_dispatch(
					&CHARLIE,
					&with_fee_path_call(path.clone()),
					&INFO,
					10
				));

				// OverrideChargeFeeMethod will be set
				assert_eq!(
					OverrideChargeFeeMethod::<Runtime>::get(),
					Some(ChargeFeeMethod::FeeAggregatedPath(vec![AggregatedSwapPath::Dex(path)]))
				);
			}

			let aggregated_path = vec![AggregatedSwapPath::Dex(vec![EDF, USSD, SEE])];
			assert_ok!(ChargeTransactionPayment::<Runtime>::from(0).pre_dispatch(
				&CHARLIE,
				&with_fee_aggregated_path_by_call(aggregated_path.clone()),
				&INFO,
				10
			));
			assert_eq!(
				OverrideChargeFeeMethod::<Runtime>::get(),
				Some(ChargeFeeMethod::FeeAggregatedPath(aggregated_path))
			);
		});
}
