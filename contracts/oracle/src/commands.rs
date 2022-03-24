use cosmwasm_bignumber::{Decimal256, Uint256};
use cosmwasm_std::{Attribute, DepsMut, Env, Response};

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
/// * **price_update_interval** is an [`Option`] field of type [`u64`]. Sets new price update interval
///
/// * **price_validity_period** is an [`Option`] field of type [`u64`]. Sets new price validity time frame
pub fn update_config(
    deps: DepsMut,
    price_update_interval: Option<u64>,
    price_validity_period: Option<u64>,
) -> Result<Response, ContractError> {
    let mut config = load_config(deps.storage)?;

    let mut attributes: Vec<Attribute> = vec![Attribute::new("action", "update_config")];

    if let Some(price_update_interval) = price_update_interval {
        config.price_update_interval = price_update_interval;
        attributes.push(Attribute::new(
            "price_update_interval_changed",
            &price_update_interval.to_string(),
        ));
    }

    if let Some(price_validity_period) = price_validity_period {
        config.price_validity_period = price_validity_period;
        attributes.push(Attribute::new(
            "price_validity_period_changed",
            &price_validity_period.to_string(),
        ));
    }

    store_config(deps.storage, &config)?;
    Ok(Response::new().add_attributes(attributes))
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
        return Ok(Response::default());
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
