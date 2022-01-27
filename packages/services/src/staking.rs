use cosmwasm_std::{Decimal, Uint128};
use cw20::{Cw20ReceiveMsg, Expiration};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// ## InstantiateMsg
/// This structure describes the basic settings for creating a contract.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct InstantiateMsg {
    /// bro token address
    pub bro_token: String,
    /// rewards pool address
    pub rewards_pool_contract: String,
    /// bbro minter address
    pub bbro_minter_contract: String,
    /// epoch manager contract address
    pub epoch_manager_contract: String,
    /// vesting period for withdrawal
    pub unbond_period_blocks: u64,
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
    /// Unbond staked amount of tokens.
    /// Tokens will be claimable only after passing unbonding period.
    Unbond {
        /// amount of tokens to unbond
        amount: Uint128,
    },
    /// ## Description
    /// Withdraw amount of tokens which have already passed unbonding period.
    Withdraw {},
    /// ## Description
    /// Claim availalble reward amount
    ClaimRewards {},
}

/// ## Cw20HookMsg
/// This structure describes the cw20 receive hook messages of the contract.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum Cw20HookMsg {
    /// ## Description
    /// Distributes received reward
    DistributeReward {
        /// last rewards distribution block
        distributed_at_block: u64,
    },
    /// ## Description
    /// Deposits specified amount of tokens to get reward shares
    Bond {},
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
    /// Returns staker info by specified address in the [`StakerInfoResponse`] object
    StakerInfo {
        /// staker address
        staker: String,
    },
    /// ## Description
    /// Returns available amount for staker to claim by specified address
    /// in the [`StakerAccruedRewardsResponse`] object
    StakerAccruedRewards {
        /// staker address
        staker: String,
    },
    /// ## Description
    /// Returns available withdrawals for staker by specified address
    /// in the [`WithdrawalsResponse`] object
    Withdrawals {
        /// staker address
        staker: String,
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
    /// bro token address
    pub bro_token: String,
    /// rewards pool address
    pub rewards_pool_contract: String,
    /// bbro minter address
    pub bbro_minter_contract: String,
    /// epoch manager contract address
    pub epoch_manager_contract: String,
    /// vesting period for withdrawal
    pub unbond_period_blocks: u64,
}

/// ## StateResponse
/// This structure describes the fields for state response message.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct StateResponse {
    /// global reward index for BRO staking rewards
    pub global_reward_index: Decimal,
    /// total amount of bonded BRO tokens by all stakers
    pub total_bond_amount: Uint128,
    /// last rewards distribution block
    pub last_distribution_block: u64,
}

/// ## StakerInfoResponse
/// This structure describes the fields for staker info response message.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct StakerInfoResponse {
    /// staker address
    pub staker: String,
    /// reward index of staker
    pub reward_index: Decimal,
    /// amount of BRO tokens bonded by staker
    pub bond_amount: Uint128,
    /// amount of pending rewards of staker
    pub pending_reward: Uint128,
    /// last balance update(bond, unbond) block
    pub last_balance_update: u64,
}

/// ## StakerAccruedRewardsResponse
/// This structure describes the fields for staker accrued rewards response message.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct StakerAccruedRewardsResponse {
    /// amount of pending rewards of staker
    pub rewards: Uint128,
    /// amount of bBRO reward from staking BRO tokens
    pub bbro_staking_reward: Uint128,
}

/// ## WithdrawalInfoResponse
/// This structure describes the fields for withdrawal info response message.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct WithdrawalInfoResponse {
    /// amount to withdraw
    pub amount: Uint128,
    /// block at which amount can be claimed
    pub claimable_at: Expiration,
}

/// ## WithdrawalsResponse
/// This structure describes the fields for withdrawals response message.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct WithdrawalsResponse {
    /// a list of staker's withdrawals
    pub claims: Vec<WithdrawalInfoResponse>,
}
