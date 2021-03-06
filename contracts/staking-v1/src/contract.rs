#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    from_binary, to_binary, Addr, Api, Binary, Decimal, Deps, DepsMut, Env, MessageInfo, Response,
    StdResult, Storage, Uint128,
};
use cw2::{get_contract_version, set_contract_version};
use cw20::Cw20ReceiveMsg;

use crate::{
    commands,
    error::ContractError,
    migration::{load_config_v100, MigrationMsgV100},
    queries,
    state::{load_config, store_config, store_state, update_owner, Config, LockupConfig, State},
};

use services::{
    ownership_proposal::{
        claim_ownership, drop_ownership_proposal, propose_new_owner, query_ownership_proposal,
    },
    querier::query_epoch_info,
    staking::{Cw20HookMsg, ExecuteMsg, InstantiateMsg, MigrateMsg, QueryMsg},
};

/// Contract name that is used for migration.
const CONTRACT_NAME: &str = "brotocol-staking-v1";
/// Contract version that is used for migration.
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

/// ## Description
/// Creates a new contract with the specified parameters in the [`InstantiateMsg`].
/// Returns the default [`Response`] object if the operation was successful, otherwise returns
/// the [`ContractError`] if the contract was not created.
/// ## Params
/// * **deps** is an object of type [`DepsMut`].
///
/// * **env** is an object of type [`Env`].
///
/// * **_info** is an object of type [`MessageInfo`].
///
/// * **msg** is a message of type [`InstantiateMsg`] which contains the basic settings for creating a contract
#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    let community_bonding_contract = if let Some(addr) = msg.community_bonding_contract {
        Some(deps.api.addr_canonicalize(&addr)?)
    } else {
        None
    };

    let epoch_info = query_epoch_info(
        &deps.querier,
        deps.api.addr_validate(&msg.epoch_manager_contract)?,
    )?;

    let config = Config {
        owner: deps.api.addr_canonicalize(&msg.owner)?,
        paused: false,
        bro_token: deps.api.addr_canonicalize(&msg.bro_token)?,
        rewards_pool_contract: deps.api.addr_canonicalize(&msg.rewards_pool_contract)?,
        bbro_minter_contract: deps.api.addr_canonicalize(&msg.bbro_minter_contract)?,
        epoch_manager_contract: deps.api.addr_canonicalize(&msg.epoch_manager_contract)?,
        community_bonding_contract,
        unstake_period_blocks: msg.unstake_period_blocks,
        min_staking_amount: msg.min_staking_amount,
        lockup_config: LockupConfig {
            min_lockup_period_epochs: msg.min_lockup_period_epochs,
            max_lockup_period_epochs: msg.max_lockup_period_epochs,
            base_rate: msg.base_rate,
            linear_growth: msg.linear_growth,
            exponential_growth: msg.exponential_growth,
        },
        prev_epoch_blocks: epoch_info.epoch,
    };

    config.validate()?;
    store_config(deps.storage, &config)?;

    store_state(
        deps.storage,
        &State {
            global_reward_index: Decimal::zero(),
            total_stake_amount: Uint128::zero(),
            last_distribution_block: env.block.height,
        },
    )?;

    Ok(Response::default())
}

