use cosmwasm_std::{Deps, StdResult};
use services::distributor::{ConfigResponse, LastDistributionResponse};

use crate::state::{load_config, load_state};

pub fn query_config(deps: Deps) -> StdResult<ConfigResponse> {
    let config = load_config(deps.storage)?;
    let resp = ConfigResponse {
        epoch_manager_contract: deps
            .api
            .addr_humanize(&config.epoch_manager_contract)?
            .to_string(),
        rewards_contract: deps
            .api
            .addr_humanize(&config.rewards_contract)?
            .to_string(),
        staking_contract: deps
            .api
            .addr_humanize(&config.staking_contract)?
            .to_string(),
        staking_distribution_amount: config.staking_distribution_amount,
        bonding_contract: deps
            .api
            .addr_humanize(&config.bonding_contract)?
            .to_string(),
        bonding_distribution_amount: config.bonding_distribution_amount,
    };

    Ok(resp)
}

pub fn query_last_distribution_block(deps: Deps) -> StdResult<LastDistributionResponse> {
    let resp = LastDistributionResponse {
        last_distribution_block: load_state(deps.storage)?.last_distribution_block,
    };

    Ok(resp)
}
