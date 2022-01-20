use cosmwasm_std::{Addr, CanonicalAddr, StdResult, Storage};
use cw_storage_plus::{Bound, Item, Map};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use services::{common::OrderBy, vesting::VestingInfo};

static CONFIG: Item<Config> = Item::new("config");
static VESTING_INFO: Map<&Addr, VestingInfo> = Map::new("vesting_info");

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Config {
    pub owner: CanonicalAddr,
    pub bro_token: CanonicalAddr,
    pub genesis_time: u64,
}

pub fn store_config(storage: &mut dyn Storage, config: &Config) -> StdResult<()> {
    CONFIG.save(storage, config)
}

pub fn load_config(storage: &dyn Storage) -> StdResult<Config> {
    CONFIG.load(storage)
}

pub fn store_vesting_info(
    storage: &mut dyn Storage,
    addr: &Addr,
    vesting_info: &VestingInfo,
) -> StdResult<()> {
    VESTING_INFO.save(storage, addr, vesting_info)
}

pub fn load_vesting_info(storage: &dyn Storage, addr: &Addr) -> StdResult<VestingInfo> {
    VESTING_INFO.load(storage, addr)
}

const MAX_LIMIT: u32 = 30;
const DEFAULT_LIMIT: u32 = 10;
pub fn read_vesting_infos(
    storage: &dyn Storage,
    start_after: Option<Addr>,
    limit: Option<u32>,
    order_by: Option<OrderBy>,
) -> StdResult<Vec<(Addr, VestingInfo)>> {
    let limit = limit.unwrap_or(DEFAULT_LIMIT).min(MAX_LIMIT) as usize;
    let (start, end, order_by) = match order_by {
        Some(OrderBy::Asc) => (
            calc_range_start_addr(start_after).map(Bound::exclusive),
            None,
            OrderBy::Asc,
        ),
        _ => (
            None,
            calc_range_end_addr(start_after).map(Bound::exclusive),
            OrderBy::Desc,
        ),
    };

    VESTING_INFO
        .range(storage, start, end, order_by.into())
        .take(limit)
        .map(|item| {
            let (k, info) = item?;
            let addr_str = std::str::from_utf8(&k)?;
            Ok((Addr::unchecked(addr_str), info))
        })
        .collect()
}

fn calc_range_start_addr(start_after: Option<Addr>) -> Option<Vec<u8>> {
    start_after.map(|addr| {
        let mut v = addr.as_bytes().to_vec();
        v.push(1);
        v
    })
}

fn calc_range_end_addr(start_after: Option<Addr>) -> Option<Vec<u8>> {
    start_after.map(|addr| addr.as_bytes().to_vec())
}
