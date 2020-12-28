use crate::{mock::*, Error};
use frame_support::{assert_noop, assert_ok};

#[test]
fn it_works_for_default_value() {
    new_test_ext().execute_with(|| {
        // Dispatch a signed extrinsic.
        assert_ok!(Stp258::do_something(Origin::signed(1), 42));
        // Read pallet storage and assert an expected result.
        assert_eq!(Stp258::something(), Some(42));
    });
}

#[test]
fn correct_error_for_none_value() {
    new_test_ext().execute_with(|| {
        // Ensure the expected error is thrown when no value is present.
        assert_noop!(
            Stp258::cause_error(Origin::signed(1)),
            Error::<Test>::NoneValue
        );
    });
}

// ------------------------------------------------------------
// init tests
#[test]
fn init_test() {
	new_test_ext().execute_with(|| {
		let shares = SettCurrency::shares();
		assert_eq!(
			shares,
			vec![
				(1, 1),
				(2, 1),
				(3, 1),
				(4, 1),
				(5, 1),
				(6, 1),
				(7, 1),
				(8, 1),
				(9, 1),
				(10, 1)
			]
		);
		let share_supply: u64 = shares.iter().map(|(_a, s)| s).sum();
		assert_eq!(share_supply, 10);
	});
}

// ------------------------------------------------------------
// balances
#[test]
fn transfer_test() {
	new_test_ext().execute_with(|| {
		let first_acc = 1;
		let second_acc = 2;
		let amount = TEST_BASE_UNIT;
		let from_balance_before = SettCurrency::get_balance(first_acc);
		let to_balance_before = SettCurrency::get_balance(second_acc);
		assert_ok!(SettCurrency::transfer_from_to(&first_acc, &second_acc, amount));
		assert_eq!(SettCurrency::get_balance(first_acc), from_balance_before - amount);
		assert_eq!(SettCurrency::get_balance(second_acc), to_balance_before + amount);
	});
}

// ------------------------------------------------------------
// currency trait
#[test]
fn slash_test() {
	new_test_ext().execute_with(|| {
		let acc = 1;
		let amount = TEST_BASE_UNIT;
		let balance_before = SettCurrency::get_balance(acc);
		assert_eq!(SettCurrency::slash(&acc, amount), 0);
		assert_eq!(SettCurrency::get_balance(acc), balance_before - amount);
	});
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


