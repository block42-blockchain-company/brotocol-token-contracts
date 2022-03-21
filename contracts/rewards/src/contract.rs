#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    to_binary, Addr, Api, Binary, CanonicalAddr, Deps, DepsMut, Env, MessageInfo, Response,
    StdResult, Storage,
};
use cw2::set_contract_version;

use crate::{
    commands,
    error::ContractError,
    queries,
    state::{load_config, store_config, update_owner, Config},
};

use services::{
    ownership_proposal::{
        claim_ownership, drop_ownership_proposal, propose_new_owner, query_ownership_proposal,
    },
    rewards::{ExecuteMsg, InstantiateMsg, MigrateMsg, QueryMsg},
};

/// Contract name that is used for migration.
const CONTRACT_NAME: &str = "brotocol-rewards-pool";
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

    let whitelist = msg
        .whitelist
        .into_iter()
        .map(|w| deps.api.addr_canonicalize(&w))
        .collect::<StdResult<Vec<CanonicalAddr>>>()?;

    store_config(
        deps.storage,
        &Config {
            owner: deps.api.addr_canonicalize(&msg.owner)?,
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
/// * **ExecuteMsg::UpdateConfig {
///         spend_limit,
///     }** Updates contract settings
///
/// * **ExecuteMsg::AddDistributor { distributor }** Adds new distributor address into whitelist
///
/// * **ExecuteMsg::RemoveDistributor { distributor }** Removes distributor from whitelist
///
/// * **ExecuteMsg::DistributeRewards { distributions }** Distributes rewards to specified contracts
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
        ExecuteMsg::UpdateConfig { spend_limit } => {
            assert_owner(deps.storage, deps.api, info.sender)?;
            commands::update_config(deps, spend_limit)
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
/// * **QueryMsg::Config {}** Returns rewards pool contract config
///
/// * **QueryMsg::Balance {}** Returns rewards pool token balance
///
/// * **QueryMsg::OwnershipProposal {}** Returns information about created ownership proposal otherwise returns not-found error
#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Config {} => to_binary(&queries::query_config(deps)?),
        QueryMsg::Balance {} => to_binary(&queries::query_balance(deps, env)?),
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
