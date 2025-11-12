use crate::AssetId;
use num_bigint::BigUint;
use serde::{Deserialize, Serialize};
use typeshare::typeshare;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct AssetBalance {
    pub asset_id: AssetId,
    pub balance: Balance,
    pub is_active: bool,
}

impl AssetBalance {
    pub fn new(asset_id: AssetId, balance: BigUint) -> Self {
        Self {
            asset_id,
            balance: Balance::coin_balance(balance),
            is_active: true,
        }
    }

    pub fn new_zero_balance(asset_id: AssetId) -> Self {
        Self::new(asset_id, BigUint::from(0u32))
    }

    pub fn new_balance(asset_id: AssetId, balance: Balance) -> Self {
        Self {
            asset_id,
            balance,
            is_active: true,
        }
    }

    pub fn new_with_active(asset_id: AssetId, balance: Balance, is_active: bool) -> Self {
        Self { asset_id, balance, is_active }
    }

    pub fn new_staking(asset_id: AssetId, staked: BigUint, pending: BigUint, rewards: BigUint) -> Self {
        Self {
            asset_id,
            balance: Balance::stake_balance(staked, pending, Some(rewards)),
            is_active: true,
        }
    }
    pub fn new_staking_with_metadata(asset_id: AssetId, staked: BigUint, pending: BigUint, rewards: BigUint, metadata: BalanceMetadata) -> Self {
        Self {
            asset_id,
            balance: Balance::stake_balance_with_metadata(staked, pending, Some(rewards), Some(metadata)),
            is_active: true,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Balance {
    #[serde(serialize_with = "serde_serializers::serialize_biguint", deserialize_with = "serde_serializers::deserialize_biguint_from_str")]
    pub available: BigUint,
    #[serde(serialize_with = "serde_serializers::serialize_biguint", deserialize_with = "serde_serializers::deserialize_biguint_from_str")]
    pub frozen: BigUint,
    #[serde(serialize_with = "serde_serializers::serialize_biguint", deserialize_with = "serde_serializers::deserialize_biguint_from_str")]
    pub locked: BigUint,
    #[serde(serialize_with = "serde_serializers::serialize_biguint", deserialize_with = "serde_serializers::deserialize_biguint_from_str")]
    pub staked: BigUint,
    #[serde(serialize_with = "serde_serializers::serialize_biguint", deserialize_with = "serde_serializers::deserialize_biguint_from_str")]
    pub pending: BigUint,
    #[serde(serialize_with = "serde_serializers::serialize_biguint", deserialize_with = "serde_serializers::deserialize_biguint_from_str")]
    pub rewards: BigUint,
    #[serde(serialize_with = "serde_serializers::serialize_biguint", deserialize_with = "serde_serializers::deserialize_biguint_from_str")]
    pub reserved: BigUint,
    #[serde(serialize_with = "serde_serializers::serialize_biguint", deserialize_with = "serde_serializers::deserialize_biguint_from_str")]
    pub withdrawable: BigUint,
    pub metadata: Option<BalanceMetadata>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
#[typeshare(swift = "Equatable, Hashable, Sendable")]
#[serde(rename_all = "camelCase")]
pub struct BalanceMetadata {
    pub votes: u32,
    pub energy_available: u32,
    pub energy_total: u32,
    pub bandwidth_available: u32,
    pub bandwidth_total: u32,
}


impl Balance {
    pub fn coin_balance(available: BigUint) -> Self {
        Self {
            available,
            frozen: BigUint::from(0u32),
            locked: BigUint::from(0u32),
            staked: BigUint::from(0u32),
            pending: BigUint::from(0u32),
            rewards: BigUint::from(0u32),
            reserved: BigUint::from(0u32),
            withdrawable: BigUint::from(0u32),
            metadata: None,
        }
    }

    pub fn zero() -> Self {
        Self::coin_balance(BigUint::from(0u32))
    }

    pub fn with_reserved(available: BigUint, reserved: BigUint) -> Self {
        Self {
            available,
            reserved,
            frozen: BigUint::from(0u32),
            locked: BigUint::from(0u32),
            staked: BigUint::from(0u32),
            pending: BigUint::from(0u32),
            rewards: BigUint::from(0u32),
            withdrawable: BigUint::from(0u32),
            metadata: None,
        }
    }

    pub fn stake_balance(staked: BigUint, pending: BigUint, rewards: Option<BigUint>) -> Self {
        Self::stake_balance_with_metadata(staked, pending, rewards, None)
    }

    pub fn stake_balance_with_metadata(staked: BigUint, pending: BigUint, rewards: Option<BigUint>, metadata: Option<BalanceMetadata>) -> Self {
        Self {
            available: BigUint::from(0u32),
            frozen: BigUint::from(0u32),
            locked: BigUint::from(0u32),
            staked,
            pending,
            rewards: rewards.unwrap_or(BigUint::from(0u32)),
            reserved: BigUint::from(0u32),
            withdrawable: BigUint::from(0u32),
            metadata,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json;

    #[test]
    fn test_balance_serialization() {
        // 模拟你的实际值：3608662477 + 34450 * 2^32 ≈ 148,113,058,094,125
        let available_value = BigUint::from(3608662477u64) + BigUint::from(34450u64) * BigUint::from(4294967296u64);

        let balance = Balance {
            available: available_value,
            frozen: BigUint::from(0u32),
            locked: BigUint::from(0u32),
            staked: BigUint::from(0u32),
            pending: BigUint::from(0u32),
            rewards: BigUint::from(0u32),
            reserved: BigUint::from(0u32),
            withdrawable: BigUint::from(0u32),
            metadata: None,
        };

        let serialized = serde_json::to_string_pretty(&balance).unwrap();
        println!("Serialized balance:\n{}", serialized);

        // 验证序列化为字符串而不是数组
        assert!(serialized.contains("\"available\": \""));
        assert!(!serialized.contains("\"available\": ["));
    }

    #[test]
    fn test_balance_deserialization() {
        let json_data = r#"{
            "available": "148113058094125",
            "frozen": "0",
            "locked": "0",
            "staked": "0",
            "pending": "0",
            "rewards": "0",
            "reserved": "0",
            "withdrawable": "0",
            "metadata": null
        }"#;

        let balance: Balance = serde_json::from_str(json_data).unwrap();
        assert_eq!(balance.available, BigUint::from(148113058094125u64));
    }

    #[test]
    fn test_asset_balance_serialization() {
        let asset_balance = AssetBalance::new(
            AssetId::new("solana").unwrap(),
            BigUint::from(1000000u64)
        );

        let serialized = serde_json::to_string_pretty(&asset_balance).unwrap();
        println!("Serialized asset balance:\n{}", serialized);

        // 验证所有 BigUint 字段都序列化为字符串
        assert!(serialized.contains("\"available\": \"1000000\""));
    }
}
