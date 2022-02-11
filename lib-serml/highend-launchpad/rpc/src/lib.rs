use std::sync::Arc;

use codec::Codec;
use jsonrpc_core::{Error as RpcError, ErrorCode, Result};
use jsonrpc_derive::rpc;
use sp_api::ProvideRuntimeApi;
use sp_blockchain::HeaderBackend;
use sp_runtime::{generic::BlockId, traits::Block as BlockT, MaybeDisplay};

pub use self::gen_client::Client as LaunchpadCrowdsalesClient;
pub use highend_launchpad_rpc_runtime_api::LaunchpadCrowdsalesApi as LaunchpadCrowdsalesRuntimeApi;

#[rpc]
pub trait LaunchpadCrowdsalesApi<BlockHash, Balance, BlockNumber, CurrencyId> {
	#[rpc(name = "launchpad_getTotalAmountsRaised")]
	fn get_total_amounts_raised(&self, at: Option<BlockHash>) -> Result<Vec<(CurrencyId, Balance)>>;
}

/// A struct that implements the [`LaunchpadCrowdsalesApi`].
pub struct LaunchpadCrowdsales<C, B> {
	client: Arc<C>,
	_marker: std::marker::PhantomData<B>,
}

impl<C, B> LaunchpadCrowdsales<C, B> {
	/// Create new `LaunchpadCrowdsales` with the given reference to the client.
	pub fn new(client: Arc<C>) -> Self {
		LaunchpadCrowdsales {
			client,
			_marker: Default::default(),
		}
	}
}

pub enum Error {
	RuntimeError,
}

impl From<Error> for i64 {
	fn from(e: Error) -> i64 {
		match e {
			Error::RuntimeError => 1,
		}
	}
}

impl<C, Block, Balance, BlockNumber, CurrencyId> LaunchpadCrowdsalesApi<<Block as BlockT>::Hash, Balance, BlockNumber, CurrencyId> for LaunchpadCrowdsales<C, Block>
where
	Block: BlockT,
	C: Send + Sync + 'static + ProvideRuntimeApi<Block> + HeaderBackend<Block>,
	C::Api: LaunchpadCrowdsalesRuntimeApi<Block, Balance, BlockNumber, CurrencyId>,
	Balance: Clone + Codec + MaybeDisplay,
	BlockNumber: Clone + Codec + MaybeDisplay,
	CurrencyId: Clone + Codec + MaybeDisplay,

{	
	// Get the total amount of funds raised in the entire protocol.
	fn get_total_amounts_raised(
		&self,
		at: Option<<Block as BlockT>::Hash>,
	) -> Result<Vec<(CurrencyId, Balance)>> {
		let api = self.client.runtime_api();
		let at = BlockId::hash(at.unwrap_or(
			// If the block hash is not supplied assume the best block.
			self.client.info().best_hash,
		));
		api.get_total_amounts_raised(&at).map_err(|e| RpcError {
			code: ErrorCode::ServerError(Error::RuntimeError.into()),
			message: "Unable to get total amounts raised.".into(),
			data: Some(format!("{:?}", e).into()),
		})
	}
}
