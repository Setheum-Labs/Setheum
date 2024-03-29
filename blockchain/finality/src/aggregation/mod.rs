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

//! Module to glue legacy and current version of the aggregator;

use std::marker::PhantomData;

use current_aleph_aggregator::NetworkError as CurrentNetworkError;
use legacy_aleph_aggregator::NetworkError as LegacyNetworkError;

use crate::{
    abft::SignatureSet,
    primitives ::BlockHash,
    crypto::Signature,
    mpsc,
    network::{
        data::{Network, SendError},
        Data,
    },
    Keychain,
};

pub type LegacyRmcNetworkData =
    legacy_aleph_aggregator::RmcNetworkData<BlockHash, Signature, SignatureSet<Signature>>;
pub type CurrentRmcNetworkData =
    current_aleph_aggregator::RmcNetworkData<BlockHash, Signature, SignatureSet<Signature>>;

pub type LegacySignableBlockHash = legacy_aleph_aggregator::SignableHash<BlockHash>;
pub type LegacyRmc<'a> =
    legacy_aleph_bft_rmc::ReliableMulticast<'a, LegacySignableBlockHash, Keychain>;

pub struct NoopMetrics;

impl legacy_aleph_aggregator::Metrics<BlockHash> for NoopMetrics {
    fn report_aggregation_complete(&mut self, _: BlockHash) {}
}

pub type LegacyAggregator<'a, N> = legacy_aleph_aggregator::IO<
    BlockHash,
    LegacyRmcNetworkData,
    NetworkWrapper<LegacyRmcNetworkData, N>,
    SignatureSet<Signature>,
    LegacyRmc<'a>,
    NoopMetrics,
>;

pub type CurrentAggregator<N> =
    current_aleph_aggregator::IO<BlockHash, NetworkWrapper<CurrentRmcNetworkData, N>, Keychain>;

enum EitherAggregator<'a, CN, LN>
where
    LN: Network<LegacyRmcNetworkData>,
    CN: Network<CurrentRmcNetworkData>,
{
    Current(Box<CurrentAggregator<CN>>),
    Legacy(LegacyAggregator<'a, LN>),
}

/// Wrapper on the aggregator, which is either current or legacy one. Depending on the inner variant
/// it behaves runs the legacy one or the current.
pub struct Aggregator<'a, CN, LN>
where
    LN: Network<LegacyRmcNetworkData>,
    CN: Network<CurrentRmcNetworkData>,
{
    agg: EitherAggregator<'a, CN, LN>,
}

impl<'a, CN, LN> Aggregator<'a, CN, LN>
where
    LN: Network<LegacyRmcNetworkData>,
    CN: Network<CurrentRmcNetworkData>,
{
    pub fn new_legacy(multikeychain: &'a Keychain, rmc_network: LN) -> Self {
        let (messages_for_rmc, messages_from_network) = mpsc::unbounded();
        let (messages_for_network, messages_from_rmc) = mpsc::unbounded();
        let scheduler = legacy_aleph_bft_rmc::DoublingDelayScheduler::new(
            tokio::time::Duration::from_millis(500),
        );
        let rmc = legacy_aleph_bft_rmc::ReliableMulticast::new(
            messages_from_network,
            messages_for_network,
            multikeychain,
            legacy_aleph_bft::Keychain::node_count(multikeychain),
            scheduler,
        );
        // For the compatibility with the legacy aggregator we need extra `Option` layer
        let aggregator = legacy_aleph_aggregator::BlockSignatureAggregator::new(NoopMetrics);
        let aggregator_io = LegacyAggregator::<LN>::new(
            messages_for_rmc,
            messages_from_rmc,
            NetworkWrapper::new(rmc_network),
            rmc,
            aggregator,
        );

        Self {
            agg: EitherAggregator::Legacy(aggregator_io),
        }
    }

    pub fn new_current(multikeychain: &'a Keychain, rmc_network: CN) -> Self {
        let scheduler = current_aleph_bft_rmc::DoublingDelayScheduler::new(
            tokio::time::Duration::from_millis(500),
        );
        let rmc_handler = current_aleph_bft_rmc::Handler::new(multikeychain.clone());
        let rmc_service = current_aleph_bft_rmc::Service::new(scheduler, rmc_handler);
        let aggregator = current_aleph_aggregator::BlockSignatureAggregator::new();
        let aggregator_io =
            CurrentAggregator::<CN>::new(NetworkWrapper::new(rmc_network), rmc_service, aggregator);

        Self {
            agg: EitherAggregator::Current(Box::new(aggregator_io)),
        }
    }

    pub async fn start_aggregation(&mut self, h: BlockHash) {
        match &mut self.agg {
            EitherAggregator::Current(agg) => agg.start_aggregation(h).await,
            EitherAggregator::Legacy(agg) => agg.start_aggregation(h).await,
        }
    }

    pub async fn next_multisigned_hash(&mut self) -> Option<(BlockHash, SignatureSet<Signature>)> {
        match &mut self.agg {
            EitherAggregator::Current(agg) => agg.next_multisigned_hash().await,
            EitherAggregator::Legacy(agg) => agg.next_multisigned_hash().await,
        }
    }

    pub fn status_report(&self) {
        match &self.agg {
            EitherAggregator::Current(agg) => agg.status_report(),
            EitherAggregator::Legacy(agg) => agg.status_report(),
        }
    }
}

pub struct NetworkWrapper<D: Data, N: Network<D>>(N, PhantomData<D>);

impl<D: Data, N: Network<D>> NetworkWrapper<D, N> {
    pub fn new(network: N) -> Self {
        Self(network, PhantomData)
    }
}

#[async_trait::async_trait]
impl<T, D> legacy_aleph_aggregator::ProtocolSink<D> for NetworkWrapper<D, T>
where
    T: Network<D>,
    D: Data,
{
    async fn next(&mut self) -> Option<D> {
        self.0.next().await
    }

    fn send(
        &self,
        data: D,
        recipient: legacy_aleph_bft::Recipient,
    ) -> Result<(), LegacyNetworkError> {
        self.0.send(data, recipient.into()).map_err(|e| match e {
            SendError::SendFailed => LegacyNetworkError::SendFail,
        })
    }
}

#[async_trait::async_trait]
impl<T, D> current_aleph_aggregator::ProtocolSink<D> for NetworkWrapper<D, T>
where
    T: Network<D>,
    D: Data,
{
    async fn next(&mut self) -> Option<D> {
        self.0.next().await
    }

    fn send(
        &self,
        data: D,
        recipient: current_aleph_bft::Recipient,
    ) -> Result<(), CurrentNetworkError> {
        self.0.send(data, recipient.into()).map_err(|e| match e {
            SendError::SendFailed => CurrentNetworkError::SendFail,
        })
    }
}
