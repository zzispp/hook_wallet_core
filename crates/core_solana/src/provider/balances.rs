use async_trait::async_trait;
use core_chain_traits::ChainBalances;
use std::error::Error;

use crate::provider::balances_mapper::{map_balance_staking, map_coin_balance, map_token_accounts};
use crate::rpc::client::SolanaClient;
use core_client::Client;
use primitives::{AssetBalance,AssetId};

#[async_trait]
impl<C: Client + Clone> ChainBalances for SolanaClient<C> {
    async fn get_balance_coin(&self, address: String) -> Result<AssetBalance, Box<dyn Error + Sync + Send>> {
        let balance = self.get_balance(&address).await?;
        Ok(map_coin_balance(&balance))
    }

    async fn get_balance_tokens(&self, address: String, token_ids: Vec<String>) -> Result<Vec<AssetBalance>, Box<dyn Error + Sync + Send>> {
        let results = self.get_token_accounts(&address, &token_ids).await?;
        let balances: Vec<AssetBalance> = results
            .iter()
            .zip(&token_ids)
            .flat_map(|(token_accounts, token_id)| map_token_accounts(token_accounts, token_id))
            .collect();

        Ok(balances)
    }

    async fn get_balance_staking(&self, address: String) -> Result<Option<AssetBalance>, Box<dyn Error + Sync + Send>> {
        let accounts = self.get_staking_balance(&address).await?;
        Ok(map_balance_staking(accounts))
    }

    async fn get_balance_assets(&self, address: String) -> Result<Vec<AssetBalance>, Box<dyn Error + Send + Sync>> {
        let token_accounts_result = self.get_token_accounts_by_owner(&address, crate::TOKEN_PROGRAM).await?;
        let balances: Vec<AssetBalance> = token_accounts_result
            .value
            .into_iter()
            .filter_map(|account| {
                let token_info = &account.account.data.parsed.info;
                if let (Some(token_amount), Some(mint)) = (&token_info.token_amount, &token_info.mint)
                    && token_amount.amount > num_bigint::BigUint::from(0u32)
                {
                    let asset_id = AssetId {
                        chain: primitives::Chain::Solana,
                        token_id: Some(mint.clone()),
                    };
                    return Some(primitives::AssetBalance::new(asset_id, token_amount.amount.clone()));
                }
                None
            })
            .collect();

        Ok(balances)
    }
}
