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

use services::bbro_minter::{ExecuteMsg, InstantiateMsg, QueryMsg};

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
            bbro_token: deps.api.addr_canonicalize(&msg.bbro_token)?,
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
            bbro_token,
        } => {
            assert_owner(deps.storage, deps.api, info.sender)?;
            commands::update_config(deps, new_gov_contract, bbro_token)
        }
        ExecuteMsg::AddMinter { minter } => {
            assert_owner(deps.storage, deps.api, info.sender)?;
            commands::add_minter(deps, minter)
        }
        ExecuteMsg::RemoveMinter { minter } => {
            assert_owner(deps.storage, deps.api, info.sender)?;
            commands::remove_minter(deps, minter)
        }
        ExecuteMsg::Mint { recipient, amount } => commands::mint(deps, info, recipient, amount),
        ExecuteMsg::Burn { owner, amount } => commands::burn(deps, info, owner, amount),
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
