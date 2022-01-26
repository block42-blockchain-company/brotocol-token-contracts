use cosmwasm_std::{CanonicalAddr, StdResult, Storage, Uint128};
use cw_storage_plus::Item;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// ## Description
/// Stores config struct of type [`Config`] at the given key
static CONFIG: Item<Config> = Item::new("config");

/// ## Description
/// Stores state struct of type [`State`] at the given key
static STATE: Item<State> = Item::new("state");

/// ## Description
/// This structure describes the main control config of distributor contract.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Config {
    /// contract/multisig address that allowed to control settings
    pub owner: CanonicalAddr,
    /// genesis block for destribution start
    pub distribution_genesis_block: u64,
    /// epoch manager contract addresss
    pub epoch_manager_contract: CanonicalAddr,
    /// rewards pool contract address
    pub rewards_contract: CanonicalAddr,
    /// staking contract address
    pub staking_contract: CanonicalAddr,
    /// amount per epoch to distribute for staking
    pub staking_distribution_amount: Uint128,
    /// bonding contract address
    pub bonding_contract: CanonicalAddr,
    /// amount per epoch to distribute for bonding
    pub bonding_distribution_amount: Uint128,
}

/// ## Description
/// This structure describes the state of distributor contract.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct State {
    /// last distribution block
    pub last_distribution_block: u64,
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

/// ## Description
/// Saves changes of [`State`] struct in [`STATE`] storage
/// ## Params
/// * **storage** is an object of type [`Storage`]
///
/// * **config** updated config struct of type [`State`]
pub fn store_state(storage: &mut dyn Storage, state: &State) -> StdResult<()> {
    STATE.save(storage, state)
}

/// ## Description
/// Returns state struct of type [`State`]
/// ## Params
/// * **storage** is an object of type [`Storage`]
pub fn load_state(storage: &dyn Storage) -> StdResult<State> {
    STATE.load(storage)
}
