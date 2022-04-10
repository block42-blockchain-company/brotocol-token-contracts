use cosmwasm_std::{
    to_binary, Addr, Attribute, CanonicalAddr, Coin, CosmosMsg, Decimal, DepsMut, Env, MessageInfo,
    QuerierWrapper, Response, StdResult, Uint128, WasmMsg,
};
use cw20::{Cw20ExecuteMsg, Expiration};
use std::str::FromStr;

use astroport::{
    asset::{Asset, AssetInfo},
    querier::query_supply,
};

use crate::{
    error::ContractError,
    state::{
        load_claims, load_config, load_state, store_claims, store_config, store_state, BondType,
        BondingMode, ClaimInfo,
    },
};

use services::{
    oracle::ExecuteMsg as OracleExecuteMsg,
    querier::{
        query_is_oracle_ready_to_trigger, query_oracle_price, query_pools, query_staking_config,
    },
    staking::Cw20HookMsg as StakingHookMsg,
};

/// ## Description
/// Distributes received reward.
/// Returns [`Response`] with specified attributes and messages if operation was successful,
/// otherwise returns [`ContractError`]
/// ## Params
/// * **deps** is an object of type [`DepsMut`]
///
/// * **amount** is an object of type [`Uint128`]
pub fn distribute_reward(deps: DepsMut, amount: Uint128) -> Result<Response, ContractError> {
    let config = load_config(deps.storage)?;
    let mut state = load_state(deps.storage)?;

    let (ust_bond_amount, lp_bond_amount) = match config.bonding_mode {
        BondingMode::Normal {
            ust_bonding_reward_ratio,
            ..
        } => {
            let ust_bond_amount = amount * ust_bonding_reward_ratio;
            let lp_bond_amount = amount.checked_sub(ust_bond_amount)?;
            (ust_bond_amount, lp_bond_amount)
        }
        BondingMode::Community { .. } => (amount, Uint128::zero()),
    };

    state.ust_bonding_balance = state.ust_bonding_balance.checked_add(ust_bond_amount)?;
    state.lp_bonding_balance = state.lp_bonding_balance.checked_add(lp_bond_amount)?;

    store_state(deps.storage, &state)?;

    Ok(Response::new().add_attributes(vec![
        ("action", "distribute_reward"),
        ("ust_bond_amount", &ust_bond_amount.to_string()),
        ("lp_bond_amount", &lp_bond_amount.to_string()),
    ]))
}

