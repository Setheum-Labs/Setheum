#![cfg(test)]

// use super::*;
// use crate as pallet_atomic_swap;

// use frame_support::{construct_runtime, parameter_types, PalletId};
// use sp_core::H256;
// use sp_runtime::{
// 	testing::Header,
// 	traits::{BlakeTwo256, IdentityLookup},
// };
// use orml_traits::parameter_type_with_key;
// use primitives::{Amount, TokenSymbol};

// Currencies constants - CurrencyId/TokenSymbol
// pub const SETM: CurrencyId = CurrencyId::Token(TokenSymbol::SETM);
// pub const SETR: CurrencyId = CurrencyId::Token(TokenSymbol::SETR);
// pub const SETUSD: CurrencyId = CurrencyId::Token(TokenSymbol::SETUSD);

// parameter_types! {
// 	pub BlockWeights: frame_system::limits::BlockWeights =
// 		frame_system::limits::BlockWeights::simple_max(1024);
// }

// parameter_types! {
// 	pub const BlockHashCount: u64 = 250;
// }

// impl frame_system::Config for Test {
// 	type BaseCallFilter = frame_support::traits::Everything;
// 	type BlockWeights = ();
// 	type BlockLength = ();
// 	type DbWeight = ();
// 	type Origin = Origin;
// 	type Index = u64;
// 	type BlockNumber = u64;
// 	type Hash = H256;
// 	type Call = Call;
// 	type Hashing = BlakeTwo256;
// 	type AccountId = u64;
// 	type Lookup = IdentityLookup<Self::AccountId>;
// 	type Header = Header;
// 	type Event = Event;
// 	type BlockHashCount = BlockHashCount;
// 	type Version = ();
// 	type PalletInfo = PalletInfo;
// 	type AccountData = pallet_balances::AccountData<u64>;
// 	type OnNewAccount = ();
// 	type OnKilledAccount = ();
// 	type SystemWeightInfo = ();
// 	type SS58Prefix = ();
// 	type OnSetCode = ();
// }

// parameter_types! {
// 	pub const ExistentialDeposit: Balance = 1;
// 	pub const ProofLimit: u32 = 1024;
// }

// impl pallet_balances::Config for Test {
// 	type MaxLocks = ();
// 	type MaxReserves = ();
// 	type ReserveIdentifier = [u8; 8];
// 	type Balance = u64;
// 	type DustRemoval = ();
// 	type Event = Event;
// 	type ExistentialDeposit = ExistentialDeposit;
// 	type AccountStore = System;
// 	type WeightInfo = ();
// }

// parameter_type_with_key! {
// 	pub ExistentialDeposits: |_currency_id: CurrencyId| -> Balance {
// 		Default::default()
// 	};
// }

// impl orml_tokens::Config for Test {
// 	type Event = Event;
// 	type Balance = Balance;
// 	type Amount = Amount;
// 	type CurrencyId = CurrencyId;
// 	type WeightInfo = ();
// 	type ExistentialDeposits = ExistentialDeposits;
// 	type OnDust = ();
// 	type MaxLocks = ();
// 	type DustRemovalWhitelist = ();
// }

// impl Config for Test {
// 	type Event = Event;
// 	type MultiCurrency = Tokens;
// 	type SwapAction = MultiCurrencySwapAction<u64, Balances>;
// 	type ProofLimit = ProofLimit;
// }

// type UncheckedExtrinsic = frame_system::mocking::MockUncheckedExtrinsic<Test>;
// type Block = frame_system::mocking::MockBlock<Test>;

// construct_runtime!(
// 	pub enum Test where
// 		Block = Block,
// 		NodeBlock = Block,
// 		UncheckedExtrinsic = UncheckedExtrinsic,
// 	{
// 		System: frame_system::{Pallet, Call, Config, Storage, Event<T>},
// 		Balances: pallet_balances::{Pallet, Call, Storage, Config<T>, Event<T>},
// 		Tokens: orml_tokens::{Pallet, Storage, Call, Event<T>},
// 		AtomicSwap: pallet_atomic_swap::{Pallet, Call, Event<T>},
// 	}
// );

// const A: u64 = 1;
// const B: u64 = 2;

// pub fn new_test_ext() -> sp_io::TestExternalities {
// 	let mut t = frame_system::GenesisConfig::default().build_storage::<Test>().unwrap();
// 	let genesis = pallet_balances::GenesisConfig::<Test> { balances: vec![(A, 100), (B, 200)] };
// 	genesis.assimilate_storage(&mut t).unwrap();
// 	t.into()
// }

// #[test]
// fn two_party_successful_swap() {
// 	let mut chain1 = new_test_ext();
// 	let mut chain2 = new_test_ext();

// 	// A generates a random proof. Keep it secret.
// 	let proof: [u8; 2] = [4, 2];
// 	// The hashed proof is the blake2_256 hash of the proof. This is public.
// 	let hashed_proof = blake2_256(&proof);

// 	// A creates the swap on chain1.
// 	chain1.execute_with(|| {
// 		AtomicSwap::create_swap(
// 			Origin::signed(A),
// 			B,
// 			hashed_proof.clone(),
// 			MultiCurrencySwapAction::new(50),
// 			1000,
// 		)
// 		.unwrap();

// 		assert_eq!(Balances::free_balance(A), 100 - 50);
// 		assert_eq!(Balances::free_balance(B), 200);
// 	});

// 	// B creates the swap on chain2.
// 	chain2.execute_with(|| {
// 		AtomicSwap::create_swap(
// 			Origin::signed(B),
// 			A,
// 			hashed_proof.clone(),
// 			MultiCurrencySwapAction::new(75),
// 			1000,
// 		)
// 		.unwrap();

// 		assert_eq!(Balances::free_balance(A), 100);
// 		assert_eq!(Balances::free_balance(B), 200 - 75);
// 	});

// 	// A reveals the proof and claims the swap on chain2.
// 	chain2.execute_with(|| {
// 		AtomicSwap::claim_swap(Origin::signed(A), proof.to_vec(), MultiCurrencySwapAction::new(75))
// 			.unwrap();

// 		assert_eq!(Balances::free_balance(A), 100 + 75);
// 		assert_eq!(Balances::free_balance(B), 200 - 75);
// 	});

// 	// B use the revealed proof to claim the swap on chain1.
// 	chain1.execute_with(|| {
// 		AtomicSwap::claim_swap(Origin::signed(B), proof.to_vec(), MultiCurrencySwapAction::new(50))
// 			.unwrap();

// 		assert_eq!(Balances::free_balance(A), 100 - 50);
// 		assert_eq!(Balances::free_balance(B), 200 + 50);
// 	});
// }
