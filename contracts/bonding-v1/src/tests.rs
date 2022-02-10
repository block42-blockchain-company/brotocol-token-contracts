use std::str::FromStr;

use crate::contract::{execute, instantiate, query};
use crate::error::ContractError;
use crate::state::{load_claims, store_claims, BondType, ClaimInfo};
use cosmwasm_std::testing::{mock_env, mock_info};
use cosmwasm_std::{
    from_binary, to_binary, BankMsg, Coin, CosmosMsg, Decimal, SubMsg, Uint128, WasmMsg,
};
use cw20::{Cw20ExecuteMsg, Cw20ReceiveMsg, Expiration};

use crate::mock_querier::{
    mock_dependencies, MOCK_ASTRO_FACTORY_ADDR, MOCK_BRO_TOKEN_ADDR, MOCK_BRO_UST_PAIR_ADDR,
    MOCK_LP_TOKEN_ADDR, MOCK_ORACLE_ADDR,
};

use services::{
    bonding::{
        ClaimInfoResponse, ClaimsResponse, ConfigResponse, Cw20HookMsg, ExecuteMsg, InstantiateMsg,
        QueryMsg, StateResponse,
    },
    oracle::ExecuteMsg as OracleExecuteMsg,
};

/// WasmMockQuerier messages:
///
/// astroport factory contract:
/// mock address: astrofactory
///
/// * **FactoryQueryMsg::Pair { .. }** returns:
/// PairInfo {
///     asset_infos: [
///         AssetInfo::Token {
///             contract_addr: Addr::unchecked(MOCK_BRO_TOKEN_ADDR),
///         },
///         AssetInfo::NativeToken {
///             denom: "uusd".to_string(),
///         },
///     ],
///     contract_addr: Addr::unchecked(MOCK_BRO_UST_PAIR_ADDR),
///     liquidity_token: Addr::unchecked(MOCK_LP_TOKEN_ADDR),
///     pair_type: astroport::factory::PairType::Xyk {},
/// }

#[test]
fn proper_initialization() {
    let mut deps = mock_dependencies(&[]);

    // invalid ust_bonding_reward_ratio
    let mut msg = InstantiateMsg {
        owner: "owner".to_string(),
        bro_token: MOCK_BRO_TOKEN_ADDR.to_string(),
        lp_token: MOCK_LP_TOKEN_ADDR.to_string(),
        treasury_contract: "treasury".to_string(),
        astroport_factory: MOCK_ASTRO_FACTORY_ADDR.to_string(),
        oracle_contract: MOCK_ORACLE_ADDR.to_string(),
        ust_bonding_reward_ratio: Decimal::from_str("1.1").unwrap(),
        ust_bonding_discount: Decimal::from_str("0.1").unwrap(),
        lp_bonding_discount: Decimal::from_str("0.05").unwrap(),
        min_bro_payout: Uint128::from(1u128),
        vesting_period_blocks: 10,
        lp_bonding_enabled: true,
    };

    let info = mock_info("addr0001", &[]);
    let res = instantiate(deps.as_mut(), mock_env(), info, msg.clone());
    match res {
        Err(ContractError::InvalidUstBondRatio {}) => assert_eq!(true, true),
        _ => panic!("DO NOT ENTER HERE"),
    }

    msg.ust_bonding_reward_ratio = Decimal::zero();
    let info = mock_info("addr0001", &[]);
    let res = instantiate(deps.as_mut(), mock_env(), info, msg.clone());
    match res {
        Err(ContractError::InvalidUstBondRatio {}) => assert_eq!(true, true),
        _ => panic!("DO NOT ENTER HERE"),
    }

    // proper initialization
    msg.ust_bonding_reward_ratio = Decimal::from_str("0.6").unwrap();
    let info = mock_info("addr0000", &[]);
    let _res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

    assert_eq!(
        from_binary::<ConfigResponse>(
            &query(deps.as_ref(), mock_env(), QueryMsg::Config {}).unwrap()
        )
        .unwrap(),
        ConfigResponse {
            owner: "owner".to_string(),
            bro_token: MOCK_BRO_TOKEN_ADDR.to_string(),
            lp_token: MOCK_LP_TOKEN_ADDR.to_string(),
            treasury_contract: "treasury".to_string(),
            astroport_factory: MOCK_ASTRO_FACTORY_ADDR.to_string(),
            oracle_contract: MOCK_ORACLE_ADDR.to_string(),
            ust_bonding_reward_ratio: Decimal::from_str("0.6").unwrap(),
            ust_bonding_discount: Decimal::from_str("0.1").unwrap(),
            lp_bonding_discount: Decimal::from_str("0.05").unwrap(),
            min_bro_payout: Uint128::from(1u128),
            vesting_period_blocks: 10,
            lp_bonding_enabled: true,
        },
    );

    assert_eq!(
        from_binary::<StateResponse>(
            &query(deps.as_ref(), mock_env(), QueryMsg::State {}).unwrap()
        )
        .unwrap(),
        StateResponse {
            ust_bonding_balance: Uint128::zero(),
            lp_bonding_balance: Uint128::zero(),
        },
    );
}

