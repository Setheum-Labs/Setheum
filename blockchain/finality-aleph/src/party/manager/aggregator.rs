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

use std::fmt::Display;

use futures::{
    channel::{mpsc, oneshot},
    pin_mut,
    stream::FusedStream,
    StreamExt,
};
use log::{debug, error, trace};
use tokio::time;

use crate::{
    abft::SignatureSet,
    aggregation::Aggregator,
    aleph_primitives::BlockHash,
    block::{
        substrate::{Justification, JustificationTranslator},
        Header, HeaderBackend,
    },
    crypto::Signature,
    justification::AlephJustification,
    metrics::{AllBlockMetrics, Checkpoint},
    network::data::Network,
    party::{
        manager::aggregator::AggregatorVersion::{Current, Legacy},
        AuthoritySubtaskCommon, Task,
    },
    sync::JustificationSubmissions,
    BlockId, CurrentRmcNetworkData, Keychain, LegacyRmcNetworkData, SessionBoundaries,
    STATUS_REPORT_INTERVAL,
};

#[derive(Debug)]
pub enum Error {
    MultisignaturesStreamTerminated,
    UnableToProcessHash,
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::MultisignaturesStreamTerminated => {
                write!(f, "The stream of multisigned hashes has ended.")
            }
            Error::UnableToProcessHash => write!(f, "Error while processing a hash."),
        }
    }
}

/// IO channels used by the aggregator task.
pub struct IO<JS>
where
    JS: JustificationSubmissions<Justification> + Send + Sync + Clone,
{
    pub blocks_from_interpreter: mpsc::UnboundedReceiver<BlockId>,
    pub justifications_for_chain: JS,
    pub justification_translator: JustificationTranslator,
}

async fn process_new_block_data<CN, LN>(
    aggregator: &mut Aggregator<'_, CN, LN>,
    block: BlockId,
    metrics: &mut AllBlockMetrics,
) where
    CN: Network<CurrentRmcNetworkData>,
    LN: Network<LegacyRmcNetworkData>,
{
    trace!(target: "aleph-party", "Received unit {:?} in aggregator.", block);
    let hash = block.hash();
    metrics.report_block(block, Checkpoint::Ordered, None);

    aggregator.start_aggregation(hash).await;
}

fn process_hash<H, C, JS>(
    hash: BlockHash,
    multisignature: SignatureSet<Signature>,
    justifications_for_chain: &mut JS,
    justification_translator: &JustificationTranslator,
    client: &C,
) -> Result<(), ()>
where
    H: Header,
    C: HeaderBackend<H> + 'static,
    JS: JustificationSubmissions<Justification> + Send + Sync + Clone,
{
    let number = client.hash_to_id(hash).unwrap().unwrap().number();
    // The unwrap might actually fail if data availability is not implemented correctly.
    let justification = match justification_translator.translate(
        AlephJustification::CommitteeMultisignature(multisignature),
        BlockId::new(hash, number),
    ) {
        Ok(justification) => justification,
        Err(e) => {
            error!(target: "aleph-party", "Issue with translating justification from Aggregator to Sync Justification: {}.", e);
            return Err(());
        }
    };
    if let Err(e) = justifications_for_chain.submit(justification) {
        error!(target: "aleph-party", "Issue with sending justification from Aggregator to JustificationHandler {}.", e);
        return Err(());
    }
    Ok(())
}

