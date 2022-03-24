use cosmwasm_std::Uint128;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use astroport::asset::AssetInfo;

/// ## InstantiateMsg
/// This structure describes the basic settings for creating a contract.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct InstantiateMsg {
    /// contract/multisig address that allowed to control settings
    pub owner: String,
    /// factory contract address
    pub factory_contract: String,
    /// assets in the pool
    pub asset_infos: [AssetInfo; 2],
    /// time interval for updating prices
    pub price_update_interval: u64,
    /// time frame for how long a price is valid after update
    pub price_validity_period: u64,
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
        /// time interval for updating prices
        price_update_interval: Option<u64>,
        price_validity_period: Option<u64>,
    },
    /// ## Description
    /// Updates cumulative prices
    UpdatePrice {},
    /// ## Description
    /// Creates an offer for a new owner.
    /// The validity period of the offer is set in the `expires_in_blocks` variable
    /// ## Executor
    /// Only owner can execute this function
    ProposeNewOwner {
        /// new contract owner
        new_owner: String,
        /// expiration period in blocks
        expires_in_blocks: u64,
    },
    /// ## Description
    /// Removes the existing offer for the new owner
    /// ## Executor
    /// Only owner can execute this function
    DropOwnershipProposal {},
    /// ## Description
    /// Used to claim(approve) new owner proposal, thus changing contract's owner
    /// ## Executor
    /// Only address proposed as a new owner can execute this function
    ClaimOwnership {},
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
    /// ## Description
    /// Returns information about created ownership proposal in the [`OwnershipProposalResponse`] object
    /// otherwise returns not-found error
    OwnershipProposal {},
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
    /// time frame for how long a price is valid after update
    pub price_validity_period: u64,
}

/// ## ConsultPriceResponse
/// This structure describes the fields for consult price response message.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct ConsultPriceResponse {
    pub amount: Uint128,
}
