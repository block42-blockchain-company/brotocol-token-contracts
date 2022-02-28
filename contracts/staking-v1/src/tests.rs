use crate::contract::{execute, instantiate, query};
use crate::error::ContractError;
use crate::math::{decimal_mul_in_256, decimal_sub_in_256, decimal_sum_in_256};
use crate::mock_querier::mock_dependencies;
use services::bbro_minter::ExecuteMsg as BbroMintMsg;
use services::staking::{
    ConfigResponse, Cw20HookMsg, ExecuteMsg, InstantiateMsg, LockupConfigResponse,
    LockupInfoResponse, QueryMsg, StakeType, StakerAccruedRewardsResponse, StakerInfoResponse,
    StateResponse, WithdrawalInfoResponse, WithdrawalsResponse,
};

use cosmwasm_std::testing::{mock_env, mock_info};
use cosmwasm_std::{from_binary, to_binary, CosmosMsg, Decimal, SubMsg, Uint128, WasmMsg};
use cw20::{Cw20ExecuteMsg, Cw20ReceiveMsg, Expiration};

use std::str::FromStr;

#[test]
fn proper_initialization() {
    let mut deps = mock_dependencies(&[]);

    let msg = InstantiateMsg {
        owner: "owner".to_string(),
        bro_token: "bro0000".to_string(),
        rewards_pool_contract: "reward0000".to_string(),
        bbro_minter_contract: "bbrominter0000".to_string(),
        epoch_manager_contract: "epoch0000".to_string(),
        unstake_period_blocks: 10,
        min_staking_amount: Uint128::zero(),
        min_lockup_period_epochs: 1,
        max_lockup_period_epochs: 365,
        base_rate: Decimal::from_str("0.0001").unwrap(),
        linear_growth: Decimal::from_str("0.0005").unwrap(),
        exponential_growth: Decimal::from_str("0.0000075").unwrap(),
    };

    let info = mock_info("addr0000", &[]);
    let _res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

    assert_eq!(
        from_binary::<ConfigResponse>(
            &query(deps.as_ref(), mock_env(), QueryMsg::Config {}).unwrap()
        )
        .unwrap(),
        ConfigResponse {
            owner: "owner".to_string(),
            bro_token: "bro0000".to_string(),
            rewards_pool_contract: "reward0000".to_string(),
            bbro_minter_contract: "bbrominter0000".to_string(),
            epoch_manager_contract: "epoch0000".to_string(),
            unstake_period_blocks: 10,
            min_staking_amount: Uint128::zero(),
            lockup_config: LockupConfigResponse {
                min_lockup_period_epochs: 1,
                max_lockup_period_epochs: 365,
                base_rate: Decimal::from_str("0.0001").unwrap(),
                linear_growth: Decimal::from_str("0.0005").unwrap(),
                exponential_growth: Decimal::from_str("0.0000075").unwrap(),
            }
        }
    );

    assert_eq!(
        from_binary::<StateResponse>(
            &query(deps.as_ref(), mock_env(), QueryMsg::State {}).unwrap()
        )
        .unwrap(),
        StateResponse {
            total_stake_amount: Uint128::zero(),
            global_reward_index: Decimal::zero(),
            last_distribution_block: 12345,
        }
    );
}

#[test]
fn test_decimal_math() {
    // test_decimal_multiplication
    let a = Uint128::from(100u128);
    let b = Decimal::from_ratio(Uint128::from(1111111u128), Uint128::from(10000000u128));
    let multiplication = decimal_mul_in_256(Decimal::from_ratio(a, Uint128::from(1u128)), b);
    assert_eq!(multiplication.to_string(), "11.11111");

    // test_decimal_sumation
    let a = Decimal::from_ratio(Uint128::from(20u128), Uint128::from(50u128));
    let b = Decimal::from_ratio(Uint128::from(10u128), Uint128::from(50u128));
    let res = decimal_sum_in_256(a, b);
    assert_eq!(res.to_string(), "0.6");

    // test_decimal_subtraction
    let a = Decimal::from_ratio(Uint128::from(20u128), Uint128::from(50u128));
    let b = Decimal::from_ratio(Uint128::from(10u128), Uint128::from(50u128));
    let res = decimal_sub_in_256(a, b);
    assert_eq!(res.to_string(), "0.2");

    // test_decimal_multiplication_in_256
    let a = Uint128::from(100u128);
    let b = Decimal::from_ratio(Uint128::from(1111111u128), Uint128::from(10000000u128));
    let multiplication = decimal_mul_in_256(Decimal::from_ratio(a, Uint128::from(1u128)), b);
    assert_eq!(multiplication.to_string(), "11.11111");

    // test_decimal_sumation_in_256
    let a = Decimal::from_ratio(Uint128::from(20u128), Uint128::from(50u128));
    let b = Decimal::from_ratio(Uint128::from(10u128), Uint128::from(50u128));
    let res = decimal_sum_in_256(a, b);
    assert_eq!(res.to_string(), "0.6");

    // test_decimal_subtraction_in_256
    let a = Decimal::from_ratio(Uint128::from(20u128), Uint128::from(50u128));
    let b = Decimal::from_ratio(Uint128::from(10u128), Uint128::from(50u128));
    let res = decimal_sub_in_256(a, b);
    assert_eq!(res.to_string(), "0.2");
}

