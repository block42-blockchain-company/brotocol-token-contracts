use astroport::asset::AssetInfo;
use cosmwasm_std::{Deps, StdError, StdResult, Uint128};

use crate::{
    state::{load_claims, load_config, load_state, BondingMode},
    utils::{apply_discount, get_share_in_assets},
};

use services::{
    bonding::{
        BondingModeMsg, ClaimInfoResponse, ClaimsResponse, ConfigResponse,
        SimulateExchangeResponse, StateResponse,
    },
    querier::{query_bro_ust_pair, query_oracle_price},
};

/// ## Description
/// Returns bonding contract config in the [`ConfigResponse`] object
/// ## Params
/// * **deps** is an object of type [`Deps`]
pub fn query_config(deps: Deps) -> StdResult<ConfigResponse> {
    let config = load_config(deps.storage)?;

    let bonding_mode = match config.bonding_mode {
        BondingMode::Normal {
            ust_bonding_reward_ratio,
            lp_token,
            lp_bonding_discount,
            vesting_period_blocks,
        } => BondingModeMsg::Normal {
            ust_bonding_reward_ratio,
            lp_token: deps.api.addr_humanize(&lp_token)?.to_string(),
            lp_bonding_discount,
            vesting_period_blocks,
        },
        BondingMode::Community {
            staking_contract,
            epochs_locked,
        } => BondingModeMsg::Community {
            staking_contract: deps.api.addr_humanize(&staking_contract)?.to_string(),
            epochs_locked,
        },
    };

    let resp = ConfigResponse {
        owner: deps.api.addr_humanize(&config.owner)?.to_string(),
        bro_token: deps.api.addr_humanize(&config.bro_token)?.to_string(),
        rewards_pool_contract: deps
            .api
            .addr_humanize(&config.rewards_pool_contract)?
            .to_string(),
        treasury_contract: deps
            .api
            .addr_humanize(&config.treasury_contract)?
            .to_string(),
        astroport_factory: deps
            .api
            .addr_humanize(&config.astroport_factory)?
            .to_string(),
        oracle_contract: deps.api.addr_humanize(&config.oracle_contract)?.to_string(),
        ust_bonding_discount: config.ust_bonding_discount,
        min_bro_payout: config.min_bro_payout,
        bonding_mode,
    };

    Ok(resp)
}

/// ## Description
/// Returns bonding contract state in the [`StateResponse`] object
/// ## Params
/// * **deps** is an object of type [`Deps`]
pub fn query_state(deps: Deps) -> StdResult<StateResponse> {
    let state = load_state(deps.storage)?;
    let resp = StateResponse {
        ust_bonding_balance: state.ust_bonding_balance,
        lp_bonding_balance: state.lp_bonding_balance,
    };

    Ok(resp)
}

/// ## Description
/// Returns available claims for bonder by specified address in the [`ClaimsResponse`] object
/// ## Params
/// * **deps** is an object of type [`Deps`]
///
/// * **address** is a field of type [`String`]
pub fn query_claims(deps: Deps, address: String) -> StdResult<ClaimsResponse> {
    let address_raw = deps.api.addr_canonicalize(&address)?;
    let claims: Vec<ClaimInfoResponse> = load_claims(deps.storage, &address_raw)?
        .into_iter()
        .map(|c| ClaimInfoResponse {
            bond_type: c.bond_type.to_string(),
            amount: c.amount,
            claimable_at: c.claimable_at,
        })
        .collect();

    let resp = ClaimsResponse { claims };
    Ok(resp)
}

/// ## Description
/// Returns simulated bro bond using specified uusd amount in the [`SimulateExchangeResponse`] object
/// ## Params
/// * **deps** is an object of type [`Deps`]
///
/// * **uusd_amount** is an object of type [`Uint128`]
pub fn simulate_ust_bond(deps: Deps, uusd_amount: Uint128) -> StdResult<SimulateExchangeResponse> {
    let config = load_config(deps.storage)?;
    let state = load_state(deps.storage)?;

    let bro_amount = query_oracle_price(
        &deps.querier,
        deps.api.addr_humanize(&config.oracle_contract)?,
        AssetInfo::NativeToken {
            denom: "uusd".to_string(),
        },
        uusd_amount,
    )?
    .amount;

    let bro_payout = apply_discount(config.ust_bonding_discount, bro_amount)?;
    let can_be_exchanged =
        bro_payout >= config.min_bro_payout && bro_payout <= state.ust_bonding_balance;

    let resp = SimulateExchangeResponse {
        bro_payout,
        can_be_exchanged,
    };

    Ok(resp)
}

/// ## Description
/// Returns simulated bro bond using specified ust/bro lp token amount in the [`SimulateExchangeResponse`] object
/// ## Params
/// * **deps** is an object of type [`Deps`]
///
/// * **lp_amount** is an object of type [`Uint128`]
pub fn simulate_lp_bond(deps: Deps, lp_amount: Uint128) -> StdResult<SimulateExchangeResponse> {
    let config = load_config(deps.storage)?;
    let state = load_state(deps.storage)?;

    let (lp_token, lp_bonding_discount) = match config.bonding_mode {
        BondingMode::Normal {
            lp_bonding_discount,
            lp_token,
            ..
        } => (lp_token, lp_bonding_discount),
        BondingMode::Community { .. } => {
            return Err(StdError::generic_err("LP Token bonding disabled"))
        }
    };

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
        deps.api.addr_humanize(&lp_token)?,
    )?;

    let bro_amount = bro_share.checked_add(
        query_oracle_price(
            &deps.querier,
            deps.api.addr_humanize(&config.oracle_contract)?,
            AssetInfo::NativeToken {
                denom: "uusd".to_string(),
            },
            ust_share,
        )?
        .amount,
    )?;

    let bro_payout = apply_discount(lp_bonding_discount, bro_amount)?;
    let can_be_exchanged =
        bro_payout >= config.min_bro_payout && bro_payout <= state.lp_bonding_balance;

    let resp = SimulateExchangeResponse {
        bro_payout,
        can_be_exchanged,
    };

    Ok(resp)
}
