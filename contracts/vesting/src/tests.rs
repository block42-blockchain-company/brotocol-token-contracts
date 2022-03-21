use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info};
use cosmwasm_std::{
    attr, from_binary, to_binary, Attribute, CosmosMsg, StdError, SubMsg, Timestamp, Uint128,
    WasmMsg,
};
use cw20::{Cw20ExecuteMsg, Expiration};
use services::ownership_proposal::OwnershipProposalResponse;

use crate::contract::{execute, instantiate, query};
use crate::error::ContractError;
use services::common::OrderBy;
use services::vesting::{
    ConfigResponse, ExecuteMsg, InstantiateMsg, QueryMsg, VestingAccount, VestingAccountResponse,
    VestingAccountsResponse, VestingInfo, VestingSchedule,
};

#[test]
fn proper_initialization() {
    let mut deps = mock_dependencies(&[]);

    let msg = InstantiateMsg {
        owner: "owner".to_string(),
        bro_token: "bro_token".to_string(),
        genesis_time: 12345u64,
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
            bro_token: "bro_token".to_string(),
            genesis_time: 12345u64,
        }
    );
}

#[test]
fn update_config() {
    let mut deps = mock_dependencies(&[]);

    let msg = InstantiateMsg {
        owner: "owner".to_string(),
        bro_token: "bro_token".to_string(),
        genesis_time: 12345u64,
    };

    let info = mock_info("addr0000", &[]);
    let _res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

    let msg = ExecuteMsg::UpdateConfig { genesis_time: None };
    let info = mock_info("owner2", &[]);
    let res = execute(deps.as_mut(), mock_env(), info, msg);
    match res {
        Err(ContractError::Unauthorized {}) => assert_eq!(true, true),
        _ => panic!("DO NOT ENTER HERE"),
    }

    let msg = ExecuteMsg::UpdateConfig {
        genesis_time: Some(1u64),
    };
    let info = mock_info("owner", &[]);
    let res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();

    assert_eq!(res.attributes[0], Attribute::new("action", "update_config"));
    assert_eq!(
        res.attributes[1],
        Attribute::new("genesis_time_changed", "1")
    );

    assert_eq!(
        from_binary::<ConfigResponse>(
            &query(deps.as_ref(), mock_env(), QueryMsg::Config {}).unwrap()
        )
        .unwrap(),
        ConfigResponse {
            owner: "owner".to_string(),
            bro_token: "bro_token".to_string(),
            genesis_time: 1u64,
        }
    );
}

