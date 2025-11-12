use crate::Chain;
use serde::{Deserialize, Serialize};
use std::str::FromStr;
use strum::{AsRefStr, EnumIter, EnumString, IntoEnumIterator};
use typeshare::typeshare;

#[derive(Debug, PartialEq)]
pub enum ChainStack {
    Native,
    Optimism,
    ZkSync,
}

#[derive(Copy, Clone, Debug, Serialize, Deserialize, EnumIter, AsRefStr, EnumString, PartialEq)]
#[typeshare(swift = "Equatable, CaseIterable, Sendable")]
#[serde(rename_all = "lowercase")]
#[strum(serialize_all = "lowercase")]
pub enum EVMChain {
    Ethereum,
    SmartChain,
    Polygon,
    Arbitrum,
}

impl EVMChain {
    pub fn all() -> Vec<Self> {
        Self::iter().collect::<Vec<_>>()
    }

    pub fn min_priority_fee(&self) -> u64 {
        match self {
            Self::Ethereum => 100_000_000,    // https://etherscan.io/gastracker
            Self::SmartChain => 50_000_000,   // https://bscscan.com/gastracker
            Self::Polygon => 30_000_000_000,  // https://polygonscan.com/gastracker
            Self::Arbitrum => 10_000_000,     // https://arbiscan.io/address/0x000000000000000000000000000000000000006C#readContract getMinimumGasPrice
        }
    }

    pub fn chain_stack(&self) -> ChainStack {
        match self {
            Self::Ethereum | Self::SmartChain | Self::Polygon | Self::Arbitrum => ChainStack::Native,
        }
    }

    pub fn is_ethereum_layer2(&self) -> bool {
        matches!(self, Self::Arbitrum)
    }

    // https://docs.optimism.io/stack/getting-started
    pub fn is_opstack(&self) -> bool {
        self.chain_stack() == ChainStack::Optimism
    }

    // https://docs.zksync.io/zk-stack/running/quickstart
    pub fn is_zkstack(&self) -> bool {
        self.chain_stack() == ChainStack::ZkSync
    }

    pub fn weth_contract(&self) -> Option<&str> {
        match self {
            Self::Ethereum => Some("0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2"),
            Self::SmartChain => Some("0xbb4CdB9CBd36B01bD1cBaEBF2De08d9173bc095c"), // WBNB
            Self::Polygon => Some("0x0d500B1d8E8eF31E21C99d1Db9A6444d3ADf1270"),    // WMATIC
            Self::Arbitrum => Some("0x82aF49447D8a07e3bd95BD0d56f35241523fBab1"),
        }
    }

    pub fn from_chain(chain: Chain) -> Option<Self> {
        EVMChain::from_str(chain.as_ref()).ok()
    }

    pub fn to_chain(&self) -> Chain {
        Chain::from_str(self.as_ref()).unwrap()
    }
}

#[cfg(test)]
mod tests {
    use crate::{Chain, EVMChain};

    #[test]
    fn test_from_chain() {
        assert_eq!(EVMChain::from_chain(Chain::Ethereum), Some(EVMChain::Ethereum));
        assert_eq!(EVMChain::from_chain(Chain::Solana), None);
    }
}