#[test]
fn distribute_reward() {
    let mut deps = mock_dependencies(&[]);
    let msg = InstantiateMsg {
        owner: "owner".to_string(),
        bro_token: MOCK_BRO_TOKEN_ADDR.to_string(),
        lp_token: MOCK_LP_TOKEN_ADDR.to_string(),
        treasury_contract: "treasury".to_string(),
        astroport_factory: MOCK_ASTRO_FACTORY_ADDR.to_string(),
        oracle_contract: MOCK_ORACLE_ADDR.to_string(),
        ust_bonding_reward_ratio: Decimal::from_str("0.6").unwrap(),
        ust_bonding_discount: Decimal::from_str("0.1").unwrap(),
        lp_bonding_discount: Decimal::from_str("0.05").unwrap(),
        min_bro_payout: Uint128::from(1u128),
        vesting_period_blocks: 10,
        lp_bonding_enabled: true,
    };

    let info = mock_info("addr0000", &[]);
    let _res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

    let msg = ExecuteMsg::Receive(Cw20ReceiveMsg {
        sender: "rewards".to_string(),
        amount: Uint128::from(100u128),
        msg: to_binary(&Cw20HookMsg::DistributeReward {}).unwrap(),
    });

    // error: unauthorized
    let info = mock_info("addr0001", &[]);
    let res = execute(deps.as_mut(), mock_env(), info, msg.clone());
    match res {
        Err(ContractError::Unauthorized {}) => assert_eq!(true, true),
        _ => panic!("DO NOT ENTER HERE"),
    }

    // proper execution
    let info = mock_info(MOCK_BRO_TOKEN_ADDR, &[]);
    let _res = execute(deps.as_mut(), mock_env(), info, msg.clone()).unwrap();

    assert_eq!(
        from_binary::<StateResponse>(
            &query(deps.as_ref(), mock_env(), QueryMsg::State {}).unwrap()
        )
        .unwrap(),
        StateResponse {
            ust_bonding_balance: Uint128::from(60u128),
            lp_bonding_balance: Uint128::from(40u128),
        },
    );
}

