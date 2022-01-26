use cosmwasm_std::{Addr, CanonicalAddr, StdResult, Storage};
use cw_storage_plus::{Item, Map, U8Key};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// ## Description
/// Stores config struct of type [`Config`] at the given key
static CONFIG: Item<Config> = Item::new("config");

/// ## Description
/// Stores latest stage of type [`u8`] at the given key
static STAGE: Item<u8> = Item::new("stage");

/// ## Description
/// A map which stores merkle roots with [`U8Key`] type as a key and [`String`] type as a value
static MERKLE_ROOT: Map<U8Key, String> = Map::new("merkle_root");

/// ## Description
/// A map which stores claims info info with ([`Addr`], [`U8Key`]) type as a key and [`bool`] type as a value
static CLAIM: Map<(&Addr, U8Key), bool> = Map::new("claim");

/// ## Description
/// This structure describes the main control config of airdrop contract.
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

/// ## Description
/// Returns config struct of type [`Config`]
/// ## Params
/// * **storage** is an object of type [`Storage`]
pub fn load_config(storage: &dyn Storage) -> StdResult<Config> {
    CONFIG.load(storage)
}

/// ## Description
/// Saves changes of [`u8`] field in [`STAGE`] storage
/// ## Params
/// * **storage** is an object of type [`Storage`]
///
/// * **stage** updated latest stage of type [`u8`]
pub fn store_latest_stage(storage: &mut dyn Storage, stage: u8) -> StdResult<()> {
    STAGE.save(storage, &stage)
}

/// ## Description
/// Returns latest stage number of type [`u8`]
/// ## Params
/// * **storage** is an object of type [`Storage`]
pub fn load_stage(storage: &dyn Storage) -> StdResult<u8> {
    STAGE.load(storage)
}

/// ## Description
/// Saves or updates changes in [`MERKLE_ROOT`] map for specified key of type [`u8`] and value of type [`String`]
/// ## Params
/// * **storage** is an object of type [`Storage`]
///
/// * **stage** is a field of type [`u8`]
///
/// * **merkle_root** is a field of type [`String`]
#[allow(clippy::ptr_arg)]
pub fn store_merkle_root(
    storage: &mut dyn Storage,
    stage: u8,
    merkle_root: &String,
) -> StdResult<()> {
    MERKLE_ROOT.save(storage, U8Key::from(stage), merkle_root)
}

/// ## Description
/// Returns merle root string of type [`String`] by specified key of type [`u8`] from map [`MERKLE_ROOT`]
/// ## Params
/// * **storage** is an object of type [`Storage`]
///
/// * **stage** is a field of type [`u8`]
pub fn load_merkle_root(storage: &dyn Storage, stage: u8) -> StdResult<String> {
    MERKLE_ROOT.load(storage, U8Key::from(stage))
}

/// ## Description
/// Saves or updates changes in [`CLAIM`] map for specified key of type ([`Addr`], [`u8`]) and value of type [`bool`]
/// ## Params
/// * **storage** is an object of type [`Storage`]
///
/// * **user** is an object of type [`Addr`]
///
/// * **stage** is a field of type [`u8`]
pub fn store_claimed(storage: &mut dyn Storage, user: &Addr, stage: u8) -> StdResult<()> {
    CLAIM.save(storage, (user, U8Key::from(stage)), &true)
}

/// ## Description
/// Returns claim info of type [`bool`] by specified key of type ([`Addr`], [`u8`]) from map [`CLAIM`]
/// ## Params
/// * **storage** is an object of type [`Storage`]
///
/// * **user** is an object of type [`Addr`]
///
/// * **stage** is a field of type [`u8`]
pub fn read_claimed(storage: &dyn Storage, user: &Addr, stage: u8) -> StdResult<bool> {
    let res = CLAIM.may_load(storage, (user, U8Key::from(stage)))?;
    match res {
        Some(v) => Ok(v),
        None => Ok(false),
    }
}
