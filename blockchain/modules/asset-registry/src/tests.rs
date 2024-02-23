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

//! Unit tests for asset registry module.

#![cfg(test)]

use super::*;
use frame_support::{assert_noop, assert_ok};
use mock::{
	alice, deploy_contracts, deploy_contracts_same_prefix, erc20_address, erc20_address_not_exists,
	erc20_address_same_prefix, AssetRegistry, CouncilAccount, ExtBuilder, Runtime, RuntimeEvent, RuntimeOrigin, System,
};
use primitives::TokenSymbol;
use sp_core::H160;
use std::str::{from_utf8, FromStr};

#[test]
fn key_to_currency_work() {
	let erc20 = CurrencyId::Erc20(EvmAddress::from_str("0x5dddfce53ee040d9eb21afbc0ae1bb4dbb0ba644").unwrap());
	let v2_location = xcm::v2::MultiLocation::new(
		0,
		xcm::v2::Junctions::X1(xcm::v2::Junction::GeneralKey(erc20.encode().try_into().unwrap())),
	);
	let v3_location_from_v2 = MultiLocation::try_from(v2_location.clone()).unwrap();
	let v3_location = MultiLocation::new(
		0,
		Junctions::X1(Junction::from(BoundedVec::try_from(erc20.encode()).unwrap())),
	);
	assert_eq!(v3_location_from_v2, v3_location);
	assert_eq!(crate::key_to_currency(v3_location), Some(erc20));
}

#[test]
fn test_v2_to_v3_incompatible_multilocation() {
	let v2_location = xcm::v2::MultiLocation::new(
		0,
		xcm::v2::Junctions::X1(xcm::v2::Junction::GeneralKey(vec![0].try_into().unwrap())),
	);

	let v3_location = MultiLocation::new(0, X1(Junction::from(BoundedVec::try_from(vec![0]).unwrap())));

	// Assert that V2 and V3 Multilocation both are encoded differently
	assert!(v2_location.encode() != v3_location.encode());
}

#[test]
fn versioned_multi_location_convert_work() {
	ExtBuilder::default().build().execute_with(|| {
		// v2
		let v2_location = VersionedMultiLocation::V2(xcm::v2::MultiLocation {
			parents: 0,
			interior: xcm::v2::Junctions::X1(xcm::v2::Junction::Parachain(1000)),
		});
		let location: MultiLocation = v2_location.try_into().unwrap();
		assert_eq!(
			location,
			MultiLocation {
				parents: 0,
				interior: xcm::v3::Junctions::X1(xcm::v3::Junction::Parachain(1000))
			}
		);

		// v3
		let v3_location = VersionedMultiLocation::V3(MultiLocation {
			parents: 0,
			interior: xcm::v3::Junctions::X1(xcm::v3::Junction::Parachain(1000)),
		});
		let location: MultiLocation = v3_location.try_into().unwrap();
		assert_eq!(
			location,
			MultiLocation {
				parents: 0,
				interior: xcm::v3::Junctions::X1(xcm::v3::Junction::Parachain(1000))
			}
		);

		// handle all of VersionedMultiLocation
		assert!(match location.into() {
			VersionedMultiLocation::V2 { .. } | VersionedMultiLocation::V3 { .. } => true,
		});
	});
}

