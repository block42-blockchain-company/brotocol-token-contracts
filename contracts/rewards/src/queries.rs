use cosmwasm_std::{Deps, Env, StdResult};
use services::{
    querier::query_token_balance,
    rewards::{ConfigResponse, RewardsPoolBalanceResponse},
};

use crate::state::load_config;

/// ## Description
/// Returns rewards pool contract config in the [`ConfigResponse`] object
/// ## Params
/// * **deps** is an object of type [`Deps`]
pub fn query_config(deps: Deps) -> StdResult<ConfigResponse> {
    let config = load_config(deps.storage)?;
    let resp = ConfigResponse {
        owner: deps.api.addr_humanize(&config.owner)?.to_string(),
        bro_token: deps.api.addr_humanize(&config.bro_token)?.to_string(),
        spend_limit: config.spend_limit,
        whitelist: config
            .whitelist
            .into_iter()
            .map(|w| match deps.api.addr_humanize(&w) {
                Ok(addr) => Ok(addr.to_string()),
                Err(e) => Err(e),
            })
            .collect::<StdResult<Vec<String>>>()?,
    };

    Ok(resp)
}

/// ## Description
/// Returns rewards pool token balance in the [`RewardsPoolBalanceResponse`] object
/// ## Params
/// * **deps** is an object of type [`Deps`]
pub fn query_balance(deps: Deps, env: Env) -> StdResult<RewardsPoolBalanceResponse> {
    let config = load_config(deps.storage)?;
    let balance = query_token_balance(
        &deps.querier,
        deps.api.addr_humanize(&config.bro_token)?,
        env.contract.address,
    )?;
    let resp = RewardsPoolBalanceResponse { balance };

    Ok(resp)
}
