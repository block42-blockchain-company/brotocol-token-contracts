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
    /// max allowed amount to spend per distribution
    pub spend_limit: Uint128,
    /// list of whitelisted addresses allowed to execute rewards distribution function
    pub whitelist: Vec<String>,
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
        /// max allowed amount to spend per distribution
        spend_limit: Option<Uint128>,
    },
    /// ## Description
    /// Adds new distributor address into whitelist
    /// ## Executor
    /// Only owner can execute this function
    AddDistributor {
        /// distributor address
        distributor: String,
    },
    /// ## Description
    /// Removes distributor from whitelist
    /// ## Executor
    /// Only owner can execute this function
    RemoveDistributor {
        /// distributor address
        distributor: String,
    },
    /// ## Description
    /// Distributes rewards to specified contracts
    /// ## Executor
    /// Only whitelisted address can execute this function
    DistributeRewards {
        /// a list of distribution messages
        distributions: Vec<DistributeRewardMsg>,
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
    /// Returns rewards pool contract config in the [`ConfigResponse`] object
    Config {},
    /// ## Description
    /// Returns rewards pool token balance in the [`RewardsPoolBalanceResponse`] object
    Balance {},
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

/// ## DistributeRewardMsg
/// This structure describes the fields for rewards distribution message.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct DistributeRewardMsg {
    /// receiver contract address
    pub contract: String,
    /// distribution amount
    pub amount: Uint128,
    /// binary msg to execute on receiver contract
    pub msg: Binary,
}

/// ## ConfigResponse
/// This structure describes the fields for config response message.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct ConfigResponse {
    /// contract/multisig address that allowed to control settings
    pub owner: String,
    /// bro token address
    pub bro_token: String,
    /// max allowed amount to spend per distribution
    pub spend_limit: Uint128,
    /// list of whitelisted addresses allowed to execute rewards distribution function
    pub whitelist: Vec<String>,
}

/// ## RewardsPoolBalanceResponse
/// This structure describes the fields for rewards pool balance response message.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct RewardsPoolBalanceResponse {
    /// rewards pool token balance
    pub balance: Uint128,
}
