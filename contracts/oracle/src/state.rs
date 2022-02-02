use cosmwasm_bignumber::Decimal256;
use cosmwasm_std::{CanonicalAddr, StdResult, Storage, Uint128};
use cw_storage_plus::Item;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use astroport::asset::{AssetInfo, PairInfo};

/// ## Description
/// Stores config struct of type [`Config`] at the given key
static CONFIG: Item<Config> = Item::new("config");

/// ## Description
/// Stores struct with pair prices of type [`PriceCumulativeLast`] at the given key
static PRICE_LAST: Item<PriceCumulativeLast> = Item::new("price_last");

/// ## Description
/// Contract global configuration
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Config {
    /// contract/multisig address that allowed to control settings
    pub owner: CanonicalAddr,
    /// factory contract address
    pub factory: CanonicalAddr,
    /// assets in the pool
    pub asset_infos: [AssetInfo; 2],
    /// pair info
    pub pair: PairInfo,
    /// time interval for updating prices
    pub price_update_interval: u64,
}

/// ## Description
/// This structure describes the main controls configs of pair
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct PriceCumulativeLast {
    /// last cumulative price 0 asset in pool
    pub price_0_cumulative_last: Uint128,
    /// last cumulative price 1 asset in pool
    pub price_1_cumulative_last: Uint128,
    /// average price 0 asset in pool
    pub price_0_average: Decimal256,
    /// average price 1 asset in pool
    pub price_1_average: Decimal256,
    /// last timestamp block in pool
    pub last_price_update_timestamp: u64,
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
/// Saves changes of [`PriceCumulativeLast`] struct in [`PRICE_LAST`] storage
/// ## Params
/// * **storage** is an object of type [`Storage`]
///
/// * **price_last** updated config struct of type [`PriceCumulativeLast`]
pub fn store_price_cumulative_last(
    storage: &mut dyn Storage,
    price_last: &PriceCumulativeLast,
) -> StdResult<()> {
    PRICE_LAST.save(storage, price_last)
}

// ## Description
/// Returns struct with pair price info of type [`PriceCumulativeLast`]
/// ## Params
/// * **storage** is an object of type [`Storage`]
pub fn load_price_cumulative_last(storage: &dyn Storage) -> StdResult<PriceCumulativeLast> {
    PRICE_LAST.load(storage)
}
