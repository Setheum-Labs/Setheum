//! RPC interface for the transaction payment module.

use jsonrpc_core::{Error as RpcError, ErrorCode, Result};
use jsonrpc_derive::rpc;
use sp_api::ProvideRuntimeApi;
use sp_blockchain::HeaderBackend;
use sp_runtime::{generic::BlockId, traits::{Block as BlockT, MaybeDisplay}};
use std::sync::Arc;
use open_grant_runtime_api::OpenGrantApi as OpenGrantRuntimeApi;
use pallet_open_grant::Project;
use codec::{Codec};
use sp_std::prelude::*;

#[rpc]
pub trait OpenGrantApi<BlockHash, ResponseType> {
	#[rpc(name = "openGrant_getProjects")]
	fn get_projects(&self, at: Option<BlockHash>) -> Result<ResponseType>;
}

/// A struct that implements the `OpenGrantApi`.
pub struct OpenGrant<C, M> {
	// If you have more generics, no need to OpenGrant<C, M, N, P, ...>
	// just use a tuple like OpenGrant<C, (M, N, P, ...)>
	client: Arc<C>,
	_marker: std::marker::PhantomData<M>,
}

impl<C, M> OpenGrant<C, M> {
	/// Create new `OpenGrant` instance with the given reference to the client.
	pub fn new(client: Arc<C>) -> Self {
		Self {
			client,
			_marker: Default::default(),
		}
	}
}

impl<C, Block, AccountId, BlockNumber> OpenGrantApi<<Block as BlockT>::Hash, Vec<Project<AccountId, BlockNumber>>> for OpenGrant<C, Block>
where
	Block: BlockT,
	C: Send + Sync + 'static,
	C: ProvideRuntimeApi<Block>,
	C: HeaderBackend<Block>,
	C::Api: OpenGrantRuntimeApi<Block, AccountId, BlockNumber>,
	AccountId: Clone + Codec + MaybeDisplay,
	BlockNumber:  Clone + Codec + MaybeDisplay,
{
	fn get_projects(&self, at: Option<<Block as BlockT>::Hash>) -> Result<Vec<Project<AccountId, BlockNumber>>> {
		let api = self.client.runtime_api();
		let at = BlockId::hash(at.unwrap_or_else(||
			// If the block hash is not supplied assume the best block.
			self.client.info().best_hash));

		let runtime_api_result = api.get_projects(&at);
		runtime_api_result.map_err(|e| RpcError {
			code: ErrorCode::ServerError(9876), // No real reason for this value
			message: "Something wrong".into(),
			data: Some(format!("{:?}", e).into()),
		})
	}
}
