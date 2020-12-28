use crate::{mock::*, Error};
use frame_support::{assert_noop, assert_ok};

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
		SettCurrency::add_bid(Bid::new(1, Perbill::from_percent(80), dinar_amount));
		SettCurrency::add_bid(Bid::new(2, Perbill::from_percent(75), 2 * BaseUnit::get()));

		let prev_supply = SettCurrency::settcurrency_supply();
		let amount = 2 * BaseUnit::get();
		assert_ok!(SettCurrency::contract_supply(prev_supply, amount));

		let bids = SettCurrency::dinar_bids();
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
			SettCurrency::settcurrency_supply(),
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

