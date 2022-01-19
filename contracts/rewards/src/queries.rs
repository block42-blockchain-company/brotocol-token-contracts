use cosmwasm_std::{Deps, StdResult};
use services::rewards::ConfigResponse;

use crate::state::load_config;

pub fn query_config(deps: Deps) -> StdResult<ConfigResponse> {
    let config = load_config(deps.storage)?;
    let resp = ConfigResponse {
        gov_contract: deps.api.addr_humanize(&config.gov_contract)?.to_string(),
        bro_token: deps.api.addr_humanize(&config.bro_token)?.to_string(),
        spend_limit: config.spend_limit,
        whitelist: config.whitelist.into_iter()
            .map(|w| match deps.api.addr_humanize(&w) {
                Ok(addr) => Ok(addr.to_string()),
                Err(e) => Err(e),
            })
            .collect::<StdResult<Vec<String>>>()?,
    };

    Ok(resp)
}