#[test]
fn register_foreign_asset_work() {
	ExtBuilder::default().build().execute_with(|| {
		// v2
		let v2_location = VersionedMultiLocation::V2(xcm::v2::MultiLocation {
			parents: 0,
			interior: xcm::v2::Junctions::X1(xcm::v2::Junction::Parachain(1000)),
		});

		assert_ok!(AssetRegistry::register_foreign_asset(
			RuntimeOrigin::signed(CouncilAccount::get()),
			Box::new(v2_location.clone()),
			Box::new(AssetMetadata {
				name: b"Token Name".to_vec(),
				symbol: b"TN".to_vec(),
				decimals: 12,
				minimal_balance: 1,
			})
		));

		let location: MultiLocation = v2_location.try_into().unwrap();
		System::assert_last_event(RuntimeEvent::AssetRegistry(crate::Event::ForeignAssetRegistered {
			asset_id: 0,
			asset_address: location.clone(),
			metadata: AssetMetadata {
				name: b"Token Name".to_vec(),
				symbol: b"TN".to_vec(),
				decimals: 12,
				minimal_balance: 1,
			},
		}));

		assert_eq!(ForeignAssetLocations::<Runtime>::get(0), Some(location.clone()));
		assert_eq!(
			AssetMetadatas::<Runtime>::get(AssetIds::ForeignAssetId(0)),
			Some(AssetMetadata {
				name: b"Token Name".to_vec(),
				symbol: b"TN".to_vec(),
				decimals: 12,
				minimal_balance: 1,
			})
		);
		assert_eq!(
			LocationToCurrencyIds::<Runtime>::get(location),
			Some(CurrencyId::ForeignAsset(0))
		);

		// v3
		let v3_location = VersionedMultiLocation::V3(xcm::v3::MultiLocation {
			parents: 0,
			interior: xcm::v3::Junctions::X1(xcm::v3::Junction::GeneralKey {
				length: 32,
				data: [0u8; 32],
			}),
		});

		assert_ok!(AssetRegistry::register_foreign_asset(
			RuntimeOrigin::signed(CouncilAccount::get()),
			Box::new(v3_location.clone()),
			Box::new(AssetMetadata {
				name: b"Another Token Name".to_vec(),
				symbol: b"ATN".to_vec(),
				decimals: 12,
				minimal_balance: 1,
			})
		));

		let location: MultiLocation = v3_location.try_into().unwrap();
		System::assert_last_event(RuntimeEvent::AssetRegistry(crate::Event::ForeignAssetRegistered {
			asset_id: 1,
			asset_address: location.clone(),
			metadata: AssetMetadata {
				name: b"Another Token Name".to_vec(),
				symbol: b"ATN".to_vec(),
				decimals: 12,
				minimal_balance: 1,
			},
		}));

		assert_eq!(ForeignAssetLocations::<Runtime>::get(1), Some(location.clone()));
		assert_eq!(
			AssetMetadatas::<Runtime>::get(AssetIds::ForeignAssetId(1)),
			Some(AssetMetadata {
				name: b"Another Token Name".to_vec(),
				symbol: b"ATN".to_vec(),
				decimals: 12,
				minimal_balance: 1,
			})
		);
		assert_eq!(
			LocationToCurrencyIds::<Runtime>::get(location),
			Some(CurrencyId::ForeignAsset(1))
		);
	});
}

#[test]
fn register_foreign_asset_should_not_work() {
	ExtBuilder::default().build().execute_with(|| {
		let v3_location = VersionedMultiLocation::V3(xcm::v3::MultiLocation {
			parents: 0,
			interior: xcm::v3::Junctions::X1(xcm::v3::Junction::Parachain(1000)),
		});

		assert_ok!(AssetRegistry::register_foreign_asset(
			RuntimeOrigin::signed(CouncilAccount::get()),
			Box::new(v3_location.clone()),
			Box::new(AssetMetadata {
				name: b"Token Name".to_vec(),
				symbol: b"TN".to_vec(),
				decimals: 12,
				minimal_balance: 1,
			})
		));

		assert_noop!(
			AssetRegistry::register_foreign_asset(
				RuntimeOrigin::signed(CouncilAccount::get()),
				Box::new(v3_location.clone()),
				Box::new(AssetMetadata {
					name: b"Token Name".to_vec(),
					symbol: b"TN".to_vec(),
					decimals: 12,
					minimal_balance: 1,
				})
			),
			Error::<Runtime>::MultiLocationExisted
		);

		NextForeignAssetId::<Runtime>::set(u16::MAX);
		assert_noop!(
			AssetRegistry::register_foreign_asset(
				RuntimeOrigin::signed(CouncilAccount::get()),
				Box::new(v3_location),
				Box::new(AssetMetadata {
					name: b"Token Name".to_vec(),
					symbol: b"TN".to_vec(),
					decimals: 12,
					minimal_balance: 1,
				})
			),
			ArithmeticError::Overflow
		);
	});
}

