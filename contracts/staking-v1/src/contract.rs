#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{to_binary, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult, from_binary, Decimal, Uint128};
use cw20::Cw20ReceiveMsg;

use crate::{
    error::ContractError,
    state::{load_config, store_config, Config, store_state, State},
    commands,
    queries,
};

use services::staking::{
    Cw20HookMsg, ExecuteMsg, InstantiateMsg, QueryMsg,
};

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    store_config(
        deps.storage,
        &Config {
            bro_token: deps.api.addr_canonicalize(&msg.bro_token)?,
            rewards_pool_contract: deps.api.addr_canonicalize(&msg.rewards_pool_contract)?,
            bbro_minter_contract: deps.api.addr_canonicalize(&msg.bbro_minter_contract)?,
            epoch_manager_contract: deps.api.addr_canonicalize(&msg.epoch_manager_contract)?,
            unbond_period_blocks: msg.unbond_period_blocks,
        },
    )?;

    store_state(
        deps.storage,
        &State {
            global_reward_index: Decimal::zero(),
            total_bond_amount: Uint128::zero(),
            last_distribution_block: env.block.height,
        },
    )?;

    Ok(Response::default())
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::Receive(msg) => receive_cw20(deps, env, info, msg),
        ExecuteMsg::Unbond {
            amount,
        } => commands::unbond(deps, env, info, amount),
        ExecuteMsg::Withdraw {} => commands::withdraw(deps, env, info),
        ExecuteMsg::ClaimRewards {} => commands::claim_rewards(deps, env, info),
    }
}

pub fn receive_cw20(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    cw20_msg: Cw20ReceiveMsg,
) -> Result<Response, ContractError> {
    let config = load_config(deps.storage)?;
    if info.sender != deps.api.addr_humanize(&config.bro_token)? {
        return Err(ContractError::Unauthorized {});
    }

    match from_binary(&cw20_msg.msg) {
        Ok(Cw20HookMsg::DistributeReward {
            distributed_at_block,
        }) => {
            // only rewards pool allowed to send bro token rewards to staking contract
            if config.rewards_pool_contract != deps.api.addr_canonicalize(&cw20_msg.sender)? {
                return Err(ContractError::Unauthorized {});
            }

            commands::distribute_reward(deps, cw20_msg.amount, distributed_at_block)
        },
        Ok(Cw20HookMsg::Bond {}) => {
            let cw20_sender = deps.api.addr_validate(&cw20_msg.sender)?;
            commands::bond(deps, env, cw20_sender, cw20_msg.amount)
        },
        Err(_) => Err(ContractError::InvalidHookData {}),
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Config {} => to_binary(&queries::query_config(deps)?),
        QueryMsg::State {} => to_binary(&queries::query_state(deps)?),
        QueryMsg::StakerInfo {
            staker,
        } => to_binary(&queries::query_staker_info(deps, env, staker)?),
        QueryMsg::StakerAccruedRewards {
            staker,
        } => to_binary(&queries::query_staker_accrued_rewards(deps, env, staker)?),
        QueryMsg::Withdrawals {
            staker,
        } => to_binary(&queries::query_withdrawals(deps, staker)?),
    }
}
