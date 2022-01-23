use cosmwasm_std::{to_binary, CosmosMsg, DepsMut, Env, Response, Uint128, WasmMsg};

use crate::{
    error::ContractError,
    state::{load_config, load_state, store_state},
};

use services::{
    bonding::Cw20HookMsg as BondingHookMsg,
    querier::{query_epoch_info, query_rewards_pool_balance},
    rewards::{DistributeRewardMsg, ExecuteMsg as RewardsMsg},
    staking::Cw20HookMsg as StakingHookMsg,
};

pub fn distribute(deps: DepsMut, env: Env) -> Result<Response, ContractError> {
    let config = load_config(deps.storage)?;
    let mut state = load_state(deps.storage)?;

    let current_block = env.block.height;
    let blocks_since_last_distribution = current_block - state.last_distribution_block;

    // query epoch from epoch_manager contract
    let epoch_blocks = query_epoch_info(
        &deps.querier,
        deps.api.addr_humanize(&config.epoch_manager_contract)?,
    )?
    .epoch;

    // distribute rewards only for passed epochs
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
        .add_attributes(vec![("action", "distribute")]))
}
