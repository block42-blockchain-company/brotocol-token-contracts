use cosmwasm_std::{
    to_binary, CanonicalAddr, CosmosMsg, DepsMut, MessageInfo, Response, Uint128, WasmMsg,
};
use cw20::Cw20ExecuteMsg;

use crate::{
    error::ContractError,
    state::{load_config, store_config},
};

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
        config.bbro_token = deps.api.addr_canonicalize(&bbro_token)?;
    }

    store_config(deps.storage, &config)?;

    Ok(Response::new().add_attributes(vec![("action", "update_config")]))
}

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

pub fn mint(
    deps: DepsMut,
    info: MessageInfo,
    recipient: String,
    amount: Uint128,
) -> Result<Response, ContractError> {
    let config = load_config(deps.storage)?;

    let minter_raw = deps.api.addr_canonicalize(&info.sender.to_string())?;
    if !config.whitelist.into_iter().any(|w| w == minter_raw) {
        return Err(ContractError::Unauthorized {});
    }

    let bbro_token = deps.api.addr_humanize(&config.bbro_token)?.to_string();
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

pub fn burn(
    deps: DepsMut,
    info: MessageInfo,
    owner: String,
    amount: Uint128,
) -> Result<Response, ContractError> {
    let config = load_config(deps.storage)?;

    let minter_raw = deps.api.addr_canonicalize(&info.sender.to_string())?;
    if !config.whitelist.into_iter().any(|w| w == minter_raw) {
        return Err(ContractError::Unauthorized {});
    }

    let bbro_token = deps.api.addr_humanize(&config.bbro_token)?.to_string();
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
