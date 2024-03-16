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

use rate_limiter::{RateLimiter, SleepingRateLimiter};
use tokio::io::AsyncRead;

use crate::{ConnectionInfo, Data, Dialer, Listener, PeerAddressInfo, Splittable, Splitted};

pub struct RateLimitedAsyncRead<Read> {
    rate_limiter: RateLimiter,
    read: Read,
}

impl<Read> RateLimitedAsyncRead<Read> {
    pub fn new(read: Read, rate_limiter: RateLimiter) -> Self {
        Self { rate_limiter, read }
    }
}

impl<Read: AsyncRead + Unpin> AsyncRead for RateLimitedAsyncRead<Read> {
    fn poll_read(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
        buf: &mut tokio::io::ReadBuf<'_>,
    ) -> std::task::Poll<std::io::Result<()>> {
        let this = self.get_mut();
        let read = std::pin::Pin::new(&mut this.read);
        this.rate_limiter.rate_limit(read, cx, buf)
    }
}

impl<Read: ConnectionInfo> ConnectionInfo for RateLimitedAsyncRead<Read> {
    fn peer_address_info(&self) -> PeerAddressInfo {
        self.read.peer_address_info()
    }
}

/// Implementation of the [Dialer] trait governing all returned [Dialer::Connection] instances by a rate-limiting wrapper.
#[derive(Clone)]
pub struct RateLimitingDialer<D> {
    dialer: D,
    rate_limiter: SleepingRateLimiter,
}

impl<D> RateLimitingDialer<D> {
    pub fn new(dialer: D, rate_limiter: SleepingRateLimiter) -> Self {
        Self {
            dialer,
            rate_limiter,
        }
    }
}

#[async_trait::async_trait]
impl<A, D> Dialer<A> for RateLimitingDialer<D>
where
    A: Data,
    D: Dialer<A>,
    <D::Connection as Splittable>::Sender: Unpin,
    <D::Connection as Splittable>::Receiver: Unpin,
{
    type Connection = Splitted<
        RateLimitedAsyncRead<<D::Connection as Splittable>::Receiver>,
        <D::Connection as Splittable>::Sender,
    >;
    type Error = D::Error;

    async fn connect(&mut self, address: A) -> Result<Self::Connection, Self::Error> {
        let connection = self.dialer.connect(address).await?;
        let (sender, receiver) = connection.split();
        Ok(Splitted(
            RateLimitedAsyncRead::new(receiver, RateLimiter::new(self.rate_limiter.clone())),
            sender,
        ))
    }
}

/// Implementation of the [Listener] trait governing all returned [Listener::Connection] instances by a rate-limiting wrapper.
pub struct RateLimitingListener<L> {
    listener: L,
    rate_limiter: SleepingRateLimiter,
}

impl<L> RateLimitingListener<L> {
    pub fn new(listener: L, rate_limiter: SleepingRateLimiter) -> Self {
        Self {
            listener,
            rate_limiter,
        }
    }
}

#[async_trait::async_trait]
impl<L: Listener + Send> Listener for RateLimitingListener<L> {
    type Connection = Splitted<
        RateLimitedAsyncRead<<L::Connection as Splittable>::Receiver>,
        <L::Connection as Splittable>::Sender,
    >;
    type Error = L::Error;

    async fn accept(&mut self) -> Result<Self::Connection, Self::Error> {
        let connection = self.listener.accept().await?;
        let (sender, receiver) = connection.split();
        Ok(Splitted(
            RateLimitedAsyncRead::new(receiver, RateLimiter::new(self.rate_limiter.clone())),
            sender,
        ))
    }
}
