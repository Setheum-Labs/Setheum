// This file is part of Setheum.

// Copyright (C) 2020-2021 Setheum Labs.
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

#![cfg(test)]

use codec::Encode;
// use cumulus_primitives_core::{DownwardMessageHandler, InboundDownwardMessage,
// XcmpMessageHandler};
use frame_support::{
	assert_noop, assert_ok,
	traits::{schedule::DispatchTime, Currency, GenesisBuild, OnFinalize, OnInitialize, OriginTrait},
};
use frame_system::RawOrigin;
use newrome_runtime::{
	dollar, get_all_module_accounts, AccountId, AuthoritysOriginId, Balance, Balances, BlockNumber, Call,
	CreateClassDeposit, CreateTokenDeposit, CurrencyId, SIFModuleId, EnabledTradingPairs, Event, 
	GetNativeCurrencyId, NativeTokenExistentialDeposit, NftModuleId, Origin, OriginCaller, Perbill, Proxy, Runtime,
	SevenDays, System, TokenSymbol, DNAR, JUSD, DOT, JEUR, NFT, JCHF,
};
use setheum_support::{SetheumDexManager, Price, Rate, Ratio};
use orml_authority::DelayedOrigin;
use orml_traits::{Change, MultiCurrency};
use sp_io::hashing::keccak_256;
use sp_runtime::{
	traits::{AccountIdConversion, BadOrigin, Zero},
	DispatchError, DispatchResult, FixedPointNumber, MultiAddress,
};

use std::str::FromStr;
// use xcm::{
// 	v0::{
// 		Junction::{self, *},
// 		MultiAsset,
// 		MultiLocation::{self, *},
// 		NetworkId, Order, Xcm,
// 	},
// 	VersionedXcm,
// };

// use primitives::currency::*;

const ORACLE1: [u8; 32] = [0u8; 32];
const ORACLE2: [u8; 32] = [1u8; 32];
const ORACLE3: [u8; 32] = [2u8; 32];

const ALICE: [u8; 32] = [4u8; 32];
const BOB: [u8; 32] = [5u8; 32];

pub type OracleModule = orml_oracle::Pallet<Runtime, orml_oracle::Instance1>;
pub type SetheumDex = SetheumDex::Pallet<Runtime>;
pub type SystemModule = frame_system::Pallet<Runtime>;
pub type AuthorityModule = orml_authority::Pallet<Runtime>;
pub type Currencies = setheum_currencies::Pallet<Runtime>;
pub type SchedulerModule = pallet_scheduler::Pallet<Runtime>;

fn run_to_block(n: u32) {
	while SystemModule::block_number() < n {
		SchedulerModule::on_finalize(SystemModule::block_number());
		SystemModule::set_block_number(SystemModule::block_number() + 1);
		SchedulerModule::on_initialize(SystemModule::block_number());
	}
}

fn last_event() -> Event {
	SystemModule::events().pop().expect("Event expected").event
}

pub struct ExtBuilder {
	endowed_accounts: Vec<(AccountId, CurrencyId, Balance)>,
}

impl Default for ExtBuilder {
	fn default() -> Self {
		Self {
			endowed_accounts: vec![],
		}
	}
}

impl ExtBuilder {
	pub fn balances(mut self, endowed_accounts: Vec<(AccountId, CurrencyId, Balance)>) -> Self {
		self.endowed_accounts = endowed_accounts;
		self
	}

