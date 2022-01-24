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
            bro_token: deps.api.addr_canonicalize(&msg.bro_token)?,
            spend_limit: msg.spend_limit,
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
/// * **ExecuteMsg::UpdateConfig {
///         new_gov_contract,
///         spend_limit,
///     }** Updates contract settings
///
/// * **ExecuteMsg::AddDistributor { distributor }** Adds new distributor address into whitelist
///
/// * **ExecuteMsg::RemoveDistributor { distributor }** Removes distributor from whitelist
///
/// * **ExecuteMsg::DistributeRewards { distributions }** Distributes rewards to specified contracts
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
            spend_limit,
        } => {
            assert_owner(deps.storage, deps.api, info.sender)?;
            commands::update_config(deps, new_gov_contract, spend_limit)
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
/// * **QueryMsg::Config {}** Returns rewards pool contract config
///
/// * **QueryMsg::Balance {}** Returns rewards pool token balance
#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Config {} => to_binary(&queries::query_config(deps)?),
        QueryMsg::Balance {} => to_binary(&queries::query_balance(deps, env)?),
    }
}
