//! Unit tests for the serp-tes module.

#![cfg(test)]
use super::*;
use frame_support::{assert_noop, assert_ok};
use mock::*;
use sp_core::H160;
use sp_runtime::traits::BadOrigin;

use traits::SettCurrency;

#[test]
fn it_works_for_default_value() {
    new_test_ext().execute_with(|| {
        // Dispatch a signed extrinsic.
        assert_ok!(SerpTes::do_something(Origin::signed(1), 42));
        // Read pallet storage and assert an expected result.
        assert_eq!(SerpTes::something(), Some(42));
    });
}

#[test]
fn correct_error_for_none_value() {
    new_test_ext().execute_with(|| {
        // Ensure the expected error is thrown when no value is present.
        assert_noop!(
            SerpTes::cause_error(Origin::signed(1)),
            Error::<Test>::NoneValue
        );
    });
}

// ------------------------------------------------------------
// expand and contract tests
#[test]
fn expand_supply_test() {
	new_test_ext().execute_with(|| {
		// payout of 120% of BaseUnit
		let payout = Fixed64::from_rational(20, 100).saturated_multiply_accumulate(BaseUnit::get());
		add_dinar(SettCurrency::new_dinar(2, payout));
		add_dinar(SettCurrency::new_dinar(3, payout));
		add_dinar(SettCurrency::new_dinar(4, payout));
		add_dinar(SettCurrency::new_dinar(5, 7 * payout));

		let prev_supply = SettCurrency::settcurrency_supply();
		let amount = 13 * BaseUnit::get();
		assert_ok!(SettCurrency::expand_supply(prev_supply, amount));

		let amount_per_acc = InitialSupply::get() / 10 + BaseUnit::get() / 10;
		assert_eq!(SettCurrency::get_balance(1), amount_per_acc);
		assert_eq!(SettCurrency::get_balance(2), amount_per_acc + payout);
		assert_eq!(SettCurrency::get_balance(3), amount_per_acc + payout);
		assert_eq!(SettCurrency::get_balance(4), amount_per_acc + payout);
		assert_eq!(SettCurrency::get_balance(5), amount_per_acc + 7 * payout);
		assert_eq!(SettCurrency::get_balance(8), amount_per_acc);
		assert_eq!(SettCurrency::get_balance(10), amount_per_acc);

		assert_eq!(
			SettCurrency::settcurrency_supply(),
			prev_supply + amount,
			"supply should be increased by amount"
		);
	});
}

#[test]
fn contract_supply_test() {
	new_test_ext().execute_with(|| {
		let dinar_amount = Ratio::new(125, 100)
			.checked_mul(&BaseUnit::get().into())
			.map(|r| r.to_integer())
			.expect("dinar_amount should not have overflowed");
		NativeCurrency::add_bid(Bid::new(1, Perbill::from_percent(80), dinar_amount));
		NativeCurrency::add_bid(Bid::new(2, Perbill::from_percent(75), 2 * BaseUnit::get()));
		SettCurrency::add_bid(Bid::new(1, Perbill::from_percent(80), dinar_amount));
		SettCurrency::add_bid(Bid::new(2, Perbill::from_percent(75), 2 * BaseUnit::get()));

		let prev_supply = SettCurrency::settcurrency_supply(SETT_USD_ID);
		let amount = 2 * BaseUnit::get();
		assert_ok!(SettCurrency::contract_supply(prev_supply, amount));

		assert_ok!(<Stp258 as ExtendedSettCurrency<AccountId>>::settcurrency_supply(
				NATIVE_SETT_USD_ID,
			));

		let bids = SettCurrency::dinar_bids(NATIVE_CURRENCY_ID);
		assert_eq!(bids.len(), 1, "exactly one bid should have been removed");
		let remainging_bid_quantity = Fixed64::from_rational(667, 1_000)
			.saturated_multiply_accumulate(BaseUnit::get())
			- BaseUnit::get();
		assert_eq!(
			bids[0],
			Bid::new(2, Perbill::from_percent(75), remainging_bid_quantity)
		);

		let (start, _) = SettCurrency::dinar_range();
		assert_eq!(SettCurrency::get_dinar(start).payout, dinar_amount);
		assert_eq!(
			SettCurrency::get_dinar(start + 1).payout,
			Fixed64::from_rational(333, 1_000).saturated_multiply_accumulate(BaseUnit::get())
		);

		assert_eq!(
			SettCurrency::settcurrency_supply(SETT_USD_ID),
			prev_supply - amount,
			"supply should be decreased by amount"
		);
	})
}

#[test]
fn serp_elast_quickcheck() {
	fn property(dinar: Vec<(u64, u64)>, prices: Vec<SettCurrency>) -> TestResult {
		new_test_ext().execute_with(|| {
			if prices.iter().any(|p| p == &0) {
				return TestResult::discard();
			}

			for (account, payout) in dinar {
				if account > 0 && payout > 0 {
					add_dinar(SettCurrency::new_dinar(account, payout));
				}
			}

			for price in prices {
				// this assert might actually produce a false positive
				// as there might be errors returned that are the correct
				// behavior for the given parameters
				assert!(matches!(
					SettCurrency::serp_elast(price),
					Ok(())
						| Err(DispatchError::Module {
							index: 0,
							error: 0,
							message: Some("SettCurrencySupplyOverflow")
						})
				));
			}

			TestResult::passed()
		})
	}

	QuickCheck::new()
		.min_tests_passed(5)
		.tests(50)
		.max_tests(500)
		.quickcheck(property as fn(Vec<(u64, u64)>, Vec<u64>) -> TestResult)
}

