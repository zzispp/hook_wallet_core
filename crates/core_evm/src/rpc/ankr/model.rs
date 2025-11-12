use num_bigint::BigUint;
use primitives::EVMChain;
use serde::Deserialize;
use serde_serializers::{deserialize_biguint_from_hex_str, deserialize_biguint_from_str};

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Transaction {
    pub hash: String,
    #[serde(deserialize_with = "deserialize_biguint_from_hex_str")]
    pub timestamp: BigUint,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Transactions {
    pub transactions: Vec<Transaction>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TokenBalances {
    pub assets: Vec<TokenBalance>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TokenBalance {
    pub contract_address: Option<String>,
    #[serde(deserialize_with = "deserialize_biguint_from_str")]
    pub balance_raw_integer: BigUint,
}

pub fn ankr_chain(chain: EVMChain) -> Option<String> {
    match chain {
        EVMChain::Ethereum => Some("eth".to_string()),
        EVMChain::Polygon => Some("polygon".to_string()),
        EVMChain::SmartChain => Some("bsc".to_string()),
        EVMChain::Arbitrum => Some("arbitrum".to_string()),
    }
}
