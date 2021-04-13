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

//! The precompiles for EVM, includes standard Ethereum precompiles, and more:
//! - MultiCurrency at address `H160::from_low_u64_be(1024)`.

#![allow(clippy::upper_case_acronyms)]

mod mock;
mod tests;

use crate::is_setheum_precompile;
use frame_support::log;
use sevm::{
	precompiles::{Precompile, Precompiles},
	Context, ExitError, ExitSucceed,
};
use module_support::PrecompileCallerFilter as PrecompileCallerFilterT;
use primitives::PRECOMPILE_ADDRESS_START;
use sp_core::H160;
use sp_std::{marker::PhantomData, prelude::*};

pub mod settindex;
pub mod input;
pub mod multicurrency;
pub mod nft;
pub mod oracle;
pub mod schedule_call;
pub mod state_rent;

pub use settindex::SettinDexPrecompile;
pub use multicurrency::MultiCurrencyPrecompile;
pub use nft::NFTPrecompile;
pub use oracle::OraclePrecompile;
pub use schedule_call::ScheduleCallPrecompile;
pub use state_rent::StateRentPrecompile;

pub type EthereumPrecompiles = (
	sevm::precompiles::ECRecover,
	sevm::precompiles::Sha256,
	sevm::precompiles::Ripemd160,
	sevm::precompiles::Identity,
);

pub struct AllPrecompiles<
	PrecompileCallerFilter,
	MultiCurrencyPrecompile,
	NFTPrecompile,
	StateRentPrecompile,
	OraclePrecompile,
	ScheduleCallPrecompile,
	SettinDexPrecompile,
>(
	PhantomData<(
		PrecompileCallerFilter,
		MultiCurrencyPrecompile,
		NFTPrecompile,
		StateRentPrecompile,
		OraclePrecompile,
		ScheduleCallPrecompile,
		SettinDexPrecompile,
	)>,
);

impl<
		PrecompileCallerFilter,
		MultiCurrencyPrecompile,
		NFTPrecompile,
		StateRentPrecompile,
		OraclePrecompile,
		ScheduleCallPrecompile,
		SettinDexPrecompile,
	> Precompiles
	for AllPrecompiles<
		PrecompileCallerFilter,
		MultiCurrencyPrecompile,
		NFTPrecompile,
		StateRentPrecompile,
		OraclePrecompile,
		ScheduleCallPrecompile,
		SettinDexPrecompile,
	> where
	MultiCurrencyPrecompile: Precompile,
	NFTPrecompile: Precompile,
	StateRentPrecompile: Precompile,
	OraclePrecompile: Precompile,
	ScheduleCallPrecompile: Precompile,
	PrecompileCallerFilter: PrecompileCallerFilterT,
	SettinDexPrecompile: Precompile,
{
	#[allow(clippy::type_complexity)]
	fn execute(
		address: H160,
		input: &[u8],
		target_gas: Option<u64>,
		context: &Context,
	) -> Option<core::result::Result<(ExitSucceed, Vec<u8>, u64), ExitError>> {
		EthereumPrecompiles::execute(address, input, target_gas, context).or_else(|| {
			if is_setheum_precompile(address) && !PrecompileCallerFilter::is_allowed(context.caller) {
				log::debug!(target: "sevm", "Precompile no permission");
				return Some(Err(ExitError::Other("no permission".into())));
			}

			if address == H160::from_low_u64_be(PRECOMPILE_ADDRESS_START) {
				Some(MultiCurrencyPrecompile::execute(input, target_gas, context))
			} else if address == H160::from_low_u64_be(PRECOMPILE_ADDRESS_START + 1) {
				Some(NFTPrecompile::execute(input, target_gas, context))
			} else if address == H160::from_low_u64_be(PRECOMPILE_ADDRESS_START + 2) {
				Some(StateRentPrecompile::execute(input, target_gas, context))
			} else if address == H160::from_low_u64_be(PRECOMPILE_ADDRESS_START + 3) {
				Some(OraclePrecompile::execute(input, target_gas, context))
			} else if address == H160::from_low_u64_be(PRECOMPILE_ADDRESS_START + 4) {
				Some(ScheduleCallPrecompile::execute(input, target_gas, context))
			} else if address == H160::from_low_u64_be(PRECOMPILE_ADDRESS_START + 5) {
				Some(SettinDexPrecompile::execute(input, target_gas, context))
			} else {
				None
			}
		})
	}
}