#[test]
fn lp_bond() {
    // setup initial balances for UST-BRO pool
    let mut deps = mock_dependencies(&[(
        MOCK_BRO_UST_PAIR_ADDR,
        &[Coin {
            denom: "uusd".to_string(),
            amount: Uint128::from(1000_000000u128),
        }],
    )]);

    deps.querier.with_token_balances(&[(
        &MOCK_BRO_TOKEN_ADDR.to_string(),
        &[(
            &MOCK_BRO_UST_PAIR_ADDR.to_string(),
            &Uint128::from(100_000000u128),
        )],
    )]);

    let msg = InstantiateMsg {
        owner: "owner".to_string(),
        bro_token: MOCK_BRO_TOKEN_ADDR.to_string(),
        lp_token: MOCK_LP_TOKEN_ADDR.to_string(),
        treasury_contract: "treasury".to_string(),
        astroport_factory: MOCK_ASTRO_FACTORY_ADDR.to_string(),
        oracle_contract: MOCK_ORACLE_ADDR.to_string(),
        ust_bonding_reward_ratio: Decimal::from_str("0.6").unwrap(),
        ust_bonding_discount: Decimal::from_str("0.1").unwrap(),
        lp_bonding_discount: Decimal::from_str("0.05").unwrap(),
        min_bro_payout: Uint128::from(3_000000u128),
        vesting_period_blocks: 10,
        lp_bonding_enabled: true,
    };

    let info = mock_info("addr0000", &[]);
    let _res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

    // distribute rewards
    let msg = ExecuteMsg::Receive(Cw20ReceiveMsg {
        sender: "rewards".to_string(),
        amount: Uint128::from(100_000000u128),
        msg: to_binary(&Cw20HookMsg::DistributeReward {}).unwrap(),
    });
    let info = mock_info(MOCK_BRO_TOKEN_ADDR, &[]);
    let _res = execute(deps.as_mut(), mock_env(), info, msg.clone()).unwrap();

    // error: unauthorized
    let msg = ExecuteMsg::Receive(Cw20ReceiveMsg {
        sender: "addr0000".to_string(),
        amount: Uint128::from(1_000_000000u128),
        msg: to_binary(&Cw20HookMsg::LpBond {}).unwrap(),
    });

    let info = mock_info("addr0001", &[]);
    let res = execute(deps.as_mut(), mock_env(), info, msg.clone());
    match res {
        Err(ContractError::Unauthorized {}) => assert_eq!(true, true),
        _ => panic!("DO NOT ENTER HERE"),
    }

    // error: bond payout is low
    let msg = ExecuteMsg::Receive(Cw20ReceiveMsg {
        sender: "addr0000".to_string(),
        amount: Uint128::from(10_000000u128),
        msg: to_binary(&Cw20HookMsg::LpBond {}).unwrap(),
    });
    let info = mock_info(MOCK_LP_TOKEN_ADDR, &[]);
    let res = execute(deps.as_mut(), mock_env(), info, msg.clone());
    match res {
        Err(ContractError::BondPayoutIsLow {}) => assert_eq!(true, true),
        _ => panic!("DO NOT ENTER HERE"),
    }

    // error: not enough balance for bond payout
    let msg = ExecuteMsg::Receive(Cw20ReceiveMsg {
        sender: "addr0000".to_string(),
        amount: Uint128::from(1_000_000000u128),
        msg: to_binary(&Cw20HookMsg::LpBond {}).unwrap(),
    });
    let info = mock_info(MOCK_LP_TOKEN_ADDR, &[]);
    let res = execute(deps.as_mut(), mock_env(), info, msg.clone());
    match res {
        Err(ContractError::NotEnoughForBondPayout {}) => assert_eq!(true, true),
        _ => panic!("DO NOT ENTER HERE"),
    }

    // proper execution
    // update min_bro_payout
    let msg = ExecuteMsg::UpdateConfig {
        owner: None,
        lp_token: None,
        treasury_contract: None,
        astroport_factory: None,
        oracle_contract: None,
        ust_bonding_reward_ratio: None,
        ust_bonding_discount: None,
        lp_bonding_discount: None,
        min_bro_payout: Some(Uint128::from(2_000000u128)),
        vesting_period_blocks: None,
        lp_bonding_enabled: None,
    };
    let info = mock_info("owner", &[]);
    let _res = execute(deps.as_mut(), mock_env(), info, msg.clone()).unwrap();

    // perform bond
    let msg = ExecuteMsg::Receive(Cw20ReceiveMsg {
        sender: "addr0000".to_string(),
        amount: Uint128::from(10_000000u128),
        msg: to_binary(&Cw20HookMsg::LpBond {}).unwrap(),
    });

    let info = mock_info(MOCK_LP_TOKEN_ADDR, &[]);
    let res = execute(deps.as_mut(), mock_env(), info, msg.clone()).unwrap();

    let lp_transfer_msg = res.messages.get(0).expect("no message");
    assert_eq!(
        lp_transfer_msg,
        &SubMsg::new(CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: MOCK_LP_TOKEN_ADDR.to_string(),
            funds: vec![],
            msg: to_binary(&Cw20ExecuteMsg::Transfer {
                recipient: "treasury".to_string(),
                amount: Uint128::from(10_000000u128),
            })
            .unwrap(),
        }))
    );

    let update_oracle_price_msg = res.messages.get(1).expect("no message");
    assert_eq!(
        update_oracle_price_msg,
        &SubMsg::new(CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: MOCK_ORACLE_ADDR.to_string(),
            funds: vec![],
            msg: to_binary(&OracleExecuteMsg::UpdatePrice {}).unwrap()
        }))
    );

    assert_eq!(
        from_binary::<StateResponse>(
            &query(deps.as_ref(), mock_env(), QueryMsg::State {}).unwrap()
        )
        .unwrap(),
        StateResponse {
            ust_bonding_balance: Uint128::from(60_000000u128),
            lp_bonding_balance: Uint128::from(37_900000u128),
        },
    );

    assert_eq!(
        from_binary::<ClaimsResponse>(
            &query(
                deps.as_ref(),
                mock_env(),
                QueryMsg::Claims {
                    address: "addr0000".to_string()
                }
            )
            .unwrap()
        )
        .unwrap(),
        ClaimsResponse {
            claims: vec![ClaimInfoResponse {
                bond_type: "lp_bond".to_string(),
                amount: Uint128::from(2_100000u128),
                claimable_at: Expiration::AtHeight(12_345 + 10),
            }],
        },
    );
}

