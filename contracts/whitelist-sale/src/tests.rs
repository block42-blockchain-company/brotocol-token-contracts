use crate::contract::{execute, instantiate, query};
use crate::error::ContractError;
use cosmwasm_std::testing::{mock_env, mock_info};
use cosmwasm_std::{
    from_binary, to_binary, BankMsg, Coin, CosmosMsg, SubMsg, Timestamp, Uint128, WasmMsg,
};
use cw20::{Cw20ExecuteMsg, Cw20ReceiveMsg};

use crate::mock_querier::mock_dependencies;

use services::whitelist_sale::{
    ConfigResponse, Cw20HookMsg, ExecuteMsg, InstantiateMsg, QueryMsg, StateResponse,
    WhitelistedAccountInfo, WhitelistedAccountInfoResponse,
};

#[test]
fn proper_initialization() {
    let mut deps = mock_dependencies(&[]);

    let msg = InstantiateMsg {
        bro_token: "bro0000".to_string(),
        bro_amount_per_uusd: Uint128::from(10u128),
        bro_amount_per_nft: Uint128::from(100u128),
        treasury_contract: "treasury".to_string(),
        rewards_pool_contract: "rewards".to_string(),
    };

    let info = mock_info("addr0000", &[]);
    let _res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

    assert_eq!(
        from_binary::<ConfigResponse>(
            &query(deps.as_ref(), mock_env(), QueryMsg::Config {}).unwrap()
        )
        .unwrap(),
        ConfigResponse {
            owner: "addr0000".to_string(),
            bro_token: "bro0000".to_string(),
            bro_amount_per_uusd: Uint128::from(10u128),
            bro_amount_per_nft: Uint128::from(100u128),
            treasury_contract: "treasury".to_string(),
            rewards_pool_contract: "rewards".to_string(),
        },
    );

    assert_eq!(
        from_binary::<StateResponse>(
            &query(deps.as_ref(), mock_env(), QueryMsg::State {}).unwrap()
        )
        .unwrap(),
        StateResponse {
            sale_registered: false,
            sale_start_time: 0,
            sale_end_time: 0,
            current_time: mock_env().block.time.seconds(),
            balance: Uint128::zero(),
        },
    );
}

