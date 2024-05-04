//! Unit tests for the vesting module.

#![cfg(test)]

use super::*;
use frame_support::{assert_noop, assert_ok, error::BadOrigin};
use mock::*;
use sp_runtime::traits::Dispatchable;
use sp_runtime::TokenError;

#[test]
fn vesting_from_chain_spec_works() {
	ExtBuilder::default().build().execute_with(|| {
		assert_ok!(Tokens::ensure_can_withdraw(
			SEE,
			&CHARLIE,
			10
		));
		assert!(Tokens::ensure_can_withdraw(SEE, &CHARLIE, 11).is_err());

		assert_eq!(
			Vesting::native_vesting_schedules(&CHARLIE),
			vec![
				VestingSchedule {
					start: 2u64,
					period: 3u64,
					period_count: 1u32,
					per_period: 5u64,
				},
				VestingSchedule {
					start: 2u64 + 3u64,
					period: 3u64,
					period_count: 3u32,
					per_period: 5u64,
				}
			]
		);

		System::set_block_number(13);

		assert_ok!(Vesting::claim(RuntimeOrigin::signed(CHARLIE), SEE));

		assert_ok!(Tokens::ensure_can_withdraw(
			SEE,
			&CHARLIE,
			25
		));
		assert!(Tokens::ensure_can_withdraw(SEE, &CHARLIE, 26).is_err());

		System::set_block_number(14);

		assert_ok!(Vesting::claim(RuntimeOrigin::signed(CHARLIE), SEE));

		assert_ok!(Tokens::ensure_can_withdraw(
			SEE,
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
		assert_ok!(Vesting::vested_transfer(
			RuntimeOrigin::signed(ALICE),
			SEE,
			BOB,
			schedule.clone()
		));
		assert_eq!(Vesting::native_vesting_schedules(&BOB), vec![schedule.clone()]);
		System::assert_last_event(RuntimeEvent::Vesting(crate::Event::VestingScheduleAdded {
			from: ALICE,
			to: BOB,
			vesting_schedule: schedule,
		}));
	});
}

#[test]
fn self_vesting() {
	ExtBuilder::default().build().execute_with(|| {
		System::set_block_number(1);

		let schedule = VestingSchedule {
			start: 0u64,
			period: 10u64,
			period_count: 1u32,
			per_period: ALICE_BALANCE,
		};

		let bad_schedule = VestingSchedule {
			start: 0u64,
			period: 10u64,
			period_count: 1u32,
			per_period: 10 * ALICE_BALANCE,
		};

		assert_noop!(
			Vesting::vested_transfer(RuntimeOrigin::signed(ALICE), SEE, ALICE, bad_schedule),
			crate::Error::<Runtime>::InsufficientBalanceToLock
		);

		assert_ok!(Vesting::vested_transfer(
			RuntimeOrigin::signed(ALICE),
			SEE,
			ALICE,
			schedule.clone()
		));

		assert_eq!(Vesting::native_vesting_schedules(&ALICE), vec![schedule.clone()]);
		System::assert_last_event(RuntimeEvent::Vesting(crate::Event::VestingScheduleAdded {
			from: ALICE,
			to: ALICE,
			vesting_schedule: schedule,
		}));
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
		assert_ok!(Vesting::vested_transfer(RuntimeOrigin::signed(ALICE), SEE, BOB, schedule));

		System::set_block_number(12);

		let another_schedule = VestingSchedule {
			start: 10u64,
			period: 13u64,
			period_count: 1u32,
			per_period: 7u64,
		};
		assert_ok!(Vesting::vested_transfer(
			RuntimeOrigin::signed(ALICE),
			SEE,
			BOB,
			another_schedule
		));

		assert_eq!(
			Tokens::locks(&BOB, SEE).get(0),
			Some(&BalanceLock {
				id: VESTING_LOCK_ID,
				amount: 17u64,
			})
		);
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
		assert_ok!(Vesting::vested_transfer(RuntimeOrigin::signed(ALICE), SEE, BOB, schedule));
		assert!(Tokens::ensure_can_withdraw(SEE, &BOB, 1).is_err());
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
			Vesting::vested_transfer(RuntimeOrigin::signed(ALICE), SEE, BOB, schedule),
			Error::<Runtime>::ZeroVestingPeriod
		);

		let schedule = VestingSchedule {
			start: 1u64,
			period: 1u64,
			period_count: 0u32,
			per_period: 100u64,
		};
		assert_noop!(
			Vesting::vested_transfer(RuntimeOrigin::signed(ALICE), SEE, BOB, schedule),
			Error::<Runtime>::ZeroVestingPeriodCount
		);
	});
}

#[test]
fn vested_transfer_fails_if_transfer_err() {
	ExtBuilder::default().build().execute_with(|| {
		let schedule = VestingSchedule {
			start: 1u64,
			period: 1u64,
			period_count: 1u32,
			per_period: 100u64,
		};
		assert_noop!(
			Vesting::vested_transfer(RuntimeOrigin::signed(BOB), SEE, ALICE, schedule),
			TokenError::FundsUnavailable,
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
			Vesting::vested_transfer(RuntimeOrigin::signed(ALICE), SEE, BOB, schedule),
			ArithmeticError::Overflow,
		);

		let another_schedule = VestingSchedule {
			start: u64::MAX,
			period: 1u64,
			period_count: 2u32,
			per_period: 1u64,
		};
		assert_noop!(
			Vesting::vested_transfer(RuntimeOrigin::signed(ALICE), SEE, BOB, another_schedule),
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
			Vesting::vested_transfer(RuntimeOrigin::signed(CHARLIE), SEE, BOB, schedule),
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
		assert_ok!(Vesting::vested_transfer(RuntimeOrigin::signed(ALICE), SEE, BOB, schedule));

		System::set_block_number(11);
		// remain locked if not claimed
		assert!(Tokens::transfer(&BOB, &ALICE, SEE, 10).is_err());
		// unlocked after claiming
		assert_ok!(Vesting::claim(RuntimeOrigin::signed(BOB), SEE));
		assert!(NativeVestingSchedules::<Runtime>::contains_key(BOB));
		assert_ok!(Tokens::transfer(&BOB, &ALICE, SEE, 10));
		// more are still locked
		assert!(Tokens::transfer(&BOB, &ALICE, SEE, 1).is_err());

		System::set_block_number(21);
		// claim more
		assert_ok!(Vesting::claim(RuntimeOrigin::signed(BOB), SEE));
		assert!(!NativeVestingSchedules::<Runtime>::contains_key(BOB));
		assert_ok!(Tokens::transfer(&BOB, &ALICE, SEE, 10));
		// all used up
		assert_eq!(Tokens::free_balance(SEE, &BOB), 0);

		// no locks anymore
		assert_eq!(Tokens::locks(&BOB, SEE), vec![]);
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
		assert_ok!(Vesting::vested_transfer(RuntimeOrigin::signed(ALICE), SEE, BOB, schedule));

		assert_ok!(Vesting::claim_for(RuntimeOrigin::signed(ALICE), SEE, BOB));

		assert_eq!(
			Tokens::locks(&BOB, SEE).get(0),
			Some(&BalanceLock {
				id: VESTING_LOCK_ID,
				amount: 20u64,
			})
		);
		assert!(NativeVestingSchedules::<Runtime>::contains_key(&BOB));

		System::set_block_number(21);

		assert_ok!(Vesting::claim_for(RuntimeOrigin::signed(ALICE), SEE, BOB));

		// no locks anymore
		assert_eq!(Tokens::locks(&BOB, SEE), vec![]);
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
		assert_ok!(Vesting::vested_transfer(RuntimeOrigin::signed(ALICE), SEE, BOB, schedule));

		let updated_schedule = VestingSchedule {
			start: 0u64,
			period: 20u64,
			period_count: 2u32,
			per_period: 10u64,
		};
		assert_ok!(Vesting::update_vesting_schedules(
			RuntimeOrigin::root(),
			SEE,
			BOB,
			vec![updated_schedule]
		));

		System::set_block_number(11);
		assert_ok!(Vesting::claim(RuntimeOrigin::signed(BOB), SEE));
		assert!(Tokens::transfer(&BOB, &ALICE, SEE, 1).is_err());

		System::set_block_number(21);
		assert_ok!(Vesting::claim(RuntimeOrigin::signed(BOB), SEE));
		assert_ok!(Tokens::transfer(&BOB, &ALICE, SEE, 10));

		// empty vesting schedules cleanup the storage and unlock the fund
		assert!(NativeVestingSchedules::<Runtime>::contains_key(BOB));
		assert_eq!(
			Tokens::locks(&BOB, SEE).get(0),
			Some(&BalanceLock {
				id: VESTING_LOCK_ID,
				amount: 10u64,
			})
		);
		assert_ok!(Vesting::update_vesting_schedules(RuntimeOrigin::root(), SEE, BOB, vec![]));
		assert!(!NativeVestingSchedules::<Runtime>::contains_key(BOB));
		assert_eq!(Tokens::locks(&BOB, SEE), vec![]);
	});
}

#[test]
fn update_vesting_schedules_fails_if_unexpected_existing_locks() {
	ExtBuilder::default().build().execute_with(|| {
		assert_ok!(Tokens::transfer(&ALICE, &BOB, SEE, 1));
		Tokens::set_lock(*b"prelocks", SEE, &BOB, 0u64);
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
			Vesting::vested_transfer(RuntimeOrigin::signed(BOB), SEE, ALICE, schedule),
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
		assert_ok!(Vesting::vested_transfer(RuntimeOrigin::signed(ALICE), SEE, BOB, schedule.clone()));

		let schedule2 = VestingSchedule {
			start: 0u64,
			period: 10u64,
			period_count: 3u32,
			per_period: 10u64,
		};
		assert_ok!(Vesting::vested_transfer(RuntimeOrigin::signed(ALICE), SEE, BOB, schedule2.clone()));

		assert_eq!(Vesting::native_vesting_schedules(&BOB), vec![schedule, schedule2.clone()]);

		System::set_block_number(21);

		assert_ok!(Vesting::claim(RuntimeOrigin::signed(BOB, SEE)));

		assert_eq!(Vesting::native_vesting_schedules(&BOB), vec![schedule2]);

		System::set_block_number(31);

		assert_ok!(Vesting::claim(RuntimeOrigin::signed(BOB, SEE)));

		assert!(!NativeVestingSchedules::<Runtime>::contains_key(&BOB));

		assert_eq!(Tokens::locks(&BOB, SEE), vec![]);
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
		assert_ok!(Vesting::vested_transfer(RuntimeOrigin::signed(ALICE), SEE, BOB, schedule.clone()));
		assert_ok!(Vesting::vested_transfer(RuntimeOrigin::signed(ALICE), SEE, BOB, schedule.clone()));

		let create = RuntimeCall::Vesting(crate::Call::<Runtime>::vested_transfer {
			currency_id: SEE,
			dest: BOB,
			schedule: schedule.clone(),
		});
		assert_noop!(
			create.dispatch(RuntimeOrigin::signed(ALICE)),
			Error::<Runtime>::MaxNativeVestingSchedulesExceeded
		);

		let schedules = vec![schedule.clone(), schedule.clone(), schedule];

		assert_noop!(
			Vesting::update_vesting_schedules(RuntimeOrigin::root(), SEE, BOB, schedules),
			Error::<Runtime>::MaxNativeVestingSchedulesExceeded
		);
	});
}

#[test]
fn cliff_vesting_works() {
	const VESTING_AMOUNT: Balance = 12;
	const VESTING_PERIOD: Balance = 20;

	ExtBuilder::default().build().execute_with(|| {
		let cliff_schedule = VestingSchedule {
			start: VESTING_PERIOD - 1,
			period: 1,
			period_count: 1,
			per_period: VESTING_AMOUNT,
		};

		let balance_lock = BalanceLock {
			id: VESTING_LOCK_ID,
			amount: VESTING_AMOUNT,
		};

		assert_eq!(Tokens::free_balance(SEE, BOB), 0);
		assert_ok!(Vesting::vested_transfer(
			RuntimeOrigin::signed(ALICE),
			SEE,
			BOB,
			cliff_schedule
		));
		assert_eq!(Tokens::free_balance(SEE, BOB), VESTING_AMOUNT);
		assert_eq!(Tokens::locks(&BOB, SEE), vec![balance_lock.clone()]);

		for i in 1..VESTING_PERIOD {
			System::set_block_number(i);
			assert_ok!(Vesting::claim(RuntimeOrigin::signed(BOB), SEE));
			assert_eq!(Tokens::free_balance(SEE, BOB), VESTING_AMOUNT);
			assert_eq!(Tokens::locks(&BOB, SEE), vec![balance_lock.clone()]);
			assert_noop!(
				Tokens::transfer(&BOB, &CHARLIE, SEE, VESTING_AMOUNT),
				TokenError::Frozen,
			);
		}

		System::set_block_number(VESTING_PERIOD);
		assert_ok!(Vesting::claim(RuntimeOrigin::signed(BOB), SEE));
		assert!(Tokens::locks(&BOB, SEE).is_empty());
		assert_ok!(Tokens::transfer(
			&BOB,
			&CHARLIE,
			SEE,
			VESTING_AMOUNT
		));
	});
}
