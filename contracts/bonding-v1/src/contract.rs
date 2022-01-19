use std::str::FromStr;

#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    to_binary, Binary, Deps, DepsMut, Env, MessageInfo, Response, 
    StdResult, Uint128, from_binary, Decimal,
};
use cw20::Cw20ReceiveMsg;

use crate::{
    error::ContractError, 
    state::{store_config, Config, store_state, State, load_config},
    commands,
    queries,
};

use services::bonding::{
    ExecuteMsg, InstantiateMsg, QueryMsg, Cw20HookMsg,
};

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    if msg.ust_bonding_reward_ratio > Decimal::from_str("1.0")? ||
        msg.ust_bonding_reward_ratio < Decimal::zero() {
        return Err(ContractError::InvalidUstBondRatio {})
    }

    store_config(
        deps.storage,
        &Config {
            owner: deps.api.addr_canonicalize(&info.sender.to_string())?,
            bro_token: deps.api.addr_canonicalize(&msg.bro_token)?,
            lp_token: deps.api.addr_canonicalize(&msg.lp_token)?,
            treasury_contract: deps.api.addr_canonicalize(&msg.treasury_contract)?,
            astroport_factory: deps.api.addr_canonicalize(&msg.astroport_factory)?,
            ust_bonding_reward_ratio: msg.ust_bonding_reward_ratio,
            ust_bonding_discount: msg.ust_bonding_discount,
            lp_bonding_discount: msg.lp_bonding_discount,
            vesting_period_blocks: msg.vesting_period_blocks,
        },
    )?;

    store_state(
        deps.storage,
        &State {
            ust_bonding_balance: Uint128::zero(),
            lp_bonding_balance: Uint128::zero(),
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
        ExecuteMsg::UstBond {} => commands::ust_bond(deps, env, info),
        ExecuteMsg::Claim {} => commands::claim(deps, env, info),
    }
}

pub fn receive_cw20(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    cw20_msg: Cw20ReceiveMsg,
) -> Result<Response, ContractError> {
    let config = load_config(deps.storage)?;

    match from_binary(&cw20_msg.msg) {
        Ok(Cw20HookMsg::DistributeReward {}) => {
            if info.sender != deps.api.addr_humanize(&config.bro_token)? {
                return Err(ContractError::Unauthorized {});
            }

            commands::distribute_reward(deps, cw20_msg.amount)
        },
        Ok(Cw20HookMsg::LpBond {}) => {
            if info.sender != deps.api.addr_humanize(&config.lp_token)? {
                return Err(ContractError::Unauthorized {});
            }

            let sender_raw = deps.api.addr_canonicalize(&cw20_msg.sender)?;
            commands::lp_bond(deps, env, sender_raw, cw20_msg.amount)
        },
        Err(_) => Err(ContractError::InvalidHookData {}),
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Config {} => to_binary(&queries::query_config(deps)?),
        QueryMsg::State {} => to_binary(&queries::query_state(deps)?),
        QueryMsg::Claims {
            address,
        } => to_binary(&queries::query_claims(deps, address)?),
    }
}
