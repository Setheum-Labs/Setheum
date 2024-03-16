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

//! Benchmarking for the Feature Control module.

use frame_benchmarking::v2::*;

#[benchmarks]
mod benchmarks {
    use frame_system::RawOrigin;
    use sp_std::vec;

    use crate::{ActiveFeatures, Call, Config, Feature, Pallet};

    #[benchmark]
    fn enable() {
        #[extrinsic_call]
        _(RawOrigin::Root, Feature::OnChainVerifier);

        assert!(ActiveFeatures::<T>::contains_key(Feature::OnChainVerifier));
    }

    #[benchmark]
    fn disable() {
        Pallet::<T>::enable(RawOrigin::Root.into(), Feature::OnChainVerifier).unwrap();

        #[extrinsic_call]
        _(RawOrigin::Root, Feature::OnChainVerifier);

        assert!(!Pallet::<T>::is_feature_enabled(Feature::OnChainVerifier));
    }

    frame_benchmarking::impl_benchmark_test_suite!(
        Pallet,
        crate::tests::new_test_ext(),
        crate::tests::TestRuntime
    );
}
