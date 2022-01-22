#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    to_binary, Addr, Api, Binary, CanonicalAddr, Deps, DepsMut, Env, MessageInfo, Response,
    StdResult, Storage,
};

use crate::{
    commands,
    error::ContractError,
    queries,
    state::{load_config, store_config, Config},
};

use services::rewards::{ExecuteMsg, InstantiateMsg, QueryMsg};

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    let whitelist = msg
        .whitelist
        .into_iter()
        .map(|w| deps.api.addr_canonicalize(&w))
        .collect::<StdResult<Vec<CanonicalAddr>>>()?;

    store_config(
        deps.storage,
        &Config {
            gov_contract: deps.api.addr_canonicalize(&msg.gov_contract)?,
            bro_token: deps.api.addr_canonicalize(&msg.bro_token)?,
            spend_limit: msg.spend_limit,
            whitelist,
        },
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
            new_gov_contract,
            bro_token,
            spend_limit,
        } => {
            assert_owner(deps.storage, deps.api, info.sender)?;
            commands::update_config(deps, new_gov_contract, bro_token, spend_limit)
        }
        ExecuteMsg::AddDistributor { distributor } => {
            assert_owner(deps.storage, deps.api, info.sender)?;
            commands::add_distributor(deps, distributor)
        }
        ExecuteMsg::RemoveDistributor { distributor } => {
            assert_owner(deps.storage, deps.api, info.sender)?;
            commands::remove_distributor(deps, distributor)
        }
        ExecuteMsg::DistributeRewards { distributions } => {
            commands::distribute_reward(deps, info, distributions)
        }
    }
}

fn assert_owner(storage: &dyn Storage, api: &dyn Api, sender: Addr) -> Result<(), ContractError> {
    if load_config(storage)?.gov_contract != api.addr_canonicalize(sender.as_str())? {
        return Err(ContractError::Unauthorized {});
    }

    Ok(())
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Config {} => to_binary(&queries::query_config(deps)?),
    }
}