#[test]
fn update_foreign_asset_work() {
	ExtBuilder::default().build().execute_with(|| {
		let v2_location = VersionedMultiLocation::V2(xcm::v2::MultiLocation {
			parents: 0,
			interior: xcm::v2::Junctions::X1(xcm::v2::Junction::Parachain(1000)),
		});

		assert_ok!(AssetRegistry::register_foreign_asset(
			RuntimeOrigin::signed(CouncilAccount::get()),
			Box::new(v2_location.clone()),
			Box::new(AssetMetadata {
				name: b"Token Name".to_vec(),
				symbol: b"TN".to_vec(),
				decimals: 12,
				minimal_balance: 1,
			})
		));

		assert_ok!(AssetRegistry::update_foreign_asset(
			RuntimeOrigin::signed(CouncilAccount::get()),
			0,
			Box::new(v2_location.clone()),
			Box::new(AssetMetadata {
				name: b"New Token Name".to_vec(),
				symbol: b"NTN".to_vec(),
				decimals: 12,
				minimal_balance: 2,
			})
		));

		let location: MultiLocation = v2_location.try_into().unwrap();
		System::assert_last_event(RuntimeEvent::AssetRegistry(crate::Event::ForeignAssetUpdated {
			asset_id: 0,
			asset_address: location.clone(),
			metadata: AssetMetadata {
				name: b"New Token Name".to_vec(),
				symbol: b"NTN".to_vec(),
				decimals: 12,
				minimal_balance: 2,
			},
		}));

		assert_eq!(
			AssetMetadatas::<Runtime>::get(AssetIds::ForeignAssetId(0)),
			Some(AssetMetadata {
				name: b"New Token Name".to_vec(),
				symbol: b"NTN".to_vec(),
				decimals: 12,
				minimal_balance: 2,
			})
		);
		assert_eq!(ForeignAssetLocations::<Runtime>::get(0), Some(location.clone()));
		assert_eq!(
			LocationToCurrencyIds::<Runtime>::get(location.clone()),
			Some(CurrencyId::ForeignAsset(0))
		);

		// modify location
		let new_location = VersionedMultiLocation::V2(xcm::v2::MultiLocation {
			parents: 0,
			interior: xcm::v2::Junctions::X1(xcm::v2::Junction::Parachain(2000)),
		});

		assert_ok!(AssetRegistry::update_foreign_asset(
			RuntimeOrigin::signed(CouncilAccount::get()),
			0,
			Box::new(new_location.clone()),
			Box::new(AssetMetadata {
				name: b"New Token Name".to_vec(),
				symbol: b"NTN".to_vec(),
				decimals: 12,
				minimal_balance: 2,
			})
		));
		assert_eq!(
			AssetMetadatas::<Runtime>::get(AssetIds::ForeignAssetId(0)),
			Some(AssetMetadata {
				name: b"New Token Name".to_vec(),
				symbol: b"NTN".to_vec(),
				decimals: 12,
				minimal_balance: 2,
			})
		);
		let new_location: MultiLocation = new_location.try_into().unwrap();
		assert_eq!(ForeignAssetLocations::<Runtime>::get(0), Some(new_location.clone()));
		assert_eq!(LocationToCurrencyIds::<Runtime>::get(location), None);
		assert_eq!(
			LocationToCurrencyIds::<Runtime>::get(new_location),
			Some(CurrencyId::ForeignAsset(0))
		);
	});
}

#[test]
fn update_foreign_asset_should_not_work() {
	ExtBuilder::default().build().execute_with(|| {
		let v2_location = VersionedMultiLocation::V2(xcm::v2::MultiLocation {
			parents: 0,
			interior: xcm::v2::Junctions::X1(xcm::v2::Junction::Parachain(1000)),
		});

		assert_noop!(
			AssetRegistry::update_foreign_asset(
				RuntimeOrigin::signed(CouncilAccount::get()),
				0,
				Box::new(v2_location.clone()),
				Box::new(AssetMetadata {
					name: b"New Token Name".to_vec(),
					symbol: b"NTN".to_vec(),
					decimals: 12,
					minimal_balance: 2,
				})
			),
			Error::<Runtime>::AssetIdNotExists
		);

		assert_ok!(AssetRegistry::register_foreign_asset(
			RuntimeOrigin::signed(CouncilAccount::get()),
			Box::new(v2_location.clone()),
			Box::new(AssetMetadata {
				name: b"Token Name".to_vec(),
				symbol: b"TN".to_vec(),
				decimals: 12,
				minimal_balance: 1,
			})
		));

		assert_ok!(AssetRegistry::update_foreign_asset(
			RuntimeOrigin::signed(CouncilAccount::get()),
			0,
			Box::new(v2_location),
			Box::new(AssetMetadata {
				name: b"New Token Name".to_vec(),
				symbol: b"NTN".to_vec(),
				decimals: 12,
				minimal_balance: 2,
			})
		));

		// existed location
		let new_location = VersionedMultiLocation::V2(xcm::v2::MultiLocation {
			parents: 0,
			interior: xcm::v2::Junctions::X1(xcm::v2::Junction::Parachain(2000)),
		});
		assert_ok!(AssetRegistry::register_foreign_asset(
			RuntimeOrigin::signed(CouncilAccount::get()),
			Box::new(new_location.clone()),
			Box::new(AssetMetadata {
				name: b"Token Name".to_vec(),
				symbol: b"TN".to_vec(),
				decimals: 12,
				minimal_balance: 1,
			})
		));
		assert_noop!(
			AssetRegistry::update_foreign_asset(
				RuntimeOrigin::signed(CouncilAccount::get()),
				0,
				Box::new(new_location),
				Box::new(AssetMetadata {
					name: b"New Token Name".to_vec(),
					symbol: b"NTN".to_vec(),
					decimals: 12,
					minimal_balance: 2,
				})
			),
			Error::<Runtime>::MultiLocationExisted
		);
	});
}

