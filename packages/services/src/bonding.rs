use cosmwasm_std::{Decimal, Uint128};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use cw20::{Cw20ReceiveMsg, Expiration};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct InstantiateMsg {
    pub bro_token: String,
    pub lp_token: String,
    pub treasury_contract: String,
    pub astroport_factory: String,
    pub ust_bonding_reward_ratio: Decimal,
    pub ust_bonding_discount: Decimal,
    pub lp_bonding_discount: Decimal,
    pub vesting_period_blocks: u64,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    Receive(Cw20ReceiveMsg),
    UstBond {},
    Claim {},
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum Cw20HookMsg {
    DistributeReward {},
    LpBond {},
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    Config {},
    State {},
    Claims {
        address: String,
    },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct ConfigResponse {
    pub bro_token: String,
    pub lp_token: String,
    pub treasury_contract: String,
    pub ust_bonding_reward_ratio: Decimal,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct StateResponse {
    pub ust_bonding_balance: Uint128,
    pub lp_bonding_balance: Uint128,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct ClaimInfoResponse {
    pub bond_type: String,
    pub amount: Uint128,
    pub claimable_at: Expiration,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct ClaimsResponse {
    pub claims: Vec<ClaimInfoResponse>,
}
