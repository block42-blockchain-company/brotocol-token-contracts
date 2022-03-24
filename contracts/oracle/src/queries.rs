use cosmwasm_bignumber::{Decimal256, Uint256};
use cosmwasm_std::{Deps, Env, StdError, StdResult, Uint128};

use crate::state::{load_config, load_price_cumulative_last};

use services::{
    oracle::{ConfigResponse, ConsultPriceResponse},
    querier::query_prices,
};

use astroport::{
    asset::{Asset, AssetInfo},
    pair::TWAP_PRECISION,
    querier::query_token_precision,
};

/// ## Description
/// Returns rewards pool contract config in the [`ConfigResponse`] object
/// ## Params
/// * **deps** is an object of type [`Deps`]
pub fn query_config(deps: Deps) -> StdResult<ConfigResponse> {
    let config = load_config(deps.storage)?;
    let resp = ConfigResponse {
        owner: deps.api.addr_humanize(&config.owner)?.to_string(),
        factory: deps.api.addr_humanize(&config.factory)?.to_string(),
        asset_infos: config.asset_infos,
        price_update_interval: config.price_update_interval,
        price_validity_period: config.price_validity_period,
    };

    Ok(resp)
}

/// ## Description
/// Returns calculated average amount with updated precision in the [`ConsultPriceResponse`] object
/// ## Params
/// * **deps** is an object of type [`Deps`]
///
/// * **env** is an object of type [`Env`].
/// 
/// * **asset** is an object of type [`AssetInfo`]
///
/// * **amount** is an object of type [`Uint128`]
pub fn consult_price(
    deps: Deps,
    env: Env,
    asset: AssetInfo,
    amount: Uint128,
) -> StdResult<ConsultPriceResponse> {
    let config = load_config(deps.storage)?;
    let price_last = load_price_cumulative_last(deps.storage)?;

    let current_time = env.block.time.seconds();
    let time_elapsed = current_time - price_last.last_price_update_timestamp;

    // return Error if last price update happened too long ago
    if time_elapsed > config.price_validity_period {
        return Err(StdError::generic_err("Last price update is too old. Invoke the UpdatePrice function!"));
    }

    let price_average = if config.asset_infos[0].equal(&asset) {
        price_last.price_0_average
    } else if config.asset_infos[1].equal(&asset) {
        price_last.price_1_average
    } else {
        return Err(StdError::generic_err("Invalid asset info"));
    };

    let consult_price = if price_average.is_zero() {
        let precision = query_token_precision(&deps.querier, asset.clone())?;
        let one = Uint128::from(10_u128.pow(precision.into()));

        let price = query_prices(
            &deps.querier,
            config.pair.contract_addr,
            Asset {
                info: asset,
                amount: one,
            },
        )?
        .return_amount;

        Uint256::from(price).multiply_ratio(Uint256::from(amount), Uint256::from(one))
    } else {
        let price_precision = Uint256::from(10_u128.pow(TWAP_PRECISION.into()));
        Uint256::from(amount) * price_average / Decimal256::from_uint256(price_precision)
    };

    Ok(ConsultPriceResponse {
        amount: consult_price.into(),
    })
}

/// ## Description
/// Returns a [`bool`] type whether prices are ready to be updated or not
/// ## Params
/// * **deps** is an object of type [`Deps`]
///
/// * **env** is an object of type [`Env`].
pub fn is_ready_to_trigger(deps: Deps, env: Env) -> StdResult<bool> {
    let config = load_config(deps.storage)?;
    let price_last = load_price_cumulative_last(deps.storage)?;

    let current_time = env.block.time.seconds();
    let time_elapsed = current_time - price_last.last_price_update_timestamp;

    // can be triggered if one full period has passed since the last update
    if time_elapsed < config.price_update_interval {
        Ok(false)
    } else {
        Ok(true)
    }
}