async fn run_aggregator<H, C, CN, LN, JS>(
    mut aggregator: Aggregator<'_, CN, LN>,
    io: IO<JS>,
    client: C,
    session_boundaries: &SessionBoundaries,
    mut metrics: AllBlockMetrics,
    mut exit_rx: oneshot::Receiver<()>,
) -> Result<(), Error>
where
    H: Header,
    JS: JustificationSubmissions<Justification> + Send + Sync + Clone,
    C: HeaderBackend<H> + 'static,
    LN: Network<LegacyRmcNetworkData>,
    CN: Network<CurrentRmcNetworkData>,
{
    let IO {
        blocks_from_interpreter,
        mut justifications_for_chain,
        justification_translator,
    } = io;

    let blocks_from_interpreter = blocks_from_interpreter.take_while(|block| {
        let block_num = block.number();
        async move {
            if block_num == session_boundaries.last_block() {
                debug!(target: "aleph-party", "Aggregator is processing last block in session.");
            }
            block_num <= session_boundaries.last_block()
        }
    });
    pin_mut!(blocks_from_interpreter);
    let mut hash_of_last_block = None;
    let mut no_more_blocks = blocks_from_interpreter.is_terminated();

    let mut status_ticker = time::interval(STATUS_REPORT_INTERVAL);

    loop {
        trace!(target: "aleph-party", "Aggregator Loop started a next iteration");
        tokio::select! {
            maybe_block = blocks_from_interpreter.next(), if !no_more_blocks => match maybe_block {
                Some(block) => {
                    hash_of_last_block = Some(block.hash());
                    process_new_block_data::<CN, LN>(
                        &mut aggregator,
                        block,
                        &mut metrics
                    ).await;
                },
                None => {
                    debug!(target: "aleph-party", "Blocks ended in aggregator.");
                    no_more_blocks = true;
                },
            },
            multisigned_hash = aggregator.next_multisigned_hash() => {
                let (hash, multisignature) = multisigned_hash.ok_or(Error::MultisignaturesStreamTerminated)?;
                process_hash(hash, multisignature, &mut justifications_for_chain, &justification_translator, &client).map_err(|_| Error::UnableToProcessHash)?;
                if Some(hash) == hash_of_last_block {
                    hash_of_last_block = None;
                }
            },
            _ = status_ticker.tick() => {
                aggregator.status_report();
            },
            _ = &mut exit_rx => {
                debug!(target: "aleph-party", "Aggregator received exit signal. Terminating.");
                break;
            }
        }
        if hash_of_last_block.is_none() && no_more_blocks {
            debug!(target: "aleph-party", "Aggregator processed all provided blocks. Terminating.");
            break;
        }
    }
    debug!(target: "aleph-party", "Aggregator finished its work.");
    Ok(())
}

pub enum AggregatorVersion<CN, LN> {
    Current(CN),
    Legacy(LN),
}

/// Runs the justification signature aggregator within a single session.
pub fn task<H, C, CN, LN, JS>(
    subtask_common: AuthoritySubtaskCommon,
    client: C,
    io: IO<JS>,
    session_boundaries: SessionBoundaries,
    metrics: AllBlockMetrics,
    multikeychain: Keychain,
    version: AggregatorVersion<CN, LN>,
) -> Task
where
    H: Header,
    JS: JustificationSubmissions<Justification> + Send + Sync + Clone + 'static,
    C: HeaderBackend<H> + 'static,
    LN: Network<LegacyRmcNetworkData> + 'static,
    CN: Network<CurrentRmcNetworkData> + 'static,
{
    let AuthoritySubtaskCommon {
        spawn_handle,
        session_id,
    } = subtask_common;
    let (stop, exit) = oneshot::channel();
    let task = {
        async move {
            let aggregator_io = match version {
                Current(rmc_network) => Aggregator::new_current(&multikeychain, rmc_network),
                Legacy(rmc_network) => Aggregator::new_legacy(&multikeychain, rmc_network),
            };
            debug!(target: "aleph-party", "Running the aggregator task for {:?}", session_id);
            let result = run_aggregator(
                aggregator_io,
                io,
                client,
                &session_boundaries,
                metrics,
                exit,
            )
            .await;
            let result = match result {
                Ok(_) => Ok(()),
                Err(err) => {
                    error!(target: "aleph-party", "Aggregator exited with error: {err}");
                    Err(())
                }
            };
            debug!(target: "aleph-party", "Aggregator task stopped for {:?}", session_id);
            result
        }
    };

    let handle =
        spawn_handle.spawn_essential_with_result("aleph/consensus_session_aggregator", task);
    Task::new(handle, stop)
}