#[test]
fn ust_bond() {
    // setup initial balances for UST-BRO pool
    let mut deps = mock_dependencies(&[(
        MOCK_BRO_UST_PAIR_ADDR,
        &[Coin {
            denom: "uusd".to_string(),
            amount: Uint128::from(1000_000000u128),
        }],
    )]);

    deps.querier.with_token_balances(&[(
        &MOCK_BRO_TOKEN_ADDR.to_string(),
        &[(
            &MOCK_BRO_UST_PAIR_ADDR.to_string(),
            &Uint128::from(100_000000u128),
        )],
    )]);

    let msg = InstantiateMsg {
        owner: "owner".to_string(),
        bro_token: MOCK_BRO_TOKEN_ADDR.to_string(),
        lp_token: MOCK_LP_TOKEN_ADDR.to_string(),
        treasury_contract: "treasury".to_string(),
        astroport_factory: MOCK_ASTRO_FACTORY_ADDR.to_string(),
        oracle_contract: MOCK_ORACLE_ADDR.to_string(),
        ust_bonding_reward_ratio: Decimal::from_str("0.6").unwrap(),
        ust_bonding_discount: Decimal::from_str("0.1").unwrap(),
        lp_bonding_discount: Decimal::from_str("0.05").unwrap(),
        min_bro_payout: Uint128::from(6_000000u128),
        vesting_period_blocks: 10,
        lp_bonding_enabled: true,
    };

    let info = mock_info("addr0000", &[]);
    let _res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

    // distribute rewards
    let msg = ExecuteMsg::Receive(Cw20ReceiveMsg {
        sender: "rewards".to_string(),
        amount: Uint128::from(100_000000u128),
        msg: to_binary(&Cw20HookMsg::DistributeReward {}).unwrap(),
    });
    let info = mock_info(MOCK_BRO_TOKEN_ADDR, &[]);
    let _res = execute(deps.as_mut(), mock_env(), info, msg.clone()).unwrap();

    // error: invalid funds input
    let msg = ExecuteMsg::UstBond {};
    let info = mock_info(
        "addr0000",
        &[
            Coin {
                denom: "uusd".to_string(),
                amount: Uint128::from(5000_000000u128),
            },
            Coin {
                denom: "uluna".to_string(),
                amount: Uint128::from(10_000000u128),
            },
        ],
    );
    let res = execute(deps.as_mut(), mock_env(), info, msg.clone());
    match res {
        Err(ContractError::InvalidFundsInput {}) => assert_eq!(true, true),
        _ => panic!("DO NOT ENTER HERE"),
    }

    let info = mock_info(
        "addr0000",
        &[Coin {
            denom: "uluna".to_string(),
            amount: Uint128::from(10_000000u128),
        }],
    );
    let res = execute(deps.as_mut(), mock_env(), info, msg.clone());
    match res {
        Err(ContractError::InvalidFundsInput {}) => assert_eq!(true, true),
        _ => panic!("DO NOT ENTER HERE"),
    }

    let info = mock_info(
        "addr0000",
        &[Coin {
            denom: "uusd".to_string(),
            amount: Uint128::zero(),
        }],
    );
    let res = execute(deps.as_mut(), mock_env(), info, msg.clone());
    match res {
        Err(ContractError::InvalidFundsInput {}) => assert_eq!(true, true),
        _ => panic!("DO NOT ENTER HERE"),
    }

    // error: bond payout is low
    let msg = ExecuteMsg::UstBond {};
    let info = mock_info(
        "addr0000",
        &[Coin {
            denom: "uusd".to_string(),
            amount: Uint128::from(50_000000u128),
        }],
    );
    let res = execute(deps.as_mut(), mock_env(), info, msg.clone());
    match res {
        Err(ContractError::BondPayoutIsLow {}) => assert_eq!(true, true),
        _ => panic!("DO NOT ENTER HERE"),
    }

    // error: not enough balance for bond payout
    let msg = ExecuteMsg::UstBond {};
    let info = mock_info(
        "addr0000",
        &[Coin {
            denom: "uusd".to_string(),
            amount: Uint128::from(5000_000000u128),
        }],
    );
    let res = execute(deps.as_mut(), mock_env(), info, msg.clone());
    match res {
        Err(ContractError::NotEnoughForBondPayout {}) => assert_eq!(true, true),
        _ => panic!("DO NOT ENTER HERE"),
    }

    // proper execution
    // update min_bro_payout
    let msg = ExecuteMsg::UpdateConfig {
        owner: None,
        lp_token: None,
        treasury_contract: None,
        astroport_factory: None,
        oracle_contract: None,
        ust_bonding_reward_ratio: None,
        ust_bonding_discount: None,
        lp_bonding_discount: None,
        min_bro_payout: Some(Uint128::from(5_000000u128)),
        vesting_period_blocks: None,
        lp_bonding_enabled: None,
    };
    let info = mock_info("owner", &[]);
    let _res = execute(deps.as_mut(), mock_env(), info, msg.clone()).unwrap();

    // perform bond
    let msg = ExecuteMsg::UstBond {};
    let info = mock_info(
        "addr0000",
        &[Coin {
            denom: "uusd".to_string(),
            amount: Uint128::from(50_000000u128),
        }],
    );
    let res = execute(deps.as_mut(), mock_env(), info, msg.clone()).unwrap();

    let ust_transfer_msg = res.messages.get(0).expect("no message");
    assert_eq!(
        ust_transfer_msg,
        &SubMsg::new(CosmosMsg::Bank(BankMsg::Send {
            to_address: "treasury".to_string(),
            amount: vec![Coin {
                denom: "uusd".to_string(),
                amount: Uint128::from(50_000000u128),
            }]
        })),
    );

    let update_oracle_price_msg = res.messages.get(1).expect("no message");
    assert_eq!(
        update_oracle_price_msg,
        &SubMsg::new(CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: MOCK_ORACLE_ADDR.to_string(),
            funds: vec![],
            msg: to_binary(&OracleExecuteMsg::UpdatePrice {}).unwrap()
        }))
    );

    assert_eq!(
        from_binary::<StateResponse>(
            &query(deps.as_ref(), mock_env(), QueryMsg::State {}).unwrap()
        )
        .unwrap(),
        StateResponse {
            ust_bonding_balance: Uint128::from(54_500000u128),
            lp_bonding_balance: Uint128::from(40_000000u128),
        },
    );

    assert_eq!(
        from_binary::<ClaimsResponse>(
            &query(
                deps.as_ref(),
                mock_env(),
                QueryMsg::Claims {
                    address: "addr0000".to_string()
                }
            )
            .unwrap()
        )
        .unwrap(),
        ClaimsResponse {
            claims: vec![ClaimInfoResponse {
                bond_type: "ust_bond".to_string(),
                amount: Uint128::from(5_500000u128),
                claimable_at: Expiration::AtHeight(12_345 + 10),
            }],
        },
    );
}

