#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    from_binary, to_binary, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult,
};
use cw2::set_contract_version;
use cw20::Cw20ReceiveMsg;

use crate::{
    commands,
    error::ContractError,
    queries,
    state::{load_config, store_config, store_latest_stage, update_owner, Config},
};

use services::{
    airdrop::{Cw20HookMsg, ExecuteMsg, InstantiateMsg, MigrateMsg, QueryMsg},
    ownership_proposal::{
        claim_ownership, drop_ownership_proposal, propose_new_owner, query_ownership_proposal,
    },
};

/// Contract name that is used for migration.
const CONTRACT_NAME: &str = "brotocol-airdrop";
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

    let stage: u8 = 0;
    store_latest_stage(deps.storage, stage)?;

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
/// * **ExecuteMsg::RegisterMerkleRoot { merkle_root }** Registers merkle root hash
///
/// * **ExecuteMsg::Claim {
///         stage,
///         amount,
///         proof,
///     }** Claims available amount for message sender at specified airdrop round
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
        ExecuteMsg::Receive(msg) => receive_cw20(deps, info, msg),
        ExecuteMsg::Claim {
            stage,
            amount,
            proof,
        } => commands::claim(deps, info, stage, amount, proof),
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
/// Receives a message of type [`Cw20ReceiveMsg`] and processes it depending on the received template.
/// If the template is not found in the received message, then an [`ContractError`] is returned,
/// otherwise returns the [`Response`] with the specified attributes if the operation was successful
/// ## Params
/// * **deps** is an object of type [`DepsMut`].
///
/// * **env** is an object of type [`Env`].
///
/// * **info** is an object of type [`MessageInfo`].
///
/// * **cw20_msg** is an object of type [`Cw20ReceiveMsg`].
pub fn receive_cw20(
    deps: DepsMut,
    info: MessageInfo,
    cw20_msg: Cw20ReceiveMsg,
) -> Result<Response, ContractError> {
    let config = load_config(deps.storage)?;

    if info.sender != deps.api.addr_humanize(&config.bro_token)? {
        return Err(ContractError::Unauthorized {});
    }

    match from_binary(&cw20_msg.msg) {
        Ok(Cw20HookMsg::RegisterMerkleRoot { merkle_root }) => {
            // only owner can register new merkle root
            if config.owner != deps.api.addr_canonicalize(&cw20_msg.sender)? {
                return Err(ContractError::Unauthorized {});
            }

            commands::register_merkle_root(deps, merkle_root)
        }
        Err(_) => Err(ContractError::InvalidHookData {}),
    }
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
/// * **QueryMsg::Config {}** Returns airdrop contract config
///
/// * **QueryMsg::LatestStage {}** Returns the number of latest stage
///
/// * **QueryMsg::MerkleRoot { stage }** Returns merkle root information by specified stage
///
/// * **QueryMsg::IsClaimed { stage, address }** Returns claim information by specified stage and address
///
/// * **QueryMsg::OwnershipProposal {}** Returns information about created ownership proposal otherwise returns not-found error
#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Config {} => to_binary(&queries::query_config(deps)?),
        QueryMsg::LatestStage {} => to_binary(&queries::query_latest_stage(deps)?),
        QueryMsg::MerkleRoot { stage } => to_binary(&queries::query_merkle_root(deps, stage)?),
        QueryMsg::IsClaimed { stage, address } => {
            to_binary(&queries::query_claimed(deps, stage, address)?)
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
