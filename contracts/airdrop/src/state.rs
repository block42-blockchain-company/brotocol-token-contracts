use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_std::{CanonicalAddr, Storage, StdResult, Addr};
use cw_storage_plus::{Item, U8Key, Map};

static CONFIG: Item<Config> = Item::new("config");
static STAGE: Item<u8> = Item::new("stage");
static MERKLE_ROOT: Map<U8Key, String> = Map::new("merkle_root");
static CLAIM: Map<(&Addr, U8Key), bool> = Map::new("claim");

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Config {
    pub owner: CanonicalAddr,
    pub bro_token: CanonicalAddr,
}

pub fn store_config(storage: &mut dyn Storage, config: &Config) -> StdResult<()> {
    CONFIG.save(storage, config)
}

pub fn load_config(storage: &dyn Storage) -> StdResult<Config> {
    CONFIG.load(storage)
}

pub fn store_latest_stage(storage: &mut dyn Storage, stage: u8) -> StdResult<()> {
    STAGE.save(storage, &stage)
}

pub fn load_stage(storage: &dyn Storage) -> StdResult<u8> {
    STAGE.load(storage)
}

pub fn store_merkle_root(storage: &mut dyn Storage, stage: u8, merkle_root: &String) -> StdResult<()> {
    MERKLE_ROOT.save(storage, U8Key::from(stage), merkle_root)
}

pub fn load_merkle_root(storage: &dyn Storage, stage: u8) -> StdResult<String> {
    MERKLE_ROOT.load(storage, U8Key::from(stage))
}

pub fn store_claimed(storage: &mut dyn Storage, user: &Addr, stage: u8) -> StdResult<()> {
    CLAIM.save(storage, (user, U8Key::from(stage)), &true)
}

pub fn read_claimed(storage: &dyn Storage, user: &Addr, stage: u8) -> StdResult<bool> {
    let res = CLAIM.may_load(storage, (user, U8Key::from(stage)))?;
    match res {
        Some(v) => Ok(v),
        None => Ok(false),
    }
}
