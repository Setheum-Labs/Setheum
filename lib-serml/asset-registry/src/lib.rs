// بِسْمِ اللَّهِ الرَّحْمَنِ الرَّحِيم

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

//! # Asset Registry Module
//!
//! Local and foreign assets management. The foreign assets can be updated without runtime upgrade.

#![cfg_attr(not(feature = "std"), no_std)]
#![allow(clippy::unused_unit)]

use frame_support::{
	dispatch::DispatchResult,
	ensure,
	pallet_prelude::*,
	traits::{Currency, EnsureOrigin},
	transactional,
	weights::constants::WEIGHT_PER_SECOND,
	RuntimeDebug,
};
use frame_system::pallet_prelude::*;
use module_support::{AssetIdMapping, EVMBridge, Erc20InfoMapping, InvokeContext};
use primitives::{
	currency::{CurrencyIdType, DexShare, DexShareType, Erc20Id, TokenInfo},
	evm::{
		is_system_contract, EvmAddress, H160_POSITION_CURRENCY_ID_TYPE, H160_POSITION_DEXSHARE_LEFT_FIELD,
		H160_POSITION_DEXSHARE_LEFT_TYPE, H160_POSITION_DEXSHARE_RIGHT_FIELD, H160_POSITION_DEXSHARE_RIGHT_TYPE,
		H160_POSITION_FOREIGN_ASSET, H160_POSITION_LIQUID_CROADLOAN, H160_POSITION_STABLE_ASSET, H160_POSITION_TOKEN,
	},
	CurrencyId,
};
use scale_info::{prelude::format, TypeInfo};
use sp_runtime::{traits::One, ArithmeticError, FixedPointNumber, FixedU128};
use sp_std::{boxed::Box, vec::Vec};

mod mock;
mod tests;
mod weights;

pub use module::*;
pub use weights::WeightInfo;

/// Type alias for currency balance.
pub type BalanceOf<T> = <<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

#[frame_support::pallet]
pub mod module {
	use super::*;

	#[pallet::config]
	pub trait Config: frame_system::Config {
		/// The overarching event type.
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;

		/// Currency type for withdraw and balance storage.
		type Currency: Currency<Self::AccountId>;

		/// Evm Bridge for getting info of contracts from the SEVM.
		type EVMBridge: EVMBridge<Self::AccountId, BalanceOf<Self>>;

		/// Required origin for registering asset.
		type RegisterOrigin: EnsureOrigin<Self::Origin>;

		/// Weight information for the extrinsics in this module.
		type WeightInfo: WeightInfo;
	}

	#[derive(Clone, Eq, PartialEq, RuntimeDebug, Encode, Decode, TypeInfo)]
	pub enum AssetIds {
		Erc20(EvmAddress),
	}

	#[derive(Clone, Eq, PartialEq, RuntimeDebug, Encode, Decode, TypeInfo)]
	pub struct AssetMetadata<Balance> {
		pub name: Vec<u8>,
		pub symbol: Vec<u8>,
		pub decimals: u8,
		pub minimal_balance: Balance,
	}

	#[pallet::error]
	pub enum Error<T> {
		/// The given location could not be used (e.g. because it cannot be expressed in the
		/// desired version of XCM).
		BadLocation,
		/// AssetId not exists
		AssetIdNotExists,
		/// AssetId exists
		AssetIdExisted,
	}

	#[pallet::event]
	#[pallet::generate_deposit(fn deposit_event)]
	pub enum Event<T: Config> {
		/// The asset registered.
		AssetRegistered {
			asset_id: AssetIds,
			metadata: AssetMetadata<BalanceOf<T>>,
		},
		/// The asset updated.
		AssetUpdated {
			asset_id: AssetIds,
			metadata: AssetMetadata<BalanceOf<T>>,
		},
	}

