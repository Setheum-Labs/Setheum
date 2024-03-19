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

use futures::channel::oneshot;
use log::{debug, trace, warn};

use crate::{
    party::{Handle, Task as PureTask},
    NodeIndex,
};

/// A wrapper for running the authority task within a specific session.
pub struct Task {
    task: PureTask,
    node_id: NodeIndex,
}

impl Task {
    /// Create a new authority task. The handle should be the handle to the actual task.
    pub fn new(handle: Handle, node_id: NodeIndex, exit: oneshot::Sender<()>) -> Self {
        Task {
            task: PureTask::new(handle, exit),
            node_id,
        }
    }

    /// Stop the authority task and wait for it to finish.
    pub async fn stop(self) -> Result<(), ()> {
        self.task.stop().await
    }

    /// If the authority task stops for any reason, this returns the associated NodeIndex, which
    /// can be used to restart the task.
    pub async fn stopped(&mut self) -> NodeIndex {
        if self.task.stopped().await.is_err() {
            debug!(target: "aleph-party", "Authority task failed for {:?}", self.node_id);
        }
        self.node_id
    }
}

/// All the subtasks required to participate in a session as an authority.
pub struct Subtasks {
    exit: oneshot::Receiver<()>,
    member: PureTask,
    aggregator: PureTask,
    refresher: PureTask,
    data_store: PureTask,
}

impl Subtasks {
    /// Create the subtask collection by passing in all the tasks.
    pub fn new(
        exit: oneshot::Receiver<()>,
        member: PureTask,
        aggregator: PureTask,
        refresher: PureTask,
        data_store: PureTask,
    ) -> Self {
        Subtasks {
            exit,
            member,
            aggregator,
            refresher,
            data_store,
        }
    }

    async fn stop(self) -> Result<(), ()> {
        // both member and aggregator are implicitly using forwarder,
        // so we should force them to exit first to avoid any panics, i.e. `send on closed channel`
        debug!(target: "aleph-party", "Started to stop all tasks");
        let mut result = Ok(());
        if self.member.stop().await.is_err() {
            warn!(target: "aleph-party", "Member stopped with en error");
            result = Err(());
        }
        trace!(target: "aleph-party", "Member stopped");
        if self.aggregator.stop().await.is_err() {
            warn!(target: "aleph-party", "Aggregator stopped with en error");
            result = Err(());
        }
        trace!(target: "aleph-party", "Aggregator stopped");
        if self.refresher.stop().await.is_err() {
            warn!(target: "aleph-party", "Refresher stopped with en error");
            result = Err(());
        }
        trace!(target: "aleph-party", "Refresher stopped");
        if self.data_store.stop().await.is_err() {
            warn!(target: "aleph-party", "DataStore stopped with en error");
            result = Err(());
        }
        trace!(target: "aleph-party", "DataStore stopped");
        result
    }

    /// Blocks until the task is done and returns Err if it quit unexpectedly.
    pub async fn wait_completion(mut self) -> Result<(), ()> {
        let result = tokio::select! {
            _ = &mut self.exit => Ok(()),
            res = self.member.stopped() => { debug!(target: "aleph-party", "Member stopped early"); res },
            res = self.aggregator.stopped() => { debug!(target: "aleph-party", "Aggregator stopped early"); res },
            res = self.refresher.stopped() => { debug!(target: "aleph-party", "Refresher stopped early"); res },
            res = self.data_store.stopped() => { debug!(target: "aleph-party", "DataStore stopped early"); res },
        };
        let stop_result = self.stop().await;
        debug!(target: "aleph-party", "Stopped all processes");
        result.and(stop_result)
    }
}
