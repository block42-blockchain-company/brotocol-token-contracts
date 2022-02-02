use crate::contract::{execute, instantiate, query};
use crate::error::ContractError;
use crate::mock_querier::mock_dependencies;
use services::bbro_minter::ExecuteMsg as BbroMintMsg;
use services::staking::{
    ConfigResponse, Cw20HookMsg, ExecuteMsg, InstantiateMsg, QueryMsg,
    StakerAccruedRewardsResponse, StakerInfoResponse, StateResponse, WithdrawalInfoResponse,
    WithdrawalsResponse,
};

use cosmwasm_std::testing::{mock_env, mock_info};
use cosmwasm_std::{from_binary, to_binary, CosmosMsg, Decimal, SubMsg, Uint128, WasmMsg};
use cw20::{Cw20ExecuteMsg, Cw20ReceiveMsg, Expiration};

use std::str::FromStr;

#[test]
fn proper_initialization() {
    let mut deps = mock_dependencies(&[]);

    let msg = InstantiateMsg {
        bro_token: "bro0000".to_string(),
        rewards_pool_contract: "reward0000".to_string(),
        bbro_minter_contract: "bbrominter0000".to_string(),
        epoch_manager_contract: "epoch0000".to_string(),
        unbond_period_blocks: 10,
    };

    let info = mock_info("addr0000", &[]);
    let _res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

    assert_eq!(
        from_binary::<ConfigResponse>(
            &query(deps.as_ref(), mock_env(), QueryMsg::Config {}).unwrap()
        )
        .unwrap(),
        ConfigResponse {
            bro_token: "bro0000".to_string(),
            rewards_pool_contract: "reward0000".to_string(),
            bbro_minter_contract: "bbrominter0000".to_string(),
            epoch_manager_contract: "epoch0000".to_string(),
            unbond_period_blocks: 10,
        }
    );

    assert_eq!(
        from_binary::<StateResponse>(
            &query(deps.as_ref(), mock_env(), QueryMsg::State {}).unwrap()
        )
        .unwrap(),
        StateResponse {
            total_bond_amount: Uint128::zero(),
            global_reward_index: Decimal::zero(),
            last_distribution_block: 12345,
        }
    );
}

