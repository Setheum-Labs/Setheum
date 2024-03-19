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

use std::{
    sync::{Arc, Mutex},
    time::Duration,
};

use futures::{
    channel::mpsc::{unbounded, UnboundedReceiver, UnboundedSender},
    StreamExt,
};
use tokio::time::timeout;

#[derive(Clone)]
pub(crate) struct SingleActionMock<CallArgs: Send> {
    timeout: Duration,
    history_tx: Arc<Mutex<UnboundedSender<CallArgs>>>,
    history_rx: Arc<Mutex<UnboundedReceiver<CallArgs>>>,
}

unsafe impl<CallArgs: Send> Send for SingleActionMock<CallArgs> {}

impl<CallArgs: Send> SingleActionMock<CallArgs> {
    pub(crate) fn new(timeout: Duration) -> Self {
        let (history_tx, history_rx) = unbounded();
        Self {
            timeout,
            history_tx: Arc::new(Mutex::new(history_tx)),
            history_rx: Arc::new(Mutex::new(history_rx)),
        }
    }

    pub(crate) fn invoke_with(&self, args: CallArgs) {
        self.history_tx
            .lock()
            .unwrap()
            .unbounded_send(args)
            .unwrap()
    }

    //This code is used only for testing.
    #[allow(clippy::await_holding_lock)]
    pub(crate) async fn has_not_been_invoked(&self) -> bool {
        timeout(self.timeout, self.history_rx.lock().unwrap().next())
            .await
            .is_err()
    }

    //This code is used only for testing.
    #[allow(clippy::await_holding_lock)]
    #[allow(clippy::significant_drop_in_scrutinee)]
    pub(crate) async fn has_been_invoked_with<P: FnOnce(CallArgs) -> bool>(
        &self,
        predicate: P,
    ) -> bool {
        match timeout(self.timeout, self.history_rx.lock().unwrap().next()).await {
            Ok(Some(args)) => predicate(args),
            _ => false,
        }
    }
}

const DEFAULT_TIMEOUT: Duration = Duration::from_millis(50);

impl<CallArgs: Send> Default for SingleActionMock<CallArgs> {
    fn default() -> Self {
        SingleActionMock::new(DEFAULT_TIMEOUT)
    }
}