#[test]
fn register_sale() {
    let mut deps = mock_dependencies(&[]);

    let msg = InstantiateMsg {
        bro_token: "bro0000".to_string(),
        bro_amount_per_uusd: Uint128::from(10u128),
        bro_amount_per_nft: Uint128::from(100u128),
        treasury_contract: "treasury".to_string(),
        rewards_pool_contract: "rewards".to_string(),
    };

    let info = mock_info("owner", &[]);
    let _res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

    // error: unauthorized (info.sender must be bro token addr)
    let msg = ExecuteMsg::Receive(Cw20ReceiveMsg {
        sender: "addr0000".to_string(),
        amount: Uint128::zero(),
        msg: to_binary(&Cw20HookMsg::RegisterSale {
            sale_start_time: 0,
            sale_end_time: 0,
            accounts: to_binary(&vec![WhitelistedAccountInfo {
                address: "addr0001".to_string(),
                owned_nfts_count: 1,
            }])
            .unwrap(),
        })
        .unwrap(),
    });

    let info = mock_info("addr0004", &[]);
    let res = execute(deps.as_mut(), mock_env(), info, msg.clone());
    match res {
        Err(ContractError::Unauthorized {}) => assert!(true),
        _ => panic!("DO NOT ENTER HERE"),
    }

    // error: unauthorized (cw20_msg.sender must be contract owner)
    let msg = ExecuteMsg::Receive(Cw20ReceiveMsg {
        sender: "addr0000".to_string(),
        amount: Uint128::zero(),
        msg: to_binary(&Cw20HookMsg::RegisterSale {
            sale_start_time: 0,
            sale_end_time: 0,
            accounts: to_binary(&vec![WhitelistedAccountInfo {
                address: "addr0001".to_string(),
                owned_nfts_count: 1,
            }])
            .unwrap(),
        })
        .unwrap(),
    });

    let info = mock_info("bro0000", &[]);
    let res = execute(deps.as_mut(), mock_env(), info, msg.clone());
    match res {
        Err(ContractError::Unauthorized {}) => assert!(true),
        _ => panic!("DO NOT ENTER HERE"),
    }

    // error: invalid sale period(end time < start time)
    let msg = ExecuteMsg::Receive(Cw20ReceiveMsg {
        sender: "owner".to_string(),
        amount: Uint128::zero(),
        msg: to_binary(&Cw20HookMsg::RegisterSale {
            sale_start_time: 0,
            sale_end_time: 0,
            accounts: to_binary(&vec![WhitelistedAccountInfo {
                address: "addr0001".to_string(),
                owned_nfts_count: 1,
            }])
            .unwrap(),
        })
        .unwrap(),
    });

    let info = mock_info("bro0000", &[]);
    let res = execute(deps.as_mut(), mock_env(), info, msg.clone());
    match res {
        Err(ContractError::InvalidSalePeriod {}) => assert!(true),
        _ => panic!("DO NOT ENTER HERE"),
    }

    // error: invalid sale period (start time < current time)
    let mut env = mock_env();
    env.block.time = Timestamp::from_seconds(11);

    let msg = ExecuteMsg::Receive(Cw20ReceiveMsg {
        sender: "owner".to_string(),
        amount: Uint128::zero(),
        msg: to_binary(&Cw20HookMsg::RegisterSale {
            sale_start_time: 10,
            sale_end_time: 20,
            accounts: to_binary(&vec![WhitelistedAccountInfo {
                address: "addr0001".to_string(),
                owned_nfts_count: 1,
            }])
            .unwrap(),
        })
        .unwrap(),
    });

    let info = mock_info("bro0000", &[]);
    let res = execute(deps.as_mut(), env.clone(), info, msg.clone());
    match res {
        Err(ContractError::InvalidSalePeriod {}) => assert!(true),
        _ => panic!("DO NOT ENTER HERE"),
    }

    // error: transfered amount less then required
    let msg = ExecuteMsg::Receive(Cw20ReceiveMsg {
        sender: "owner".to_string(),
        amount: Uint128::from(100u128),
        msg: to_binary(&Cw20HookMsg::RegisterSale {
            sale_start_time: 50,
            sale_end_time: 100,
            accounts: to_binary(&vec![WhitelistedAccountInfo {
                address: "addr0001".to_string(),
                owned_nfts_count: 2,
            }])
            .unwrap(),
        })
        .unwrap(),
    });

    let info = mock_info("bro0000", &[]);
    let res = execute(deps.as_mut(), env.clone(), info, msg.clone());
    match res {
        Err(ContractError::ReceivedAmountMustBeHigherThenRequiredAmountForSale {}) => assert!(true),
        _ => panic!("DO NOT ENTER HERE"),
    }

    // proper execution
    let msg = ExecuteMsg::Receive(Cw20ReceiveMsg {
        sender: "owner".to_string(),
        amount: Uint128::from(201u128),
        msg: to_binary(&Cw20HookMsg::RegisterSale {
            sale_start_time: 50,
            sale_end_time: 100,
            accounts: to_binary(&vec![WhitelistedAccountInfo {
                address: "addr0001".to_string(),
                owned_nfts_count: 2,
            }])
            .unwrap(),
        })
        .unwrap(),
    });

    let info = mock_info("bro0000", &[]);
    let _res = execute(deps.as_mut(), env.clone(), info, msg.clone()).unwrap();

    assert_eq!(
        from_binary::<StateResponse>(
            &query(deps.as_ref(), env.clone(), QueryMsg::State {}).unwrap()
        )
        .unwrap(),
        StateResponse {
            sale_registered: true,
            sale_start_time: 50,
            sale_end_time: 100,
            current_time: 11,
            balance: Uint128::from(201u128),
        },
    );

    assert_eq!(
        from_binary::<WhitelistedAccountInfoResponse>(
            &query(
                deps.as_ref(),
                mock_env(),
                QueryMsg::WhitelistedAccount {
                    address: "addr0001".to_string()
                }
            )
            .unwrap()
        )
        .unwrap(),
        WhitelistedAccountInfoResponse {
            address: "addr0001".to_string(),
            available_purchase_amount: Uint128::from(200u128),
        },
    );

    // error: sale already registered
    let msg = ExecuteMsg::Receive(Cw20ReceiveMsg {
        sender: "owner".to_string(),
        amount: Uint128::from(201u128),
        msg: to_binary(&Cw20HookMsg::RegisterSale {
            sale_start_time: 50,
            sale_end_time: 100,
            accounts: to_binary(&vec![WhitelistedAccountInfo {
                address: "addr0001".to_string(),
                owned_nfts_count: 2,
            }])
            .unwrap(),
        })
        .unwrap(),
    });

    let info = mock_info("bro0000", &[]);
    let res = execute(deps.as_mut(), env.clone(), info, msg.clone());
    match res {
        Err(ContractError::SaleWasAlreadyRegistered {}) => assert!(true),
        _ => panic!("DO NOT ENTER HERE"),
    }
}

