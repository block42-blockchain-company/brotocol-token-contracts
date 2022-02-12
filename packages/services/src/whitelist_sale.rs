use cosmwasm_std::Uint128;
use cw20::Cw20ReceiveMsg;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// ## InstantiateMsg
/// This structure describes the basic settings for creating a contract.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct InstantiateMsg {
    pub bro_token: String,
    pub bro_amount_per_uusd: Uint128,
    pub bro_amount_per_nft: Uint128,
    pub treasury_contract: String,
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
    Purchase {},
    WithdrawRemainingBalance {},
}

/// ## Cw20HookMsg
/// This structure describes the cw20 receive hook messages of the contract.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum Cw20HookMsg {
    RegisterSale {
        sale_start_time: u64,
        sale_end_time: u64,
        accounts: Vec<WhitelistedAccountInfo>,
    },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct WhitelistedAccountInfo {
    pub address: String,
    pub owned_nfts_count: u64,
}

/// ## QueryMsg
/// This structure describes the query messages of the contract
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    Config {},
    State {},
    WhitelistedAccount { address: String },
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
    pub owner: String,
    pub bro_token: String,
    pub bro_amount_per_uusd: Uint128,
    pub bro_amount_per_nft: Uint128,
    pub treasury_contract: String,
    pub rewards_pool_contract: String,
}

/// ## StateResponse
/// This structure describes the fields for state response message.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct StateResponse {
    pub sale_registered: bool,
    pub sale_start_time: u64,
    pub sale_end_time: u64,
    pub current_time: u64,
    pub balance: Uint128,
}

/// ## WhitelistedAccountInfoResponse
/// /// This structure describes the fields for whitelisted account response message.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct WhitelistedAccountInfoResponse {
    pub address: String,
    pub available_purchase_amount: Uint128,
}