#[test]
fn register_vesting_accounts() {
    let mut deps = mock_dependencies(&[]);

    let msg = InstantiateMsg {
        owner: "owner".to_string(),
        bro_token: "bro_token".to_string(),
        genesis_time: 100u64,
    };

    let info = mock_info("addr0000", &[]);
    let _res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

    // error: invalid schedule
    let msg = ExecuteMsg::RegisterVestingAccounts {
        vesting_accounts: vec![VestingAccount {
            address: "addr0000".to_string(),
            schedules: vec![VestingSchedule {
                start_time: 100u64,
                end_time: 99u64,
                bro_amount: Uint128::zero(),
            }],
        }],
    };
    let info = mock_info("owner", &[]);
    let res = execute(deps.as_mut(), mock_env(), info, msg.clone()).unwrap_err();
    match res {
        ContractError::Std(StdError::GenericErr { msg, .. }) => {
            assert_eq!(msg, "end_time must be bigger than start_time".to_string())
        }
        _ => panic!("DO NOT ENTER HERE"),
    }

    let acct1 = "addr0001".to_string();
    let acct2 = "addr0002".to_string();
    let acct3 = "addr0003".to_string();

    // error: unauthorized
    let msg = ExecuteMsg::RegisterVestingAccounts {
        vesting_accounts: vec![
            VestingAccount {
                address: acct1.clone(),
                schedules: vec![
                    VestingSchedule {
                        start_time: 100u64,
                        end_time: 101u64,
                        bro_amount: Uint128::from(100u128),
                    },
                    VestingSchedule {
                        start_time: 100u64,
                        end_time: 110u64,
                        bro_amount: Uint128::from(100u128),
                    },
                    VestingSchedule {
                        start_time: 100u64,
                        end_time: 200u64,
                        bro_amount: Uint128::from(100u128),
                    },
                ],
            },
            VestingAccount {
                address: acct2.clone(),
                schedules: vec![VestingSchedule {
                    start_time: 100u64,
                    end_time: 110u64,
                    bro_amount: Uint128::from(100u128),
                }],
            },
            VestingAccount {
                address: acct3.clone(),
                schedules: vec![VestingSchedule {
                    start_time: 100u64,
                    end_time: 200u64,
                    bro_amount: Uint128::from(100u128),
                }],
            },
        ],
    };
    let info = mock_info("addr0000", &[]);
    let res = execute(deps.as_mut(), mock_env(), info, msg.clone());
    match res {
        Err(ContractError::Unauthorized {}) => assert_eq!(true, true),
        _ => panic!("DO NOT ENTER HERE"),
    }

    // proper execution
    let info = mock_info("owner", &[]);
    let _res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();
    assert_eq!(
        from_binary::<VestingAccountResponse>(
            &query(
                deps.as_ref(),
                mock_env(),
                QueryMsg::VestingAccount {
                    address: acct1.clone(),
                }
            )
            .unwrap()
        )
        .unwrap(),
        VestingAccountResponse {
            address: acct1.clone(),
            info: VestingInfo {
                last_claim_time: 100u64,
                schedules: vec![
                    VestingSchedule {
                        start_time: 100u64,
                        end_time: 101u64,
                        bro_amount: Uint128::from(100u128),
                    },
                    VestingSchedule {
                        start_time: 100u64,
                        end_time: 110u64,
                        bro_amount: Uint128::from(100u128),
                    },
                    VestingSchedule {
                        start_time: 100u64,
                        end_time: 200u64,
                        bro_amount: Uint128::from(100u128),
                    },
                ],
            }
        }
    );

    assert_eq!(
        from_binary::<VestingAccountsResponse>(
            &query(
                deps.as_ref(),
                mock_env(),
                QueryMsg::VestingAccounts {
                    limit: None,
                    start_after: None,
                    order_by: Some(OrderBy::Asc),
                }
            )
            .unwrap()
        )
        .unwrap(),
        VestingAccountsResponse {
            vesting_accounts: vec![
                VestingAccountResponse {
                    address: acct1,
                    info: VestingInfo {
                        last_claim_time: 100u64,
                        schedules: vec![
                            VestingSchedule {
                                start_time: 100u64,
                                end_time: 101u64,
                                bro_amount: Uint128::from(100u128),
                            },
                            VestingSchedule {
                                start_time: 100u64,
                                end_time: 110u64,
                                bro_amount: Uint128::from(100u128),
                            },
                            VestingSchedule {
                                start_time: 100u64,
                                end_time: 200u64,
                                bro_amount: Uint128::from(100u128),
                            },
                        ],
                    }
                },
                VestingAccountResponse {
                    address: acct2,
                    info: VestingInfo {
                        last_claim_time: 100u64,
                        schedules: vec![VestingSchedule {
                            start_time: 100u64,
                            end_time: 110u64,
                            bro_amount: Uint128::from(100u128),
                        }],
                    }
                },
                VestingAccountResponse {
                    address: acct3,
                    info: VestingInfo {
                        last_claim_time: 100u64,
                        schedules: vec![VestingSchedule {
                            start_time: 100u64,
                            end_time: 200u64,
                            bro_amount: Uint128::from(100u128),
                        }],
                    }
                }
            ]
        }
    );
}

