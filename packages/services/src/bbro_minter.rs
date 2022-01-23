use cosmwasm_std::Uint128;
use cw20_base::msg::InstantiateMsg as TokenInstantiateMsg;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// ## InstantiateMsg
/// This structure describes the basic settings for creating a contract.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct InstantiateMsg {
    /// contract/multisig address that allowed to control settings
    pub gov_contract: String,
    /// list of whitelisted addresses allowed to execute mint/burn functions
    pub whitelist: Vec<String>,
}

/// ## ExecuteMsg
/// This structure describes the execute messages of the contract.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    /// ## Description
    /// Creates new token contract
    /// ## Executor
    /// Only owner can execute this function
    InstantiateToken {
        // code id of deployed wasm contract
        code_id: u64,
        // instantiate message of deployed wasm contract
        token_instantiate_msg: TokenInstantiateMsg,
    },
    /// ## Description
    /// Updates contract settings
    /// ## Executor
    /// Only owner can execute this function
    UpdateConfig {
        /// new contract owner
        new_gov_contract: Option<String>,
        /// new bbro token address
        bbro_token: Option<String>,
    },
    /// ## Description
    /// Adds new minter address into whitelist
    /// ## Executor
    /// Only owner can execute this function
    AddMinter {
        /// minter address
        minter: String,
    },
    /// ## Description
    /// Removes minter from whitelist
    /// ## Executor
    /// Only owner can execute this function
    RemoveMinter {
        /// minter address
        minter: String,
    },
    /// ## Description
    /// Mints specified amount for specified address
    /// ## Executor
    /// Only whitelisted address can execute this function
    Mint {
        /// token receiver address
        recipient: String,
        /// amount of tokens to receive
        amount: Uint128,
    },
    /// ## Description
    /// Burns specified amount from specified address balance
    /// ## Executor
    /// Only whitelisted address can execute this function
    Burn {
        /// token owner address
        owner: String,
        /// amount of tokens to burn
        amount: Uint128,
    },
}

/// ## QueryMsg
/// This structure describes the query messages of the contract
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    /// ## Description
    /// Returns bbro-minter contract config in the [`ConfigResponse`] object
    Config {},
}

/// ## ConfigResponse
/// This structure describes the fields for config response message.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct ConfigResponse {
    /// contract/multisig address that allowed to control settings
    pub gov_contract: String,
    /// bbro token address
    pub bbro_token: String,
    /// list of whitelisted addresses allowed to execute mint/burn functions
    pub whitelist: Vec<String>,
}
