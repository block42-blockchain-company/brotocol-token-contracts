use cosmwasm_std::{Deps, StdResult};

use crate::state::{load_config, load_merkle_root, load_stage, read_claimed};

use services::airdrop::{
    ConfigResponse, IsClaimedResponse, LatestStageResponse, MerkleRootResponse,
};

/// ## Description
/// Returns airdrop contract config in the [`ConfigResponse`] object
/// ## Params
/// * **deps** is an object of type [`Deps`]
pub fn query_config(deps: Deps) -> StdResult<ConfigResponse> {
    let config = load_config(deps.storage)?;
    let resp = ConfigResponse {
        owner: deps.api.addr_humanize(&config.owner)?.to_string(),
        bro_token: deps.api.addr_humanize(&config.bro_token)?.to_string(),
    };

    Ok(resp)
}

/// ## Description
/// Returns the number of latest stage in the [`LatestStageResponse`] object
/// ## Params
/// * **deps** is an object of type [`Deps`]
pub fn query_latest_stage(deps: Deps) -> StdResult<LatestStageResponse> {
    let latest_stage = load_stage(deps.storage)?;
    let resp = LatestStageResponse { latest_stage };

    Ok(resp)
}

/// ## Description
/// Returns merkle root information by specified stage in the [`MerkleRootResponse`] object
/// ## Params
/// * **deps** is an object of type [`Deps`]
///
/// * **stage** is a field of type [`u8`]
pub fn query_merkle_root(deps: Deps, stage: u8) -> StdResult<MerkleRootResponse> {
    let merkle_root = load_merkle_root(deps.storage, stage)?;
    let resp = MerkleRootResponse { stage, merkle_root };

    Ok(resp)
}

/// ## Description
/// Returns claim information by specified stage and address in the [`IsClaimedResponse`] object
/// ## Params
/// * **deps** is an object of type [`Deps`]
///
/// * **stage** is a field of type [`u8`]
///
/// * **address** is a field of type [`String`]
pub fn query_claimed(deps: Deps, stage: u8, address: String) -> StdResult<IsClaimedResponse> {
    let user = deps.api.addr_validate(&address)?;
    let is_claimed = read_claimed(deps.storage, &user, stage)?;
    let resp = IsClaimedResponse { is_claimed };

    Ok(resp)
}