#[test]
fn purchase() {
    let mut deps = mock_dependencies(&[]);

    let msg = InstantiateMsg {
        bro_token: "bro0000".to_string(),
        bro_amount_per_uusd: Uint128::from(10u128),
        bro_amount_per_nft: Uint128::from(100_000000u128),
        treasury_contract: "treasury".to_string(),
        rewards_pool_contract: "rewards".to_string(),
    };

    let info = mock_info("owner", &[]);
    let _res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

    // error: sale is not on(sale is not registered)
    let msg = ExecuteMsg::Purchase {};
    let info = mock_info("addr0000", &[]);
    let res = execute(deps.as_mut(), mock_env(), info, msg.clone());
    match res {
        Err(ContractError::SaleIsNotLive {}) => assert!(true),
        _ => panic!("DO NOT ENTER HERE"),
    }

    // whitelist accounts
    let mut env = mock_env();
    env.block.time = Timestamp::from_seconds(11);

    let msg = ExecuteMsg::Receive(Cw20ReceiveMsg {
        sender: "owner".to_string(),
        amount: Uint128::from(600_000000u128),
        msg: to_binary(&Cw20HookMsg::RegisterSale {
            sale_start_time: 50,
            sale_end_time: 100,
            accounts: to_binary(&vec![
                WhitelistedAccountInfo {
                    address: "addr0001".to_string(),
                    owned_nfts_count: 1,
                },
                WhitelistedAccountInfo {
                    address: "addr0002".to_string(),
                    owned_nfts_count: 2,
                },
                WhitelistedAccountInfo {
                    address: "addr0003".to_string(),
                    owned_nfts_count: 3,
                },
            ])
            .unwrap(),
        })
        .unwrap(),
    });

    let info = mock_info("bro0000", &[]);
    let _res = execute(deps.as_mut(), env.clone(), info, msg.clone()).unwrap();

    // error: sale is not on(not started)
    let msg = ExecuteMsg::Purchase {};
    let info = mock_info("addr0001", &[]);
    let res = execute(deps.as_mut(), env.clone(), info, msg.clone());
    match res {
        Err(ContractError::SaleIsNotLive {}) => assert!(true),
        _ => panic!("DO NOT ENTER HERE"),
    }

    // error sale is not on(already finished)
    env.block.time = Timestamp::from_seconds(101);
    let info = mock_info("addr0001", &[]);
    let res = execute(deps.as_mut(), env.clone(), info, msg.clone());
    match res {
        Err(ContractError::SaleIsNotLive {}) => assert!(true),
        _ => panic!("DO NOT ENTER HERE"),
    }

    // error: invalid funds input
    env.block.time = Timestamp::from_seconds(60);

    let info = mock_info(
        "addr0001",
        &[Coin {
            denom: "uluna".to_string(),
            amount: Uint128::from(100u128),
        }],
    );
    let res = execute(deps.as_mut(), env.clone(), info, msg.clone());
    match res {
        Err(ContractError::InvalidFundsInput {}) => assert!(true),
        _ => panic!("DO NOT ENTER HERE"),
    }

    // error: address is not whitelisted
    let info = mock_info(
        "addr0007",
        &[Coin {
            denom: "uusd".to_string(),
            amount: Uint128::from(100u128),
        }],
    );
    let res = execute(deps.as_mut(), env.clone(), info, msg.clone());
    match res {
        Err(ContractError::AddressIsNotWhitelisted {}) => assert!(true),
        _ => panic!("DO NOT ENTER HERE"),
    }

    // error: purchase amount too high
    // addr0001 can buy at maximum 100BRO, try to input ust for more bro
    let info = mock_info(
        "addr0001",
        &[Coin {
            denom: "uusd".to_string(),
            amount: Uint128::from(100_000000u128),
        }],
    );
    let res = execute(deps.as_mut(), env.clone(), info, msg.clone());
    match res {
        Err(ContractError::PurchaseAmountIsTooHigh {}) => assert!(true),
        _ => panic!("DO NOT ENTER HERE"),
    }

    // proper execution
    let info = mock_info(
        "addr0003",
        &[Coin {
            denom: "uusd".to_string(),
            amount: Uint128::from(10_000000u128),
        }],
    );
    let res = execute(deps.as_mut(), env.clone(), info, msg.clone()).unwrap();

    let transfer_ust_msg = res.messages.get(0).expect("no message");
    assert_eq!(
        transfer_ust_msg,
        &SubMsg::new(CosmosMsg::Bank(BankMsg::Send {
            to_address: "treasury".to_string(),
            amount: vec![Coin {
                denom: "uusd".to_string(),
                amount: Uint128::from(10_000000u128),
            }],
        }))
    );

    let transfer_purchased_bro_msg = res.messages.get(1).expect("no message");
    assert_eq!(
        transfer_purchased_bro_msg,
        &SubMsg::new(CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: "bro0000".to_string(),
            funds: vec![],
            msg: to_binary(&Cw20ExecuteMsg::Transfer {
                recipient: "addr0003".to_string(),
                amount: Uint128::from(100_000000u128),
            })
            .unwrap()
        }))
    );

    assert_eq!(
        from_binary::<StateResponse>(
            &query(deps.as_ref(), env.clone(), QueryMsg::State {}).unwrap()
        )
        .unwrap(),
        StateResponse {
            sale_registered: true,
            sale_start_time: 50,
            sale_end_time: 100,
            current_time: 60,
            balance: Uint128::from(500_000000u128),
        },
    );

    assert_eq!(
        from_binary::<WhitelistedAccountInfoResponse>(
            &query(
                deps.as_ref(),
                mock_env(),
                QueryMsg::WhitelistedAccount {
                    address: "addr0003".to_string()
                }
            )
            .unwrap()
        )
        .unwrap(),
        WhitelistedAccountInfoResponse {
            address: "addr0003".to_string(),
            available_purchase_amount: Uint128::from(200_000000u128),
        },
    );

    assert_eq!(
        from_binary::<WhitelistedAccountInfoResponse>(
            &query(
                deps.as_ref(),
                mock_env(),
                QueryMsg::WhitelistedAccount {
                    address: "addr0001".to_string()
                }
            )
            .unwrap()
        )
        .unwrap(),
        WhitelistedAccountInfoResponse {
            address: "addr0001".to_string(),
            available_purchase_amount: Uint128::from(100_000000u128),
        },
    );
}

