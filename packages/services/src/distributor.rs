use cosmwasm_std::Uint128;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct InstantiateMsg {
    pub epoch_manager_contract: String,
    pub rewards_contract: String,
    pub staking_contract: String,
    pub staking_distribution_amount: Uint128,
    pub bonding_contract: String,
    pub bonding_distribution_amount: Uint128,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    Distribute {},
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    Config {},
    LastDistribution {},
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct ConfigResponse {
    pub epoch_manager_contract: String,
    pub rewards_contract: String,
    pub staking_contract: String,
    pub staking_distribution_amount: Uint128,
    pub bonding_contract: String,
    pub bonding_distribution_amount: Uint128,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct LastDistributionResponse {
    pub last_distribution_block: u64,
}
