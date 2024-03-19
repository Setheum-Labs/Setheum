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

use std::{io::Error as IoError, iter, net::ToSocketAddrs as _};

use derive_more::{AsRef, Display};
use log::info;
use network_clique::{Dialer, Listener, PeerId, PublicKey, SecretKey};
use parity_scale_codec::{Decode, Encode};
use sp_core::crypto::KeyTypeId;
use tokio::net::{TcpListener, TcpStream, ToSocketAddrs};

use crate::{
    primitives ::AuthorityId,
    crypto::{verify, AuthorityPen, Signature},
    network::{AddressingInformation, NetworkIdentity},
};

const LOG_TARGET: &str = "tcp-network";

pub const KEY_TYPE: KeyTypeId = KeyTypeId(*b"set/vn");

#[derive(PartialEq, Eq, Clone, Debug, Display, Hash, Decode, Encode, AsRef)]
#[as_ref(forward)]
pub struct AuthorityIdWrapper(AuthorityId);

impl From<AuthorityId> for AuthorityIdWrapper {
    fn from(value: AuthorityId) -> Self {
        AuthorityIdWrapper(value)
    }
}

impl PeerId for AuthorityIdWrapper {}

impl PublicKey for AuthorityIdWrapper {
    type Signature = Signature;

    fn verify(&self, message: &[u8], signature: &Self::Signature) -> bool {
        verify(&self.0, message, signature)
    }
}

#[async_trait::async_trait]
impl SecretKey for AuthorityPen {
    type Signature = Signature;
    type PublicKey = AuthorityIdWrapper;

    fn sign(&self, message: &[u8]) -> Self::Signature {
        AuthorityPen::sign(self, message)
    }

    fn public_key(&self) -> Self::PublicKey {
        self.authority_id().into()
    }
}

/// What can go wrong when handling addressing information.
#[derive(Debug, Hash, Clone, PartialEq, Eq)]
pub enum AddressingInformationError {
    /// Construction of an addressing information object requires at least one address.
    NoAddress,
}

#[derive(Debug, Hash, Encode, Decode, Clone, PartialEq, Eq)]
struct TcpAddressingInformation {
    peer_id: AuthorityId,
    // Easiest way to ensure that the Vec below is nonempty...
    primary_address: String,
    other_addresses: Vec<String>,
}

impl TcpAddressingInformation {
    fn new(
        addresses: Vec<String>,
        peer_id: AuthorityId,
    ) -> Result<TcpAddressingInformation, AddressingInformationError> {
        let mut addresses = addresses.into_iter();
        let primary_address = match addresses.next() {
            Some(address) => address,
            None => return Err(AddressingInformationError::NoAddress),
        };
        Ok(TcpAddressingInformation {
            primary_address,
            other_addresses: addresses.collect(),
            peer_id,
        })
    }

    fn peer_id(&self) -> AuthorityId {
        self.peer_id.clone()
    }
}

/// A representation of TCP addressing information with an associated peer ID, self-signed.
#[derive(Debug, Hash, Encode, Decode, Clone, PartialEq, Eq)]
pub struct SignedTcpAddressingInformation {
    addressing_information: TcpAddressingInformation,
    signature: Signature,
}

impl AddressingInformation for SignedTcpAddressingInformation {
    type PeerId = AuthorityIdWrapper;

    fn peer_id(&self) -> Self::PeerId {
        self.addressing_information.peer_id().into()
    }

    fn verify(&self) -> bool {
        self.peer_id()
            .verify(&self.addressing_information.encode(), &self.signature)
    }

    fn address(&self) -> String {
        self.addressing_information.primary_address.clone()
    }
}

impl NetworkIdentity for SignedTcpAddressingInformation {
    type PeerId = AuthorityIdWrapper;
    type AddressingInformation = SignedTcpAddressingInformation;

    fn identity(&self) -> Self::AddressingInformation {
        self.clone()
    }
}

impl SignedTcpAddressingInformation {
    fn new(
        addresses: Vec<String>,
        authority_pen: &AuthorityPen,
    ) -> Result<SignedTcpAddressingInformation, AddressingInformationError> {
        let peer_id = authority_pen.authority_id();
        let addressing_information = TcpAddressingInformation::new(addresses, peer_id)?;
        let signature = authority_pen.sign(&addressing_information.encode());
        Ok(SignedTcpAddressingInformation {
            addressing_information,
            signature,
        })
    }
}

#[derive(Clone)]
struct TcpDialer;

#[async_trait::async_trait]
impl Dialer<SignedTcpAddressingInformation> for TcpDialer {
    type Connection = TcpStream;
    type Error = std::io::Error;

    async fn connect(
        &mut self,
        address: SignedTcpAddressingInformation,
    ) -> Result<Self::Connection, Self::Error> {
        let SignedTcpAddressingInformation {
            addressing_information,
            ..
        } = address;
        let TcpAddressingInformation {
            primary_address,
            other_addresses,
            ..
        } = addressing_information;
        let parsed_addresses: Vec<_> = iter::once(primary_address)
            .chain(other_addresses)
            .filter_map(|address| address.to_socket_addrs().ok())
            .flatten()
            .collect();
        let stream = TcpStream::connect(&parsed_addresses[..]).await?;
        if stream.set_linger(None).is_err() {
            info!(target: LOG_TARGET, "stream.set_linger(None) failed.");
        };
        Ok(stream)
    }
}

/// Possible errors when creating a TCP network.
#[derive(Debug)]
pub enum Error {
    Io(IoError),
    AddressingInformation(AddressingInformationError),
}

impl From<IoError> for Error {
    fn from(e: IoError) -> Self {
        Error::Io(e)
    }
}

impl From<AddressingInformationError> for Error {
    fn from(e: AddressingInformationError) -> Self {
        Error::AddressingInformation(e)
    }
}

/// Create a new tcp network, including an identity that can be used for constructing
/// authentications for other peers.
pub async fn new_tcp_network<A: ToSocketAddrs>(
    listening_addresses: A,
    external_addresses: Vec<String>,
    authority_pen: &AuthorityPen,
) -> Result<
    (
        impl Dialer<SignedTcpAddressingInformation>,
        impl Listener,
        impl NetworkIdentity<
            AddressingInformation = SignedTcpAddressingInformation,
            PeerId = AuthorityIdWrapper,
        >,
    ),
    Error,
> {
    let listener = TcpListener::bind(listening_addresses).await?;
    let identity = SignedTcpAddressingInformation::new(external_addresses, authority_pen)?;
    Ok((TcpDialer {}, listener, identity))
}

#[cfg(test)]
pub mod testing {

    use super::{AuthorityIdWrapper, SignedTcpAddressingInformation};
    use crate::{crypto::AuthorityPen, network::NetworkIdentity};

    /// Creates a realistic identity.
    pub fn new_identity(
        external_addresses: Vec<String>,
        authority_pen: &AuthorityPen,
    ) -> impl NetworkIdentity<
        AddressingInformation = SignedTcpAddressingInformation,
        PeerId = AuthorityIdWrapper,
    > {
        SignedTcpAddressingInformation::new(external_addresses, authority_pen)
            .expect("the provided addresses are fine")
    }
}
