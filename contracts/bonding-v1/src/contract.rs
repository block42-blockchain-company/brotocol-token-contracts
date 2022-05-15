#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    from_binary, to_binary, Addr, Api, Binary, Deps, DepsMut, Env, MessageInfo, Response,
    StdResult, Storage, Uint128,
};
use cw2::{get_contract_version, set_contract_version};
use cw20::Cw20ReceiveMsg;

use crate::{
    commands,
    error::ContractError,
    migration::{load_config_v100, load_state_v110, MigrationMsgV100},
    queries,
    state::{load_config, store_config, store_state, update_owner, BondingMode, Config, State},
};

use services::{
    bonding::{Cw20HookMsg, ExecuteMsg, InstantiateMsg, MigrateMsg, QueryMsg},
    ownership_proposal::{
        claim_ownership, drop_ownership_proposal, propose_new_owner, query_ownership_proposal,
    },
};

/// Contract name that is used for migration.
const CONTRACT_NAME: &str = "brotocol-bonding-v1";
/// Contract version that is used for migration.
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

/// ## Description
/// Creates a new contract with the specified parameters in the [`InstantiateMsg`].
/// Returns the default [`Response`] object if the operation was successful, otherwise returns
/// the [`ContractError`] if the contract was not created.
/// ## Params
/// * **deps** is an object of type [`DepsMut`].
///
/// * **_env** is an object of type [`Env`].
///
/// * **_info** is an object of type [`MessageInfo`].
///
/// * **msg** is a message of type [`InstantiateMsg`] which contains the basic settings for creating a contract
#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    let bonding_mode = BondingMode::from_msg(msg.bonding_mode, &deps.querier, deps.api)?;

    let config = Config {
        owner: deps.api.addr_canonicalize(&msg.owner)?,
        bro_token: deps.api.addr_canonicalize(&msg.bro_token)?,
        rewards_pool_contract: deps.api.addr_canonicalize(&msg.rewards_pool_contract)?,
        treasury_contract: deps.api.addr_canonicalize(&msg.treasury_contract)?,
        astroport_factory: deps.api.addr_canonicalize(&msg.astroport_factory)?,
        oracle_contract: deps.api.addr_canonicalize(&msg.oracle_contract)?,
        ust_bonding_discount: msg.ust_bonding_discount,
        min_bro_payout: msg.min_bro_payout,
        bonding_mode,
    };

    config.validate()?;
    store_config(deps.storage, &config)?;

    store_state(deps.storage, &State::default())?;

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
/// * **ExecuteMsg::UstBond {}** Bond bro tokens by providing ust amount
///
/// * **ExecuteMsg::Claim {}** Claim available reward amount
///
/// * **ExecuteMsg::UpdateConfig {
///         rewards_pool_contract,
///         treasury_contract,
///         astroport_factory,
///         oracle_contract,
///         ust_bonding_discount,
///         min_bro_payout,
///     }** Updates contract settings
///
/// * **ExecuteMsg::UpdateBondingModeConfig {
///         ust_bonding_reward_ratio_normal,
///         lp_token_normal,
///         lp_bonding_discount_normal,
///         vesting_period_blocks_normal,
///         staking_contract_community,
///         epochs_locked_community,
///     }** Updates specific settings for bonding mode config
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
        ExecuteMsg::Receive(msg) => receive_cw20(deps, env, info, msg),
        ExecuteMsg::UstBond {} => commands::ust_bond(deps, env, info),
        ExecuteMsg::Claim {} => commands::claim(deps, env, info),
        ExecuteMsg::UpdateConfig {
            rewards_pool_contract,
            treasury_contract,
            astroport_factory,
            oracle_contract,
            ust_bonding_discount,
            min_bro_payout,
        } => {
            assert_owner(deps.storage, deps.api, info.sender)?;
            commands::update_config(
                deps,
                rewards_pool_contract,
                treasury_contract,
                astroport_factory,
                oracle_contract,
                ust_bonding_discount,
                min_bro_payout,
            )
        }
        ExecuteMsg::UpdateBondingModeConfig {
            ust_bonding_reward_ratio_normal,
            lp_token_normal,
            lp_bonding_discount_normal,
            vesting_period_blocks_normal,
            staking_contract_community,
            epochs_locked_community,
        } => {
            assert_owner(deps.storage, deps.api, info.sender)?;
            commands::update_bonding_mode_config(
                deps,
                ust_bonding_reward_ratio_normal,
                lp_token_normal,
                lp_bonding_discount_normal,
                vesting_period_blocks_normal,
                staking_contract_community,
                epochs_locked_community,
            )
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

    match from_binary(&cw20_msg.msg) {
        Ok(Cw20HookMsg::DistributeReward {}) => {
            if info.sender != deps.api.addr_humanize(&config.bro_token)? {
                return Err(ContractError::Unauthorized {});
            }

            // only rewards pool allowed to send bro token rewards to bonding contract
            if config.rewards_pool_contract != deps.api.addr_canonicalize(&cw20_msg.sender)? {
                return Err(ContractError::Unauthorized {});
            }

            commands::distribute_reward(deps, cw20_msg.amount)
        }
        Ok(Cw20HookMsg::LpBond {}) => {
            let (lp_token, lp_bonding_discount, vesting_period_blocks) = match config.bonding_mode {
                BondingMode::Normal {
                    lp_bonding_discount,
                    lp_token,
                    vesting_period_blocks,
                    ..
                } => (lp_token, lp_bonding_discount, vesting_period_blocks),
                BondingMode::Community { .. } => return Err(ContractError::LpBondingDisabled {}),
            };

            if info.sender != deps.api.addr_humanize(&lp_token)? {
                return Err(ContractError::Unauthorized {});
            }

            let sender_raw = deps.api.addr_canonicalize(&cw20_msg.sender)?;
            commands::lp_bond(
                deps,
                env,
                sender_raw,
                cw20_msg.amount,
                lp_token,
                lp_bonding_discount,
                vesting_period_blocks,
            )
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
/// Available query messages of the contract
/// ## Params
/// * **deps** is an object of type [`Deps`].
///
/// * **_env** is an object of type [`Env`].
///
/// * **msg** is an object of type [`ExecuteMsg`].
///
/// ## Queries
///
/// * **QueryMsg::Config {}** Returns bonding contract config
///
/// * **QueryMsg::State {}** Returns bonding contract state
///
/// * **QueryMsg::Claims { address }** Returns available claims for bonder by specified address
///
/// * **QueryMsg::SimulateUstBond { uusd_amount }** Returns simulated bro bond using specified uusd amount
///
/// * **QueryMsg::SimulateLpBond { lp_amount }** Returns simulated bro bond using specified ust/bro lp token amount
///
/// * **QueryMsg::OwnershipProposal {}** Returns information about created ownership proposal otherwise returns not-found error
#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Config {} => to_binary(&queries::query_config(deps)?),
        QueryMsg::State {} => to_binary(&queries::query_state(deps)?),
        QueryMsg::Claims { address } => to_binary(&queries::query_claims(deps, address)?),
        QueryMsg::SimulateUstBond { uusd_amount } => {
            to_binary(&queries::simulate_ust_bond(deps, uusd_amount)?)
        }
        QueryMsg::SimulateLpBond { lp_amount } => {
            to_binary(&queries::simulate_lp_bond(deps, lp_amount)?)
        }
        QueryMsg::OwnershipProposal {} => to_binary(&query_ownership_proposal(deps)?),
    }
}

/// ## Description
/// Used for migration of contract. Returns the default object of type [`Response`].
/// ## Params
/// * **_deps** is an object of type [`Deps`].
///
/// * **_env** is an object of type [`Env`].
///
/// * **_msg** is an object of type [`MigrateMsg`].
#[cfg_attr(not(feature = "library"), entry_point)]
pub fn migrate(deps: DepsMut, _env: Env, msg: MigrateMsg) -> Result<Response, ContractError> {
    let contract_version = get_contract_version(deps.storage)?;

    match contract_version.contract.as_ref() {
        "brotocol-bonding-v1" => match contract_version.version.as_ref() {
            "1.0.0" => {
                let msg: MigrationMsgV100 = from_binary(&msg.params)?;
                let config = load_config_v100(deps.storage)?;

                let bonding_mode =
                    BondingMode::from_msg(msg.bonding_mode, &deps.querier, deps.api)?;

                let new_config = Config {
                    owner: config.owner,
                    bro_token: config.bro_token,
                    rewards_pool_contract: config.rewards_pool_contract,
                    treasury_contract: config.treasury_contract,
                    astroport_factory: config.astroport_factory,
                    oracle_contract: config.oracle_contract,
                    ust_bonding_discount: config.ust_bonding_discount,
                    min_bro_payout: config.min_bro_payout,
                    bonding_mode,
                };

                new_config.validate()?;
                store_config(deps.storage, &new_config)?;

                let state = load_state_v110(deps.storage)?;

                let new_state = State {
                    ust_bonding_balance: state.ust_bonding_balance,
                    lp_bonding_balance: state.lp_bonding_balance,
                    bonded_bro_amount: Uint128::zero(),
                };

                store_state(deps.storage, &new_state)?;
            }
            "1.1.0" => {
                let state = load_state_v110(deps.storage)?;

                let new_state = State {
                    ust_bonding_balance: state.ust_bonding_balance,
                    lp_bonding_balance: state.lp_bonding_balance,
                    bonded_bro_amount: Uint128::zero(),
                };

                store_state(deps.storage, &new_state)?;
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