	/// The storages for EvmAddress.
	///
	/// Erc20IdToAddress: map Erc20Id => Option<EvmAddress>
	#[pallet::storage]
	#[pallet::getter(fn erc20_id_to_address)]
	pub type Erc20IdToAddress<T: Config> = StorageMap<_, Twox64Concat, Erc20Id, EvmAddress, OptionQuery>;

	/// The storages for AssetMetadatas.
	///
	/// AssetMetadatas: map AssetIds => Option<AssetMetadata>
	#[pallet::storage]
	#[pallet::getter(fn asset_metadatas)]
	pub type AssetMetadatas<T: Config> =
		StorageMap<_, Twox64Concat, AssetIds, AssetMetadata<BalanceOf<T>>, OptionQuery>;

	#[pallet::pallet]
	pub struct Pallet<T>(_);

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		#[pallet::weight(T::WeightInfo::register_erc20_asset())]
		#[transactional]
		pub fn register_erc20_asset(
			origin: OriginFor<T>,
			contract: EvmAddress,
			minimal_balance: BalanceOf<T>,
		) -> DispatchResult {
			T::RegisterOrigin::ensure_origin(origin)?;

			let metadata = Self::do_register_erc20_asset(contract, minimal_balance)?;

			Self::deposit_event(Event::<T>::AssetRegistered {
				asset_id: AssetIds::Erc20(contract),
				metadata,
			});
			Ok(())
		}

		#[pallet::weight(T::WeightInfo::update_erc20_asset())]
		#[transactional]
		pub fn update_erc20_asset(
			origin: OriginFor<T>,
			contract: EvmAddress,
			metadata: Box<AssetMetadata<BalanceOf<T>>>,
		) -> DispatchResult {
			T::RegisterOrigin::ensure_origin(origin)?;

			Self::do_update_erc20_asset(contract, &metadata)?;

			Self::deposit_event(Event::<T>::AssetUpdated {
				asset_id: AssetIds::Erc20(contract),
				metadata: *metadata,
			});
			Ok(())
		}
	}
}

impl<T: Config> Pallet<T> {
	fn do_register_erc20_asset(
		contract: EvmAddress,
		minimal_balance: BalanceOf<T>,
	) -> Result<AssetMetadata<BalanceOf<T>>, DispatchError> {
		let invoke_context = InvokeContext {
			contract,
			sender: Default::default(),
			origin: Default::default(),
		};

		let metadata = AssetMetadata {
			name: T::EVMBridge::name(invoke_context)?,
			symbol: T::EVMBridge::symbol(invoke_context)?,
			decimals: T::EVMBridge::decimals(invoke_context)?,
			minimal_balance,
		};

		let erc20_id = Into::<Erc20Id>::into(DexShare::Erc20(contract));

		AssetMetadatas::<T>::try_mutate(AssetIds::Erc20(contract), |maybe_asset_metadatas| -> DispatchResult {
			ensure!(maybe_asset_metadatas.is_none(), Error::<T>::AssetIdExisted);

			Erc20IdToAddress::<T>::try_mutate(erc20_id, |maybe_address| -> DispatchResult {
				ensure!(maybe_address.is_none(), Error::<T>::AssetIdExisted);
				*maybe_address = Some(contract);

				Ok(())
			})?;

			*maybe_asset_metadatas = Some(metadata.clone());
			Ok(())
		})?;

		Ok(metadata)
	}

	fn do_update_erc20_asset(contract: EvmAddress, metadata: &AssetMetadata<BalanceOf<T>>) -> DispatchResult {
		AssetMetadatas::<T>::try_mutate(AssetIds::Erc20(contract), |maybe_asset_metadatas| -> DispatchResult {
			ensure!(maybe_asset_metadatas.is_some(), Error::<T>::AssetIdNotExists);

			*maybe_asset_metadatas = Some(metadata.clone());
			Ok(())
		})
	}
}

pub struct AssetIdMaps<T>(sp_std::marker::PhantomData<T>);