#[test]
fn register_erc20_asset_work() {
	ExtBuilder::default()
		.balances(vec![(alice(), 1_000_000_000_000)])
		.build()
		.execute_with(|| {
			deploy_contracts();
			assert_ok!(AssetRegistry::register_erc20_asset(
				RuntimeOrigin::signed(CouncilAccount::get()),
				erc20_address(),
				1
			));

			System::assert_last_event(RuntimeEvent::AssetRegistry(crate::Event::AssetRegistered {
				asset_id: AssetIds::Erc20(erc20_address()),
				metadata: AssetMetadata {
					name: b"long string name, long string name, long string name, long string name, long string name"
						.to_vec(),
					symbol: b"TestToken".to_vec(),
					decimals: 17,
					minimal_balance: 1,
				},
			}));

			assert_eq!(Erc20IdToAddress::<Runtime>::get(0x5dddfce5), Some(erc20_address()));

			assert_eq!(
				AssetMetadatas::<Runtime>::get(AssetIds::Erc20(erc20_address())),
				Some(AssetMetadata {
					name: b"long string name, long string name, long string name, long string name, long string name"
						.to_vec(),
					symbol: b"TestToken".to_vec(),
					decimals: 17,
					minimal_balance: 1,
				})
			);
		});
}

#[test]
fn register_erc20_asset_should_not_work() {
	ExtBuilder::default()
		.balances(vec![(alice(), 1_000_000_000_000)])
		.build()
		.execute_with(|| {
			deploy_contracts();
			deploy_contracts_same_prefix();
			assert_ok!(AssetRegistry::register_erc20_asset(
				RuntimeOrigin::signed(CouncilAccount::get()),
				erc20_address(),
				1
			));

			assert_noop!(
				AssetRegistry::register_erc20_asset(
					RuntimeOrigin::signed(CouncilAccount::get()),
					erc20_address_same_prefix(),
					1
				),
				Error::<Runtime>::AssetIdExisted
			);

			assert_noop!(
				AssetRegistry::register_erc20_asset(
					RuntimeOrigin::signed(CouncilAccount::get()),
					erc20_address_not_exists(),
					1
				),
				module_evm_bridge::Error::<Runtime>::InvalidReturnValue,
			);
		});
}

#[test]
fn update_erc20_asset_work() {
	ExtBuilder::default()
		.balances(vec![(alice(), 1_000_000_000_000)])
		.build()
		.execute_with(|| {
			deploy_contracts();
			assert_ok!(AssetRegistry::register_erc20_asset(
				RuntimeOrigin::signed(CouncilAccount::get()),
				erc20_address(),
				1
			));

			assert_ok!(AssetRegistry::update_erc20_asset(
				RuntimeOrigin::signed(CouncilAccount::get()),
				erc20_address(),
				Box::new(AssetMetadata {
					name: b"New Token Name".to_vec(),
					symbol: b"NTN".to_vec(),
					decimals: 12,
					minimal_balance: 2,
				})
			));

			System::assert_last_event(RuntimeEvent::AssetRegistry(crate::Event::AssetUpdated {
				asset_id: AssetIds::Erc20(erc20_address()),
				metadata: AssetMetadata {
					name: b"New Token Name".to_vec(),
					symbol: b"NTN".to_vec(),
					decimals: 12,
					minimal_balance: 2,
				},
			}));

			assert_eq!(
				AssetMetadatas::<Runtime>::get(AssetIds::Erc20(erc20_address())),
				Some(AssetMetadata {
					name: b"New Token Name".to_vec(),
					symbol: b"NTN".to_vec(),
					decimals: 12,
					minimal_balance: 2,
				})
			);
		});
}

