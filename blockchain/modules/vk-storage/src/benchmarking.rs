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

//! Benchmarking for the VK Storage module.

#![allow(clippy::let_unit_value)]

use frame_benchmarking::{account, benchmarks};
use frame_support::traits::Get;
use frame_system::RawOrigin;
use sp_std::vec;

use crate::{Call, Config, Pallet};

const SEED: u32 = 41;

fn caller<T: Config>() -> RawOrigin<<T as frame_system::Config>::AccountId> {
    RawOrigin::Signed(account("caller", 0, SEED))
}

benchmarks! {
    store_key {
        let l in 1 .. T::MaximumKeyLength::get();
        let key = vec![l as u8; l as usize];
    } : _(caller::<T>(), key)

    impl_benchmark_test_suite!(Pallet, crate::tests::new_test_ext(), crate::tests::TestRuntime);
}