#[test]
fn claim() {
    let mut deps = mock_dependencies(&[]);

    let msg = InstantiateMsg {
        owner: "owner".to_string(),
        bro_token: MOCK_BRO_TOKEN_ADDR.to_string(),
        lp_token: MOCK_LP_TOKEN_ADDR.to_string(),
        treasury_contract: "treasury".to_string(),
        astroport_factory: MOCK_ASTRO_FACTORY_ADDR.to_string(),
        oracle_contract: MOCK_ORACLE_ADDR.to_string(),
        ust_bonding_reward_ratio: Decimal::from_str("0.6").unwrap(),
        ust_bonding_discount: Decimal::from_str("0.1").unwrap(),
        lp_bonding_discount: Decimal::from_str("0.05").unwrap(),
        min_bro_payout: Uint128::from(1u128),
        vesting_period_blocks: 10,
        lp_bonding_enabled: true,
    };

    let info = mock_info("addr0000", &[]);
    let _res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

    // store claims
    let addr_raw = deps.as_mut().api.addr_canonicalize("addr0000").unwrap();
    let mut claims = load_claims(deps.as_mut().storage, &addr_raw).unwrap();
    claims.push(ClaimInfo {
        bond_type: BondType::UstBond,
        amount: Uint128::from(10_000000u128),
        claimable_at: Expiration::AtHeight(12_355),
    });
    claims.push(ClaimInfo {
        bond_type: BondType::LpBond,
        amount: Uint128::from(20_000000u128),
        claimable_at: Expiration::AtHeight(12_370),
    });
    store_claims(deps.as_mut().storage, &addr_raw, &claims).unwrap();

    // error: nothing to claim
    let mut env = mock_env();

    let msg = ExecuteMsg::Claim {};
    let info = mock_info("addr0000", &[]);
    let res = execute(deps.as_mut(), env.clone(), info, msg.clone());
    match res {
        Err(ContractError::NothingToClaim {}) => assert_eq!(true, true),
        _ => panic!("DO NOT ENTER HERE"),
    }

    // proper execution
    // claim first bond
    env.block.height = 12355;

    let info = mock_info("addr0000", &[]);
    let res = execute(deps.as_mut(), env.clone(), info, msg.clone()).unwrap();

    let send_bro_msg = res.messages.get(0).expect("no message");
    assert_eq!(
        send_bro_msg,
        &SubMsg::new(CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: MOCK_BRO_TOKEN_ADDR.to_string(),
            funds: vec![],
            msg: to_binary(&Cw20ExecuteMsg::Transfer {
                recipient: "addr0000".to_string(),
                amount: Uint128::from(10_000000u128),
            })
            .unwrap()
        }))
    );

    // ust bond must be removed
    assert_eq!(
        from_binary::<ClaimsResponse>(
            &query(
                deps.as_ref(),
                mock_env(),
                QueryMsg::Claims {
                    address: "addr0000".to_string()
                }
            )
            .unwrap()
        )
        .unwrap(),
        ClaimsResponse {
            claims: vec![ClaimInfoResponse {
                bond_type: "lp_bond".to_string(),
                amount: Uint128::from(20_000000u128),
                claimable_at: Expiration::AtHeight(12_370),
            }],
        },
    );

    // claim second bond
    env.block.height = 12371;

    let info = mock_info("addr0000", &[]);
    let res = execute(deps.as_mut(), env.clone(), info, msg.clone()).unwrap();

    let send_bro_msg = res.messages.get(0).expect("no message");
    assert_eq!(
        send_bro_msg,
        &SubMsg::new(CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: MOCK_BRO_TOKEN_ADDR.to_string(),
            funds: vec![],
            msg: to_binary(&Cw20ExecuteMsg::Transfer {
                recipient: "addr0000".to_string(),
                amount: Uint128::from(20_000000u128),
            })
            .unwrap()
        }))
    );

    // claims must be empty
    assert_eq!(
        from_binary::<ClaimsResponse>(
            &query(
                deps.as_ref(),
                mock_env(),
                QueryMsg::Claims {
                    address: "addr0000".to_string()
                }
            )
            .unwrap()
        )
        .unwrap(),
        ClaimsResponse { claims: vec![] },
    );
}