/// ## Description
/// Available execute messages of the contract
/// ## Params
/// * **deps** is an object of type [`Deps`].
///
/// * **env** is an object of type [`Env`].
///
/// * **info** is an object of type [`MessageInfo`].
///
/// * **msg** is an object of type [`ExecuteMsg`].
///
/// ## Messages
///
/// * **ExecuteMsg::Receive(msg)** Receives a message of type [`Cw20ReceiveMsg`]
/// and processes it depending on the received template
///
/// * **ExecuteMsg::LockupStaked {
///         amount,
///         epochs_locked,
///     }** Lockup unlocked staked amount
///
/// * **ExecuteMsg::Unstake { amount }** Unstake staked amount of tokens
///
/// * **ExecuteMsg::Withdraw {}** Withdraw the amount of tokens that have already passed the unstaking period
///
/// * **ExecuteMsg::ClaimBroRewards {}** Claim available bro reward amount
///
/// * **ExecuteMsg::ClaimBbroRewards {}** Claim available bbro reward amount
///
/// * **ExecuteMsg::UpdateConfig {
///         paused,
///         unstake_period_blocks,
///         min_staking_amount,
///         min_lockup_period_epochs,
///         max_lockup_period_epochs,
///         base_rate,
///         linear_growth,
///         exponential_growth,
///         community_bonding_contract,
///     }** Updates contract settings
///
/// * **ExecuteMsg::ProposeNewOwner {
///         new_owner,
///         expires_in_blocks,
///     }** Creates an offer for a new owner
///
/// * **ExecuteMsg::DropOwnershipProposal {}** Removes the existing offer for the new owner
///
/// * **ExecuteMsg::ClaimOwnership {}** Used to claim(approve) new owner proposal, thus changing contract's owner
#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::Receive(msg) => {
            assert_not_paused(deps.storage)?;
            receive_cw20(deps, env, info, msg)
        }
        ExecuteMsg::LockupStaked {
            amount,
            epochs_locked,
        } => {
            assert_not_paused(deps.storage)?;
            commands::lockup_staked(deps, env, info, amount, epochs_locked)
        }
        ExecuteMsg::Unstake { amount } => {
            assert_not_paused(deps.storage)?;
            commands::unstake(deps, env, info, amount)
        }
        ExecuteMsg::Withdraw {} => {
            assert_not_paused(deps.storage)?;
            commands::withdraw(deps, env, info)
        }
        ExecuteMsg::ClaimBroRewards {} => {
            assert_not_paused(deps.storage)?;
            commands::claim_bro_rewards(deps, env, info)
        }
        ExecuteMsg::ClaimBbroRewards {} => {
            assert_not_paused(deps.storage)?;
            commands::claim_bbro_rewards(deps, env, info)
        }
        ExecuteMsg::UpdateConfig {
            paused,
            unstake_period_blocks,
            min_staking_amount,
            min_lockup_period_epochs,
            max_lockup_period_epochs,
            base_rate,
            linear_growth,
            exponential_growth,
            community_bonding_contract,
        } => {
            assert_owner(deps.storage, deps.api, info.sender)?;
            commands::update_config(
                deps,
                paused,
                unstake_period_blocks,
                min_staking_amount,
                min_lockup_period_epochs,
                max_lockup_period_epochs,
                base_rate,
                linear_growth,
                exponential_growth,
                community_bonding_contract,
            )
        }
        ExecuteMsg::UpdateStakerLockups { stakers } => {
            commands::update_staker_lockups(deps, env, stakers)
        }
        ExecuteMsg::ProposeNewOwner {
            new_owner,
            expires_in_blocks,
        } => {
            let config = load_config(deps.storage)?;

            Ok(propose_new_owner(
                deps,
                env,
                info,
                config.owner,
                new_owner,
                expires_in_blocks,
            )?)
        }
        ExecuteMsg::DropOwnershipProposal {} => {
            let config = load_config(deps.storage)?;

            Ok(drop_ownership_proposal(deps, info, config.owner)?)
        }
        ExecuteMsg::ClaimOwnership {} => Ok(claim_ownership(deps, env, info, update_owner)?),
    }
}

/// ## Description
/// Receives a message of type [`Cw20ReceiveMsg`] and processes it depending on the received template.
/// If the template is not found in the received message, then an [`ContractError`] is returned,
/// otherwise returns the [`Response`] with the specified attributes if the operation was successful
/// ## Params
/// * **deps** is an object of type [`DepsMut`].
///
/// * **env** is an object of type [`Env`].
///
/// * **info** is an object of type [`MessageInfo`].
///
/// * **cw20_msg** is an object of type [`Cw20ReceiveMsg`].
pub fn receive_cw20(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    cw20_msg: Cw20ReceiveMsg,
) -> Result<Response, ContractError> {
    let config = load_config(deps.storage)?;
    if info.sender != deps.api.addr_humanize(&config.bro_token)? {
        return Err(ContractError::Unauthorized {});
    }

    match from_binary(&cw20_msg.msg) {
        Ok(Cw20HookMsg::DistributeReward {
            distributed_at_block,
        }) => {
            // only rewards pool allowed to send bro token rewards to staking contract
            if config.rewards_pool_contract != deps.api.addr_canonicalize(&cw20_msg.sender)? {
                return Err(ContractError::Unauthorized {});
            }

            commands::distribute_reward(deps, cw20_msg.amount, distributed_at_block)
        }
        Ok(Cw20HookMsg::Stake { stake_type }) => {
            let cw20_sender = deps.api.addr_validate(&cw20_msg.sender)?;
            commands::stake(deps, env, cw20_sender, cw20_msg.amount, stake_type)
        }
        Ok(Cw20HookMsg::CommunityBondLock {
            sender,
            epochs_locked,
        }) => {
            let community_bonding_contract = match config.community_bonding_contract {
                Some(addr) => addr,
                None => {
                    return Err(ContractError::StakingFromCommunityBondingContractIsNotEnabled {})
                }
            };

            // only community bonding contract allowed to stake bonded bro tokens with locked staking type
            if community_bonding_contract != deps.api.addr_canonicalize(&cw20_msg.sender)? {
                return Err(ContractError::Unauthorized {});
            }

            commands::community_bond_lock(deps, env, sender, cw20_msg.amount, epochs_locked)
        }
        Err(_) => Err(ContractError::InvalidHookData {}),
    }
}

