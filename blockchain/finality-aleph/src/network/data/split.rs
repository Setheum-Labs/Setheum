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

// WARNING: if you plan to substitute the `ComponentNetwork` with SimpleNetwork (or something similar),
// you might find that it will require you to leak all private types declared here.
use std::{marker::PhantomData, sync::Arc};

use futures::channel::mpsc;
use log::{debug, trace};
use parity_scale_codec::{Decode, Encode};
use tokio::sync::Mutex;

use crate::{
    network::{
        data::{
            component::{Network, NetworkExt, Receiver, Sender, SimpleNetwork},
            SendError,
        },
        Data,
    },
    Recipient, Version, Versioned,
};

/// Used for routing data through split networks.
#[derive(Clone, Encode, Decode)]
pub enum Split<LeftData: Data, RightData: Data> {
    Left(LeftData),
    Right(RightData),
}

impl<LeftData: Versioned + Data, RightData: Data> Versioned for Split<LeftData, RightData> {
    const VERSION: Version = LeftData::VERSION;
}

trait Convert {
    type From;
    type To;

    fn convert(from: Self::From) -> Self::To;
}

#[derive(Clone)]
struct ToLeftSplitConvert<A, B> {
    _phantom: PhantomData<(A, B)>,
}

impl<A: Data, B: Data> Convert for ToLeftSplitConvert<A, B> {
    type From = A;
    type To = Split<A, B>;

    fn convert(from: Self::From) -> Self::To {
        Split::Left(from)
    }
}

#[derive(Clone)]
struct ToRightSplitConvert<A, B> {
    _phantom: PhantomData<(A, B)>,
}

impl<A: Data, B: Data> Convert for ToRightSplitConvert<A, B> {
    type From = B;
    type To = Split<A, B>;

    fn convert(b: Self::From) -> Self::To {
        Split::Right(b)
    }
}

#[derive(Clone)]
struct SplitSender<
    LeftData: Data,
    RightData: Data,
    S: Sender<Split<LeftData, RightData>>,
    Conv: Convert,
> {
    sender: S,
    phantom: PhantomData<(LeftData, RightData, Conv)>,
}

impl<
        LeftData: Data,
        RightData: Data,
        S: Sender<Split<LeftData, RightData>>,
        Conv: Convert<To = Split<LeftData, RightData>> + Clone + Send + Sync,
    > Sender<Conv::From> for SplitSender<LeftData, RightData, S, Conv>
where
    <Conv as Convert>::From: Data,
    <Conv as Convert>::To: Data,
{
    fn send(&self, data: Conv::From, recipient: Recipient) -> Result<(), SendError> {
        self.sender.send(Conv::convert(data), recipient)
    }
}

type LeftSender<LeftData, RightData, S> =
    SplitSender<LeftData, RightData, S, ToLeftSplitConvert<LeftData, RightData>>;

type RightSender<LeftData, RightData, S> =
    SplitSender<LeftData, RightData, S, ToRightSplitConvert<LeftData, RightData>>;

struct SplitReceiver<
    LeftData: Data,
    RightData: Data,
    R: Receiver<Split<LeftData, RightData>>,
    TranslatedData: Data,
> {
    receiver: Arc<Mutex<R>>,
    translated_receiver: mpsc::UnboundedReceiver<TranslatedData>,
    left_sender: mpsc::UnboundedSender<LeftData>,
    right_sender: mpsc::UnboundedSender<RightData>,
    name: &'static str,
}

#[async_trait::async_trait]
impl<
        LeftData: Data,
        RightData: Data,
        R: Receiver<Split<LeftData, RightData>>,
        TranslatedData: Data,
    > Receiver<TranslatedData> for SplitReceiver<LeftData, RightData, R, TranslatedData>
{
    async fn next(&mut self) -> Option<TranslatedData> {
        loop {
            tokio::select! {
                data = self.translated_receiver.next() => {
                    return data;
                },
                should_go_on = forward_or_wait(&self.receiver, &self.left_sender, &self.right_sender, self.name) => {
                    if !should_go_on {
                        return None;
                    }
                },
            }
        }
    }
}

