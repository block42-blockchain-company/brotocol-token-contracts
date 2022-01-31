#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    from_binary, to_binary, Binary, Decimal, Deps, DepsMut, Env, MessageInfo, Response, StdResult,
    Uint128,
};
use cw20::Cw20ReceiveMsg;

use crate::{
    commands,
    error::ContractError,
    queries,
    state::{load_config, store_config, store_state, Config, State},
};

use services::staking::{Cw20HookMsg, ExecuteMsg, InstantiateMsg, MigrateMsg, QueryMsg};

/// ## Description
/// Creates a new contract with the specified parameters in the [`InstantiateMsg`].
/// Returns the default [`Response`] object if the operation was successful, otherwise returns
/// the [`ContractError`] if the contract was not created.
/// ## Params
/// * **deps** is an object of type [`DepsMut`].
///
/// * **env** is an object of type [`Env`].
///
/// * **_info** is an object of type [`MessageInfo`].
///
/// * **msg** is a message of type [`InstantiateMsg`] which contains the basic settings for creating a contract
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

/// ## Description
/// Available execute messages of the contract
/// ## Params
/// * **deps** is the object of type [`Deps`].
///
/// * **env** is the object of type [`Env`].
///
/// * **info** is the object of type [`MessageInfo`].
///
/// * **msg** is the object of type [`ExecuteMsg`].
///
/// ## Messages
///
/// * **ExecuteMsg::Receive(msg)** Receives a message of type [`Cw20ReceiveMsg`]
/// and processes it depending on the received template
///
/// * **ExecuteMsg::Unbond { amount }** Unbond staked amount of tokens
///
/// * **ExecuteMsg::Withdraw {}** Withdraw amount of tokens which have already passed unbonding period
///
/// * **ExecuteMsg::ClaimRewards {}** Claim availalble reward amount
#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::Receive(msg) => receive_cw20(deps, env, info, msg),
        ExecuteMsg::Unbond { amount } => commands::unbond(deps, env, info, amount),
        ExecuteMsg::Withdraw {} => commands::withdraw(deps, env, info),
        ExecuteMsg::ClaimRewards {} => commands::claim_rewards(deps, env, info),
    }
}

/// ## Description
/// Receives a message of type [`Cw20ReceiveMsg`] and processes it depending on the received template.
/// If the template is not found in the received message, then an [`ContractError`] is returned,
/// otherwise returns the [`Response`] with the specified attributes if the operation was successful
/// ## Params
/// * **deps** is the object of type [`DepsMut`].
///
/// * **env** is the object of type [`Env`].
///
/// * **info** is the object of type [`MessageInfo`].
///
/// * **cw20_msg** is the object of type [`Cw20ReceiveMsg`].
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
        }
        Ok(Cw20HookMsg::Bond {}) => {
            let cw20_sender = deps.api.addr_validate(&cw20_msg.sender)?;
            commands::bond(deps, env, cw20_sender, cw20_msg.amount)
        }
        Err(_) => Err(ContractError::InvalidHookData {}),
    }
}

/// ## Description
/// Available query messages of the contract
/// ## Params
/// * **deps** is the object of type [`Deps`].
///
/// * **env** is the object of type [`Env`].
///
/// * **msg** is the object of type [`ExecuteMsg`].
///
/// ## Queries
///
/// * **QueryMsg::Config {}** Returns staking contract config
///
/// * **QueryMsg::State {}** Returns staking contract state
///
/// * **QueryMsg::StakerInfo { staker }** Returns staker info by specified address
///
/// * **QueryMsg::StakerAccruedRewards { staker }** Returns available amount for staker to claim by specified address
///
/// * **QueryMsg::Withdrawals { staker }** Returns available withdrawals for staker by specified address
#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Config {} => to_binary(&queries::query_config(deps)?),
        QueryMsg::State {} => to_binary(&queries::query_state(deps)?),
        QueryMsg::StakerInfo { staker } => {
            to_binary(&queries::query_staker_info(deps, env, staker)?)
        }
        QueryMsg::StakerAccruedRewards { staker } => {
            to_binary(&queries::query_staker_accrued_rewards(deps, env, staker)?)
        }
        QueryMsg::Withdrawals { staker } => to_binary(&queries::query_withdrawals(deps, staker)?),
    }
}

/// ## Description
/// Used for migration of contract. Returns the default object of type [`Response`].
/// ## Params
/// * **_deps** is the object of type [`Deps`].
///
/// * **_env** is the object of type [`Env`].
///
/// * **_msg** is the object of type [`MigrateMsg`].
#[cfg_attr(not(feature = "library"), entry_point)]
pub fn migrate(_deps: DepsMut, _env: Env, _msg: MigrateMsg) -> StdResult<Response> {
    Ok(Response::default())
}
