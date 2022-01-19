use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use cosmwasm_std::{CanonicalAddr, Storage, StdResult};
use cw_storage_plus::Item;

static CONFIG: Item<Config> = Item::new("config");

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Config {
    pub owner: CanonicalAddr,
}

pub fn store_config(storage: &mut dyn Storage, config: &Config) -> StdResult<()> {
    CONFIG.save(storage, config)
}

pub fn load_config(storage: &dyn Storage) -> StdResult<Config> {
    CONFIG.load(storage)
}
