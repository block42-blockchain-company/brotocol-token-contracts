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
    state::{load_config, store_config, store_latest_stage, Config},
};

use services::airdrop::{ExecuteMsg, InstantiateMsg, MigrateMsg, QueryMsg};

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
/// * **ExecuteMsg::UpdateConfig { owner }** Updates contract settings
///
/// * **ExecuteMsg::RegisterMerkleRoot { merkle_root }** Registers merkle root hash
///
/// * **ExecuteMsg::Claim {
///         stage,
///         amount,
///         proof,
///     }** Claims availalble amount for message sender at specified airdrop round
#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::UpdateConfig { owner } => {
            assert_owner(deps.storage, deps.api, info.sender)?;
            commands::update_config(deps, owner)
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
/// * **QueryMsg::Config {}** Returns airdrop contract config
///
/// * **QueryMsg::LatestStage {}** Returns the number of latest stage
///
/// * **QueryMsg::MerkleRoot { stage }** Returns merkle root information by specified stage
///
/// * **QueryMsg::IsClaimed { stage, address }** Returns claim information by specified stage and address
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
