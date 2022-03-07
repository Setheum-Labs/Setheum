//! Unit tests for the vesting module.

#![cfg(test)]

use super::*;
use frame_support::{assert_noop, assert_ok, error::BadOrigin};
use mock::{Event, SETM, *};

#[test]
fn vesting_from_chain_spec_works() {
	ExtBuilder::default().build().execute_with(|| {
		assert_ok!(Tokens::ensure_can_withdraw(
			SETM,
			&CHARLIE,
			10
		));

		assert_eq!(
			Vesting::native_vesting_schedules(&CHARLIE),
			vec![VestingSchedule {
				start: 2u64,
				period: 3u64,
				period_count: 4u32,
				per_period: 5u64,
			}]
		);

		System::set_block_number(13);

		assert_ok!(Vesting::claim(Origin::signed(CHARLIE), SETM));

		assert_ok!(Tokens::ensure_can_withdraw(
			SETM,
			&CHARLIE,
			25
		));

		System::set_block_number(14);

		assert_ok!(Vesting::claim(Origin::signed(CHARLIE), SETM));

		assert_ok!(Tokens::ensure_can_withdraw(
			SETM,
			&CHARLIE,
			30
		));
	});
}

#[test]
fn vested_transfer_works() {
	ExtBuilder::default().build().execute_with(|| {
		System::set_block_number(1);

		let schedule = VestingSchedule {
			start: 0u64,
			period: 10u64,
			period_count: 1u32,
			per_period: 100u64,
		};
		assert_ok!(Vesting::vested_transfer(Origin::signed(ALICE), SETM, BOB, schedule.clone()));
		assert_eq!(Vesting::native_vesting_schedules(&BOB), vec![schedule.clone()]);
	});
}

#[test]
fn add_new_vesting_schedule_merges_with_current_locked_balance_and_until() {
	ExtBuilder::default().build().execute_with(|| {
		let schedule = VestingSchedule {
			start: 0u64,
			period: 10u64,
			period_count: 2u32,
			per_period: 10u64,
		};
		assert_ok!(Vesting::vested_transfer(Origin::signed(ALICE), SETM, BOB, schedule));

		System::set_block_number(12);

		let another_schedule = VestingSchedule {
			start: 10u64,
			period: 13u64,
			period_count: 1u32,
			per_period: 7u64,
		};
		assert_ok!(Vesting::vested_transfer(Origin::signed(ALICE), SETM, BOB, another_schedule));
	});
}

#[test]
fn cannot_use_fund_if_not_claimed() {
	ExtBuilder::default().build().execute_with(|| {
		let schedule = VestingSchedule {
			start: 10u64,
			period: 10u64,
			period_count: 1u32,
			per_period: 50u64,
		};
		assert_ok!(Vesting::vested_transfer(Origin::signed(ALICE), SETM, BOB, schedule));
		assert!(Tokens::ensure_can_withdraw(SETM, &BOB, 1).is_err());
	});
}

#[test]
fn vested_transfer_fails_if_zero_period_or_count() {
	ExtBuilder::default().build().execute_with(|| {
		let schedule = VestingSchedule {
			start: 1u64,
			period: 0u64,
			period_count: 1u32,
			per_period: 100u64,
		};
		assert_noop!(
			Vesting::vested_transfer(Origin::signed(ALICE), SETM, BOB, schedule),
			Error::<Runtime>::ZeroVestingPeriod
		);

		let schedule = VestingSchedule {
			start: 1u64,
			period: 1u64,
			period_count: 0u32,
			per_period: 100u64,
		};
		assert_noop!(
			Vesting::vested_transfer(Origin::signed(ALICE), SETM, BOB, schedule),
			Error::<Runtime>::ZeroVestingPeriodCount
		);
	});
}

