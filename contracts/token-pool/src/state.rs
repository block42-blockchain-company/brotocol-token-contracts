use cosmwasm_std::{CanonicalAddr, StdError, StdResult, Storage};
use cw_storage_plus::Item;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// ## Description
/// Stores config struct of type [`Config`] at the given key
static CONFIG: Item<Config> = Item::new("config");

/// ## Description
/// This structure describes the main control config of token pool contract.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Config {
    /// contract/multisig address that allowed to control settings
    pub owner: CanonicalAddr,
    /// bro token address
    pub bro_token: CanonicalAddr,
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

// ## Description
/// Returns config struct of type [`Config`]
/// ## Params
/// * **storage** is an object of type [`Storage`]
pub fn load_config(storage: &dyn Storage) -> StdResult<Config> {
    CONFIG.load(storage)
}

/// ## Description
/// Updates owner field in [`Config`] object
/// ## Params
/// * **storage** is an object of type [`Storage`]
///
/// * **new_owner** is an object of type [`CanonicalAddr`]
pub fn update_owner(storage: &mut dyn Storage, new_owner: CanonicalAddr) -> StdResult<()> {
    CONFIG.update::<_, StdError>(storage, |mut c| {
        c.owner = new_owner;
        Ok(c)
    })?;

    Ok(())
}
