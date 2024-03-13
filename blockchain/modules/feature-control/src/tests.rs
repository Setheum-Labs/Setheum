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

//! Tests for the Feature Control module.

use frame_support::{
    assert_err, assert_ok, construct_runtime, derive_impl, sp_runtime,
    sp_runtime::DispatchError::BadOrigin,
};
use frame_system::{mocking::MockBlock, EnsureRoot};
use sp_io::TestExternalities;
use sp_runtime::BuildStorage;

use crate::{Event, Feature};

construct_runtime!(
    pub struct TestRuntime {
        System: frame_system,
        FeatureControl: crate,
    }
);

#[derive_impl(frame_system::config_preludes::TestDefaultConfig as frame_system::DefaultConfig)]
impl frame_system::Config for TestRuntime {
    type Block = MockBlock<TestRuntime>;
}

impl crate::Config for TestRuntime {
    type RuntimeEvent = RuntimeEvent;
    type WeightInfo = ();
    type Supervisor = EnsureRoot<Self::AccountId>;
}

pub fn new_test_ext() -> TestExternalities {
    let t = <frame_system::GenesisConfig<TestRuntime> as BuildStorage>::build_storage(
        &frame_system::GenesisConfig::default(),
    )
    .expect("Storage should be build.");
    let mut t = TestExternalities::from(t);

    // We must set the block number to 1 so that the events are collected (they are not during
    // genesis).
    t.execute_with(|| System::set_block_number(1));
    t
}

const FEATURE: Feature = Feature::OnChainVerifier;

#[test]
fn enabling_and_disabling_feature_works() {
    new_test_ext().execute_with(|| {
        assert!(!FeatureControl::is_feature_enabled(FEATURE));

        assert_ok!(FeatureControl::enable(RuntimeOrigin::root(), FEATURE));
        assert!(FeatureControl::is_feature_enabled(FEATURE));
        // Enabling is idempotent.
        assert_ok!(FeatureControl::enable(RuntimeOrigin::root(), FEATURE));
        assert!(FeatureControl::is_feature_enabled(FEATURE));

        assert_ok!(FeatureControl::disable(RuntimeOrigin::root(), FEATURE));
        assert!(!FeatureControl::is_feature_enabled(FEATURE));
        // Disabling is idempotent.
        assert_ok!(FeatureControl::disable(RuntimeOrigin::root(), FEATURE));
        assert!(!FeatureControl::is_feature_enabled(FEATURE));
    });
}

#[test]
fn enabling_and_disabling_feature_emits_event() {
    new_test_ext().execute_with(|| {
        assert_ok!(FeatureControl::enable(RuntimeOrigin::root(), FEATURE));
        assert!(System::events().iter().any(|record| {
            matches!(
                record.event,
                RuntimeEvent::FeatureControl(Event::FeatureEnabled(f)) if f == FEATURE
            )
        }));

        System::reset_events();

        assert_ok!(FeatureControl::disable(RuntimeOrigin::root(), FEATURE));
        assert!(System::events().iter().any(|record| {
            matches!(
                record.event,
                RuntimeEvent::FeatureControl(Event::FeatureDisabled(f)) if f == FEATURE
            )
        }));
    });
}

#[test]
fn enabling_and_disabling_feature_requires_root() {
    new_test_ext().execute_with(|| {
        assert_err!(
            FeatureControl::enable(RuntimeOrigin::signed(1), FEATURE),
            BadOrigin
        );
        assert_err!(
            FeatureControl::disable(RuntimeOrigin::signed(1), FEATURE),
            BadOrigin
        );
    });
}