#[test]
fn vested_transfer_fails_if_overflow() {
	ExtBuilder::default().build().execute_with(|| {
		let schedule = VestingSchedule {
			start: 1u64,
			period: 1u64,
			period_count: 2u32,
			per_period: u64::MAX,
		};
		assert_noop!(
			Vesting::vested_transfer(Origin::signed(ALICE), SETM, BOB, schedule),
			ArithmeticError::Overflow,
		);

		let another_schedule = VestingSchedule {
			start: u64::MAX,
			period: 1u64,
			period_count: 2u32,
			per_period: 1u64,
		};
		assert_noop!(
			Vesting::vested_transfer(Origin::signed(ALICE), SETM, BOB, another_schedule),
			ArithmeticError::Overflow,
		);
	});
}

#[test]
fn vested_transfer_fails_if_bad_origin() {
	ExtBuilder::default().build().execute_with(|| {
		let schedule = VestingSchedule {
			start: 0u64,
			period: 10u64,
			period_count: 1u32,
			per_period: 100u64,
		};
		assert_noop!(
			Vesting::vested_transfer(Origin::signed(CHARLIE), SETM, BOB, schedule),
			BadOrigin
		);
	});
}

#[test]
fn claim_works() {
	ExtBuilder::default().build().execute_with(|| {
		let schedule = VestingSchedule {
			start: 0u64,
			period: 10u64,
			period_count: 2u32,
			per_period: 10u64,
		};
		assert_ok!(Vesting::vested_transfer(Origin::signed(ALICE), SETM, BOB, schedule));

		System::set_block_number(11);
		// remain locked if not claimed
		assert!(Tokens::transfer(Origin::signed(BOB), ALICE, SETM, 10).is_err());
		// unlocked after claiming
		assert_ok!(Vesting::claim(Origin::signed(BOB), SETM));
		assert!(NativeVestingSchedules::<Runtime>::contains_key(BOB));
		assert_ok!(Tokens::transfer(Origin::signed(BOB), ALICE, SETM, 10));
		// more are still locked
		assert!(Tokens::transfer(Origin::signed(BOB), ALICE, SETM, 1).is_err());

		System::set_block_number(21);
		// claim more
		assert_ok!(Vesting::claim(Origin::signed(BOB), SETM));
		assert!(!NativeVestingSchedules::<Runtime>::contains_key(BOB));
		assert_ok!(Tokens::transfer(Origin::signed(BOB), ALICE, SETM, 10));
		// all used up
		assert_eq!(Tokens::free_balance(SETM, &BOB), 0);
		assert_eq!(PalletBalances::free_balance(BOB), 0);

		// no locks anymore
		assert_eq!(PalletBalances::locks(&BOB), vec![]);
	});
}

#[test]
fn claim_for_works() {
	ExtBuilder::default().build().execute_with(|| {
		let schedule = VestingSchedule {
			start: 0u64,
			period: 10u64,
			period_count: 2u32,
			per_period: 10u64,
		};
		assert_ok!(Vesting::vested_transfer(Origin::signed(ALICE), SETM, BOB, schedule));

		assert_ok!(Vesting::claim_for(Origin::signed(ALICE), SETM, BOB));

		assert!(NativeVestingSchedules::<Runtime>::contains_key(&BOB));

		System::set_block_number(21);

		assert_ok!(Vesting::claim_for(Origin::signed(ALICE), SETM, BOB));

		// no locks anymore
		assert_eq!(PalletBalances::locks(&BOB), vec![]);
		assert!(!NativeVestingSchedules::<Runtime>::contains_key(&BOB));
	});
}

