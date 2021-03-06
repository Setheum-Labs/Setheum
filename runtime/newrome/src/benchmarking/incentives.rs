// This file is part of Setheum.

// Copyright (C) 2019-2021 Setheum Labs.
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
	dollar, AccountId, ReserveCurrencyIds, CurrencyId, GetStableCurrencyId, Incentives, Rate, Rewards, Runtime,
	TokenSymbol, DNAR, USDJ, SETT,
};

use super::utils::set_balance;
use frame_benchmarking::account;
use frame_system::RawOrigin;
use setheum_incentives::PoolId;
use orml_benchmarking::runtime_benchmarks;
use sp_std::prelude::*;

const SEED: u32 = 0;
const DNAR_USDJ_LP: CurrencyId = CurrencyId::DexShare(TokenSymbol::CHFJ, TokenSymbol::USDJ);

runtime_benchmarks! {
	{ Runtime, setheum_incentives }

	_ {}

	deposit_dex_share {
		let caller: AccountId = account("caller", 0, SEED);
		set_balance(DNAR_USDJ_LP, &caller, 10_000 * dollar(USDJ));
	}: _(RawOrigin::Signed(caller), DNAR_USDJ_LP, 10_000 * dollar(USDJ))

	withdraw_dex_share {
		let caller: AccountId = account("caller", 0, SEED);
		set_balance(DNAR_USDJ_LP, &caller, 10_000 * dollar(USDJ));
		Incentives::deposit_dex_share(
			RawOrigin::Signed(caller.clone()).into(),
			DNAR_USDJ_LP,
			10_000 * dollar(USDJ)
		)?;
	}: _(RawOrigin::Signed(caller), DNAR_USDJ_LP, 8000 * dollar(USDJ))

	claim_rewards {
		let caller: AccountId = account("caller", 0, SEED);
		let pool_id = PoolId::SettmintManager(SETT);

		Rewards::add_share(&caller, pool_id, 100);
		orml_rewards::Pools::<Runtime>::mutate(pool_id, |pool_info| {
			pool_info.total_rewards += 5000;
		});
	}: _(RawOrigin::Signed(caller), pool_id)

	// TODO: Update - add all other dex rewards ...
	update_dex_incentive_rewards {
		let c in 0 .. ReserveCurrencyIds::get().len().saturating_sub(1) as u32;
		let currency_ids = ReserveCurrencyIds::get();
		let caller: AccountId = account("caller", 0, SEED);
		let mut values = vec![];
		let base_currency_id = GetStableCurrencyId::get();

		for i in 0 .. c {
			let currency_id = currency_ids[i as usize];
			let lp_share_currency_id = match (currency_id, base_currency_id) {
				(CurrencyId::Token(other_currency_symbol), CurrencyId::Token(base_currency_symbol)) => {
					CurrencyId::DexShare(other_currency_symbol, base_currency_symbol)
				}
				_ => return Err("invalid currency id"),
			};
			values.push((lp_share_currency_id, 100 * dollar(DNAR)));
		}
	}: _(RawOrigin::Root, values)

}

#[cfg(test)]
mod tests {
	use super::*;
	use frame_support::assert_ok;

	fn new_test_ext() -> sp_io::TestExternalities {
		frame_system::GenesisConfig::default()
			.build_storage::<Runtime>()
			.unwrap()
			.into()
	}

	#[test]
	fn test_deposit_dex_share() {
		new_test_ext().execute_with(|| {
			assert_ok!(test_benchmark_deposit_dex_share());
		});
	}

	#[test]
	fn test_withdraw_dex_share() {
		new_test_ext().execute_with(|| {
			assert_ok!(test_benchmark_withdraw_dex_share());
		});
	}

	#[test]
	fn test_claim_rewards() {
		new_test_ext().execute_with(|| {
			assert_ok!(test_benchmark_claim_rewards());
		});
	}

	#[test]
	fn test_update_reserves_incentive_rewards() {
		new_test_ext().execute_with(|| {
			assert_ok!(test_benchmark_update_reserves_incentive_rewards());
		});
	}

	#[test]
	fn test_update_dex_incentive_rewards() {
		new_test_ext().execute_with(|| {
			assert_ok!(test_benchmark_update_dex_incentive_rewards());
		});
	}

	#[test]
	fn test_update_dex_saving_rates() {
		new_test_ext().execute_with(|| {
			assert_ok!(test_benchmark_update_dex_saving_rates());
		});
	}
}
