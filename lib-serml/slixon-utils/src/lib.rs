// This file is part of Setheum.

// Copyright (C) 2019-2021 Setheum Labs.
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

#![cfg_attr(not(feature = "std"), no_std)]

use codec::{Decode, Encode};
use frame_support::{
    decl_error, decl_module, decl_storage, decl_event,
    dispatch::{DispatchError, DispatchResult}, ensure,
    traits::{
        Currency, Get,
        Imbalance, OnUnbalanced,
    },
};
use frame_system as system;

#[cfg(feature = "std")]
use serde::Deserialize;
use sp_runtime::RuntimeDebug;
use sp_std::{
    collections::btree_set::BTreeSet,
    prelude::*,
};

#[cfg(test)]
mod mock;
pub mod mock_functions;

#[cfg(test)]
mod tests;

pub mod rpc;

pub type ChannelId = u64;
pub type PostId = u64;

#[derive(Encode, Decode, Clone, Eq, PartialEq, RuntimeDebug)]
pub struct WhoAndWhen<T: Trait> {
    pub account: T::AccountId,
    pub block: T::BlockNumber,
    pub time: T::Moment,
}

impl<T: Trait> WhoAndWhen<T> {
    pub fn new(account: T::AccountId) -> Self {
        WhoAndWhen {
            account,
            block: <system::Module<T>>::block_number(),
            time: <pallet_timestamp::Module<T>>::now(),
        }
    }
}

#[derive(Encode, Decode, Ord, PartialOrd, Clone, Eq, PartialEq, RuntimeDebug)]
pub enum User<AccountId> {
    Account(AccountId),
    Channel(ChannelId),
}

impl<AccountId> User<AccountId> {
    pub fn maybe_account(self) -> Option<AccountId> {
        if let User::Account(account_id) = self {
            Some(account_id)
        } else {
            None
        }
    }

    pub fn maybe_channel(self) -> Option<ChannelId> {
        if let User::Channel(channel_id) = self {
            Some(channel_id)
        } else {
            None
        }
    }
}

#[derive(Encode, Decode, Clone, Eq, PartialEq, RuntimeDebug)]
#[cfg_attr(feature = "std", derive(Deserialize))]
#[cfg_attr(feature = "std", serde(tag = "contentType", content = "contentId"))]
pub enum Content {
    /// No content.
    None,
    /// A raw vector of bytes.
    Raw(Vec<u8>),
    /// IPFS CID v0 of content.
    #[allow(clippy::upper_case_acronyms)]
    IPFS(Vec<u8>),
    /// Hypercore protocol (former DAT) id of content.
    Hyper(Vec<u8>),
}

impl From<Content> for Vec<u8> {
    fn from(content: Content) -> Vec<u8> {
        match content {
            Content::None => vec![],
            Content::Raw(vec_u8) => vec_u8,
            Content::IPFS(vec_u8) => vec_u8,
            Content::Hyper(vec_u8) => vec_u8,
        }
    }
}

impl Default for Content {
    fn default() -> Self {
        Self::None
    }
}

impl Content {
    pub fn is_none(&self) -> bool {
        self == &Self::None
    }

    pub fn is_some(&self) -> bool {
        !self.is_none()
    }

    pub fn is_ipfs(&self) -> bool {
        matches!(self, Self::IPFS(_))
    }
}

type BalanceOf<T> = <<T as Trait>::Currency as Currency<<T as system::Trait>::AccountId>>::Balance;

type NegativeImbalanceOf<T> = <<T as Trait>::Currency as Currency<<T as frame_system::Trait>::AccountId>>::NegativeImbalance;

pub trait Trait: system::Trait + pallet_timestamp::Trait
{
    /// The overarching event type.
    type Event: From<Event<Self>> + Into<<Self as system::Trait>::Event>;

    /// The currency mechanism.
    type Currency: Currency<Self::AccountId>;

    /// Minimal length of channel/profile handle
    type MinHandleLen: Get<u32>;

    /// Max length of a channel handle.
    type MaxHandleLen: Get<u32>;
}

decl_storage! {
    trait Store for Module<T: Trait> as UtilsModule {
        pub TreasuryAccount get(fn treasury_account) build(|config| config.treasury_account.clone()): T::AccountId;
    }
    add_extra_genesis {
        config(treasury_account): T::AccountId;
        build(|config| {
			// Create Treasury account
			let _ = T::Currency::make_free_balance_be(
				&config.treasury_account,
				T::Currency::minimum_balance(),
			);
		});
    }
}

decl_module! {
    pub struct Module<T: Trait> for enum Call where origin: T::Origin {

        const MinHandleLen: u32 = T::MinHandleLen::get();

        const MaxHandleLen: u32 = T::MaxHandleLen::get();

        // Initializing errors
        type Error = Error<T>;

        // Initializing events
        fn deposit_event() = default;
    }
}

