use cosmwasm_std::{Addr, Coin, Decimal, QuerierWrapper, StdResult, Uint128};
use std::str::FromStr;

use crate::ContractError;

use astroport::{
    asset::{Asset, AssetInfo},
    querier::query_supply,
};

/// ## Description
/// Extracts ust amount from provided info.funds input.
/// Otherwise returns [`ContractError`]
/// ## Params
/// * **funds** is an object of type [`&[Coin]`]
pub fn extract_native_token(funds: &[Coin]) -> Result<Asset, ContractError> {
    if funds.len() != 1 || funds[0].denom != "uusd" || funds[0].amount.is_zero() {
        return Err(ContractError::InvalidFundsInput {});
    }

    Ok(Asset {
        info: AssetInfo::NativeToken {
            denom: "uusd".to_string(),
        },
        amount: funds[0].amount,
    })
}

/// ## Description
/// Applies bonding discount for provided token amount
/// and returns result in the [`Uint128`] object
/// ## Params
/// * **discount_ratio** is an object of type [`Decimal`]
///
/// * **amount** is an object of type [`Uint128`]
pub fn apply_discount(discount_ratio: Decimal, amount: Uint128) -> StdResult<Uint128> {
    let discount = Decimal::from_str("1.0")? + discount_ratio;
    let payout = amount * discount;
    Ok(payout)
}

/// ## Description
/// Returns the share of assets in the [`Uint128`] object
/// ## Params
/// * **querier** is an object of type [`QuerierWrapper`]
///
/// * **bro_pool** is an object of type [`Asset`]
///
/// * **ust_pool** is an object of type [`Asset`]
///
/// * **lp_amount** is an object of type [`Uint128`]
///
/// * **lp_token_addr** is an object of type [`Addr`]
pub fn get_share_in_assets(
    querier: &QuerierWrapper,
    bro_pool: &Asset,
    ust_pool: &Asset,
    lp_amount: Uint128,
    lp_token_addr: Addr,
) -> StdResult<(Uint128, Uint128)> {
    let total_share = query_supply(querier, lp_token_addr)?;
    let share_ratio = Decimal::from_ratio(lp_amount, total_share);

    Ok((bro_pool.amount * share_ratio, ust_pool.amount * share_ratio))
}
