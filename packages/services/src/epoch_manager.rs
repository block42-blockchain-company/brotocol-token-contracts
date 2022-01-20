use cosmwasm_std::{Decimal, Uint128};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct InstantiateMsg {
    pub epoch: u64,
    pub blocks_per_year: u64,
    pub bbro_emission_rate: Decimal,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    UpdateConfig {
        owner: Option<String>,
    },
    UpdateState {
        epoch: Option<u64>,
        blocks_per_year: Option<u64>,
        bbro_emission_rate: Option<Decimal>,
    },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    Config {},
    EpochInfo {},
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct ConfigResponse {
    pub owner: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct EpochInfoResponse {
    pub epoch: u64,
    pub blocks_per_year: u64,
    pub bbro_emission_rate: Decimal,
}

impl EpochInfoResponse {
    pub fn epochs_per_year(&self) -> Uint128 {
        Uint128::from(self.blocks_per_year / self.epoch)
    }
}
