use cosmwasm_std::{Deps, StdResult};
use services::epoch_manager::{ConfigResponse, EpochInfoResponse};

use crate::state::{load_config, load_state};

/// ## Description
/// Returns epoch manager contract config in the [`ConfigResponse`] object
/// ## Params
/// * **deps** is an object of type [`Deps`]
pub fn query_config(deps: Deps) -> StdResult<ConfigResponse> {
    let config = load_config(deps.storage)?;
    let resp = ConfigResponse {
        owner: deps.api.addr_humanize(&config.owner)?.to_string(),
    };

    Ok(resp)
}

/// ## Description
/// Returns epoch-manager contract state in the [`EpochInfoResponse`] object
/// ## Params
/// * **deps** is an object of type [`Deps`]
pub fn query_epoch_info(deps: Deps) -> StdResult<EpochInfoResponse> {
    let state = load_state(deps.storage)?;
    let resp = EpochInfoResponse {
        epoch: state.epoch,
        blocks_per_year: state.blocks_per_year,
        bbro_emission_rate: state.bbro_emission_rate,
    };

    Ok(resp)
}