#[test]
fn update_config() {
    let mut deps = mock_dependencies(&[]);

    let msg = InstantiateMsg {
        owner: "owner".to_string(),
        bro_token: MOCK_BRO_TOKEN_ADDR.to_string(),
        lp_token: MOCK_LP_TOKEN_ADDR.to_string(),
        treasury_contract: "treasury".to_string(),
        astroport_factory: MOCK_ASTRO_FACTORY_ADDR.to_string(),
        oracle_contract: MOCK_ORACLE_ADDR.to_string(),
        ust_bonding_reward_ratio: Decimal::from_str("0.6").unwrap(),
        ust_bonding_discount: Decimal::from_str("0.1").unwrap(),
        lp_bonding_discount: Decimal::from_str("0.05").unwrap(),
        min_bro_payout: Uint128::from(1u128),
        vesting_period_blocks: 10,
        lp_bonding_enabled: true,
    };

    let info = mock_info("addr0000", &[]);
    let _res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

    // unauthorized: only owner allowed to execute
    let msg = ExecuteMsg::UpdateConfig {
        owner: Some("new_owner".to_string()),
        lp_token: Some("new_lp".to_string()),
        treasury_contract: Some("new_treasury".to_string()),
        astroport_factory: Some("new_astro".to_string()),
        oracle_contract: Some("new_oracle".to_string()),
        ust_bonding_reward_ratio: Some(Decimal::from_str("0.61").unwrap()),
        ust_bonding_discount: Some(Decimal::from_str("0.11").unwrap()),
        lp_bonding_discount: Some(Decimal::from_str("0.06").unwrap()),
        min_bro_payout: Some(Uint128::from(2u128)),
        vesting_period_blocks: Some(11),
        lp_bonding_enabled: Some(false),
    };

    let info = mock_info("addr0000", &[]);
    let res = execute(deps.as_mut(), mock_env(), info, msg.clone());
    match res {
        Err(ContractError::Unauthorized {}) => assert_eq!(true, true),
        _ => panic!("DO NOT ENTER HERE"),
    }

    // proper execution
    let info = mock_info("owner", &[]);
    let _res = execute(deps.as_mut(), mock_env(), info, msg.clone()).unwrap();

    assert_eq!(
        from_binary::<ConfigResponse>(
            &query(deps.as_ref(), mock_env(), QueryMsg::Config {}).unwrap()
        )
        .unwrap(),
        ConfigResponse {
            owner: "new_owner".to_string(),
            bro_token: MOCK_BRO_TOKEN_ADDR.to_string(),
            lp_token: "new_lp".to_string(),
            treasury_contract: "new_treasury".to_string(),
            astroport_factory: "new_astro".to_string(),
            oracle_contract: "new_oracle".to_string(),
            ust_bonding_reward_ratio: Decimal::from_str("0.61").unwrap(),
            ust_bonding_discount: Decimal::from_str("0.11").unwrap(),
            lp_bonding_discount: Decimal::from_str("0.06").unwrap(),
            min_bro_payout: Uint128::from(2u128),
            vesting_period_blocks: 11,
            lp_bonding_enabled: false,
        },
    );
}
