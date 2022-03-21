use crate::contract::{execute, instantiate, query};
use crate::error::ContractError;
use cosmwasm_std::testing::{mock_env, mock_info};
use cosmwasm_std::{from_binary, to_binary, Attribute, CosmosMsg, SubMsg, Uint128, WasmMsg};
use cw20::Expiration;
use services::ownership_proposal::OwnershipProposalResponse;

use crate::mock_querier::{mock_dependencies, MOCK_EPOCH_MANAGER_ADDR, MOCK_REWARDS_POOL_ADDR};

use services::{
    bonding::Cw20HookMsg as BondingHookMsg,
    distributor::{ConfigResponse, ExecuteMsg, InstantiateMsg, LastDistributionResponse, QueryMsg},
    rewards::{DistributeRewardMsg, ExecuteMsg as RewardsMsg},
    staking::Cw20HookMsg as StakingHookMsg,
};

/// WasmMockQuerier messages:
///
/// epoch-manager contract:
/// mock address: epochmanager
///
/// * **EpochManagerQueryMsg::EpochInfo {}** returns:
/// EpochInfoResponse {
///     epoch: 100,
///     blocks_per_year: 6_307_200,
///     bbro_emission_rate: 1.0,
/// }
///
/// rewards pool contract:
/// mock address: rewards
///
/// * **RewardsPoolQueryMsg::Balance {}** returns:
/// RewardsPoolBalanceResponse {
///     balance: 1000,
/// }

#[test]
fn proper_initialization() {
    let mut deps = mock_dependencies(&[]);
    let env = mock_env();

    let msg = InstantiateMsg {
        owner: "owner".to_string(),
        distribution_genesis_block: 12500,
        epoch_manager_contract: MOCK_EPOCH_MANAGER_ADDR.to_string(),
        rewards_contract: MOCK_REWARDS_POOL_ADDR.to_string(),
        staking_contract: "staking".to_string(),
        staking_distribution_amount: Uint128::from(1u128),
        bonding_contract: "bonding".to_string(),
        bonding_distribution_amount: Uint128::from(2u128),
    };

    let info = mock_info("addr0000", &[]);
    let _res = instantiate(deps.as_mut(), env.clone(), info, msg).unwrap();

    assert_eq!(
        from_binary::<ConfigResponse>(
            &query(deps.as_ref(), mock_env(), QueryMsg::Config {}).unwrap()
        )
        .unwrap(),
        ConfigResponse {
            owner: "owner".to_string(),
            paused: false,
            distribution_genesis_block: 12500,
            epoch_manager_contract: MOCK_EPOCH_MANAGER_ADDR.to_string(),
            rewards_contract: MOCK_REWARDS_POOL_ADDR.to_string(),
            staking_contract: "staking".to_string(),
            staking_distribution_amount: Uint128::from(1u128),
            bonding_contract: "bonding".to_string(),
            bonding_distribution_amount: Uint128::from(2u128),
        },
    );

    assert_eq!(
        from_binary::<LastDistributionResponse>(
            &query(deps.as_ref(), mock_env(), QueryMsg::LastDistribution {}).unwrap()
        )
        .unwrap(),
        LastDistributionResponse {
            last_distribution_block: 12500,
        },
    );
}

