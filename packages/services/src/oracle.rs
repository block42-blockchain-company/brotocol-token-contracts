use cosmwasm_std::Uint128;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use astroport::asset::AssetInfo;

/// ## InstantiateMsg
/// This structure describes the basic settings for creating a contract.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct InstantiateMsg {
    /// factory contract address
    pub factory_contract: String,
    /// assets in the pool
    pub asset_infos: [AssetInfo; 2],
    /// time interval for updating prices
    pub price_update_interval: u64,
}

/// ## ExecuteMsg
/// This structure describes the execute messages of the contract.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    /// ## Description
    /// Updates contract settings
    /// ## Executor
    /// Only owner can execute this function
    UpdateConfig {
        /// contract/multisig address that allowed to control settings
        owner: Option<String>,
        /// time interval for updating prices
        price_update_interval: Option<u64>,
    },
    /// ## Description
    /// Updates cumulative prices
    UpdatePrice {},
}

/// ## QueryMsg
/// This structure describes the query messages of the contract
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    /// ## Description
    /// Returns rewards pool contract config in the [`ConfigResponse`] object
    Config {},
    /// ## Description
    /// Returns calculated average amount with updated precision in the [`ConsultPriceResponse`] object
    ConsultPrice {
        /// asset info
        asset: AssetInfo,
        /// amount of specified asset
        amount: Uint128,
    },
    /// ## Description
    /// Returns a [`bool`] type whether prices are ready to be updated or not
    IsReadyToTrigger {},
}

/// ## MigrateMsg
/// This structure describes a migration message.
/// We currently take no arguments for migrations.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct MigrateMsg {}

/// ## ConfigResponse
/// This structure describes the fields for config response message.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct ConfigResponse {
    /// contract/multisig address that allowed to control settings
    pub owner: String,
    /// factory contract address
    pub factory: String,
    /// assets in the pool
    pub asset_infos: [AssetInfo; 2],
    /// time interval for updating prices
    pub price_update_interval: u64,
}

/// ## ConsultPriceResponse
/// This structure describes the fields for consult price response message.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct ConsultPriceResponse {
    pub amount: Uint128,
}
