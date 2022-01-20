use cosmwasm_std::{Deps, StdResult};

use crate::state::{load_config, load_stage, load_merkle_root, read_claimed};
use services::airdrop::{ConfigResponse, LatestStageResponse, MerkleRootResponse, IsClaimedResponse};

pub fn query_config(deps: Deps) -> StdResult<ConfigResponse> {
    let config = load_config(deps.storage)?;
    let resp = ConfigResponse {
        owner: deps.api.addr_humanize(&config.owner)?.to_string(),
        bro_token: deps.api.addr_humanize(&config.bro_token)?.to_string(),
    };

    Ok(resp)
}

pub fn query_latest_stage(deps: Deps) -> StdResult<LatestStageResponse> {
    let latest_stage = load_stage(deps.storage)?;
    let resp = LatestStageResponse { latest_stage };

    Ok(resp)
}

pub fn query_merkle_root(deps: Deps, stage: u8) -> StdResult<MerkleRootResponse> {
    let merkle_root = load_merkle_root(deps.storage, stage)?;
    let resp = MerkleRootResponse {
        stage,
        merkle_root,
    };

    Ok(resp)
}

pub fn query_claimed(deps: Deps, stage: u8, address: String) -> StdResult<IsClaimedResponse> {
    let user = deps.api.addr_validate(&address)?;
    let is_claimed = read_claimed(deps.storage, &user, stage)?;
    let resp = IsClaimedResponse { is_claimed };

    Ok(resp)
}