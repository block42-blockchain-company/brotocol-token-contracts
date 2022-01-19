#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{to_binary, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult, Storage, Api, Addr};

use crate::{
    error::ContractError, state::{store_config, Config, store_state, State, load_config},
    commands,
    queries,
};

use services::epoch_manager::{
    InstantiateMsg, ExecuteMsg, QueryMsg,
};

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    store_config(
        deps.storage,
        &Config {
            owner: deps.api.addr_canonicalize(&info.sender.to_string())?,
        },
    )?;

    store_state(
        deps.storage,
        &State {
            epoch: msg.epoch,
            blocks_per_year: msg.blocks_per_year,
            bbro_emission_rate: msg.bbro_emission_rate,
        }
    )?;

    Ok(Response::default())
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::UpdateConfig {
            owner,
        } => {
            assert_owner(deps.storage, deps.api, info.sender)?;
            commands::update_config(deps, owner)
        },
        ExecuteMsg::UpdateState {
            epoch,
            blocks_per_year,
            bbro_emission_rate,
        } => {
            assert_owner(deps.storage, deps.api, info.sender)?;
            commands::update_state(deps, epoch, blocks_per_year, bbro_emission_rate)
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
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Config {} => to_binary(&queries::query_config(deps)?),
        QueryMsg::EpochInfo {} => to_binary(&queries::query_epoch_info(deps)?),
    }
}

