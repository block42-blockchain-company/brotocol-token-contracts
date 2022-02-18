use cosmwasm_std::{
    from_binary, to_binary, Binary, Coin, CosmosMsg, DepsMut, Env, MessageInfo, Response, Uint128,
    WasmMsg,
};
use cw20::Cw20ExecuteMsg;

use astroport::asset::{Asset, AssetInfo};

use crate::{
    error::ContractError,
    state::{
        load_config, load_state, load_whitelisted_account, store_state, store_whitelisted_account,
        State,
    },
};

use services::whitelist_sale::WhitelistedAccountInfo;

/// ## Description
/// Registers sale and whitelists addresses.
/// Returns [`Response`] with specified attributes and messages if operation was successful,
/// otherwise returns [`ContractError`]
/// ## Params
/// * **deps** is an object of type [`DepsMut`]
///
/// * **env** is an object of type [`Env`]
///
/// * **sale_start_time** is a field of type [`u64`]
///
/// * **sale_end_time** is a field of type [`u64`]
///
/// * **accounts** is an object of type [`Binary`]
///
/// * **transfer_amount** is an object of type [`Uint128`]
pub fn register_sale(
    deps: DepsMut,
    env: Env,
    sale_start_time: u64,
    sale_end_time: u64,
    accounts: Binary,
    transfer_amount: Uint128,
) -> Result<Response, ContractError> {
    let config = load_config(deps.storage)?;

    if load_state(deps.storage)?.sale_registered {
        return Err(ContractError::SaleWasAlreadyRegistered {});
    }

    if sale_end_time <= sale_start_time || env.block.time.seconds() >= sale_start_time {
        return Err(ContractError::InvalidSalePeriod {});
    }

    let accounts = from_binary::<Vec<WhitelistedAccountInfo>>(&accounts)?;

    let mut required_transfer_amount = Uint128::zero();
    for account in accounts.iter() {
        let available_purchase_amount = config
            .bro_amount_per_nft
            .checked_mul(Uint128::from(account.owned_nfts_count))?;

        store_whitelisted_account(
            deps.storage,
            &deps.api.addr_canonicalize(&account.address)?,
            &available_purchase_amount,
        )?;

        required_transfer_amount += available_purchase_amount;
    }

    if required_transfer_amount > transfer_amount {
        return Err(ContractError::ReceivedAmountMustBeHigherThenRequiredAmountForSale {});
    }

    store_state(
        deps.storage,
        &State {
            sale_registered: true,
            sale_start_time,
            sale_end_time,
            balance: transfer_amount,
        },
    )?;

    Ok(Response::new().add_attributes(vec![
        ("action", "register_sale"),
        ("sale_start_time", &sale_start_time.to_string()),
        ("sale_end_time", &sale_end_time.to_string()),
        ("transfer_amount", &transfer_amount.to_string()),
    ]))
}

/// ## Description
/// Purchase bro by fixed price by providing ust amount.
/// Returns [`Response`] with specified attributes and messages if operation was successful,
/// otherwise returns [`ContractError`]
/// ## Params
/// * **deps** is an object of type [`DepsMut`]
///
/// * **env** is an object of type [`Env`]
///
/// * **info** is an object of type [`MessageInfo`]
pub fn purchase(deps: DepsMut, env: Env, info: MessageInfo) -> Result<Response, ContractError> {
    let config = load_config(deps.storage)?;
    let mut state = load_state(deps.storage)?;

    if !state.sale_is_live(env.block.time.seconds()) {
        return Err(ContractError::SaleIsNotLive {});
    }

    let received_ust = extract_native_token(&info.funds)?;
    let purchase_amount = received_ust
        .amount
        .checked_mul(config.bro_amount_per_uusd)?;

    let sender_raw = deps.api.addr_canonicalize(&info.sender.to_string())?;
    let available_purchase_amount = load_whitelisted_account(deps.storage, &sender_raw)
        .map_err(|_| ContractError::AddressIsNotWhitelisted {})?;

    if purchase_amount > available_purchase_amount {
        return Err(ContractError::PurchaseAmountIsTooHigh {});
    }

    let available_purchase_amount = available_purchase_amount.checked_sub(purchase_amount)?;
    store_whitelisted_account(deps.storage, &sender_raw, &available_purchase_amount)?;

    state.balance = state.balance.checked_sub(purchase_amount)?;
    store_state(deps.storage, &state)?;

    Ok(Response::new()
        .add_messages(vec![
            received_ust.into_msg(&deps.querier, deps.api.addr_humanize(&config.ust_receiver)?)?,
            CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: deps.api.addr_humanize(&config.bro_token)?.to_string(),
                funds: vec![],
                msg: to_binary(&Cw20ExecuteMsg::Transfer {
                    recipient: info.sender.to_string(),
                    amount: purchase_amount,
                })?,
            }),
        ])
        .add_attributes(vec![
            ("action", "purchase"),
            ("purchase_amount", &purchase_amount.to_string()),
        ]))
}

/// ## Description
/// Withdraw remaining bro balance after sale is over.
/// Returns [`Response`] with specified attributes and messages if operation was successful,
/// otherwise returns [`ContractError`]
/// ## Params
/// * **deps** is an object of type [`DepsMut`]
///
/// * **env** is an object of type [`Env`]
pub fn withdraw_remaining_balance(deps: DepsMut, env: Env) -> Result<Response, ContractError> {
    let config = load_config(deps.storage)?;
    let mut state = load_state(deps.storage)?;

    if !state.sale_finished(env.block.time.seconds()) {
        return Err(ContractError::SaleIsNotFinishedYet {});
    }

    let remaining_balance = state.balance;
    state.balance = Uint128::zero();
    store_state(deps.storage, &state)?;

    Ok(Response::new()
        .add_message(CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: deps.api.addr_humanize(&config.bro_token)?.to_string(),
            funds: vec![],
            msg: to_binary(&Cw20ExecuteMsg::Transfer {
                recipient: deps
                    .api
                    .addr_humanize(&config.rewards_pool_contract)?
                    .to_string(),
                amount: remaining_balance,
            })?,
        }))
        .add_attributes(vec![
            ("action", "withdraw_remaining_balance"),
            ("remaining_balance", &remaining_balance.to_string()),
        ]))
}

/// ## Description
/// Extracts ust amount from provided info.funds input.
/// Otherwise returns [`ContractError`]
/// ## Params
/// * **funds** is an object of type [`&[Coin]`]
fn extract_native_token(funds: &[Coin]) -> Result<Asset, ContractError> {
    if funds.len() != 1 || funds[0].denom != "uusd" || funds[0].amount.is_zero() {
        return Err(ContractError::InvalidFundsInput {});
    }

    Ok(Asset {
        info: AssetInfo::NativeToken {
            denom: "uusd".to_string(),
        },
        amount: funds[0].amount,
    })
}
