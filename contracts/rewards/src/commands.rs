use cosmwasm_std::{
    to_binary, CanonicalAddr, CosmosMsg, DepsMut, MessageInfo, Response, Uint128, WasmMsg,
};
use cw20::Cw20ExecuteMsg;
use services::rewards::DistributeRewardMsg;

use crate::{
    error::ContractError,
    state::{load_config, store_config},
};

/// ## Description
/// Updates contract settings.
/// Returns [`Response`] with specified attributes and messages if operation was successful,
/// otherwise returns [`ContractError`]
/// ## Params
/// * **deps** is an object of type [`DepsMut`]
///
/// * **owner** is an [`Option`] field of type [`String`]. Sets new contract owner address
///
/// * **spend_limit** is an [`Option`] field of type [`Uint128`]. Sets new spend limit
pub fn update_config(
    deps: DepsMut,
    owner: Option<String>,
    spend_limit: Option<Uint128>,
) -> Result<Response, ContractError> {
    let mut config = load_config(deps.storage)?;

    if let Some(owner) = owner {
        config.owner = deps.api.addr_canonicalize(&owner)?;
    }

    if let Some(spend_limit) = spend_limit {
        config.spend_limit = spend_limit;
    }

    store_config(deps.storage, &config)?;

    Ok(Response::new().add_attribute("action", "update_config"))
}

/// ## Description
/// Adds new distributor address into whitelist.
/// Returns [`Response`] with specified attributes and messages if operation was successful,
/// otherwise returns [`ContractError`]
/// ## Params
/// * **deps** is an object of type [`DepsMut`]
///
/// * **distributor** is a field of type [`String`]
pub fn add_distributor(deps: DepsMut, distributor: String) -> Result<Response, ContractError> {
    let mut config = load_config(deps.storage)?;

    let distributor_raw = deps.api.addr_canonicalize(&distributor)?;
    if config
        .whitelist
        .clone()
        .into_iter()
        .any(|w| w == distributor_raw)
    {
        return Err(ContractError::DistributorAlreadyRegistered {});
    }

    config.whitelist.push(distributor_raw);
    store_config(deps.storage, &config)?;

    Ok(Response::new().add_attributes(vec![
        ("action", "add_distributor"),
        ("distributor", distributor.as_str()),
    ]))
}

/// ## Description
/// Removes distributor from whitelist.
/// Returns [`Response`] with specified attributes and messages if operation was successful,
/// otherwise returns [`ContractError`]
/// ## Params
/// * **deps** is an object of type [`DepsMut`]
///
/// * **distributor** is a field of type [`String`]
pub fn remove_distributor(deps: DepsMut, distributor: String) -> Result<Response, ContractError> {
    let mut config = load_config(deps.storage)?;

    let distributor_raw = deps.api.addr_canonicalize(&distributor)?;
    let whitelist_len = config.whitelist.len();
    let whitelist: Vec<CanonicalAddr> = config
        .whitelist
        .into_iter()
        .filter(|w| *w != distributor_raw)
        .collect();

    if whitelist_len == whitelist.len() {
        return Err(ContractError::DistributorNotFound {});
    }

    config.whitelist = whitelist;
    store_config(deps.storage, &config)?;

    Ok(Response::new().add_attributes(vec![
        ("action", "remove_distributor"),
        ("distributor", distributor.as_str()),
    ]))
}

/// ## Description
/// Distributes rewards to specified contracts.
/// Returns [`Response`] with specified attributes and messages if operation was successful,
/// otherwise returns [`ContractError`]
/// ## Params
/// * **deps** is an object of type [`DepsMut`]
///
/// * **info** is an object of type [`MessageInfo`]
///
/// * **distributions** is a [`Vec`] of type [`DistributeRewardMsg`]
pub fn distribute_reward(
    deps: DepsMut,
    info: MessageInfo,
    distributions: Vec<DistributeRewardMsg>,
) -> Result<Response, ContractError> {
    let config = load_config(deps.storage)?;
    let bro_token = deps.api.addr_humanize(&config.bro_token)?.to_string();

    let distributor_raw = deps.api.addr_canonicalize(&info.sender.to_string())?;
    if !config.whitelist.into_iter().any(|w| w == distributor_raw) {
        return Err(ContractError::Unauthorized {});
    }

    let mut msgs: Vec<CosmosMsg> = vec![];
    for distribution in distributions {
        if config.spend_limit < distribution.amount {
            return Err(ContractError::SpendLimitReached {});
        }

        msgs.push(CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: bro_token.clone(),
            funds: vec![],
            msg: to_binary(&Cw20ExecuteMsg::Send {
                contract: distribution.contract,
                amount: distribution.amount,
                msg: distribution.msg,
            })?,
        }));
    }

    Ok(Response::new()
        .add_messages(msgs)
        .add_attributes(vec![("action", "spend")]))
}