/// ## Description
/// Bond bro tokens by providing lp token amount.
/// Returns [`Response`] with specified attributes and messages if operation was successful,
/// otherwise returns [`ContractError`]
/// ## Params
/// * **deps** is an object of type [`DepsMut`]
///
/// * **env** is an object of type [`Env`]
///
/// * **sender_raw** is an object of type [`CanonicalAddr`]
///
/// * **lp_amount** is an object of type [`Uint128`]
///
/// * **lp_token** is an object of type [`CanonicalAddr`]
///
/// * **lp_bonding_discount** is an object of type [`Decimal`]
///
/// * **vesting_period_blocks** is a field of type [`u64`]
pub fn lp_bond(
    deps: DepsMut,
    env: Env,
    sender_raw: CanonicalAddr,
    lp_amount: Uint128,
    lp_token: CanonicalAddr,
    lp_bonding_discount: Decimal,
    vesting_period_blocks: u64,
) -> Result<Response, ContractError> {
    let config = load_config(deps.storage)?;
    let mut state = load_state(deps.storage)?;

    let (bro_pool, ust_pool) = query_bro_ust_pair(
        &deps.querier,
        deps.api.addr_humanize(&config.astroport_factory)?,
        deps.api.addr_humanize(&config.bro_token)?,
    )?;

    let (bro_share, ust_share) = get_share_in_assets(
        &deps.querier,
        &bro_pool,
        &ust_pool,
        lp_amount,
        deps.api.addr_humanize(&lp_token)?,
    )?;

    let oracle_contract = deps.api.addr_humanize(&config.oracle_contract)?;
    let bro_amount = bro_share.checked_add(
        query_oracle_price(
            &deps.querier,
            oracle_contract.clone(),
            AssetInfo::NativeToken {
                denom: "uusd".to_string(),
            },
            ust_share,
        )?
        .amount,
    )?;

    let bro_payout = apply_discount(lp_bonding_discount, bro_amount)?;
    if bro_payout < config.min_bro_payout {
        return Err(ContractError::BondPayoutIsTooLow {});
    }

    if bro_payout > state.lp_bonding_balance {
        return Err(ContractError::NotEnoughForBondPayout {});
    }

    state.lp_bonding_balance = state.lp_bonding_balance.checked_sub(bro_payout)?;
    store_state(deps.storage, &state)?;

    let mut claims = load_claims(deps.storage, &sender_raw)?;
    claims.push(ClaimInfo {
        bond_type: BondType::LpBond,
        amount: bro_payout,
        claimable_at: Expiration::AtHeight(env.block.height + vesting_period_blocks),
    });

    store_claims(deps.storage, &sender_raw, &claims)?;

    let mut msgs: Vec<CosmosMsg> = vec![CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: deps.api.addr_humanize(&lp_token)?.to_string(),
        funds: vec![],
        msg: to_binary(&Cw20ExecuteMsg::Transfer {
            recipient: deps
                .api
                .addr_humanize(&config.treasury_contract)?
                .to_string(),
            amount: lp_amount,
        })?,
    })];

    let oracle_can_be_updated =
        query_is_oracle_ready_to_trigger(&deps.querier, oracle_contract.clone())?;
    if oracle_can_be_updated {
        msgs.push(CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: oracle_contract.to_string(),
            funds: vec![],
            msg: to_binary(&OracleExecuteMsg::UpdatePrice {})?,
        }))
    }

    Ok(Response::new().add_messages(msgs).add_attributes(vec![
        ("action", "lp_bond"),
        ("sender", &deps.api.addr_humanize(&sender_raw)?.to_string()),
        ("lp_amount", &lp_amount.to_string()),
        ("bro_payout", &bro_payout.to_string()),
    ]))
}

/// ## Description
/// Bond bro tokens by providing ust amount.
/// Returns [`Response`] with specified attributes and messages if operation was successful,
/// otherwise returns [`ContractError`]
/// ## Params
/// * **deps** is an object of type [`DepsMut`]
///
/// * **env** is an object of type [`Env`]
///
/// * **info** is an object of type [`MessageInfo`]
pub fn ust_bond(deps: DepsMut, env: Env, info: MessageInfo) -> Result<Response, ContractError> {
    let config = load_config(deps.storage)?;
    let mut state = load_state(deps.storage)?;

    let bond_asset = extract_native_token(&info.funds)?;

    let oracle_contract = deps.api.addr_humanize(&config.oracle_contract)?;
    let bro_amount = query_oracle_price(
        &deps.querier,
        oracle_contract.clone(),
        bond_asset.info.clone(),
        bond_asset.amount,
    )?
    .amount;

    let bro_payout = apply_discount(config.ust_bonding_discount, bro_amount)?;
    if bro_payout < config.min_bro_payout {
        return Err(ContractError::BondPayoutIsTooLow {});
    }

    if bro_payout > state.ust_bonding_balance {
        return Err(ContractError::NotEnoughForBondPayout {});
    }

    state.ust_bonding_balance = state.ust_bonding_balance.checked_sub(bro_payout)?;
    store_state(deps.storage, &state)?;

    let mut msgs: Vec<CosmosMsg> = vec![bond_asset.into_msg(
        &deps.querier,
        deps.api.addr_humanize(&config.treasury_contract)?,
    )?];

    let oracle_can_be_updated =
        query_is_oracle_ready_to_trigger(&deps.querier, oracle_contract.clone())?;
    if oracle_can_be_updated {
        msgs.push(CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: oracle_contract.to_string(),
            funds: vec![],
            msg: to_binary(&OracleExecuteMsg::UpdatePrice {})?,
        }))
    }

    match config.bonding_mode {
        BondingMode::Normal {
            vesting_period_blocks,
            ..
        } => {
            let sender_raw = deps.api.addr_canonicalize(&info.sender.to_string())?;
            let mut claims = load_claims(deps.storage, &sender_raw)?;
            claims.push(ClaimInfo {
                bond_type: BondType::UstBond,
                amount: bro_payout,
                claimable_at: Expiration::AtHeight(env.block.height + vesting_period_blocks),
            });

            store_claims(deps.storage, &sender_raw, &claims)?;
        }
        BondingMode::Community {
            staking_contract,
            epochs_locked,
        } => msgs.push(CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: deps.api.addr_humanize(&config.bro_token)?.to_string(),
            funds: vec![],
            msg: to_binary(&Cw20ExecuteMsg::Send {
                contract: deps.api.addr_humanize(&staking_contract)?.to_string(),
                amount: bro_payout,
                msg: to_binary(&StakingHookMsg::CommunityBondStake {
                    sender: info.sender.to_string(),
                    epochs_locked,
                })?,
            })?,
        })),
    };

    Ok(Response::new().add_messages(msgs).add_attributes(vec![
        ("action", "ust_bond"),
        ("sender", &info.sender.to_string()),
        ("bro_payout", &bro_payout.to_string()),
    ]))
}

