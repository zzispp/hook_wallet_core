use std::error::Error;
use core_chain_traits::ChainProvider;
use core_chain_traits::{ChainAccount, ChainAddressStatus, ChainPerpetual, ChainTraits};
use core_client::Client;
use core_jsonrpc::{client::JsonRpcClient as GenericJsonRpcClient, types::JsonRpcError};
use primitives::Chain;
use crate::models::{TokenAccountInfo, ValueResult};
use crate::models::balances::SolanaBalance;

pub struct SolanaClient<C: Client + Clone> {
    client: GenericJsonRpcClient<C>,
    pub chain: Chain,
}

pub fn token_accounts_by_owner_params(owner: &str, program_id: &str) -> serde_json::Value {
    serde_json::json!([
        owner,
        {
            "programId": program_id
        },
        {
            "encoding": "jsonParsed"
        }
    ])
}

pub fn token_accounts_by_mint_params(owner: &str, mint: &str) -> serde_json::Value {
    serde_json::json!([
        owner,
        {
            "mint": mint
        },
        {
            "encoding": "jsonParsed"
        }
    ])
}

impl<C: Client + Clone> SolanaClient<C> {
    pub fn new(client: GenericJsonRpcClient<C>) -> Self {
        Self { client, chain: Chain::Solana }
    }

    pub fn get_client(&self) -> &GenericJsonRpcClient<C> {
        &self.client
    }

    pub fn get_chain(&self) -> Chain {
        self.chain
    }

    pub async fn rpc_call<T>(&self, method: &str, params: serde_json::Value) -> Result<T, JsonRpcError>
    where
        T: serde::de::DeserializeOwned,
    {
        self.client.call(method, params).await
    }

    pub async fn get_balance(&self, address: &str) -> Result<SolanaBalance, JsonRpcError> {
        self.rpc_call("getBalance", serde_json::json!([address])).await
    }


    pub async fn get_staking_balance(&self, address: &str) -> Result<Vec<TokenAccountInfo>, JsonRpcError> {
        let stake_program_id = "Stake11111111111111111111111111111111111111";
        let params = serde_json::json!([
            stake_program_id,
            {
                "encoding": "jsonParsed",
                "filters": [
                    {
                        "memcmp": {
                            "offset": 12,
                            "bytes": address
                        }
                    }
                ]
            }
        ]);

        self.rpc_call("getProgramAccounts", params).await
    }

    pub async fn get_genesis_hash(&self) -> Result<String, JsonRpcError> {
        self.rpc_call("getGenesisHash", serde_json::json!([])).await
    }

    pub async fn get_slot(&self) -> Result<u64, JsonRpcError> {
        self.rpc_call("getSlot", serde_json::json!([])).await
    }

    pub async fn get_token_accounts_by_owner(&self, owner: &str, program_id: &str) -> Result<ValueResult<Vec<TokenAccountInfo>>, JsonRpcError> {
        let params = token_accounts_by_owner_params(owner, program_id);
        self.rpc_call("getTokenAccountsByOwner", params).await
    }

    pub async fn get_token_accounts(
        &self,
        address: &str,
        token_mints: &[String],
    ) -> Result<Vec<ValueResult<Vec<TokenAccountInfo>>>, Box<dyn Error + Send + Sync>> {
        let calls: Vec<(String, serde_json::Value)> = token_mints
            .iter()
            .map(|mint| ("getTokenAccountsByOwner".to_string(), token_accounts_by_mint_params(address, mint)))
            .collect();

        let results = self.get_client().batch_call(calls).await?.extract();
        Ok(results)
    }

}

#[async_trait::async_trait]
impl<C: Client + Clone> ChainAccount for SolanaClient<C> {}

#[async_trait::async_trait]
impl<C: Client + Clone> ChainPerpetual for SolanaClient<C> {}

#[async_trait::async_trait]
impl<C: Client + Clone> ChainAddressStatus for SolanaClient<C> {}

impl<C: Client + Clone> ChainTraits for SolanaClient<C> {}

impl<C: Client + Clone> ChainProvider for SolanaClient<C> {
    fn get_chain(&self) -> Chain {
        Chain::Solana
    }
}