	pub fn build(self) -> sp_io::TestExternalities {
		let mut t = frame_system::GenesisConfig::default()
			.build_storage::<Runtime>()
			.unwrap();

		let native_currency_id = GetNativeCurrencyId::get();
		let existential_deposit = NativeTokenExistentialDeposit::get();
		let initial_enabled_trading_pairs = EnabledTradingPairs::get();

		setheum_dex::GenesisConfig::<Runtime> {
			initial_enabled_trading_pairs: initial_enabled_trading_pairs,
			initial_listing_trading_pairs: Default::default(),
			initial_added_liquidity_pools: vec![],
		}
		.assimilate_storage(&mut t)
		.unwrap();

		pallet_balances::GenesisConfig::<Runtime> {
			balances: self
				.endowed_accounts
				.clone()
				.into_iter()
				.filter(|(_, currency_id, _)| *currency_id == native_currency_id)
				.map(|(account_id, _, initial_balance)| (account_id, initial_balance))
				.chain(
					get_all_module_accounts()
						.iter()
						.map(|x| (x.clone(), existential_deposit)),
				)
				.collect::<Vec<_>>(),
		}
		.assimilate_storage(&mut t)
		.unwrap();

		orml_tokens::GenesisConfig::<Runtime> {
			endowed_accounts: self
				.endowed_accounts
				.into_iter()
				.filter(|(_, currency_id, _)| *currency_id != native_currency_id)
				.collect::<Vec<_>>(),
		}
		.assimilate_storage(&mut t)
		.unwrap();

		pallet_membership::GenesisConfig::<Runtime, pallet_membership::Instance5> {
			members: vec![
				AccountId::from(ORACLE1),
				AccountId::from(ORACLE2),
				AccountId::from(ORACLE3),
			],
			phantom: Default::default(),
		}
		.assimilate_storage(&mut t)
		.unwrap();

		let mut ext = sp_io::TestExternalities::new(t);
		ext.execute_with(|| SystemModule::set_block_number(1));
		ext
	}
}

pub fn origin_of(account_id: AccountId) -> <Runtime as frame_system::Config>::Origin {
	<Runtime as frame_system::Config>::Origin::signed(account_id)
}

fn set_oracle_price(prices: Vec<(CurrencyId, Price)>) -> DispatchResult {
	OracleModule::on_finalize(0);
	assert_ok!(OracleModule::feed_values(
		origin_of(AccountId::from(ORACLE1)),
		prices.clone(),
	));
	assert_ok!(OracleModule::feed_values(
		origin_of(AccountId::from(ORACLE2)),
		prices.clone(),
	));
	assert_ok!(OracleModule::feed_values(origin_of(AccountId::from(ORACLE3)), prices,));
	Ok(())
}

fn alice() -> secp256k1::SecretKey {
	secp256k1::SecretKey::parse(&keccak_256(b"Alice")).unwrap()
}

fn bob() -> secp256k1::SecretKey {
	secp256k1::SecretKey::parse(&keccak_256(b"Bob")).unwrap()
}

