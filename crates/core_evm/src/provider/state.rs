use std::error::Error;

use async_trait::async_trait;
use core_chain_traits::ChainState;

use crate::provider::state_mapper;
use crate::rpc::client::EthereumClient;
use core_client::Client;
use primitives::NodeSyncStatus;

#[async_trait]
impl<C: Client + Clone> ChainState for EthereumClient<C> {
    async fn get_chain_id(&self) -> Result<String, Box<dyn Error + Sync + Send>> {
        let chain_id = EthereumClient::get_chain_id(self).await?;
        Ok(u64::from_str_radix(chain_id.trim_start_matches("0x"), 16)?.to_string())
    }

    async fn get_node_status(&self) -> Result<NodeSyncStatus, Box<dyn Error + Sync + Send>> {
        let sync_status = self.get_sync_status().await?;
        let latest_block = self.get_block_latest_number().await?;
        state_mapper::map_node_status(&sync_status, latest_block)
    }

    async fn get_block_latest_number(&self) -> Result<u64, Box<dyn Error + Sync + Send>> {
        let block_number = self.get_latest_block().await?;
        Ok(block_number)
    }
}
