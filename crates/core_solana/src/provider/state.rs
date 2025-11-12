use async_trait::async_trait;
use core_chain_traits::ChainState;
use std::error::Error;

use core_client::Client;
use primitives::NodeSyncStatus;
use crate::provider::state_mapper;
use crate::rpc::client::SolanaClient;

#[async_trait]
impl<C: Client + Clone> ChainState for SolanaClient<C> {
    async fn get_chain_id(&self) -> Result<String, Box<dyn Error + Sync + Send>> {
        Ok(self.get_genesis_hash().await?)
    }

    async fn get_node_status(&self) -> Result<NodeSyncStatus, Box<dyn Error + Sync + Send>> {
        let slot = self.get_slot().await?;
        state_mapper::map_node_status(slot)
    }

    async fn get_block_latest_number(&self) -> Result<u64, Box<dyn Error + Sync + Send>> {
        Ok(self.get_slot().await?)
    }
}

