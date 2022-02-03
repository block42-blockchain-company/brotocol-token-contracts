use cosmwasm_std::{
    to_binary, Addr, CanonicalAddr, Coin, CosmosMsg, Decimal, DepsMut, Env, MessageInfo,
    QuerierWrapper, Response, StdResult, Uint128, WasmMsg,
};
use cw20::{Cw20ExecuteMsg, Expiration};
use std::str::FromStr;

use astroport::{
    asset::{Asset, AssetInfo},
    querier::query_supply,
};

use crate::{
    error::ContractError,
    state::{
        load_claims, load_config, load_state, store_claims, store_config, store_state, BondType,
        ClaimInfo,
    },
};

use services::{
    oracle::ExecuteMsg as OracleExecuteMsg,
    querier::{query_oracle_price, query_pools},
};

/// ## Description
/// Distributes received reward.
/// Returns [`Response`] with specified attributes and messages if operation was successful,
/// otherwise returns [`ContractError`]
/// ## Params
/// * **deps** is an object of type [`DepsMut`]
///
/// * **amount** is an object of type [`Uint128`]
pub fn distribute_reward(deps: DepsMut, amount: Uint128) -> Result<Response, ContractError> {
    let config = load_config(deps.storage)?;
    let mut state = load_state(deps.storage)?;

    let ust_bond_amount = amount * config.ust_bonding_reward_ratio;
    let lp_bond_amount = amount.checked_sub(ust_bond_amount)?;

    state.ust_bonding_balance = state.ust_bonding_balance.checked_add(ust_bond_amount)?;
    state.lp_bonding_balance = state.lp_bonding_balance.checked_add(lp_bond_amount)?;

    store_state(deps.storage, &state)?;

    Ok(Response::new().add_attributes(vec![
        ("action", "distribute_reward"),
        ("ust_bond_amount", &ust_bond_amount.to_string()),
        ("lp_bond_amount", &lp_bond_amount.to_string()),
    ]))
}

/// ## Description
/// Bond bro tokens by providing lp token amount.
/// Returns [`Response`] with specified attributes and messages if operation was successful,
/// otherwise returns [`ContractError`]
/// ## Params
/// * **deps** is an object of type [`DepsMut`]
///
/// * **env** is an object of type [`Env`]
///
/// * **sender_raw** is an object of type [`CanonicalAddr`]
///
/// * **lp_amount** is an object of type [`Uint128`]
pub fn lp_bond(
    deps: DepsMut,
    env: Env,
    sender_raw: CanonicalAddr,
    lp_amount: Uint128,
) -> Result<Response, ContractError> {
    let config = load_config(deps.storage)?;
    let mut state = load_state(deps.storage)?;

    let (bro_pool, ust_pool) = query_bro_ust_pair(
        &deps.querier,
        deps.api.addr_humanize(&config.astroport_factory)?,
        deps.api.addr_humanize(&config.bro_token)?,
    )?;

    let (bro_share, ust_share) = get_share_in_assets(
        &deps.querier,
        &bro_pool,
        &ust_pool,
        lp_amount,
        deps.api.addr_humanize(&config.lp_token)?,
    )?;

    let oracle_contract = deps.api.addr_humanize(&config.oracle_contract)?;
    let bro_amount = bro_share.checked_add(
        query_oracle_price(
            &deps.querier,
            oracle_contract.clone(),
            AssetInfo::NativeToken {
                denom: "uusd".to_string(),
            },
            ust_share,
        )?
        .amount,
    )?;

    let bro_payout = apply_discount(config.lp_bonding_discount, bro_amount)?;
    if bro_payout < config.min_bro_payout {
        return Err(ContractError::BondPayoutIsLow {});
    }

    if bro_payout > state.lp_bonding_balance {
        return Err(ContractError::NotEnoughForBondPayout {});
    }

    state.lp_bonding_balance = state.lp_bonding_balance.checked_sub(bro_payout)?;
    store_state(deps.storage, &state)?;

    let mut claims = load_claims(deps.storage, &sender_raw)?;
    claims.push(ClaimInfo {
        bond_type: BondType::LpBond,
        amount: bro_payout,
        claimable_at: Expiration::AtHeight(env.block.height + config.vesting_period_blocks),
    });

    store_claims(deps.storage, &sender_raw, &claims)?;

    Ok(Response::new()
        .add_messages(vec![
            CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: deps.api.addr_humanize(&config.lp_token)?.to_string(),
                funds: vec![],
                msg: to_binary(&Cw20ExecuteMsg::Transfer {
                    recipient: deps
                        .api
                        .addr_humanize(&config.treasury_contract)?
                        .to_string(),
                    amount: lp_amount,
                })?,
            }),
            CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: oracle_contract.to_string(),
                funds: vec![],
                msg: to_binary(&OracleExecuteMsg::UpdatePrice {})?,
            }),
        ])
        .add_attributes(vec![("action", "lp_bond")]))
}

