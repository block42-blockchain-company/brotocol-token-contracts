use cosmwasm_std::{Binary, Uint128};
use cw20::Cw20ReceiveMsg;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// ## InstantiateMsg
/// This structure describes the basic settings for creating a contract.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct InstantiateMsg {
    /// bro token address
    pub bro_token: String,
    /// bro amount per uusd
    pub bro_amount_per_uusd: Uint128,
    /// bro amount for purchase per nft
    pub bro_amount_per_nft: Uint128,
    /// address for sending received ust
    pub ust_receiver: String,
    /// rewards pool address
    pub rewards_pool_contract: String,
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
    /// Purchase bro by fixed price by providing ust amount.
    Purchase {},
    /// ## Description
    /// Withdraw remaining bro balance after sale is over.
    WithdrawRemainingBalance {},
}

/// ## Cw20HookMsg
/// This structure describes the cw20 receive hook messages of the contract.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum Cw20HookMsg {
    /// ## Description
    /// Registers sale and whitelists addresses
    RegisterSale {
        /// sale start time
        sale_start_time: u64,
        /// sale end time
        sale_end_time: u64,
        /// whitelisted accounts
        accounts: Binary,
    },
}

/// ## WhitelistedAccountInfo
/// This structure describes the fields for whitelisted account info object.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct WhitelistedAccountInfo {
    /// account address
    pub address: String,
    /// amount of owned nfts
    pub owned_nfts_count: u64,
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
    /// Returns staking contract state in the [`StateResponse`] object
    State {},
    /// ## Description
    /// Returns whitelisted account info in the [`WhitelistedAccountInfoResponse`] object
    WhitelistedAccount {
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
    /// bro amount per uusd
    pub bro_amount_per_uusd: Uint128,
    /// bro amount for purchase per nft
    pub bro_amount_per_nft: Uint128,
    /// address for sending received ust
    pub ust_receiver: String,
    /// rewards pool address
    pub rewards_pool_contract: String,
}

/// ## StateResponse
/// This structure describes the fields for state response message.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct StateResponse {
    /// sets sale either to registered or not
    pub sale_registered: bool,
    /// sale start time
    pub sale_start_time: u64,
    /// sale end time
    pub sale_end_time: u64,
    /// current time
    pub current_time: u64,
    /// remaining contract balance
    pub balance: Uint128,
}

/// ## WhitelistedAccountInfoResponse
/// This structure describes the fields for whitelisted account response message.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct WhitelistedAccountInfoResponse {
    /// account address
    pub address: String,
    /// available purchase amount
    pub available_purchase_amount: Uint128,
}
