use cosmwasm_std::{Uint128, Decimal};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use cw20::{Cw20ReceiveMsg, Expiration};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct InstantiateMsg {
    pub bro_token: String,
    pub rewards_pool_contract: String,
    pub bbro_minter_contract: String,
    pub epoch_manager_contract: String,
    pub unbond_period_blocks: u64,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    Receive(Cw20ReceiveMsg),
    Unbond {
        amount: Uint128,
    },
    Withdraw {},
    ClaimRewards {},
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum Cw20HookMsg {
    DistributeReward {
        distributed_at_block: u64,
    },
    Bond {},
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    Config {},
    State {},
    StakerInfo {
        staker: String,
    },
    StakerAccruedRewards {
        staker: String,
    },
    Withdrawals {
        staker: String,
    },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct ConfigResponse {
    pub bro_token: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct StateResponse {
    pub global_reward_index: Decimal,
    pub total_bond_amount: Uint128,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct StakerInfoResponse {
    pub staker: String,
    pub reward_index: Decimal,
    pub bond_amount: Uint128,
    pub pending_reward: Uint128,
    pub last_balance_update: u64,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct StakerAccruedRewardsResponse {
    pub rewards: Uint128,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct WithdrawalInfoResponse {
    pub amount: Uint128,
    pub claimable_at: Expiration,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct WithdrawalsResponse {
    pub claims: Vec<WithdrawalInfoResponse>,
}