#[test]
fn serp_elast_smoketest() {
	new_test_ext().execute_with(|| {
		let mut rng = rand::thread_rng();

		let dinar: Vec<(u64, u64)> = (0..100)
			.map(|_| (rng.gen_range(1, 200), rng.gen_range(1, 10 * BaseUnit::get())))
			.collect();

		for (account, payout) in dinar {
			add_dinar(SettCurrency::new_dinar(account, payout));
		}

		for _ in 0..150 {
			let price = RandomPrice::fetch_price();
			SettCurrency::on_block_with_price(0, price).unwrap_or_else(|e| {
				log::error!("could not adjust supply: {:?}", e);
			});
		}
	})
}

#[test]
fn supply_change_calculation() {
	let price = TEST_BASE_UNIT + 100;
	let supply = u64::max_value();
	let contract_by = SettCurrency::calculate_supply_change(price, TEST_BASE_UNIT, supply);
	// the error should be low enough
	assert_ge!(contract_by, u64::max_value() / 10 - 1);
	assert_le!(contract_by, u64::max_value() / 10 + 1);
}

// ------------------------------------------------------------
// dinar
#[test]
fn adding_dinar() {
	new_test_ext().execute_with(|| {
		let payout = Fixed64::from_rational(20, 100).saturated_multiply_accumulate(BaseUnit::get());
		add_dinar(SettCurrency::new_dinar(3, payout));

		let (start, length) = SettCurrency::dinar_range();
		// computing the length this way is fine because there was no overflow
		assert_eq!(length, 1);
		let dinar = &SettCurrency::get_dinar(start);
	})
}

#[test]
fn expand_supply() {
	new_test_ext_with(vec![1]).execute_with(|| {
		let first_acc = 3;
		let prev_first_acc_balance = SettCurrency::get_balance(first_acc);
		// 1.2 * BaseUnit
		let payout = Fixed64::from_rational(20, 100).saturated_multiply_accumulate(BaseUnit::get());
		add_dinar(SettCurrency::new_dinar(first_acc, payout));

		let (start, length) = SettCurrency::dinar_range();
		// computing the length this way is fine because there was no overflow
		assert_eq!(length, 1);
		let dinar = &SettCurrency::get_dinar(start);

		let prev_supply = SettCurrency::settcurrency_supply();
		let second_acc = first_acc + 1;
		let prev_second_acc_balance = SettCurrency::get_balance(second_acc);
		// Add a new dinar
		add_dinar(SettCurrency::new_dinar(second_acc, payout));
		add_dinar(SettCurrency::new_dinar(second_acc, payout));
		add_dinar(SettCurrency::new_dinar(second_acc, payout));
		// Note: this one is from first_acc
		add_dinar(SettCurrency::new_dinar(first_acc, payout));

		// check dinar length
		let (_, length) = SettCurrency::dinar_range();
		// computing the length this way is fine because there was no overflow
		assert_eq!(length, 5);
		// expand the supply, only hitting the last dinar that was added to the queue, but not fully filling it
		let new_SettCurrency = payout;
		assert_ok!(SettCurrency::expand_supply(SettCurrency::settcurrency_supply(), new_settcurrency));
		// make sure there are only four dinar left (the first one got consumed)
		let (_, length) = SettCurrency::dinar_range();
		// computing the length this way is fine because there was no overflow
		assert_eq!(length, 3);
		// make sure the first account's balance hasn't changed
		assert_eq!(prev_first_acc_balance, SettCurrency::get_balance(first_acc));
		// make sure the second account's balance has increased by `new_settcurrency`
		let intermediate_second_acc_balance = prev_second_acc_balance + new_settcurrency;
		assert_eq!(
			prev_second_acc_balance + new_settcurrency,
			SettCurrency::get_balance(second_acc)
		);
		// make sure total supply increased by `new_settcurrency`
		assert_eq!(prev_supply + new_settcurrency, SettCurrency::settcurrency_supply());

		let intermediate_supply = SettCurrency::settcurrency_supply();

		// try to expand_supply
		let new_settcurrency = 42;
		assert_ok!(SettCurrency::expand_supply(intermediate_supply, new_settcurrency));

		// make sure there are no dinar left
		let (_, length) = SettCurrency::dinar_range();
		// computing the length this way is fine because there was no overflow
		assert_eq!(length, 0);

		// make sure first and second's balances haven't changed
		assert_eq!(prev_first_acc_balance, SettCurrency::get_balance(first_acc));
		assert_eq!(
			intermediate_second_acc_balance,
			SettCurrency::get_balance(second_acc)
		);

		// Make sure settcurrency supply has increased by `new_settcurrency`
		assert_eq!(
			intermediate_supply + new_settcurrency,
			SettCurrency::settcurrency_supply(),
		);
	});
}

