use cosmwasm_std::{CanonicalAddr, StdError, StdResult, Storage, Uint128};
use cw_storage_plus::{Item, Map};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// ## Description
/// Stores config struct of type [`Config`] at the given key
static CONFIG: Item<Config> = Item::new("config");

/// ## Description
/// Stores state struct of type [`State`] at the given key
static STATE: Item<State> = Item::new("state");

/// ## Description
/// A map which stores whitelisted addresses info with [`CanonicalAddr`] type as key and [`Uint128`] type as value
static WHITELISTED_ACCOUNTS: Map<&[u8], Uint128> = Map::new("whitelist");

/// ## Description
/// This structure describes the main control config of whitelist sale contract.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Config {
    /// contract/multisig address that allowed to control settings
    pub owner: CanonicalAddr,
    /// bro token address
    pub bro_token: CanonicalAddr,
    /// bro amount per uusd
    pub bro_amount_per_uusd: Uint128,
    /// bro amount for purchase per nft
    pub bro_amount_per_nft: Uint128,
    /// address for sending received ust
    pub ust_receiver: CanonicalAddr,
    /// rewards pool address
    pub rewards_pool_contract: CanonicalAddr,
}

/// ## Description
/// This structure describes state of whitelist sale contract.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema, Default)]
pub struct State {
    /// sets sale either to registered or not
    pub sale_registered: bool,
    /// sale start time
    pub sale_start_time: u64,
    /// sale end time
    pub sale_end_time: u64,
    /// required transfer amount to register sale
    pub required_transfer_amount: Uint128,
    /// remaining contract balance
    pub balance: Uint128,
}

impl State {
    pub fn sale_is_live(&self, current_time: u64) -> bool {
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

/// ## Description
/// Saves or updates changes in [`WHITELISTED_ACCOUNTS`] map for specified key of type [`CanonicalAddr`] and value of type [`Uint128`]
/// ## Params
/// * **storage** is an object of type [`Storage`]
///
/// * **account** is an object of type [`CanonicalAddr`]
///
/// * **amount** is an object of type [`Uint128`]
pub fn store_whitelisted_account(
    storage: &mut dyn Storage,
    account: &CanonicalAddr,
    amount: &Uint128,
) -> StdResult<()> {
    WHITELISTED_ACCOUNTS.save(storage, account.as_slice(), amount)
}

/// ## Description
/// Returns available purchase amount for whitelisted address of type [`Uint128`] by specified key of type [`CanonicalAddr`] from map [`CLAIMS`]
/// ## Params
/// * **storage** is an object of type [`Storage`]
///
/// * **account** is an object of type [`CanonicalAddr`]
pub fn load_whitelisted_account(
    storage: &dyn Storage,
    account: &CanonicalAddr,
) -> StdResult<Uint128> {
    WHITELISTED_ACCOUNTS.load(storage, account.as_slice())
}