#[test]
fn update_vesting_schedules_works() {
	ExtBuilder::default().build().execute_with(|| {
		let schedule = VestingSchedule {
			start: 0u64,
			period: 10u64,
			period_count: 2u32,
			per_period: 10u64,
		};
		assert_ok!(Vesting::vested_transfer(Origin::signed(ALICE), SETM, BOB, schedule));

		let updated_schedule = VestingSchedule {
			start: 0u64,
			period: 20u64,
			period_count: 2u32,
			per_period: 10u64,
		};
		assert_ok!(Vesting::update_vesting_schedules(
			Origin::signed(ALICE),
			SETM,
			BOB,
			vec![updated_schedule]
		));

		System::set_block_number(11);
		assert_ok!(Vesting::claim(Origin::signed(BOB), SETM));
		assert!(Tokens::transfer(Origin::signed(BOB), ALICE, SETM, 1).is_err());

		System::set_block_number(21);
		assert_ok!(Vesting::claim(Origin::signed(BOB), SETM));
		assert_ok!(Tokens::transfer(Origin::signed(BOB), ALICE, SETM, 10));

		// empty vesting schedules cleanup the storage and unlock the fund
		assert!(NativeVestingSchedules::<Runtime>::contains_key(BOB));

		assert_ok!(Vesting::update_vesting_schedules(Origin::signed(ALICE), SETM, BOB, vec![]));
		assert!(!NativeVestingSchedules::<Runtime>::contains_key(BOB));
		assert_eq!(PalletBalances::locks(&BOB), vec![]);
	});
}

#[test]
fn update_vesting_schedules_fails_if_unexpected_existing_locks() {
	ExtBuilder::default().build().execute_with(|| {
		assert_ok!(Tokens::transfer(Origin::signed(ALICE), BOB, SETM, 1));
		Tokens::set_lock(*b"prelocks", SETM, &BOB, 0u64);
	});
}

#[test]
fn vested_transfer_check_for_min() {
	ExtBuilder::default().build().execute_with(|| {
		let schedule = VestingSchedule {
			start: 1u64,
			period: 1u64,
			period_count: 1u32,
			per_period: 3u64,
		};
		assert_noop!(
			Vesting::vested_transfer(Origin::signed(BOB), SETM, ALICE, schedule),
			Error::<Runtime>::AmountLow
		);
	});
}

#[test]
fn multiple_vesting_schedule_claim_works() {
	ExtBuilder::default().build().execute_with(|| {
		let schedule = VestingSchedule {
			start: 0u64,
			period: 10u64,
			period_count: 2u32,
			per_period: 10u64,
		};
		assert_ok!(Vesting::vested_transfer(Origin::signed(ALICE), SETM, BOB, schedule.clone()));

		let schedule2 = VestingSchedule {
			start: 0u64,
			period: 10u64,
			period_count: 3u32,
			per_period: 10u64,
		};
		assert_ok!(Vesting::vested_transfer(Origin::signed(ALICE), SETM, BOB, schedule2.clone()));

		assert_eq!(Vesting::native_vesting_schedules(&BOB), vec![schedule, schedule2.clone()]);

		System::set_block_number(21);

		assert_ok!(Vesting::claim(Origin::signed(BOB), SETM));

		assert_eq!(Vesting::native_vesting_schedules(&BOB), vec![schedule2]);

		System::set_block_number(31);

		assert_ok!(Vesting::claim(Origin::signed(BOB), SETM));

		assert!(!NativeVestingSchedules::<Runtime>::contains_key(&BOB));

		assert_eq!(PalletBalances::locks(&BOB), vec![]);
	});
}

#[test]
fn exceeding_maximum_schedules_should_fail() {
	ExtBuilder::default().build().execute_with(|| {
		let schedule = VestingSchedule {
			start: 0u64,
			period: 10u64,
			period_count: 2u32,
			per_period: 10u64,
		};
		assert_ok!(Vesting::vested_transfer(Origin::signed(ALICE), SETM, BOB, schedule.clone()));
		assert_ok!(Vesting::vested_transfer(Origin::signed(ALICE), SETM, BOB, schedule.clone()));
		assert_noop!(
			Vesting::vested_transfer(Origin::signed(ALICE), SETM, BOB, schedule.clone()),
			Error::<Runtime>::MaxVestingSchedulesExceeded
		);

		let schedules = vec![schedule.clone(), schedule.clone(), schedule];

		assert_noop!(
			Vesting::update_vesting_schedules(Origin::signed(ALICE), SETM, BOB, schedules),
			Error::<Runtime>::MaxVestingSchedulesExceeded
		);
	});
}