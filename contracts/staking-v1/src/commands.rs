use cosmwasm_std::{
    to_binary, Addr, CosmosMsg, Decimal, DepsMut, Env, MessageInfo, Response, Uint128, WasmMsg,
};
use cw20::{Cw20ExecuteMsg, Expiration};

use crate::{
    error::ContractError,
    state::{
        load_config, load_state, load_withdrawals, read_staker_info, remove_staker_info,
        store_staker_info, store_state, store_withdrawals, WithdrawalInfo,
    },
};

use services::bbro_minter::ExecuteMsg as BbroMintMsg;

/// ## Description
/// Distributes received reward.
/// Returns [`Response`] with specified attributes and messages if operation was successful,
/// otherwise returns [`ContractError`]
/// ## Params
/// * **deps** is an object of type [`DepsMut`]
///
/// * **reward_amount** is an object of type [`Uint128`]
///
/// * **distributed_at_block** is a field of type [`u64`]
pub fn distribute_reward(
    deps: DepsMut,
    reward_amount: Uint128,
    distributed_at_block: u64,
) -> Result<Response, ContractError> {
    let config = load_config(deps.storage)?;
    let mut state = load_state(deps.storage)?;

    // because total_bond_amount is zero and we cannot distribute received rewards
    // we send it back to rewards pool
    if state.total_bond_amount.is_zero() {
        return Ok(Response::new()
            .add_messages(vec![CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: deps.api.addr_humanize(&config.bro_token)?.to_string(),
                funds: vec![],
                msg: to_binary(&Cw20ExecuteMsg::Transfer {
                    recipient: deps
                        .api
                        .addr_humanize(&config.rewards_pool_contract)?
                        .to_string(),
                    amount: reward_amount,
                })?,
            })])
            .add_attributes(vec![
                ("action", "distribute_reward"),
                ("reward_amount", "0"),
            ]));
    }

    state.last_distribution_block = distributed_at_block;
    state.global_reward_index =
        state.global_reward_index + Decimal::from_ratio(reward_amount, state.total_bond_amount);

    store_state(deps.storage, &state)?;

    Ok(Response::new().add_attributes(vec![
        ("action", "distribute_reward"),
        ("reward_amount", &reward_amount.to_string()),
    ]))
}

/// ## Description
/// Deposits specified amount of tokens to get reward shares.
/// Returns [`Response`] with specified attributes and messages if operation was successful,
/// otherwise returns [`ContractError`]
/// ## Params
/// * **deps** is an object of type [`DepsMut`]
///
/// * **env** is an object of type [`Env`]
///
/// * **sender_addr** is an object of type [`Addr`]
///
/// * **amount** is an object of type [`Uint128`]
pub fn bond(
    deps: DepsMut,
    env: Env,
    sender_addr: Addr,
    amount: Uint128,
) -> Result<Response, ContractError> {
    let sender_raw = deps.api.addr_canonicalize(&sender_addr.to_string())?;

    let config = load_config(deps.storage)?;
    let mut state = load_state(deps.storage)?;
    let mut staker_info = read_staker_info(deps.storage, &sender_raw, env.block.height)?;

    // calculate bbro reward using current bro staked amount
    let bbro_staking_reward = staker_info.compute_staker_bbro_reward(
        &deps.querier,
        deps.api.addr_humanize(&config.epoch_manager_contract)?,
        &state,
    )?;

    staker_info.compute_staker_reward(&state)?;
    state.increase_bond_amount(&mut staker_info, amount, env.block.height);

    store_state(deps.storage, &state)?;
    store_staker_info(deps.storage, &sender_raw, &staker_info)?;

    let mut msgs: Vec<CosmosMsg> = vec![];
    if !bbro_staking_reward.is_zero() {
        msgs.push(CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: deps
                .api
                .addr_humanize(&config.bbro_minter_contract)?
                .to_string(),
            funds: vec![],
            msg: to_binary(&BbroMintMsg::Mint {
                recipient: sender_addr.to_string(),
                amount: bbro_staking_reward,
            })?,
        }))
    }

    Ok(Response::new().add_messages(msgs).add_attributes(vec![
        ("action", "bond"),
        ("staker", &sender_addr.to_string()),
        ("amount", &amount.to_string()),
    ]))
}

