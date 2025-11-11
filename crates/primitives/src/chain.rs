//! 区块链网络枚举和实现
//!
//! 定义了支持的区块链网络类型及其相关属性（网络 ID、区块时间等）。

use serde::{Deserialize, Serialize};
use std::fmt;
use strum::{AsRefStr, EnumIter, EnumString};

/// 支持的区块链网络枚举
///
/// 当前支持的 EVM 兼容链：
/// - Ethereum (ETH) - 以太坊主网
/// - SmartChain (BSC) - 币安智能链
/// - Arbitrum (ARB) - Arbitrum One
/// - Polygon (MATIC) - Polygon 主网
#[derive(
    Copy,
    Clone,
    Debug,
    Serialize,
    Deserialize,
    EnumIter,
    AsRefStr,
    EnumString,
    PartialEq,
    Ord,
    PartialOrd,
    Eq,
    Hash,
)]
#[serde(rename_all = "lowercase")]
#[strum(serialize_all = "lowercase")]
pub enum Chain {
    /// 以太坊主网 (Chain ID: 1)
    Ethereum,
    /// 币安智能链 (Chain ID: 56)
    SmartChain,
    /// Arbitrum One (Chain ID: 42161)
    Arbitrum,
    /// Polygon 主网 (Chain ID: 137)
    Polygon,
}

impl fmt::Display for Chain {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.as_ref())
    }
}

impl Chain {
    /// 获取链的网络 ID (Chain ID)
    ///
    /// # 返回值
    /// 返回该链的标准网络 ID 字符串
    ///
    /// # 示例
    /// ```
    /// use primitives::Chain;
    ///
    /// assert_eq!(Chain::Ethereum.network_id(), "1");
    /// assert_eq!(Chain::SmartChain.network_id(), "56");
    /// assert_eq!(Chain::Arbitrum.network_id(), "42161");
    /// assert_eq!(Chain::Polygon.network_id(), "137");
    /// ```
    pub fn network_id(&self) -> &str {
        match self {
            Self::Ethereum => "1",
            Self::SmartChain => "56",
            Self::Arbitrum => "42161",
            Self::Polygon => "137",
        }
    }

    /// 根据 Chain ID 获取对应的链
    ///
    /// # 参数
    /// - `chain_id` - 链 ID 数字
    ///
    /// # 返回值
    /// - `Some(Chain)` - 找到对应的链
    /// - `None` - 不支持的链 ID
    ///
    /// # 示例
    /// ```
    /// use primitives::Chain;
    ///
    /// assert_eq!(Chain::from_chain_id(1), Some(Chain::Ethereum));
    /// assert_eq!(Chain::from_chain_id(56), Some(Chain::SmartChain));
    /// assert_eq!(Chain::from_chain_id(999), None);
    /// ```
    pub fn from_chain_id(chain_id: u64) -> Option<Self> {
        use strum::IntoEnumIterator;
        Self::iter().find(|&x| x.network_id() == chain_id.to_string())
    }

    /// 获取链的 SLIP-44 币种代码
    ///
    /// SLIP-44 定义了 BIP-44 派生路径中使用的币种代码。
    /// 所有 EVM 链使用相同的币种代码 60 (Ethereum)。
    ///
    /// # 返回值
    /// SLIP-44 币种代码
    ///
    /// # 示例
    /// ```
    /// use primitives::Chain;
    ///
    /// // 所有 EVM 链都使用 60
    /// assert_eq!(Chain::Ethereum.as_slip44(), 60);
    /// assert_eq!(Chain::SmartChain.as_slip44(), 60);
    /// ```
    pub fn as_slip44(&self) -> i64 {
        match self {
            Self::Ethereum | Self::Arbitrum | Self::SmartChain | Self::Polygon => 60,
        }
    }

    /// 获取链的平均区块时间（毫秒）
    ///
    /// # 返回值
    /// 区块时间（毫秒）
    ///
    /// # 示例
    /// ```
    /// use primitives::Chain;
    ///
    /// assert_eq!(Chain::Ethereum.block_time(), 12_000); // 12 秒
    /// assert_eq!(Chain::SmartChain.block_time(), 3_000); // 3 秒
    /// ```
    pub fn block_time(&self) -> u32 {
        match self {
            Self::SmartChain => 3_000,         // 3 秒
            Self::Arbitrum => 250,             // 0.25 秒
            Self::Polygon => 2_000,            // 2 秒
            Self::Ethereum => 12_000,          // 12 秒
        }
    }

    /// 获取链的显示优先级/排名
    ///
    /// 数值越高，显示优先级越高。
    ///
    /// # 返回值
    /// 排名分数（0-100）
    ///
    /// # 示例
    /// ```
    /// use primitives::Chain;
    ///
    /// assert!(Chain::Ethereum.rank() > Chain::SmartChain.rank());
    /// ```
    pub fn rank(&self) -> i32 {
        match self {
            Self::Ethereum => 100,    // 最高优先级
            Self::SmartChain => 80,   // 高优先级
            Self::Arbitrum => 70,     // 中高优先级
            Self::Polygon => 70,      // 中高优先级
        }
    }

