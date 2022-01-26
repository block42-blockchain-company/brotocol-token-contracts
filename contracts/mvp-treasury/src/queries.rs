use cosmwasm_std::{Deps, Env, StdResult};
use terraswap::{
    asset::AssetInfo,
    querier::{query_balance, query_token_balance},
};

use crate::state::load_config;

use services::treasury::{BalanceResponse, ConfigResponse};

/// ## Description
/// Returns mvp treasury contract config in the [`ConfigResponse`] object
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
/// Returns mvp-treasuty contract balance of specified asset in the [`BalanceResponse`] object
/// ## Params
/// * **deps** is an object of type [`Deps`]
///
/// * **env** is an object of type [`Env`]
///
/// * **asset_info** is an object of type [`AssetInfo`]
pub fn query_asset_balance(
    deps: Deps,
    env: Env,
    asset_info: AssetInfo,
) -> StdResult<BalanceResponse> {
    let amount = match asset_info {
        AssetInfo::NativeToken { denom } => {
            query_balance(&deps.querier, env.contract.address, denom)?
        }
        AssetInfo::Token { contract_addr } => query_token_balance(
            &deps.querier,
            deps.api.addr_validate(&contract_addr)?,
            env.contract.address,
        )?,
    };
    let resp = BalanceResponse { amount };

    Ok(resp)
}
