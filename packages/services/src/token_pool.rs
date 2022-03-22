use cosmwasm_std::{Binary, Uint128};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// ## InstantiateMsg
/// This structure describes the basic settings for creating a contract.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct InstantiateMsg {
    /// contract/multisig address that allowed to control settings
    pub owner: String,
    /// bro token address
    pub bro_token: String,
}

/// ## ExecuteMsg
/// This structure describes the execute messages of the contract.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    /// ## Description
    /// Transfer specified amount to specified address.
    Transfer {
        /// recipient address
        recipient: String,
        /// transfer amount
        amount: Uint128,
    },
    /// ## Description
    /// Transfer specified amount to specified contract with provided execute msg.
    Send {
        /// contract address
        contract: String,
        /// transfer amount
        amount: Uint128,
        /// message to execute
        msg: Binary,
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
    /// Returns staking contract config in the [`ConfigResponse`] object
    Config {},
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
    /// bro token address
    pub bro_token: String,
}
