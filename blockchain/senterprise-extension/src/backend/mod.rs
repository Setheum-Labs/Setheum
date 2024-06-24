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

use setheum_runtime_interfaces::snark_verifier::VerifierError::*;
use environment::Environment as EnvironmentT;
use executor::BackendExecutor as BackendExecutorT;
use frame_support::{pallet_prelude::DispatchError, sp_runtime::AccountId32};
use frame_system::Config as SystemConfig;
use log::error;
use pallet_contracts::{
    chain_extension::{
        ChainExtension, Environment as SubstrateEnvironment, Ext, InitState,
        Result as ChainExtensionResult, RetVal,
    },
    Config as ContractsConfig,
};
use module_feature_control::{
    Config as FeatureControlConfig, Feature, Pallet as FeatureControlPallet,
};
use module_vk_storage::Config as VkStorageConfig;
use sp_std::marker::PhantomData;

use crate::{
    backend::weights::{AlephWeight, WeightInfo},
    extension_ids::{EXTENSION_ID as SENTERPRISE_EXTENSION_ID, VERIFY_FUNC_ID},
    status_codes::*,
};

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;
mod environment;
mod executor;
#[cfg(test)]
mod tests;
mod weights;

#[cfg(feature = "runtime-benchmarks")]
pub use benchmarking::ChainExtensionBenchmarking;

type ByteCount = u32;

/// Minimal runtime configuration required by the standard chain extension executor.
pub trait MinimalRuntime: VkStorageConfig + ContractsConfig + FeatureControlConfig {}
impl<R: VkStorageConfig + ContractsConfig + FeatureControlConfig> MinimalRuntime for R {}

/// The actual implementation of the chain extension. This is the code on the runtime side that will
/// be executed when the chain extension is called.
pub struct SenterpriseChainExtension<Runtime> {
    _config: PhantomData<Runtime>,
}

impl<Runtime> Default for SenterpriseChainExtension<Runtime> {
    fn default() -> Self {
        Self {
            _config: PhantomData,
        }
    }
}

impl<Runtime: MinimalRuntime> ChainExtension<Runtime> for SenterpriseChainExtension<Runtime>
where
    <Runtime as SystemConfig>::RuntimeOrigin: From<Option<AccountId32>>,
{
    fn call<E: Ext<T = Runtime>>(
        &mut self,
        env: SubstrateEnvironment<E, InitState>,
    ) -> ChainExtensionResult<RetVal> {
        let (ext_id, func_id) = (env.ext_id(), env.func_id());
        match (ext_id, func_id) {
            (SENTERPRISE_EXTENSION_ID, VERIFY_FUNC_ID) => {
                Self::verify::<Runtime, _, AlephWeight<Runtime>>(env.buf_in_buf_out())
            }
            _ => {
                error!("There is no function `{func_id}` registered for an extension `{ext_id}`");
                Err(DispatchError::Other("Called an unregistered `func_id`"))
            }
        }
    }

    fn enabled() -> bool {
        FeatureControlPallet::<Runtime>::is_feature_enabled(Feature::OnChainVerifier)
    }
}

impl<Runtime: MinimalRuntime> SenterpriseChainExtension<Runtime>
where
    <Runtime as SystemConfig>::RuntimeOrigin: From<Option<AccountId32>>,
{
    /// Handle `verify` chain extension call.
    pub fn verify<
        BackendExecutor: BackendExecutorT,
        Environment: EnvironmentT,
        Weighting: WeightInfo,
    >(
        mut env: Environment,
    ) -> ChainExtensionResult<RetVal> {
        // ------- Pre-charge optimistic weight. ---------------------------------------------------
        let _pre_charge = env.charge_weight(Weighting::verify())?;

        // ------- Read the arguments. -------------------------------------------------------------
        env.charge_weight(Weighting::verify_read_args(env.in_len()))?;
        let args = env.read_as_unbounded(env.in_len())?;

        // ------- Forward the call. ---------------------------------------------------------------
        let result = BackendExecutor::verify(args);

        // ------- Translate the status. -----------------------------------------------------------
        let status = match result {
            Ok(()) => VERIFY_SUCCESS,
            Err(DeserializingPublicInputFailed) => VERIFY_DESERIALIZING_INPUT_FAIL,
            Err(UnknownVerificationKeyIdentifier) => VERIFY_UNKNOWN_IDENTIFIER,
            Err(DeserializingVerificationKeyFailed) => VERIFY_DESERIALIZING_KEY_FAIL,
            Err(VerificationFailed) => VERIFY_VERIFICATION_FAIL,
            Err(IncorrectProof) => VERIFY_INCORRECT_PROOF,
        };
        Ok(RetVal::Converging(status))
    }
}
