use std::error::Error;

use async_trait::async_trait;
use core_chain_traits::ChainBalances;
use primitives::{AssetBalance, EVMChain};

use crate::provider::balances_mapper::{map_assets_balances, map_balance_coin, map_balance_tokens};
use crate::rpc::client::EthereumClient;
use core_client::Client;

#[async_trait]
impl<C: Client + Clone> ChainBalances for EthereumClient<C> {
    async fn get_balance_coin(
        &self,
        address: String,
    ) -> Result<AssetBalance, Box<dyn Error + Sync + Send>> {
        map_balance_coin(self.get_eth_balance(&address).await?, self.get_chain())
    }

    async fn get_balance_tokens(
        &self,
        address: String,
        token_ids: Vec<String>,
    ) -> Result<Vec<AssetBalance>, Box<dyn Error + Sync + Send>> {
        let balance_results = self.batch_token_balance_calls(&address, &token_ids).await?;
        map_balance_tokens(balance_results, token_ids, self.get_chain())
    }

    async fn get_balance_staking(
        &self,
        address: String,
    ) -> Result<Option<AssetBalance>, Box<dyn Error + Sync + Send>> {
        match self.chain {
            EVMChain::Ethereum => self.get_ethereum_staking_balance(&address).await,
            EVMChain::SmartChain => self.get_smartchain_staking_balance(&address).await,
            _ => Ok(None),
        }
    }

    async fn get_balance_assets(
        &self,
        address: String,
    ) -> Result<Vec<AssetBalance>, Box<dyn Error + Send + Sync>> {
        if let Some(ankr_client) = &self.ankr_client {
            let balances = ankr_client
                .get_token_balances(address.as_str())
                .await?
                .assets
                .into_iter()
                .filter_map(|asset| {
                    asset
                        .contract_address
                        .map(|addr| (addr, asset.balance_raw_integer))
                })
                .collect();
            return Ok(map_assets_balances(balances, self.get_chain()));
        }
        Ok(vec![])
    }
}