#[test]
fn test_fractional_rewards() {
    let mut deps = mock_dependencies(&[]);

    ////////////////////////////////////////////////////////////////////////////
    /////// instantiate the contract
    ////////////////////////////////////////////////////////////////////////////

    let msg = InstantiateMsg {
        bro_token: "bro0000".to_string(),
        rewards_pool_contract: "reward0000".to_string(),
        bbro_minter_contract: "bbrominter0000".to_string(),
        epoch_manager_contract: "epoch0000".to_string(),
        unbond_period_blocks: 10,
    };

    let info = mock_info("addr0000", &[]);
    let _res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();
    let mut env = mock_env();

    ////////////////////////////////////////////////////////////////////////////
    /////// addr0000 bonds 100 tokens for 3 addresses, but keep the reward pool at 0
    ////////////////////////////////////////////////////////////////////////////

    let info = mock_info("bro0000", &[]);
    env.block.height += 1;

    let msg = ExecuteMsg::Receive(Cw20ReceiveMsg {
        sender: "addr0000".to_string(),
        amount: Uint128::from(100u128),
        msg: to_binary(&Cw20HookMsg::Bond {}).unwrap(),
    });
    let _res = execute(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();

    let info = mock_info("bro0000", &[]);
    env.block.height += 1;

    let msg = ExecuteMsg::Receive(Cw20ReceiveMsg {
        sender: "addr0001".to_string(),
        amount: Uint128::from(100u128),
        msg: to_binary(&Cw20HookMsg::Bond {}).unwrap(),
    });
    let _res = execute(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();

    let info = mock_info("bro0000", &[]);
    env.block.height += 1;

    let msg = ExecuteMsg::Receive(Cw20ReceiveMsg {
        sender: "addr0002".to_string(),
        amount: Uint128::from(100u128),
        msg: to_binary(&Cw20HookMsg::Bond {}).unwrap(),
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
            bond_amount: Uint128::from(100u128),
            pending_reward: Uint128::from(333u128),
            last_balance_update: 12346,
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
            bond_amount: Uint128::from(100u128),
            pending_reward: Uint128::from(333u128),
            last_balance_update: 12347,
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
            bond_amount: Uint128::from(100u128),
            pending_reward: Uint128::from(333u128),
            last_balance_update: 12348,
        }
    );

    assert_eq!(
        from_binary::<StateResponse>(
            &query(deps.as_ref(), mock_env(), QueryMsg::State {}).unwrap()
        )
        .unwrap(),
        StateResponse {
            total_bond_amount: Uint128::from(300u128),
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
            bond_amount: Uint128::from(100u128),
            pending_reward: Uint128::from(999u128),
            last_balance_update: 12346,
        }
    );
}

#[test]
fn test_bond_tokens() {
    let mut deps = mock_dependencies(&[]);

    ////////////////////////////////////////////////////////////////////////////
    /////// instantiate the contract
    ////////////////////////////////////////////////////////////////////////////

    let msg = InstantiateMsg {
        bro_token: "bro0000".to_string(),
        rewards_pool_contract: "reward0000".to_string(),
        bbro_minter_contract: "bbrominter0000".to_string(),
        epoch_manager_contract: "epoch0000".to_string(),
        unbond_period_blocks: 10,
    };

    let info = mock_info("addr0000", &[]);
    let _res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();
    let mut env = mock_env();

    ////////////////////////////////////////////////////////////////////////////
    /////// calling distribute reward when total bonding is 0
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
            bond_amount: Uint128::zero(),
            pending_reward: Uint128::zero(),
            last_balance_update: 12345,
        }
    );

    assert_eq!(
        from_binary::<StateResponse>(
            &query(deps.as_ref(), mock_env(), QueryMsg::State {}).unwrap()
        )
        .unwrap(),
        StateResponse {
            total_bond_amount: Uint128::zero(),
            global_reward_index: Decimal::zero(),
            last_distribution_block: 12345,
        }
    );

    ////////////////////////////////////////////////////////////////////////////
    /////// addr0000 bonds 100 tokens, but keep the reward pool at 0
    ////////////////////////////////////////////////////////////////////////////

    let info = mock_info("bro0000", &[]);
    env.block.height += 1;

    let msg = ExecuteMsg::Receive(Cw20ReceiveMsg {
        sender: "addr0000".to_string(),
        amount: Uint128::from(100u128),
        msg: to_binary(&Cw20HookMsg::Bond {}).unwrap(),
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
            bond_amount: Uint128::from(100u128),
            pending_reward: Uint128::zero(),
            last_balance_update: env.block.height,
        }
    );

    assert_eq!(
        from_binary::<StateResponse>(
            &query(deps.as_ref(), mock_env(), QueryMsg::State {}).unwrap()
        )
        .unwrap(),
        StateResponse {
            total_bond_amount: Uint128::from(100u128),
            global_reward_index: Decimal::zero(),
            last_distribution_block: 12345,
        }
    );

    ////////////////////////////////////////////////////////////////////////////
    /////// addr0001 bonds 100 tokens, but keep the reward pool at 0
    ////////////////////////////////////////////////////////////////////////////

    let msg = ExecuteMsg::Receive(Cw20ReceiveMsg {
        sender: "addr0001".to_string(),
        amount: Uint128::from(100u128),
        msg: to_binary(&Cw20HookMsg::Bond {}).unwrap(),
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
            bond_amount: Uint128::from(100u128),
            pending_reward: Uint128::zero(),
            last_balance_update: env.block.height,
        }
    );

    assert_eq!(
        from_binary::<StateResponse>(
            &query(deps.as_ref(), mock_env(), QueryMsg::State {}).unwrap()
        )
        .unwrap(),
        StateResponse {
            total_bond_amount: Uint128::from(200u128),
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
            bond_amount: Uint128::from(100u128),
            pending_reward: Uint128::from(500u128),
            last_balance_update: 12346,
        }
    );

    assert_eq!(
        from_binary::<StateResponse>(
            &query(deps.as_ref(), mock_env(), QueryMsg::State {}).unwrap()
        )
        .unwrap(),
        StateResponse {
            total_bond_amount: Uint128::from(200u128),
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
            bbro_staking_reward: Uint128::new(60),
        }
    );

    ////////////////////////////////////////////////////////////////////////////
    /////// addr0000 claims reward
    ////////////////////////////////////////////////////////////////////////////

    let info = mock_info("addr0000", &[]);

    let msg = ExecuteMsg::ClaimRewards {};
    env.block.height += 1;

    let res = execute(deps.as_mut(), env.clone(), info, msg).unwrap();
    assert_eq!(res.attributes.len(), 3);
    assert_eq!(res.attributes[0].key, "action");
    assert_eq!(res.attributes[0].value, "claim_rewards");
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
            bond_amount: Uint128::from(100u128),
            pending_reward: Uint128::zero(),
            last_balance_update: 12346,
        }
    );

    assert_eq!(
        from_binary::<StateResponse>(
            &query(deps.as_ref(), mock_env(), QueryMsg::State {}).unwrap()
        )
        .unwrap(),
        StateResponse {
            total_bond_amount: Uint128::from(200u128),
            global_reward_index: Decimal::from_ratio(5u128, 1u128),
            last_distribution_block: 12348,
        }
    );

    ////////////////////////////////////////////////////////////////////////////
    /////// addr0000 tries to claim reward twice
    ////////////////////////////////////////////////////////////////////////////

    let info = mock_info("addr0000", &[]);

    let msg = ExecuteMsg::ClaimRewards {};

    match execute(deps.as_mut(), env.clone(), info, msg) {
        Err(ContractError::NothingToClaim {}) => (),
        _ => panic!("expecting ContractError::NothingToClaim error"),
    }

    ////////////////////////////////////////////////////////////////////////////
    /////// addr0000 tries to unbond 150 while only bonded 100
    ////////////////////////////////////////////////////////////////////////////

    let info = mock_info("addr0000", &[]);

    env.block.height += 1;

    let msg = ExecuteMsg::Unbond {
        amount: Uint128::new(150),
    };

    match execute(deps.as_mut(), env.clone(), info, msg) {
        Err(ContractError::ForbiddenToUnbondMoreThanBonded {}) => (),
        _ => panic!("expecting failure due to unbonding too much"),
    }

    ////////////////////////////////////////////////////////////////////////////
    /////// addr0000 unbonds 50
    ////////////////////////////////////////////////////////////////////////////

    let info = mock_info("addr0000", &[]);

    let msg = ExecuteMsg::Unbond {
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
    assert_eq!(res.attributes[0].value, "unbond");
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
            bond_amount: Uint128::from(50u128),
            pending_reward: Uint128::zero(),
            last_balance_update: 12350,
        }
    );

    assert_eq!(
        from_binary::<StateResponse>(
            &query(deps.as_ref(), mock_env(), QueryMsg::State {}).unwrap()
        )
        .unwrap(),
        StateResponse {
            total_bond_amount: Uint128::from(150u128),
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
            bond_amount: Uint128::from(50u128),
            pending_reward: Uint128::from(333u128),
            last_balance_update: 12350,
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
            bond_amount: Uint128::from(100u128),
            pending_reward: Uint128::from(1166u128),
            last_balance_update: 12347,
        }
    );

    ////////////////////////////////////////////////////////////////////////////
    /////// addr0000 whithdaws 50 BRO too early
    ////////////////////////////////////////////////////////////////////////////

    let info = mock_info("addr0000", &[]);

    let msg = ExecuteMsg::Withdraw {};
    env.block.height += 1;

    match execute(deps.as_mut(), env.clone(), info, msg) {
        Err(ContractError::NothingToClaim {}) => (),
        _ => panic!("expecting nothing to claim"),
    }

    ////////////////////////////////////////////////////////////////////////////
    /////// addr0000 whithdaws 50 BRO successfully
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
    /////// addr0001 unbonds all his 100
    ////////////////////////////////////////////////////////////////////////////

    let info = mock_info("addr0001", &[]);

    let msg = ExecuteMsg::Unbond {
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
    assert_eq!(res.attributes[0].value, "unbond");
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
            bond_amount: Uint128::from(0u128),
            pending_reward: Uint128::from(1166u128),
            last_balance_update: 12362,
        }
    );

    assert_eq!(
        from_binary::<StateResponse>(
            &query(deps.as_ref(), mock_env(), QueryMsg::State {}).unwrap()
        )
        .unwrap(),
        StateResponse {
            total_bond_amount: Uint128::from(50u128),
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
    /////// addr0001 claims reward after he unbonded everything
    ////////////////////////////////////////////////////////////////////////////

    let info = mock_info("addr0001", &[]);

    let msg = ExecuteMsg::ClaimRewards {};
    env.block.height += 1;

    let res = execute(deps.as_mut(), env.clone(), info, msg).unwrap();
    assert_eq!(res.attributes.len(), 3);
    assert_eq!(res.attributes[0].key, "action");
    assert_eq!(res.attributes[0].value, "claim_rewards");
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
            bond_amount: Uint128::zero(),
            pending_reward: Uint128::zero(),
            last_balance_update: 12363,
        }
    );
}
