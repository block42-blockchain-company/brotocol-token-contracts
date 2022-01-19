use cosmwasm_std::{Deps, StdResult};
use services::epoch_manager::{EpochInfoResponse, ConfigResponse};

use crate::state::{load_state, load_config};

pub fn query_config(deps: Deps) -> StdResult<ConfigResponse> {
    let config = load_config(deps.storage)?;
    let resp = ConfigResponse {
        owner: deps.api.addr_humanize(&config.owner)?.to_string(),
    };

    Ok(resp)
}

pub fn query_epoch_info(deps: Deps) -> StdResult<EpochInfoResponse> {
    let state = load_state(deps.storage)?;
    let resp = EpochInfoResponse {
        epoch: state.epoch,
        blocks_per_year: state.blocks_per_year,
        bbro_emission_rate: state.bbro_emission_rate,
    };

    Ok(resp)
}