#[test]
fn test_fractional_rewards() {
    let mut deps = mock_dependencies(&[]);

    ////////////////////////////////////////////////////////////////////////////
    /////// instantiate the contract
    ////////////////////////////////////////////////////////////////////////////

    let msg = InstantiateMsg {
        owner: "owner".to_string(),
        bro_token: "bro0000".to_string(),
        rewards_pool_contract: "reward0000".to_string(),
        bbro_minter_contract: "bbrominter0000".to_string(),
        epoch_manager_contract: "epoch0000".to_string(),
        unstake_period_blocks: 10,
        min_staking_amount: Uint128::zero(),
        min_lockup_period_epochs: 1,
        max_lockup_period_epochs: 365,
        base_rate: Decimal::from_str("0.0001").unwrap(),
        linear_growth: Decimal::from_str("0.0005").unwrap(),
        exponential_growth: Decimal::from_str("0.0000075").unwrap(),
    };

    let info = mock_info("addr0000", &[]);
    let _res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();
    let mut env = mock_env();

    ////////////////////////////////////////////////////////////////////////////
    /////// addr0000 stakes 100 tokens for 3 addresses, but keep the reward pool at 0
    ////////////////////////////////////////////////////////////////////////////

    let info = mock_info("bro0000", &[]);
    env.block.height += 1;

    let msg = ExecuteMsg::Receive(Cw20ReceiveMsg {
        sender: "addr0000".to_string(),
        amount: Uint128::from(100u128),
        msg: to_binary(&Cw20HookMsg::Stake {
            stake_type: StakeType::Unlocked {},
        })
        .unwrap(),
    });
    let _res = execute(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();

    let info = mock_info("bro0000", &[]);
    env.block.height += 1;

    let msg = ExecuteMsg::Receive(Cw20ReceiveMsg {
        sender: "addr0001".to_string(),
        amount: Uint128::from(100u128),
        msg: to_binary(&Cw20HookMsg::Stake {
            stake_type: StakeType::Unlocked {},
        })
        .unwrap(),
    });
    let _res = execute(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();

    let info = mock_info("bro0000", &[]);
    env.block.height += 1;

    let msg = ExecuteMsg::Receive(Cw20ReceiveMsg {
        sender: "addr0002".to_string(),
        amount: Uint128::from(100u128),
        msg: to_binary(&Cw20HookMsg::Stake {
            stake_type: StakeType::Unlocked {},
        })
        .unwrap(),
    });
    let _res = execute(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();

    ////////////////////////////////////////////////////////////////////////////
    /////// distribute 1000 reward, but using the sending address
    //////  every user should receive 333
    ////////////////////////////////////////////////////////////////////////////

    let info = mock_info("bro0000", &[]);

    let msg = ExecuteMsg::Receive(Cw20ReceiveMsg {
        sender: "reward0000".to_string(),
        amount: Uint128::from(1000u128),
        msg: to_binary(&Cw20HookMsg::DistributeReward {
            distributed_at_block: 12349,
        })
        .unwrap(),
    });

    let _res = execute(deps.as_mut(), env.clone(), info, msg).unwrap();

    // checking pending rewards through StakerInfoResponse
    assert_eq!(
        from_binary::<StakerInfoResponse>(
            &query(
                deps.as_ref(),
                mock_env(),
                QueryMsg::StakerInfo {
                    staker: "addr0000".to_string()
                },
            )
            .unwrap(),
        )
        .unwrap(),
        StakerInfoResponse {
            staker: "addr0000".to_string(),
            reward_index: Decimal::from_ratio(10u128, 3u128),
            unlocked_stake_amount: Uint128::from(100u128),
            locked_stake_amount: Uint128::zero(),
            pending_reward: Uint128::from(333u128),
            last_balance_update: 12346,
            lockups: vec![],
        }
    );

    // checking pending rewards through StakerInfoResponse
    assert_eq!(
        from_binary::<StakerInfoResponse>(
            &query(
                deps.as_ref(),
                mock_env(),
                QueryMsg::StakerInfo {
                    staker: "addr0001".to_string()
                },
            )
            .unwrap(),
        )
        .unwrap(),
        StakerInfoResponse {
            staker: "addr0001".to_string(),
            reward_index: Decimal::from_ratio(10u128, 3u128),
            unlocked_stake_amount: Uint128::from(100u128),
            locked_stake_amount: Uint128::zero(),
            pending_reward: Uint128::from(333u128),
            last_balance_update: 12347,
            lockups: vec![],
        }
    );

    // checking pending rewards through StakerInfoResponse
    assert_eq!(
        from_binary::<StakerInfoResponse>(
            &query(
                deps.as_ref(),
                mock_env(),
                QueryMsg::StakerInfo {
                    staker: "addr0002".to_string()
                },
            )
            .unwrap(),
        )
        .unwrap(),
        StakerInfoResponse {
            staker: "addr0002".to_string(),
            reward_index: Decimal::from_ratio(10u128, 3u128),
            unlocked_stake_amount: Uint128::from(100u128),
            locked_stake_amount: Uint128::zero(),
            pending_reward: Uint128::from(333u128),
            last_balance_update: 12348,
            lockups: vec![],
        }
    );

    assert_eq!(
        from_binary::<StateResponse>(
            &query(deps.as_ref(), mock_env(), QueryMsg::State {}).unwrap()
        )
        .unwrap(),
        StateResponse {
            total_stake_amount: Uint128::from(300u128),
            global_reward_index: Decimal::from_ratio(10u128, 3u128),
            last_distribution_block: 12349,
        }
    );

    ////////////////////////////////////////////////////////////////////////////
    /////// distribute 2000 reward, but using the sending address
    //////  every user should receive 667 (it is currently 666)
    ////////////////////////////////////////////////////////////////////////////

    let info = mock_info("bro0000", &[]);
    env.block.height += 1;

    let msg = ExecuteMsg::Receive(Cw20ReceiveMsg {
        sender: "reward0000".to_string(),
        amount: Uint128::from(2000u128),
        msg: to_binary(&Cw20HookMsg::DistributeReward {
            distributed_at_block: 12350,
        })
        .unwrap(),
    });

    let _res = execute(deps.as_mut(), env.clone(), info, msg).unwrap();

    // checking pending rewards through StakerInfoResponse
    assert_eq!(
        from_binary::<StakerInfoResponse>(
            &query(
                deps.as_ref(),
                mock_env(),
                QueryMsg::StakerInfo {
                    staker: "addr0000".to_string()
                },
            )
            .unwrap(),
        )
        .unwrap(),
        StakerInfoResponse {
            staker: "addr0000".to_string(),
            reward_index: Decimal::from_ratio(10u128, 1u128)
                - Decimal::from_ratio(1u128, 1000000000000000000u128),
            unlocked_stake_amount: Uint128::from(100u128),
            locked_stake_amount: Uint128::zero(),
            pending_reward: Uint128::from(999u128),
            last_balance_update: 12346,
            lockups: vec![],
        }
    );
}

#[test]
fn test_unlocked_stake_tokens() {
    let mut deps = mock_dependencies(&[]);

    ////////////////////////////////////////////////////////////////////////////
    /////// instantiate the contract
    ////////////////////////////////////////////////////////////////////////////

    let msg = InstantiateMsg {
        owner: "owner".to_string(),
        bro_token: "bro0000".to_string(),
        rewards_pool_contract: "reward0000".to_string(),
        bbro_minter_contract: "bbrominter0000".to_string(),
        epoch_manager_contract: "epoch0000".to_string(),
        unstake_period_blocks: 10,
        min_staking_amount: Uint128::from(1u128),
        min_lockup_period_epochs: 1,
        max_lockup_period_epochs: 365,
        base_rate: Decimal::from_str("0.0001").unwrap(),
        linear_growth: Decimal::from_str("0.0005").unwrap(),
        exponential_growth: Decimal::from_str("0.0000075").unwrap(),
    };

    let info = mock_info("addr0000", &[]);
    let _res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();
    let mut env = mock_env();

    ////////////////////////////////////////////////////////////////////////////
    /////// calling distribute reward when total staking is 0
    ////////////////////////////////////////////////////////////////////////////

    let info = mock_info("bro0000", &[]);
    let msg = ExecuteMsg::Receive(Cw20ReceiveMsg {
        sender: "reward0000".to_string(),
        amount: Uint128::from(1000u128),
        msg: to_binary(&Cw20HookMsg::DistributeReward {
            distributed_at_block: 12348,
        })
        .unwrap(),
    });

    let _res = execute(deps.as_mut(), env.clone(), info, msg).unwrap();

    // checking pending rewards through StakerInfoResponse
    assert_eq!(
        from_binary::<StakerInfoResponse>(
            &query(
                deps.as_ref(),
                mock_env(),
                QueryMsg::StakerInfo {
                    staker: "addr0000".to_string()
                },
            )
            .unwrap(),
        )
        .unwrap(),
        StakerInfoResponse {
            staker: "addr0000".to_string(),
            reward_index: Decimal::zero(),
            unlocked_stake_amount: Uint128::zero(),
            locked_stake_amount: Uint128::zero(),
            pending_reward: Uint128::zero(),
            last_balance_update: 12345,
            lockups: vec![],
        }
    );

    assert_eq!(
        from_binary::<StateResponse>(
            &query(deps.as_ref(), mock_env(), QueryMsg::State {}).unwrap()
        )
        .unwrap(),
        StateResponse {
            total_stake_amount: Uint128::zero(),
            global_reward_index: Decimal::zero(),
            last_distribution_block: 12345,
        }
    );

    ////////////////////////////////////////////////////////////////////////////
    /////// addr0000 is trying to stake 0 BRO tokens
    ////////////////////////////////////////////////////////////////////////////
    let info = mock_info("bro0000", &[]);
    env.block.height += 1;

    let msg = ExecuteMsg::Receive(Cw20ReceiveMsg {
        sender: "addr0000".to_string(),
        amount: Uint128::from(0u128),
        msg: to_binary(&Cw20HookMsg::Stake {
            stake_type: StakeType::Unlocked {},
        })
        .unwrap(),
    });

    match execute(deps.as_mut(), env.clone(), info, msg) {
        Err(ContractError::StakingAmountMustBeHigherThanMinAmount {}) => (),
        _ => panic!("expecting ContractError::StakingAmountMustBeHigherThanMinAmount"),
    }

    ////////////////////////////////////////////////////////////////////////////
    /////// addr0000 stakes 100 tokens, but keep the reward pool at 0
    ////////////////////////////////////////////////////////////////////////////

    let info = mock_info("bro0000", &[]);

    let msg = ExecuteMsg::Receive(Cw20ReceiveMsg {
        sender: "addr0000".to_string(),
        amount: Uint128::from(100u128),
        msg: to_binary(&Cw20HookMsg::Stake {
            stake_type: StakeType::Unlocked {},
        })
        .unwrap(),
    });
    let res = execute(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();

    assert_eq!(res.messages.len(), 0);

    assert_eq!(
        from_binary::<StakerInfoResponse>(
            &query(
                deps.as_ref(),
                mock_env(),
                QueryMsg::StakerInfo {
                    staker: "addr0000".to_string()
                },
            )
            .unwrap(),
        )
        .unwrap(),
        StakerInfoResponse {
            staker: "addr0000".to_string(),
            reward_index: Decimal::zero(),
            unlocked_stake_amount: Uint128::from(100u128),
            locked_stake_amount: Uint128::zero(),
            pending_reward: Uint128::zero(),
            last_balance_update: env.block.height,
            lockups: vec![],
        }
    );

    assert_eq!(
        from_binary::<StateResponse>(
            &query(deps.as_ref(), mock_env(), QueryMsg::State {}).unwrap()
        )
        .unwrap(),
        StateResponse {
            total_stake_amount: Uint128::from(100u128),
            global_reward_index: Decimal::zero(),
            last_distribution_block: 12345,
        }
    );

    ////////////////////////////////////////////////////////////////////////////
    /////// addr0001 stakes 100 tokens, but keep the reward pool at 0
    ////////////////////////////////////////////////////////////////////////////

    let msg = ExecuteMsg::Receive(Cw20ReceiveMsg {
        sender: "addr0001".to_string(),
        amount: Uint128::from(100u128),
        msg: to_binary(&Cw20HookMsg::Stake {
            stake_type: StakeType::Unlocked {},
        })
        .unwrap(),
    });
    env.block.height += 1;

    let res = execute(deps.as_mut(), env.clone(), info, msg).unwrap();

    assert_eq!(res.messages.len(), 0);

    assert_eq!(
        from_binary::<StakerInfoResponse>(
            &query(
                deps.as_ref(),
                mock_env(),
                QueryMsg::StakerInfo {
                    staker: "addr0001".to_string()
                },
            )
            .unwrap(),
        )
        .unwrap(),
        StakerInfoResponse {
            staker: "addr0001".to_string(),
            reward_index: Decimal::zero(),
            unlocked_stake_amount: Uint128::from(100u128),
            locked_stake_amount: Uint128::zero(),
            pending_reward: Uint128::zero(),
            last_balance_update: env.block.height,
            lockups: vec![],
        }
    );

    assert_eq!(
        from_binary::<StateResponse>(
            &query(deps.as_ref(), mock_env(), QueryMsg::State {}).unwrap()
        )
        .unwrap(),
        StateResponse {
            total_stake_amount: Uint128::from(200u128),
            global_reward_index: Decimal::zero(),
            last_distribution_block: 12345,
        }
    );

    ////////////////////////////////////////////////////////////////////////////
    /////// distribute 1000 reward, but using the wrong address
    ////////////////////////////////////////////////////////////////////////////

    let info = mock_info("attacker0000", &[]);

    let msg = ExecuteMsg::Receive(Cw20ReceiveMsg {
        sender: "reward0000".to_string(),
        amount: Uint128::from(1000u128),
        msg: to_binary(&Cw20HookMsg::DistributeReward {
            distributed_at_block: 12348,
        })
        .unwrap(),
    });
    env.block.height += 1;

    match execute(deps.as_mut(), env.clone(), info, msg) {
        Err(ContractError::Unauthorized {}) => (),
        _ => panic!("expecting ContractError::Unauthorized"),
    }

    ////////////////////////////////////////////////////////////////////////////
    /////// distribute 1000 reward
    ////////////////////////////////////////////////////////////////////////////

    let info = mock_info("bro0000", &[]);

    let msg = ExecuteMsg::Receive(Cw20ReceiveMsg {
        sender: "reward0000".to_string(),
        amount: Uint128::from(1000u128),
        msg: to_binary(&Cw20HookMsg::DistributeReward {
            distributed_at_block: 12348,
        })
        .unwrap(),
    });

    let _res = execute(deps.as_mut(), env.clone(), info, msg).unwrap();

    // checking pending rewards through StakerInfoResponse
    assert_eq!(
        from_binary::<StakerInfoResponse>(
            &query(
                deps.as_ref(),
                mock_env(),
                QueryMsg::StakerInfo {
                    staker: "addr0000".to_string()
                },
            )
            .unwrap(),
        )
        .unwrap(),
        StakerInfoResponse {
            staker: "addr0000".to_string(),
            reward_index: Decimal::from_ratio(5u128, 1u128),
            unlocked_stake_amount: Uint128::from(100u128),
            locked_stake_amount: Uint128::zero(),
            pending_reward: Uint128::from(500u128),
            last_balance_update: 12346,
            lockups: vec![],
        }
    );

    assert_eq!(
        from_binary::<StateResponse>(
            &query(deps.as_ref(), mock_env(), QueryMsg::State {}).unwrap()
        )
        .unwrap(),
        StateResponse {
            total_stake_amount: Uint128::from(200u128),
            global_reward_index: Decimal::from_ratio(5u128, 1u128),
            last_distribution_block: 12348,
        }
    );

    // checking pending rewards through StakerAccruedRewardsResponse
    assert_eq!(
        from_binary::<StakerAccruedRewardsResponse>(
            &query(
                deps.as_ref(),
                mock_env(),
                QueryMsg::StakerAccruedRewards {
                    staker: "addr0000".to_string()
                }
            )
            .unwrap()
        )
        .unwrap(),
        StakerAccruedRewardsResponse {
            rewards: Uint128::new(500),
            bbro_stake_reward: Uint128::new(60),
        }
    );

    ////////////////////////////////////////////////////////////////////////////
    /////// addr0000 claims reward
    ////////////////////////////////////////////////////////////////////////////

    let info = mock_info("addr0000", &[]);

    let msg = ExecuteMsg::ClaimStakingRewards {};
    env.block.height += 1;

    let res = execute(deps.as_mut(), env.clone(), info, msg).unwrap();
    assert_eq!(res.attributes.len(), 3);
    assert_eq!(res.attributes[0].key, "action");
    assert_eq!(res.attributes[0].value, "claim_bro_rewards");
    assert_eq!(res.attributes[1].key, "staker");
    assert_eq!(res.attributes[1].value, "addr0000");
    assert_eq!(res.attributes[2].key, "amount");
    assert_eq!(
        Decimal::from_str(&res.attributes[2].value).unwrap(),
        Decimal::from_ratio(500u128, 1u128)
    );

    assert_eq!(res.messages.len(), 1);
    if let SubMsg {
        msg: CosmosMsg::Wasm(WasmMsg::Execute { msg: wasm_msg, .. }),
        ..
    } = &res.messages[0]
    {
        if let Cw20ExecuteMsg::Transfer { recipient, amount } = from_binary(wasm_msg).unwrap() {
            assert_eq!(recipient, "addr0000".to_string());
            assert_eq!(amount, Uint128::new(500));
        } else {
            assert!(false);
        }
    } else {
        assert!(false);
    }

    assert_eq!(
        from_binary::<StakerInfoResponse>(
            &query(
                deps.as_ref(),
                mock_env(),
                QueryMsg::StakerInfo {
                    staker: "addr0000".to_string()
                },
            )
            .unwrap(),
        )
        .unwrap(),
        StakerInfoResponse {
            staker: "addr0000".to_string(),
            reward_index: Decimal::from_ratio(5u128, 1u128),
            unlocked_stake_amount: Uint128::from(100u128),
            locked_stake_amount: Uint128::zero(),
            pending_reward: Uint128::zero(),
            last_balance_update: 12346,
            lockups: vec![],
        }
    );

    assert_eq!(
        from_binary::<StateResponse>(
            &query(deps.as_ref(), mock_env(), QueryMsg::State {}).unwrap()
        )
        .unwrap(),
        StateResponse {
            total_stake_amount: Uint128::from(200u128),
            global_reward_index: Decimal::from_ratio(5u128, 1u128),
            last_distribution_block: 12348,
        }
    );

    ////////////////////////////////////////////////////////////////////////////
    /////// addr0000 tries to claim reward twice
    ////////////////////////////////////////////////////////////////////////////

    let info = mock_info("addr0000", &[]);

    let msg = ExecuteMsg::ClaimStakingRewards {};

    match execute(deps.as_mut(), env.clone(), info, msg) {
        Err(ContractError::NothingToClaim {}) => (),
        _ => panic!("expecting ContractError::NothingToClaim error"),
    }

    ////////////////////////////////////////////////////////////////////////////
    /////// addr0000 tries to unstake 150 while only staked 100
    ////////////////////////////////////////////////////////////////////////////

    let info = mock_info("addr0000", &[]);

    env.block.height += 1;

    let msg = ExecuteMsg::Unstake {
        amount: Uint128::new(150),
    };

    match execute(deps.as_mut(), env.clone(), info, msg) {
        Err(ContractError::ForbiddenToUnstakeMoreThanUnlocked {}) => (),
        _ => panic!("expecting failure due to unstaking too much"),
    }

    ////////////////////////////////////////////////////////////////////////////
    /////// addr0000 unstakes 50
    ////////////////////////////////////////////////////////////////////////////

    let info = mock_info("addr0000", &[]);

    let msg = ExecuteMsg::Unstake {
        amount: Uint128::new(50),
    };
    let res = execute(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();

    assert_eq!(res.messages.len(), 1);
    if let SubMsg {
        msg: CosmosMsg::Wasm(WasmMsg::Execute { msg: wasm_msg, .. }),
        ..
    } = &res.messages[0]
    {
        if let BbroMintMsg::Mint { recipient, amount } = from_binary(wasm_msg).unwrap() {
            assert_eq!(recipient, "addr0000".to_string());
            assert_eq!(amount, Uint128::new(60));
        } else {
            assert!(false);
        }
    } else {
        assert!(false);
    }

    assert_eq!(res.attributes.len(), 3);
    assert_eq!(res.attributes[0].key, "action");
    assert_eq!(res.attributes[0].value, "unstake");
    assert_eq!(res.attributes[1].key, "staker");
    assert_eq!(res.attributes[1].value, "addr0000");
    assert_eq!(res.attributes[2].key, "amount");
    assert_eq!(
        Decimal::from_str(&res.attributes[2].value).unwrap(),
        Decimal::from_ratio(50u128, 1u128)
    );

    assert_eq!(
        from_binary::<StakerInfoResponse>(
            &query(
                deps.as_ref(),
                mock_env(),
                QueryMsg::StakerInfo {
                    staker: "addr0000".to_string()
                },
            )
            .unwrap(),
        )
        .unwrap(),
        StakerInfoResponse {
            staker: "addr0000".to_string(),
            reward_index: Decimal::from_ratio(5u128, 1u128),
            unlocked_stake_amount: Uint128::from(50u128),
            locked_stake_amount: Uint128::zero(),
            pending_reward: Uint128::zero(),
            last_balance_update: 12350,
            lockups: vec![],
        }
    );

    assert_eq!(
        from_binary::<StateResponse>(
            &query(deps.as_ref(), mock_env(), QueryMsg::State {}).unwrap()
        )
        .unwrap(),
        StateResponse {
            total_stake_amount: Uint128::from(150u128),
            global_reward_index: Decimal::from_ratio(5u128, 1u128),
            last_distribution_block: 12348,
        }
    );

    // checking withdrawal
    assert_eq!(
        from_binary::<WithdrawalsResponse>(
            &query(
                deps.as_ref(),
                mock_env(),
                QueryMsg::Withdrawals {
                    staker: "addr0000".to_string()
                }
            )
            .unwrap()
        )
        .unwrap(),
        WithdrawalsResponse {
            claims: vec![WithdrawalInfoResponse {
                amount: Uint128::new(50),
                claimable_at: Expiration::AtHeight(12360)
            }]
        }
    );

    ////////////////////////////////////////////////////////////////////////////
    /////// distributing an other 1000, addr0001 should receive twice the reward
    //////  of addr0000
    ////////////////////////////////////////////////////////////////////////////

    let info = mock_info("bro0000", &[]);

    let msg = ExecuteMsg::Receive(Cw20ReceiveMsg {
        sender: "reward0000".to_string(),
        amount: Uint128::from(1000u128),
        msg: to_binary(&Cw20HookMsg::DistributeReward {
            distributed_at_block: 12348,
        })
        .unwrap(),
    });

    let _res = execute(deps.as_mut(), env.clone(), info, msg).unwrap();

    assert_eq!(
        from_binary::<StakerInfoResponse>(
            &query(
                deps.as_ref(),
                mock_env(),
                QueryMsg::StakerInfo {
                    staker: "addr0000".to_string()
                },
            )
            .unwrap(),
        )
        .unwrap(),
        StakerInfoResponse {
            staker: "addr0000".to_string(),
            reward_index: Decimal::from_ratio(5u128, 1u128) + Decimal::from_ratio(100u128, 15u128),
            unlocked_stake_amount: Uint128::from(50u128),
            locked_stake_amount: Uint128::zero(),
            pending_reward: Uint128::from(333u128),
            last_balance_update: 12350,
            lockups: vec![],
        }
    );

    assert_eq!(
        from_binary::<StakerInfoResponse>(
            &query(
                deps.as_ref(),
                mock_env(),
                QueryMsg::StakerInfo {
                    staker: "addr0001".to_string()
                },
            )
            .unwrap(),
        )
        .unwrap(),
        StakerInfoResponse {
            staker: "addr0001".to_string(),
            reward_index: Decimal::from_ratio(5u128, 1u128) + Decimal::from_ratio(100u128, 15u128),
            unlocked_stake_amount: Uint128::from(100u128),
            locked_stake_amount: Uint128::zero(),
            pending_reward: Uint128::from(1166u128),
            last_balance_update: 12347,
            lockups: vec![],
        }
    );

    ////////////////////////////////////////////////////////////////////////////
    /////// addr0000 withdaws 50 BRO too early
    ////////////////////////////////////////////////////////////////////////////

    let info = mock_info("addr0000", &[]);

    let msg = ExecuteMsg::Withdraw {};
    env.block.height += 1;

    match execute(deps.as_mut(), env.clone(), info, msg) {
        Err(ContractError::NothingToClaim {}) => (),
        _ => panic!("expecting nothing to claim"),
    }

    ////////////////////////////////////////////////////////////////////////////
    /////// addr0000 withdaws 50 BRO successfully
    ////////////////////////////////////////////////////////////////////////////

    let info = mock_info("addr0000", &[]);

    let msg = ExecuteMsg::Withdraw {};
    env.block.height += 10;

    let res = execute(deps.as_mut(), env.clone(), info, msg).unwrap();

    assert_eq!(res.messages.len(), 1);
    if let SubMsg {
        msg: CosmosMsg::Wasm(WasmMsg::Execute { msg: wasm_msg, .. }),
        ..
    } = &res.messages[0]
    {
        if let Cw20ExecuteMsg::Transfer { recipient, amount } = from_binary(wasm_msg).unwrap() {
            assert_eq!(recipient, "addr0000".to_string());
            assert_eq!(amount, Uint128::new(50));
        } else {
            assert!(false);
        }
    } else {
        assert!(false);
    }

    assert_eq!(res.attributes.len(), 3);
    assert_eq!(res.attributes[0].key, "action");
    assert_eq!(res.attributes[0].value, "withdraw");
    assert_eq!(res.attributes[1].key, "staker");
    assert_eq!(res.attributes[1].value, "addr0000");
    assert_eq!(res.attributes[2].key, "amount");
    assert_eq!(
        Decimal::from_str(&res.attributes[2].value).unwrap(),
        Decimal::from_ratio(50u128, 1u128)
    );

    ////////////////////////////////////////////////////////////////////////////
    /////// addr000 tries to withdraw twice
    ////////////////////////////////////////////////////////////////////////////

    let info = mock_info("addr0000", &[]);

    let msg = ExecuteMsg::Withdraw {};
    env.block.height += 1;

    match execute(deps.as_mut(), env.clone(), info, msg) {
        Err(ContractError::NothingToClaim {}) => (),
        _ => panic!("expecting nothing to claim"),
    }

    ////////////////////////////////////////////////////////////////////////////
    /////// addr0001 unstakes all his 100
    ////////////////////////////////////////////////////////////////////////////

    let info = mock_info("addr0001", &[]);

    let msg = ExecuteMsg::Unstake {
        amount: Uint128::new(100),
    };
    let res = execute(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();

    assert_eq!(res.messages.len(), 1);
    if let SubMsg {
        msg: CosmosMsg::Wasm(WasmMsg::Execute { msg: wasm_msg, .. }),
        ..
    } = &res.messages[0]
    {
        if let BbroMintMsg::Mint { recipient, amount } = from_binary(wasm_msg).unwrap() {
            assert_eq!(recipient, "addr0001".to_string());
            assert_eq!(amount, Uint128::new(30));
        } else {
            assert!(false);
        }
    } else {
        assert!(false);
    }

    assert_eq!(res.attributes.len(), 3);
    assert_eq!(res.attributes[0].key, "action");
    assert_eq!(res.attributes[0].value, "unstake");
    assert_eq!(res.attributes[1].key, "staker");
    assert_eq!(res.attributes[1].value, "addr0001");
    assert_eq!(res.attributes[2].key, "amount");
    assert_eq!(
        Decimal::from_str(&res.attributes[2].value).unwrap(),
        Decimal::from_ratio(100u128, 1u128)
    );

    assert_eq!(
        from_binary::<StakerInfoResponse>(
            &query(
                deps.as_ref(),
                mock_env(),
                QueryMsg::StakerInfo {
                    staker: "addr0001".to_string()
                },
            )
            .unwrap(),
        )
        .unwrap(),
        StakerInfoResponse {
            staker: "addr0001".to_string(),
            reward_index: Decimal::from_ratio(5u128, 1u128) + Decimal::from_ratio(100u128, 15u128),
            unlocked_stake_amount: Uint128::zero(),
            locked_stake_amount: Uint128::zero(),
            pending_reward: Uint128::from(1166u128),
            last_balance_update: 12362,
            lockups: vec![],
        }
    );

    assert_eq!(
        from_binary::<StateResponse>(
            &query(deps.as_ref(), mock_env(), QueryMsg::State {}).unwrap()
        )
        .unwrap(),
        StateResponse {
            total_stake_amount: Uint128::from(50u128),
            global_reward_index: Decimal::from_ratio(5u128, 1u128)
                + Decimal::from_ratio(100u128, 15u128),
            last_distribution_block: 12348,
        }
    );

    // checking withdrawal
    assert_eq!(
        from_binary::<WithdrawalsResponse>(
            &query(
                deps.as_ref(),
                mock_env(),
                QueryMsg::Withdrawals {
                    staker: "addr0001".to_string()
                }
            )
            .unwrap()
        )
        .unwrap(),
        WithdrawalsResponse {
            claims: vec![WithdrawalInfoResponse {
                amount: Uint128::new(100),
                claimable_at: Expiration::AtHeight(12372)
            }]
        }
    );

    ////////////////////////////////////////////////////////////////////////////
    /////// addr0001 claims reward after it unstaked everything
    ////////////////////////////////////////////////////////////////////////////

    let info = mock_info("addr0001", &[]);

    let msg = ExecuteMsg::ClaimStakingRewards {};
    env.block.height += 1;

    let res = execute(deps.as_mut(), env.clone(), info, msg).unwrap();
    assert_eq!(res.attributes.len(), 3);
    assert_eq!(res.attributes[0].key, "action");
    assert_eq!(res.attributes[0].value, "claim_bro_rewards");
    assert_eq!(res.attributes[1].key, "staker");
    assert_eq!(res.attributes[1].value, "addr0001");
    assert_eq!(res.attributes[2].key, "amount");
    assert_eq!(
        Decimal::from_str(&res.attributes[2].value).unwrap(),
        Decimal::from_ratio(1166u128, 1u128)
    );

    assert_eq!(
        from_binary::<StakerInfoResponse>(
            &query(
                deps.as_ref(),
                env.clone(),
                QueryMsg::StakerInfo {
                    staker: "addr0001".to_string()
                },
            )
            .unwrap(),
        )
        .unwrap(),
        StakerInfoResponse {
            staker: "addr0001".to_string(),
            reward_index: Decimal::from_ratio(5u128, 1u128) + Decimal::from_ratio(100u128, 15u128),
            unlocked_stake_amount: Uint128::zero(),
            locked_stake_amount: Uint128::zero(),
            pending_reward: Uint128::zero(),
            last_balance_update: 12363,
            lockups: vec![],
        }
    );
}

#[test]
fn test_locked_stake_tokens() {
    let mut deps = mock_dependencies(&[]);
    let mut env = mock_env();

    let msg = InstantiateMsg {
        owner: "owner".to_string(),
        bro_token: "bro0000".to_string(),
        rewards_pool_contract: "reward0000".to_string(),
        bbro_minter_contract: "bbrominter0000".to_string(),
        epoch_manager_contract: "epoch0000".to_string(),
        unstake_period_blocks: 10,
        min_staking_amount: Uint128::zero(),
        min_lockup_period_epochs: 1,
        max_lockup_period_epochs: 365,
        base_rate: Decimal::from_str("0.0001").unwrap(),
        linear_growth: Decimal::from_str("0.0005").unwrap(),
        exponential_growth: Decimal::from_str("0.0000075").unwrap(),
    };

    let info = mock_info("addr0000", &[]);
    let _res = instantiate(deps.as_mut(), env.clone(), info, msg).unwrap();

    // stake tokens with locked type
    env.block.height += 1;

    let addr1 = "addr0001".to_string();
    let msg = ExecuteMsg::Receive(Cw20ReceiveMsg {
        sender: addr1.clone(),
        amount: Uint128::from(1_000000u128),
        msg: to_binary(&Cw20HookMsg::Stake {
            stake_type: StakeType::Locked { epochs_locked: 1 },
        })
        .unwrap(),
    });

    let info = mock_info("bro0000", &[]);
    let res = execute(deps.as_mut(), env.clone(), info, msg).unwrap();

    let bbro_premium_reward_msg = res.messages.get(0).expect("no message");
    assert_eq!(
        bbro_premium_reward_msg,
        &SubMsg::new(CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: "bbrominter0000".to_string(),
            funds: vec![],
            msg: to_binary(&BbroMintMsg::Mint {
                recipient: addr1.clone(),
                amount: Uint128::from(107u128),
            })
            .unwrap(),
        }))
    );

    assert_eq!(
        from_binary::<StakerInfoResponse>(
            &query(
                deps.as_ref(),
                env.clone(),
                QueryMsg::StakerInfo {
                    staker: addr1.clone(),
                },
            )
            .unwrap(),
        )
        .unwrap(),
        StakerInfoResponse {
            staker: addr1.clone(),
            reward_index: Decimal::zero(),
            unlocked_stake_amount: Uint128::zero(),
            locked_stake_amount: Uint128::from(1_000000u128),
            pending_reward: Uint128::zero(),
            last_balance_update: 12346,
            lockups: vec![LockupInfoResponse {
                amount: Uint128::from(1_000000u128),
                unlocked_at: Expiration::AtHeight(12347),
            }],
        },
    );

    // stake more tokens with locked type, previos lock must move to unlocked
    env.block.height += 1;
    let msg = ExecuteMsg::Receive(Cw20ReceiveMsg {
        sender: addr1.clone(),
        amount: Uint128::from(1_000000u128),
        msg: to_binary(&Cw20HookMsg::Stake {
            stake_type: StakeType::Locked { epochs_locked: 5 },
        })
        .unwrap(),
    });

    let info = mock_info("bro0000", &[]);
    let res = execute(deps.as_mut(), env.clone(), info, msg).unwrap();

    let bbro_premium_reward_msg = res.messages.get(0).expect("no message");
    assert_eq!(
        bbro_premium_reward_msg,
        &SubMsg::new(CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: "bbrominter0000".to_string(),
            funds: vec![],
            msg: to_binary(&BbroMintMsg::Mint {
                recipient: addr1.clone(),
                amount: Uint128::from(2287u128),
            })
            .unwrap(),
        }))
    );

    assert_eq!(
        from_binary::<StakerInfoResponse>(
            &query(
                deps.as_ref(),
                env.clone(),
                QueryMsg::StakerInfo {
                    staker: addr1.clone(),
                },
            )
            .unwrap(),
        )
        .unwrap(),
        StakerInfoResponse {
            staker: addr1.clone(),
            reward_index: Decimal::zero(),
            unlocked_stake_amount: Uint128::from(1_000000u128),
            locked_stake_amount: Uint128::from(1_000000u128),
            pending_reward: Uint128::zero(),
            last_balance_update: 12347,
            lockups: vec![LockupInfoResponse {
                amount: Uint128::from(1_000000u128),
                unlocked_at: Expiration::AtHeight(12352),
            }],
        },
    );

    assert_eq!(
        from_binary::<StateResponse>(
            &query(deps.as_ref(), mock_env(), QueryMsg::State {}).unwrap()
        )
        .unwrap(),
        StateResponse {
            total_stake_amount: Uint128::from(2_000000u128),
            global_reward_index: Decimal::zero(),
            last_distribution_block: 12345,
        },
    );

    // distribute tokens for claiming bbro reward
    let info = mock_info("bro0000", &[]);

    let msg = ExecuteMsg::Receive(Cw20ReceiveMsg {
        sender: "reward0000".to_string(),
        amount: Uint128::from(1000u128),
        msg: to_binary(&Cw20HookMsg::DistributeReward {
            distributed_at_block: 12348,
        })
        .unwrap(),
    });

    let _res = execute(deps.as_mut(), env.clone(), info, msg).unwrap();

    // claim bbro reward for the whole staked amount
    env.block.height = 12352;

    let msg = ExecuteMsg::ClaimBbroRewards {};
    let info = mock_info(&addr1, &[]);
    let res = execute(deps.as_mut(), env.clone(), info, msg).unwrap();

    let bbro_reward_msg = res.messages.get(0).expect("no message");
    assert_eq!(
        bbro_reward_msg,
        &SubMsg::new(CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: "bbrominter0000".to_string(),
            funds: vec![],
            msg: to_binary(&BbroMintMsg::Mint {
                recipient: addr1.clone(),
                amount: Uint128::from(600000u128),
            })
            .unwrap(),
        }))
    );

    // all locks must be released
    assert_eq!(
        from_binary::<StakerInfoResponse>(
            &query(
                deps.as_ref(),
                env.clone(),
                QueryMsg::StakerInfo {
                    staker: addr1.clone(),
                },
            )
            .unwrap(),
        )
        .unwrap(),
        StakerInfoResponse {
            staker: addr1.clone(),
            reward_index: Decimal::from_str("0.0005").unwrap(),
            unlocked_stake_amount: Uint128::from(2_000000u128),
            locked_stake_amount: Uint128::zero(),
            pending_reward: Uint128::from(1000u128),
            last_balance_update: 12352,
            lockups: vec![],
        },
    );

    // try to claim twice
    let msg = ExecuteMsg::ClaimBbroRewards {};
    let info = mock_info(&addr1, &[]);
    match execute(deps.as_mut(), env.clone(), info, msg) {
        Err(ContractError::NothingToClaim {}) => (),
        _ => panic!("expecting ContractError::NothingToClaim"),
    }

    // try to lockup more than unlocked
    let msg = ExecuteMsg::LockupStaked {
        amount: Uint128::from(3_000000u128),
        epochs_locked: 5,
    };
    let info = mock_info(&addr1, &[]);
    match execute(deps.as_mut(), env.clone(), info, msg) {
        Err(ContractError::ForbiddenToLockupMoreThanUnlocked {}) => (),
        _ => panic!("expecting ContractError::ForbiddenToLockupMoreThanUnlocked"),
    }

    // pass invalid epoch amount
    let msg = ExecuteMsg::LockupStaked {
        amount: Uint128::from(2_000000u128),
        epochs_locked: 366,
    };
    let info = mock_info(&addr1, &[]);
    match execute(deps.as_mut(), env.clone(), info, msg) {
        Err(ContractError::InvalidLockupPeriod {}) => (),
        _ => panic!("expecting ContractError::InvalidLockupPeriod"),
    }

    // proper lockup
    env.block.height = 12370;
    let msg = ExecuteMsg::LockupStaked {
        amount: Uint128::from(2_000000u128),
        epochs_locked: 5,
    };

    let info = mock_info(&addr1, &[]);
    let res = execute(deps.as_mut(), env.clone(), info, msg).unwrap();

    let bbro_premium_reward_msg = res.messages.get(0).expect("no message");
    assert_eq!(
        bbro_premium_reward_msg,
        &SubMsg::new(CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: "bbrominter0000".to_string(),
            funds: vec![],
            msg: to_binary(&BbroMintMsg::Mint {
                recipient: addr1.clone(),
                amount: Uint128::from(4575u128),
            })
            .unwrap(),
        }))
    );

    assert_eq!(
        from_binary::<StakerInfoResponse>(
            &query(
                deps.as_ref(),
                env.clone(),
                QueryMsg::StakerInfo {
                    staker: addr1.clone(),
                },
            )
            .unwrap(),
        )
        .unwrap(),
        StakerInfoResponse {
            staker: addr1.clone(),
            reward_index: Decimal::from_str("0.0005").unwrap(),
            unlocked_stake_amount: Uint128::zero(),
            locked_stake_amount: Uint128::from(2_000000u128),
            pending_reward: Uint128::from(1000u128),
            last_balance_update: 12352,
            lockups: vec![LockupInfoResponse {
                amount: Uint128::from(2_000000u128),
                unlocked_at: Expiration::AtHeight(12375),
            }],
        },
    );
}

