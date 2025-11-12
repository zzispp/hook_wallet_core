use std::{collections::HashSet, error::Error};

use serde::{Deserialize, Serialize};
use typeshare::typeshare;

use crate::{AssetBasic, AssetProperties, AssetScore, Chain, asset_id::AssetId, asset_type::AssetType};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[typeshare(swift = "Equatable, Hashable, Sendable")]
#[serde(rename_all = "camelCase")]
pub struct Asset {
    pub id: AssetId,
    #[typeshare(skip)]
    pub chain: Chain,
    #[typeshare(skip)]
    pub token_id: Option<String>,
    pub name: String,
    pub symbol: String,
    pub decimals: i32,
    #[serde(rename = "type")]
    pub asset_type: AssetType,
}

impl Chain {
    pub fn new_asset(&self, name: String, symbol: String, decimals: i32, asset_type: AssetType) -> Asset {
        Asset {
            id: self.as_asset_id(),
            chain: *self,
            token_id: None,
            name,
            symbol,
            decimals,
            asset_type,
        }
    }
}

impl Asset {
    pub fn new(id: AssetId, name: String, symbol: String, decimals: i32, asset_type: AssetType) -> Asset {
        Asset {
            id: id.clone(),
            chain: id.chain,
            token_id: id.token_id.clone(),
            name,
            symbol,
            decimals,
            asset_type,
        }
    }

    pub fn chain(&self) -> Chain {
        self.id.chain
    }

    pub fn full_name(&self) -> String {
        format!("{} ({})", self.name, self.symbol)
    }

    pub fn as_basic_primitive(&self) -> AssetBasic {
        AssetBasic::new(self.clone(), AssetProperties::default(self.id.clone()), AssetScore::default())
    }

    pub fn from_chain(chain: Chain) -> Asset {
        match chain {
            Chain::Ethereum => chain.new_asset("Ethereum".to_string(), "ETH".to_string(), 18, AssetType::NATIVE),
            Chain::SmartChain => chain.new_asset("BNB Chain".to_string(), "BNB".to_string(), 18, AssetType::NATIVE),
            Chain::Polygon => chain.new_asset("Polygon".to_string(), "POL".to_string(), 18, AssetType::NATIVE),
            Chain::Solana => chain.new_asset("Solana".to_string(), "SOL".to_string(), 9, AssetType::NATIVE),
            Chain::Arbitrum => chain.new_asset("Arbitrum ETH".to_string(), "ETH".to_string(), 18, AssetType::NATIVE),
        }
    }
}

pub trait AssetVecExt {
    fn ids(&self) -> Vec<AssetId>;
    fn ids_set(&self) -> HashSet<AssetId>;
    fn asset(&self, asset_id: AssetId) -> Option<Asset>;
    fn asset_result(&self, asset_id: AssetId) -> Result<&Asset, Box<dyn Error + Send + Sync>>;
}

impl AssetVecExt for Vec<Asset> {
    fn ids(&self) -> Vec<AssetId> {
        self.iter().map(|x| x.id.clone()).collect()
    }

    fn ids_set(&self) -> HashSet<AssetId> {
        self.iter().map(|x| x.id.clone()).collect()
    }

    fn asset(&self, asset_id: AssetId) -> Option<Asset> {
        self.iter().find(|x| x.id == asset_id).cloned()
    }

    fn asset_result(&self, asset_id: AssetId) -> Result<&Asset, Box<dyn Error + Send + Sync>> {
        self.iter().find(|x| x.id == asset_id).ok_or("Asset not found".into())
    }
}

pub trait AssetHashSetExt {
    fn ids(&self) -> Vec<String>;
}

impl AssetHashSetExt for HashSet<AssetId> {
    fn ids(&self) -> Vec<String> {
        self.iter().cloned().map(|x| x.to_string()).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_asset_id() {
        let asset = Asset::from_chain(Chain::Ethereum);

        assert_eq!(asset.symbol, "ETH");
    }
}