#[test]
fn register_native_asset_works() {
	ExtBuilder::default().build().execute_with(|| {
		assert_ok!(AssetRegistry::register_native_asset(
			RuntimeOrigin::signed(CouncilAccount::get()),
			CurrencyId::Token(TokenSymbol::EDF),
			Box::new(AssetMetadata {
				name: b"Token Name".to_vec(),
				symbol: b"TN".to_vec(),
				decimals: 12,
				minimal_balance: 1,
			})
		));
		System::assert_last_event(RuntimeEvent::AssetRegistry(crate::Event::AssetRegistered {
			asset_id: AssetIds::NativeAssetId(CurrencyId::Token(TokenSymbol::EDF)),
			metadata: AssetMetadata {
				name: b"Token Name".to_vec(),
				symbol: b"TN".to_vec(),
				decimals: 12,
				minimal_balance: 1,
			},
		}));

		assert_eq!(
			AssetMetadatas::<Runtime>::get(AssetIds::NativeAssetId(CurrencyId::Token(TokenSymbol::EDF))),
			Some(AssetMetadata {
				name: b"Token Name".to_vec(),
				symbol: b"TN".to_vec(),
				decimals: 12,
				minimal_balance: 1,
			})
		);
		// Can't duplicate
		assert_noop!(
			AssetRegistry::register_native_asset(
				RuntimeOrigin::signed(CouncilAccount::get()),
				CurrencyId::Token(TokenSymbol::EDF),
				Box::new(AssetMetadata {
					name: b"Token Name".to_vec(),
					symbol: b"TN".to_vec(),
					decimals: 12,
					minimal_balance: 1,
				})
			),
			Error::<Runtime>::AssetIdExisted
		);
	});
}

#[test]
fn update_native_asset_works() {
	ExtBuilder::default().build().execute_with(|| {
		assert_noop!(
			AssetRegistry::update_native_asset(
				RuntimeOrigin::signed(CouncilAccount::get()),
				CurrencyId::Token(TokenSymbol::EDF),
				Box::new(AssetMetadata {
					name: b"New Token Name".to_vec(),
					symbol: b"NTN".to_vec(),
					decimals: 12,
					minimal_balance: 2,
				})
			),
			Error::<Runtime>::AssetIdNotExists
		);

		assert_ok!(AssetRegistry::register_native_asset(
			RuntimeOrigin::signed(CouncilAccount::get()),
			CurrencyId::Token(TokenSymbol::EDF),
			Box::new(AssetMetadata {
				name: b"Token Name".to_vec(),
				symbol: b"TN".to_vec(),
				decimals: 12,
				minimal_balance: 1,
			})
		));

		assert_ok!(AssetRegistry::update_native_asset(
			RuntimeOrigin::signed(CouncilAccount::get()),
			CurrencyId::Token(TokenSymbol::EDF),
			Box::new(AssetMetadata {
				name: b"New Token Name".to_vec(),
				symbol: b"NTN".to_vec(),
				decimals: 12,
				minimal_balance: 2,
			})
		));

		System::assert_last_event(RuntimeEvent::AssetRegistry(crate::Event::AssetUpdated {
			asset_id: AssetIds::NativeAssetId(CurrencyId::Token(TokenSymbol::EDF)),
			metadata: AssetMetadata {
				name: b"New Token Name".to_vec(),
				symbol: b"NTN".to_vec(),
				decimals: 12,
				minimal_balance: 2,
			},
		}));

		assert_eq!(
			AssetMetadatas::<Runtime>::get(AssetIds::NativeAssetId(CurrencyId::Token(TokenSymbol::EDF))),
			Some(AssetMetadata {
				name: b"New Token Name".to_vec(),
				symbol: b"NTN".to_vec(),
				decimals: 12,
				minimal_balance: 2,
			})
		);
	});
}

#[test]
fn update_erc20_asset_should_not_work() {
	ExtBuilder::default().build().execute_with(|| {
		
		assert_noop!(
			AssetRegistry::update_erc20_asset(
				RuntimeOrigin::signed(CouncilAccount::get()),
				erc20_address(),
				Box::new(AssetMetadata {
					name: b"New Token Name".to_vec(),
					symbol: b"TOKEN".to_vec(),
					decimals: 12,
					minimal_balance: 2,
				})
			),
			Error::<Runtime>::AssetIdNotExists
		);
	});
}

