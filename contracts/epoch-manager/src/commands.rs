use cosmwasm_std::{Attribute, Decimal, DepsMut, Response};

use crate::{
    error::ContractError,
    state::{load_config, load_state, store_config, store_state},
};

/// ## Description
/// Updates contract settings.
/// Returns [`Response`] with specified attributes and messages if operation was successful,
/// otherwise returns [`ContractError`]
/// ## Params
/// * **deps** is an object of type [`DepsMut`]
///
/// * **owner** is an [`Option`] field of type [`String`]. Sets new contract owner address
pub fn update_config(deps: DepsMut, owner: Option<String>) -> Result<Response, ContractError> {
    let mut config = load_config(deps.storage)?;

    let mut attributes: Vec<Attribute> = vec![Attribute::new("action", "update_config")];

    if let Some(owner) = owner {
        config.owner = deps.api.addr_canonicalize(&owner)?;
        attributes.push(Attribute::new("owner_changed", &owner));
    }

    store_config(deps.storage, &config)?;

    Ok(Response::new().add_attributes(attributes))
}

/// ## Description
/// Updates contract state.
/// Returns [`Response`] with specified attributes and messages if operation was successful,
/// otherwise returns [`ContractError`]
/// ## Params
/// * **deps** is an object of type [`DepsMut`]
///
/// * **epoch** is an [`Option`] of type [`u64`]. Sets new epoch blocks amount
///
/// * **blocks_per_year** is an [`Option`] of type [`u64`]. Sets new blocks per year amount
///
/// * **bbro_emission_rate** is an [`Option`] of type [`Decimal`]. Sets new bbro emission rate
pub fn update_state(
    deps: DepsMut,
    epoch: Option<u64>,
    blocks_per_year: Option<u64>,
    bbro_emission_rate: Option<Decimal>,
) -> Result<Response, ContractError> {
    let mut state = load_state(deps.storage)?;

    let mut attributes: Vec<Attribute> = vec![Attribute::new("action", "update_state")];

    if let Some(epoch) = epoch {
        state.epoch = epoch;
        attributes.push(Attribute::new("epoch_changed", &epoch.to_string()));
    }

    if let Some(blocks_per_year) = blocks_per_year {
        state.blocks_per_year = blocks_per_year;
        attributes.push(Attribute::new(
            "blocks_per_year_changed",
            &blocks_per_year.to_string(),
        ));
    }

    if let Some(bbro_emission_rate) = bbro_emission_rate {
        state.bbro_emission_rate = bbro_emission_rate;
        attributes.push(Attribute::new(
            "bbro_emission_rate_changed",
            &bbro_emission_rate.to_string(),
        ));
    }

    store_state(deps.storage, &state)?;

    Ok(Response::new().add_attributes(attributes))
}
