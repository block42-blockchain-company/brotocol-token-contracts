use cosmwasm_std::{
    to_binary, Addr, Attribute, CosmosMsg, Decimal, DepsMut, Env, MessageInfo, Response, Uint128,
    WasmMsg,
};
use cw20::{Cw20ExecuteMsg, Expiration};

use crate::{
    error::ContractError,
    state::{
        load_config, load_state, load_withdrawals, read_staker_info, remove_staker_info,
        store_config, store_staker_info, store_state, store_withdrawals, WithdrawalInfo,
    },
};

use services::{
    bbro_minter::ExecuteMsg as BbroMintMsg, querier::query_epoch_info, staking::StakeType,
};

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

    // because total_stake_amount is zero and we cannot distribute received rewards
    // we send it back to rewards pool
    if state.total_stake_amount.is_zero() {
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
        state.global_reward_index + Decimal::from_ratio(reward_amount, state.total_stake_amount);

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
///
/// * **stake_type** is an object of type [`StakeType`]
pub fn stake(
    deps: DepsMut,
    env: Env,
    sender_addr: Addr,
    amount: Uint128,
    stake_type: StakeType,
) -> Result<Response, ContractError> {
    let sender_raw = deps.api.addr_canonicalize(&sender_addr.to_string())?;

    let config = load_config(deps.storage)?;
    let mut state = load_state(deps.storage)?;
    let mut staker_info = read_staker_info(deps.storage, &sender_raw, env.block.height)?;

    if amount < config.min_staking_amount {
        return Err(ContractError::StakingAmountMustBeHigherThanMinAmount {});
    }

    let epoch_info = query_epoch_info(
        &deps.querier,
        deps.api.addr_humanize(&config.epoch_manager_contract)?,
    )?;

    staker_info.compute_normal_bbro_reward(&epoch_info, &state, env.block.height)?;
    staker_info.compute_bro_reward(&state)?;

    let msgs: Vec<CosmosMsg> = match stake_type {
        StakeType::Unlocked {} => {
            staker_info.unlocked_stake_amount =
                staker_info.unlocked_stake_amount.checked_add(amount)?;

            vec![]
        }
        StakeType::Locked { epochs_locked } => {
            if !config.lockup_config.valid_lockup_period(epochs_locked) {
                return Err(ContractError::InvalidLockupPeriod {});
            }

            let bbro_premium_lockup_reward = staker_info.compute_premium_bbro_reward(
                &config.lockup_config,
                epochs_locked,
                amount,
            );

            staker_info.add_lockup(env.block.height, amount, epochs_locked)?;

            vec![CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: deps
                    .api
                    .addr_humanize(&config.bbro_minter_contract)?
                    .to_string(),
                funds: vec![],
                msg: to_binary(&BbroMintMsg::Mint {
                    recipient: sender_addr.to_string(),
                    amount: bbro_premium_lockup_reward,
                })?,
            })]
        }
    };

    staker_info.unlock_expired_lockups(&env.block, &epoch_info)?;
    store_staker_info(deps.storage, &sender_raw, &staker_info)?;

    // increase total stake amount
    state.total_stake_amount = state.total_stake_amount.checked_add(amount)?;
    store_state(deps.storage, &state)?;

    Ok(Response::new().add_messages(msgs).add_attributes(vec![
        ("action", "stake"),
        ("staker", &sender_addr.to_string()),
        ("amount", &amount.to_string()),
    ]))
}