#[test]
fn name_works() {
	ExtBuilder::default()
		.balances(vec![(alice(), 1_000_000_000_000)])
		.build()
		.execute_with(|| {
			deploy_contracts();
			assert_ok!(AssetRegistry::register_erc20_asset(
				RuntimeOrigin::signed(CouncilAccount::get()),
				erc20_address(),
				1
			));
			assert_eq!(
				EvmErc20InfoMapping::<Runtime>::name(CurrencyId::Token(TokenSymbol::SEE)),
				Some(b"Setheum".to_vec())
			);
			assert_eq!(
				EvmErc20InfoMapping::<Runtime>::name(CurrencyId::Erc20(erc20_address())),
				Some(b"long string name, long string name, long string name, long string name, long string name"[..32].to_vec())
			);

			assert_eq!(
				EvmErc20InfoMapping::<Runtime>::name(CurrencyId::Erc20(erc20_address_not_exists())),
				None
			);

			assert_eq!(
				EvmErc20InfoMapping::<Runtime>::name(CurrencyId::DexShare(DexShare::Token(TokenSymbol::SEE), DexShare::Token(TokenSymbol::USSD))),
				Some(b"LP Setheum - Slick USD".to_vec())
			);

			assert_eq!(
				EvmErc20InfoMapping::<Runtime>::name(CurrencyId::DexShare(DexShare::Erc20(erc20_address()), DexShare::Token(TokenSymbol::USSD))),
				Some(b"LP long string name, long string name, long string name, long string name, long string name - Slick USD"[..32].to_vec())
			);

			assert_eq!(
				EvmErc20InfoMapping::<Runtime>::name(CurrencyId::DexShare(DexShare::Erc20(erc20_address()), DexShare::Erc20(erc20_address()))),
				Some(b"LP long string name, long string name, long string name, long string name, long string name - long string name, long string name, long string name, long string name, long string name"[..32].to_vec())
			);

			assert_eq!(
				EvmErc20InfoMapping::<Runtime>::name(CurrencyId::DexShare(DexShare::Token(TokenSymbol::SEE), DexShare::Erc20(erc20_address_not_exists()))),
				None
			);

			assert_eq!(
				EvmErc20InfoMapping::<Runtime>::name(CurrencyId::DexShare(DexShare::Erc20(erc20_address()), DexShare::Erc20(erc20_address_not_exists()))),
				None
			);
		});
}

#[test]
fn symbol_works() {
	ExtBuilder::default()
		.balances(vec![(alice(), 1_000_000_000_000)])
		.build()
		.execute_with(|| {
			deploy_contracts();
			assert_ok!(AssetRegistry::register_erc20_asset(
				RuntimeOrigin::signed(CouncilAccount::get()),
				erc20_address(),
				1
			));
			assert_eq!(
				EvmErc20InfoMapping::<Runtime>::symbol(CurrencyId::Token(TokenSymbol::SEE)),
				Some(b"SEE".to_vec())
			);
			assert_eq!(
				EvmErc20InfoMapping::<Runtime>::symbol(CurrencyId::Erc20(erc20_address())),
				Some(b"TestToken".to_vec())
			);

			assert_eq!(
				EvmErc20InfoMapping::<Runtime>::symbol(CurrencyId::Erc20(erc20_address_not_exists())),
				None
			);

			assert_eq!(
				EvmErc20InfoMapping::<Runtime>::symbol(CurrencyId::DexShare(
					DexShare::Token(TokenSymbol::SEE),
					DexShare::Token(TokenSymbol::USSD)
				)),
				Some(b"LP_SEE_USSD".to_vec())
			);

			assert_eq!(
				EvmErc20InfoMapping::<Runtime>::symbol(CurrencyId::DexShare(
					DexShare::Erc20(erc20_address()),
					DexShare::Token(TokenSymbol::USSD)
				)),
				Some(b"LP_TestToken_USSD".to_vec())
			);

			assert_eq!(
				EvmErc20InfoMapping::<Runtime>::symbol(CurrencyId::DexShare(
					DexShare::Erc20(erc20_address()),
					DexShare::Erc20(erc20_address())
				)),
				Some(b"LP_TestToken_TestToken".to_vec())
			);

			assert_eq!(
				EvmErc20InfoMapping::<Runtime>::symbol(CurrencyId::DexShare(
					DexShare::Token(TokenSymbol::SEE),
					DexShare::Erc20(erc20_address_not_exists())
				)),
				None
			);

			assert_eq!(
				EvmErc20InfoMapping::<Runtime>::symbol(CurrencyId::DexShare(
					DexShare::Erc20(erc20_address()),
					DexShare::Erc20(erc20_address_not_exists())
				)),
				None
			);
		});
}

