use cosmwasm_std::{to_binary, Binary, CosmosMsg, StdResult, Uint128, WasmMsg};
use cw20::Cw20ExecuteMsg;

use crate::address::Address;

pub fn transfer_msg(
    token_addr: &Address,
    recipient: &Address,
    amount: Uint128,
) -> StdResult<CosmosMsg> {
    Ok(CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: token_addr.to_string()?,
        funds: vec![],
        msg: to_binary(&Cw20ExecuteMsg::Transfer {
            recipient: recipient.to_string()?,
            amount,
        })?,
    }))
}

pub fn send_msg(
    token_addr: &Address,
    contract: &Address,
    amount: Uint128,
    msg: Binary,
) -> StdResult<CosmosMsg> {
    Ok(CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: token_addr.to_string()?,
        funds: vec![],
        msg: to_binary(&Cw20ExecuteMsg::Send {
            contract: contract.to_string()?,
            amount,
            msg,
        })?,
    }))
}

pub fn mint_msg(
    token_addr: &Address,
    recipient: &Address,
    amount: Uint128,
) -> StdResult<CosmosMsg> {
    Ok(CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: token_addr.to_string()?,
        funds: vec![],
        msg: to_binary(&Cw20ExecuteMsg::Mint {
            recipient: recipient.to_string()?,
            amount,
        })?,
    }))
}
