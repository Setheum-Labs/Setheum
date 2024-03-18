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

use std::{fs, path::PathBuf};

use setheum_client::{pallets::vk_storage::VkStorageUserApi, SignedConnection, TxStatus};
use anyhow::Result;

fn read_bytes(file: &PathBuf) -> Result<Vec<u8>> {
    fs::read(file).map_err(|e| e.into())
}

/// Calls `module_vk_storage::store_key`.
pub async fn store_key(connection: SignedConnection, vk_file: PathBuf) -> Result<()> {
    let vk = read_bytes(&vk_file)?;
    connection
        .store_key(vk, TxStatus::Finalized)
        .await
        .map(|_| ())
}
