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

use substrate_prometheus_endpoint::{register, Gauge, PrometheusError, Registry, U64};

#[derive(Clone)]
pub enum Metrics {
    Prometheus {
        incoming_connections: Gauge<U64>,
        missing_incoming_connections: Gauge<U64>,
        outgoing_connections: Gauge<U64>,
        missing_outgoing_connections: Gauge<U64>,
    },
    Noop,
}

pub enum Event {
    NewOutgoing,
    NewIncoming,
    DelOutgoing,
    DelIncoming,
    ConnectedOutgoing,
    ConnectedIncoming,
    DisconnectedOutgoing,
    DisconnectedIncoming,
}

impl Metrics {
    pub fn new(registry: Option<Registry>) -> Result<Self, PrometheusError> {
        match registry {
            Some(registry) => Ok(Metrics::Prometheus {
                incoming_connections: register(
                    Gauge::new(
                        "clique_network_incoming_connections",
                        "present incoming connections",
                    )?,
                    &registry,
                )?,
                missing_incoming_connections: register(
                    Gauge::new(
                        "clique_network_missing_incoming_connections",
                        "difference between expected and present incoming connections",
                    )?,
                    &registry,
                )?,
                outgoing_connections: register(
                    Gauge::new(
                        "clique_network_outgoing_connections",
                        "present outgoing connections",
                    )?,
                    &registry,
                )?,
                missing_outgoing_connections: register(
                    Gauge::new(
                        "clique_network_missing_outgoing_connections",
                        "difference between expected and present outgoing connections",
                    )?,
                    &registry,
                )?,
            }),
            None => Ok(Metrics::Noop),
        }
    }

    pub fn noop() -> Self {
        Metrics::Noop
    }

    pub fn report_event(&self, event: Event) {
        use Event::*;
        if let Metrics::Prometheus {
            incoming_connections,
            outgoing_connections,
            missing_incoming_connections,
            missing_outgoing_connections,
        } = self
        {
            match event {
                NewIncoming => missing_incoming_connections.inc(),
                NewOutgoing => missing_outgoing_connections.inc(),
                DelIncoming => missing_incoming_connections.dec(),
                DelOutgoing => missing_outgoing_connections.dec(),
                ConnectedIncoming => {
                    incoming_connections.inc();
                    missing_incoming_connections.dec();
                }
                ConnectedOutgoing => {
                    outgoing_connections.inc();
                    missing_outgoing_connections.dec();
                }
                DisconnectedIncoming => {
                    incoming_connections.dec();
                    missing_incoming_connections.inc();
                }
                DisconnectedOutgoing => {
                    outgoing_connections.dec();
                    missing_outgoing_connections.inc();
                }
            }
        }
    }
}