/// ## Description
/// Claim available reward amount.
/// Returns [`Response`] with specified attributes and messages if operation was successful,
/// otherwise returns [`ContractError`]
/// ## Params
/// * **deps** is an object of type [`DepsMut`]
///
/// * **env** is an object of type [`Env`]
///
/// * **info** is an object of type [`MessageInfo`]
pub fn claim(deps: DepsMut, env: Env, info: MessageInfo) -> Result<Response, ContractError> {
    let config = load_config(deps.storage)?;

    let sender_raw = deps.api.addr_canonicalize(&info.sender.to_string())?;

    let mut amount = Uint128::zero();
    // if claim passed vesting period add claimable amount and remove it from claims list
    let claims: Vec<ClaimInfo> = load_claims(deps.storage, &sender_raw)?
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

    store_claims(deps.storage, &sender_raw, &claims)?;

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
            ("action", "claim"),
            ("sender", &info.sender.to_string()),
            ("amount", &amount.to_string()),
        ]))
}

/// ## Description
/// Updates contract settings.
/// Returns [`Response`] with specified attributes and messages if operation was successful,
/// otherwise returns [`ContractError`]
/// ## Params
/// * **deps** is an object of type [`DepsMut`]
///
/// * **rewards_pool_contract** is an [`Option`] of type [`String`]
///
/// * **treasury_contract** is an [`Option`] of type [`String`]
///
/// * **astroport_factory** is an [`Option`] of type [`String`]
///
/// * **oracle_contract** is an [`Option`] of type [`String`]
///
/// * **ust_bonding_discount** is an [`Option`] of type [`Decimal`]
///
/// * **min_bro_payout** is an [`Option`] of type [`Uint128`]
#[allow(clippy::too_many_arguments)]
pub fn update_config(
    deps: DepsMut,
    rewards_pool_contract: Option<String>,
    treasury_contract: Option<String>,
    astroport_factory: Option<String>,
    oracle_contract: Option<String>,
    ust_bonding_discount: Option<Decimal>,
    min_bro_payout: Option<Uint128>,
) -> Result<Response, ContractError> {
    let mut config = load_config(deps.storage)?;

    let mut attributes: Vec<Attribute> = vec![Attribute::new("action", "update_config")];

    if let Some(rewards_pool_contract) = rewards_pool_contract {
        config.rewards_pool_contract = deps.api.addr_canonicalize(&rewards_pool_contract)?;
        attributes.push(Attribute::new(
            "rewards_pool_contract_changed",
            &rewards_pool_contract,
        ));
    }

    if let Some(treasury_contract) = treasury_contract {
        config.treasury_contract = deps.api.addr_canonicalize(&treasury_contract)?;
        attributes.push(Attribute::new(
            "treasury_contract_changed",
            &treasury_contract,
        ));
    }

    if let Some(astroport_factory) = astroport_factory {
        config.astroport_factory = deps.api.addr_canonicalize(&astroport_factory)?;
        attributes.push(Attribute::new(
            "astroport_factory_changed",
            &astroport_factory,
        ));
    }

    if let Some(oracle_contract) = oracle_contract {
        config.oracle_contract = deps.api.addr_canonicalize(&oracle_contract)?;
        attributes.push(Attribute::new("oracle_contract_changed", &oracle_contract));
    }

    if let Some(ust_bonding_discount) = ust_bonding_discount {
        config.ust_bonding_discount = ust_bonding_discount;
        attributes.push(Attribute::new(
            "ust_bonding_discount_changed",
            &ust_bonding_discount.to_string(),
        ));
    }

    if let Some(min_bro_payout) = min_bro_payout {
        config.min_bro_payout = min_bro_payout;
        attributes.push(Attribute::new(
            "min_bro_payout_changed",
            &min_bro_payout.to_string(),
        ));
    }

    config.validate()?;
    store_config(deps.storage, &config)?;

    Ok(Response::new().add_attributes(attributes))
}

