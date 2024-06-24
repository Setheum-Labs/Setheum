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

#![cfg_attr(not(feature = "std"), no_std, no_main)]

#[ink::contract(env = senterprise_extension::Environment)]
mod test_contract {
    use ink::prelude::vec;

    #[ink(storage)]
    pub struct TestContract {}

    impl TestContract {
        #[ink(constructor)]
        pub fn new() -> Self {
            Self {}
        }

        #[ink(message)]
        pub fn call_verify(&self) {
            self.env()
                .extension()
                .verify(Default::default(), vec![0; 41], vec![0; 82])
                .unwrap();
        }
    }

    #[cfg(test)]
    mod tests {
        use ink::env::test::register_chain_extension;

        use super::*;

        struct MockedVerifyExtension;
        impl ink::env::test::ChainExtension for MockedVerifyExtension {
            fn ext_id(&self) -> u16 {
                senterprise_extension::extension_ids::EXTENSION_ID
            }

            fn call(&mut self, func_id: u16, _: &[u8], _: &mut Vec<u8>) -> u32 {
                assert_eq!(
                    func_id,
                    senterprise_extension::extension_ids::VERIFY_FUNC_ID
                );
                senterprise_extension::status_codes::VERIFY_SUCCESS
            }
        }

        #[ink::test]
        fn verify_works() {
            register_chain_extension(MockedVerifyExtension);
            TestContract::new().call_verify();
        }
    }
}
