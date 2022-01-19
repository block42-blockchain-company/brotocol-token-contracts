use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use cosmwasm_std::{CanonicalAddr, Uint128, Storage, StdResult};
use cw_storage_plus::Item;

static CONFIG: Item<Config> = Item::new("config");

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Config {
    pub gov_contract: CanonicalAddr,
    pub bro_token: CanonicalAddr,
    pub spend_limit: Uint128,
    pub whitelist: Vec<CanonicalAddr>,
}

pub fn store_config(storage: &mut dyn Storage, config: &Config) -> StdResult<()> {
    CONFIG.save(storage, config)
}

pub fn load_config(storage: &dyn Storage) -> StdResult<Config> {
    CONFIG.load(storage)
}
