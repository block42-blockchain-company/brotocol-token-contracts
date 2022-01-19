use cosmwasm_std::{DepsMut, Uint128, Response, CosmosMsg, WasmMsg, to_binary, MessageInfo, CanonicalAddr, Binary,};
use cw20::Cw20ExecuteMsg;

use crate::{
    error::ContractError,
    state::{load_config, store_config},
};

pub fn update_config(
    deps: DepsMut,
    new_gov_contract: Option<String>,
    bro_token: Option<String>,
    spend_limit: Option<Uint128>,
) -> Result<Response, ContractError> {
    let mut config = load_config(deps.storage)?;

    if let Some(new_gov_contract) = new_gov_contract {
        config.gov_contract = deps.api.addr_canonicalize(&new_gov_contract)?;
    }

    if let Some(bro_token) = bro_token {
        config.bro_token = deps.api.addr_canonicalize(&bro_token)?;
    }
    
    if let Some(spend_limit) = spend_limit {
        config.spend_limit = spend_limit;
    }

    store_config(deps.storage, &config)?;

    Ok(Response::new().add_attribute("action", "update_config"))
}

pub fn add_distributor(deps: DepsMut, distributor: String) -> Result<Response, ContractError> {
    let mut config = load_config(deps.storage)?;

    let distributor_raw = deps.api.addr_canonicalize(&distributor)?;
    if config.whitelist.clone().into_iter().any(|w| w == distributor_raw) {
        return Err(ContractError::DistributorAlreadyRegistered {});
    }

    config.whitelist.push(distributor_raw);
    store_config(deps.storage, &config)?;

    Ok(Response::new()
        .add_attributes(vec![
            ("action", "add_distributor"),
            ("distributor", distributor.as_str()),
        ])
    )
}

pub fn remove_distributor(deps: DepsMut, distributor: String) -> Result<Response, ContractError> {
    let mut config = load_config(deps.storage)?;

    let distributor_raw = deps.api.addr_canonicalize(&distributor)?;
    let whitelist_len = config.whitelist.len();
    let whitelist: Vec<CanonicalAddr> = config.whitelist
        .into_iter()
        .filter(|w| *w != distributor_raw)
        .collect();

    if whitelist_len == whitelist.len() {
        return Err(ContractError::DistributorNotFound {});
    }

    config.whitelist = whitelist;
    store_config(deps.storage, &config)?;

    Ok(Response::new()
        .add_attributes(vec![
            ("action", "remove_distributor"),
            ("distributor", distributor.as_str()),
        ])
    )
}

pub fn reward(
    deps: DepsMut,
    info: MessageInfo,
    contract: String,
    amount: Uint128,
    msg: Binary,
) -> Result<Response, ContractError> {
    let config = load_config(deps.storage)?;

    let distributor_raw = deps.api.addr_canonicalize(&info.sender.to_string())?;
    if !config.whitelist.into_iter().any(|w| w == distributor_raw) {
        return Err(ContractError::Unauthorized {});
    }

    if config.spend_limit < amount {
        return Err(ContractError::SpendLimitReached {});
    }

    let bro_token = deps.api.addr_humanize(&config.bro_token)?.to_string();
    Ok(Response::new()
        .add_messages(vec![CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: bro_token,
            funds: vec![],
            msg: to_binary(&Cw20ExecuteMsg::Send {
                contract: contract.clone(),
                amount,
                msg,
            })?
        })])
        .add_attributes(vec![
            ("action", "spend"),
            ("receive_contract", contract.as_str()),
            ("amount", &amount.to_string()),
        ])
    )
}