#[test]
fn withdraw_remaining_balance() {
    let mut deps = mock_dependencies(&[]);

    let msg = InstantiateMsg {
        bro_token: "bro0000".to_string(),
        bro_amount_per_uusd: Uint128::from(10u128),
        bro_amount_per_nft: Uint128::from(100u128),
        treasury_contract: "treasury".to_string(),
        rewards_pool_contract: "rewards".to_string(),
    };

    let info = mock_info("owner", &[]);
    let _res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

    // error: unauthorized(only owner can withdraw)
    let msg = ExecuteMsg::WithdrawRemainingBalance {};

    let info = mock_info("addr0000", &[]);
    let res = execute(deps.as_mut(), mock_env(), info, msg.clone());
    match res {
        Err(ContractError::Unauthorized {}) => assert!(true),
        _ => panic!("DO NOT ENTER HERE"),
    }

    // error: sale is not finished(not registered)
    let info = mock_info("owner", &[]);
    let res = execute(deps.as_mut(), mock_env(), info, msg.clone());
    match res {
        Err(ContractError::SaleIsNotFinishedYet {}) => assert!(true),
        _ => panic!("DO NOT ENTER HERE"),
    }

    // whitelist accounts
    let mut env = mock_env();
    env.block.time = Timestamp::from_seconds(11);

    let msg = ExecuteMsg::Receive(Cw20ReceiveMsg {
        sender: "owner".to_string(),
        amount: Uint128::from(200_000000u128),
        msg: to_binary(&Cw20HookMsg::RegisterSale {
            sale_start_time: 50,
            sale_end_time: 100,
            accounts: to_binary(&vec![WhitelistedAccountInfo {
                address: "addr0001".to_string(),
                owned_nfts_count: 1,
            }])
            .unwrap(),
        })
        .unwrap(),
    });

    let info = mock_info("bro0000", &[]);
    let _res = execute(deps.as_mut(), env.clone(), info, msg.clone()).unwrap();

    // error: sale is not finished(not passed yet)
    env.block.time = Timestamp::from_seconds(99);
    let msg = ExecuteMsg::WithdrawRemainingBalance {};

    let info = mock_info("owner", &[]);
    let res = execute(deps.as_mut(), env.clone(), info, msg.clone());
    match res {
        Err(ContractError::SaleIsNotFinishedYet {}) => assert!(true),
        _ => panic!("DO NOT ENTER HERE"),
    }

    // proper execution
    env.block.time = Timestamp::from_seconds(101);
    let info = mock_info("owner", &[]);
    let res = execute(deps.as_mut(), env.clone(), info, msg.clone()).unwrap();

    let transfer_remaining_bro_msg = res.messages.get(0).expect("no message");
    assert_eq!(
        transfer_remaining_bro_msg,
        &SubMsg::new(CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: "bro0000".to_string(),
            funds: vec![],
            msg: to_binary(&Cw20ExecuteMsg::Transfer {
                recipient: "rewards".to_string(),
                amount: Uint128::from(200_000000u128),
            })
            .unwrap(),
        }))
    );

    assert_eq!(
        from_binary::<StateResponse>(
            &query(deps.as_ref(), env.clone(), QueryMsg::State {}).unwrap()
        )
        .unwrap(),
        StateResponse {
            sale_registered: true,
            sale_start_time: 50,
            sale_end_time: 100,
            current_time: 101,
            balance: Uint128::zero(),
        },
    );
}