type LeftReceiver<LeftData, RightData, R> = SplitReceiver<LeftData, RightData, R, LeftData>;

type RightReceiver<LeftData, RightData, R> = SplitReceiver<LeftData, RightData, R, RightData>;

async fn forward_or_wait<
    LeftData: Data,
    RightData: Data,
    R: Receiver<Split<LeftData, RightData>>,
>(
    receiver: &Arc<Mutex<R>>,
    left_sender: &mpsc::UnboundedSender<LeftData>,
    right_sender: &mpsc::UnboundedSender<RightData>,
    name: &str,
) -> bool {
    // It's totally fine if we are unable to send a message on left_sender or right_sender.
    // The other half of the channel can be dropped for any reason,
    // but it's not our responsibility to react for it here.
    match receiver.lock().await.next().await {
        Some(Split::Left(data)) => {
            if left_sender.unbounded_send(data).is_err() {
                debug!(target: "aleph-network", "Unable to send to LeftNetwork ({}) - already disabled", name);
            }
            true
        }
        Some(Split::Right(data)) => {
            if right_sender.unbounded_send(data).is_err() {
                debug!(target: "aleph-network", "Unable to send to RightNetwork ({}) - already disabled", name);
            }
            true
        }
        None => {
            trace!(target: "aleph-network", "Split data channel ended");
            left_sender.close_channel();
            right_sender.close_channel();
            false
        }
    }
}

fn split_sender<LeftData: Data, RightData: Data, S: Sender<Split<LeftData, RightData>>>(
    sender: S,
) -> (
    LeftSender<LeftData, RightData, S>,
    RightSender<LeftData, RightData, S>,
) {
    (
        LeftSender {
            sender: sender.clone(),
            phantom: PhantomData,
        },
        RightSender {
            sender,
            phantom: PhantomData,
        },
    )
}

fn split_receiver<LeftData: Data, RightData: Data, R: Receiver<Split<LeftData, RightData>>>(
    receiver: R,
    left_name: &'static str,
    right_name: &'static str,
) -> (
    LeftReceiver<LeftData, RightData, R>,
    RightReceiver<LeftData, RightData, R>,
) {
    let receiver = Arc::new(Mutex::new(receiver));
    let (left_sender, left_receiver) = mpsc::unbounded();
    let (right_sender, right_receiver) = mpsc::unbounded();
    (
        LeftReceiver {
            receiver: receiver.clone(),
            translated_receiver: left_receiver,
            left_sender: left_sender.clone(),
            right_sender: right_sender.clone(),
            name: left_name,
        },
        RightReceiver {
            receiver,
            translated_receiver: right_receiver,
            left_sender,
            right_sender,
            name: right_name,
        },
    )
}

/// Split a single component network into two separate ones. This way multiple components can send
/// data to the same underlying session not knowing what types of data the other ones use.
///
/// Internally the returned networks compete for data returned by their parent network when
/// `next()` is polled, and unpack it to two separate channels. At the same time each polls
/// the end of those channels which contains the type that it is supposed to return.
///
/// The main example for now is creating an `aleph_bft::Network` and a separate one for accumulating
/// signatures for justifications.
pub fn split<LeftData: Data, RightData: Data, CN: Network<Split<LeftData, RightData>>>(
    network: CN,
    left_name: &'static str,
    right_name: &'static str,
) -> (impl NetworkExt<LeftData>, impl NetworkExt<RightData>) {
    let (sender, receiver) = network.into();
    let (left_sender, right_sender) = split_sender(sender);
    let (left_receiver, right_receiver) = split_receiver(receiver, left_name, right_name);
    (
        SimpleNetwork::new(left_receiver, left_sender),
        SimpleNetwork::new(right_receiver, right_sender),
    )
}
