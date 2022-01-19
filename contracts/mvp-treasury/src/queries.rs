use cosmwasm_std::{Deps, StdResult, Env};
use terraswap::{
    asset::AssetInfo,
    querier::{query_balance, query_token_balance},
};

use crate::{
    state::load_config
};

use services::treasury::{
    BalanceResponse,
    ConfigResponse,
};


pub fn query_config(deps: Deps) -> StdResult<ConfigResponse> {
    let config = load_config(deps.storage)?;
    let resp = ConfigResponse {
        owner: deps.api.addr_humanize(&config.owner)?.to_string(),
    };

    Ok(resp)
}

pub fn query_asset_balance(deps: Deps, env: Env, asset_info: AssetInfo) -> StdResult<BalanceResponse> {
    let amount = match asset_info {
        AssetInfo::NativeToken { denom } => {
            query_balance(&deps.querier, env.contract.address.clone(), denom)?
        },
        AssetInfo::Token { contract_addr } => {
            query_token_balance(
                &deps.querier, 
                deps.api.addr_validate(&contract_addr)?, 
                env.contract.address.clone()
            )?
        }
    };
    let resp = BalanceResponse { amount };

    Ok(resp)
}
