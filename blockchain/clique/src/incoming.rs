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

use std::fmt::{Display, Error as FmtError, Formatter};

use futures::channel::{mpsc, oneshot};
use log::{debug, info};

use crate::{
    metrics::Metrics,
    protocols::{protocol, ProtocolError, ProtocolNegotiationError, ResultForService},
    Data, PublicKey, SecretKey, Splittable, LOG_TARGET,
};

enum IncomingError<PK: PublicKey> {
    ProtocolNegotiationError(ProtocolNegotiationError),
    ProtocolError(ProtocolError<PK>),
}

impl<PK: PublicKey> Display for IncomingError<PK> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), FmtError> {
        use IncomingError::*;
        match self {
            ProtocolNegotiationError(e) => write!(f, "protocol negotiation error: {e}"),
            ProtocolError(e) => write!(f, "protocol error: {e}"),
        }
    }
}

impl<PK: PublicKey> From<ProtocolNegotiationError> for IncomingError<PK> {
    fn from(e: ProtocolNegotiationError) -> Self {
        IncomingError::ProtocolNegotiationError(e)
    }
}

impl<PK: PublicKey> From<ProtocolError<PK>> for IncomingError<PK> {
    fn from(e: ProtocolError<PK>) -> Self {
        IncomingError::ProtocolError(e)
    }
}

async fn manage_incoming<SK: SecretKey, D: Data, S: Splittable>(
    secret_key: SK,
    stream: S,
    result_for_parent: mpsc::UnboundedSender<ResultForService<SK::PublicKey, D>>,
    data_for_user: mpsc::UnboundedSender<D>,
    authorization_requests_sender: mpsc::UnboundedSender<(SK::PublicKey, oneshot::Sender<bool>)>,
    metrics: Metrics,
) -> Result<(), IncomingError<SK::PublicKey>> {
    debug!(
        target: LOG_TARGET,
        "Performing incoming protocol negotiation."
    );
    let (stream, protocol) = protocol(stream).await?;
    debug!(target: LOG_TARGET, "Negotiated protocol, running.");
    Ok(protocol
        .manage_incoming(
            stream,
            secret_key,
            result_for_parent,
            data_for_user,
            authorization_requests_sender,
            metrics,
        )
        .await?)
}

/// Manage an incoming connection. After the handshake it will send the recognized PublicKey to
/// the parent, together with an exit channel for this process. When this channel is dropped the
/// process ends. Whenever data arrives on this connection it will be passed to the user. Any
/// failures in receiving data result in the process stopping, we assume the other side will
/// reestablish it if necessary.
pub async fn incoming<SK: SecretKey, D: Data, S: Splittable>(
    secret_key: SK,
    stream: S,
    result_for_parent: mpsc::UnboundedSender<ResultForService<SK::PublicKey, D>>,
    data_for_user: mpsc::UnboundedSender<D>,
    authorization_requests_sender: mpsc::UnboundedSender<(SK::PublicKey, oneshot::Sender<bool>)>,
    metrics: Metrics,
) {
    let addr = stream.peer_address_info();
    if let Err(e) = manage_incoming(
        secret_key,
        stream,
        result_for_parent,
        data_for_user,
        authorization_requests_sender,
        metrics,
    )
    .await
    {
        info!(
            target: LOG_TARGET,
            "Incoming connection from {} failed: {}.", addr, e
        );
    }
}