/// ## Description
/// Bond bro tokens by providing ust amount.
/// Returns [`Response`] with specified attributes and messages if operation was successful,
/// otherwise returns [`ContractError`]
/// ## Params
/// * **deps** is an object of type [`DepsMut`]
///
/// * **env** is an object of type [`Env`]
///
/// * **info** is an object of type [`MessageInfo`]
pub fn ust_bond(deps: DepsMut, env: Env, info: MessageInfo) -> Result<Response, ContractError> {
    let config = load_config(deps.storage)?;
    let mut state = load_state(deps.storage)?;

    let bond_amount = extract_ust_amount(&info.funds)?;

    let oracle_contract = deps.api.addr_humanize(&config.oracle_contract)?;
    let bro_amount = query_oracle_price(
        &deps.querier,
        oracle_contract.clone(),
        AssetInfo::NativeToken {
            denom: "uusd".to_string(),
        },
        bond_amount,
    )?
    .amount;

    let bro_payout = apply_discount(config.ust_bonding_discount, bro_amount)?;
    if bro_payout < config.min_bro_payout {
        return Err(ContractError::BondPayoutIsLow {});
    }

    if bro_payout > state.ust_bonding_balance {
        return Err(ContractError::NotEnoughForBondPayout {});
    }

    state.ust_bonding_balance = state.ust_bonding_balance.checked_sub(bro_payout)?;
    store_state(deps.storage, &state)?;

    let sender_raw = deps.api.addr_canonicalize(&info.sender.to_string())?;
    let mut claims = load_claims(deps.storage, &sender_raw)?;
    claims.push(ClaimInfo {
        bond_type: BondType::UstBond,
        amount: bro_payout,
        claimable_at: Expiration::AtHeight(env.block.height + config.vesting_period_blocks),
    });

    store_claims(deps.storage, &sender_raw, &claims)?;

    let ust_transfer = Asset {
        info: AssetInfo::NativeToken {
            denom: "uusd".to_string(),
        },
        amount: bond_amount,
    };

    Ok(Response::new()
        .add_messages(vec![
            ust_transfer.into_msg(
                &deps.querier,
                deps.api.addr_humanize(&config.treasury_contract)?,
            )?,
            CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: oracle_contract.to_string(),
                funds: vec![],
                msg: to_binary(&OracleExecuteMsg::UpdatePrice {})?,
            }),
        ])
        .add_attributes(vec![("action", "ust_bond")]))
}

/// ## Description
/// Claim availalble reward amount.
/// Returns [`Response`] with specified attributes and messages if operation was successful,
/// otherwise returns [`ContractError`]
/// ## Params
/// * **deps** is an object of type [`DepsMut`]
///
/// * **env** is an object of type [`Env`]
///
/// * **info** is an object of type [`MessageInfo`]
pub fn claim(deps: DepsMut, env: Env, info: MessageInfo) -> Result<Response, ContractError> {
    let config = load_config(deps.storage)?;

    let sender_raw = deps.api.addr_canonicalize(&info.sender.to_string())?;

    let mut amount = Uint128::zero();
    // if claim passed vesting period add claimable amount and remove it from claims list
    let claims: Vec<ClaimInfo> = load_claims(deps.storage, &sender_raw)?
        .into_iter()
        .filter(|c| {
            if c.claimable_at.is_expired(&env.block) {
                amount += c.amount;
                false
            } else {
                true
            }
        })
        .collect();

    if amount.is_zero() {
        return Err(ContractError::NothingToClaim {});
    }

    store_claims(deps.storage, &sender_raw, &claims)?;

    Ok(Response::new()
        .add_messages(vec![CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: deps.api.addr_humanize(&config.bro_token)?.to_string(),
            funds: vec![],
            msg: to_binary(&Cw20ExecuteMsg::Transfer {
                recipient: info.sender.to_string(),
                amount,
            })?,
        })])
        .add_attributes(vec![("action", "claim")]))
}

