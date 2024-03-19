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

use std::collections::{HashMap, HashSet};

use parity_scale_codec::{DecodeAll, Error as CodecError};
use sc_network::{config::FullNetworkConfiguration, ExtendedPeerInfo, PeerId};
use sc_network_common::{role::Roles, sync::message::BlockAnnouncesHandshake};
use sp_runtime::traits::{Block, Header, Saturating};

use crate::{BlockHash, BlockNumber};

pub enum ConnectError {
    BadlyEncodedHandshake(CodecError),
    BadHandshakeGenesis,
    PeerAlreadyConnected,
    TooManyFullInboundPeers,
    TooManyFullOutboundPeers,
    TooManyLightPeers,
}

pub enum DisconnectError {
    PeerWasNotConnected,
}

struct PeerInfo {
    role: Roles,
    is_inbound: bool,
}

/// Handler for the base protocol.
pub struct Handler<B>
where
    B: Block<Hash = BlockHash>,
    B::Header: Header<Number = BlockNumber>,
{
    reserved_nodes: HashSet<PeerId>,
    peers: HashMap<PeerId, PeerInfo>,
    // the below counters and bounds ignore the nodes which belong to `reserved_nodes`
    num_full_in_peers: usize,
    num_full_out_peers: usize,
    num_light_peers: usize,
    max_full_in_peers: usize,
    max_full_out_peers: usize,
    max_light_peers: usize,
    genesis_hash: B::Hash,
}

impl<B> Handler<B>
where
    B: Block<Hash = BlockHash>,
    B::Header: Header<Number = BlockNumber>,
{
    /// Create a new handler.
    pub fn new(genesis_hash: B::Hash, net_config: &FullNetworkConfiguration) -> Self {
        let reserved_nodes = net_config
            .network_config
            .default_peers_set
            .reserved_nodes
            .iter()
            .map(|reserved| reserved.peer_id)
            .collect();

        // It is assumed that `default_peers_set.out_peers` only refers to full nodes, but
        // `default_peers_set.in_peers` refers to both full and light nodes.
        // Moreover, `default_peers_set_num_full` refers to the total of full nodes.
        let max_full_out_peers = net_config.network_config.default_peers_set.out_peers as usize;
        let max_full_in_peers = (net_config.network_config.default_peers_set_num_full as usize)
            .saturating_sub(max_full_out_peers);
        let max_light_peers = (net_config.network_config.default_peers_set.in_peers as usize)
            .saturating_sub(max_full_in_peers);

        Handler {
            reserved_nodes,
            peers: HashMap::new(),
            max_full_in_peers,
            max_full_out_peers,
            max_light_peers,
            num_full_in_peers: 0,
            num_full_out_peers: 0,
            num_light_peers: 0,
            genesis_hash,
        }
    }

    /// Returns a list of connected peers with some additional information.
    // TODO: This shouldn't need to return the substrate type after replacing RPCs.
    pub fn peers_info(&self) -> Vec<(PeerId, ExtendedPeerInfo<B>)>
    where
        B: Block<Hash = BlockHash>,
        B::Header: Header<Number = BlockNumber>,
    {
        self.peers
            .iter()
            .map(|(id, info)| {
                (
                    *id,
                    ExtendedPeerInfo {
                        roles: info.role,
                        best_hash: Default::default(),
                        best_number: 0,
                    },
                )
            })
            .collect()
    }

    fn verify_connection(
        &self,
        peer_id: PeerId,
        handshake: Vec<u8>,
        is_inbound: bool,
    ) -> Result<Roles, ConnectError> {
        let handshake = BlockAnnouncesHandshake::<B>::decode_all(&mut &handshake[..])
            .map_err(ConnectError::BadlyEncodedHandshake)?;
        if handshake.genesis_hash != self.genesis_hash {
            return Err(ConnectError::BadHandshakeGenesis);
        }

        if self.peers.contains_key(&peer_id) {
            return Err(ConnectError::PeerAlreadyConnected);
        }

        if self.reserved_nodes.contains(&peer_id) {
            return Ok(handshake.roles);
        }

        // Check slot constraints depending on the node's role and the connection's direction.
        if is_inbound
            && handshake.roles.is_full()
            && self.num_full_in_peers >= self.max_full_in_peers
        {
            return Err(ConnectError::TooManyFullInboundPeers);
        }
        if !is_inbound
            && handshake.roles.is_full()
            && self.num_full_out_peers >= self.max_full_out_peers
        {
            return Err(ConnectError::TooManyFullOutboundPeers);
        }
        if handshake.roles.is_light() && self.num_light_peers >= self.max_light_peers {
            return Err(ConnectError::TooManyLightPeers);
        }

        Ok(handshake.roles)
    }

    pub fn on_peer_connect(
        &mut self,
        peer_id: PeerId,
        handshake: Vec<u8>,
        is_inbound: bool,
    ) -> Result<(), ConnectError> {
        let role = self.verify_connection(peer_id, handshake, is_inbound)?;

        self.peers.insert(peer_id, PeerInfo { role, is_inbound });

        if self.reserved_nodes.contains(&peer_id) {
            return Ok(());
        }

        // Assign a slot for the node depending on their role and the connection's direction.
        if is_inbound && role.is_full() {
            self.num_full_in_peers += 1;
        } else if !is_inbound && role.is_full() {
            self.num_full_out_peers += 1;
        } else if role.is_light() {
            self.num_light_peers += 1;
        }

        Ok(())
    }

    pub fn on_peer_disconnect(&mut self, peer_id: PeerId) -> Result<(), DisconnectError> {
        let info = self
            .peers
            .remove(&peer_id)
            .ok_or(DisconnectError::PeerWasNotConnected)?;

        if self.reserved_nodes.contains(&peer_id) {
            return Ok(());
        }

        // Free the slot of the node depending on their role and the connection's direction.
        if info.is_inbound && info.role.is_full() {
            self.num_full_in_peers.saturating_dec();
        } else if !info.is_inbound && info.role.is_full() {
            self.num_full_out_peers.saturating_dec();
        } else if info.role.is_light() {
            self.num_light_peers.saturating_dec();
        }

        Ok(())
    }
}