/// ## Description
/// Updates specific settings for bonding mode config.
/// Returns [`Response`] with specified attributes and messages if operation was successful,
/// otherwise returns [`ContractError`]
/// ## Params
/// * **deps** is an object of type [`DepsMut`]
///
/// * **ust_bonding_reward_ratio_normal** is an [`Option`] of type [`Decimal`]
///
/// * **lp_token_normal** is an [`Option`] of type [`String`]
///
/// * **lp_bonding_discount_normal** is an [`Option`] of type [`Decimal`]
///
/// * **vesting_period_blocks_normal** is an [`Option`] of type [`u64`]
///
/// * **staking_contract_community** is an [`Option`] of type [`String`]
///
/// * **epochs_locked_community** is an [`Option`] of type [`u64`]
#[allow(clippy::too_many_arguments)]
pub fn update_bonding_mode_config(
    deps: DepsMut,
    ust_bonding_reward_ratio_normal: Option<Decimal>,
    lp_token_normal: Option<String>,
    lp_bonding_discount_normal: Option<Decimal>,
    vesting_period_blocks_normal: Option<u64>,
    staking_contract_community: Option<String>,
    epochs_locked_community: Option<u64>,
) -> Result<Response, ContractError> {
    let mut config = load_config(deps.storage)?;

    let mut attributes: Vec<Attribute> =
        vec![Attribute::new("action", "update_bonding_mode_config")];

    match config.bonding_mode {
        BondingMode::Normal {
            mut ust_bonding_reward_ratio,
            mut lp_token,
            mut lp_bonding_discount,
            mut vesting_period_blocks,
        } => {
            if let Some(ust_bonding_reward_ratio_normal) = ust_bonding_reward_ratio_normal {
                ust_bonding_reward_ratio = ust_bonding_reward_ratio_normal;
                attributes.push(Attribute::new(
                    "ust_bonding_reward_ratio_changed",
                    &ust_bonding_reward_ratio_normal.to_string(),
                ));
            }

            if let Some(lp_token_normal) = lp_token_normal {
                lp_token = deps.api.addr_canonicalize(&lp_token_normal)?;
                attributes.push(Attribute::new("lp_token_changed", &lp_token_normal));
            }

            if let Some(lp_bonding_discount_normal) = lp_bonding_discount_normal {
                lp_bonding_discount = lp_bonding_discount_normal;
                attributes.push(Attribute::new(
                    "lp_bonding_discount_changed",
                    &lp_bonding_discount_normal.to_string(),
                ));
            }

            if let Some(vesting_period_blocks_normal) = vesting_period_blocks_normal {
                vesting_period_blocks = vesting_period_blocks_normal;
                attributes.push(Attribute::new(
                    "vesting_period_blocks_changed",
                    &vesting_period_blocks_normal.to_string(),
                ));
            }

            config.bonding_mode = BondingMode::Normal {
                ust_bonding_reward_ratio,
                lp_token,
                lp_bonding_discount,
                vesting_period_blocks,
            };
        }
        BondingMode::Community {
            mut staking_contract,
            mut epochs_locked,
        } => {
            if let Some(staking_contract_community) = staking_contract_community {
                staking_contract = deps.api.addr_canonicalize(&staking_contract_community)?;
                attributes.push(Attribute::new(
                    "staking_contract_changed",
                    &staking_contract_community,
                ));
            }

            if let Some(epochs_locked_community) = epochs_locked_community {
                let staking_config = query_staking_config(
                    &deps.querier,
                    deps.api.addr_humanize(&staking_contract)?,
                )?;

                if epochs_locked_community < staking_config.lockup_config.min_lockup_period_epochs
                    || epochs_locked_community
                        > staking_config.lockup_config.max_lockup_period_epochs
                {
                    return Err(ContractError::InvalidLockupPeriodForCommunityBondingMode {});
                }

                epochs_locked = epochs_locked_community;
                attributes.push(Attribute::new(
                    "epochs_locked_changed",
                    &epochs_locked_community.to_string(),
                ));
            }

            config.bonding_mode = BondingMode::Community {
                staking_contract,
                epochs_locked,
            };
        }
    };

    config.validate()?;
    store_config(deps.storage, &config)?;

    Ok(Response::new().add_attributes(attributes))
}

