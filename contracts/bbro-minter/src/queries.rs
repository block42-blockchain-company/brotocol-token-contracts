use cosmwasm_std::{Deps, StdResult};
use services::bbro_minter::ConfigResponse;

use crate::state::load_config;

/// ## Description
/// Returns bbro minter contract config in the [`ConfigResponse`] object
/// ## Params
/// * **deps** is an object of type [`Deps`]
pub fn query_config(deps: Deps) -> StdResult<ConfigResponse> {
    let config = load_config(deps.storage)?;
    let bbro_token = if let Some(bbro_token) = config.bbro_token {
        deps.api.addr_humanize(&bbro_token)?.to_string()
    } else {
        "".to_string()
    };

    let resp = ConfigResponse {
        owner: deps.api.addr_humanize(&config.owner)?.to_string(),
        bbro_token,
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
