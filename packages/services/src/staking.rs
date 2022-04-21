use cosmwasm_std::{Binary, Decimal, Uint128};
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
    /// rewards pool address
    pub rewards_pool_contract: String,
    /// bbro minter address
    pub bbro_minter_contract: String,
    /// epoch manager contract address
    pub epoch_manager_contract: String,
    /// community bonding address,
    /// if value is set to none
    /// than option to stake from community bonding contract is disabled
    pub community_bonding_contract: Option<String>,
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
    /// Claim available bro reward amount
    ClaimBroRewards {},
    /// ## Description
    /// Claim available bbro reward amount
    ClaimBbroRewards {},
    /// ## Description
    /// Updates contract settings
    /// ## Executor
    /// Only owner can execute this function
    UpdateConfig {
        /// defines either contract paused or not
        paused: Option<bool>,
        /// vesting period for withdrawal
        unstake_period_blocks: Option<u64>,
        /// minimum staking amount
        min_staking_amount: Option<Uint128>,
        /// min lockup period
        min_lockup_period_epochs: Option<u64>,
        /// max lockup period
        max_lockup_period_epochs: Option<u64>,
        /// base rate for bbro premium reward calculation
        base_rate: Option<Decimal>,
        /// linear growth for bbro premium reward calculation
        linear_growth: Option<Decimal>,
        /// exponential growth for bbro premium reward calculation
        exponential_growth: Option<Decimal>,
        /// community bonding contract
        community_bonding_contract: Option<String>,
    },
    UpdateStakerLockups {
        stakers: Vec<String>,
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
    /// ## Description
    /// Locks bonded amount of tokens via community bonding contract
    /// to get reward shares
    /// ## Executor
    /// Only community bonding contract can execute this function
    CommunityBondLock {
        /// address which performed bond via community bonding contract
        sender: String,
        /// how many epochs specified amount will be locked
        epochs_locked: u64,
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
    /// Returns available withdrawals for staker by specified address
    /// in the [`WithdrawalsResponse`] object
    Withdrawals {
        /// staker address
        staker: String,
    },
    StakersWithDeprecatedLockups {
        skip: u32,
        limit: Option<u32>,
    },
    /// ## Description
    /// Returns information about created ownership proposal in the [`OwnershipProposalResponse`] object
    /// otherwise returns not-found error
    OwnershipProposal {},
}

/// ## MigrateMsg
/// This structure describes a migration message.
/// We currently take no arguments for migrations.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct MigrateMsg {
    /// params for performing migration
    pub params: Binary,
}

/// ## ConfigResponse
/// This structure describes the fields for config response message.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct ConfigResponse {
    /// contract/multisig address that allowed to control settings
    pub owner: String,
    /// defines either contract paused or not
    pub paused: bool,
    /// bro token address
    pub bro_token: String,
    /// rewards pool address
    pub rewards_pool_contract: String,
    /// bbro minter address
    pub bbro_minter_contract: String,
    /// epoch manager contract address
    pub epoch_manager_contract: String,
    /// community bonding address,
    /// if value is set to none
    /// than option to stake from community bonding contract is disabled
    pub community_bonding_contract: Option<String>,
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
    /// amount of pending bro rewards of staker
    pub pending_bro_reward: Uint128,
    /// amount of pending bbro rewards of staker
    pub pending_bbro_reward: Uint128,
    /// last balance update(stake, unstake, claim) block
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
    /// block at whick locup was created
    pub locked_at_block: u64,
    /// amount of epochs until lockup will be unlocked
    pub epochs_locked: u64,
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