#[test]
fn update_config() {
    let mut deps = mock_dependencies(&[]);
    let env = mock_env();

    let msg = InstantiateMsg {
        owner: "owner".to_string(),
        bro_token: "bro0000".to_string(),
        rewards_pool_contract: "reward0000".to_string(),
        bbro_minter_contract: "bbrominter0000".to_string(),
        epoch_manager_contract: "epoch0000".to_string(),
        unstake_period_blocks: 10,
        min_staking_amount: Uint128::zero(),
        min_lockup_period_epochs: 1,
        max_lockup_period_epochs: 365,
        base_rate: Decimal::from_str("0.0001").unwrap(),
        linear_growth: Decimal::from_str("0.0005").unwrap(),
        exponential_growth: Decimal::from_str("0.0000075").unwrap(),
    };

    let info = mock_info("addr0000", &[]);
    let _res = instantiate(deps.as_mut(), env.clone(), info, msg).unwrap();

    // unauthorized: only owner allowed to execute
    let msg = ExecuteMsg::UpdateConfig {
        owner: None,
        unstake_period_blocks: None,
        min_staking_amount: None,
        min_lockup_period_epochs: None,
        max_lockup_period_epochs: None,
        base_rate: None,
        linear_growth: None,
        exponential_growth: None,
    };

    let info = mock_info("addr0000", &[]);
    let res = execute(deps.as_mut(), mock_env(), info, msg);
    match res {
        Err(ContractError::Unauthorized {}) => assert_eq!(true, true),
        _ => panic!("DO NOT ENTER HERE"),
    }

    // proper execution
    let msg = ExecuteMsg::UpdateConfig {
        owner: Some("new_owner".to_string()),
        unstake_period_blocks: Some(11),
        min_staking_amount: Some(Uint128::from(1u128)),
        min_lockup_period_epochs: Some(2),
        max_lockup_period_epochs: Some(364),
        base_rate: Some(Decimal::from_str("0.0002").unwrap()),
        linear_growth: Some(Decimal::from_str("0.0006").unwrap()),
        exponential_growth: Some(Decimal::from_str("0.0000076").unwrap()),
    };

    let info = mock_info("owner", &[]);
    let _res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();

    assert_eq!(
        from_binary::<ConfigResponse>(
            &query(deps.as_ref(), mock_env(), QueryMsg::Config {}).unwrap()
        )
        .unwrap(),
        ConfigResponse {
            owner: "new_owner".to_string(),
            bro_token: "bro0000".to_string(),
            rewards_pool_contract: "reward0000".to_string(),
            bbro_minter_contract: "bbrominter0000".to_string(),
            epoch_manager_contract: "epoch0000".to_string(),
            unstake_period_blocks: 11,
            min_staking_amount: Uint128::from(1u128),
            lockup_config: LockupConfigResponse {
                min_lockup_period_epochs: 2,
                max_lockup_period_epochs: 364,
                base_rate: Decimal::from_str("0.0002").unwrap(),
                linear_growth: Decimal::from_str("0.0006").unwrap(),
                exponential_growth: Decimal::from_str("0.0000076").unwrap(),
            }
        }
    );
}