/// ## Description
/// Locks bonded amount of tokens via community bonding contract
/// to get reward shares.
/// Returns [`Response`] with specified attributes and messages if operation was successful,
/// otherwise returns [`ContractError`]
/// ## Params
/// * **deps** is an object of type [`DepsMut`]
///
/// * **env** is an object of type [`Env`]
///
/// * **sender** is a field of type [`String`]
///
/// * **amount** is an object of type [`Uint128`]
///
/// * **epochs_locked** is a field of type [`u64`]
pub fn community_bond_stake(
    deps: DepsMut,
    env: Env,
    sender: String,
    amount: Uint128,
    epochs_locked: u64,
) -> Result<Response, ContractError> {
    let sender_raw = deps.api.addr_canonicalize(&sender)?;

    let config = load_config(deps.storage)?;
    let mut state = load_state(deps.storage)?;
    let mut staker_info = read_staker_info(deps.storage, &sender_raw, env.block.height)?;

    if amount < config.min_staking_amount {
        return Err(ContractError::StakingAmountMustBeHigherThanMinAmount {});
    }

    let epoch_info = query_epoch_info(
        &deps.querier,
        deps.api.addr_humanize(&config.epoch_manager_contract)?,
    )?;

    staker_info.compute_normal_bbro_reward(&epoch_info, &state, env.block.height)?;
    staker_info.compute_bro_reward(&state)?;

    if !config.lockup_config.valid_lockup_period(epochs_locked) {
        return Err(ContractError::InvalidLockupPeriod {});
    }

    let bbro_premium_lockup_reward =
        staker_info.compute_premium_bbro_reward(&config.lockup_config, epochs_locked, amount);

    staker_info.add_lockup(env.block.height, amount, epochs_locked)?;
    staker_info.unlock_expired_lockups(&env.block, &epoch_info)?;

    store_staker_info(deps.storage, &sender_raw, &staker_info)?;

    // increase total stake amount
    state.total_stake_amount = state.total_stake_amount.checked_add(amount)?;
    store_state(deps.storage, &state)?;

    Ok(Response::new()
        .add_messages(vec![CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: deps
                .api
                .addr_humanize(&config.bbro_minter_contract)?
                .to_string(),
            funds: vec![],
            msg: to_binary(&BbroMintMsg::Mint {
                recipient: sender.clone(),
                amount: bbro_premium_lockup_reward,
            })?,
        })])
        .add_attributes(vec![
            ("action", "community_bond_stake"),
            ("staker", &sender),
            ("amount", &amount.to_string()),
            (
                "bbro_premium_lockup_reward",
                &bbro_premium_lockup_reward.to_string(),
            ),
        ]))
}

/// ## Description
/// Locks a staked amount that is unlocked.
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
///
/// * **epochs_locked** is a field of type [`u64`]
pub fn lockup_staked(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    amount: Uint128,
    epochs_locked: u64,
) -> Result<Response, ContractError> {
    let sender_raw = deps.api.addr_canonicalize(&info.sender.to_string())?;

    let config = load_config(deps.storage)?;
    let mut staker_info = read_staker_info(deps.storage, &sender_raw, env.block.height)?;

    let epoch_info = query_epoch_info(
        &deps.querier,
        deps.api.addr_humanize(&config.epoch_manager_contract)?,
    )?;

    staker_info.unlock_expired_lockups(&env.block, &epoch_info)?;
    if staker_info.unlocked_stake_amount < amount {
        return Err(ContractError::ForbiddenToLockupMoreThanUnlocked {});
    }

    if !config.lockup_config.valid_lockup_period(epochs_locked) {
        return Err(ContractError::InvalidLockupPeriod {});
    }

    let bbro_premium_lockup_reward =
        staker_info.compute_premium_bbro_reward(&config.lockup_config, epochs_locked, amount);

    if bbro_premium_lockup_reward.is_zero() {
        return Err(ContractError::LockupPremiumRewardIsZero {});
    }

    staker_info.add_lockup(env.block.height, amount, epochs_locked)?;
    staker_info.unlocked_stake_amount = staker_info.unlocked_stake_amount.checked_sub(amount)?;
    store_staker_info(deps.storage, &sender_raw, &staker_info)?;

    Ok(Response::new()
        .add_messages(vec![CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: deps
                .api
                .addr_humanize(&config.bbro_minter_contract)?
                .to_string(),
            funds: vec![],
            msg: to_binary(&BbroMintMsg::Mint {
                recipient: info.sender.to_string(),
                amount: bbro_premium_lockup_reward,
            })?,
        })])
        .add_attributes(vec![
            ("action", "lockup_staked"),
            ("sender", &info.sender.to_string()),
            ("lockup_amount", &amount.to_string()),
            (
                "bbro_premium_lockup_reward",
                &bbro_premium_lockup_reward.to_string(),
            ),
        ]))
}

