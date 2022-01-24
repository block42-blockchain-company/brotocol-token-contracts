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

use services::querier::query_pools;

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

pub fn lp_bond(
    deps: DepsMut,
    env: Env,
    sender_raw: CanonicalAddr,
    lp_amount: Uint128,
) -> Result<Response, ContractError> {
    let config = load_config(deps.storage)?;
    let mut state = load_state(deps.storage)?;

    let pools = query_bro_ust_pool(
        &deps.querier,
        deps.api.addr_humanize(&config.astroport_factory)?,
        deps.api.addr_humanize(&config.bro_token)?,
    )?;

    let (bro_amount, ust_amount) = convert_lp_into_amounts(
        &deps.querier,
        &pools,
        lp_amount,
        deps.api.addr_humanize(&config.lp_token)?,
    )?;

    let bond_amount = ust_amount.checked_add(convert_bro_into_ust(bro_amount, &pools)?)?;
    let bro_amount = convert_ust_into_bro(bond_amount, &pools)?;

    let bro_payout = apply_discount(config.lp_bonding_discount, bro_amount)?;
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
        .add_messages(vec![CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: deps.api.addr_humanize(&config.lp_token)?.to_string(),
            funds: vec![],
            msg: to_binary(&Cw20ExecuteMsg::Transfer {
                recipient: deps
                    .api
                    .addr_humanize(&config.treasury_contract)?
                    .to_string(),
                amount: lp_amount,
            })?,
        })])
        .add_attributes(vec![("action", "lp_bond")]))
}

pub fn ust_bond(deps: DepsMut, env: Env, info: MessageInfo) -> Result<Response, ContractError> {
    let config = load_config(deps.storage)?;
    let mut state = load_state(deps.storage)?;

    let pools = query_bro_ust_pool(
        &deps.querier,
        deps.api.addr_humanize(&config.astroport_factory)?,
        deps.api.addr_humanize(&config.bro_token)?,
    )?;

    let bond_amount = extract_ust_amount(&info.funds)?;
    let bro_amount = convert_ust_into_bro(bond_amount, &pools)?;

    let bro_payout = apply_discount(config.ust_bonding_discount, bro_amount)?;
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
        .add_messages(vec![ust_transfer.into_msg(
            &deps.querier,
            deps.api.addr_humanize(&config.treasury_contract)?,
        )?])
        .add_attributes(vec![("action", "ust_bond")]))
}

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

#[allow(clippy::too_many_arguments)]
pub fn update_config(
    deps: DepsMut,
    owner: Option<String>,
    lp_token: Option<String>,
    treasury_contract: Option<String>,
    astroport_factory: Option<String>,
    ust_bonding_reward_ratio: Option<Decimal>,
    ust_bonding_discount: Option<Decimal>,
    lp_bonding_discount: Option<Decimal>,
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

    if let Some(ust_bonding_reward_ratio) = ust_bonding_reward_ratio {
        config.ust_bonding_reward_ratio = ust_bonding_reward_ratio;
    }

    if let Some(ust_bonding_discount) = ust_bonding_discount {
        config.ust_bonding_discount = ust_bonding_discount;
    }

    if let Some(lp_bonding_discount) = lp_bonding_discount {
        config.lp_bonding_discount = lp_bonding_discount;
    }

    if let Some(vesting_period_blocks) = vesting_period_blocks {
        config.vesting_period_blocks = vesting_period_blocks;
    }

    store_config(deps.storage, &config)?;
    Ok(Response::new().add_attributes(vec![("action", "update_config")]))
}

fn extract_ust_amount(funds: &[Coin]) -> Result<Uint128, ContractError> {
    if funds.len() != 1 || funds[0].denom != "uusd" || funds[0].amount.is_zero() {
        return Err(ContractError::InvalidFundsInput {});
    }

    Ok(funds[0].amount)
}

fn convert_ust_into_bro(ust_amount: Uint128, pools: &[Asset; 2]) -> StdResult<Uint128> {
    let bro_amount = pools[0]
        .amount
        .checked_div(pools[1].amount)?
        .checked_mul(ust_amount)?;

    Ok(bro_amount)
}

fn convert_bro_into_ust(bro_amount: Uint128, pools: &[Asset; 2]) -> StdResult<Uint128> {
    let ust_amount = pools[1]
        .amount
        .checked_div(pools[0].amount)?
        .checked_mul(bro_amount)?;

    Ok(ust_amount)
}

// pools[0] - bro asset info
// pools[1] - ust asset info
fn query_bro_ust_pool(
    querier: &QuerierWrapper,
    astro_factory: Addr,
    bro_token: Addr,
) -> StdResult<[Asset; 2]> {
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
        AssetInfo::Token { .. } => Ok(pools),
        AssetInfo::NativeToken { .. } => Ok([pools[1].clone(), pools[0].clone()]),
    }
}

fn convert_lp_into_amounts(
    querier: &QuerierWrapper,
    pools: &[Asset; 2],
    lp_amount: Uint128,
    lp_token_addr: Addr,
) -> StdResult<(Uint128, Uint128)> {
    let total_share = query_supply(querier, lp_token_addr)?;
    let share_ratio = Decimal::from_ratio(lp_amount, total_share);

    Ok((pools[0].amount * share_ratio, pools[1].amount * share_ratio))
}

fn apply_discount(discount_ratio: Decimal, amount: Uint128) -> StdResult<Uint128> {
    let discount = Decimal::from_str("1.0")? + discount_ratio;
    let payout = amount * discount;
    Ok(payout)
}
