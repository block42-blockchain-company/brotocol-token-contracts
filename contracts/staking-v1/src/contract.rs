#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    from_binary, to_binary, Binary, Decimal, Deps, DepsMut, Env, MessageInfo, Response, StdResult,
    Uint128,
};
use cw2::set_contract_version;
use cw20::Cw20ReceiveMsg;

use crate::{
    commands,
    error::ContractError,
    queries,
    state::{load_config, store_config, store_state, Config, LockupConfig, State},
};

use services::staking::{Cw20HookMsg, ExecuteMsg, InstantiateMsg, MigrateMsg, QueryMsg};

/// Contract name that is used for migration.
const CONTRACT_NAME: &str = "brotocol-staking-v1";
/// Contract version that is used for migration.
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

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
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    store_config(
        deps.storage,
        &Config {
            bro_token: deps.api.addr_canonicalize(&msg.bro_token)?,
            rewards_pool_contract: deps.api.addr_canonicalize(&msg.rewards_pool_contract)?,
            bbro_minter_contract: deps.api.addr_canonicalize(&msg.bbro_minter_contract)?,
            epoch_manager_contract: deps.api.addr_canonicalize(&msg.epoch_manager_contract)?,
            unstake_period_blocks: msg.unstake_period_blocks,
            min_staking_amount: msg.min_staking_amount,
            lockup_config: LockupConfig {
                min_lockup_period_epochs: msg.min_lockup_period_epochs,
                max_lockup_period_epochs: msg.max_lockup_period_epochs,
                base_rate: msg.base_rate,
                linear_growth: msg.linear_growth,
                exponential_growth: msg.exponential_growth,
            },
        },
    )?;

    store_state(
        deps.storage,
        &State {
            global_reward_index: Decimal::zero(),
            total_stake_amount: Uint128::zero(),
            last_distribution_block: env.block.height,
        },
    )?;

    Ok(Response::default())
}

/// ## Description
/// Available execute messages of the contract
/// ## Params
/// * **deps** is an object of type [`Deps`].
///
/// * **env** is an object of type [`Env`].
///
/// * **info** is an object of type [`MessageInfo`].
///
/// * **msg** is an object of type [`ExecuteMsg`].
///
/// ## Messages
///
/// * **ExecuteMsg::Receive(msg)** Receives a message of type [`Cw20ReceiveMsg`]
/// and processes it depending on the received template
///
/// * **ExecuteMsg::LockupStaked {
///         amount,
///         epochs_locked,
///     }** Lockup unlocked staked amount
///
/// * **ExecuteMsg::Unstake { amount }** Unstake staked amount of tokens
///
/// * **ExecuteMsg::Withdraw {}** Withdraw the amount of tokens that have already passed the unstaking period
///
/// * **ExecuteMsg::ClaimStakingRewards {}** Claim availalble bro reward amount
///
/// * **ExecuteMsg::ClaimBbroRewards {}** Claim availalble bbro reward amount
#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::Receive(msg) => receive_cw20(deps, env, info, msg),
        ExecuteMsg::LockupStaked {
            amount,
            epochs_locked,
        } => commands::lockup_staked(deps, env, info, amount, epochs_locked),
        ExecuteMsg::Unstake { amount } => commands::unstake(deps, env, info, amount),
        ExecuteMsg::Withdraw {} => commands::withdraw(deps, env, info),
        ExecuteMsg::ClaimStakingRewards {} => commands::claim_staking_rewards(deps, env, info),
        ExecuteMsg::ClaimBbroRewards {} => commands::claim_bbro_rewards(deps, env, info),
    }
}

/// ## Description
/// Receives a message of type [`Cw20ReceiveMsg`] and processes it depending on the received template.
/// If the template is not found in the received message, then an [`ContractError`] is returned,
/// otherwise returns the [`Response`] with the specified attributes if the operation was successful
/// ## Params
/// * **deps** is an object of type [`DepsMut`].
///
/// * **env** is an object of type [`Env`].
///
/// * **info** is an object of type [`MessageInfo`].
///
/// * **cw20_msg** is an object of type [`Cw20ReceiveMsg`].
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
        Ok(Cw20HookMsg::Stake { stake_type }) => {
            let cw20_sender = deps.api.addr_validate(&cw20_msg.sender)?;
            commands::stake(deps, env, cw20_sender, cw20_msg.amount, stake_type)
        }
        Err(_) => Err(ContractError::InvalidHookData {}),
    }
}

/// ## Description
/// Available query messages of the contract
/// ## Params
/// * **deps** is an object of type [`Deps`].
///
/// * **env** is an object of type [`Env`].
///
/// * **msg** is an object of type [`ExecuteMsg`].
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
/// * **_deps** is an object of type [`Deps`].
///
/// * **_env** is an object of type [`Env`].
///
/// * **_msg** is an object of type [`MigrateMsg`].
#[cfg_attr(not(feature = "library"), entry_point)]
pub fn migrate(_deps: DepsMut, _env: Env, _msg: MigrateMsg) -> StdResult<Response> {
    Ok(Response::default())
}
