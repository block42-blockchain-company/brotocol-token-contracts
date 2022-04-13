use crate::contract::{execute, instantiate, query};
use crate::error::ContractError;
use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info};
use cosmwasm_std::{from_binary, to_binary, Attribute, CosmosMsg, SubMsg, Uint128, WasmMsg};
use cw20::{Cw20ExecuteMsg, Expiration};

use services::ownership_proposal::OwnershipProposalResponse;
use services::rewards::{
    ConfigResponse, DistributeRewardMsg, ExecuteMsg, InstantiateMsg, QueryMsg,
};

#[test]
fn proper_initialization() {
    let mut deps = mock_dependencies(&[]);

    let msg = InstantiateMsg {
        owner: "gov".to_string(),
        bro_token: "bro".to_string(),
        spend_limit: Uint128::from(1000000u128),
        whitelist: vec!["distr0000".to_string()],
    };

    let info = mock_info("addr0000", &[]);
    let _res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

    let config: ConfigResponse =
        from_binary(&query(deps.as_ref(), mock_env(), QueryMsg::Config {}).unwrap()).unwrap();
    assert_eq!("gov", config.owner.as_str());
    assert_eq!("bro", config.bro_token.as_str());
    assert_eq!(Uint128::from(1000000u128), config.spend_limit);
    assert_eq!(vec!["distr0000".to_string()], config.whitelist);
}

#[test]
fn update_config() {
    let mut deps = mock_dependencies(&[]);

    let msg = InstantiateMsg {
        owner: "gov".to_string(),
        bro_token: "bro".to_string(),
        spend_limit: Uint128::from(1000000u128),
        whitelist: vec!["distr0000".to_string()],
    };

    let info = mock_info("addr0000", &[]);
    let _res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

    let config: ConfigResponse =
        from_binary(&query(deps.as_ref(), mock_env(), QueryMsg::Config {}).unwrap()).unwrap();
    assert_eq!("gov", config.owner.as_str());
    assert_eq!("bro", config.bro_token.as_str());
    assert_eq!(Uint128::from(1000000u128), config.spend_limit);

    // update spend_limit and owner addr
    let msg = ExecuteMsg::UpdateConfig {
        spend_limit: Some(Uint128::from(500000u128)),
    };
    let info = mock_info("addr0000", &[]);
    let res = execute(deps.as_mut(), mock_env(), info, msg.clone());

    // error: Unauthorized
    match res {
        Err(ContractError::Unauthorized {}) => assert_eq!(true, true),
        _ => panic!("DO NOT ENTER HERE"),
    }

    // proper execution
    let info = mock_info("gov", &[]);
    let res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();

    assert_eq!(res.attributes[0], Attribute::new("action", "update_config"));
    assert_eq!(
        res.attributes[1],
        Attribute::new("spend_limit_changed", "500000")
    );

    assert_eq!(
        from_binary::<ConfigResponse>(
            &query(deps.as_ref(), mock_env(), QueryMsg::Config {}).unwrap(),
        )
        .unwrap(),
        ConfigResponse {
            owner: "gov".to_string(),
            bro_token: "bro".to_string(),
            spend_limit: Uint128::from(500000u128),
            whitelist: vec!["distr0000".to_string()],
        }
    );
}

#[test]
fn add_distributor() {
    let mut deps = mock_dependencies(&[]);

    let msg = InstantiateMsg {
        owner: "gov".to_string(),
        bro_token: "bro".to_string(),
        spend_limit: Uint128::from(1000000u128),
        whitelist: vec!["distr0000".to_string()],
    };

    let info = mock_info("addr0000", &[]);
    let _res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

    // error: try to add already existing distributor
    let msg = ExecuteMsg::AddDistributor {
        distributor: "distr0000".to_string(),
    };
    let info = mock_info("gov", &[]);
    let res = execute(deps.as_mut(), mock_env(), info, msg.clone());

    match res {
        Err(ContractError::DistributorAlreadyRegistered {}) => assert_eq!(true, true),
        _ => panic!("DO NOT ENTER HERE"),
    }

    // proper execution
    let msg = ExecuteMsg::AddDistributor {
        distributor: "distr0001".to_string(),
    };
    let info = mock_info("gov", &[]);
    let _res = execute(deps.as_mut(), mock_env(), info, msg.clone());

    let config: ConfigResponse =
        from_binary(&query(deps.as_ref(), mock_env(), QueryMsg::Config {}).unwrap()).unwrap();
    assert_eq!(
        config,
        ConfigResponse {
            owner: "gov".to_string(),
            bro_token: "bro".to_string(),
            spend_limit: Uint128::from(1000000u128),
            whitelist: vec!["distr0000".to_string(), "distr0001".to_string()],
        }
    );
}

