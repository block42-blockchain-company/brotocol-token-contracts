use cosmwasm_std::{Uint128, Binary};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct InstantiateMsg {
    pub gov_contract: String,
    pub bro_token: String,
    pub spend_limit: Uint128,
    pub whitelist: Vec<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    UpdateConfig {
        new_gov_contract: Option<String>,
        bro_token: Option<String>,
        spend_limit: Option<Uint128>,
    },
    AddDistributor {
        distributor: String,
    },
    RemoveDistributor {
        distributor: String,
    },
    Reward {
        contract: String,
        amount: Uint128,
        msg: Binary,
    },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    Config {},
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct ConfigResponse {
    pub gov_contract: String,
    pub bro_token: String,
    pub spend_limit: Uint128,
    pub whitelist: Vec<String>,
}
