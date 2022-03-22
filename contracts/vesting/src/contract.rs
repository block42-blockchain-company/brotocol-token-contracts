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
    state::{load_config, store_config, update_owner, Config},
};

use services::{
    ownership_proposal::{
        claim_ownership, drop_ownership_proposal, propose_new_owner, query_ownership_proposal,
    },
    vesting::{ExecuteMsg, InstantiateMsg, MigrateMsg, QueryMsg},
};

/// Contract name that is used for migration.
const CONTRACT_NAME: &str = "brotocol-vesting";
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
            genesis_time: msg.genesis_time,
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
///         genesis_time,
///     }** Updates contract settings
///
/// * **RegisterVestingAccounts { vesting_accounts }** Registers vesting accounts
/// for future distribution
///
/// * **ExecuteMsg::Claim {}** Claims available amount for message sender
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
        ExecuteMsg::UpdateConfig { genesis_time } => {
            assert_owner(deps.storage, deps.api, info.sender)?;
            commands::update_config(deps, genesis_time)
        }
        ExecuteMsg::RegisterVestingAccounts { vesting_accounts } => {
            assert_owner(deps.storage, deps.api, info.sender)?;
            commands::register_vesting_accounts(deps, vesting_accounts)
        }
        ExecuteMsg::Claim {} => commands::claim(deps, env, info),
        ExecuteMsg::ProposeNewOwner {
            new_owner,
            expires_in_blocks,
        } => {
            let config = load_config(deps.storage)?;

            Ok(propose_new_owner(
                deps,
                env,
                info,
                config.owner,
                new_owner,
                expires_in_blocks,
            )?)
        }
        ExecuteMsg::DropOwnershipProposal {} => {
            let config = load_config(deps.storage)?;

            Ok(drop_ownership_proposal(deps, info, config.owner)?)
        }
        ExecuteMsg::ClaimOwnership {} => Ok(claim_ownership(deps, env, info, update_owner)?),
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
/// * **QueryMsg::Config {}** Returns vesting contract config
///
/// * **QueryMsg::VestingAccount { address }** Returns vesting schedules for specified account
///
/// * **QueryMsg::VestingAccounts {
///         start_after,
///         limit,
///         order_by,
///     }** Returns a list of accounts for given input params
///
/// * **QueryMsg::Claimable { address }** Returns available amount to claim for specified account
///
/// * **QueryMsg::OwnershipProposal {}** Returns information about created ownership proposal otherwise returns not-found error
#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Config {} => to_binary(&queries::query_config(deps)?),
        QueryMsg::VestingAccount { address } => {
            to_binary(&queries::query_vesting_account(deps, address)?)
        }
        QueryMsg::VestingAccounts {
            start_after,
            limit,
            order_by,
        } => to_binary(&queries::query_vesting_accounts(
            deps,
            start_after,
            limit,
            order_by,
        )?),
        QueryMsg::Claimable { address } => {
            to_binary(&queries::query_claimable_amount(deps, env, address)?)
        }
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