#[test]
fn remove_distributor() {
    let mut deps = mock_dependencies(&[]);

    let msg = InstantiateMsg {
        owner: "gov".to_string(),
        bro_token: "bro".to_string(),
        spend_limit: Uint128::from(1000000u128),
        whitelist: vec!["distr0000".to_string()],
    };

    let info = mock_info("addr0000", &[]);
    let _res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

    // error: try to remove not existing distributor
    let msg = ExecuteMsg::RemoveDistributor {
        distributor: "distr0001".to_string(),
    };

    let info = mock_info("gov", &[]);
    let res = execute(deps.as_mut(), mock_env(), info, msg.clone());
    match res {
        Err(ContractError::DistributorNotFound {}) => assert_eq!(true, true),
        _ => panic!("DO NOT ENTER HERE"),
    }

    // proper execution
    let msg = ExecuteMsg::RemoveDistributor {
        distributor: "distr0000".to_string(),
    };
    let info = mock_info("gov", &[]);
    let _res = execute(deps.as_mut(), mock_env(), info, msg.clone());

    let config: ConfigResponse =
        from_binary(&query(deps.as_ref(), mock_env(), QueryMsg::Config {}).unwrap()).unwrap();
    assert_eq!(
        config,
        ConfigResponse {
            owner: "gov".to_string(),
            bro_token: "bro".to_string(),
            spend_limit: Uint128::from(1000000u128),
            whitelist: vec![],
        }
    );
}

#[test]
fn distribute_reward() {
    let mut deps = mock_dependencies(&[]);

    let msg = InstantiateMsg {
        owner: "gov".to_string(),
        bro_token: "bro".to_string(),
        spend_limit: Uint128::from(1000000u128),
        whitelist: vec!["distr0000".to_string()],
    };

    let info = mock_info("addr0000", &[]);

    // binary message receive from distributor
    let execute_msg = to_binary(&Cw20ExecuteMsg::Burn {
        amount: Uint128::from(1u128),
    })
    .unwrap();

    let _res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

    // error: permission failed
    let msg = ExecuteMsg::DistributeRewards {
        distributions: vec![DistributeRewardMsg {
            contract: "staking0000".to_string(),
            amount: Uint128::from(1000000u128),
            msg: execute_msg.clone(),
        }],
    };

    let info = mock_info("addr0000", &[]);
    let res = execute(deps.as_mut(), mock_env(), info, msg);
    match res {
        Err(ContractError::Unauthorized {}) => assert_eq!(true, true),
        _ => panic!("DO NOT ENTER HERE"),
    }

    // error: failed due to spend limit(1 msg)
    let msg = ExecuteMsg::DistributeRewards {
        distributions: vec![DistributeRewardMsg {
            contract: "staking0000".to_string(),
            amount: Uint128::from(2000000u128),
            msg: execute_msg.clone(),
        }],
    };

    let info = mock_info("distr0000", &[]);
    let res = execute(deps.as_mut(), mock_env(), info, msg);
    match res {
        Err(ContractError::SpendLimitReached {}) => assert_eq!(true, true),
        _ => panic!("DO NOT ENTER HERE"),
    }

    // error: failed due to spend limit(3 msgs)
    let msg = ExecuteMsg::DistributeRewards {
        distributions: vec![
            DistributeRewardMsg {
                contract: "staking0000".to_string(),
                amount: Uint128::from(333_333u128),
                msg: execute_msg.clone(),
            },
            DistributeRewardMsg {
                contract: "staking0001".to_string(),
                amount: Uint128::from(333_333u128),
                msg: execute_msg.clone(),
            },
            DistributeRewardMsg {
                contract: "staking0001".to_string(),
                amount: Uint128::from(333_335u128),
                msg: execute_msg.clone(),
            },
        ],
    };

    let info = mock_info("distr0000", &[]);
    let res = execute(deps.as_mut(), mock_env(), info, msg);
    match res {
        Err(ContractError::SpendLimitReached {}) => assert_eq!(true, true),
        _ => panic!("DO NOT ENTER HERE"),
    }

    // proper execution
    let msg = ExecuteMsg::DistributeRewards {
        distributions: vec![DistributeRewardMsg {
            contract: "staking0000".to_string(),
            amount: Uint128::from(1000000u128),
            msg: execute_msg.clone(),
        }],
    };

    let info = mock_info("distr0000", &[]);
    let res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();
    assert_eq!(
        res.messages,
        vec![SubMsg::new(CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: "bro".to_string(),
            funds: vec![],
            msg: to_binary(&Cw20ExecuteMsg::Send {
                contract: "staking0000".to_string(),
                amount: Uint128::from(1000000u128),
                msg: execute_msg.clone(),
            })
            .unwrap(),
        }))]
    );
}

#[test]
fn update_owner() {
    let mut deps = mock_dependencies(&[]);

    let msg = InstantiateMsg {
        owner: "owner0000".to_string(),
        bro_token: "bro".to_string(),
        spend_limit: Uint128::from(1000000u128),
        whitelist: vec!["distr0000".to_string()],
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
            bro_token: "bro".to_string(),
            spend_limit: Uint128::from(1000000u128),
            whitelist: vec!["distr0000".to_string()],
        },
    );
}
