#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    to_binary, Addr, Api, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult, Storage,
};

use crate::{
    commands,
    error::ContractError,
    queries,
    state::{load_config, store_config, store_latest_stage, Config},
};

use services::airdrop::{ExecuteMsg, InstantiateMsg, QueryMsg};

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
        },
    )?;

    let stage: u8 = 0;
    store_latest_stage(deps.storage, stage)?;

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
        ExecuteMsg::UpdateConfig { owner, bro_token } => {
            assert_owner(deps.storage, deps.api, info.sender)?;
            commands::update_config(deps, owner, bro_token)
        }
        ExecuteMsg::RegisterMerkleRoot { merkle_root } => {
            assert_owner(deps.storage, deps.api, info.sender)?;
            commands::register_merkle_root(deps, merkle_root)
        }
        ExecuteMsg::Claim {
            stage,
            amount,
            proof,
        } => commands::claim(deps, info, stage, amount, proof),
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
        QueryMsg::LatestStage {} => to_binary(&queries::query_latest_stage(deps)?),
        QueryMsg::MerkleRoot { stage } => to_binary(&queries::query_merkle_root(deps, stage)?),
        QueryMsg::IsClaimed { stage, address } => {
            to_binary(&queries::query_claimed(deps, stage, address)?)
        }
    }
}
