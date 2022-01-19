#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult, StdError};

use cw20_base::{
    ContractError,
    msg::{InstantiateMsg, ExecuteMsg, QueryMsg},
    allowances::{
        execute_increase_allowance as cw20_increase_allowance,
        execute_decrease_allowance as cw20_decrease_allowance,
        execute_burn_from as cw20_burn_from,
    },
    contract::{
        instantiate as cw20_instantiate,
        execute_mint as cw20_mint,
        execute_update_marketing as cw20_update_marketing,
        execute_upload_logo as cw20_upload_logo,
        query as cw20_query,
    },
};

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    cw20_instantiate(deps, env, info, msg)
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::IncreaseAllowance {
            spender,
            amount,
            expires,
        } => cw20_increase_allowance(deps, env, info, spender, amount, expires),
        ExecuteMsg::DecreaseAllowance {
            spender,
            amount,
            expires,
        } => cw20_decrease_allowance(deps, env, info, spender, amount, expires),
        ExecuteMsg::Mint {
            recipient,
            amount,
        } => cw20_mint(deps, env, info, recipient, amount),
        ExecuteMsg::BurnFrom {
            owner,
            amount,
        } => cw20_burn_from(deps, env, info, owner, amount),
        ExecuteMsg::UpdateMarketing {
            project,
            description,
            marketing,
        } => cw20_update_marketing(deps, env, info, project, description, marketing),
        ExecuteMsg::UploadLogo(logo) => cw20_upload_logo(deps, env, info, logo),
        _ => Err(StdError::generic_err("not allowed to execute function").into()),
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> StdResult<Binary> {
    cw20_query(deps, env, msg)
}