/// ## Description
/// Unbond staked amount of tokens. Tokens will be claimable only after passing unbonding period.
/// Returns [`Response`] with specified attributes and messages if operation was successful,
/// otherwise returns [`ContractError`]
/// ## Params
/// * **deps** is an object of type [`DepsMut`]
///
/// * **env** is an object of type [`Env`]
///
/// * **info** is an object of type [`MessageInfo`]
///
/// * **amount** is an object of type [`Uint128`]
pub fn unbond(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    amount: Uint128,
) -> Result<Response, ContractError> {
    let config = load_config(deps.storage)?;
    let mut state = load_state(deps.storage)?;

    let sender_addr_raw = deps.api.addr_canonicalize(&info.sender.to_string())?;
    let mut staker_info = read_staker_info(deps.storage, &sender_addr_raw, env.block.height)?;

    if staker_info.bond_amount < amount {
        return Err(ContractError::ForbiddenToUnbondMoreThanBonded {});
    }

    // calculate bbro reward using current bro staked amount
    let bbro_staking_reward = staker_info.compute_staker_bbro_reward(
        &deps.querier,
        deps.api.addr_humanize(&config.epoch_manager_contract)?,
        &state,
    )?;

    staker_info.compute_staker_reward(&state)?;
    state.decrease_bond_amount(&mut staker_info, amount, env.block.height)?;

    if staker_info.pending_reward.is_zero() && staker_info.bond_amount.is_zero() {
        remove_staker_info(deps.storage, &sender_addr_raw);
    } else {
        store_staker_info(deps.storage, &sender_addr_raw, &staker_info)?;
    }

    store_state(deps.storage, &state)?;

    // create withdrawal info
    let claimable_at = Expiration::AtHeight(env.block.height + config.unbond_period_blocks);
    let mut staker_withdrawals = load_withdrawals(deps.storage, &sender_addr_raw)?;
    staker_withdrawals.push(WithdrawalInfo {
        amount,
        claimable_at,
    });

    store_withdrawals(deps.storage, &sender_addr_raw, &staker_withdrawals)?;

    let mut msgs: Vec<CosmosMsg> = vec![];
    if !bbro_staking_reward.is_zero() {
        msgs.push(CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: deps
                .api
                .addr_humanize(&config.bbro_minter_contract)?
                .to_string(),
            funds: vec![],
            msg: to_binary(&BbroMintMsg::Mint {
                recipient: info.sender.to_string(),
                amount: bbro_staking_reward,
            })?,
        }))
    }

    Ok(Response::new().add_messages(msgs).add_attributes(vec![
        ("action", "unbond"),
        ("staker", &info.sender.to_string()),
        ("amount", &amount.to_string()),
    ]))
}

/// ## Description
/// Withdraw amount of tokens which have already passed unbonding period.
/// Returns [`Response`] with specified attributes and messages if operation was successful,
/// otherwise returns [`ContractError`]
/// ## Params
/// * **deps** is an object of type [`DepsMut`]
///
/// * **env** is an object of type [`Env`]
///
/// * **info** is an object of type [`MessageInfo`]
pub fn withdraw(deps: DepsMut, env: Env, info: MessageInfo) -> Result<Response, ContractError> {
    let config = load_config(deps.storage)?;
    let sender_addr_raw = deps.api.addr_canonicalize(&info.sender.to_string())?;

    let mut amount = Uint128::zero();
    // if withdrawal passed unbonding period add claimable amount and remove it from withdrawals list
    let staker_withdrawals: Vec<WithdrawalInfo> = load_withdrawals(deps.storage, &sender_addr_raw)?
        .into_iter()
        .filter(|c| {
            if c.claimable_at.is_expired(&env.block) {
                amount += c.amount;
                false
            } else {
                true
            }
        })
        .collect();

    if amount.is_zero() {
        return Err(ContractError::NothingToClaim {});
    }

    store_withdrawals(deps.storage, &sender_addr_raw, &staker_withdrawals)?;

    Ok(Response::new()
        .add_messages(vec![CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: deps.api.addr_humanize(&config.bro_token)?.to_string(),
            funds: vec![],
            msg: to_binary(&Cw20ExecuteMsg::Transfer {
                recipient: info.sender.to_string(),
                amount,
            })?,
        })])
        .add_attributes(vec![
            ("action", "withdraw"),
            ("staker", &info.sender.to_string()),
            ("amount", &amount.to_string()),
        ]))
}

/// ## Description
/// Claim availalble reward amount.
/// Returns [`Response`] with specified attributes and messages if operation was successful,
/// otherwise returns [`ContractError`]
/// ## Params
/// * **deps** is an object of type [`DepsMut`]
///
/// * **env** is an object of type [`Env`]
///
/// * **info** is an object of type [`MessageInfo`]
pub fn claim_rewards(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
) -> Result<Response, ContractError> {
    let config = load_config(deps.storage)?;
    let state = load_state(deps.storage)?;

    let sender_addr_raw = deps.api.addr_canonicalize(&info.sender.to_string())?;
    let mut staker_info = read_staker_info(deps.storage, &sender_addr_raw, env.block.height)?;

    staker_info.compute_staker_reward(&state)?;

    let amount = staker_info.pending_reward;

    if amount == Uint128::zero() {
        return Err(ContractError::NothingToClaim {});
    }

    staker_info.pending_reward = Uint128::zero();

    if staker_info.bond_amount.is_zero() {
        remove_staker_info(deps.storage, &sender_addr_raw);
    } else {
        store_staker_info(deps.storage, &sender_addr_raw, &staker_info)?;
    }

    Ok(Response::new()
        .add_messages(vec![CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: deps.api.addr_humanize(&config.bro_token)?.to_string(),
            funds: vec![],
            msg: to_binary(&Cw20ExecuteMsg::Transfer {
                recipient: info.sender.to_string(),
                amount,
            })?,
        })])
        .add_attributes(vec![
            ("action", "claim_rewards"),
            ("staker", &info.sender.to_string()),
            ("amount", &amount.to_string()),
        ]))
}
