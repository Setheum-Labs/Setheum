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

use crate::{
    block::{Block, UnverifiedHeader},
    data_io::{AlephData, UnvalidatedAlephProposal},
    testing::mocks::{TBlock, THeader},
};

pub fn unvalidated_proposal_from_headers(
    mut headers: Vec<THeader>,
) -> UnvalidatedAlephProposal<THeader> {
    let head = headers.pop().unwrap();
    let tail = headers
        .into_iter()
        .map(|header| header.id().hash())
        .collect();
    UnvalidatedAlephProposal::new(head, tail)
}

pub fn aleph_data_from_blocks(blocks: Vec<TBlock>) -> AlephData<THeader> {
    let headers = blocks.into_iter().map(|b| b.header().clone()).collect();
    aleph_data_from_headers(headers)
}

pub fn aleph_data_from_headers(headers: Vec<THeader>) -> AlephData<THeader> {
    AlephData {
        head_proposal: unvalidated_proposal_from_headers(headers),
    }
}
