use std::str::FromStr;

use crate::contract::{execute, instantiate, query};
use crate::error::ContractError;
use crate::state::{load_claims, store_claims, BondType, ClaimInfo};
use cosmwasm_std::testing::{mock_env, mock_info};
use cosmwasm_std::{
    from_binary, to_binary, Attribute, BankMsg, Coin, CosmosMsg, Decimal, StdError, SubMsg,
    Uint128, WasmMsg,
};
use cw20::{Cw20ExecuteMsg, Cw20ReceiveMsg, Expiration};
use services::bonding::SimulateExchangeResponse;

use crate::mock_querier::{
    mock_dependencies, MOCK_ASTRO_FACTORY_ADDR, MOCK_BRO_TOKEN_ADDR, MOCK_BRO_UST_PAIR_ADDR,
    MOCK_LP_TOKEN_ADDR, MOCK_ORACLE_ADDR, MOCK_STAKING_ADDR,
};

use services::{
    bonding::{
        BondingModeMsg, ClaimInfoResponse, ClaimsResponse, ConfigResponse, Cw20HookMsg, ExecuteMsg,
        InstantiateMsg, QueryMsg, StateResponse,
    },
    oracle::ExecuteMsg as OracleExecuteMsg,
    ownership_proposal::OwnershipProposalResponse,
    staking::Cw20HookMsg as StakingCw20HookMsg,
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

    // normal mode
    // invalid ust_bonding_reward_ratio
    let mut msg = InstantiateMsg {
        owner: "owner".to_string(),
        bro_token: MOCK_BRO_TOKEN_ADDR.to_string(),
        rewards_pool_contract: "rewards".to_string(),
        treasury_contract: "treasury".to_string(),
        astroport_factory: MOCK_ASTRO_FACTORY_ADDR.to_string(),
        oracle_contract: MOCK_ORACLE_ADDR.to_string(),
        ust_bonding_discount: Decimal::from_str("1.1").unwrap(),
        min_bro_payout: Uint128::from(1u128),
        bonding_mode: BondingModeMsg::Normal {
            ust_bonding_reward_ratio: Decimal::from_str("1.1").unwrap(),
            lp_token: MOCK_LP_TOKEN_ADDR.to_string(),
            lp_bonding_discount: Decimal::from_str("1.05").unwrap(),
            vesting_period_blocks: 10,
        },
    };

    let info = mock_info("addr0001", &[]);
    let res = instantiate(deps.as_mut(), mock_env(), info, msg.clone()).unwrap_err();
    match res {
        ContractError::Std(StdError::GenericErr { msg, .. }) => {
            assert_eq!(
                msg,
                "ust_bonding_discount must be less than 1.0 and non-negative".to_string()
            )
        }
        _ => panic!("DO NOT ENTER HERE"),
    }

    msg.ust_bonding_discount = Decimal::from_str("0.1").unwrap();
    let info = mock_info("addr0001", &[]);
    let res = instantiate(deps.as_mut(), mock_env(), info, msg.clone()).unwrap_err();
    match res {
        ContractError::Std(StdError::GenericErr { msg, .. }) => {
            assert_eq!(
                msg,
                "ust_bonding_reward_ratio must be less than 1.0 and non-negative".to_string()
            )
        }
        _ => panic!("DO NOT ENTER HERE"),
    }

    msg.bonding_mode = BondingModeMsg::Normal {
        ust_bonding_reward_ratio: Decimal::zero(),
        lp_token: MOCK_LP_TOKEN_ADDR.to_string(),
        lp_bonding_discount: Decimal::from_str("1.05").unwrap(),
        vesting_period_blocks: 10,
    };

    let info = mock_info("addr0001", &[]);
    let res = instantiate(deps.as_mut(), mock_env(), info, msg.clone()).unwrap_err();
    match res {
        ContractError::Std(StdError::GenericErr { msg, .. }) => {
            assert_eq!(
                msg,
                "ust_bonding_reward_ratio must be less than 1.0 and non-negative".to_string()
            )
        }
        _ => panic!("DO NOT ENTER HERE"),
    }

    msg.bonding_mode = BondingModeMsg::Normal {
        ust_bonding_reward_ratio: Decimal::from_str("0.6").unwrap(),
        lp_token: MOCK_LP_TOKEN_ADDR.to_string(),
        lp_bonding_discount: Decimal::from_str("1.05").unwrap(),
        vesting_period_blocks: 10,
    };

    let info = mock_info("addr0001", &[]);
    let res = instantiate(deps.as_mut(), mock_env(), info, msg.clone()).unwrap_err();
    match res {
        ContractError::Std(StdError::GenericErr { msg, .. }) => {
            assert_eq!(
                msg,
                "lp_bonding_discount must be less than 1.0 and non-negative".to_string()
            )
        }
        _ => panic!("DO NOT ENTER HERE"),
    }

    // proper initialization
    msg.bonding_mode = BondingModeMsg::Normal {
        ust_bonding_reward_ratio: Decimal::from_str("0.6").unwrap(),
        lp_token: MOCK_LP_TOKEN_ADDR.to_string(),
        lp_bonding_discount: Decimal::from_str("0.05").unwrap(),
        vesting_period_blocks: 10,
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
            bro_token: MOCK_BRO_TOKEN_ADDR.to_string(),
            rewards_pool_contract: "rewards".to_string(),
            treasury_contract: "treasury".to_string(),
            astroport_factory: MOCK_ASTRO_FACTORY_ADDR.to_string(),
            oracle_contract: MOCK_ORACLE_ADDR.to_string(),
            ust_bonding_discount: Decimal::from_str("0.1").unwrap(),
            min_bro_payout: Uint128::from(1u128),
            bonding_mode: BondingModeMsg::Normal {
                ust_bonding_reward_ratio: Decimal::from_str("0.6").unwrap(),
                lp_token: MOCK_LP_TOKEN_ADDR.to_string(),
                lp_bonding_discount: Decimal::from_str("0.05").unwrap(),
                vesting_period_blocks: 10,
            },
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

    // community mode
    // invalid epochs locked
    let msg = InstantiateMsg {
        owner: "owner".to_string(),
        bro_token: MOCK_BRO_TOKEN_ADDR.to_string(),
        rewards_pool_contract: "rewards".to_string(),
        treasury_contract: "treasury".to_string(),
        astroport_factory: MOCK_ASTRO_FACTORY_ADDR.to_string(),
        oracle_contract: MOCK_ORACLE_ADDR.to_string(),
        ust_bonding_discount: Decimal::from_str("0.1").unwrap(),
        min_bro_payout: Uint128::from(1u128),
        bonding_mode: BondingModeMsg::Community {
            staking_contract: MOCK_STAKING_ADDR.to_string(),
            epochs_locked: 800,
        },
    };

    let info = mock_info("addr0001", &[]);
    let res = instantiate(deps.as_mut(), mock_env(), info, msg.clone());
    match res {
        Err(ContractError::InvalidLockupPeriodForCommunityBondingMode {}) => (),
        _ => panic!("DO NOT ENTER HERE!"),
    }

    // proper initialization
    let msg = InstantiateMsg {
        owner: "owner".to_string(),
        bro_token: MOCK_BRO_TOKEN_ADDR.to_string(),
        rewards_pool_contract: "rewards".to_string(),
        treasury_contract: "treasury".to_string(),
        astroport_factory: MOCK_ASTRO_FACTORY_ADDR.to_string(),
        oracle_contract: MOCK_ORACLE_ADDR.to_string(),
        ust_bonding_discount: Decimal::from_str("0.1").unwrap(),
        min_bro_payout: Uint128::from(1u128),
        bonding_mode: BondingModeMsg::Community {
            staking_contract: MOCK_STAKING_ADDR.to_string(),
            epochs_locked: 500,
        },
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
            bro_token: MOCK_BRO_TOKEN_ADDR.to_string(),
            rewards_pool_contract: "rewards".to_string(),
            treasury_contract: "treasury".to_string(),
            astroport_factory: MOCK_ASTRO_FACTORY_ADDR.to_string(),
            oracle_contract: MOCK_ORACLE_ADDR.to_string(),
            ust_bonding_discount: Decimal::from_str("0.1").unwrap(),
            min_bro_payout: Uint128::from(1u128),
            bonding_mode: BondingModeMsg::Community {
                staking_contract: MOCK_STAKING_ADDR.to_string(),
                epochs_locked: 500,
            },
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
        rewards_pool_contract: "rewards".to_string(),
        treasury_contract: "treasury".to_string(),
        astroport_factory: MOCK_ASTRO_FACTORY_ADDR.to_string(),
        oracle_contract: MOCK_ORACLE_ADDR.to_string(),
        ust_bonding_discount: Decimal::from_str("0.1").unwrap(),
        min_bro_payout: Uint128::from(1u128),
        bonding_mode: BondingModeMsg::Normal {
            ust_bonding_reward_ratio: Decimal::from_str("0.6").unwrap(),
            lp_token: MOCK_LP_TOKEN_ADDR.to_string(),
            lp_bonding_discount: Decimal::from_str("0.05").unwrap(),
            vesting_period_blocks: 10,
        },
    };

    let info = mock_info("addr0000", &[]);
    let _res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

    let msg = ExecuteMsg::Receive(Cw20ReceiveMsg {
        sender: "addr0004".to_string(),
        amount: Uint128::from(100u128),
        msg: to_binary(&Cw20HookMsg::DistributeReward {}).unwrap(),
    });

    // error: unauthorized (info.sender must be bro token addr)
    let info = mock_info("addr0001", &[]);
    let res = execute(deps.as_mut(), mock_env(), info, msg.clone());
    match res {
        Err(ContractError::Unauthorized {}) => assert_eq!(true, true),
        _ => panic!("DO NOT ENTER HERE"),
    }

    // error: unauthorized (cw20_msg.sender must be rewards pool)
    let info = mock_info(MOCK_BRO_TOKEN_ADDR, &[]);
    let res = execute(deps.as_mut(), mock_env(), info, msg.clone());
    match res {
        Err(ContractError::Unauthorized {}) => assert_eq!(true, true),
        _ => panic!("DO NOT ENTER HERE"),
    }

    // // normal type distribution
    let msg = ExecuteMsg::Receive(Cw20ReceiveMsg {
        sender: "rewards".to_string(),
        amount: Uint128::from(100u128),
        msg: to_binary(&Cw20HookMsg::DistributeReward {}).unwrap(),
    });

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

    // community type distribution
    let mut deps = mock_dependencies(&[]);
    let msg = InstantiateMsg {
        owner: "owner".to_string(),
        bro_token: MOCK_BRO_TOKEN_ADDR.to_string(),
        rewards_pool_contract: "rewards".to_string(),
        treasury_contract: "treasury".to_string(),
        astroport_factory: MOCK_ASTRO_FACTORY_ADDR.to_string(),
        oracle_contract: MOCK_ORACLE_ADDR.to_string(),
        ust_bonding_discount: Decimal::from_str("0.1").unwrap(),
        min_bro_payout: Uint128::from(1u128),
        bonding_mode: BondingModeMsg::Community {
            staking_contract: MOCK_STAKING_ADDR.to_string(),
            epochs_locked: 10,
        },
    };

    let info = mock_info("addr0000", &[]);
    let _res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

    let msg = ExecuteMsg::Receive(Cw20ReceiveMsg {
        sender: "rewards".to_string(),
        amount: Uint128::from(100u128),
        msg: to_binary(&Cw20HookMsg::DistributeReward {}).unwrap(),
    });

    let info = mock_info(MOCK_BRO_TOKEN_ADDR, &[]);
    let _res = execute(deps.as_mut(), mock_env(), info, msg.clone()).unwrap();

    assert_eq!(
        from_binary::<StateResponse>(
            &query(deps.as_ref(), mock_env(), QueryMsg::State {}).unwrap()
        )
        .unwrap(),
        StateResponse {
            ust_bonding_balance: Uint128::from(100u128),
            lp_bonding_balance: Uint128::zero(),
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
        rewards_pool_contract: "rewards".to_string(),
        treasury_contract: "treasury".to_string(),
        astroport_factory: MOCK_ASTRO_FACTORY_ADDR.to_string(),
        oracle_contract: MOCK_ORACLE_ADDR.to_string(),
        ust_bonding_discount: Decimal::from_str("0.1").unwrap(),
        min_bro_payout: Uint128::from(3_000000u128),
        bonding_mode: BondingModeMsg::Normal {
            ust_bonding_reward_ratio: Decimal::from_str("0.6").unwrap(),
            lp_token: MOCK_LP_TOKEN_ADDR.to_string(),
            lp_bonding_discount: Decimal::from_str("0.05").unwrap(),
            vesting_period_blocks: 10,
        },
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
        Err(ContractError::BondPayoutIsTooLow {}) => assert_eq!(true, true),
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
        min_bro_payout: Some(Uint128::from(2_000000u128)),
        rewards_pool_contract: None,
        treasury_contract: None,
        astroport_factory: None,
        oracle_contract: None,
        ust_bonding_discount: None,
    };
    let info = mock_info("owner", &[]);
    let _res = execute(deps.as_mut(), mock_env(), info, msg.clone()).unwrap();

    // simulate lp bond
    assert_eq!(
        from_binary::<SimulateExchangeResponse>(
            &query(
                deps.as_ref(),
                mock_env(),
                QueryMsg::SimulateLpBond {
                    lp_amount: Uint128::from(10_000000u128),
                }
            )
            .unwrap()
        )
        .unwrap(),
        SimulateExchangeResponse {
            bro_payout: Uint128::from(2_100000u128),
            can_be_exchanged: true,
        },
    );

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

    // try to bond lp with community mode
    let msg = InstantiateMsg {
        owner: "owner".to_string(),
        bro_token: MOCK_BRO_TOKEN_ADDR.to_string(),
        rewards_pool_contract: "rewards".to_string(),
        treasury_contract: "treasury".to_string(),
        astroport_factory: MOCK_ASTRO_FACTORY_ADDR.to_string(),
        oracle_contract: MOCK_ORACLE_ADDR.to_string(),
        ust_bonding_discount: Decimal::from_str("0.1").unwrap(),
        min_bro_payout: Uint128::from(3_000000u128),
        bonding_mode: BondingModeMsg::Community {
            staking_contract: MOCK_STAKING_ADDR.to_string(),
            epochs_locked: 10,
        },
    };

    let info = mock_info("addr0000", &[]);
    let _res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

    // error: lp bonding disabled
    let msg = ExecuteMsg::Receive(Cw20ReceiveMsg {
        sender: "addr0000".to_string(),
        amount: Uint128::from(10_000000u128),
        msg: to_binary(&Cw20HookMsg::LpBond {}).unwrap(),
    });
    let info = mock_info(MOCK_LP_TOKEN_ADDR, &[]);
    let res = execute(deps.as_mut(), mock_env(), info, msg.clone());
    match res {
        Err(ContractError::LpBondingDisabled {}) => assert!(true),
        _ => panic!("DO NOT ENTER HERE"),
    }
}

#[test]
fn ust_bond_normal_mode() {
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
        rewards_pool_contract: "rewards".to_string(),
        treasury_contract: "treasury".to_string(),
        astroport_factory: MOCK_ASTRO_FACTORY_ADDR.to_string(),
        oracle_contract: MOCK_ORACLE_ADDR.to_string(),
        ust_bonding_discount: Decimal::from_str("0.1").unwrap(),
        min_bro_payout: Uint128::from(6_000000u128),
        bonding_mode: BondingModeMsg::Normal {
            ust_bonding_reward_ratio: Decimal::from_str("0.6").unwrap(),
            lp_token: MOCK_LP_TOKEN_ADDR.to_string(),
            lp_bonding_discount: Decimal::from_str("0.05").unwrap(),
            vesting_period_blocks: 10,
        },
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
        Err(ContractError::BondPayoutIsTooLow {}) => assert_eq!(true, true),
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
        min_bro_payout: Some(Uint128::from(5_000000u128)),
        rewards_pool_contract: None,
        treasury_contract: None,
        astroport_factory: None,
        oracle_contract: None,
        ust_bonding_discount: None,
    };
    let info = mock_info("owner", &[]);
    let _res = execute(deps.as_mut(), mock_env(), info, msg.clone()).unwrap();

    // simulate ust bond
    assert_eq!(
        from_binary::<SimulateExchangeResponse>(
            &query(
                deps.as_ref(),
                mock_env(),
                QueryMsg::SimulateUstBond {
                    uusd_amount: Uint128::from(50_000000u128),
                }
            )
            .unwrap()
        )
        .unwrap(),
        SimulateExchangeResponse {
            bro_payout: Uint128::from(5_500000u128),
            can_be_exchanged: true,
        },
    );

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
fn ust_bond_community_mode() {
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
        rewards_pool_contract: "rewards".to_string(),
        treasury_contract: "treasury".to_string(),
        astroport_factory: MOCK_ASTRO_FACTORY_ADDR.to_string(),
        oracle_contract: MOCK_ORACLE_ADDR.to_string(),
        ust_bonding_discount: Decimal::from_str("0.1").unwrap(),
        min_bro_payout: Uint128::from(1_000000u128),
        bonding_mode: BondingModeMsg::Community {
            staking_contract: MOCK_STAKING_ADDR.to_string(),
            epochs_locked: 100,
        },
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

    let stake_bonded_tokens_msg = res.messages.get(2).expect("no message");
    assert_eq!(
        stake_bonded_tokens_msg,
        &SubMsg::new(CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: MOCK_BRO_TOKEN_ADDR.to_string(),
            funds: vec![],
            msg: to_binary(&Cw20ExecuteMsg::Send {
                contract: MOCK_STAKING_ADDR.to_string(),
                amount: Uint128::from(5_500000u128),
                msg: to_binary(&StakingCw20HookMsg::CommunityBondStake {
                    sender: "addr0000".to_string(),
                    epochs_locked: 100,
                })
                .unwrap(),
            })
            .unwrap(),
        }))
    );

    assert_eq!(
        from_binary::<StateResponse>(
            &query(deps.as_ref(), mock_env(), QueryMsg::State {}).unwrap()
        )
        .unwrap(),
        StateResponse {
            ust_bonding_balance: Uint128::from(94_500000u128),
            lp_bonding_balance: Uint128::zero(),
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
        ClaimsResponse { claims: vec![] },
    );

    // error: simulate lp bond is disabled
    assert_eq!(
        query(
            deps.as_ref(),
            mock_env(),
            QueryMsg::SimulateLpBond {
                lp_amount: Uint128::from(10_000000u128),
            }
        )
        .unwrap_err(),
        StdError::generic_err("LP Token bonding disabled"),
    );
}

#[test]
fn claim() {
    let mut deps = mock_dependencies(&[]);

    let msg = InstantiateMsg {
        owner: "owner".to_string(),
        bro_token: MOCK_BRO_TOKEN_ADDR.to_string(),
        rewards_pool_contract: "rewards".to_string(),
        treasury_contract: "treasury".to_string(),
        astroport_factory: MOCK_ASTRO_FACTORY_ADDR.to_string(),
        oracle_contract: MOCK_ORACLE_ADDR.to_string(),
        ust_bonding_discount: Decimal::from_str("0.1").unwrap(),
        min_bro_payout: Uint128::from(1u128),
        bonding_mode: BondingModeMsg::Normal {
            ust_bonding_reward_ratio: Decimal::from_str("0.6").unwrap(),
            lp_token: MOCK_LP_TOKEN_ADDR.to_string(),
            lp_bonding_discount: Decimal::from_str("0.05").unwrap(),
            vesting_period_blocks: 10,
        },
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
        rewards_pool_contract: "rewards".to_string(),
        treasury_contract: "treasury".to_string(),
        astroport_factory: MOCK_ASTRO_FACTORY_ADDR.to_string(),
        oracle_contract: MOCK_ORACLE_ADDR.to_string(),
        ust_bonding_discount: Decimal::from_str("0.1").unwrap(),
        min_bro_payout: Uint128::from(1u128),
        bonding_mode: BondingModeMsg::Normal {
            ust_bonding_reward_ratio: Decimal::from_str("0.6").unwrap(),
            lp_token: MOCK_LP_TOKEN_ADDR.to_string(),
            lp_bonding_discount: Decimal::from_str("0.05").unwrap(),
            vesting_period_blocks: 10,
        },
    };

    let info = mock_info("addr0000", &[]);
    let _res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

    // unauthorized: only owner allowed to execute
    let msg = ExecuteMsg::UpdateConfig {
        rewards_pool_contract: Some("new_rewards".to_string()),
        treasury_contract: Some("new_treasury".to_string()),
        astroport_factory: Some("new_astro".to_string()),
        oracle_contract: Some("new_oracle".to_string()),
        ust_bonding_discount: Some(Decimal::from_str("1.1").unwrap()),
        min_bro_payout: Some(Uint128::from(2u128)),
    };

    let info = mock_info("addr0000", &[]);
    let res = execute(deps.as_mut(), mock_env(), info, msg.clone());
    match res {
        Err(ContractError::Unauthorized {}) => assert_eq!(true, true),
        _ => panic!("DO NOT ENTER HERE"),
    }

    // error: invalid ust_bonding_discount
    let info = mock_info("owner", &[]);
    let res = execute(deps.as_mut(), mock_env(), info, msg.clone()).unwrap_err();
    match res {
        ContractError::Std(StdError::GenericErr { msg, .. }) => {
            assert_eq!(
                msg,
                "ust_bonding_discount must be less than 1.0 and non-negative".to_string()
            )
        }
        _ => panic!("DO NOT ENTER HERE"),
    }

    // proper execution
    let msg = ExecuteMsg::UpdateConfig {
        rewards_pool_contract: Some("new_rewards".to_string()),
        treasury_contract: Some("new_treasury".to_string()),
        astroport_factory: Some("new_astro".to_string()),
        oracle_contract: Some("new_oracle".to_string()),
        ust_bonding_discount: Some(Decimal::from_str("0.11").unwrap()),
        min_bro_payout: Some(Uint128::from(2u128)),
    };

    let info = mock_info("owner", &[]);
    let res = execute(deps.as_mut(), mock_env(), info, msg.clone()).unwrap();

    assert_eq!(res.attributes[0], Attribute::new("action", "update_config"));
    assert_eq!(
        res.attributes[1],
        Attribute::new("rewards_pool_contract_changed", "new_rewards")
    );
    assert_eq!(
        res.attributes[2],
        Attribute::new("treasury_contract_changed", "new_treasury")
    );
    assert_eq!(
        res.attributes[3],
        Attribute::new("astroport_factory_changed", "new_astro")
    );
    assert_eq!(
        res.attributes[4],
        Attribute::new("oracle_contract_changed", "new_oracle")
    );
    assert_eq!(
        res.attributes[5],
        Attribute::new("ust_bonding_discount_changed", "0.11")
    );
    assert_eq!(
        res.attributes[6],
        Attribute::new("min_bro_payout_changed", "2")
    );

    assert_eq!(
        from_binary::<ConfigResponse>(
            &query(deps.as_ref(), mock_env(), QueryMsg::Config {}).unwrap()
        )
        .unwrap(),
        ConfigResponse {
            owner: "owner".to_string(),
            bro_token: MOCK_BRO_TOKEN_ADDR.to_string(),
            rewards_pool_contract: "new_rewards".to_string(),
            treasury_contract: "new_treasury".to_string(),
            astroport_factory: "new_astro".to_string(),
            oracle_contract: "new_oracle".to_string(),
            ust_bonding_discount: Decimal::from_str("0.11").unwrap(),
            min_bro_payout: Uint128::from(2u128),
            bonding_mode: BondingModeMsg::Normal {
                ust_bonding_reward_ratio: Decimal::from_str("0.6").unwrap(),
                lp_token: MOCK_LP_TOKEN_ADDR.to_string(),
                lp_bonding_discount: Decimal::from_str("0.05").unwrap(),
                vesting_period_blocks: 10,
            }
        },
    );
}

#[test]
fn update_bonding_mode_config() {
    // noramal mode
    let mut deps = mock_dependencies(&[]);

    let msg = InstantiateMsg {
        owner: "owner".to_string(),
        bro_token: MOCK_BRO_TOKEN_ADDR.to_string(),
        rewards_pool_contract: "rewards".to_string(),
        treasury_contract: "treasury".to_string(),
        astroport_factory: MOCK_ASTRO_FACTORY_ADDR.to_string(),
        oracle_contract: MOCK_ORACLE_ADDR.to_string(),
        ust_bonding_discount: Decimal::from_str("0.1").unwrap(),
        min_bro_payout: Uint128::from(1u128),
        bonding_mode: BondingModeMsg::Normal {
            ust_bonding_reward_ratio: Decimal::from_str("0.6").unwrap(),
            lp_token: MOCK_LP_TOKEN_ADDR.to_string(),
            lp_bonding_discount: Decimal::from_str("0.05").unwrap(),
            vesting_period_blocks: 10,
        },
    };
    let info = mock_info("addr0000", &[]);
    let _res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

    // error: unauthorized
    let msg = ExecuteMsg::UpdateBondingModeConfig {
        ust_bonding_reward_ratio_normal: None,
        lp_token_normal: None,
        lp_bonding_discount_normal: None,
        vesting_period_blocks_normal: None,
        staking_contract_community: None,
        epochs_locked_community: None,
    };
    let info = mock_info("addr000", &[]);
    let res = execute(deps.as_mut(), mock_env(), info, msg.clone());
    match res {
        Err(ContractError::Unauthorized {}) => assert_eq!(true, true),
        _ => panic!("DO NOT ENTER HERE"),
    }

    // error: invalid ust_bonding_reward_ratio
    let msg = ExecuteMsg::UpdateBondingModeConfig {
        ust_bonding_reward_ratio_normal: Some(Decimal::from_str("1.1").unwrap()),
        lp_token_normal: None,
        lp_bonding_discount_normal: None,
        vesting_period_blocks_normal: None,
        staking_contract_community: None,
        epochs_locked_community: None,
    };
    let info = mock_info("owner", &[]);
    let res = execute(deps.as_mut(), mock_env(), info, msg.clone()).unwrap_err();
    match res {
        ContractError::Std(StdError::GenericErr { msg, .. }) => {
            assert_eq!(
                msg,
                "ust_bonding_reward_ratio must be less than 1.0 and non-negative".to_string()
            )
        }
        _ => panic!("DO NOT ENTER HERE"),
    }

    // error: invalid lp_bonding_discount
    let msg = ExecuteMsg::UpdateBondingModeConfig {
        ust_bonding_reward_ratio_normal: None,
        lp_token_normal: None,
        lp_bonding_discount_normal: Some(Decimal::from_str("1.1").unwrap()),
        vesting_period_blocks_normal: None,
        staking_contract_community: None,
        epochs_locked_community: None,
    };
    let info = mock_info("owner", &[]);
    let res = execute(deps.as_mut(), mock_env(), info, msg.clone()).unwrap_err();
    match res {
        ContractError::Std(StdError::GenericErr { msg, .. }) => {
            assert_eq!(
                msg,
                "lp_bonding_discount must be less than 1.0 and non-negative".to_string()
            )
        }
        _ => panic!("DO NOT ENTER HERE"),
    }

    // error: invalid vesting_period_blocks
    let msg = ExecuteMsg::UpdateBondingModeConfig {
        ust_bonding_reward_ratio_normal: None,
        lp_token_normal: None,
        lp_bonding_discount_normal: None,
        vesting_period_blocks_normal: Some(0),
        staking_contract_community: None,
        epochs_locked_community: None,
    };
    let info = mock_info("owner", &[]);
    let res = execute(deps.as_mut(), mock_env(), info, msg.clone()).unwrap_err();
    match res {
        ContractError::Std(StdError::GenericErr { msg, .. }) => {
            assert_eq!(
                msg,
                "vesting_period_blocks must be greater than zero".to_string()
            )
        }
        _ => panic!("DO NOT ENTER HERE"),
    }

    // proper execution
    let msg = ExecuteMsg::UpdateBondingModeConfig {
        ust_bonding_reward_ratio_normal: Some(Decimal::from_str("0.5").unwrap()),
        lp_token_normal: Some("new_lp_token".to_string()),
        lp_bonding_discount_normal: Some(Decimal::from_str("0.06").unwrap()),
        vesting_period_blocks_normal: Some(11),
        staking_contract_community: None,
        epochs_locked_community: None,
    };

    let info = mock_info("owner", &[]);
    let res = execute(deps.as_mut(), mock_env(), info, msg.clone()).unwrap();

    assert_eq!(
        res.attributes[0],
        Attribute::new("action", "update_bonding_mode_config")
    );
    assert_eq!(
        res.attributes[1],
        Attribute::new("ust_bonding_reward_ratio_changed", "0.5")
    );
    assert_eq!(
        res.attributes[2],
        Attribute::new("lp_token_changed", "new_lp_token")
    );
    assert_eq!(
        res.attributes[3],
        Attribute::new("lp_bonding_discount_changed", "0.06")
    );
    assert_eq!(
        res.attributes[4],
        Attribute::new("vesting_period_blocks_changed", "11")
    );

    assert_eq!(
        from_binary::<ConfigResponse>(
            &query(deps.as_ref(), mock_env(), QueryMsg::Config {}).unwrap()
        )
        .unwrap(),
        ConfigResponse {
            owner: "owner".to_string(),
            bro_token: MOCK_BRO_TOKEN_ADDR.to_string(),
            rewards_pool_contract: "rewards".to_string(),
            treasury_contract: "treasury".to_string(),
            astroport_factory: MOCK_ASTRO_FACTORY_ADDR.to_string(),
            oracle_contract: MOCK_ORACLE_ADDR.to_string(),
            ust_bonding_discount: Decimal::from_str("0.1").unwrap(),
            min_bro_payout: Uint128::from(1u128),
            bonding_mode: BondingModeMsg::Normal {
                ust_bonding_reward_ratio: Decimal::from_str("0.5").unwrap(),
                lp_token: "new_lp_token".to_string(),
                lp_bonding_discount: Decimal::from_str("0.06").unwrap(),
                vesting_period_blocks: 11,
            }
        },
    );

    // community mode
    let msg = InstantiateMsg {
        owner: "owner".to_string(),
        bro_token: MOCK_BRO_TOKEN_ADDR.to_string(),
        rewards_pool_contract: "rewards".to_string(),
        treasury_contract: "treasury".to_string(),
        astroport_factory: MOCK_ASTRO_FACTORY_ADDR.to_string(),
        oracle_contract: MOCK_ORACLE_ADDR.to_string(),
        ust_bonding_discount: Decimal::from_str("0.1").unwrap(),
        min_bro_payout: Uint128::from(1u128),
        bonding_mode: BondingModeMsg::Community {
            staking_contract: MOCK_STAKING_ADDR.to_string(),
            epochs_locked: 10,
        },
    };

    let info = mock_info("addr0000", &[]);
    let _res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

    // error: invalid epochs_locked
    let msg = ExecuteMsg::UpdateBondingModeConfig {
        ust_bonding_reward_ratio_normal: None,
        lp_token_normal: None,
        lp_bonding_discount_normal: None,
        vesting_period_blocks_normal: None,
        staking_contract_community: None,
        epochs_locked_community: Some(1),
    };
    let info = mock_info("owner", &[]);
    let res = execute(deps.as_mut(), mock_env(), info, msg.clone());
    match res {
        Err(ContractError::InvalidLockupPeriodForCommunityBondingMode {}) => assert_eq!(true, true),
        _ => panic!("DO NOT ENTER HERE"),
    }

    // proper execution
    let msg = ExecuteMsg::UpdateBondingModeConfig {
        ust_bonding_reward_ratio_normal: None,
        lp_token_normal: None,
        lp_bonding_discount_normal: None,
        vesting_period_blocks_normal: None,
        staking_contract_community: None,
        epochs_locked_community: Some(11),
    };

    let info = mock_info("owner", &[]);
    let res = execute(deps.as_mut(), mock_env(), info, msg.clone()).unwrap();

    assert_eq!(
        res.attributes[0],
        Attribute::new("action", "update_bonding_mode_config")
    );
    assert_eq!(
        res.attributes[1],
        Attribute::new("epochs_locked_changed", "11")
    );

    assert_eq!(
        from_binary::<ConfigResponse>(
            &query(deps.as_ref(), mock_env(), QueryMsg::Config {}).unwrap()
        )
        .unwrap(),
        ConfigResponse {
            owner: "owner".to_string(),
            bro_token: MOCK_BRO_TOKEN_ADDR.to_string(),
            rewards_pool_contract: "rewards".to_string(),
            treasury_contract: "treasury".to_string(),
            astroport_factory: MOCK_ASTRO_FACTORY_ADDR.to_string(),
            oracle_contract: MOCK_ORACLE_ADDR.to_string(),
            ust_bonding_discount: Decimal::from_str("0.1").unwrap(),
            min_bro_payout: Uint128::from(1u128),
            bonding_mode: BondingModeMsg::Community {
                staking_contract: MOCK_STAKING_ADDR.to_string(),
                epochs_locked: 11,
            }
        },
    );
}

#[test]
fn update_owner() {
    let mut deps = mock_dependencies(&[]);

    let msg = InstantiateMsg {
        owner: "owner0000".to_string(),
        bro_token: MOCK_BRO_TOKEN_ADDR.to_string(),
        rewards_pool_contract: "rewards".to_string(),
        treasury_contract: "treasury".to_string(),
        astroport_factory: MOCK_ASTRO_FACTORY_ADDR.to_string(),
        oracle_contract: MOCK_ORACLE_ADDR.to_string(),
        ust_bonding_discount: Decimal::from_str("0.1").unwrap(),
        min_bro_payout: Uint128::from(1u128),
        bonding_mode: BondingModeMsg::Normal {
            ust_bonding_reward_ratio: Decimal::from_str("0.6").unwrap(),
            lp_token: MOCK_LP_TOKEN_ADDR.to_string(),
            lp_bonding_discount: Decimal::from_str("0.05").unwrap(),
            vesting_period_blocks: 10,
        },
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
            bro_token: MOCK_BRO_TOKEN_ADDR.to_string(),
            rewards_pool_contract: "rewards".to_string(),
            treasury_contract: "treasury".to_string(),
            astroport_factory: MOCK_ASTRO_FACTORY_ADDR.to_string(),
            oracle_contract: MOCK_ORACLE_ADDR.to_string(),
            ust_bonding_discount: Decimal::from_str("0.1").unwrap(),
            min_bro_payout: Uint128::from(1u128),
            bonding_mode: BondingModeMsg::Normal {
                ust_bonding_reward_ratio: Decimal::from_str("0.6").unwrap(),
                lp_token: MOCK_LP_TOKEN_ADDR.to_string(),
                lp_bonding_discount: Decimal::from_str("0.05").unwrap(),
                vesting_period_blocks: 10,
            }
        },
    );
}
