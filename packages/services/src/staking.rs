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
    pub unstake_period_blocks: u64,
    /// minimum staking amount
    pub min_staking_amount: Uint128,
    /// min lockup period
    pub min_lockup_period_epochs: u64,
    /// max lockup period
    pub max_lockup_period_epochs: u64,
    /// base rate for bbro premium reward calculation
    pub base_rate: Decimal,
    /// linear growth for bbro premium reward calculation
    pub linear_growth: Decimal,
    /// exponential growth for bbro premium reward calculation
    pub exponential_growth: Decimal,
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
    /// Lockup unlocked staked amount
    LockupStaked {
        /// amount of tokens to lock
        amount: Uint128,
        /// how many epochs specified amount will be locked
        epochs_locked: u64,
    },
    /// ## Description
    /// Unstake staked amount of tokens.
    /// Tokens will be claimable only after passing the unstaking period.
    Unstake {
        /// amount of tokens to unstake
        amount: Uint128,
    },
    /// ## Description
    /// Withdraw the amount of tokens that have already passed the unstaking period.
    Withdraw {},
    /// ## Description
    /// Claim availalble bro reward amount
    ClaimStakingRewards {},
    /// ## Description
    /// Claim availalble bbro reward amount
    ClaimBbroRewards {},
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
    Stake {
        /// staking type
        stake_type: StakeType,
    },
}

/// ## StakeType
/// This structure describes the stake type.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum StakeType {
    /// ## Description
    /// Type of staking when staked amount will be unlocked
    Unlocked {},
    /// ## Description
    /// Type of staking when staked amount will be locked
    /// for specified amount of epochs
    Locked {
        /// how many epochs specified amount will be locked
        epochs_locked: u64,
    },
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
    pub unstake_period_blocks: u64,
    /// minimum staking amount
    pub min_staking_amount: Uint128,
    /// lockup config
    pub lockup_config: LockupConfigResponse,
}

/// ## LockupConfigResponse
/// This structure describes the fields for lockup config response message.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct LockupConfigResponse {
    /// min lockup period
    pub min_lockup_period_epochs: u64,
    /// max lockup period
    pub max_lockup_period_epochs: u64,
    /// base rate for bbro premium reward calculation
    pub base_rate: Decimal,
    /// linear growth for bbro premium reward calculation
    pub linear_growth: Decimal,
    /// exponential growth for bbro premium reward calculation
    pub exponential_growth: Decimal,
}

/// ## StateResponse
/// This structure describes the fields for state response message.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct StateResponse {
    /// global reward index for BRO staking rewards
    pub global_reward_index: Decimal,
    /// total amount of staked BRO tokens by all stakers
    pub total_stake_amount: Uint128,
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
    /// amount of unlocked BRO tokens staked by staker
    pub unlocked_stake_amount: Uint128,
    /// amount of locked BRO tokens staked by staker
    pub locked_stake_amount: Uint128,
    /// amount of pending rewards of staker
    pub pending_reward: Uint128,
    /// last balance update(stake, unstake) block
    pub last_balance_update: u64,
    /// amounts locked for specified amount of epochs
    pub lockups: Vec<LockupInfoResponse>,
}

/// ## Description
/// This structure describes the fields for lockup info response message.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct LockupInfoResponse {
    /// locked amount
    pub amount: Uint128,
    /// block at which amount will be unlocked
    pub unlocked_at: Expiration,
}

/// ## StakerAccruedRewardsResponse
/// This structure describes the fields for staker accrued rewards response message.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct StakerAccruedRewardsResponse {
    /// amount of pending rewards of staker
    pub rewards: Uint128,
    /// amount of bBRO reward from staking BRO tokens
    pub bbro_stake_reward: Uint128,
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