#[test]
fn claim() {
    let mut deps = mock_dependencies(&[]);

    let msg = InstantiateMsg {
        owner: "owner".to_string(),
        bro_token: "bro_token".to_string(),
        genesis_time: 10u64,
    };

    let info = mock_info("addr0000", &[]);
    let _res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

    let msg = ExecuteMsg::RegisterVestingAccounts {
        vesting_accounts: vec![VestingAccount {
            address: "addr0000".to_string(),
            schedules: vec![
                VestingSchedule {
                    start_time: 100u64,
                    end_time: 110u64,
                    bro_amount: Uint128::from(100u128),
                },
                VestingSchedule {
                    start_time: 100u64,
                    end_time: 150u64,
                    bro_amount: Uint128::from(110u128),
                },
                VestingSchedule {
                    start_time: 140u64,
                    end_time: 160u64,
                    bro_amount: Uint128::from(105u128),
                },
                VestingSchedule {
                    start_time: 180u64,
                    end_time: 200u64,
                    bro_amount: Uint128::from(120u128),
                },
            ],
        }],
    };
    let info = mock_info("owner", &[]);
    let _res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();

    let info = mock_info("addr0000", &[]);
    let mut env = mock_env();
    env.block.time = Timestamp::from_seconds(90);

    let msg = ExecuteMsg::Claim {};
    let res = execute(deps.as_mut(), env.clone(), info.clone(), msg.clone()).unwrap();
    assert_eq!(
        res.attributes,
        vec![
            attr("action", "claim"),
            attr("address", "addr0000"),
            attr("claim_amount", "0"),
            attr("last_claim_time", "90"),
        ]
    );
    assert_eq!(res.messages, vec![]);

    env.block.time = Timestamp::from_seconds(105);
    let res = execute(deps.as_mut(), env.clone(), info.clone(), msg.clone()).unwrap();
    assert_eq!(
        res.attributes,
        vec![
            attr("action", "claim"),
            attr("address", "addr0000"),
            attr("claim_amount", "0"),
            attr("last_claim_time", "105"),
        ]
    );
    assert_eq!(res.messages, vec![]);

    env.block.time = Timestamp::from_seconds(120);
    let res = execute(deps.as_mut(), env.clone(), info.clone(), msg.clone()).unwrap();
    assert_eq!(
        res.attributes,
        vec![
            attr("action", "claim"),
            attr("address", "addr0000"),
            attr("claim_amount", "100"),
            attr("last_claim_time", "120"),
        ]
    );
    assert_eq!(
        res.messages,
        vec![SubMsg::new(CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: "bro_token".to_string(),
            msg: to_binary(&Cw20ExecuteMsg::Transfer {
                recipient: "addr0000".to_string(),
                amount: Uint128::from(100u128),
            })
            .unwrap(),
            funds: vec![],
        }))]
    );

    env.block.time = Timestamp::from_seconds(130);
    let res = execute(deps.as_mut(), env.clone(), info.clone(), msg.clone()).unwrap();
    assert_eq!(
        res.attributes,
        vec![
            attr("action", "claim"),
            attr("address", "addr0000"),
            attr("claim_amount", "0"),
            attr("last_claim_time", "130"),
        ]
    );
    assert_eq!(res.messages, vec![]);

    env.block.time = Timestamp::from_seconds(170);
    let res = execute(deps.as_mut(), env.clone(), info.clone(), msg.clone()).unwrap();
    assert_eq!(
        res.attributes,
        vec![
            attr("action", "claim"),
            attr("address", "addr0000"),
            attr("claim_amount", "215"),
            attr("last_claim_time", "170"),
        ]
    );
    assert_eq!(
        res.messages,
        vec![SubMsg::new(CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: "bro_token".to_string(),
            msg: to_binary(&Cw20ExecuteMsg::Transfer {
                recipient: "addr0000".to_string(),
                amount: Uint128::from(215u128),
            })
            .unwrap(),
            funds: vec![],
        }))]
    );

    env.block.time = Timestamp::from_seconds(210);
    let res = execute(deps.as_mut(), env.clone(), info.clone(), msg.clone()).unwrap();
    assert_eq!(
        res.attributes,
        vec![
            attr("action", "claim"),
            attr("address", "addr0000"),
            attr("claim_amount", "120"),
            attr("last_claim_time", "210"),
        ]
    );
    assert_eq!(
        res.messages,
        vec![SubMsg::new(CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: "bro_token".to_string(),
            msg: to_binary(&Cw20ExecuteMsg::Transfer {
                recipient: "addr0000".to_string(),
                amount: Uint128::from(120u128),
            })
            .unwrap(),
            funds: vec![],
        }))]
    );

    env.block.time = Timestamp::from_seconds(250);
    let res = execute(deps.as_mut(), env.clone(), info.clone(), msg.clone()).unwrap();
    assert_eq!(
        res.attributes,
        vec![
            attr("action", "claim"),
            attr("address", "addr0000"),
            attr("claim_amount", "0"),
            attr("last_claim_time", "250"),
        ]
    );
    assert_eq!(res.messages, vec![]);
}

#[test]
fn update_owner() {
    let mut deps = mock_dependencies(&[]);

    let msg = InstantiateMsg {
        owner: "owner0000".to_string(),
        bro_token: "bro_token".to_string(),
        genesis_time: 12345u64,
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
            bro_token: "bro_token".to_string(),
            genesis_time: 12345u64,
        },
    );
}