decl_error! {
    pub enum Error for Module<T: Trait> {
        /// Account is blocked in a given channel.
        AccountIsBlocked,
        /// Content is blocked in a given channel.
        ContentIsBlocked,
        /// Post is blocked in a given channel.
        PostIsBlocked,
        /// IPFS CID is invalid.
        InvalidIpfsCid,
        /// `Raw` content type is not yet supported.
        RawContentTypeNotSupported,
        /// `Hyper` content type is not yet supported.
        HypercoreContentTypeNotSupported,
        /// Channel handle is too short.
        HandleIsTooShort,
        /// Channel handle is too long.
        HandleIsTooLong,
        /// Channel handle contains invalid characters.
        HandleContainsInvalidChars,
        /// Content type is `None`.
        ContentIsEmpty,
    }
}

decl_event!(
    pub enum Event<T> where Balance = BalanceOf<T>
    {
		Deposit(Balance),
    }
);

fn num_bits<P>() -> usize {
    sp_std::mem::size_of::<P>() * 8
}

/// Returns `None` for `x == 0`.
pub fn log_2(x: u32) -> Option<u32> {
    if x > 0 {
        Some(
            num_bits::<u32>() as u32
                - x.leading_zeros()
                - 1
        )
    } else { None }
}

pub fn remove_from_vec<F: PartialEq>(vector: &mut Vec<F>, element: F) {
    if let Some(index) = vector.iter().position(|x| *x == element) {
        vector.swap_remove(index);
    }
}

pub fn bool_to_option(value: bool) -> Option<bool> {
    if value { Some(value) } else { None }
}

impl<T: Trait> Module<T> {
    pub fn is_valid_content(content: Content) -> DispatchResult {
        match content {
            Content::None => Ok(()),
            Content::Raw(_) => Err(Error::<T>::RawContentTypeNotSupported.into()),
            Content::IPFS(ipfs_cid) => {
                let len = ipfs_cid.len();
                // IPFS CID v0 is 46 bytes.
                // IPFS CID v1 is 59 bytes.df-integration-tests/src/lib.rs:272:5
                ensure!(len == 46 || len == 59, Error::<T>::InvalidIpfsCid);
                Ok(())
            },
            Content::Hyper(_) => Err(Error::<T>::HypercoreContentTypeNotSupported.into())
        }
    }

    pub fn convert_users_vec_to_btree_set(
        users_vec: Vec<User<T::AccountId>>
    ) -> Result<BTreeSet<User<T::AccountId>>, DispatchError> {
        let mut users_set: BTreeSet<User<T::AccountId>> = BTreeSet::new();

        for user in users_vec.iter() {
            users_set.insert(user.clone());
        }

        Ok(users_set)
    }

    /// Check if a handle contains only valid chars: 0-9, a-z, _.
    /// An example of a valid handle: `good_handle_123`.
    fn is_valid_handle_char(c: u8) -> bool {
        matches!(c, b'0'..=b'9' | b'a'..=b'z' | b'_')
    }

    /// Lowercase a handle.
    pub fn lowercase_handle(handle: Vec<u8>) -> Vec<u8> {
        handle.to_ascii_lowercase()
    }

    /// This function does the next:
    /// - Check if a handle length fits into min/max length constraints.
    /// - Lowercase a handle.
    /// - Check if a handle contains only valid chars: 0-9, a-z, _.
    pub fn lowercase_and_validate_a_handle(handle: Vec<u8>) -> Result<Vec<u8>, DispatchError> {

        // Check if a handle length fits into min/max length constraints:
        ensure!(handle.len() >= T::MinHandleLen::get() as usize, Error::<T>::HandleIsTooShort);
        ensure!(handle.len() <= T::MaxHandleLen::get() as usize, Error::<T>::HandleIsTooLong);

        let handle_in_lowercase = Self::lowercase_handle(handle);

        // Check if a handle contains only valid chars: 0-9, a-z, _.
        let is_only_valid_chars = handle_in_lowercase.iter().all(|&x| Self::is_valid_handle_char(x));
        ensure!(is_only_valid_chars, Error::<T>::HandleContainsInvalidChars);

        // Return a lower-cased version of a handle.
        Ok(handle_in_lowercase)
    }

    /// Ensure that a given content is not `None`.
    pub fn ensure_content_is_some(content: &Content) -> DispatchResult {
        ensure!(content.is_some(), Error::<T>::ContentIsEmpty);
        Ok(())
    }
}

impl<T: Trait> OnUnbalanced<NegativeImbalanceOf<T>> for Module<T> {
    fn on_nonzero_unbalanced(amount: NegativeImbalanceOf<T>) {
        let numeric_amount = amount.peek();
        let treasury_account = TreasuryAccount::<T>::get();

        // Must resolve into existing but better to be safe.
        let _ = T::Currency::resolve_creating(&treasury_account, amount);

        Self::deposit_event(RawEvent::Deposit(numeric_amount));
    }
}
