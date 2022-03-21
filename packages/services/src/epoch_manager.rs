use cosmwasm_std::{Decimal, Uint128};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// ## InstantiateMsg
/// This structure describes the basic settings for creating a contract.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct InstantiateMsg {
    /// contract/multisig address that allowed to control settings
    pub owner: String,
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
    /// Returns epoch-manager contract config in the [`ConfigResponse`] object
    Config {},
    /// ## Description
    /// Returns epoch-manager contract state in the [`EpochInfoResponse`] object
    EpochInfo {},
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
