use cosmwasm_std::{DepsMut, Env, Response, Uint128};

use terraswap::asset::{Asset, AssetInfo};

use crate::{error::ContractError, queries};

/// ## Description
/// Sends whole treasury balance of specified asset to recipient.
/// Returns [`Response`] with specified attributes and messages if operation was successful,
/// otherwise returns [`ContractError`]
/// ## Params
/// * **deps** is an object of type [`DepsMut`]
///
/// * **env** is an object of type [`Env`]
///
/// * **asset_info** is an object of type [`AssetInfo`]
///
/// * **amount** is an object of type [`Uint128`]
///
/// * **recipient** is a field of type [`String`]
pub fn spend(
    deps: DepsMut,
    env: Env,
    asset_info: AssetInfo,
    amount: Uint128,
    recipient: String,
) -> Result<Response, ContractError> {
    let balance = queries::query_asset_balance(deps.as_ref(), env, asset_info.clone())?.amount;
    if balance.is_zero() || balance < amount {
        return Err(ContractError::InsufficientFunds {});
    }

    let asset = Asset {
        info: asset_info,
        amount,
    };

    Ok(Response::new()
        .add_messages(vec![
            asset.into_msg(&deps.querier, deps.api.addr_validate(&recipient)?)?
        ])
        .add_attributes(vec![
            ("action", "spend"),
            ("recipient", &recipient),
            ("amount", &amount.to_string()),
        ]))
}