/// ## Description
/// Unstake staked amount of tokens. Tokens will be claimable only after passing the unstaking period.
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
pub fn unstake(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    amount: Uint128,
) -> Result<Response, ContractError> {
    let config = load_config(deps.storage)?;
    let mut state = load_state(deps.storage)?;

    let sender_addr_raw = deps.api.addr_canonicalize(&info.sender.to_string())?;
    let mut staker_info = read_staker_info(deps.storage, &sender_addr_raw, env.block.height)?;

    let epoch_info = query_epoch_info(
        &deps.querier,
        deps.api.addr_humanize(&config.epoch_manager_contract)?,
    )?;

    staker_info.unlock_expired_lockups(&env.block, &epoch_info)?;
    if staker_info.unlocked_stake_amount < amount {
        return Err(ContractError::ForbiddenToUnstakeMoreThanUnlocked {});
    }

    staker_info.compute_normal_bbro_reward(&epoch_info, &state, env.block.height)?;
    staker_info.compute_bro_reward(&state)?;

    // decrease stake amount
    state.total_stake_amount = state.total_stake_amount.checked_sub(amount)?;
    staker_info.unlocked_stake_amount = staker_info.unlocked_stake_amount.checked_sub(amount)?;

    if staker_info.can_be_removed()? {
        remove_staker_info(deps.storage, &sender_addr_raw);
    } else {
        store_staker_info(deps.storage, &sender_addr_raw, &staker_info)?;
    }

    store_state(deps.storage, &state)?;

    // create withdrawal info
    let claimable_at = Expiration::AtHeight(env.block.height + config.unstake_period_blocks);
    let mut staker_withdrawals = load_withdrawals(deps.storage, &sender_addr_raw)?;
    staker_withdrawals.push(WithdrawalInfo {
        amount,
        claimable_at,
    });

    store_withdrawals(deps.storage, &sender_addr_raw, &staker_withdrawals)?;

    Ok(Response::new().add_attributes(vec![
        ("action", "unstake"),
        ("staker", &info.sender.to_string()),
        ("amount", &amount.to_string()),
    ]))
}

/// ## Description
/// Withdraw the amount of tokens that have already passed the unstaking period.
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
    // if withdrawal passed unstaking period add claimable amount and remove it from withdrawals list
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
/// Claim available bro reward amount.
/// Returns [`Response`] with specified attributes and messages if operation was successful,
/// otherwise returns [`ContractError`]
/// ## Params
/// * **deps** is an object of type [`DepsMut`]
///
/// * **env** is an object of type [`Env`]
///
/// * **info** is an object of type [`MessageInfo`]
pub fn claim_bro_rewards(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
) -> Result<Response, ContractError> {
    let config = load_config(deps.storage)?;
    let state = load_state(deps.storage)?;

    let sender_addr_raw = deps.api.addr_canonicalize(&info.sender.to_string())?;
    let mut staker_info = read_staker_info(deps.storage, &sender_addr_raw, env.block.height)?;

    staker_info.compute_bro_reward(&state)?;

    let amount = staker_info.pending_bro_reward;
    if amount == Uint128::zero() {
        return Err(ContractError::NothingToClaim {});
    }

    let epoch_info = query_epoch_info(
        &deps.querier,
        deps.api.addr_humanize(&config.epoch_manager_contract)?,
    )?;

    staker_info.pending_bro_reward = Uint128::zero();
    staker_info.unlock_expired_lockups(&env.block, &epoch_info)?;

    if staker_info.can_be_removed()? {
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
            ("action", "claim_bro_rewards"),
            ("staker", &info.sender.to_string()),
            ("amount", &amount.to_string()),
        ]))
}

/// ## Description
/// Claim available bbro reward amount.
/// Returns [`Response`] with specified attributes and messages if operation was successful,
/// otherwise returns [`ContractError`]
/// ## Params
/// * **deps** is an object of type [`DepsMut`]
///
/// * **env** is an object of type [`Env`]
///
/// * **info** is an object of type [`MessageInfo`]
pub fn claim_bbro_rewards(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
) -> Result<Response, ContractError> {
    let config = load_config(deps.storage)?;
    let state = load_state(deps.storage)?;

    let sender_addr_raw = deps.api.addr_canonicalize(&info.sender.to_string())?;
    let mut staker_info = read_staker_info(deps.storage, &sender_addr_raw, env.block.height)?;

    let epoch_info = query_epoch_info(
        &deps.querier,
        deps.api.addr_humanize(&config.epoch_manager_contract)?,
    )?;

    staker_info.compute_normal_bbro_reward(&epoch_info, &state, env.block.height)?;

    let bbro_reward = staker_info.pending_bbro_reward;
    if bbro_reward.is_zero() {
        return Err(ContractError::NothingToClaim {});
    }

    staker_info.pending_bbro_reward = Uint128::zero();
    staker_info.unlock_expired_lockups(&env.block, &epoch_info)?;
    store_staker_info(deps.storage, &sender_addr_raw, &staker_info)?;

    Ok(Response::new()
        .add_messages(vec![CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: deps
                .api
                .addr_humanize(&config.bbro_minter_contract)?
                .to_string(),
            funds: vec![],
            msg: to_binary(&BbroMintMsg::Mint {
                recipient: info.sender.to_string(),
                amount: bbro_reward,
            })?,
        })])
        .add_attributes(vec![
            ("action", "claim_bbro_rewards"),
            ("staker", &info.sender.to_string()),
            ("bbro_reward", &bbro_reward.to_string()),
        ]))
}

