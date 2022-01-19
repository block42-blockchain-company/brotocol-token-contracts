use cosmwasm_std::{StdResult, QueryRequest, QuerierWrapper, WasmQuery, Addr, to_binary};
use astroport::{
    asset::{Asset, AssetInfo},
    querier::query_pair_info,
};

use crate::{
    epoch_manager::{
        QueryMsg as EpochManagerQuery,
        EpochInfoResponse,
    },
};

pub fn query_epoch_info(
    querier: &QuerierWrapper,
    epoch_manager_contract: Addr,
) -> StdResult<EpochInfoResponse> {
    querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
        contract_addr: epoch_manager_contract.to_string(),
        msg: to_binary(&EpochManagerQuery::EpochInfo {})?,
    }))
}

pub fn query_pools(
    querier: &QuerierWrapper,
    astro_factory: Addr,
    asset_info: &[AssetInfo; 2],
) -> StdResult<[Asset; 2]> {
    let pair_info = query_pair_info(
        querier, 
        astro_factory,
        asset_info,
    )?;

    let pair_addr = pair_info.contract_addr.clone();
    let pools = pair_info.query_pools(querier, pair_addr)?;
    Ok(pools)
}
