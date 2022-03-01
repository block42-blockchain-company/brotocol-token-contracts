use cosmwasm_std::{to_binary, CosmosMsg, DepsMut, Env, Response, Uint128, WasmMsg};

use crate::{
    error::ContractError,
    state::{load_config, load_state, store_config, store_state},
};

use services::{
    bonding::Cw20HookMsg as BondingHookMsg,
    querier::{query_epoch_info, query_rewards_pool_balance},
    rewards::{DistributeRewardMsg, ExecuteMsg as RewardsMsg},
    staking::Cw20HookMsg as StakingHookMsg,
};

/// ## Description
/// Performs token distribution.
/// Returns [`Response`] with specified attributes and messages if operation was successful,
/// otherwise returns [`ContractError`]
/// ## Params
/// * **deps** is an object of type [`DepsMut`]
///
/// * **env** is an object of type [`Env`]
pub fn distribute(deps: DepsMut, env: Env) -> Result<Response, ContractError> {
    let config = load_config(deps.storage)?;
    let mut state = load_state(deps.storage)?;

    if config.distribution_genesis_block > env.block.height {
        return Err(ContractError::DistributionIsNotStartedYet {});
    }

    // query epoch from epoch_manager contract
    let epoch_blocks = query_epoch_info(
        &deps.querier,
        deps.api.addr_humanize(&config.epoch_manager_contract)?,
    )?
    .epoch;

    // distribute rewards only for passed epochs
    let blocks_since_last_distribution = env.block.height - state.last_distribution_block;
    let passed_epochs = blocks_since_last_distribution / epoch_blocks;
    if passed_epochs == 0 {
        return Err(ContractError::NoRewards {});
    }

    let staking_distribution_amount = config
        .staking_distribution_amount
        .checked_mul(Uint128::from(passed_epochs))?;
    let bonding_distribution_amount = config
        .bonding_distribution_amount
        .checked_mul(Uint128::from(passed_epochs))?;

    // check that rewards pool balance is greater than distribution amount
    let rewards_pool_contract = deps.api.addr_humanize(&config.rewards_contract)?;
    let rewards_pool_balance =
        query_rewards_pool_balance(&deps.querier, rewards_pool_contract.clone())?.balance;

    let total_distribution_amount =
        staking_distribution_amount.checked_add(bonding_distribution_amount)?;
    if total_distribution_amount > rewards_pool_balance {
        return Err(ContractError::NotEnoughBalanceForRewards {});
    }

    state.last_distribution_block += epoch_blocks * passed_epochs;
    store_state(deps.storage, &state)?;

    Ok(Response::new()
        .add_messages(vec![CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: rewards_pool_contract.to_string(),
            funds: vec![],
            msg: to_binary(&RewardsMsg::DistributeRewards {
                distributions: vec![
                    DistributeRewardMsg {
                        contract: deps
                            .api
                            .addr_humanize(&config.staking_contract)?
                            .to_string(),
                        amount: staking_distribution_amount,
                        msg: to_binary(&StakingHookMsg::DistributeReward {
                            distributed_at_block: env.block.height,
                        })?,
                    },
                    DistributeRewardMsg {
                        contract: deps
                            .api
                            .addr_humanize(&config.bonding_contract)?
                            .to_string(),
                        amount: bonding_distribution_amount,
                        msg: to_binary(&BondingHookMsg::DistributeReward {})?,
                    },
                ],
            })?,
        })])
        .add_attributes(vec![
            ("action", "distribute"),
            ("passed_epochs", &passed_epochs.to_string()),
            (
                "staking_distribution_amount",
                &staking_distribution_amount.to_string(),
            ),
            (
                "bonding_distribution_amount",
                &bonding_distribution_amount.to_string(),
            ),
        ]))
}

/// ## Description
/// Updates contract settings.
/// Returns [`Response`] with specified attributes and messages if operation was successful,
/// otherwise returns [`ContractError`]
/// ## Params
/// * **deps** is an object of type [`DepsMut`]
///
/// * **owner** is an [`Option`] of type [`String`]. Sets new contract owner address
///
/// * **epoch_manager_contract** is an [`Option`] of type [`String`]. Sets new epoch manager contract address
///
/// * **rewards_contract** is an [`Option`] of type [`String`]. Sets new rewards pool contract address
///
/// * **staking_contract** is an [`Option`] of type [`String`]. Sets new staking contract address
///
/// * **staking_distribution_amount**is an [`Option`] of type [`Uint128`]. Sets new distribution amount for staking contract
///
/// * **bonding_contract** is an [`Option`] of type [`String`]. Sets new bonding contract address
///
/// * **bonding_distribution_amount** is an [`Option`] of type [`Uint128`]. Sets new distribution amount for bonding contract
#[allow(clippy::too_many_arguments)]
pub fn update_config(
    deps: DepsMut,
    owner: Option<String>,
    epoch_manager_contract: Option<String>,
    rewards_contract: Option<String>,
    staking_contract: Option<String>,
    staking_distribution_amount: Option<Uint128>,
    bonding_contract: Option<String>,
    bonding_distribution_amount: Option<Uint128>,
) -> Result<Response, ContractError> {
    let mut config = load_config(deps.storage)?;

    if let Some(owner) = owner {
        config.owner = deps.api.addr_canonicalize(&owner)?;
    }

    if let Some(epoch_manager_contract) = epoch_manager_contract {
        config.epoch_manager_contract = deps.api.addr_canonicalize(&epoch_manager_contract)?;
    }

    if let Some(rewards_contract) = rewards_contract {
        config.rewards_contract = deps.api.addr_canonicalize(&rewards_contract)?;
    }

    if let Some(staking_contract) = staking_contract {
        config.staking_contract = deps.api.addr_canonicalize(&staking_contract)?;
    }

    if let Some(staking_distribution_amount) = staking_distribution_amount {
        config.staking_distribution_amount = staking_distribution_amount;
    }

    if let Some(bonding_contract) = bonding_contract {
        config.bonding_contract = deps.api.addr_canonicalize(&bonding_contract)?;
    }

    if let Some(bonding_distribution_amount) = bonding_distribution_amount {
        config.bonding_distribution_amount = bonding_distribution_amount;
    }

    store_config(deps.storage, &config)?;

    Ok(Response::new().add_attributes(vec![("action", "update_config")]))
}
