mod chain_providers;
mod provider_config;
pub use chain_providers::ChainProviders;
use core_client::{ReqwestClient, retry_policy};
pub use provider_config::ProviderConfig;
pub use settings::ChainURLType;

use core_chain_traits::ChainTraits;
use core_evm::rpc::ankr::AnkrClient;
use core_evm::rpc::EthereumClient;
use core_jsonrpc::JsonRpcClient;
use core_solana::rpc::client::SolanaClient;

use primitives::{Chain, EVMChain, NodeType};
use settings::Settings;

pub struct ProviderFactory {}

impl ProviderFactory {
    pub fn new_from_settings(chain: Chain, settings: &Settings) -> Box<dyn ChainTraits> {
        Self::new_from_settings_with_user_agent(chain, settings, "")
    }

    pub fn new_from_settings_with_user_agent(
        chain: Chain,
        settings: &Settings,
        user_agent: &str,
    ) -> Box<dyn ChainTraits> {
        let chain_config = Self::get_chain_config(chain, settings);
        let node_type = Self::get_node_type(chain_config.node.clone());

        Self::new_provider(
            ProviderConfig::new(
                chain,
                &chain_config.url,
                node_type,
                settings.ankr.key.secret.as_str(),
                settings.trongrid.key.secret.as_str(),
            ),
            user_agent,
        )
    }

    pub fn new_providers(settings: &Settings) -> Vec<Box<dyn ChainTraits>> {
        Chain::all()
            .iter()
            .map(|x| Self::new_from_settings(*x, &settings.clone()))
            .collect()
    }

    pub fn new_providers_with_user_agent(
        settings: &Settings,
        user_agent: &str,
    ) -> Vec<Box<dyn ChainTraits>> {
        Chain::all()
            .iter()
            .map(|x| Self::new_from_settings_with_user_agent(*x, &settings.clone(), user_agent))
            .collect()
    }

    pub fn new_provider(config: ProviderConfig, user_agent: &str) -> Box<dyn ChainTraits> {
        let host = config
            .url
            .parse::<url::Url>()
            .ok()
            .and_then(|u| u.host_str().map(String::from))
            .unwrap_or_default();

        let retry_policy_config = retry_policy(host, 3);
        let reqwest_client = core_client::builder()
            .retry(retry_policy_config)
            .build()
            .expect("Failed to build reqwest client");

        let chain = config.chain;
        let url = config.url.clone();
        let node_type = config.clone().node_type;
        let gem_client = ReqwestClient::new_with_user_agent(
            url.clone(),
            reqwest_client.clone(),
            user_agent.to_string(),
        );

        match chain {
            Chain::Solana => Box::new(SolanaClient::new(JsonRpcClient::new(gem_client.clone()))),
            Chain::Ethereum | Chain::SmartChain | Chain::Polygon | Chain::Arbitrum => {
                let chain = EVMChain::from_chain(chain).unwrap();
                let client = gem_client.clone();
                let rpc_client = JsonRpcClient::new(client.clone());
                let ethereum_client = EthereumClient::new(rpc_client.clone(), chain)
                    .with_node_type(node_type)
                    .with_ankr_client(AnkrClient::new(
                        JsonRpcClient::new(ReqwestClient::new(config.clone().ankr_url(), reqwest_client.clone())),
                        chain,
                    ));
                Box::new(ethereum_client)
            }
        }
    }

    pub fn get_chain_config(chain: Chain, settings: &Settings) -> &settings::Chain {
        match chain {
            Chain::Ethereum => &settings.chains.ethereum,
            Chain::SmartChain => &settings.chains.smartchain,
            Chain::Solana => &settings.chains.solana,
            Chain::Polygon => &settings.chains.polygon,
            Chain::Arbitrum => &settings.chains.arbitrum,
        }
    }

    pub fn get_node_type(url_type: ChainURLType) -> NodeType {
        match url_type {
            ChainURLType::Default => NodeType::Default,
            ChainURLType::Archival => NodeType::Archival,
        }
    }
}