    /// 获取所有支持的链
    ///
    /// # 返回值
    /// 包含所有支持链的向量
    ///
    /// # 示例
    /// ```
    /// use primitives::Chain;
    ///
    /// let chains = Chain::all();
    /// assert_eq!(chains.len(), 4);
    /// ```
    pub fn all() -> Vec<Self> {
        use strum::IntoEnumIterator;
        Self::iter().collect::<Vec<_>>()
    }

    /// 是否为 EVM 兼容链
    ///
    /// # 返回值
    /// 当前所有支持的链都是 EVM 兼容的，始终返回 `true`
    ///
    /// # 示例
    /// ```
    /// use primitives::Chain;
    ///
    /// assert!(Chain::Ethereum.is_evm());
    /// assert!(Chain::SmartChain.is_evm());
    /// ```
    pub fn is_evm(&self) -> bool {
        // 当前所有支持的链都是 EVM 兼容的
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;

    #[test]
    fn test_chain_network_id() {
        assert_eq!(Chain::Ethereum.network_id(), "1");
        assert_eq!(Chain::SmartChain.network_id(), "56");
        assert_eq!(Chain::Arbitrum.network_id(), "42161");
        assert_eq!(Chain::Polygon.network_id(), "137");
    }

    #[test]
    fn test_chain_from_chain_id() {
        assert_eq!(Chain::from_chain_id(1), Some(Chain::Ethereum));
        assert_eq!(Chain::from_chain_id(56), Some(Chain::SmartChain));
        assert_eq!(Chain::from_chain_id(42161), Some(Chain::Arbitrum));
        assert_eq!(Chain::from_chain_id(137), Some(Chain::Polygon));
        assert_eq!(Chain::from_chain_id(999), None);
    }

    #[test]
    fn test_chain_slip44() {
        // 所有 EVM 链都应该返回 60
        assert_eq!(Chain::Ethereum.as_slip44(), 60);
        assert_eq!(Chain::SmartChain.as_slip44(), 60);
        assert_eq!(Chain::Arbitrum.as_slip44(), 60);
        assert_eq!(Chain::Polygon.as_slip44(), 60);
    }

    #[test]
    fn test_chain_block_time() {
        assert_eq!(Chain::Ethereum.block_time(), 12_000);
        assert_eq!(Chain::SmartChain.block_time(), 3_000);
        assert_eq!(Chain::Arbitrum.block_time(), 250);
        assert_eq!(Chain::Polygon.block_time(), 2_000);
    }

    #[test]
    fn test_chain_rank() {
        assert_eq!(Chain::Ethereum.rank(), 100);
        assert_eq!(Chain::SmartChain.rank(), 80);
        assert!(Chain::Ethereum.rank() > Chain::Arbitrum.rank());
    }

    #[test]
    fn test_chain_all() {
        let chains = Chain::all();
        assert_eq!(chains.len(), 4);
        assert!(chains.contains(&Chain::Ethereum));
        assert!(chains.contains(&Chain::SmartChain));
        assert!(chains.contains(&Chain::Arbitrum));
        assert!(chains.contains(&Chain::Polygon));
    }

    #[test]
    fn test_chain_is_evm() {
        assert!(Chain::Ethereum.is_evm());
        assert!(Chain::SmartChain.is_evm());
        assert!(Chain::Arbitrum.is_evm());
        assert!(Chain::Polygon.is_evm());
    }

    #[test]
    fn test_chain_display() {
        assert_eq!(Chain::Ethereum.to_string(), "ethereum");
        assert_eq!(Chain::SmartChain.to_string(), "smartchain");
        assert_eq!(Chain::Arbitrum.to_string(), "arbitrum");
        assert_eq!(Chain::Polygon.to_string(), "polygon");
    }

    #[test]
    fn test_chain_from_str() {
        assert_eq!(Chain::from_str("ethereum").unwrap(), Chain::Ethereum);
        assert_eq!(Chain::from_str("smartchain").unwrap(), Chain::SmartChain);
        assert_eq!(Chain::from_str("arbitrum").unwrap(), Chain::Arbitrum);
        assert_eq!(Chain::from_str("polygon").unwrap(), Chain::Polygon);
        assert!(Chain::from_str("unknown").is_err());
    }

    #[test]
    fn test_chain_serialization() {
        let chain = Chain::Ethereum;
        let serialized = serde_json::to_string(&chain).unwrap();
        assert_eq!(serialized, r#""ethereum""#);

        let deserialized: Chain = serde_json::from_str(&serialized).unwrap();
        assert_eq!(deserialized, Chain::Ethereum);
    }

    #[test]
    fn test_chain_ordering() {
        let mut chains = vec![Chain::Polygon, Chain::Ethereum, Chain::Arbitrum, Chain::SmartChain];
        chains.sort();

        // 验证排序后的顺序（按枚举定义顺序）
        // Ethereum < SmartChain < Arbitrum < Polygon
        assert_eq!(chains[0], Chain::Ethereum);
        assert_eq!(chains[1], Chain::SmartChain);
        assert_eq!(chains[2], Chain::Arbitrum);
        assert_eq!(chains[3], Chain::Polygon);
    }

    #[test]
    fn test_chain_hash() {
        use std::collections::HashSet;

        let mut set = HashSet::new();
        set.insert(Chain::Ethereum);
        set.insert(Chain::Ethereum); // 重复插入

        assert_eq!(set.len(), 1);
        assert!(set.contains(&Chain::Ethereum));
    }
}
