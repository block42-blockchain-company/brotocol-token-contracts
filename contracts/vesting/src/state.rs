use cosmwasm_std::{Addr, CanonicalAddr, StdError, StdResult, Storage};
use cw_storage_plus::{Bound, Item, Map};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use services::{common::OrderBy, vesting::VestingInfo};

/// ## Description
/// Stores config struct of type [`Config`] at the given key
static CONFIG: Item<Config> = Item::new("config");

/// ## Description
/// A map which stores accounts vesting info with [`Addr`] type as a key and [`VestingInfo`] type as a value
static VESTING_INFO: Map<&Addr, VestingInfo> = Map::new("vesting_info");

/// ## Description
/// This structure describes the main control config of vesting contract.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Config {
    /// contract/multisig address that allowed to control settings
    pub owner: CanonicalAddr,
    /// bro token address
    pub bro_token: CanonicalAddr,
    /// genesis time frame for vesting schedules
    pub genesis_time: u64,
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
/// Saves or updates changes in [`VESTING_INFO`] map for specified key of type [`Addr`] and value of type [`VestingInfo`]
/// ## Params
/// * **storage** is an object of type [`Storage`]
///
/// * **addr** is an object of type [`Addr`]
///
/// * **vesting_info** is an object of type [`VestingInfo`]
pub fn store_vesting_info(
    storage: &mut dyn Storage,
    addr: &Addr,
    vesting_info: &VestingInfo,
) -> StdResult<()> {
    VESTING_INFO.save(storage, addr, vesting_info)
}

/// ## Description
/// Returns accounts vesting info object of type [`VestingInfo`] by specified key of type [`Addr`] from map [`VESTING_INFO`]
/// ## Params
/// * **storage** is an object of type [`Storage`]
///
/// * **addr** is an object of type [`Addr`]
pub fn load_vesting_info(storage: &dyn Storage, addr: &Addr) -> StdResult<VestingInfo> {
    VESTING_INFO.load(storage, addr)
}

/// max storage read limit
const MAX_LIMIT: u32 = 30;
/// default storage read limit
const DEFAULT_LIMIT: u32 = 10;

/// ## Description
/// Returns the empty vector if does not found data to read, otherwise returns the vector that
/// contains the objects of type [`VESTING_INFO`]
/// ## Params
/// * **storage** is an object of type [`Storage`]
///
/// * **start_after** is an [`Option`] field of type [`Addr`]. Sets the index to start reading
///
/// * **limit** is an [`Option`] field of type [`u32`]. Sets the limit to reading
///
/// * **order_by** is an [`Option`] field of type [`OrderBy`]
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