/// ## Description
/// Updates contract settings.
/// Returns [`Response`] with specified attributes and messages if operation was successful,
/// otherwise returns [`ContractError`]
/// ## Params
/// * **deps** is an object of type [`DepsMut`]
///
/// * **owner** is an [`Option`] of type [`String`]
///
/// * **lp_token** is an [`Option`] of type [`String`]
///
/// * **treasury_contract** is an [`Option`] of type [`String`]
///
/// * **astroport_factory** is an [`Option`] of type [`String`]
///
/// * **oracle_contract** is an [`Option`] of type [`String`]
///
/// * **ust_bonding_reward_ratio** is an [`Option`] of type [`Decimal`]
///
/// * **ust_bonding_discount** is an [`Option`] of type [`Decimal`]
///
/// * **lp_bonding_discount** is an [`Option`] of type [`Decimal`]
///
/// * **min_bro_payout** is an [`Option`] of type [`Uint128`]
///
/// * **vesting_period_blocks** is an [`Option`] of type [`u64`]
#[allow(clippy::too_many_arguments)]
pub fn update_config(
    deps: DepsMut,
    owner: Option<String>,
    lp_token: Option<String>,
    treasury_contract: Option<String>,
    astroport_factory: Option<String>,
    oracle_contract: Option<String>,
    ust_bonding_reward_ratio: Option<Decimal>,
    ust_bonding_discount: Option<Decimal>,
    lp_bonding_discount: Option<Decimal>,
    min_bro_payout: Option<Uint128>,
    vesting_period_blocks: Option<u64>,
) -> Result<Response, ContractError> {
    let mut config = load_config(deps.storage)?;

    if let Some(owner) = owner {
        config.owner = deps.api.addr_canonicalize(&owner)?;
    }

    if let Some(lp_token) = lp_token {
        config.lp_token = deps.api.addr_canonicalize(&lp_token)?;
    }

    if let Some(treasury_contract) = treasury_contract {
        config.treasury_contract = deps.api.addr_canonicalize(&treasury_contract)?;
    }

    if let Some(astroport_factory) = astroport_factory {
        config.astroport_factory = deps.api.addr_canonicalize(&astroport_factory)?;
    }

    if let Some(oracle_contract) = oracle_contract {
        config.oracle_contract = deps.api.addr_canonicalize(&oracle_contract)?;
    }

    if let Some(ust_bonding_reward_ratio) = ust_bonding_reward_ratio {
        config.ust_bonding_reward_ratio = ust_bonding_reward_ratio;
    }

    if let Some(ust_bonding_discount) = ust_bonding_discount {
        config.ust_bonding_discount = ust_bonding_discount;
    }

    if let Some(lp_bonding_discount) = lp_bonding_discount {
        config.lp_bonding_discount = lp_bonding_discount;
    }

    if let Some(min_bro_payout) = min_bro_payout {
        config.min_bro_payout = min_bro_payout;
    }

    if let Some(vesting_period_blocks) = vesting_period_blocks {
        config.vesting_period_blocks = vesting_period_blocks;
    }

    store_config(deps.storage, &config)?;
    Ok(Response::new().add_attributes(vec![("action", "update_config")]))
}

/// ## Description
/// Extracts ust amount from provided info.funds input.
/// Otherwise returns [`ContractError`]
/// ## Params
/// * **funds** is an object of type [`&[Coin]`]
fn extract_ust_amount(funds: &[Coin]) -> Result<Uint128, ContractError> {
    if funds.len() != 1 || funds[0].denom != "uusd" || funds[0].amount.is_zero() {
        return Err(ContractError::InvalidFundsInput {});
    }

    Ok(funds[0].amount)
}

/// ## Description
/// Returns the share of assets in the [`Uint128`] object
/// ## Params
/// * **querier** is an object of type [`QuerierWrapper`]
///
/// * **bro_pool** is an object of type [`Asset`]
///
/// * **ust_pool** is an object of type [`Asset`]
///
/// * **lp_amount** is an object of type [`Uint128`]
///
/// * **lp_token_addr** is an object of type [`Addr`]
fn get_share_in_assets(
    querier: &QuerierWrapper,
    bro_pool: &Asset,
    ust_pool: &Asset,
    lp_amount: Uint128,
    lp_token_addr: Addr,
) -> StdResult<(Uint128, Uint128)> {
    let total_share = query_supply(querier, lp_token_addr)?;
    let share_ratio = Decimal::from_ratio(lp_amount, total_share);

    Ok((bro_pool.amount * share_ratio, ust_pool.amount * share_ratio))
}

/// ## Description
/// Queries bro/ust pair using astroport factory.
/// result.0 - bro asset info of type [`Asset`]
/// result.1 - ust asset info of type [`Asset`]
/// ## Params
/// * **querier** is an object of type [`QuerierWrapper`]
///
/// * **astro_factory** is an object of type [`Addr`]
///
/// * **bro_token** is an object of type [`Addr`]
fn query_bro_ust_pair(
    querier: &QuerierWrapper,
    astro_factory: Addr,
    bro_token: Addr,
) -> StdResult<(Asset, Asset)> {
    let asset_info = [
        AssetInfo::NativeToken {
            denom: "uusd".to_string(),
        },
        AssetInfo::Token {
            contract_addr: bro_token,
        },
    ];

    let pools = query_pools(querier, astro_factory, &asset_info)?;
    match &pools[0].info {
        AssetInfo::Token { .. } => Ok((pools[0].clone(), pools[1].clone())),
        AssetInfo::NativeToken { .. } => Ok((pools[1].clone(), pools[0].clone())),
    }
}

/// ## Description
/// Applies bonding discount for provided token amount
/// and returns result in the [`Uint128`] object
/// ## Params
/// * **discount_ratio** is an object of type [`Decimal`]
///
/// * **amount** is an object of type [`Uint128`]
fn apply_discount(discount_ratio: Decimal, amount: Uint128) -> StdResult<Uint128> {
    let discount = Decimal::from_str("1.0")? + discount_ratio;
    let payout = amount * discount;
    Ok(payout)
}
