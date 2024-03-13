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

//! Tests for the VK Storage module.

use frame_support::{
    construct_runtime,
    pallet_prelude::ConstU32,
    parameter_types, sp_runtime,
    sp_runtime::{testing::H256, traits::IdentityLookup},
    traits::Everything,
};
use frame_system::mocking::MockBlock;
use sp_io::TestExternalities;
use sp_runtime::{traits::BlakeTwo256, BuildStorage};

use crate as module_vk_storage;
use crate::StorageCharge;

construct_runtime!(
    pub struct TestRuntime {
        System: frame_system,
        VkStorage: module_vk_storage,
    }
);

impl frame_system::Config for TestRuntime {
    type RuntimeEvent = RuntimeEvent;
    type BaseCallFilter = Everything;
    type BlockWeights = ();
    type BlockLength = ();
    type RuntimeOrigin = RuntimeOrigin;
    type RuntimeCall = RuntimeCall;
    type Nonce = u64;
    type Hash = H256;
    type Hashing = BlakeTwo256;
    type AccountId = u128;
    type Lookup = IdentityLookup<Self::AccountId>;
    type Block = MockBlock<TestRuntime>;
    type BlockHashCount = ();
    type DbWeight = ();
    type Version = ();
    type PalletInfo = PalletInfo;
    type AccountData = u64;
    type OnNewAccount = ();
    type OnKilledAccount = ();
    type SystemWeightInfo = ();
    type SS58Prefix = ();
    type OnSetCode = ();
    type MaxConsumers = ConstU32<16>;
}

parameter_types! {
    pub const VkStorageCharge: StorageCharge = StorageCharge::linear(1, 10);
}

impl module_vk_storage::Config for TestRuntime {
    type RuntimeEvent = RuntimeEvent;
    type WeightInfo = ();
    type MaximumKeyLength = ConstU32<10_000>;
    type StorageCharge = VkStorageCharge;
}

pub fn new_test_ext() -> TestExternalities {
    let t = <frame_system::GenesisConfig<TestRuntime> as BuildStorage>::build_storage(
        &frame_system::GenesisConfig::default(),
    )
    .expect("Storage should be build.");
    TestExternalities::new(t)
}
