use cosmwasm_std::{
    to_binary, CanonicalAddr, CosmosMsg, DepsMut, Env, MessageInfo, Response, Uint128, WasmMsg,
};
use cw20::{Cw20ExecuteMsg, MinterResponse};
use cw20_base::msg::InstantiateMsg as TokenInstantiateMsg;

use crate::{
    error::ContractError,
    state::{load_config, store_config},
};

/// ## Description
/// Creates new token contract.
/// Returns [`Response`] with specified attributes and messages if operation was successful,
/// otherwise returns [`ContractError`]
/// ## Params
/// * **env** is an object of type [`Env`]
///
/// * **code_id** is a field of type [`u64`]
///
/// * **token_instantiate_msg** is an object of type [`TokenInstantiateMsg`]
pub fn instantiate_token(
    env: Env,
    code_id: u64,
    token_instantiate_msg: TokenInstantiateMsg,
) -> Result<Response, ContractError> {
    let mut token_instantiate_msg = token_instantiate_msg;

    if !token_instantiate_msg.initial_balances.is_empty() {
        return Err(ContractError::InitialBalancesMustBeEmpty {});
    }

    if token_instantiate_msg.mint.is_some() {
        return Err(ContractError::InitialMinterInfoMustBeEmpty {});
    }

    token_instantiate_msg.mint = Some(MinterResponse {
        minter: env.contract.address.to_string(),
        cap: None,
    });

    Ok(Response::new()
        .add_messages(vec![CosmosMsg::Wasm(WasmMsg::Instantiate {
            admin: Some(env.contract.address.to_string()),
            code_id,
            msg: to_binary(&token_instantiate_msg)?,
            funds: vec![],
            label: "".to_string(),
        })])
        .add_attributes(vec![
            ("action", "instantiate_token"),
            ("code_id", &code_id.to_string()),
        ]))
}

/// ## Description
/// Updates contract settings.
/// Returns [`Response`] with specified attributes and messages if operation was successful,
/// otherwise returns [`ContractError`]
/// ## Params
/// * **deps** is an object of type [`DepsMut`]
///
/// * **new_gov_contract** is an [`Option`] field of type [`String`]. Sets new contract owner address
///
/// * **bbro_token** is an [`Option`] field of type [`String`]. Sets new bbro token address
pub fn update_config(
    deps: DepsMut,
    new_gov_contract: Option<String>,
    bbro_token: Option<String>,
) -> Result<Response, ContractError> {
    let mut config = load_config(deps.storage)?;

    if let Some(new_gov_contract) = new_gov_contract {
        config.gov_contract = deps.api.addr_canonicalize(&new_gov_contract)?;
    }

    if let Some(bbro_token) = bbro_token {
        config.bbro_token = Some(deps.api.addr_canonicalize(&bbro_token)?);
    }

    store_config(deps.storage, &config)?;

    Ok(Response::new().add_attributes(vec![("action", "update_config")]))
}

/// ## Description
/// Adds new minter address into whitelist.
/// Returns [`Response`] with specified attributes and messages if operation was successful,
/// otherwise returns [`ContractError`]
/// ## Params
/// * **deps** is an object of type [`DepsMut`]
///
/// * **minter** is a field of type [`String`]
pub fn add_minter(deps: DepsMut, minter: String) -> Result<Response, ContractError> {
    let mut config = load_config(deps.storage)?;

    let minter_raw = deps.api.addr_canonicalize(&minter)?;
    if config
        .whitelist
        .clone()
        .into_iter()
        .any(|w| w == minter_raw)
    {
        return Err(ContractError::MinterAlreadyRegistered {});
    }

    config.whitelist.push(minter_raw);
    store_config(deps.storage, &config)?;

    Ok(Response::new().add_attributes(vec![("action", "add_minter"), ("minter", minter.as_str())]))
}

