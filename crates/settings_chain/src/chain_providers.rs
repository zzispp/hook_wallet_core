use std::error::Error;

use core_chain_traits::ChainTraits;
use primitives::{AssetBalance, Chain};
use settings::Settings;

use crate::ProviderFactory;

pub struct ChainProviders {
    providers: Vec<Box<dyn ChainTraits>>,
}

impl ChainProviders {
    pub fn new(providers: Vec<Box<dyn ChainTraits>>) -> Self {
        Self { providers }
    }

    pub fn from_settings(settings: &Settings, service_name: &str) -> Self {
        Self::new(ProviderFactory::new_providers_with_user_agent(
            settings,
            service_name,
        ))
    }

    fn get_provider(&self, chain: Chain) -> Result<&dyn ChainTraits, Box<dyn Error + Send + Sync>> {
        tracing::debug!(
            "Looking for provider for chain: {:?}, available providers: {}",
            chain,
            self.providers.len()
        );

        let provider = self
            .providers
            .iter()
            .find(|x| {
                let provider_chain = x.get_chain();
                tracing::debug!("Checking provider with chain: {:?}", provider_chain);
                provider_chain == chain
            })
            .map(|provider| provider.as_ref())
            .ok_or_else(|| -> Box<dyn Error + Send + Sync> {
                format!("Provider for chain {} not found", chain.as_ref()).into()
            })?;

        tracing::info!("Found provider for chain: {:?}", chain);
        Ok(provider)
    }

    pub async fn get_balance_coin(
        &self,
        chain: Chain,
        address: String,
    ) -> Result<AssetBalance, Box<dyn Error + Send + Sync>> {
        self.get_provider(chain)?.get_balance_coin(address).await
    }

    pub async fn get_balance_tokens(
        &self,
        chain: Chain,
        address: String,
        token_ids: Vec<String>,
    ) -> Result<Vec<AssetBalance>, Box<dyn Error + Send + Sync>> {
        self.get_provider(chain)?
            .get_balance_tokens(address, token_ids)
            .await
    }

    pub async fn get_balance_assets(
        &self,
        chain: Chain,
        address: String,
    ) -> Result<Vec<AssetBalance>, Box<dyn Error + Send + Sync>> {
        self.get_provider(chain)?.get_balance_assets(address).await
    }

    pub async fn get_balance_staking(
        &self,
        chain: Chain,
        address: String,
    ) -> Result<Option<AssetBalance>, Box<dyn Error + Send + Sync>> {
        self.get_provider(chain)?.get_balance_staking(address).await
    }
}
