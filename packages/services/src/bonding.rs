use cosmwasm_std::{Decimal, Uint128};
use cw20::{Cw20ReceiveMsg, Expiration};
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
    /// bro/ust lp token address
    pub lp_token: String,
    /// rewards pool address
    pub rewards_pool_contract: String,
    /// treasury contract address
    pub treasury_contract: String,
    /// astroport factory contract address
    pub astroport_factory: String,
    /// price oracle contract address
    pub oracle_contract: String,
    /// distributed reward percentage for ust bonding balance
    pub ust_bonding_reward_ratio: Decimal,
    /// discount percentage for ust bonding
    pub ust_bonding_discount: Decimal,
    /// discount percentage for lp bonding
    pub lp_bonding_discount: Decimal,
    /// minimum amount of bro to receive via bonding
    pub min_bro_payout: Uint128,
    /// vesting period for withdrawal
    pub vesting_period_blocks: u64,
    /// sets lp bonding option either to enabled or disabled
    pub lp_bonding_enabled: bool,
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
    /// Bond bro tokens by providing ust amount.
    UstBond {},
    /// ## Description
    /// Claim available reward amount.
    Claim {},
    /// ## Description
    /// Updates contract settings
    /// ## Executor
    /// Only owner can execute this function
    UpdateConfig {
        /// new contract owner
        owner: Option<String>,
        /// new bro/ust lp token address
        lp_token: Option<String>,
        /// rewards pool address
        rewards_pool_contract: Option<String>,
        /// new treasury contract address
        treasury_contract: Option<String>,
        /// new astroport factory address
        astroport_factory: Option<String>,
        /// price oracle contract address
        oracle_contract: Option<String>,
        /// new distributed reward percentage for ust bonding balance
        ust_bonding_reward_ratio: Option<Decimal>,
        /// new discount percentage for ust bonding
        ust_bonding_discount: Option<Decimal>,
        /// new discount percentage for lp bonding
        lp_bonding_discount: Option<Decimal>,
        /// new minimum amount of bro to receive via bonding
        min_bro_payout: Option<Uint128>,
        /// new vesting period for withdrawal
        vesting_period_blocks: Option<u64>,
        /// sets lp bonding option either to enabled or disabled
        lp_bonding_enabled: Option<bool>,
    },
}

/// ## Cw20HookMsg
/// This structure describes the cw20 receive hook messages of the contract.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum Cw20HookMsg {
    /// ## Description
    /// Distributes received reward
    DistributeReward {},
    /// ## Description
    /// Bond bro tokens by providing lp token amount.
    LpBond {},
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
    /// Returns available claims for bonder by specified address
    /// in the [`ClaimsResponse`] object
    Claims {
        /// bonder address
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
    /// bro/ust lp token address
    pub lp_token: String,
    /// rewards pool address
    pub rewards_pool_contract: String,
    /// treasury contract address
    pub treasury_contract: String,
    /// astroport factory contract address
    pub astroport_factory: String,
    /// price oracle contract address
    pub oracle_contract: String,
    /// distributed reward percentage for ust bonding balance
    pub ust_bonding_reward_ratio: Decimal,
    /// discount percentage for ust bonding
    pub ust_bonding_discount: Decimal,
    /// discount percentage for lp bonding
    pub lp_bonding_discount: Decimal,
    /// minimum amount of bro to receive via bonding
    pub min_bro_payout: Uint128,
    /// vesting period for withdrawal
    pub vesting_period_blocks: u64,
    /// sets lp bonding option either to enabled or disabled
    pub lp_bonding_enabled: bool,
}

/// ## StateResponse
/// This structure describes the fields for state response message.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct StateResponse {
    /// available bro balance for ust bonding
    pub ust_bonding_balance: Uint128,
    /// available bro balance for lp token bonding
    pub lp_bonding_balance: Uint128,
}

/// ## ClaimInfoResponse
/// This structure describes the fields for claim info response message.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct ClaimInfoResponse {
    /// bond type
    pub bond_type: String,
    /// amount to claim
    pub amount: Uint128,
    /// block at which amount can be claimed
    pub claimable_at: Expiration,
}

/// ## ClaimsResponse
/// This structure describes the fields for claims response message.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct ClaimsResponse {
    /// a list of bonder claims
    pub claims: Vec<ClaimInfoResponse>,
}
