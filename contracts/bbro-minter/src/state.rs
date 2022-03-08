use cosmwasm_std::{CanonicalAddr, StdResult, Storage};
use cw_storage_plus::Item;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// ## Description
/// Stores config struct of type [`Config`] at the given key
static CONFIG: Item<Config> = Item::new("config");

/// ## Description
/// This structure describes the main control config of bbro minter contract.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Config {
    /// bbro token address
    pub bbro_token: Option<CanonicalAddr>,
    /// list of whitelisted addresses allowed to execute mint/burn functions
    pub whitelist: Vec<CanonicalAddr>,
}

/// ## Description
/// Saves changes of [`Config`] struct in [`CONFIG`] storage
/// ## Params
/// * **storage** is an object of type [`Storage`]
///
/// * **config** updated config struct of type [`Config`]
pub fn store_config(storage: &mut dyn Storage, config: &Config) -> StdResult<()> {
    CONFIG.save(storage, config)
}

/// ## Description
/// Returns config struct of type [`Config`]
/// ## Params
/// * **storage** is an object of type [`Storage`]
pub fn load_config(storage: &dyn Storage) -> StdResult<Config> {
    CONFIG.load(storage)
}