#[test]
fn distribute() {
    let mut deps = mock_dependencies(&[]);
    let mut env = mock_env();

    let msg = InstantiateMsg {
        owner: "owner".to_string(),
        distribution_genesis_block: 12500,
        epoch_manager_contract: MOCK_EPOCH_MANAGER_ADDR.to_string(),
        rewards_contract: MOCK_REWARDS_POOL_ADDR.to_string(),
        staking_contract: "staking".to_string(),
        staking_distribution_amount: Uint128::from(500u128),
        bonding_contract: "bonding".to_string(),
        bonding_distribution_amount: Uint128::from(501u128),
    };

    let info = mock_info("addr0000", &[]);
    let _res = instantiate(deps.as_mut(), env.clone(), info, msg).unwrap();

    // error: distribution is not started yet
    env.block.height = 12499;
    let msg = ExecuteMsg::Distribute {};
    let info = mock_info("addr0000", &[]);
    let res = execute(deps.as_mut(), env.clone(), info, msg.clone());
    match res {
        Err(ContractError::DistributionIsNotStartedYet {}) => assert_eq!(true, true),
        _ => panic!("DO NOT ENTER HERE"),
    }

    // error: 0 passed epochs since last distribution
    env.block.height = 12599;
    let msg = ExecuteMsg::Distribute {};
    let info = mock_info("addr0000", &[]);
    let res = execute(deps.as_mut(), env.clone(), info, msg.clone());
    match res {
        Err(ContractError::NoRewards {}) => assert_eq!(true, true),
        _ => panic!("DO NOT ENTER HERE"),
    }

    assert_eq!(
        from_binary::<bool>(
            &query(deps.as_ref(), env.clone(), QueryMsg::IsReadyToTrigger {}).unwrap()
        )
        .unwrap(),
        false,
    );

    // error: not enough balance in rewards pool for distribution
    env.block.height = 12600;
    let info = mock_info("addr0000", &[]);
    let res = execute(deps.as_mut(), env.clone(), info, msg.clone());
    match res {
        Err(ContractError::NotEnoughBalanceForRewards {}) => assert_eq!(true, true),
        _ => panic!("DO NOT ENTER HERE"),
    }

    //proper execution
    // update distribution amounts
    let msg = ExecuteMsg::UpdateConfig {
        paused: None,
        epoch_manager_contract: None,
        rewards_contract: None,
        staking_contract: None,
        staking_distribution_amount: None,
        bonding_contract: None,
        bonding_distribution_amount: Some(Uint128::from(500u128)),
    };

    let info = mock_info("owner", &[]);
    let _res = execute(deps.as_mut(), env.clone(), info, msg.clone()).unwrap();

    // distribute rewards
    assert_eq!(
        from_binary::<bool>(
            &query(deps.as_ref(), env.clone(), QueryMsg::IsReadyToTrigger {}).unwrap()
        )
        .unwrap(),
        true,
    );

    env.block.height = 12630;
    let msg = ExecuteMsg::Distribute {};
    let info = mock_info("addr0000", &[]);
    let res = execute(deps.as_mut(), env.clone(), info, msg.clone()).unwrap();

    let distribution_msg = res.messages.get(0).expect("no message");
    assert_eq!(
        distribution_msg,
        &SubMsg::new(CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: MOCK_REWARDS_POOL_ADDR.to_string(),
            funds: vec![],
            msg: to_binary(&RewardsMsg::DistributeRewards {
                distributions: vec![
                    DistributeRewardMsg {
                        contract: "staking".to_string(),
                        amount: Uint128::from(500u128),
                        msg: to_binary(&StakingHookMsg::DistributeReward {
                            distributed_at_block: 12600,
                        })
                        .unwrap(),
                    },
                    DistributeRewardMsg {
                        contract: "bonding".to_string(),
                        amount: Uint128::from(500u128),
                        msg: to_binary(&BondingHookMsg::DistributeReward {}).unwrap(),
                    }
                ]
            })
            .unwrap(),
        })),
    );

    assert_eq!(
        from_binary::<LastDistributionResponse>(
            &query(deps.as_ref(), mock_env(), QueryMsg::LastDistribution {}).unwrap()
        )
        .unwrap(),
        LastDistributionResponse {
            last_distribution_block: 12600,
        },
    );

    // pause contract
    let msg = ExecuteMsg::UpdateConfig {
        paused: Some(true),
        epoch_manager_contract: None,
        rewards_contract: None,
        staking_contract: None,
        staking_distribution_amount: None,
        bonding_contract: None,
        bonding_distribution_amount: None,
    };

    let info = mock_info("owner", &[]);
    let _res = execute(deps.as_mut(), env.clone(), info, msg.clone()).unwrap();

    // error: contract is paused
    let msg = ExecuteMsg::Distribute {};
    let info = mock_info("addr0000", &[]);
    let res = execute(deps.as_mut(), env.clone(), info, msg.clone());
    match res {
        Err(ContractError::ContractIsPaused {}) => assert_eq!(true, true),
        _ => panic!("DO NOT ENTER HERE"),
    }
}