/// ## Description
/// Updates contract settings.
/// Returns [`Response`] with specified attributes and messages if operation was successful,
/// otherwise returns [`ContractError`]
/// ## Params
/// * **deps** is an object of type [`DepsMut`]
///
/// * **paused** is an [`Option`] of type [`bool`]
///
/// * **unstake_period_blocks** is an [`Option`] of type [`u64`]
///
/// * **min_staking_amount** is an [`Option`] of type [`Uint128`]
///
/// * **min_lockup_period_epochs** is an [`Option`] of type [`u64`]
///
/// * **max_lockup_period_epochs** is an [`Option`] of type [`u64`]
///
/// * **base_rate** is an [`Option`] of type [`Decimal`]
///
/// * **linear_growth** is an [`Option`] of type [`Decimal`]
///
/// * **exponential_growth** is an [`Option`] of type [`Decimal`]
///
/// * **community_bonding_contract** is an [`Option`] of type [`String`]
#[allow(clippy::too_many_arguments)]
pub fn update_config(
    deps: DepsMut,
    paused: Option<bool>,
    unstake_period_blocks: Option<u64>,
    min_staking_amount: Option<Uint128>,
    min_lockup_period_epochs: Option<u64>,
    max_lockup_period_epochs: Option<u64>,
    base_rate: Option<Decimal>,
    linear_growth: Option<Decimal>,
    exponential_growth: Option<Decimal>,
    community_bonding_contract: Option<String>,
) -> Result<Response, ContractError> {
    let mut config = load_config(deps.storage)?;

    let mut attributes: Vec<Attribute> = vec![Attribute::new("action", "update_config")];

    if let Some(paused) = paused {
        config.paused = paused;
        attributes.push(Attribute::new("paused_changed", &paused.to_string()));
    }

    if let Some(unstake_period_blocks) = unstake_period_blocks {
        config.unstake_period_blocks = unstake_period_blocks;
        attributes.push(Attribute::new(
            "unstake_period_blocks_changed",
            &unstake_period_blocks.to_string(),
        ));
    }

    if let Some(min_staking_amount) = min_staking_amount {
        config.min_staking_amount = min_staking_amount;
        attributes.push(Attribute::new(
            "min_staking_amount_changed",
            &min_staking_amount.to_string(),
        ));
    }

    if let Some(min_lockup_period_epochs) = min_lockup_period_epochs {
        config.lockup_config.min_lockup_period_epochs = min_lockup_period_epochs;
        attributes.push(Attribute::new(
            "min_lockup_period_epochs_changed",
            &min_lockup_period_epochs.to_string(),
        ));
    }

    if let Some(max_lockup_period_epochs) = max_lockup_period_epochs {
        config.lockup_config.max_lockup_period_epochs = max_lockup_period_epochs;
        attributes.push(Attribute::new(
            "max_lockup_period_epochs_changed",
            &max_lockup_period_epochs.to_string(),
        ));
    }

    if let Some(base_rate) = base_rate {
        config.lockup_config.base_rate = base_rate;
        attributes.push(Attribute::new("base_rate_changed", &base_rate.to_string()));
    }

    if let Some(linear_growth) = linear_growth {
        config.lockup_config.linear_growth = linear_growth;
        attributes.push(Attribute::new(
            "linear_growth_changed",
            &linear_growth.to_string(),
        ));
    }

    if let Some(exponential_growth) = exponential_growth {
        config.lockup_config.exponential_growth = exponential_growth;
        attributes.push(Attribute::new(
            "exponential_growth_changed",
            &exponential_growth.to_string(),
        ));
    }

    if let Some(community_bonding_contract) = community_bonding_contract {
        config.community_bonding_contract =
            Some(deps.api.addr_canonicalize(&community_bonding_contract)?);
        attributes.push(Attribute::new(
            "community_bonding_contract_changed",
            &community_bonding_contract,
        ));
    }

    config.validate()?;
    store_config(deps.storage, &config)?;

    Ok(Response::new().add_attributes(attributes))
}
