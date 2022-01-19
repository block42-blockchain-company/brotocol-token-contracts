use cosmwasm_std::Uint128;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct InstantiateMsg {
    pub gov_contract: String,
    pub bbro_token: String,
    pub whitelist: Vec<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    UpdateConfig {
        new_gov_contract: Option<String>,
        bbro_token: Option<String>,
    },
    AddMinter {
        minter: String,
    },
    RemoveMinter {
        minter: String,
    },
    Mint {
        recipient: String,
        amount: Uint128,
    },
    Burn {
        owner: String,
        amount: Uint128,
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
    pub bbro_token: String,
    pub whitelist: Vec<String>,
}
