use cosmwasm_std::{Deps, StdResult};
use services::bonding::{ClaimInfoResponse, ClaimsResponse, ConfigResponse, StateResponse};

use crate::state::{load_claims, load_config, load_state};

pub fn query_config(deps: Deps) -> StdResult<ConfigResponse> {
    let config = load_config(deps.storage)?;
    let resp = ConfigResponse {
        bro_token: deps.api.addr_humanize(&config.bro_token)?.to_string(),
        lp_token: deps.api.addr_humanize(&config.lp_token)?.to_string(),
        treasury_contract: deps
            .api
            .addr_humanize(&config.treasury_contract)?
            .to_string(),
        ust_bonding_reward_ratio: config.ust_bonding_reward_ratio,
    };

    Ok(resp)
}

pub fn query_state(deps: Deps) -> StdResult<StateResponse> {
    let state = load_state(deps.storage)?;
    let resp = StateResponse {
        ust_bonding_balance: state.ust_bonding_balance,
        lp_bonding_balance: state.lp_bonding_balance,
    };

    Ok(resp)
}

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
