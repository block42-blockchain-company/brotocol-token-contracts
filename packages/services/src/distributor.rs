use cosmwasm_std::Uint128;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// ## InstantiateMsg
/// This structure describes the basic settings for creating a contract.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct InstantiateMsg {
    /// contract/multisig address that allowed to control settings
    pub owner: String,
    /// genesis block for destribution start
    pub distribution_genesis_block: u64,
    /// epoch manager contract addresss
    pub epoch_manager_contract: String,
    /// rewards pool contract address
    pub rewards_contract: String,
    /// staking contract address
    pub staking_contract: String,
    /// amount per epoch to distribute for staking
    pub staking_distribution_amount: Uint128,
    /// bonding contract address
    pub bonding_contract: String,
    /// amount per epoch to distribute for bonding
    pub bonding_distribution_amount: Uint128,
}

/// ## ExecuteMsg
/// This structure describes the execute messages of the contract.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    /// ## Description
    /// Performs token distribution
    Distribute {},
    /// ## Description
    /// Updates contract settings
    /// ## Executor
    /// Only owner can execute this function
    UpdateConfig {
        /// defines either contract paused or not
        paused: Option<bool>,
        /// epoch manager contract addresss
        epoch_manager_contract: Option<String>,
        /// rewards pool contract address
        rewards_contract: Option<String>,
        /// staking contract address
        staking_contract: Option<String>,
        /// amount per epoch to distribute for staking
        staking_distribution_amount: Option<Uint128>,
        /// bonding contract address
        bonding_contract: Option<String>,
        /// amount per epoch to distribute for bonding
        bonding_distribution_amount: Option<Uint128>,
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
    /// Returns bbro-minter contract config in the [`ConfigResponse`] object
    Config {},
    /// ## Description
    /// Returns information about last distribution in the [`LastDistributionResponse`] object
    LastDistribution {},
    /// ## Description
    /// Returns a [`bool`] type whether the contract is ready to be triggered or not
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
    /// defines either contract paused or not
    pub paused: bool,
    /// genesis block for destribution start
    pub distribution_genesis_block: u64,
    /// epoch manager contract addresss
    pub epoch_manager_contract: String,
    /// rewards pool contract address
    pub rewards_contract: String,
    /// staking contract address
    pub staking_contract: String,
    /// amount per epoch to distribute for staking
    pub staking_distribution_amount: Uint128,
    /// bonding contract address
    pub bonding_contract: String,
    /// amount per epoch to distribute for bonding
    pub bonding_distribution_amount: Uint128,
}

/// ## LastDistributionResponse
/// This structure describes the fields for last ditribution response message.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct LastDistributionResponse {
    /// last distribution block
    pub last_distribution_block: u64,
}
