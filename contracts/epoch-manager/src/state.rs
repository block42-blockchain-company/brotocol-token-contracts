use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use cosmwasm_std::{Storage, StdResult, CanonicalAddr, Decimal};
use cw_storage_plus::Item;

static CONFIG: Item<Config> = Item::new("config");
static STATE: Item<State> = Item::new("state");

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Config {
    pub owner: CanonicalAddr,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct State {
    pub epoch: u64,
    pub blocks_per_year: u64,
    pub bbro_emission_rate: Decimal,
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