#[test]
fn test_dex_module() {
	ExtBuilder::default()
		.balances(vec![
			(AccountId::from(ALICE), JUSD, (1_000_000_000_000_000_000u128)),
			(AccountId::from(ALICE), JCHF, (1_000_000_000_000_000_000u128)),
			(AccountId::from(BOB), JUSD, (1_000_000_000_000_000_000u128)),
			(AccountId::from(BOB), JCHF, (1_000_000_000_000_000_000u128)),
		])
		.build()
		.execute_with(|| {
			assert_eq!(SetheumDex::get_liquidity_pool(JCHF, JUSD), (0, 0));
			assert_eq!(
				Currencies::total_issuance(CurrencyId::DEXShare(TokenSymbol::JUSD, TokenSymbol::JCHF)),
				0
			);
			assert_eq!(
				Currencies::free_balance(
					CurrencyId::DEXShare(TokenSymbol::JUSD, TokenSymbol::JCHF),
					&AccountId::from(ALICE)
				),
				0
			);

			assert_noop!(
				SetheumDex::add_liquidity(origin_of(AccountId::from(ALICE)), JCHF, JUSD, 0, 10000000, false,),
				setheum_dex::Error::<Runtime>::InvalidLiquidityIncrement,
			);

			assert_ok!(SetheumDex::add_liquidity(
				origin_of(AccountId::from(ALICE)),
				JCHF,
				JUSD,
				10000,
				10000000,
				false,
			));

			let add_liquidity_event = Event::setheum_dex(setheum_dex::Event::AddLiquidity(
				AccountId::from(ALICE),
				JUSD,
				10000000,
				JCHF,
				10000,
				20000000,
			));
			assert!(SystemModule::events()
				.iter()
				.any(|record| record.event == add_liquidity_event));

			assert_eq!(SetheumDex::get_liquidity_pool(JCHF, JUSD), (10000, 10000000));
			assert_eq!(
				Currencies::total_issuance(CurrencyId::DEXShare(TokenSymbol::JUSD, TokenSymbol::JCHF)),
				20000000
			);
			assert_eq!(
				Currencies::free_balance(
					CurrencyId::DEXShare(TokenSymbol::JUSD, TokenSymbol::JCHF),
					&AccountId::from(ALICE)
				),
				20000000
			);
			assert_ok!(SetheumDex::add_liquidity(
				origin_of(AccountId::from(BOB)),
				JCHF,
				JUSD,
				1,
				1000,
				false,
			));
			assert_eq!(SetheumDex::get_liquidity_pool(JCHF, JUSD), (10001, 10001000));
			assert_eq!(
				Currencies::total_issuance(CurrencyId::DEXShare(TokenSymbol::JUSD, TokenSymbol::JCHF)),
				20002000
			);
			assert_eq!(
				Currencies::free_balance(
					CurrencyId::DEXShare(TokenSymbol::JUSD, TokenSymbol::JCHF),
					&AccountId::from(BOB)
				),
				2000
			);
			assert_noop!(
				SetheumDex::add_liquidity(origin_of(AccountId::from(BOB)), JCHF, JUSD, 1, 999, false,),
				setheum_dex::Error::<Runtime>::InvalidLiquidityIncrement,
			);
			assert_eq!(SetheumDex::get_liquidity_pool(JCHF, JUSD), (10001, 10001000));
			assert_eq!(
				Currencies::total_issuance(CurrencyId::DEXShare(TokenSymbol::JUSD, TokenSymbol::JCHF)),
				20002000
			);
			assert_eq!(
				Currencies::free_balance(
					CurrencyId::DEXShare(TokenSymbol::JUSD, TokenSymbol::JCHF),
					&AccountId::from(BOB)
				),
				2000
			);
			assert_ok!(SetheumDex::add_liquidity(
				origin_of(AccountId::from(BOB)),
				JCHF,
				JUSD,
				2,
				1000,
				false,
			));
			assert_eq!(SetheumDex::get_liquidity_pool(JCHF, JUSD), (10002, 10002000));
			assert_ok!(SetheumDex::add_liquidity(
				origin_of(AccountId::from(BOB)),
				JCHF,
				JUSD,
				1,
				1001,
				false,
			));
			assert_eq!(SetheumDex::get_liquidity_pool(JCHF, JUSD), (10003, 10003000));

			assert_eq!(
				Currencies::total_issuance(CurrencyId::DEXShare(TokenSymbol::JUSD, TokenSymbol::JCHF)),
				20005998
			);
		});
}

