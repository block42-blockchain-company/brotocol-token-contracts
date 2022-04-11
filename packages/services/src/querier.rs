use cosmwasm_std::{to_binary, Addr, QuerierWrapper, QueryRequest, StdResult, Uint128, WasmQuery};
use cw20::{BalanceResponse as Cw20BalanceResponse, Cw20QueryMsg};

use crate::{
    epoch_manager::{EpochInfoResponse, QueryMsg as EpochManagerQueryMsg},
    oracle::{ConsultPriceResponse, QueryMsg as OracleQueryMsg},
    rewards::{QueryMsg as RewardsPoolQueryMsg, RewardsPoolBalanceResponse},
    staking::{ConfigResponse as StakingConfigResponse, QueryMsg as StakingQueryMsg},
};

use astroport::{
    asset::{Asset, AssetInfo},
    pair::{CumulativePricesResponse, QueryMsg as PairQueryMsg, SimulationResponse},
    querier::query_pair_info,
};

/// ## Description
/// Returns the token balance at the specified contract address.
/// ## Params
/// * **querier** is an object of type [`QuerierWrapper`].
///
/// * **contract_addr** is an object of type [`Addr`]. Sets the address of the contract for which
/// the balance will be requested
///
/// * **account_addr** is an object of type [`Addr`].
pub fn query_token_balance(
    querier: &QuerierWrapper,
    contract_addr: Addr,
    account_addr: Addr,
) -> StdResult<Uint128> {
    // load balance from the token contract
    let res: Cw20BalanceResponse = querier
        .query(&QueryRequest::Wasm(WasmQuery::Smart {
            contract_addr: String::from(contract_addr),
            msg: to_binary(&Cw20QueryMsg::Balance {
                address: String::from(account_addr),
            })?,
        }))
        .unwrap_or_else(|_| Cw20BalanceResponse {
            balance: Uint128::zero(),
        });

    Ok(res.balance)
}

/// ## Description
/// Returns the epoch info at the specified contract address.
/// ## Params
/// * **querier** is an object of type [`QuerierWrapper`].
///
/// * **epoch_manager_contract** is an object of type [`Addr`]. Sets the address of the contract for which
/// the epoch-manager will be requested
pub fn query_epoch_info(
    querier: &QuerierWrapper,
    epoch_manager_contract: Addr,
) -> StdResult<EpochInfoResponse> {
    querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
        contract_addr: epoch_manager_contract.to_string(),
        msg: to_binary(&EpochManagerQueryMsg::EpochInfo {})?,
    }))
}

/// ## Description
/// Returns the rewards pool balance info at the specified contract address.
/// ## Params
/// * **querier** is an object of type [`QuerierWrapper`].
///
/// * **rewards_pool_contract** is an object of type [`Addr`]. Sets the address of the contract for which
/// the rewards-pool will be requested
pub fn query_rewards_pool_balance(
    querier: &QuerierWrapper,
    rewards_pool_contract: Addr,
) -> StdResult<RewardsPoolBalanceResponse> {
    querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
        contract_addr: rewards_pool_contract.to_string(),
        msg: to_binary(&RewardsPoolQueryMsg::Balance {})?,
    }))
}

/// ## Description
/// Returns the asset info of pair at the specified contract address.
/// ## Params
/// * **querier** is an object of type [`QuerierWrapper`].
///
/// * **astro_factory** is an object of type [`Addr`]. Sets the address of the contract for which
/// the pair will be requested
///
/// * **asset_info** is a slice of type [`AssetInfo`]
pub fn query_pools(
    querier: &QuerierWrapper,
    astro_factory: Addr,
    asset_info: &[AssetInfo; 2],
) -> StdResult<[Asset; 2]> {
    let pair_info = query_pair_info(querier, astro_factory, asset_info)?;

    let pair_addr = pair_info.contract_addr.clone();
    let pools = pair_info.query_pools(querier, pair_addr)?;
    Ok(pools)
}

