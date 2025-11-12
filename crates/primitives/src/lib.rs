//! 区块链原语类型定义
//!
//! 本模块定义了支持的区块链网络类型和相关属性。

mod chain;
mod node_sync_status;

pub use chain::Chain;
pub use self::node_sync_status::{NodeStatusState, NodeSyncStatus};

pub mod asset_balance;
pub use self::asset_balance::{AssetBalance, Balance};

pub mod asset_id;
pub use self::asset_id::{AssetId, AssetIdVecExt};

pub mod asset_type;
pub use self::asset_type::{AssetSubtype, AssetType};

pub mod node;
pub use self::node::{Node, NodeType};

pub mod signer_error;
pub use self::signer_error::SignerError;

pub mod chain_address;
pub use self::chain_address::ChainAddress;

pub mod json_rpc;
pub use self::json_rpc::JsonRpcResult;

pub type UInt64 = u64;