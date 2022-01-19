use cosmwasm_std::{Storage, StdResult, CanonicalAddr};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use cw_storage_plus::Item;

static CONFIG: Item<Config> = Item::new("config");

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Config {
    pub gov_contract: CanonicalAddr,
    pub bbro_token: CanonicalAddr,
    pub whitelist: Vec<CanonicalAddr>,
}

pub fn store_config(storage: &mut dyn Storage, config: &Config) -> StdResult<()> {
    CONFIG.save(storage, config)
}

pub fn load_config(storage: &dyn Storage) -> StdResult<Config> {
    CONFIG.load(storage)
}