#[test]
fn test_authority_module() {
	const AUTHORITY_ORIGIN_ID: u8 = 27u8;

	ExtBuilder::default()
		.balances(vec![
			(AccountId::from(ALICE), JUSD, 1_000 * dollar(JUSD)),
			(AccountId::from(ALICE), JCHF, 1_000 * dollar(JCHF)),
			(SIFModuleId::get().into_account(), JUSD, 1000 * dollar(JUSD)),
		])
		.build()
		.execute_with(|| {
			let ensure_root_call = Call::System(frame_system::Call::fill_block(Perbill::one()));
			let call = Call::Authority(orml_authority::Call::dispatch_as(
				AuthoritysOriginId::Root,
				Box::new(ensure_root_call.clone()),
			));

			// dispatch_as
			assert_ok!(AuthorityModule::dispatch_as(
				Origin::root(),
				AuthoritysOriginId::Root,
				Box::new(ensure_root_call.clone())
			));

			assert_noop!(
				AuthorityModule::dispatch_as(
					Origin::signed(AccountId::from(BOB)),
					AuthoritysOriginId::Root,
					Box::new(ensure_root_call.clone())
				),
				BadOrigin
			);

			assert_noop!(
				AuthorityModule::dispatch_as(
					Origin::signed(AccountId::from(BOB)),
					AuthoritysOriginId::SetheumTreasury,
					Box::new(ensure_root_call.clone())
				),
				BadOrigin
			);

			// schedule_dispatch
			run_to_block(1);
			// SIF transfer
			let transfer_call = Call::Currencies(setheum_currencies::Call::transfer(
				AccountId::from(BOB).into(),
				JUSD,
				500 * dollar(JUSD),
			));
			let sif_call = Call::Authority(orml_authority::Call::dispatch_as(
				AuthoritysOriginId::SIF,
				Box::new(transfer_call.clone()),
			));
			assert_ok!(AuthorityModule::schedule_dispatch(
				Origin::root(),
				DispatchTime::At(2),
				0,
				true,
				Box::new(sif_call.clone())
			));

			assert_ok!(AuthorityModule::schedule_dispatch(
				Origin::root(),
				DispatchTime::At(2),
				0,
				true,
				Box::new(call.clone())
			));

			let event = Event::orml_authority(orml_authority::Event::Scheduled(
				OriginCaller::orml_authority(DelayedOrigin {
					delay: 1,
					origin: Box::new(OriginCaller::system(RawOrigin::Root)),
				}),
				1,
			));
			assert_eq!(last_event(), event);

			run_to_block(2);
			assert_eq!(
				Currencies::free_balance(JUSD, &SIFModuleId::get().into_account()),
				500 * dollar(JUSD)
			);
			assert_eq!(
				Currencies::free_balance(JUSD, &AccountId::from(BOB)),
				500 * dollar(JUSD)
			);

			// delay < SevenDays
			let event = Event::pallet_scheduler(pallet_scheduler::RawEvent::Dispatched(
				(2, 1),
				Some([AUTHORITY_ORIGIN_ID, 1, 0, 0, 0, 0, 0, 1, 0, 0, 0].to_vec()),
				Err(DispatchError::BadOrigin),
			));
			assert_eq!(last_event(), event);

			// delay = SevenDays
			assert_ok!(AuthorityModule::schedule_dispatch(
				Origin::root(),
				DispatchTime::At(SevenDays::get() + 2),
				0,
				true,
				Box::new(call.clone())
			));

			run_to_block(SevenDays::get() + 2);
			let event = Event::pallet_scheduler(pallet_scheduler::RawEvent::Dispatched(
				(100802, 0),
				Some([AUTHORITY_ORIGIN_ID, 192, 137, 1, 0, 0, 0, 2, 0, 0, 0].to_vec()),
				Ok(()),
			));
			assert_eq!(last_event(), event);

			// with_delayed_origin = false
			assert_ok!(AuthorityModule::schedule_dispatch(
				Origin::root(),
				DispatchTime::At(SevenDays::get() + 3),
				0,
				false,
				Box::new(call.clone())
			));
			let event = Event::orml_authority(orml_authority::Event::Scheduled(
				OriginCaller::system(RawOrigin::Root),
				3,
			));
			assert_eq!(last_event(), event);

			run_to_block(SevenDays::get() + 3);
			let event = Event::pallet_scheduler(pallet_scheduler::RawEvent::Dispatched(
				(100803, 0),
				Some([0, 0, 3, 0, 0, 0].to_vec()),
				Ok(()),
			));
			assert_eq!(last_event(), event);

			assert_ok!(AuthorityModule::schedule_dispatch(
				Origin::root(),
				DispatchTime::At(SevenDays::get() + 4),
				0,
				false,
				Box::new(call.clone())
			));

			// fast_track_scheduled_dispatch
			assert_ok!(AuthorityModule::fast_track_scheduled_dispatch(
				Origin::root(),
				frame_system::RawOrigin::Root.into(),
				4,
				DispatchTime::At(SevenDays::get() + 5),
			));

			// delay_scheduled_dispatch
			assert_ok!(AuthorityModule::delay_scheduled_dispatch(
				Origin::root(),
				frame_system::RawOrigin::Root.into(),
				4,
				4,
			));

			// cancel_scheduled_dispatch
			assert_ok!(AuthorityModule::schedule_dispatch(
				Origin::root(),
				DispatchTime::At(SevenDays::get() + 4),
				0,
				true,
				Box::new(call.clone())
			));
			let event = Event::orml_authority(orml_authority::Event::Scheduled(
				OriginCaller::orml_authority(DelayedOrigin {
					delay: 1,
					origin: Box::new(OriginCaller::system(RawOrigin::Root)),
				}),
				5,
			));
			assert_eq!(last_event(), event);

			let schedule_origin = {
				let origin: <Runtime as orml_authority::Config>::Origin = From::from(Origin::root());
				let origin: <Runtime as orml_authority::Config>::Origin = From::from(DelayedOrigin::<
					BlockNumber,
					<Runtime as orml_authority::Config>::PalletsOrigin,
				> {
					delay: 1,
					origin: Box::new(origin.caller().clone()),
				});
				origin
			};

			let pallets_origin = schedule_origin.caller().clone();
			assert_ok!(AuthorityModule::cancel_scheduled_dispatch(
				Origin::root(),
				pallets_origin,
				5
			));
			let event = Event::orml_authority(orml_authority::Event::Cancelled(
				OriginCaller::orml_authority(DelayedOrigin {
					delay: 1,
					origin: Box::new(OriginCaller::system(RawOrigin::Root)),
				}),
				5,
			));
			assert_eq!(last_event(), event);

			assert_ok!(AuthorityModule::schedule_dispatch(
				Origin::root(),
				DispatchTime::At(SevenDays::get() + 5),
				0,
				false,
				Box::new(call.clone())
			));
			let event = Event::orml_authority(orml_authority::Event::Scheduled(
				OriginCaller::system(RawOrigin::Root),
				6,
			));
			assert_eq!(last_event(), event);

			assert_ok!(AuthorityModule::cancel_scheduled_dispatch(
				Origin::root(),
				frame_system::RawOrigin::Root.into(),
				6
			));
			let event = Event::orml_authority(orml_authority::Event::Cancelled(
				OriginCaller::system(RawOrigin::Root),
				6,
			));
			assert_eq!(last_event(), event);
		});
}

