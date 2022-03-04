use cosmwasm_std::{Deps, StdResult, Env};
use services::{
    distributor::{ConfigResponse, LastDistributionResponse},
    querier::{query_epoch_info},
};

use crate::state::{load_config, load_state};

/// ## Description
/// Returns distributor contract config in the [`ConfigResponse`] object
/// ## Params
/// * **deps** is an object of type [`Deps`]
pub fn query_config(deps: Deps) -> StdResult<ConfigResponse> {
    let config = load_config(deps.storage)?;
    let resp = ConfigResponse {
        owner: deps.api.addr_humanize(&config.owner)?.to_string(),
        paused: config.paused,
        distribution_genesis_block: config.distribution_genesis_block,
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

/// ## Description
/// Returns information about last distribution in the [`LastDistributionResponse`] object
/// ## Params
/// * **deps** is an object of type [`Deps`]
pub fn query_last_distribution_block(deps: Deps) -> StdResult<LastDistributionResponse> {
    let resp = LastDistributionResponse {
        last_distribution_block: load_state(deps.storage)?.last_distribution_block,
    };

    Ok(resp)
}

/// ## Description
/// Returns a [`bool`] type whether funds can be distributed or not
/// ## Params
/// * **deps** is an object of type [`Deps`]
pub fn is_ready_to_trigger(deps: Deps, env: Env) -> StdResult<bool> {
    let config = load_config(deps.storage)?;
    if config.paused {
        return Ok(false)
    }

    let state = load_state(deps.storage)?;
    
    if config.distribution_genesis_block > env.block.height {
        return Ok(false)
    }

    // query epoch from epoch_manager contract
    let epoch_blocks = query_epoch_info(
        &deps.querier,
        deps.api.addr_humanize(&config.epoch_manager_contract)?,
    )?
    .epoch;

    // only ready to be triggered if some epochs passed
    let blocks_since_last_distribution = env.block.height - state.last_distribution_block;
    let passed_epochs = blocks_since_last_distribution / epoch_blocks;
    if passed_epochs == 0 {
        Ok(false)
    } else {
        Ok(true)
    }
}
