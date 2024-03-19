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

//! Status codes for the black-box-extension pallet.
//!
//! Every extension function (like `verify`) comes with:
//! * its own success code,
//! * and a set of error codes (usually starting at the success code + 1).

#![allow(missing_docs)] // Error constants are self-descriptive.

// ---- `verify` errors ----------------------------------------------------------------------------
const VERIFY_BASE: u32 = 11_000;
pub const VERIFY_SUCCESS: u32 = VERIFY_BASE;
pub const VERIFY_DESERIALIZING_INPUT_FAIL: u32 = VERIFY_BASE + 1;
pub const VERIFY_UNKNOWN_IDENTIFIER: u32 = VERIFY_BASE + 2;
pub const VERIFY_DESERIALIZING_KEY_FAIL: u32 = VERIFY_BASE + 3;
pub const VERIFY_VERIFICATION_FAIL: u32 = VERIFY_BASE + 4;
pub const VERIFY_INCORRECT_PROOF: u32 = VERIFY_BASE + 5;
