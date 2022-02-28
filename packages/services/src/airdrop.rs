use cosmwasm_std::Uint128;
use cw20::Cw20ReceiveMsg;
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
    /// Receives a message of type [`Cw20ReceiveMsg`] and processes it depending on the received
    /// template.
    Receive(Cw20ReceiveMsg),
    /// ## Description
    /// Updates contract settings
    /// ## Executor
    /// Only owner can execute this function
    UpdateConfig {
        /// new contract owner
        owner: Option<String>,
    },
    /// ## Description
    /// Claims available amount for message sender at specified airdrop round
    Claim {
        /// airdrop stage
        stage: u8,
        /// claim amount
        amount: Uint128,
        /// proofs that message sender allowed to claim provided amount
        proof: Vec<String>,
    },
}

/// ## Cw20HookMsg
/// This structure describes the cw20 receive hook messages of the contract.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum Cw20HookMsg {
    /// ## Description
    /// Registers merkle root hash
    RegisterMerkleRoot {
        /// merkle root string represented as hash
        merkle_root: String,
    },
}

/// ## QueryMsg
/// This structure describes the query messages of the contract
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    /// ## Description
    /// Returns airdrop contract config in the [`ConfigResponse`] object
    Config {},
    /// ## Description
    /// Returns the number of latest stage in the [`LatestStageResponse`] object
    LatestStage {},
    /// ## Description
    /// Returns merkle root information by specified stage in the [`MerkleRootResponse`] object
    MerkleRoot {
        /// airdrop stage
        stage: u8,
    },
    /// ## Description
    /// Returns claim information by specified stage and address in the [`IsClaimedResponse`] object
    IsClaimed {
        /// airdrop stage
        stage: u8,
        /// account address
        address: String,
    },
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

/// ## LatestStageResponse
/// This structure describes the fields for latest stage response message.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct LatestStageResponse {
    /// latest airdrop stage number
    pub latest_stage: u8,
}

/// ## MerkleRootResponse
/// This structure describes the fields for merkle root response message.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct MerkleRootResponse {
    /// airdrop stage
    pub stage: u8,
    /// merkle root string represented as hash
    pub merkle_root: String,
}

/// ## IsClaimedResponse
/// This structure describes the fields for claim info response message.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct IsClaimedResponse {
    /// was airdrop amount already claimed
    pub is_claimed: bool,
}
