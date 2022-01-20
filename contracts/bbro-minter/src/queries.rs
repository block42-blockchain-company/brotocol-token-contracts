use cosmwasm_std::{Deps, StdResult};
use services::bbro_minter::ConfigResponse;

use crate::state::load_config;

pub fn query_config(deps: Deps) -> StdResult<ConfigResponse> {
    let config = load_config(deps.storage)?;
    let resp = ConfigResponse {
        gov_contract: deps.api.addr_humanize(&config.gov_contract)?.to_string(),
        bbro_token: deps.api.addr_humanize(&config.bbro_token)?.to_string(),
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