#[test]
fn test_nft_module() {
	ExtBuilder::default()
		.balances(vec![(AccountId::from(ALICE), DNAR, 1_000 * dollar(DNAR))])
		.build()
		.execute_with(|| {
			assert_eq!(Balances::free_balance(AccountId::from(ALICE)), 1_000 * dollar(DNAR));
			assert_ok!(NFT::create_class(
				origin_of(AccountId::from(ALICE)),
				vec![1],
				setheum_nft::Properties(setheum_nft::ClassProperty::Transferable | setheum_nft::ClassProperty::Burnable)
			));
			assert_eq!(
				Balances::deposit_into_existing(&NftModuleId::get().into_sub_account(0), 1 * CreateTokenDeposit::get())
					.is_ok(),
				true
			);
			assert_ok!(NFT::mint(
				origin_of(NftModuleId::get().into_sub_account(0)),
				MultiAddress::Id(AccountId::from(BOB)),
				0,
				vec![1],
				1
			));
			assert_ok!(NFT::burn(origin_of(AccountId::from(BOB)), (0, 0)));
			assert_eq!(Balances::free_balance(AccountId::from(BOB)), CreateTokenDeposit::get());
			assert_ok!(NFT::destroy_class(
				origin_of(NftModuleId::get().into_sub_account(0)),
				0,
				MultiAddress::Id(AccountId::from(BOB))
			));
			assert_eq!(
				Balances::free_balance(AccountId::from(BOB)),
				CreateClassDeposit::get() + CreateTokenDeposit::get()
			);
			assert_eq!(Balances::reserved_balance(AccountId::from(BOB)), 0);
			assert_eq!(
				Balances::free_balance(AccountId::from(ALICE)),
				1_000 * dollar(DNAR) - (CreateClassDeposit::get() + Proxy::deposit(1u32))
			);
		});
}

