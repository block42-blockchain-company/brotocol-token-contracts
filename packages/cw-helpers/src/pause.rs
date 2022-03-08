use cosmwasm_std::{StdError, StdResult, Storage};
use cw_storage_plus::Item;

static PAUSE: Item<bool> = Item::new("contract-pause");

pub fn assert_not_paused(storage: &dyn Storage) -> StdResult<()> {
    if PAUSE.load(storage)? {
        return Err(StdError::generic_err("Contract is paused"));
    }

    Ok(())
}

pub fn store_pause(storage: &mut dyn Storage, pause: &bool) -> StdResult<()> {
    PAUSE.save(storage, pause)
}

pub fn load_pause(storage: &dyn Storage) -> StdResult<bool> {
    PAUSE.load(storage)
}
