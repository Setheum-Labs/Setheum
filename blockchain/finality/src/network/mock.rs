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

use std::{sync::Arc, time::Duration};

use futures::{channel::mpsc, StreamExt};
use parity_scale_codec::{Decode, Encode, Output};
use sc_keystore::LocalKeystore;
use sp_keystore::Keystore as _;
use tokio::time::timeout;

use crate::{
    primitives ::KEY_TYPE,
    crypto::{AuthorityPen, AuthorityVerifier},
    AuthorityId, NodeIndex,
};

#[derive(Hash, Debug, Clone, PartialEq, Eq)]
pub struct MockData {
    data: u32,
    filler: Vec<u8>,
    decodes: bool,
}

impl MockData {
    pub fn new(data: u32, filler_size: usize) -> MockData {
        MockData {
            data,
            filler: vec![0; filler_size],
            decodes: true,
        }
    }
}

impl Encode for MockData {
    fn size_hint(&self) -> usize {
        self.data.size_hint() + self.filler.size_hint() + self.decodes.size_hint()
    }

    fn encode_to<T: Output + ?Sized>(&self, dest: &mut T) {
        // currently this is exactly the default behaviour, but we still
        // need it here to make sure that decode works in the future
        self.data.encode_to(dest);
        self.filler.encode_to(dest);
        self.decodes.encode_to(dest);
    }
}

impl Decode for MockData {
    fn decode<I: parity_scale_codec::Input>(
        value: &mut I,
    ) -> Result<Self, parity_scale_codec::Error> {
        let data = u32::decode(value)?;
        let filler = Vec::<u8>::decode(value)?;
        let decodes = bool::decode(value)?;
        if !decodes {
            return Err("Simulated decode failure.".into());
        }
        Ok(Self {
            data,
            filler,
            decodes,
        })
    }
}

#[derive(Clone)]
pub struct Channel<T>(
    pub mpsc::UnboundedSender<T>,
    pub Arc<tokio::sync::Mutex<mpsc::UnboundedReceiver<T>>>,
);

const TIMEOUT_FAIL: Duration = Duration::from_secs(10);

impl<T> Channel<T> {
    pub fn new() -> Self {
        let (tx, rx) = mpsc::unbounded();
        Channel(tx, Arc::new(tokio::sync::Mutex::new(rx)))
    }

    pub async fn next(&mut self) -> Option<T> {
        timeout(TIMEOUT_FAIL, self.1.lock().await.next())
            .await
            .ok()
            .flatten()
    }

    pub async fn take(&mut self, n: usize) -> Vec<T> {
        timeout(
            TIMEOUT_FAIL,
            self.1.lock().await.by_ref().take(n).collect::<Vec<_>>(),
        )
        .await
        .unwrap_or_default()
    }

    pub async fn try_next(&self) -> Option<T> {
        self.1.lock().await.try_next().unwrap_or(None)
    }

    pub async fn close(self) -> Option<T> {
        self.0.close_channel();
        self.try_next().await
    }
}

impl<T> Default for Channel<T> {
    fn default() -> Self {
        Self::new()
    }
}

pub fn crypto_basics(
    num_crypto_basics: usize,
) -> (Vec<(NodeIndex, AuthorityPen)>, AuthorityVerifier) {
    let keystore = Arc::new(LocalKeystore::in_memory());
    let mut auth_ids = Vec::with_capacity(num_crypto_basics);
    for _ in 0..num_crypto_basics {
        let pk = keystore.ed25519_generate_new(KEY_TYPE, None).unwrap();
        auth_ids.push(AuthorityId::from(pk));
    }
    let mut result = Vec::with_capacity(num_crypto_basics);
    for (i, auth_id) in auth_ids.iter().enumerate() {
        result.push((
            NodeIndex(i),
            AuthorityPen::new(auth_id.clone(), keystore.clone())
                .expect("The keys should sign successfully"),
        ));
    }
    (result, AuthorityVerifier::new(auth_ids))
}
