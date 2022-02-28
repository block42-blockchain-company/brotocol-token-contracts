use cosmwasm_std::{Deps, Env, StdResult};

use services::staking::{
    ConfigResponse, LockupConfigResponse, LockupInfoResponse, StakerAccruedRewardsResponse,
    StakerInfoResponse, StateResponse, WithdrawalInfoResponse, WithdrawalsResponse,
};

use crate::state::{load_config, load_state, load_withdrawals, read_staker_info};

/// ## Description
/// Returns staking contract config in the [`ConfigResponse`] object
/// ## Params
/// * **deps** is an object of type [`Deps`]
pub fn query_config(deps: Deps) -> StdResult<ConfigResponse> {
    let config = load_config(deps.storage)?;
    let resp = ConfigResponse {
        owner: deps.api.addr_humanize(&config.owner)?.to_string(),
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
        unstake_period_blocks: config.unstake_period_blocks,
        min_staking_amount: config.min_staking_amount,
        lockup_config: LockupConfigResponse {
            min_lockup_period_epochs: config.lockup_config.min_lockup_period_epochs,
            max_lockup_period_epochs: config.lockup_config.max_lockup_period_epochs,
            base_rate: config.lockup_config.base_rate,
            linear_growth: config.lockup_config.linear_growth,
            exponential_growth: config.lockup_config.exponential_growth,
        },
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
        total_stake_amount: state.total_stake_amount,
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

    staker_info.compute_staking_reward(&state)?;
    staker_info.unlock_expired_lockups(&env.block)?;
    let resp = StakerInfoResponse {
        staker,
        reward_index: staker_info.reward_index,
        unlocked_stake_amount: staker_info.unlocked_stake_amount,
        locked_stake_amount: staker_info.locked_stake_amount,
        pending_bro_reward: staker_info.pending_bro_reward,
        last_balance_update: staker_info.last_balance_update,
        lockups: staker_info
            .lockups
            .into_iter()
            .map(|l| LockupInfoResponse {
                amount: l.amount,
                unlocked_at: l.unlocked_at,
            })
            .collect(),
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

    staker_info.compute_normal_bbro_reward(
        &deps.querier,
        deps.api.addr_humanize(&config.epoch_manager_contract)?,
        &state,
        env.block.height,
    )?;

    staker_info.compute_staking_reward(&state)?;
    let resp = StakerAccruedRewardsResponse {
        pending_bro_reward: staker_info.pending_bro_reward,
        pending_bbro_reward: staker_info.pending_bbro_reward,
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
