use cosmwasm_std::{Deps, Env, StdError, StdResult};

use crate::state::{load_config, load_state, load_whitelisted_account};

use services::whitelist_sale::{ConfigResponse, StateResponse, WhitelistedAccountInfoResponse};

/// ## Description
/// Returns whitelist sale contract config in the [`ConfigResponse`] object
/// ## Params
/// * **deps** is an object of type [`Deps`]
pub fn query_config(deps: Deps) -> StdResult<ConfigResponse> {
    let config = load_config(deps.storage)?;
    let resp = ConfigResponse {
        owner: deps.api.addr_humanize(&config.owner)?.to_string(),
        bro_token: deps.api.addr_humanize(&config.bro_token)?.to_string(),
        bro_price_per_uusd: config.bro_price_per_uusd,
        bro_amount_per_nft: config.bro_amount_per_nft,
        treasury_contract: deps
            .api
            .addr_humanize(&config.treasury_contract)?
            .to_string(),
        rewards_pool_contract: deps
            .api
            .addr_humanize(&config.rewards_pool_contract)?
            .to_string(),
    };

    Ok(resp)
}

/// ## Description
/// Returns whitelist contract contract state in the [`StateResponse`] object
/// ## Params
/// * **deps** is an object of type [`Deps`]
///
/// * **env** is an object of type [`Env`]
pub fn query_state(deps: Deps, env: Env) -> StdResult<StateResponse> {
    let state = load_state(deps.storage)?;
    let resp = StateResponse {
        sale_registered: state.sale_registered,
        sale_start_time: state.sale_start_time,
        sale_end_time: state.sale_end_time,
        current_time: env.block.time.seconds(),
        balance: state.balance,
    };

    Ok(resp)
}

/// ## Description
/// Returns whitelisted account info in the [`WhitelistedAccountInfoResponse`] object
/// ## Params
/// * **deps** is an object of type [`Deps`]
///
/// * **address** is an object of type [`String`]
pub fn query_whitelisted_account(
    deps: Deps,
    address: String,
) -> StdResult<WhitelistedAccountInfoResponse> {
    let address_raw = deps.api.addr_canonicalize(&address)?;
    let available_purchase_amount = load_whitelisted_account(deps.storage, &address_raw)
        .map_err(|_| StdError::generic_err("address is not whitelisted"))?;

    Ok(WhitelistedAccountInfoResponse {
        address,
        available_purchase_amount,
    })
}
