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
    state::{load_config, store_config, store_state, update_owner, Config, State},
};

use services::{
    distributor::{ExecuteMsg, InstantiateMsg, MigrateMsg, QueryMsg},
    ownership_proposal::{
        claim_ownership, drop_ownership_proposal, propose_new_owner, query_ownership_proposal,
    },
};

/// Contract name that is used for migration.
const CONTRACT_NAME: &str = "brotocol-distributor-v1";
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
            paused: false,
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
/// * **deps** is an object of type [`Deps`].
///
/// * **env** is an object of type [`Env`].
///
/// * **info** is an object of type [`MessageInfo`].
///
/// * **msg** is an object of type [`ExecuteMsg`].
///
/// ## Messages
///
/// * **ExecuteMsg::Distribute {}** Performs token distribution
///
/// * **ExecuteMsg::UpdateConfig {
///         paused,
///         epoch_manager_contract,
///         rewards_contract,
///         staking_contract,
///         staking_distribution_amount,
///         bonding_contract,
///         bonding_distribution_amount,
///     }** Updates contract settings
///
/// * **ExecuteMsg::ProposeNewOwner {
///         new_owner,
///         expires_in_blocks,
///     }** Creates an offer for a new owner
///
/// * **ExecuteMsg::DropOwnershipProposal {}** Removes the existing offer for the new owner
///
/// * **ExecuteMsg::ClaimOwnership {}** Used to claim(approve) new owner proposal, thus changing contract's owner
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
            paused,
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
                paused,
                epoch_manager_contract,
                rewards_contract,
                staking_contract,
                staking_distribution_amount,
                bonding_contract,
                bonding_distribution_amount,
            )
        }
        ExecuteMsg::ProposeNewOwner {
            new_owner,
            expires_in_blocks,
        } => {
            let config = load_config(deps.storage)?;

            propose_new_owner(deps, env, info, config.owner, new_owner, expires_in_blocks)
                .map_err(|e| e.into())
        }
        ExecuteMsg::DropOwnershipProposal {} => {
            let config = load_config(deps.storage)?;

            drop_ownership_proposal(deps, info, config.owner).map_err(|e| e.into())
        }
        ExecuteMsg::ClaimOwnership {} => {
            claim_ownership(deps, env, info, update_owner).map_err(|e| e.into())
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
/// * **env** is an object of type [`Env`].
///
/// * **msg** is an object of type [`ExecuteMsg`].
///
/// ## Queries
///
/// * **QueryMsg::Config {}** Returns distributor contract config
///
/// * **QueryMsg::LastDistribution {}** Returns information about last distribution
///
/// * **QueryMsg::IsReadyToTrigger {}** Returns whether funds can be distributed or not
///
/// * **QueryMsg::OwnershipProposal {}** Returns information about created ownership proposal otherwise returns not-found error
#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Config {} => to_binary(&queries::query_config(deps)?),
        QueryMsg::LastDistribution {} => to_binary(&queries::query_last_distribution_block(deps)?),
        QueryMsg::IsReadyToTrigger {} => to_binary(&queries::is_ready_to_trigger(deps, env)?),
        QueryMsg::OwnershipProposal {} => to_binary(&query_ownership_proposal(deps)?),
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