// ------------------------------------------------------------
// handout tests

#[test]
fn simple_handout_test() {
	new_test_ext().execute_with(|| {
		let balance_per_acc = InitialSupply::get() / 10;
		assert_eq!(SettCurrency::get_balance(1), balance_per_acc);
		assert_eq!(SettCurrency::get_balance(10), balance_per_acc);

		let amount = 30 * BaseUnit::get();
		assert_ok!(SettCurrency::hand_out_settcurrency(
			&SettCurrency::shares(),
			amount,
			SettCurrency::settcurrency_supply()
		));

		let amount_per_acc = 3 * BaseUnit::get();
		assert_eq!(SettCurrency::get_balance(1), balance_per_acc + amount_per_acc);
		assert_eq!(SettCurrency::get_balance(2), balance_per_acc + amount_per_acc);
		assert_eq!(SettCurrency::get_balance(3), balance_per_acc + amount_per_acc);
		assert_eq!(SettCurrency::get_balance(7), balance_per_acc + amount_per_acc);
		assert_eq!(SettCurrency::get_balance(10), balance_per_acc + amount_per_acc);
	});
}

#[test]
fn handout_less_than_shares_test() {
	new_test_ext().execute_with(|| {
		let balance_per_acc = InitialSupply::get() / 10;
		assert_eq!(SettCurrency::get_balance(1), balance_per_acc);
		assert_eq!(SettCurrency::get_balance(10), balance_per_acc);

		let amount = 8;
		assert_ok!(SettCurrency::hand_out_settcurrency(
			&SettCurrency::shares(),
			amount,
			SettCurrency::settcurrency_supply()
		));

		let amount_per_acc = 1;
		assert_eq!(SettCurrency::get_balance(1), balance_per_acc + amount_per_acc);
		assert_eq!(SettCurrency::get_balance(2), balance_per_acc + amount_per_acc);
		assert_eq!(SettCurrency::get_balance(3), balance_per_acc + amount_per_acc);
		assert_eq!(SettCurrency::get_balance(7), balance_per_acc + amount_per_acc);
		assert_eq!(SettCurrency::get_balance(8), balance_per_acc + amount_per_acc);
		assert_eq!(SettCurrency::get_balance(9), balance_per_acc);
		assert_eq!(SettCurrency::get_balance(10), balance_per_acc);
	});
}

#[test]
fn handout_more_than_shares_test() {
	new_test_ext().execute_with(|| {
		let balance_per_acc = InitialSupply::get() / 10;
		assert_eq!(SettCurrency::get_balance(1), balance_per_acc);
		assert_eq!(SettCurrency::get_balance(10), balance_per_acc);

		let amount = 13;
		assert_ok!(SettCurrency::hand_out_settcurrency(
			&SettCurrency::shares(),
			amount,
			SettCurrency::settcurrency_supply()
		));

		let amount_per_acc = 1;
		assert_eq!(SettCurrency::get_balance(1), balance_per_acc + amount_per_acc + 1);
		assert_eq!(SettCurrency::get_balance(2), balance_per_acc + amount_per_acc + 1);
		assert_eq!(SettCurrency::get_balance(3), balance_per_acc + amount_per_acc + 1);
		assert_eq!(SettCurrency::get_balance(4), balance_per_acc + amount_per_acc);
		assert_eq!(SettCurrency::get_balance(8), balance_per_acc + amount_per_acc);
		assert_eq!(SettCurrency::get_balance(10), balance_per_acc + amount_per_acc);
	});
}

#[test]
fn handout_quickcheck() {
	fn property(shareholders: Vec<AccountId>, amount: SettCurrency) -> TestResult {
		let len = shareholders.len();
		if amount == 0 {
			return TestResult::discard();
		}
		// Expects between 1 and 999 shareholders.
		if len < 1 || len > 999 {
			return TestResult::discard();
		}
		// 0 is not a valid AccountId
		if shareholders.iter().any(|s| *s == 0) {
			return TestResult::discard();
		}
		// make sure shareholders are distinct
		if shareholders.iter().unique().count() != len {
			return TestResult::discard();
		}

		let first = shareholders[0];

		new_test_ext_with(shareholders).execute_with(|| {
			let amount = amount;
			// this assert might actually produce a false positive
			// as there might be errors returned that are the correct
			// behavior for the given parameters
			assert_ok!(SettCurrency::hand_out_settcurrency(
				&SettCurrency::shares(),
				amount,
				SettCurrency::settcurrency_supply()
			));

			let len = len as u64;
			let payout = amount;
			let balance = SettCurrency::get_balance(first);
			assert_ge!(balance, InitialSupply::get() / len + payout / len);
			assert_le!(balance, InitialSupply::get() / len + 1 + payout / len + 1);

			TestResult::passed()
		})
	}

	QuickCheck::new()
		.min_tests_passed(5)
		.tests(50)
		.max_tests(500)
		.quickcheck(property as fn(Vec<u64>, u64) -> TestResult)
}
