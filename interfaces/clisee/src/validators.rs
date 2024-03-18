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

use setheum_client::{
    pallets::elections::ElectionsSudoApi, primitives::CommitteeSeats, RootConnection, TxStatus,
};

use crate::commands::ChangeValidatorArgs;

/// Change validators to the provided list by calling the provided node.
pub async fn change_validators(
    root_connection: RootConnection,
    change_validator_args: ChangeValidatorArgs,
) {
    root_connection
        .change_validators(
            change_validator_args.reserved_validators,
            change_validator_args.non_reserved_validators,
            change_validator_args
                .committee_size
                .map(|s| CommitteeSeats {
                    reserved_seats: s.reserved_seats,
                    non_reserved_seats: s.non_reserved_seats,
                    non_reserved_finality_seats: s.non_reserved_finality_seats,
                }),
            TxStatus::Finalized,
        )
        .await
        .unwrap();
    // TODO we need to check state here whether change members actually succeed
    // not only here, but for all clisee commands
    // see https://cardinal-cryptography.atlassian.net/browse/AZ-699
}
