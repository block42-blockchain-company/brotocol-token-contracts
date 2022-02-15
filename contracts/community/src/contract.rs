#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    to_binary, Addr, Api, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult, Storage,
};
use cw2::set_contract_version;

use crate::{
    commands,
    error::ContractError,
    queries,
    state::{load_config, store_config, Config},
};

use services::community::{ExecuteMsg, InstantiateMsg, MigrateMsg, QueryMsg};

/// Contract name that is used for migration.
const CONTRACT_NAME: &str = "brotocol-community";
/// Contract version that is used for migration.
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

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
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    store_config(
        deps.storage,
        &Config {
            owner: deps.api.addr_canonicalize(&msg.owner)?,
            bro_token: deps.api.addr_canonicalize(&msg.bro_token)?,
        },
    )?;

    Ok(Response::default())
}

/// ## Description
/// Available execute messages of the contract
/// ## Params
/// * **deps** is an object of type [`Deps`].
///
/// * **_env** is an object of type [`Env`].
///
/// * **info** is an object of type [`MessageInfo`].
///
/// * **msg** is an object of type [`ExecuteMsg`].
///
/// ## Messages
///
/// * **ExecuteMsg::Transfer { recipient, amount }** Transfer specified amount to specified address
///
/// * **ExecuteMsg::Send {
///         contract,
///         amount,
///         msg,
///     }** Transfer specified amount to specified contract with provided execute msg
#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::Transfer { recipient, amount } => {
            assert_owner(deps.storage, deps.api, info.sender)?;
            commands::transfer(deps, recipient, amount)
        }
        ExecuteMsg::Send {
            contract,
            amount,
            msg,
        } => {
            assert_owner(deps.storage, deps.api, info.sender)?;
            commands::send(deps, contract, amount, msg)
        }
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
    if load_config(storage)?.owner != api.addr_canonicalize(sender.as_str())? {
        return Err(ContractError::Unauthorized {});
    }

    Ok(())
}

/// ## Description
/// Available query messages of the contract
/// ## Params
/// * **deps** is an object of type [`Deps`].
///
/// * **_env** is an object of type [`Env`].
///
/// * **msg** is an object of type [`ExecuteMsg`].
///
/// ## Queries
///
/// * **QueryMsg::Config {}** Returns community pool contract config
#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Config {} => to_binary(&queries::query_config(deps)?),
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