impl<T: Config> AssetIdMapping<AssetMetadata<BalanceOf<T>>>
	for AssetIdMaps<T>
{
	fn get_erc20_asset_metadata(contract: EvmAddress) -> Option<AssetMetadata<BalanceOf<T>>> {
		Pallet::<T>::asset_metadatas(AssetIds::Erc20(contract))
	}
}

pub struct EvmErc20InfoMapping<T>(sp_std::marker::PhantomData<T>);

impl<T: Config> Erc20InfoMapping for EvmErc20InfoMapping<T> {
	// Returns the name associated with a given CurrencyId.
	// If CurrencyId is CurrencyId::DexShare and contain DexShare::Erc20,
	// the EvmAddress must have been mapped.
	fn name(currency_id: CurrencyId) -> Option<Vec<u8>> {
		let name = match currency_id {
			CurrencyId::Token(_) => currency_id.name().map(|v| v.as_bytes().to_vec()),
			CurrencyId::DexShare(symbol_0, symbol_1) => {
				let name_0 = match symbol_0 {
					DexShare::Token(symbol) => CurrencyId::Token(symbol).name().map(|v| v.as_bytes().to_vec()),
					DexShare::Erc20(address) => AssetMetadatas::<T>::get(AssetIds::Erc20(address)).map(|v| v.name),
				}?;
				let name_1 = match symbol_1 {
					DexShare::Token(symbol) => CurrencyId::Token(symbol).name().map(|v| v.as_bytes().to_vec()),
					DexShare::Erc20(address) => AssetMetadatas::<T>::get(AssetIds::Erc20(address)).map(|v| v.name),
				}?;

				let mut vec = Vec::new();
				vec.extend_from_slice(&b"LP "[..]);
				vec.extend_from_slice(&name_0);
				vec.extend_from_slice(&b" - ".to_vec());
				vec.extend_from_slice(&name_1);
				Some(vec)
			}
			CurrencyId::Erc20(address) => AssetMetadatas::<T>::get(AssetIds::Erc20(address)).map(|v| v.name),
		}?;

		// More than 32 bytes will be truncated.
		if name.len() > 32 {
			Some(name[..32].to_vec())
		} else {
			Some(name)
		}
	}

	// Returns the symbol associated with a given CurrencyId.
	// If CurrencyId is CurrencyId::DexShare and contain DexShare::Erc20,
	// the EvmAddress must have been mapped.
	fn symbol(currency_id: CurrencyId) -> Option<Vec<u8>> {
		let symbol = match currency_id {
			CurrencyId::Token(_) => currency_id.symbol().map(|v| v.as_bytes().to_vec()),
			CurrencyId::DexShare(symbol_0, symbol_1) => {
				let token_symbol_0 = match symbol_0 {
					DexShare::Token(symbol) => CurrencyId::Token(symbol).symbol().map(|v| v.as_bytes().to_vec()),
					DexShare::Erc20(address) => AssetMetadatas::<T>::get(AssetIds::Erc20(address)).map(|v| v.symbol),
				}?;
				let token_symbol_1 = match symbol_1 {
					DexShare::Token(symbol) => CurrencyId::Token(symbol).symbol().map(|v| v.as_bytes().to_vec()),
					DexShare::Erc20(address) => AssetMetadatas::<T>::get(AssetIds::Erc20(address)).map(|v| v.symbol),
				}?;

				let mut vec = Vec::new();
				vec.extend_from_slice(&b"LP_"[..]);
				vec.extend_from_slice(&token_symbol_0);
				vec.extend_from_slice(&b"_".to_vec());
				vec.extend_from_slice(&token_symbol_1);
				Some(vec)
			}
			CurrencyId::Erc20(address) => AssetMetadatas::<T>::get(AssetIds::Erc20(address)).map(|v| v.symbol),
		}?;

		// More than 32 bytes will be truncated.
		if symbol.len() > 32 {
			Some(symbol[..32].to_vec())
		} else {
			Some(symbol)
		}
	}

