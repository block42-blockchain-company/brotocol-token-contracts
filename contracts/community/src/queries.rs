use cosmwasm_std::{Deps, StdResult};

use crate::state::load_config;

use services::community::ConfigResponse;

/// ## Description
/// Returns bonding contract config in the [`ConfigResponse`] object
/// ## Params
/// * **deps** is an object of type [`Deps`]
pub fn query_config(deps: Deps) -> StdResult<ConfigResponse> {
    let config = load_config(deps.storage)?;
    let resp = ConfigResponse {
        owner: deps.api.addr_humanize(&config.owner)?.to_string(),
        bro_token: deps.api.addr_humanize(&config.bro_token)?.to_string(),
    };

    Ok(resp)
}
