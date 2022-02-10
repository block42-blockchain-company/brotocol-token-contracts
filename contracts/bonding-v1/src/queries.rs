use cosmwasm_std::{Deps, StdResult};
use services::bonding::{ClaimInfoResponse, ClaimsResponse, ConfigResponse, StateResponse};

use crate::state::{load_claims, load_config, load_state};

/// ## Description
/// Returns bonding contract config in the [`ConfigResponse`] object
/// ## Params
/// * **deps** is an object of type [`Deps`]
pub fn query_config(deps: Deps) -> StdResult<ConfigResponse> {
    let config = load_config(deps.storage)?;
    let resp = ConfigResponse {
        owner: deps.api.addr_humanize(&config.owner)?.to_string(),
        bro_token: deps.api.addr_humanize(&config.bro_token)?.to_string(),
        lp_token: deps.api.addr_humanize(&config.lp_token)?.to_string(),
        treasury_contract: deps
            .api
            .addr_humanize(&config.treasury_contract)?
            .to_string(),
        astroport_factory: deps
            .api
            .addr_humanize(&config.astroport_factory)?
            .to_string(),
        oracle_contract: deps.api.addr_humanize(&config.oracle_contract)?.to_string(),
        ust_bonding_reward_ratio: config.ust_bonding_reward_ratio,
        ust_bonding_discount: config.ust_bonding_discount,
        lp_bonding_discount: config.lp_bonding_discount,
        min_bro_payout: config.min_bro_payout,
        vesting_period_blocks: config.vesting_period_blocks,
        lp_bonding_enabled: config.lp_bonding_enabled,
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