	// Returns the decimals associated with a given CurrencyId.
	// If CurrencyId is CurrencyId::DexShare and contain DexShare::Erc20,
	// the EvmAddress must have been mapped.
	fn decimals(currency_id: CurrencyId) -> Option<u8> {
		match currency_id {
			CurrencyId::Token(_) => currency_id.decimals(),
			CurrencyId::DexShare(symbol_0, _) => {
				// initial dex share amount is calculated based on currency_id_0,
				// use the decimals of currency_id_0 as the decimals of lp token.
				match symbol_0 {
					DexShare::Token(symbol) => CurrencyId::Token(symbol).decimals(),
					DexShare::Erc20(address) => AssetMetadatas::<T>::get(AssetIds::Erc20(address)).map(|v| v.decimals),
				}
			}
			CurrencyId::Erc20(address) => AssetMetadatas::<T>::get(AssetIds::Erc20(address)).map(|v| v.decimals),
		}
	}

	// Encode the CurrencyId to EvmAddress.
	// If is CurrencyId::DexShare and contain DexShare::Erc20,
	// will use the u32 to get the DexShare::Erc20 from the mapping.
	fn encode_evm_address(v: CurrencyId) -> Option<EvmAddress> {
		match v {
			CurrencyId::DexShare(left, right) => {
				match left {
					DexShare::Erc20(address) => {
						// ensure erc20 is mapped
						AssetMetadatas::<T>::get(AssetIds::Erc20(address)).map(|_| ())?;
					}
					DexShare::Token(_) => {}
				};
				match right {
					DexShare::Erc20(address) => {
						// ensure erc20 is mapped
						AssetMetadatas::<T>::get(AssetIds::Erc20(address)).map(|_| ())?;
					}
					DexShare::Token(_) => {}
				};
			}
			CurrencyId::Token(_)
			| CurrencyId::Erc20(_) => {}
		};

		EvmAddress::try_from(v).ok()
	}

	// Decode the CurrencyId from EvmAddress.
	// If is CurrencyId::DexShare and contain DexShare::Erc20,
	// will use the u32 to get the DexShare::Erc20 from the mapping.
	fn decode_evm_address(addr: EvmAddress) -> Option<CurrencyId> {
		if !is_system_contract(addr) {
			return Some(CurrencyId::Erc20(addr));
		}

		let address = addr.as_bytes();
		let currency_id = match CurrencyIdType::try_from(address[H160_POSITION_CURRENCY_ID_TYPE]).ok()? {
			CurrencyIdType::Token => address[H160_POSITION_TOKEN].try_into().map(CurrencyId::Token).ok(),
			CurrencyIdType::DexShare => {
				let left = match DexShareType::try_from(address[H160_POSITION_DEXSHARE_LEFT_TYPE]).ok()? {
					DexShareType::Token => address[H160_POSITION_DEXSHARE_LEFT_FIELD][3]
						.try_into()
						.map(DexShare::Token)
						.ok(),
					DexShareType::Erc20 => {
						let id = u32::from_be_bytes(address[H160_POSITION_DEXSHARE_LEFT_FIELD].try_into().ok()?);
						Erc20IdToAddress::<T>::get(id).map(DexShare::Erc20)
					}
				}?;
				let right = match DexShareType::try_from(address[H160_POSITION_DEXSHARE_RIGHT_TYPE]).ok()? {
					DexShareType::Token => address[H160_POSITION_DEXSHARE_RIGHT_FIELD][3]
						.try_into()
						.map(DexShare::Token)
						.ok(),
					DexShareType::Erc20 => {
						let id = u32::from_be_bytes(address[H160_POSITION_DEXSHARE_RIGHT_FIELD].try_into().ok()?);
						Erc20IdToAddress::<T>::get(id).map(DexShare::Erc20)
					}
				}?;

				Some(CurrencyId::DexShare(left, right))
			}
		};

		// Make sure that every bit of the address is the same
		Self::encode_evm_address(currency_id?).and_then(|encoded| if encoded == addr { currency_id } else { None })
	}
}
