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

pub mod chain_evm;
pub use self::chain_evm::EVMChain;

pub mod transaction_state;
pub use self::transaction_state::TransactionState;

pub mod delegation;
pub use self::delegation::{Delegation, DelegationBase, DelegationState, DelegationValidator};

pub mod price;
pub use self::price::{Price, PriceFull};

pub mod validator;
pub use self::validator::StakeValidator;

pub mod scan;
pub use self::scan::{AddressType, ScanAddress, ScanAddressTarget, ScanTransaction, ScanTransactionPayload};

pub mod transaction_type;
pub use self::transaction_type::TransactionType;

pub mod asset_price;
pub use self::asset_price::{AssetMarket, AssetPrice, AssetPrices, AssetPricesRequest, ChartPeriod, ChartValue, Charts};

pub mod price_alert;
pub use self::price_alert::{DevicePriceAlert, PriceAlert, PriceAlertDirection, PriceAlertType, PriceAlerts};

pub mod device;
pub use self::device::Device;

pub mod asset;
pub use self::asset::{Asset, AssetVecExt};

pub mod asset_details;
pub use self::asset_details::{AssetBasic, AssetFull, AssetLink, AssetMarketPrice, AssetPriceMetadata, AssetProperties};

pub mod asset_score;
pub use self::asset_score::AssetScore;

pub mod platform;
pub use self::platform::{Platform, PlatformStore};

pub mod perpetual;
pub use self::perpetual::{
    AccountDataType, CancelOrderData, Perpetual, PerpetualBalance, PerpetualBasic, PerpetualConfirmData, PerpetualDirection, PerpetualModifyConfirmData,
    PerpetualModifyPositionType, PerpetualPositionData, PerpetualPositionsSummary, PerpetualReduceData, PerpetualType, TPSLOrderData,
};

pub mod perpetual_provider;
pub use self::perpetual_provider::PerpetualProvider;

pub mod perpetual_position;
pub use self::perpetual_position::{PerpetualMarginType, PerpetualOrderType, PerpetualPosition, PerpetualTriggerOrder};

pub mod link_type;
pub use self::link_type::LinkType;

pub mod chain_stake;
pub use self::chain_stake::StakeChain;

pub type UInt64 = u64;


pub const DEFAULT_FIAT_CURRENCY: &str = "USD";