#[test]
fn update_config() {
    let mut deps = mock_dependencies(&[]);

    let msg = InstantiateMsg {
        owner: "owner".to_string(),
        distribution_genesis_block: 12500,
        epoch_manager_contract: MOCK_EPOCH_MANAGER_ADDR.to_string(),
        rewards_contract: MOCK_REWARDS_POOL_ADDR.to_string(),
        staking_contract: "staking".to_string(),
        staking_distribution_amount: Uint128::from(1u128),
        bonding_contract: "bonding".to_string(),
        bonding_distribution_amount: Uint128::from(2u128),
    };

    let info = mock_info("addr0000", &[]);
    let _res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

    // unauthorized: only owner allowed to execute
    let msg = ExecuteMsg::UpdateConfig {
        paused: Some(true),
        epoch_manager_contract: Some("new_epochmanager".to_string()),
        rewards_contract: Some("new_rewards".to_string()),
        staking_contract: Some("new_staking".to_string()),
        staking_distribution_amount: Some(Uint128::from(100u128)),
        bonding_contract: Some("new_bonding".to_string()),
        bonding_distribution_amount: Some(Uint128::from(200u128)),
    };

    let info = mock_info("addr0000", &[]);
    let res = execute(deps.as_mut(), mock_env(), info, msg.clone());
    match res {
        Err(ContractError::Unauthorized {}) => assert_eq!(true, true),
        _ => panic!("DO NOT ENTER HERE"),
    }

    // proper execution
    let info = mock_info("owner", &[]);
    let res = execute(deps.as_mut(), mock_env(), info, msg.clone()).unwrap();

    assert_eq!(res.attributes[0], Attribute::new("action", "update_config"));
    assert_eq!(res.attributes[1], Attribute::new("paused_changed", "true"));
    assert_eq!(
        res.attributes[2],
        Attribute::new("epoch_manager_contract_changed", "new_epochmanager")
    );
    assert_eq!(
        res.attributes[3],
        Attribute::new("rewards_contract_changed", "new_rewards")
    );
    assert_eq!(
        res.attributes[4],
        Attribute::new("staking_contract_changed", "new_staking")
    );
    assert_eq!(
        res.attributes[5],
        Attribute::new("staking_distribution_amount_changed", "100")
    );
    assert_eq!(
        res.attributes[6],
        Attribute::new("bonding_contract_changed", "new_bonding")
    );
    assert_eq!(
        res.attributes[7],
        Attribute::new("bonding_distribution_amount_changed", "200")
    );

    assert_eq!(
        from_binary::<ConfigResponse>(
            &query(deps.as_ref(), mock_env(), QueryMsg::Config {}).unwrap()
        )
        .unwrap(),
        ConfigResponse {
            owner: "owner".to_string(),
            paused: true,
            distribution_genesis_block: 12500,
            epoch_manager_contract: "new_epochmanager".to_string(),
            rewards_contract: "new_rewards".to_string(),
            staking_contract: "new_staking".to_string(),
            staking_distribution_amount: Uint128::from(100u128),
            bonding_contract: "new_bonding".to_string(),
            bonding_distribution_amount: Uint128::from(200u128),
        },
    );
}

#[test]
fn update_owner() {
    let mut deps = mock_dependencies(&[]);

    let msg = InstantiateMsg {
        owner: "owner0000".to_string(),
        distribution_genesis_block: 12500,
        epoch_manager_contract: MOCK_EPOCH_MANAGER_ADDR.to_string(),
        rewards_contract: MOCK_REWARDS_POOL_ADDR.to_string(),
        staking_contract: "staking".to_string(),
        staking_distribution_amount: Uint128::from(1u128),
        bonding_contract: "bonding".to_string(),
        bonding_distribution_amount: Uint128::from(2u128),
    };

    let info = mock_info("addr0000", &[]);
    let _res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

    // propose ownership
    let msg = ExecuteMsg::ProposeNewOwner {
        new_owner: "owner0001".to_string(),
        expires_in_blocks: 100,
    };

    let info = mock_info("owner0000", &[]);
    let _res = execute(deps.as_mut(), mock_env(), info.clone(), msg.clone()).unwrap();

    assert_eq!(
        from_binary::<OwnershipProposalResponse>(
            &query(deps.as_ref(), mock_env(), QueryMsg::OwnershipProposal {}).unwrap()
        )
        .unwrap(),
        OwnershipProposalResponse {
            proposed_owner: "owner0001".to_string(),
            expires_at: Expiration::AtHeight(12_345 + 100),
        },
    );

    // claim ownership
    let msg = ExecuteMsg::ClaimOwnership {};

    let info = mock_info("owner0001", &[]);
    let _res = execute(deps.as_mut(), mock_env(), info.clone(), msg.clone()).unwrap();

    // verify that owner was changed
    assert_eq!(
        from_binary::<ConfigResponse>(
            &query(deps.as_ref(), mock_env(), QueryMsg::Config {}).unwrap()
        )
        .unwrap(),
        ConfigResponse {
            owner: "owner0001".to_string(),
            paused: false,
            distribution_genesis_block: 12500,
            epoch_manager_contract: MOCK_EPOCH_MANAGER_ADDR.to_string(),
            rewards_contract: MOCK_REWARDS_POOL_ADDR.to_string(),
            staking_contract: "staking".to_string(),
            staking_distribution_amount: Uint128::from(1u128),
            bonding_contract: "bonding".to_string(),
            bonding_distribution_amount: Uint128::from(2u128),
        },
    );
}
