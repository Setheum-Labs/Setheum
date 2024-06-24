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

mod arguments;
mod environment;
mod executor;
mod weighting;

use setheum_runtime::Runtime as SetheumRuntime;
use frame_support::pallet_prelude::Weight;
use pallet_contracts::chain_extension::RetVal;

use crate::{
    backend::{
        executor::BackendExecutor,
        tests::{
            arguments::verify_args,
            environment::{CorruptedMode, MockedEnvironment, StandardMode, VerifyMode},
            executor::{Panicker, VerifierError::*, VerifyErrorer, VerifyOkayer},
            weighting::TestWeight,
        },
        weights::WeightInfo,
    },
    status_codes::*,
    SenterpriseChainExtension,
};

fn simulate_verify<Exc: BackendExecutor>(expected_ret_val: u32) {
    let mut charged = Weight::zero();
    let env = MockedEnvironment::<VerifyMode, StandardMode>::new(&mut charged, verify_args());

    let result = SenterpriseChainExtension::<SetheumRuntime>::verify::<Exc, _, TestWeight>(env);

    assert!(matches!(result, Ok(RetVal::Converging(ret_val)) if ret_val == expected_ret_val));

    let expected_charged = <TestWeight as WeightInfo>::verify()
        + TestWeight::verify_read_args(verify_args().len() as u32);
    assert_eq!(charged, expected_charged);
}

#[test]
#[allow(non_snake_case)]
fn verify__charges_before_reading_arguments() {
    let in_len = 41;
    let mut charged = Weight::zero();
    // `CorruptedMode` ensures that the CE call will fail at argument reading/decoding phase.
    let env = MockedEnvironment::<VerifyMode, CorruptedMode>::new(&mut charged, in_len);

    // `Panicker` ensures that the call will not be forwarded to the pallet.
    let result = SenterpriseChainExtension::<SetheumRuntime>::verify::<Panicker, _, TestWeight>(env);

    assert!(matches!(result, Err(_)));
    assert_eq!(
        charged,
        TestWeight::verify_read_args(in_len) + TestWeight::verify()
    );
}

#[test]
#[allow(non_snake_case)]
fn verify__positive_scenario() {
    simulate_verify::<VerifyOkayer>(VERIFY_SUCCESS)
}

#[test]
#[allow(non_snake_case)]
fn verify__pallet_says_input_deserialization_failed() {
    simulate_verify::<VerifyErrorer<{ DeserializingPublicInputFailed }>>(
        VERIFY_DESERIALIZING_INPUT_FAIL,
    )
}

#[test]
#[allow(non_snake_case)]
fn verify__pallet_says_no_such_vk() {
    simulate_verify::<VerifyErrorer<{ UnknownVerificationKeyIdentifier }>>(
        VERIFY_UNKNOWN_IDENTIFIER,
    )
}

#[test]
#[allow(non_snake_case)]
fn verify__pallet_says_vk_deserialization_failed() {
    simulate_verify::<VerifyErrorer<{ DeserializingVerificationKeyFailed }>>(
        VERIFY_DESERIALIZING_KEY_FAIL,
    )
}

#[test]
#[allow(non_snake_case)]
fn verify__pallet_says_verification_failed() {
    simulate_verify::<VerifyErrorer<{ VerificationFailed }>>(VERIFY_VERIFICATION_FAIL)
}

#[test]
#[allow(non_snake_case)]
fn verify__pallet_says_incorrect_proof() {
    simulate_verify::<VerifyErrorer<{ IncorrectProof }>>(VERIFY_INCORRECT_PROOF)
}