// #[cfg(not(feature = "standalone"))]
// mod parachain_tests {
// 	use super::*;

// 	use codec::Encode;
// 	use cumulus_primitives_core::{
// 		DownwardMessageHandler, HrmpMessageHandler, InboundDownwardMessage, InboundHrmpMessage,
// 	};
// 	use polkadot_parachain::primitives::Sibling;
// 	use xcm::{
// 		v0::{Junction, MultiAsset, MultiLocation, NetworkId, Order, Xcm},
// 		VersionedXcm,
// 	};

// 	use newrome_runtime::{Tokens, XcmHandler, PLM};

// 	#[test]
// 	fn receive_cross_chain_assets() {
// 		ExtBuilder::default().build().execute_with(|| {
// 			let dot_amount = 1000 * dollar(DOT);

// 			// receive relay chain token
// 			let msg: VersionedXcm = Xcm::ReserveAssetDeposit {
// 				assets: vec![MultiAsset::ConcreteFungible {
// 					id: MultiLocation::X1(Junction::Parent),
// 					amount: dot_amount,
// 				}],
// 				effects: vec![Order::DepositAsset {
// 					assets: vec![MultiAsset::All],
// 					dest: MultiLocation::X1(Junction::AccountId32 {
// 						network: NetworkId::Named("setheum".into()),
// 						id: ALICE,
// 					}),
// 				}],
// 			}
// 			.into();
// 			XcmHandler::handle_downward_message(InboundDownwardMessage {
// 				sent_at: 10,
// 				msg: msg.encode(),
// 			});
// 			assert_eq!(Tokens::free_balance(DOT, &ALICE.into()), dot_amount);

// 			let sibling_para_id = 5000;
// 			let sibling_parachain_acc: AccountId = Sibling::from(sibling_para_id).into_account();

// 			// receive owned token
// 			let dnr_amount = 1000 * dollar(DNAR);
// 			assert_ok!(Currencies::deposit(DNAR, &sibling_parachain_acc, 1100 * dollar(DNAR)));

// 			let msg1: VersionedXcm = Xcm::WithdrawAsset {
// 				assets: vec![MultiAsset::ConcreteFungible {
// 					id: MultiLocation::X1(Junction::GeneralKey("DNAR".into())),
// 					amount: dnr_amount,
// 				}],
// 				effects: vec![Order::DepositAsset {
// 					assets: vec![MultiAsset::All],
// 					dest: MultiLocation::X1(Junction::AccountId32 {
// 						network: NetworkId::Named("setheum".into()),
// 						id: ALICE,
// 					}),
// 				}],
// 			}
// 			.into();
// 			XcmHandler::handle_hrmp_message(
// 				sibling_para_id.into(),
// 				InboundHrmpMessage {
// 					sent_at: 10,
// 					data: msg1.encode(),
// 				},
// 			);
// 			assert_eq!(Currencies::free_balance(DNAR, &sibling_parachain_acc), 100 * dollar(DNAR));
// 			assert_eq!(Currencies::free_balance(DNAR, &ALICE.into()), dnr_amount);

// 			// receive non-owned token
// 			let plm_amount = 1000 * dollar(PLM);
// 			let msg2: VersionedXcm = Xcm::ReserveAssetDeposit {
// 				assets: vec![MultiAsset::ConcreteFungible {
// 					id: MultiLocation::X1(Junction::GeneralKey("PLM".into())),
// 					amount: plm_amount,
// 				}],
// 				effects: vec![Order::DepositAsset {
// 					assets: vec![MultiAsset::All],
// 					dest: MultiLocation::X1(Junction::AccountId32 {
// 						network: NetworkId::Named("setheum".into()),
// 						id: ALICE,
// 					}),
// 				}],
// 			}
// 			.into();
// 			XcmHandler::handle_hrmp_message(
// 				sibling_para_id.into(),
// 				InboundHrmpMessage {
// 					sent_at: 10,
// 					data: msg2.encode(),
// 				},
// 			);
// 			assert_eq!(Currencies::free_balance(PLM, &ALICE.into()), plm_amount);
// 		});
// 	}
// }
