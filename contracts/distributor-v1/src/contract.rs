#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    to_binary, Addr, Api, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult, Storage,
};

use crate::{
    commands,
    error::ContractError,
    queries,
    state::{load_config, store_config, store_state, Config, State},
};

use services::distributor::{ExecuteMsg, InstantiateMsg, MigrateMsg, QueryMsg};

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
    store_config(
        deps.storage,
        &Config {
            owner: deps.api.addr_canonicalize(&msg.owner)?,
            distribution_genesis_block: msg.distribution_genesis_block,
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
            last_distribution_block: msg.distribution_genesis_block,
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
/// * **ExecuteMsg::Distribute {}** Performs token distribution
///
/// * **ExecuteMsg::UpdateConfig {
///         owner,
///         epoch_manager_contract,
///         rewards_contract,
///         staking_contract,
///         staking_distribution_amount,
///         bonding_contract,
///         bonding_distribution_amount,
///     }** Updates contract settings
#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::Distribute {} => commands::distribute(deps, env),
        ExecuteMsg::UpdateConfig {
            owner,
            epoch_manager_contract,
            rewards_contract,
            staking_contract,
            staking_distribution_amount,
            bonding_contract,
            bonding_distribution_amount,
        } => {
            assert_owner(deps.storage, deps.api, info.sender)?;
            commands::update_config(
                deps,
                owner,
                epoch_manager_contract,
                rewards_contract,
                staking_contract,
                staking_distribution_amount,
                bonding_contract,
                bonding_distribution_amount,
            )
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
/// * **deps** is the object of type [`Deps`].
///
/// * **env** is the object of type [`Env`].
///
/// * **msg** is the object of type [`ExecuteMsg`].
///
/// ## Queries
///
/// * **QueryMsg::Config {}** Returns distributor contract config
///
/// * **QueryMsg::LastDistribution {}** Returns information about last distribution
#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Config {} => to_binary(&queries::query_config(deps)?),
        QueryMsg::LastDistribution {} => to_binary(&queries::query_last_distribution_block(deps)?),
    }
}

/// ## Description
/// Used for migration of contract. Returns the default object of type [`Response`].
/// ## Params
/// * **_deps** is the object of type [`Deps`].
///
/// * **_env** is the object of type [`Env`].
///
/// * **_msg** is the object of type [`MigrateMsg`].
#[cfg_attr(not(feature = "library"), entry_point)]
pub fn migrate(_deps: DepsMut, _env: Env, _msg: MigrateMsg) -> StdResult<Response> {
    Ok(Response::default())
}