#[test]
fn decimals_works() {
	ExtBuilder::default()
		.balances(vec![(alice(), 1_000_000_000_000)])
		.build()
		.execute_with(|| {
			deploy_contracts();
			assert_ok!(AssetRegistry::register_erc20_asset(
				RuntimeOrigin::signed(CouncilAccount::get()),
				erc20_address(),
				1
			));
			assert_eq!(
				EvmErc20InfoMapping::<Runtime>::decimals(CurrencyId::Token(TokenSymbol::SEE)),
				Some(12)
			);
			assert_eq!(
				EvmErc20InfoMapping::<Runtime>::decimals(CurrencyId::Erc20(erc20_address())),
				Some(17)
			);

			assert_eq!(
				EvmErc20InfoMapping::<Runtime>::decimals(CurrencyId::Erc20(erc20_address_not_exists())),
				None
			);

			assert_eq!(
				EvmErc20InfoMapping::<Runtime>::decimals(CurrencyId::DexShare(
					DexShare::Token(TokenSymbol::SEE),
					DexShare::Token(TokenSymbol::USSD)
				)),
				Some(12)
			);

			assert_eq!(
				EvmErc20InfoMapping::<Runtime>::decimals(CurrencyId::DexShare(
					DexShare::Erc20(erc20_address()),
					DexShare::Token(TokenSymbol::USSD)
				)),
				Some(17)
			);

			assert_eq!(
				EvmErc20InfoMapping::<Runtime>::decimals(CurrencyId::DexShare(
					DexShare::Erc20(erc20_address()),
					DexShare::Erc20(erc20_address())
				)),
				Some(17)
			);

			assert_eq!(
				EvmErc20InfoMapping::<Runtime>::decimals(CurrencyId::DexShare(
					DexShare::Erc20(erc20_address()),
					DexShare::Erc20(erc20_address_not_exists())
				)),
				Some(17)
			);
}

#[test]
fn encode_evm_address_works() {
	ExtBuilder::default()
		.balances(vec![(alice(), 1_000_000_000_000)])
		.build()
		.execute_with(|| {
			deploy_contracts();
			assert_ok!(AssetRegistry::register_erc20_asset(
				RuntimeOrigin::signed(CouncilAccount::get()),
				erc20_address(),
				1
			));

			// Token
			assert_eq!(
				EvmErc20InfoMapping::<Runtime>::encode_evm_address(CurrencyId::Token(TokenSymbol::SEE)),
				H160::from_str("0x0000000000000000000100000000000000000000").ok()
			);

			// Erc20
			assert_eq!(
				EvmErc20InfoMapping::<Runtime>::encode_evm_address(CurrencyId::Erc20(erc20_address())),
				Some(erc20_address())
			);

			assert_eq!(
				EvmErc20InfoMapping::<Runtime>::encode_evm_address(CurrencyId::Erc20(erc20_address_not_exists())),
				Some(erc20_address_not_exists())
			);

			// DexShare
			assert_eq!(
				EvmErc20InfoMapping::<Runtime>::encode_evm_address(CurrencyId::DexShare(
					DexShare::Token(TokenSymbol::SEE),
					DexShare::Token(TokenSymbol::USSD)
				)),
				H160::from_str("0x0000000000000000000200000000000000000001").ok()
			);

			assert_eq!(
				EvmErc20InfoMapping::<Runtime>::encode_evm_address(CurrencyId::DexShare(
					DexShare::Erc20(erc20_address()),
					DexShare::Token(TokenSymbol::USSD)
				)),
				H160::from_str("0x00000000000000000002015dddfce50000000001").ok()
			);

			assert_eq!(
				EvmErc20InfoMapping::<Runtime>::encode_evm_address(CurrencyId::DexShare(
					DexShare::Token(TokenSymbol::USSD),
					DexShare::Erc20(erc20_address())
				)),
				H160::from_str("0x000000000000000000020000000001015dddfce5").ok()
			);

			assert_eq!(
				EvmErc20InfoMapping::<Runtime>::encode_evm_address(CurrencyId::DexShare(
					DexShare::Erc20(erc20_address()),
					DexShare::Erc20(erc20_address())
				)),
				H160::from_str("0x00000000000000000002015dddfce5015dddfce5").ok()
			);

			assert_eq!(
				EvmErc20InfoMapping::<Runtime>::encode_evm_address(CurrencyId::DexShare(
					DexShare::Token(TokenSymbol::SEE),
					DexShare::Erc20(erc20_address_not_exists())
				)),
				None
			);

			assert_eq!(
				EvmErc20InfoMapping::<Runtime>::encode_evm_address(CurrencyId::DexShare(
					DexShare::Erc20(erc20_address()),
					DexShare::Erc20(erc20_address_not_exists())
				)),
				None
			);

			// ForeignAsset
			assert_eq!(
				EvmErc20InfoMapping::<Runtime>::encode_evm_address(CurrencyId::ForeignAsset(1)),
				H160::from_str("0x0000000000000000000500000000000000000001").ok()
			);
		});
}

#[test]
fn decode_evm_address_works() {
	ExtBuilder::default()
		.balances(vec![(alice(), 1_000_000_000_000)])
		.build()
		.execute_with(|| {
			deploy_contracts();
			assert_ok!(AssetRegistry::register_erc20_asset(
				RuntimeOrigin::signed(CouncilAccount::get()),
				erc20_address(),
				1
			));

			// Token
			assert_eq!(
				EvmErc20InfoMapping::<Runtime>::decode_evm_address(
					EvmErc20InfoMapping::<Runtime>::encode_evm_address(CurrencyId::Token(TokenSymbol::SEE)).unwrap()
				),
				Some(CurrencyId::Token(TokenSymbol::SEE))
			);

			// Erc20
			assert_eq!(
				EvmErc20InfoMapping::<Runtime>::decode_evm_address(
					EvmErc20InfoMapping::<Runtime>::encode_evm_address(CurrencyId::Erc20(erc20_address())).unwrap()
				),
				Some(CurrencyId::Erc20(erc20_address()))
			);

			assert_eq!(
				EvmErc20InfoMapping::<Runtime>::decode_evm_address(
					EvmErc20InfoMapping::<Runtime>::encode_evm_address(CurrencyId::Erc20(erc20_address_not_exists()))
						.unwrap()
				),
				None,
			);

			// DexShare
			assert_eq!(
				EvmErc20InfoMapping::<Runtime>::decode_evm_address(
					EvmErc20InfoMapping::<Runtime>::encode_evm_address(CurrencyId::DexShare(
						DexShare::Token(TokenSymbol::SEE),
						DexShare::Token(TokenSymbol::USSD)
					))
					.unwrap(),
				),
				Some(CurrencyId::DexShare(
					DexShare::Token(TokenSymbol::SEE),
					DexShare::Token(TokenSymbol::USSD)
				))
			);

			assert_eq!(
				EvmErc20InfoMapping::<Runtime>::decode_evm_address(
					EvmErc20InfoMapping::<Runtime>::encode_evm_address(CurrencyId::DexShare(
						DexShare::Erc20(erc20_address()),
						DexShare::Token(TokenSymbol::USSD)
					))
					.unwrap()
				),
				Some(CurrencyId::DexShare(
					DexShare::Erc20(erc20_address()),
					DexShare::Token(TokenSymbol::USSD)
				))
			);

			assert_eq!(
				EvmErc20InfoMapping::<Runtime>::decode_evm_address(
					EvmErc20InfoMapping::<Runtime>::encode_evm_address(CurrencyId::DexShare(
						DexShare::Erc20(erc20_address()),
						DexShare::Erc20(erc20_address())
					))
					.unwrap()
				),
				Some(CurrencyId::DexShare(
					DexShare::Erc20(erc20_address()),
					DexShare::Erc20(erc20_address())
				))
			);

			// decode invalid evm address
			// CurrencyId::DexShare(DexShare::Token(TokenSymbol::SEE),
			// DexShare::Erc20(erc20_address_not_exists()))
			assert_eq!(
				EvmErc20InfoMapping::<Runtime>::decode_evm_address(
					H160::from_str("0x0000000000000000000000010000000002000001").unwrap()
				),
				None
			);

			// decode invalid evm address
			// CurrencyId::DexShare(DexShare::Erc20(erc20_address()),
			// DexShare::Erc20(erc20_address_not_exists()))
			assert_eq!(
				EvmErc20InfoMapping::<Runtime>::decode_evm_address(
					H160::from_str("0x0000000000000000000000010200000002000001").unwrap()
				),
				None
			);

			// Allow non-system contracts
			let non_system_contracts = H160::from_str("0x1000000000000000000000000000000000000000").unwrap();
			assert_eq!(
				EvmErc20InfoMapping::<Runtime>::decode_evm_address(non_system_contracts),
				Some(CurrencyId::Erc20(non_system_contracts))
			);

			// ForeignAsset
			assert_eq!(
				EvmErc20InfoMapping::<Runtime>::decode_evm_address(
					EvmErc20InfoMapping::<Runtime>::encode_evm_address(CurrencyId::ForeignAsset(1)).unwrap()
				),
				Some(CurrencyId::ForeignAsset(1))
			);
		});
}
