use cosmwasm_std::{
    to_binary, CosmosMsg, DepsMut, Env, MessageInfo, Response, StdError, StdResult, SubMsg, WasmMsg,
};
use cw20::Cw20ExecuteMsg;

use crate::{
    error::ContractError,
    state::{load_config, load_vesting_info, store_config, store_vesting_info},
};

use services::vesting::{VestingAccount, VestingInfo, VestingSchedule};

/// ## Description
/// Claims availalble amount for message sender.
/// Returns [`Response`] with specified attributes and messages if operation was succussful,
/// otherwise returns [`ContractError`]
/// ## Params
/// * **deps** is an object of type [`DepsMut`]
///
/// * **env** is an object of type [`Env`]
///
/// * **info** is an object of type [`MessageInfo`]
pub fn claim(deps: DepsMut, env: Env, info: MessageInfo) -> Result<Response, ContractError> {
    let current_time = env.block.time.seconds();
    let address = info.sender;

    let config = load_config(deps.storage)?;
    let mut vesting_info = load_vesting_info(deps.storage, &address)?;

    let claim_amount = vesting_info.compute_claim_amount(current_time);
    let msgs: Vec<SubMsg> = if claim_amount.is_zero() {
        vec![]
    } else {
        vec![SubMsg::new(CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: deps.api.addr_humanize(&config.bro_token)?.to_string(),
            funds: vec![],
            msg: to_binary(&Cw20ExecuteMsg::Transfer {
                recipient: address.to_string(),
                amount: claim_amount,
            })?,
        }))]
    };

    vesting_info.last_claim_time = current_time;
    store_vesting_info(deps.storage, &address, &vesting_info)?;

    Ok(Response::new().add_submessages(msgs).add_attributes(vec![
        ("action", "claim"),
        ("address", &address.to_string()),
        ("claim_amount", &claim_amount.to_string()),
        ("last_claim_time", &current_time.to_string()),
    ]))
}

/// ## Description
/// Updates contract settings.
/// Returns [`Response`] with specified attributes and messages if operation was succussful,
/// otherwise returns [`ContractError`]
/// ## Params
/// * **deps** is an object of type [`DepsMut`]
///
/// * **owner** is an [`Option`] field of type [`String`]. Sets new contract owner address
///
/// * **genesis_time** is an [`Option`] field of type [`u64`]. Sets new genesis time frame
pub fn update_config(
    deps: DepsMut,
    owner: Option<String>,
    genesis_time: Option<u64>,
) -> Result<Response, ContractError> {
    let mut config = load_config(deps.storage)?;
    if let Some(owner) = owner {
        config.owner = deps.api.addr_canonicalize(&owner)?;
    }

    if let Some(genesis_time) = genesis_time {
        config.genesis_time = genesis_time;
    }

    store_config(deps.storage, &config)?;

    Ok(Response::new().add_attribute("action", "update_config"))
}

/// ## Description
/// Registers vesting accounts for future distribution
/// /// Returns [`Response`] with specified attributes and messages if operation was succussful,
/// otherwise returns [`ContractError`]
/// ## Params
/// * **deps** is an object of type [`DepsMut`]
///
/// * **vesting_accounts** is a [`Vec`] of type [`VestingAccount`]
pub fn register_vesting_accounts(
    deps: DepsMut,
    vesting_accounts: Vec<VestingAccount>,
) -> Result<Response, ContractError> {
    let config = load_config(deps.storage)?;
    for vesting_account in vesting_accounts.iter() {
        validate_vesting_schedules(&vesting_account.schedules)?;

        let vesting_addr = deps.api.addr_validate(&vesting_account.address)?;
        store_vesting_info(
            deps.storage,
            &vesting_addr,
            &VestingInfo {
                last_claim_time: config.genesis_time,
                schedules: vesting_account.schedules.clone(),
            },
        )?;
    }

    Ok(Response::new().add_attribute("action", "register_vesting_accounts"))
}

/// ## Description
/// Validates provided vesting schedules
fn validate_vesting_schedules(vesting_schedules: &[VestingSchedule]) -> StdResult<()> {
    for schedule in vesting_schedules.iter() {
        if schedule.start_time >= schedule.end_time {
            return Err(StdError::generic_err(
                "end_time must be bigger than start_time",
            ));
        }
    }

    Ok(())
}
