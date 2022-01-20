use crate::contract::{execute, instantiate, query};
use crate::error::ContractError;
use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info};
use cosmwasm_std::{from_binary, to_binary, CosmosMsg, SubMsg, Uint128, WasmMsg};
use cw20::Cw20ExecuteMsg;

use services::rewards::{ConfigResponse, ExecuteMsg, InstantiateMsg, QueryMsg};

#[test]
fn proper_initialization() {
    let mut deps = mock_dependencies(&[]);

    let msg = InstantiateMsg {
        gov_contract: "gov".to_string(),
        bro_token: "bro".to_string(),
        spend_limit: Uint128::from(1000000u128),
        whitelist: vec!["distr0000".to_string()],
    };

    let info = mock_info("addr0000", &[]);

    // we can just call .unwrap() to assert this was a success
    let _res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

    // it worked, let's query the state
    let config: ConfigResponse =
        from_binary(&query(deps.as_ref(), mock_env(), QueryMsg::Config {}).unwrap()).unwrap();
    assert_eq!("gov", config.gov_contract.as_str());
    assert_eq!("bro", config.bro_token.as_str());
    assert_eq!(Uint128::from(1000000u128), config.spend_limit);
    assert_eq!(vec!["distr0000".to_string()], config.whitelist);
}

#[test]
fn update_config() {
    let mut deps = mock_dependencies(&[]);

    let msg = InstantiateMsg {
        gov_contract: "gov".to_string(),
        bro_token: "bro".to_string(),
        spend_limit: Uint128::from(1000000u128),
        whitelist: vec!["distr0000".to_string()],
    };

    let info = mock_info("addr0000", &[]);

    // we can just call .unwrap() to assert this was a success
    let _res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

    // it worked, let's query the state
    let config: ConfigResponse =
        from_binary(&query(deps.as_ref(), mock_env(), QueryMsg::Config {}).unwrap()).unwrap();
    assert_eq!("gov", config.gov_contract.as_str());
    assert_eq!("bro", config.bro_token.as_str());
    assert_eq!(Uint128::from(1000000u128), config.spend_limit);

    // update spend_limit and gov_contract addr
    let msg = ExecuteMsg::UpdateConfig {
        new_gov_contract: Some("new_gov".to_string()),
        bro_token: None,
        spend_limit: Some(Uint128::from(500000u128)),
    };
    let info = mock_info("addr0000", &[]);
    let res = execute(deps.as_mut(), mock_env(), info, msg.clone());

    match res {
        Err(ContractError::Unauthorized {}) => assert_eq!(true, true),
        _ => panic!("DO NOT ENTER HERE"),
    }

    let info = mock_info("gov", &[]);
    let _res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();
    let config: ConfigResponse =
        from_binary(&query(deps.as_ref(), mock_env(), QueryMsg::Config {}).unwrap()).unwrap();
    assert_eq!(
        config,
        ConfigResponse {
            gov_contract: "new_gov".to_string(),
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
        gov_contract: "gov".to_string(),
        bro_token: "bro".to_string(),
        spend_limit: Uint128::from(1000000u128),
        whitelist: vec!["distr0000".to_string()],
    };

    let info = mock_info("addr0000", &[]);

    // we can just call .unwrap() to assert this was a success
    let _res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

    // try to add already existing distributor
    let msg = ExecuteMsg::AddDistributor {
        distributor: "distr0000".to_string(),
    };
    let info = mock_info("gov", &[]);
    let res = execute(deps.as_mut(), mock_env(), info, msg.clone());

    match res {
        Err(ContractError::DistributorAlreadyRegistered {}) => assert_eq!(true, true),
        _ => panic!("DO NOT ENTER HERE"),
    }

    // add new one
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
            gov_contract: "gov".to_string(),
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
        gov_contract: "gov".to_string(),
        bro_token: "bro".to_string(),
        spend_limit: Uint128::from(1000000u128),
        whitelist: vec!["distr0000".to_string()],
    };

    let info = mock_info("addr0000", &[]);

    // we can just call .unwrap() to assert this was a success
    let _res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

    // try to remove not existing distributor
    let msg = ExecuteMsg::RemoveDistributor {
        distributor: "distr0001".to_string(),
    };
    let info = mock_info("gov", &[]);
    let res = execute(deps.as_mut(), mock_env(), info, msg.clone());

    match res {
        Err(ContractError::DistributorNotFound {}) => assert_eq!(true, true),
        _ => panic!("DO NOT ENTER HERE"),
    }

    // remove proper one
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
            gov_contract: "gov".to_string(),
            bro_token: "bro".to_string(),
            spend_limit: Uint128::from(1000000u128),
            whitelist: vec![],
        }
    );
}

#[test]
fn test_reward() {
    let mut deps = mock_dependencies(&[]);

    let msg = InstantiateMsg {
        gov_contract: "gov".to_string(),
        bro_token: "bro".to_string(),
        spend_limit: Uint128::from(1000000u128),
        whitelist: vec!["distr0000".to_string()],
    };

    let info = mock_info("addr0000", &[]);

    let execute_msg = to_binary(&Cw20ExecuteMsg::Burn {
        amount: Uint128::from(1u128),
    })
    .unwrap();

    // we can just call .unwrap() to assert this was a success
    let _res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

    // permission failed
    let msg = ExecuteMsg::Reward {
        contract: "staking0000".to_string(),
        amount: Uint128::from(1000000u128),
        msg: execute_msg.clone(),
    };

    let info = mock_info("addr0000", &[]);
    let res = execute(deps.as_mut(), mock_env(), info, msg);
    match res {
        Err(ContractError::Unauthorized {}) => assert_eq!(true, true),
        _ => panic!("DO NOT ENTER HERE"),
    }

    // failed due to spend limit
    let msg = ExecuteMsg::Reward {
        contract: "staking0000".to_string(),
        amount: Uint128::from(2000000u128),
        msg: execute_msg.clone(),
    };

    let info = mock_info("distr0000", &[]);
    let res = execute(deps.as_mut(), mock_env(), info, msg);
    match res {
        Err(ContractError::SpendLimitReached {}) => assert_eq!(true, true),
        _ => panic!("DO NOT ENTER HERE"),
    }

    let msg = ExecuteMsg::Reward {
        contract: "staking0000".to_string(),
        amount: Uint128::from(1000000u128),
        msg: execute_msg.clone(),
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
