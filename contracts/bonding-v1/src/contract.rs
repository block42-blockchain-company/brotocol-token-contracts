use std::str::FromStr;

#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    from_binary, to_binary, Addr, Api, Binary, CosmosMsg, Decimal, Deps, DepsMut, Env, MessageInfo,
    Response, StdResult, Storage, Uint128, WasmMsg,
};
use cw2::set_contract_version;
use cw20::Cw20ReceiveMsg;

use crate::{
    commands,
    error::ContractError,
    queries,
    state::{load_config, store_config, store_state, Config, State},
};

use services::{
    bonding::{Cw20HookMsg, ExecuteMsg, InstantiateMsg, MigrateMsg, QueryMsg},
    oracle::ExecuteMsg as OracleExecuteMsg,
};

/// Contract name that is used for migration.
const CONTRACT_NAME: &str = "brotocol-bonding-v1";
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

    if msg.ust_bonding_reward_ratio > Decimal::from_str("1.0")?
        || msg.ust_bonding_reward_ratio <= Decimal::zero()
    {
        return Err(ContractError::InvalidUstBondRatio {});
    }

    store_config(
        deps.storage,
        &Config {
            owner: deps.api.addr_canonicalize(&msg.owner)?,
            bro_token: deps.api.addr_canonicalize(&msg.bro_token)?,
            lp_token: deps.api.addr_canonicalize(&msg.lp_token)?,
            treasury_contract: deps.api.addr_canonicalize(&msg.treasury_contract)?,
            astroport_factory: deps.api.addr_canonicalize(&msg.astroport_factory)?,
            oracle_contract: deps.api.addr_canonicalize(&msg.oracle_contract)?,
            ust_bonding_reward_ratio: msg.ust_bonding_reward_ratio,
            ust_bonding_discount: msg.ust_bonding_discount,
            lp_bonding_discount: msg.lp_bonding_discount,
            min_bro_payout: msg.min_bro_payout,
            vesting_period_blocks: msg.vesting_period_blocks,
        },
    )?;

    store_state(
        deps.storage,
        &State {
            ust_bonding_balance: Uint128::zero(),
            lp_bonding_balance: Uint128::zero(),
        },
    )?;

    Ok(
        Response::new().add_messages(vec![CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: msg.oracle_contract,
            funds: vec![],
            msg: to_binary(&OracleExecuteMsg::UpdatePrice {})?,
        })]),
    )
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
/// * **ExecuteMsg::Receive(msg)** Receives a message of type [`Cw20ReceiveMsg`]
/// and processes it depending on the received template
///
/// * **ExecuteMsg::UstBond {}** Bond bro tokens by providing ust amount
///
/// * **ExecuteMsg::Claim {}** Claim availalble reward amount
///
/// * **ExecuteMsg::UpdateConfig {
///         owner,
///         lp_token,
///         treasury_contract,
///         astroport_factory,
///         oracle_contract,
///         ust_bonding_reward_ratio,
///         ust_bonding_discount,
///         lp_bonding_discount,
///         min_bro_payout,
///         vesting_period_blocks,
///     }** Updates contract settings
#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::Receive(msg) => receive_cw20(deps, env, info, msg),
        ExecuteMsg::UstBond {} => commands::ust_bond(deps, env, info),
        ExecuteMsg::Claim {} => commands::claim(deps, env, info),
        ExecuteMsg::UpdateConfig {
            owner,
            lp_token,
            treasury_contract,
            astroport_factory,
            oracle_contract,
            ust_bonding_reward_ratio,
            ust_bonding_discount,
            lp_bonding_discount,
            min_bro_payout,
            vesting_period_blocks,
        } => {
            assert_owner(deps.storage, deps.api, info.sender)?;
            commands::update_config(
                deps,
                owner,
                lp_token,
                treasury_contract,
                astroport_factory,
                oracle_contract,
                ust_bonding_reward_ratio,
                ust_bonding_discount,
                lp_bonding_discount,
                min_bro_payout,
                vesting_period_blocks,
            )
        }
    }
}

/// ## Description
/// Receives a message of type [`Cw20ReceiveMsg`] and processes it depending on the received template.
/// If the template is not found in the received message, then an [`ContractError`] is returned,
/// otherwise returns the [`Response`] with the specified attributes if the operation was successful
/// ## Params
/// * **deps** is the object of type [`DepsMut`].
///
/// * **env** is the object of type [`Env`].
///
/// * **info** is the object of type [`MessageInfo`].
///
/// * **cw20_msg** is the object of type [`Cw20ReceiveMsg`].
pub fn receive_cw20(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    cw20_msg: Cw20ReceiveMsg,
) -> Result<Response, ContractError> {
    let config = load_config(deps.storage)?;

    match from_binary(&cw20_msg.msg) {
        Ok(Cw20HookMsg::DistributeReward {}) => {
            if info.sender != deps.api.addr_humanize(&config.bro_token)? {
                return Err(ContractError::Unauthorized {});
            }

            commands::distribute_reward(deps, cw20_msg.amount)
        }
        Ok(Cw20HookMsg::LpBond {}) => {
            if info.sender != deps.api.addr_humanize(&config.lp_token)? {
                return Err(ContractError::Unauthorized {});
            }

            let sender_raw = deps.api.addr_canonicalize(&cw20_msg.sender)?;
            commands::lp_bond(deps, env, sender_raw, cw20_msg.amount)
        }
        Err(_) => Err(ContractError::InvalidHookData {}),
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
/// * **_env** is the object of type [`Env`].
///
/// * **msg** is the object of type [`ExecuteMsg`].
///
/// ## Queries
///
/// * **QueryMsg::Config {}** Returns bonding contract config
///
/// * **QueryMsg::State {}** Returns bonding contract state
///
/// * **QueryMsg::Claims { address }** Returns available claims for bonder by specified address
#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Config {} => to_binary(&queries::query_config(deps)?),
        QueryMsg::State {} => to_binary(&queries::query_state(deps)?),
        QueryMsg::Claims { address } => to_binary(&queries::query_claims(deps, address)?),
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
