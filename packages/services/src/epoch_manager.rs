use cosmwasm_std::{Decimal, Uint128};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// ## InstantiateMsg
/// This structure describes the basic settings for creating a contract.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct InstantiateMsg {
    /// amount of blocks per epoch
    pub epoch: u64,
    /// amount of blocks per one year
    pub blocks_per_year: u64,
    /// bbro emission rate
    pub bbro_emission_rate: Decimal,
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
        /// new contract owner
        owner: Option<String>,
    },
    /// ## Description
    /// Updates contract state
    /// ## Executor
    /// Only owner can execute this function
    UpdateState {
        /// amount of blocks per epoch
        epoch: Option<u64>,
        /// amount of blocks per one year
        blocks_per_year: Option<u64>,
        /// bbro emission rate
        bbro_emission_rate: Option<Decimal>,
    },
}

/// ## QueryMsg
/// This structure describes the query messages of the contract
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    /// ## Description
    /// Returns epoch-manager contract config in the [`ConfigResponse`] object
    Config {},
    /// ## Description
    /// Returns epoch-manager contract state in the [`EpochInfoResponse`] object
    EpochInfo {},
}

/// ## ConfigResponse
/// This structure describes the fields for config response message.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct ConfigResponse {
    /// contract/multisig address that allowed to control settings
    pub owner: String,
}

/// ## EpochInfoResponse
/// This structure describes the fields for epoch info response message.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct EpochInfoResponse {
    /// amount of blocks per epoch
    pub epoch: u64,
    /// amount of blocks per one year
    pub blocks_per_year: u64,
    /// bbro emission rate
    pub bbro_emission_rate: Decimal,
}

impl EpochInfoResponse {
    pub fn epochs_per_year(&self) -> Uint128 {
        Uint128::from(self.blocks_per_year / self.epoch)
    }
}
