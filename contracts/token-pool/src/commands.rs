use cosmwasm_std::{to_binary, Attribute, Binary, CosmosMsg, DepsMut, Response, Uint128, WasmMsg};
use cw20::Cw20ExecuteMsg;

use crate::{
    error::ContractError,
    state::{load_config, store_config},
};

/// ## Description
/// Transfer specified amount to specified address.
/// Returns [`Response`] with specified attributes and messages if operation was successful,
/// otherwise returns [`ContractError`]
/// ## Params
/// * **deps** is an object of type [`DepsMut`]
///
/// * **recipient** is an object of type [`String`]
///
/// * **amount** is an object of type [`Uint128`]
pub fn transfer(
    deps: DepsMut,
    recipient: String,
    amount: Uint128,
) -> Result<Response, ContractError> {
    let config = load_config(deps.storage)?;

    let bro_token = deps.api.addr_humanize(&config.bro_token)?.to_string();
    Ok(Response::new()
        .add_messages(vec![CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: bro_token,
            funds: vec![],
            msg: to_binary(&Cw20ExecuteMsg::Transfer {
                recipient: recipient.clone(),
                amount,
            })?,
        })])
        .add_attributes(vec![
            ("action", "transfer"),
            ("recipient", &recipient),
            ("amount", &amount.to_string()),
        ]))
}

/// ## Description
/// Transfer specified amount to specified contract with provided execute msg.
/// Returns [`Response`] with specified attributes and messages if operation was successful,
/// otherwise returns [`ContractError`]
/// ## Params
/// * **deps** is an object of type [`DepsMut`]
///
/// * **contract** is an object of type [`String`]
///
/// * **amount** is an object of type [`Uint128`]
///
/// * **msg** is an object of type [`Binary`]
pub fn send(
    deps: DepsMut,
    contract: String,
    amount: Uint128,
    msg: Binary,
) -> Result<Response, ContractError> {
    let config = load_config(deps.storage)?;

    let bro_token = deps.api.addr_humanize(&config.bro_token)?.to_string();
    Ok(Response::new()
        .add_messages(vec![CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: bro_token,
            funds: vec![],
            msg: to_binary(&Cw20ExecuteMsg::Send {
                contract: contract.clone(),
                amount,
                msg,
            })?,
        })])
        .add_attributes(vec![
            ("action", "send"),
            ("contract", &contract),
            ("amount", &amount.to_string()),
        ]))
}

/// ## Description
/// Updates contract settings.
/// Returns [`Response`] with specified attributes and messages if operation was successful,
/// otherwise returns [`ContractError`]
/// ## Params
/// * **deps** is an object of type [`DepsMut`]
///
/// * **owner** is an [`Option`] of type [`String`]
pub fn update_config(deps: DepsMut, owner: Option<String>) -> Result<Response, ContractError> {
    let mut config = load_config(deps.storage)?;

    let mut attributes: Vec<Attribute> = vec![Attribute::new("action", "update_config")];

    if let Some(owner) = owner {
        config.owner = deps.api.addr_canonicalize(&owner)?;
        attributes.push(Attribute::new("owner_changed", &owner));
    }

    store_config(deps.storage, &config)?;

    Ok(Response::new().add_attributes(attributes))
}
