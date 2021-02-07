use crate::{mock::*, Error};
use frame_support::{assert_noop, assert_ok};

#[test]
fn it_works_for_default_value() {
    new_test_ext().execute_with(|| {
        // Dispatch a signed extrinsic.
        assert_ok!(SerpMarket::do_something(Origin::signed(1), 42));
        // Read pallet storage and assert an expected result.
        assert_eq!(SerpMarket::something(), Some(42));
    });
}

#[test]
fn correct_error_for_none_value() {
    new_test_ext().execute_with(|| {
        // Ensure the expected error is thrown when no value is present.
        assert_noop!(
            SerpMarket::cause_error(Origin::signed(1)),
            Error::<Test>::NoneValue
        );
    });
}

// ------------------------------------------------------------
// bids
#[test]
fn bids_are_sorted_highest_to_lowest() {
	new_test_ext().execute_with(|| {
		let bid_amount = 5 * BaseUnit::get();
		SettCurrency::add_bid(Bid::new(1, Perbill::from_percent(25), bid_amount));
		SettCurrency::add_bid(Bid::new(1, Perbill::from_percent(33), bid_amount));
		SettCurrency::add_bid(Bid::new(1, Perbill::from_percent(50), bid_amount));

		let bids = SettCurrency::dinar_bids();
		let prices: Vec<_> = bids.into_iter().map(|Bid { price, .. }| price).collect();
		// largest bid is stored last so we can pop
		assert_eq!(
			prices,
			vec![
				Perbill::from_percent(25),
				Perbill::from_percent(33),
				Perbill::from_percent(50),
			]
		);
	});
}

#[test]
fn amount_of_bids_is_limited() {
	new_test_ext().execute_with(|| {
		let bid_amount = 5 * BaseUnit::get();
		for _i in 0..(2 * MaximumBids::get()) {
			SettCurrency::add_bid(Bid::new(1, Perbill::from_percent(25), bid_amount));
		}

		assert_eq!(SettCurrency::dinar_bids().len() as u64, MaximumBids::get());
	});
}

#[test]
fn truncated_bids_are_refunded() {
	new_test_ext_with(vec![1]).execute_with(|| {
		let price = Perbill::from_percent(25);
		let quantity = BaseUnit::get();
		for _i in 0..(MaximumBids::get() + 1) {
			assert_ok!(SettCurrency::bid_for_dinar(Origin::signed(1), price, quantity));
		}

		assert_eq!(SettCurrency::dinar_bids().len() as u64, MaximumBids::get());
		let expected = InitialSupply::get() - price * quantity * (MaximumBids::get() as u64);
		assert_eq!(SettCurrency::get_balance(1), expected);
	});
}

#[test]
fn cancel_all_bids_test() {
	new_test_ext().execute_with(|| {
		let bid_amount = 5 * BaseUnit::get();
		SettCurrency::add_bid(Bid::new(1, Perbill::from_percent(25), bid_amount));
		SettCurrency::add_bid(Bid::new(2, Perbill::from_percent(33), bid_amount));
		SettCurrency::add_bid(Bid::new(1, Perbill::from_percent(50), bid_amount));
		SettCurrency::add_bid(Bid::new(3, Perbill::from_percent(50), bid_amount));
		assert_eq!(SettCurrency::dinar_bids().len(), 4);

		assert_ok!(SettCurrency::cancel_all_bids(Origin::signed(1)));

		let bids = SettCurrency::dinar_bids();
		assert_eq!(bids.len(), 2);
		for bid in bids {
			assert!(bid.account != 1);
		}
	});
}

#[test]
fn cancel_selected_bids_test() {
	new_test_ext().execute_with(|| {
		let bid_amount = 5 * BaseUnit::get();
		SettCurrency::add_bid(Bid::new(1, Perbill::from_percent(25), bid_amount));
		SettCurrency::add_bid(Bid::new(2, Perbill::from_percent(33), bid_amount));
		SettCurrency::add_bid(Bid::new(1, Perbill::from_percent(45), bid_amount));
		SettCurrency::add_bid(Bid::new(1, Perbill::from_percent(50), bid_amount));
		SettCurrency::add_bid(Bid::new(3, Perbill::from_percent(55), bid_amount));
		assert_eq!(SettCurrency::dinar_bids().len(), 5);

		assert_ok!(SettCurrency::cancel_bids_at_or_below(
			Origin::signed(1),
			Perbill::from_percent(45)
		));

		let bids = SettCurrency::dinar_bids();
		assert_eq!(bids.len(), 3);
		let bids: Vec<(_, _)> = bids
			.into_iter()
			.map(|Bid { account, price, .. }| (account, price))
			.collect();
		// highest bid is last so we can pop
		assert_eq!(
			bids,
			vec![
				(2, Perbill::from_percent(33)),
				(1, Perbill::from_percent(50)),
				(3, Perbill::from_percent(55)),
			]
		);
	});
}
