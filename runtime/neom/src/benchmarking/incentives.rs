// This file is part of Setheum.

// Copyright (C) 2020-2021 Setheum Labs.
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

use crate::{
	dollar, AccountId, Currencies, CurrencyId, Incentives, 
	Runtime, System, TokenSymbol, JGBP, JUSD,
};

use super::utils::set_balance;
use frame_benchmarking::{account, whitelisted_caller};
use frame_support::traits::OnInitialize;
use frame_system::RawOrigin;
use orml_benchmarking::runtime_benchmarks;
use orml_traits::MultiCurrency;
use primitives::DexShare;
use sp_std::prelude::*;

const SEED: u32 = 0;
const JCHF_JUSD_LP: CurrencyId =
	CurrencyId::DexShare(DexShare::Token(TokenSymbol::JCHF), DexShare::Token(TokenSymbol::JUSD));

runtime_benchmarks! {
	{ Runtime, module_incentives }

	on_initialize {
		Incentives::on_initialize(1);
		System::set_block_number(block_number);
	}: {
		Incentives::on_initialize(System::block_number());
	}

	deposit_dex_share {
		let caller: AccountId = whitelisted_caller();
		set_balance(JCHF_JUSD_LP, &caller, 10_000 * dollar(JUSD));
	}: _(RawOrigin::Signed(caller), JCHF_JUSD_LP, 10_000 * dollar(JUSD))

	withdraw_dex_share {
		let caller: AccountId = whitelisted_caller();
		set_balance(JCHF_JUSD_LP, &caller, 10_000 * dollar(JUSD));
		Incentives::deposit_dex_share(
			RawOrigin::Signed(caller.clone()).into(),
			JCHF_JUSD_LP,
			10_000 * dollar(JUSD)
		)?;
	}: _(RawOrigin::Signed(caller), JCHF_JUSD_LP, 8000 * dollar(JUSD))
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::benchmarking::utils::tests::new_test_ext;
	use orml_benchmarking::impl_benchmark_test_suite;

	impl_benchmark_test_suite!(new_test_ext(),);
}
