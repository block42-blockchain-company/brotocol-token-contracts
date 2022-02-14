use cosmwasm_std::{CanonicalAddr, StdResult, Storage, Uint128};
use cw_storage_plus::{Item, Map};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

static CONFIG: Item<Config> = Item::new("config");

static STATE: Item<State> = Item::new("state");

static WHITELISTED_ACCOUNTS: Map<&[u8], Uint128> = Map::new("whitelist");

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Config {
    pub owner: CanonicalAddr,
    pub bro_token: CanonicalAddr,
    pub bro_price_per_uusd: Uint128,
    pub bro_amount_per_nft: Uint128,
    pub treasury_contract: CanonicalAddr,
    pub rewards_pool_contract: CanonicalAddr,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct State {
    pub sale_registered: bool,
    pub sale_start_time: u64,
    pub sale_end_time: u64,
    pub balance: Uint128,
}

impl State {
    pub fn sale_is_on(&self, current_time: u64) -> bool {
        if !self.sale_registered
            || self.sale_start_time > current_time
            || self.sale_end_time < current_time
        {
            return false;
        }

        true
    }

    pub fn sale_finished(&self, current_time: u64) -> bool {
        if self.sale_registered && self.sale_end_time < current_time {
            return true;
        }

        false
    }
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
/// * **state** updated config struct of type [`State`]
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

pub fn store_whitelisted_account(
    storage: &mut dyn Storage,
    account: &CanonicalAddr,
    amount: &Uint128,
) -> StdResult<()> {
    WHITELISTED_ACCOUNTS.save(storage, account.as_slice(), amount)
}

pub fn load_whitelisted_account(
    storage: &dyn Storage,
    account: &CanonicalAddr,
) -> StdResult<Uint128> {
    WHITELISTED_ACCOUNTS.load(storage, account.as_slice())
}
