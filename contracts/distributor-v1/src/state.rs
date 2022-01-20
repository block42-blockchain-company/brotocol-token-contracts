use cosmwasm_std::{CanonicalAddr, StdResult, Storage, Uint128};
use cw_storage_plus::Item;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

static CONFIG: Item<Config> = Item::new("config");
static STATE: Item<State> = Item::new("state");

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Config {
    pub epoch_manager_contract: CanonicalAddr,
    pub rewards_contract: CanonicalAddr,
    pub staking_contract: CanonicalAddr,
    pub staking_distribution_amount: Uint128,
    pub bonding_contract: CanonicalAddr,
    pub bonding_distribution_amount: Uint128,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct State {
    pub last_distribution_block: u64,
}

pub fn store_config(storage: &mut dyn Storage, config: &Config) -> StdResult<()> {
    CONFIG.save(storage, config)
}

pub fn load_config(storage: &dyn Storage) -> StdResult<Config> {
    CONFIG.load(storage)
}

pub fn store_state(storage: &mut dyn Storage, state: &State) -> StdResult<()> {
    STATE.save(storage, state)
}

pub fn load_state(storage: &dyn Storage) -> StdResult<State> {
    STATE.load(storage)
}
