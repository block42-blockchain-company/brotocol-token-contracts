#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    to_binary, Addr, Api, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult, Storage,
};
use cw2::set_contract_version;

use cosmwasm_bignumber::Decimal256;

use crate::{
    commands,
    error::ContractError,
    queries,
    state::{
        load_config, store_config, store_price_cumulative_last, update_owner, Config,
        PriceCumulativeLast,
    },
};

use services::{
    oracle::{ExecuteMsg, InstantiateMsg, MigrateMsg, QueryMsg},
    ownership_proposal::{
        claim_ownership, drop_ownership_proposal, propose_new_owner, query_ownership_proposal,
    },
    querier::query_cumulative_prices,
};

use astroport::querier::query_pair_info;

/// Contract name that is used for migration.
const CONTRACT_NAME: &str = "brotocol-oracle";
/// Contract version that is used for migration.
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

/// ## Description
/// Creates a new contract with the specified parameters in the [`InstantiateMsg`].
/// Returns the default [`Response`] object if the operation was successful, otherwise returns
/// the [`ContractError`] if the contract was not created.
/// ## Params
/// * **deps** is an object of type [`DepsMut`].
///
/// * **env** is an object of type [`Env`].
///
/// * **_info** is an object of type [`MessageInfo`].
///
/// * **msg** is a message of type [`InstantiateMsg`] which contains the basic settings for creating a contract
#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    msg.asset_infos[0].check(deps.api)?;
    msg.asset_infos[1].check(deps.api)?;

    let pair_info = query_pair_info(
        &deps.querier,
        deps.api.addr_validate(&msg.factory_contract)?,
        &msg.asset_infos,
    )?;

    store_config(
        deps.storage,
        &Config {
            owner: deps.api.addr_canonicalize(&msg.owner)?,
            factory: deps.api.addr_canonicalize(&msg.factory_contract)?,
            asset_infos: msg.asset_infos.clone(),
            pair: pair_info.clone(),
            price_update_interval: msg.price_update_interval,
            price_validity_period: msg.price_validity_period,
        },
    )?;

    let prices = query_cumulative_prices(&deps.querier, pair_info.contract_addr)?;
    store_price_cumulative_last(
        deps.storage,
        &PriceCumulativeLast {
            price_0_cumulative_last: prices.price0_cumulative_last,
            price_1_cumulative_last: prices.price1_cumulative_last,
            price_0_average: Decimal256::zero(),
            price_1_average: Decimal256::zero(),
            last_price_update_timestamp: env.block.time.seconds(),
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
///         price_update_interval,
///         price_validity_period,
///     }** Updates contract settings
///
/// * **ExecuteMsg::UpdatePrice {}** Updates cumulative prices
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
        ExecuteMsg::UpdateConfig {
            price_update_interval,
            price_validity_period,
        } => {
            assert_owner(deps.storage, deps.api, info.sender)?;
            commands::update_config(deps, price_update_interval, price_validity_period)
        }
        ExecuteMsg::UpdatePrice {} => commands::update_price(deps, env),
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
/// * **QueryMsg::Config {}** Returns oracle contract config
///
/// * **QueryMsg::ConsultPrice { asset, amount }** Returns calculated average amount with updated precision
///
/// * **QueryMsg::IsReadyToTrigger {}** Returns a [`bool`] type whether prices are ready to be updated or not
///
/// * **QueryMsg::OwnershipProposal {}** Returns information about created ownership proposal otherwise returns not-found error
#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Config {} => to_binary(&queries::query_config(deps)?),
        QueryMsg::ConsultPrice { asset, amount } => {
            to_binary(&queries::consult_price(deps, env, asset, amount)?)
        }
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
