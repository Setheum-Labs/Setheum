use std::sync::Arc;

use codec::Codec;
use jsonrpc_core::{Error as RpcError, ErrorCode, Result};
use jsonrpc_derive::rpc;
use sp_api::ProvideRuntimeApi;
use sp_blockchain::HeaderBackend;
use sp_runtime::{generic::BlockId, traits::Block as BlockT};

pub use self::gen_client::Client as LaunchpadCrowdsalesClient;
pub use launchpad_crowdsales_rpc_runtime_api::LaunchpadCrowdsalesApi as LaunchpadCrowdsalesRuntimeApi;

#[rpc]
pub trait LaunchpadCrowdsalesApi<BlockHash, ResponseType> {
	#[rpc(name = "launchpad_getProposal")]
	fn get_proposal(&self, campaign_id: CampaignId, at: Option<BlockHash>) -> Result<ResponseType>;
	#[rpc(name = "launchpad_getAllProposals")]
	fn get_all_proposals(&self, at: Option<BlockHash>) -> Result<Vec<(Key, Option<Value>)>>;
	#[rpc(name = "launchpad_getCampaign")]
	fn get_campaign(&self, campaign_id: CampaignId, at: Option<BlockHash>) -> Result<Vec<(Key, Option<Value>)>>;
	#[rpc(name = "launchpad_getAllCampaigns")]
	fn get_all_campaigns(&self, at: Option<BlockHash>) -> Result<Vec<(Key, Option<Value>)>>;
	#[rpc(name = "launchpad_getTotalAmountsRaised")]
	fn get_total_amounts_raised(&self, at: Option<BlockHash>) -> Result<Vec<(Key, Option<Value>)>>;
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

impl<C, Block, AccountId, Balance, BlockNumber, CurrencyId> LaunchpadCrowdsalesApi<<Block as BlockT>::Hash, AccountId, Balance, BlockNumber, CurrencyId> for LaunchpadCrowdsales<C, Block>
where
	Block: BlockT,
	C: Send + Sync + 'static + ProvideRuntimeApi<Block> + HeaderBackend<Block>,
	C::Api: LaunchpadCrowdsalesRuntimeApi<Block, AccountId, Balance, BlockNumber, CurrencyId>,
	AccountId: Clone + Codec + MaybeDisplay,
	Balance: Clone + Codec + MaybeDisplay,
	BlockNumber: Clone + Codec + MaybeDisplay,
	CurrencyId: Clone + Codec + MaybeDisplay,

{	
	// Get the campaign info for a given proposal's campaign id.
	fn get_proposal(
		&self,
		campaign_id: CampaignId,
		at: Option<<Block as BlockT>::Hash>,
	) -> Result<Option<CampaignInfo>> {
		let api = self.client.runtime_api();
		let at = BlockId::hash(at.unwrap_or(
			// If the block hash is not supplied assume the best block.
			self.client.info().best_hash,
		));
		api.get_proposal(&at, campaign_id).map_err(|e| RpcError {
			code: ErrorCode::ServerError(Error::RuntimeError.into()),
			message: "Unable to get value.".into(),
			data: Some(format!("{:?}", e).into()),
		})
	}

	// Get all the proposals.
	fn get_all_proposals(
		&self,
		at: Option<<Block as BlockT>::Hash>,
	) -> Result<Vec<CampaignInfo<AccountId, Balance, BlockNumber>>> {
		let api = self.client.runtime_api();
		let at = BlockId::hash(at.unwrap_or(
			// If the block hash is not supplied assume the best block.
			self.client.info().best_hash,
		));
		api.get_all_proposals(&at).map_err(|e| RpcError {
			code: ErrorCode::ServerError(Error::RuntimeError.into()),
			message: "Unable to get all proposals.".into(),
			data: Some(format!("{:?}", e).into()),
		})
	}

	// Get the campaign info for a given campaign's campaign id.
	fn get_campaign(
		&self,
		campaign_id: CampaignId,
		at: Option<<Block as BlockT>::Hash>,
	) -> Result<Option<CampaignInfo>> {
		let api = self.client.runtime_api();
		let at = BlockId::hash(at.unwrap_or(
			// If the block hash is not supplied assume the best block.
			self.client.info().best_hash,
		));
		api.get_campaign(&at, campaign_id).map_err(|e| RpcError {
			code: ErrorCode::ServerError(Error::RuntimeError.into()),
			message: "Unable to get campaign.".into(),
			data: Some(format!("{:?}", e).into()),
		})
	}

	// Get all the campaigns.
	fn get_all_campaigns(
		&self,
		at: Option<<Block as BlockT>::Hash>,
	) -> Result<Vec<CampaignInfo<AccountId, Balance, BlockNumber>>> {
		let api = self.client.runtime_api();
		let at = BlockId::hash(at.unwrap_or(
			// If the block hash is not supplied assume the best block.
			self.client.info().best_hash,
		));
		api.get_all_campaigns(&at).map_err(|e| RpcError {
			code: ErrorCode::ServerError(Error::RuntimeError.into()),
			message: "Unable to get all campaigns.".into(),
			data: Some(format!("{:?}", e).into()),
		})
	}

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