/// ## Description
/// Returns information about the cumulative prices in a [`CumulativePricesResponse`] object.
/// ## Params
/// * **querier** is an object of type [`QuerierWrapper`].
///
/// * **pair_contract** is an object of type [`Addr`].
pub fn query_cumulative_prices(
    querier: &QuerierWrapper,
    pair_contract: Addr,
) -> StdResult<CumulativePricesResponse> {
    querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
        contract_addr: pair_contract.to_string(),
        msg: to_binary(&PairQueryMsg::CumulativePrices {})?,
    }))
}

/// ## Description
/// Returns information about the prices in a [`SimulationResponse`] object.
/// ## Params
/// * **querier** is an object of type [`QuerierWrapper`].
///
/// * **pair_contract** is an object of type [`Addr`].
///
/// * **asset** is an object of type [`Asset`].
pub fn query_prices(
    querier: &QuerierWrapper,
    pair_contract: Addr,
    asset: Asset,
) -> StdResult<SimulationResponse> {
    querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
        contract_addr: pair_contract.to_string(),
        msg: to_binary(&PairQueryMsg::Simulation { offer_asset: asset })?,
    }))
}

/// ## Description
/// Returns calculated average amount with updated precision in the [`ConsultPriceResponse`] object.
/// ## Params
/// * **querier** is an object of type [`QuerierWrapper`].
///
/// * **oracle_contract** is an object of type [`Addr`].
///
/// * **asset_info** is an object of type [`AssetInfo`]
///
/// * **amount** is an object of type [`Uint128`]
pub fn query_oracle_price(
    querier: &QuerierWrapper,
    oracle_contract: Addr,
    asset_info: AssetInfo,
    amount: Uint128,
) -> StdResult<ConsultPriceResponse> {
    querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
        contract_addr: oracle_contract.to_string(),
        msg: to_binary(&OracleQueryMsg::ConsultPrice {
            asset: asset_info,
            amount,
        })?,
    }))
}

/// ## Description
/// Returns a [`bool`] type whether prices are ready to be updated or not.
/// ## Params
/// * **querier** is an object of type [`QuerierWrapper`].
///
/// * **oracle_contract** is an object of type [`Addr`].
pub fn query_is_oracle_ready_to_trigger(
    querier: &QuerierWrapper,
    oracle_contract: Addr,
) -> StdResult<bool> {
    querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
        contract_addr: oracle_contract.to_string(),
        msg: to_binary(&OracleQueryMsg::IsReadyToTrigger {})?,
    }))
}

/// ## Description
/// Returns staking contract config in the [`StakingConfigResponse`] object.
/// ## Params
/// * **querier** is an object of type [`QuerierWrapper`].
///
/// * **staking_contract** is an object of type [`Addr`]
pub fn query_staking_config(
    querier: &QuerierWrapper,
    staking_contract: Addr,
) -> StdResult<StakingConfigResponse> {
    querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
        contract_addr: staking_contract.to_string(),
        msg: to_binary(&StakingQueryMsg::Config {})?,
    }))
}

/// ## Description
/// Queries bro/ust pair using astroport factory.
/// result.0 - bro asset info of type [`Asset`]
/// result.1 - ust asset info of type [`Asset`]
/// ## Params
/// * **querier** is an object of type [`QuerierWrapper`]
///
/// * **astro_factory** is an object of type [`Addr`]
///
/// * **bro_token** is an object of type [`Addr`]
pub fn query_bro_ust_pair(
    querier: &QuerierWrapper,
    astro_factory: Addr,
    bro_token: Addr,
) -> StdResult<(Asset, Asset)> {
    let asset_info = [
        AssetInfo::NativeToken {
            denom: "uusd".to_string(),
        },
        AssetInfo::Token {
            contract_addr: bro_token,
        },
    ];

    let pools = query_pools(querier, astro_factory, &asset_info)?;
    match &pools[0].info {
        AssetInfo::Token { .. } => Ok((pools[0].clone(), pools[1].clone())),
        AssetInfo::NativeToken { .. } => Ok((pools[1].clone(), pools[0].clone())),
    }
}
