#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{to_binary, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult};

use crate::{
    error::ContractError, state::{store_config, Config, store_state, State},
    commands,
    queries,
};

use services::distributor::{
    ExecuteMsg, InstantiateMsg, QueryMsg,
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
            epoch_manager_contract: deps.api.addr_canonicalize(&msg.epoch_manager_contract)?,
            rewards_contract: deps.api.addr_canonicalize(&msg.rewards_contract)?,
            staking_contract: deps.api.addr_canonicalize(&msg.staking_contract)?,
            staking_distribution_amount: msg.staking_distribution_amount,
            bonding_contract: deps.api.addr_canonicalize(&msg.bonding_contract)?,
            bonding_distribution_amount: msg.bonding_distribution_amount,
        },
    )?;

    store_state(
        deps.storage, 
        &State {
            last_distribution_block: env.block.height,
        },
    )?;

    Ok(Response::default())
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    _info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::Distribute {} => commands::distribute(deps, env),
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Config {} => to_binary(&queries::query_config(deps)?),
        QueryMsg::LastDistribution {} => to_binary(&queries::query_last_distribution_block(deps)?),
    }
}