/// ## Description
/// Removes minter from whitelist.
/// Returns [`Response`] with specified attributes and messages if operation was successful,
/// otherwise returns [`ContractError`]
/// ## Params
/// * **deps** is an object of type [`DepsMut`]
///
/// * **minter** is a field of type [`String`]
pub fn remove_minter(deps: DepsMut, minter: String) -> Result<Response, ContractError> {
    let mut config = load_config(deps.storage)?;

    let minter_raw = deps.api.addr_canonicalize(&minter)?;
    let whitelist_len = config.whitelist.len();
    let whitelist: Vec<CanonicalAddr> = config
        .whitelist
        .into_iter()
        .filter(|w| *w != minter_raw)
        .collect();

    if whitelist_len == whitelist.len() {
        return Err(ContractError::MinterNotFound {});
    }

    config.whitelist = whitelist;
    store_config(deps.storage, &config)?;

    Ok(Response::new().add_attributes(vec![("action", "remove_minter"), ("minter", &minter)]))
}

/// ## Description
/// Mints specified amount for specified address.
/// Returns [`Response`] with specified attributes and messages if operation was successful,
/// otherwise returns [`ContractError`]
/// ## Params
/// * **deps** is an object of type [`DepsMut`]
///
/// * **info** is an object of type [`MessageInfo`]
///
/// * **recipient** is a field of type [`String`]
///
/// * **amount** is a field of type [`Uint128`]
pub fn mint(
    deps: DepsMut,
    info: MessageInfo,
    recipient: String,
    amount: Uint128,
) -> Result<Response, ContractError> {
    let config = load_config(deps.storage)?;
    let bbro_token = if let Some(bbro_token) = config.bbro_token {
        deps.api.addr_humanize(&bbro_token)?.to_string()
    } else {
        return Err(ContractError::BbroContractAddressIsNotSet {});
    };

    let minter_raw = deps.api.addr_canonicalize(&info.sender.to_string())?;
    if !config.whitelist.into_iter().any(|w| w == minter_raw) {
        return Err(ContractError::Unauthorized {});
    }

    Ok(Response::new()
        .add_messages(vec![CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: bbro_token,
            funds: vec![],
            msg: to_binary(&Cw20ExecuteMsg::Mint {
                recipient: recipient.clone(),
                amount,
            })?,
        })])
        .add_attributes(vec![
            ("action", "mint"),
            ("minter", &info.sender.to_string()),
            ("recipient", &recipient),
            ("amount", &amount.to_string()),
        ]))
}

/// ## Description
/// Burns specified amount from specified address balance.
/// Returns [`Response`] with specified attributes and messages if operation was successful,
/// otherwise returns [`ContractError`]
/// ## Params
/// * **deps** is an object of type [`DepsMut`]
///
/// * **info** is an object of type [`MessageInfo`]
///
/// * **owner** is a field of type [`String`]
///
/// * **amount** is a field of type [`Uint128`]
pub fn burn(
    deps: DepsMut,
    info: MessageInfo,
    owner: String,
    amount: Uint128,
) -> Result<Response, ContractError> {
    let config = load_config(deps.storage)?;
    let bbro_token = if let Some(bbro_token) = config.bbro_token {
        deps.api.addr_humanize(&bbro_token)?.to_string()
    } else {
        return Err(ContractError::BbroContractAddressIsNotSet {});
    };

    let minter_raw = deps.api.addr_canonicalize(&info.sender.to_string())?;
    if !config.whitelist.into_iter().any(|w| w == minter_raw) {
        return Err(ContractError::Unauthorized {});
    }

    Ok(Response::new()
        .add_messages(vec![CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: bbro_token,
            funds: vec![],
            msg: to_binary(&Cw20ExecuteMsg::BurnFrom {
                owner: owner.clone(),
                amount,
            })?,
        })])
        .add_attributes(vec![
            ("action", "burn"),
            ("minter", &info.sender.to_string()),
            ("recipient", &owner),
            ("amount", &amount.to_string()),
        ]))
}
