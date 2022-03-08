use cosmwasm_std::{Addr, Api, CanonicalAddr, StdError, StdResult, Storage};
use cw_storage_plus::Item;

static OWNER: Item<CanonicalAddr> = Item::new("contract-owner");

pub fn assert_owner(storage: &dyn Storage, api: &dyn Api, sender: Addr) -> StdResult<()> {
    if load_owner_addr(storage, api)? != sender {
        return Err(StdError::generic_err("Unauthorized"));
    }

    Ok(())
}

pub fn store_owner(storage: &mut dyn Storage, owner: &CanonicalAddr) -> StdResult<()> {
    OWNER.save(storage, owner)
}

pub fn change_owner(storage: &mut dyn Storage, new_owner: &CanonicalAddr) -> StdResult<()> {
    OWNER.remove(storage);
    OWNER.save(storage, new_owner)
}

pub fn load_owner_canonical(storage: &dyn Storage) -> StdResult<CanonicalAddr> {
    OWNER.load(storage)
}

pub fn load_owner_addr(storage: &dyn Storage, api: &dyn Api) -> StdResult<Addr> {
    api.addr_humanize(&load_owner_canonical(storage)?)
}

pub fn load_owner_str(storage: &dyn Storage, api: &dyn Api) -> StdResult<String> {
    Ok(load_owner_addr(storage, api)?.to_string())
}
