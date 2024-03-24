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

use parity_scale_codec::{Decode, Encode, Error, Input, Output};

use crate::{network::Data, SessionId};

/// Data inside session, sent to validator network.
/// Wrapper for data send over network. We need it to ensure compatibility.
/// The order of the data and session_id is fixed in encode and the decode expects it to be data, session_id.
/// Since data is versioned, i.e. it's encoding starts with a version number in the standardized way,
/// this will allow us to retrofit versioning here if we ever need to change this structure.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DataInSession<D: Data> {
    pub data: D,
    pub session_id: SessionId,
}

impl<D: Data> Decode for DataInSession<D> {
    fn decode<I: Input>(input: &mut I) -> Result<Self, Error> {
        let data = D::decode(input)?;
        let session_id = SessionId::decode(input)?;

        Ok(Self { data, session_id })
    }
}

impl<D: Data> Encode for DataInSession<D> {
    fn size_hint(&self) -> usize {
        self.data.size_hint() + self.session_id.size_hint()
    }

    fn encode_to<T: Output + ?Sized>(&self, dest: &mut T) {
        self.data.encode_to(dest);
        self.session_id.encode_to(dest);
    }
}
