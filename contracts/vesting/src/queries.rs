use cosmwasm_std::{Deps, Env, StdResult};

use crate::state::{load_config, load_vesting_info, read_vesting_infos};

use services::{
    common::OrderBy,
    vesting::{
        ClaimableAmountResponse, ConfigResponse, VestingAccountResponse, VestingAccountsResponse,
    },
};

/// ## Description
/// Returns vesting contract config in the [`ConfigResponse`] object
/// ## Params
/// * **deps** is an object of type [`Deps`]
pub fn query_config(deps: Deps) -> StdResult<ConfigResponse> {
    let config = load_config(deps.storage)?;
    let resp = ConfigResponse {
        owner: deps.api.addr_humanize(&config.owner)?.to_string(),
        bro_token: deps.api.addr_humanize(&config.bro_token)?.to_string(),
        genesis_time: config.genesis_time,
    };

    Ok(resp)
}

/// ## Description
/// Returns vesting schedules for specified account in the [`VestingAccountResponse`] object
/// ## Params
/// * **deps** is an object of type [`Deps`]
///
/// * **address** is a field of type [`String`]
pub fn query_vesting_account(deps: Deps, address: String) -> StdResult<VestingAccountResponse> {
    let info = load_vesting_info(deps.storage, &deps.api.addr_validate(&address)?)?;
    let resp = VestingAccountResponse { address, info };

    Ok(resp)
}

/// ## Description
/// Returns a list of accounts for given input params in the [`VestingAccountsResponse`] object
/// ## Params
/// * **deps** is an object of type [`Deps`]
///
/// * **start_after** is an [`Option`] field of type [`Addr`]
///
/// * **limit** is an [`Option`] field of type [`u32`]
///
/// * **order_by** is an [`Option`] field of type [`OrderBy`]
pub fn query_vesting_accounts(
    deps: Deps,
    start_after: Option<String>,
    limit: Option<u32>,
    order_by: Option<OrderBy>,
) -> StdResult<VestingAccountsResponse> {
    let vesting_infos = if let Some(start_after) = start_after {
        let start_after = Some(deps.api.addr_validate(&start_after)?);
        read_vesting_infos(deps.storage, start_after, limit, order_by)?
    } else {
        read_vesting_infos(deps.storage, None, limit, order_by)?
    };

    let vesting_accounts_response: StdResult<Vec<VestingAccountResponse>> = vesting_infos
        .iter()
        .map(|vesting_account| {
            Ok(VestingAccountResponse {
                address: vesting_account.0.to_string(),
                info: vesting_account.1.clone(),
            })
        })
        .collect();

    Ok(VestingAccountsResponse {
        vesting_accounts: vesting_accounts_response?,
    })
}

/// ## Description
/// Returns available amount to claim for specified account in the [`ClaimableAmountResponse`] object
/// ## Params
/// * **deps** is an object of type [`Deps`]
///
/// * **env** is an object of type [`Env`]
///
/// * **address** is a field of type [`String`]
pub fn query_claimable_amount(
    deps: Deps,
    env: Env,
    address: String,
) -> StdResult<ClaimableAmountResponse> {
    let current_time = env.block.time.seconds();
    let info = load_vesting_info(deps.storage, &deps.api.addr_validate(&address)?)?;
    let claimable_amount = info.compute_claim_amount(current_time);
    let resp = ClaimableAmountResponse {
        address,
        claimable_amount,
    };

    Ok(resp)
}