/// ## Description
/// Verifies that message sender is a contract owner.
/// Returns [`Ok`] if address is valid, otherwise returns [`ContractError`]
/// ## Params
/// * **storage** is an object of type [`Storage`]
///
/// * **api** is an object of type [`Api`]
///
/// * **sender** is an object of type [`Addr`]
fn assert_owner(storage: &dyn Storage, api: &dyn Api, sender: Addr) -> Result<(), ContractError> {
    if load_config(storage)?.owner != api.addr_canonicalize(sender.as_str())? {
        return Err(ContractError::Unauthorized {});
    }

    Ok(())
}

/// ## Description
/// Verifies that contract is not paused.
/// Returns [`Ok`] if address is valid, otherwise returns [`ContractError`]
/// ## Params
/// * **storage** is an object of type [`Storage`]
fn assert_not_paused(storage: &dyn Storage) -> Result<(), ContractError> {
    if load_config(storage)?.paused {
        return Err(ContractError::ContractIsPaused {});
    }

    Ok(())
}

/// ## Description
/// Available query messages of the contract
/// ## Params
/// * **deps** is an object of type [`Deps`].
///
/// * **env** is an object of type [`Env`].
///
/// * **msg** is an object of type [`ExecuteMsg`].
///
/// ## Queries
///
/// * **QueryMsg::Config {}** Returns staking contract config
///
/// * **QueryMsg::State {}** Returns staking contract state
///
/// * **QueryMsg::StakerInfo { staker }** Returns staker info by specified address
///
/// * **QueryMsg::Withdrawals { staker }** Returns available withdrawals for staker by specified address
///
/// * **QueryMsg::OwnershipProposal {}** Returns information about created ownership proposal otherwise returns not-found error
#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Config {} => to_binary(&queries::query_config(deps)?),
        QueryMsg::State {} => to_binary(&queries::query_state(deps)?),
        QueryMsg::StakerInfo { staker } => {
            to_binary(&queries::query_staker_info(deps, env, staker)?)
        }
        QueryMsg::Withdrawals { staker } => to_binary(&queries::query_withdrawals(deps, staker)?),
        QueryMsg::StakersWithDeprecatedLockups { skip, limit } => to_binary(
            &queries::query_stakers_with_deprecated_lockups(deps, skip, limit)?,
        ),
        QueryMsg::OwnershipProposal {} => to_binary(&query_ownership_proposal(deps)?),
    }
}

/// ## Description
/// Used for migration of contract. Returns the default object of type [`Response`].
/// ## Params
/// * **deps** is an object of type [`Deps`].
///
/// * **_env** is an object of type [`Env`].
///
/// * **msg** is an object of type [`MigrateMsg`].
#[cfg_attr(not(feature = "library"), entry_point)]
pub fn migrate(deps: DepsMut, _env: Env, msg: MigrateMsg) -> Result<Response, ContractError> {
    let contract_version = get_contract_version(deps.storage)?;

    match contract_version.contract.as_ref() {
        "brotocol-staking-v1" => match contract_version.version.as_ref() {
            "1.0.0" => {
                let msg: MigrationMsgV100 = from_binary(&msg.params)?;
                let config = load_config_v100(deps.storage)?;

                let community_bonding_contract = if let Some(addr) = msg.community_bonding_contract
                {
                    Some(deps.api.addr_canonicalize(&addr)?)
                } else {
                    None
                };

                let new_config = Config {
                    owner: config.owner,
                    paused: config.paused,
                    bro_token: config.bro_token,
                    rewards_pool_contract: config.rewards_pool_contract,
                    bbro_minter_contract: config.bbro_minter_contract,
                    epoch_manager_contract: config.epoch_manager_contract,
                    community_bonding_contract,
                    unstake_period_blocks: config.unstake_period_blocks,
                    min_staking_amount: config.min_staking_amount,
                    lockup_config: config.lockup_config,
                    prev_epoch_blocks: msg.prev_epoch_blocks,
                };

                new_config.validate()?;
                store_config(deps.storage, &new_config)?;
            }
            _ => return Err(ContractError::MigrationError {}),
        },
        _ => return Err(ContractError::MigrationError {}),
    }

    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    Ok(Response::new().add_attributes(vec![
        ("action", "migrate"),
        ("previous_contract_name", &contract_version.contract),
        ("previous_contract_version", &contract_version.version),
        ("new_contract_name", CONTRACT_NAME),
        ("new_contract_version", CONTRACT_VERSION),
    ]))
}
