use std::error::Error;

use primitives::{AssetBalance, ChainAddress};
use settings_chain::ChainProviders;

pub struct ChainClient {
    providers: ChainProviders,
}

impl ChainClient {
    pub fn new(providers: ChainProviders) -> Self {
        Self { providers }
    }

    pub async fn get_balances_coin(
        &self,
        request: ChainAddress,
    ) -> Result<AssetBalance, Box<dyn Error + Send + Sync>> {
        self.providers
            .get_balance_coin(request.chain, request.address)
            .await
    }

    pub async fn get_balances_staking(
        &self,
        request: ChainAddress,
    ) -> Result<Option<AssetBalance>, Box<dyn Error + Send + Sync>> {
        self.providers
            .get_balance_staking(request.chain, request.address)
            .await
    }

    pub async fn get_balances_assets(
        &self,
        request: ChainAddress,
    ) -> Result<Vec<AssetBalance>, Box<dyn Error + Send + Sync>> {
        self.providers
            .get_balance_assets(request.chain, request.address)
            .await
    }
}
