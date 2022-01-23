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

/// ## Description
/// Creates a new contract with the specified parameters in the [`InstantiateMsg`].
/// Returns the default [`Response`] object if the operation was successful, otherwise returns
/// the [`ContractError`] if the contract was not created.
/// ## Params
/// * **deps** is an object of type [`DepsMut`].
///
/// * **_env** is an object of type [`Env`].
///
/// * **_info** is an object of type [`MessageInfo`].
///
/// * **msg** is a message of type [`InstantiateMsg`] which contains the basic settings for creating a contract
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
            bbro_token: None,
            whitelist,
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
/// * **ExecuteMsg::InstantiateToken {
///         code_id,
///         token_instantiate_msg,
///     }** Creates new token contract
///
/// * **ExecuteMsg::UpdateConfig {
///         new_gov_contract,
///         bbro_token,
///     }** Updates contract settings
///
/// * **ExecuteMsg::AddMinter { minter }** Adds new minter address into whitelist
///
/// * **ExecuteMsg::RemoveMinter { minter }** Removes minter from whitelist
///
/// * **ExecuteMsg::Mint { recipient, amount }** Mints specified amount for specified address
///
/// * **ExecuteMsg::Burn { owner, amount }** Burns specified amount from specified address balance
#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::InstantiateToken {
            code_id,
            token_instantiate_msg,
        } => {
            assert_owner(deps.storage, deps.api, info.sender)?;
            commands::instantiate_token(env, code_id, token_instantiate_msg)
        }
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

/// ## Description
/// Verifies that message sender is a contract owner.
/// Returns [`Ok`] if address is valid, otherwise returns [`ContractError`]
/// ## Params
/// * **storage** is an object of type [`Storage`]
///
/// * **api** is an object of type [`Api`]
///
/// * **sender** is an object of type [`Addr`]
fn assert_owner(storage: &dyn Storage, api: &dyn Api, sender: Addr) -> Result<(), ContractError> {
    if load_config(storage)?.gov_contract != api.addr_canonicalize(sender.as_str())? {
        return Err(ContractError::Unauthorized {});
    }

    Ok(())
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
/// * **QueryMsg::Config {}** Returns bbro-minter contract config
#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Config {} => to_binary(&queries::query_config(deps)?),
    }
}