/// ## Description
/// Extracts ust amount from provided info.funds input.
/// Otherwise returns [`ContractError`]
/// ## Params
/// * **funds** is an object of type [`&[Coin]`]
fn extract_native_token(funds: &[Coin]) -> Result<Asset, ContractError> {
    if funds.len() != 1 || funds[0].denom != "uusd" || funds[0].amount.is_zero() {
        return Err(ContractError::InvalidFundsInput {});
    }

    Ok(Asset {
        info: AssetInfo::NativeToken {
            denom: "uusd".to_string(),
        },
        amount: funds[0].amount,
    })
}

/// ## Description
/// Returns the share of assets in the [`Uint128`] object
/// ## Params
/// * **querier** is an object of type [`QuerierWrapper`]
///
/// * **bro_pool** is an object of type [`Asset`]
///
/// * **ust_pool** is an object of type [`Asset`]
///
/// * **lp_amount** is an object of type [`Uint128`]
///
/// * **lp_token_addr** is an object of type [`Addr`]
fn get_share_in_assets(
    querier: &QuerierWrapper,
    bro_pool: &Asset,
    ust_pool: &Asset,
    lp_amount: Uint128,
    lp_token_addr: Addr,
) -> StdResult<(Uint128, Uint128)> {
    let total_share = query_supply(querier, lp_token_addr)?;
    let share_ratio = Decimal::from_ratio(lp_amount, total_share);

    Ok((bro_pool.amount * share_ratio, ust_pool.amount * share_ratio))
}

/// ## Description
/// Queries bro/ust pair using astroport factory.
/// result.0 - bro asset info of type [`Asset`]
/// result.1 - ust asset info of type [`Asset`]
/// ## Params
/// * **querier** is an object of type [`QuerierWrapper`]
///
/// * **astro_factory** is an object of type [`Addr`]
///
/// * **bro_token** is an object of type [`Addr`]
fn query_bro_ust_pair(
    querier: &QuerierWrapper,
    astro_factory: Addr,
    bro_token: Addr,
) -> StdResult<(Asset, Asset)> {
    let asset_info = [
        AssetInfo::NativeToken {
            denom: "uusd".to_string(),
        },
        AssetInfo::Token {
            contract_addr: bro_token,
        },
    ];

    let pools = query_pools(querier, astro_factory, &asset_info)?;
    match &pools[0].info {
        AssetInfo::Token { .. } => Ok((pools[0].clone(), pools[1].clone())),
        AssetInfo::NativeToken { .. } => Ok((pools[1].clone(), pools[0].clone())),
    }
}

/// ## Description
/// Applies bonding discount for provided token amount
/// and returns result in the [`Uint128`] object
/// ## Params
/// * **discount_ratio** is an object of type [`Decimal`]
///
/// * **amount** is an object of type [`Uint128`]
fn apply_discount(discount_ratio: Decimal, amount: Uint128) -> StdResult<Uint128> {
    let discount = Decimal::from_str("1.0")? + discount_ratio;
    let payout = amount * discount;
    Ok(payout)
}
