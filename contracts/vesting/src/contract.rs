#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    to_binary, Addr, Api, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult, Storage,
};

use crate::{
    commands,
    error::ContractError,
    queries,
    state::{load_config, store_config, Config},
};

use services::vesting::{ExecuteMsg, InstantiateMsg, QueryMsg};

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    store_config(
        deps.storage,
        &Config {
            owner: deps.api.addr_canonicalize(&msg.owner)?,
            bro_token: deps.api.addr_canonicalize(&msg.bro_token)?,
            genesis_time: msg.genesis_time,
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
        ExecuteMsg::Claim {} => commands::claim(deps, env, info),
        ExecuteMsg::UpdateConfig {
            owner,
            bro_token,
            genesis_time,
        } => {
            assert_owner(deps.storage, deps.api, info.sender)?;
            commands::update_config(deps, owner, bro_token, genesis_time)
        }
        ExecuteMsg::RegisterVestingAccounts { vesting_accounts } => {
            assert_owner(deps.storage, deps.api, info.sender)?;
            commands::register_vesting_accounts(deps, vesting_accounts)
        }
    }
}

fn assert_owner(storage: &dyn Storage, api: &dyn Api, sender: Addr) -> Result<(), ContractError> {
    if load_config(storage)?.owner != api.addr_canonicalize(sender.as_str())? {
        return Err(ContractError::Unauthorized {});
    }

    Ok(())
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Config {} => to_binary(&queries::query_config(deps)?),
        QueryMsg::VestingAccount { address } => {
            to_binary(&queries::query_vesting_account(deps, address)?)
        }
        QueryMsg::VestingAccounts {
            start_after,
            limit,
            order_by,
        } => to_binary(&queries::query_vesting_accounts(
            deps,
            start_after,
            limit,
            order_by,
        )?),
        QueryMsg::Claimable { address } => {
            to_binary(&queries::query_claimable_amount(deps, env, address)?)
        }
    }
}
