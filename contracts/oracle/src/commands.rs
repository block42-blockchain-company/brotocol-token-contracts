use cosmwasm_bignumber::{Decimal256, Uint256};
use cosmwasm_std::{DepsMut, Env, Response};

use crate::{
    error::ContractError,
    state::{load_config, load_price_cumulative_last, store_config, store_price_cumulative_last},
};

use services::querier::query_cumulative_prices;

/// ## Description
/// Updates contract settings.
/// Returns [`Response`] with specified attributes and messages if operation was successful,
/// otherwise returns [`ContractError`]
/// ## Params
/// * **deps** is an object of type [`DepsMut`]
///
/// * **owner** is an [`Option`] field of type [`String`]. Sets new contract owner address
///
/// * **price_update_interval** is an [`Option`] field of type [`u64`]. Sets new price update interval
pub fn update_config(
    deps: DepsMut,
    owner: Option<String>,
    price_update_interval: Option<u64>,
) -> Result<Response, ContractError> {
    let mut config = load_config(deps.storage)?;

    if let Some(owner) = owner {
        config.owner = deps.api.addr_canonicalize(&owner)?;
    }

    if let Some(price_update_interval) = price_update_interval {
        config.price_update_interval = price_update_interval;
    }

    store_config(deps.storage, &config)?;
    Ok(Response::new().add_attribute("action", "update_config"))
}

/// ## Description
/// Updates cumulative prices.
/// Returns [`Response`] with specified attributes and messages if operation was successful,
/// otherwise returns [`ContractError`]
/// ## Params
/// * **deps** is an object of type [`DepsMut`]
///
/// * **env** is an object of type [`Env`]
pub fn update_price(deps: DepsMut, env: Env) -> Result<Response, ContractError> {
    let config = load_config(deps.storage)?;
    let mut price_last = load_price_cumulative_last(deps.storage)?;

    let current_time = env.block.time.seconds();
    let time_elapsed = current_time - price_last.last_price_update_timestamp;

    // ensure that at least one full period has passed since the last update
    if time_elapsed < config.price_update_interval {
        return Err(ContractError::UpdatePriceIntervalError {})
    }

    let prices = query_cumulative_prices(&deps.querier, config.pair.contract_addr)?;

    price_last.price_0_average = Decimal256::from_ratio(
        Uint256::from(
            prices
                .price0_cumulative_last
                .wrapping_sub(price_last.price_0_cumulative_last),
        ),
        time_elapsed,
    );

    price_last.price_1_average = Decimal256::from_ratio(
        Uint256::from(
            prices
                .price1_cumulative_last
                .wrapping_sub(price_last.price_1_cumulative_last),
        ),
        time_elapsed,
    );

    price_last.price_0_cumulative_last = prices.price0_cumulative_last;
    price_last.price_1_cumulative_last = prices.price1_cumulative_last;
    price_last.last_price_update_timestamp = current_time;
    store_price_cumulative_last(deps.storage, &price_last)?;

    Ok(Response::new().add_attribute("action", "update_price"))
}
