use cosmwasm_std::{Deps, Env, StdResult};

use services::staking::{
    ConfigResponse, StakerAccruedRewardsResponse, StakerInfoResponse, StateResponse,
    WithdrawalInfoResponse, WithdrawalsResponse,
};

use crate::state::{load_config, load_state, load_withdrawals, read_staker_info};

/// ## Description
/// Returns staking contract config in the [`ConfigResponse`] object
/// ## Params
/// * **deps** is an object of type [`Deps`]
pub fn query_config(deps: Deps) -> StdResult<ConfigResponse> {
    let config = load_config(deps.storage)?;
    let resp = ConfigResponse {
        bro_token: deps.api.addr_humanize(&config.bro_token)?.to_string(),
        rewards_pool_contract: deps
            .api
            .addr_humanize(&config.rewards_pool_contract)?
            .to_string(),
        bbro_minter_contract: deps
            .api
            .addr_humanize(&config.bbro_minter_contract)?
            .to_string(),
        epoch_manager_contract: deps
            .api
            .addr_humanize(&config.epoch_manager_contract)?
            .to_string(),
        unbond_period_blocks: config.unbond_period_blocks,
    };

    Ok(resp)
}

/// ## Description
/// Returns staking contract state in the [`StateResponse`] object
/// ## Params
/// * **deps** is an object of type [`Deps`]
pub fn query_state(deps: Deps) -> StdResult<StateResponse> {
    let state = load_state(deps.storage)?;
    let resp = StateResponse {
        global_reward_index: state.global_reward_index,
        total_bond_amount: state.total_bond_amount,
        last_distribution_block: state.last_distribution_block,
    };

    Ok(resp)
}

/// ## Description
/// Returns staker info by specified address in the [`StakerInfoResponse`] object
/// ## Params
/// * **deps** is an object of type [`Deps`]
///
/// * **env** is an object of type [`Env`]
///
/// * **staker** is a field of type [`String`]
pub fn query_staker_info(deps: Deps, env: Env, staker: String) -> StdResult<StakerInfoResponse> {
    let staker_raw = deps.api.addr_canonicalize(&staker)?;
    let state = load_state(deps.storage)?;
    let mut staker_info = read_staker_info(deps.storage, &staker_raw, env.block.height)?;

    staker_info.compute_staker_reward(&state)?;
    let resp = StakerInfoResponse {
        staker,
        reward_index: staker_info.reward_index,
        bond_amount: staker_info.bond_amount,
        pending_reward: staker_info.pending_reward,
        last_balance_update: staker_info.last_balance_update,
    };

    Ok(resp)
}

/// ## Description
/// Returns available amount for staker to claim by specified address in the [`StakerAccruedRewardsResponse`] object
/// ## Params
/// * **deps** is an object of type [`Deps`]
///
/// * **env** is an object of type [`Env`]
///
/// * **staker** is a field of type [`String`]
pub fn query_staker_accrued_rewards(
    deps: Deps,
    env: Env,
    staker: String,
) -> StdResult<StakerAccruedRewardsResponse> {
    let staker_addr_raw = deps.api.addr_canonicalize(&staker)?;

    let config = load_config(deps.storage)?;
    let state = load_state(deps.storage)?;
    let mut staker_info = read_staker_info(deps.storage, &staker_addr_raw, env.block.height)?;

    let bbro_staking_reward = staker_info.compute_staker_bbro_reward(
        &deps.querier,
        deps.api.addr_humanize(&config.epoch_manager_contract)?,
        &state,
    )?;

    staker_info.compute_staker_reward(&state)?;
    let resp = StakerAccruedRewardsResponse {
        rewards: staker_info.pending_reward,
        bbro_staking_reward,
    };

    Ok(resp)
}

/// ## Description
/// Returns available withdrawals for staker by specified address in the [`WithdrawalsResponse`] object
/// ## Params
/// * **deps** is an object of type [`Deps`]
///
/// * **staker** is a field of type [`String`]
pub fn query_withdrawals(deps: Deps, staker: String) -> StdResult<WithdrawalsResponse> {
    let staker_addr_raw = deps.api.addr_canonicalize(&staker)?;
    let claims: Vec<WithdrawalInfoResponse> = load_withdrawals(deps.storage, &staker_addr_raw)?
        .into_iter()
        .map(|c| WithdrawalInfoResponse {
            amount: c.amount,
            claimable_at: c.claimable_at,
        })
        .collect();

    let resp = WithdrawalsResponse { claims };
    Ok(resp)
}
