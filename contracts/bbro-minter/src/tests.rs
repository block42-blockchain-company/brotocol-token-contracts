use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info};
use cosmwasm_std::{from_binary, to_binary, CosmosMsg, SubMsg, Uint128, WasmMsg};
use cw20::Cw20ExecuteMsg;

use crate::contract::{execute, instantiate, query};
use crate::error::ContractError;
use services::bbro_minter::{ConfigResponse, ExecuteMsg, InstantiateMsg, QueryMsg};

#[test]
fn proper_initialization() {
    let mut deps = mock_dependencies(&[]);

    let msg = InstantiateMsg {
        owner: "owner".to_string(),
        whitelist: vec!["minter0000".to_string()],
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
            bbro_token: "".to_string(),
            whitelist: vec!["minter0000".to_string()],
        },
    );
}

#[test]
fn update_config() {
    let mut deps = mock_dependencies(&[]);

    let msg = InstantiateMsg {
        owner: "owner".to_string(),
        whitelist: vec!["minter0000".to_string()],
    };

    let info = mock_info("addr0000", &[]);
    let _res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

    // unauthorized: only owner allowed to execute
    let msg = ExecuteMsg::UpdateConfig {
        owner: None,
        bbro_token: None,
    };

    let info = mock_info("addr0000", &[]);
    let res = execute(deps.as_mut(), mock_env(), info, msg);
    match res {
        Err(ContractError::Unauthorized {}) => assert_eq!(true, true),
        _ => panic!("DO NOT ENTER HERE"),
    }

    // proper execution
    // update bbro_token address
    let msg = ExecuteMsg::UpdateConfig {
        owner: None,
        bbro_token: Some("bbro_token".to_string()),
    };

    let info = mock_info("owner", &[]);
    let _res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();

    assert_eq!(
        from_binary::<ConfigResponse>(
            &query(deps.as_ref(), mock_env(), QueryMsg::Config {}).unwrap()
        )
        .unwrap(),
        ConfigResponse {
            owner: "owner".to_string(),
            bbro_token: "bbro_token".to_string(),
            whitelist: vec!["minter0000".to_string()],
        },
    );

    // update owner
    let msg = ExecuteMsg::UpdateConfig {
        owner: Some("new_owner".to_string()),
        bbro_token: None,
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
            bbro_token: "bbro_token".to_string(),
            whitelist: vec!["minter0000".to_string()],
        },
    );

    // unauthorized: try update with old owner
    let msg = ExecuteMsg::UpdateConfig {
        owner: None,
        bbro_token: None,
    };

    let info = mock_info("owner", &[]);
    let res = execute(deps.as_mut(), mock_env(), info, msg);
    match res {
        Err(ContractError::Unauthorized {}) => assert_eq!(true, true),
        _ => panic!("DO NOT ENTER HERE"),
    }
}

#[test]
fn add_minter() {
    let mut deps = mock_dependencies(&[]);

    let msg = InstantiateMsg {
        owner: "owner".to_string(),
        whitelist: vec!["minter0000".to_string()],
    };

    let info = mock_info("addr0000", &[]);
    let _res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

    // unauthorized: only owner allowed to execute
    let msg = ExecuteMsg::AddMinter {
        minter: "minter0000".to_string(),
    };

    let info = mock_info("addr0000", &[]);
    let res = execute(deps.as_mut(), mock_env(), info, msg.clone());
    match res {
        Err(ContractError::Unauthorized {}) => assert_eq!(true, true),
        _ => panic!("DO NOT ENTER HERE"),
    }

    // minter already registered error
    let info = mock_info("owner", &[]);
    let res = execute(deps.as_mut(), mock_env(), info, msg);
    match res {
        Err(ContractError::MinterAlreadyRegistered {}) => assert_eq!(true, true),
        _ => panic!("DO NOT ENTER HERE"),
    }

    // proper execution
    let msg = ExecuteMsg::AddMinter {
        minter: "minter0001".to_string(),
    };

    let info = mock_info("owner", &[]);
    let _res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();

    assert_eq!(
        from_binary::<ConfigResponse>(
            &query(deps.as_ref(), mock_env(), QueryMsg::Config {}).unwrap()
        )
        .unwrap(),
        ConfigResponse {
            owner: "owner".to_string(),
            bbro_token: "".to_string(),
            whitelist: vec!["minter0000".to_string(), "minter0001".to_string()],
        },
    );
}

#[test]
fn remove_minter() {
    let mut deps = mock_dependencies(&[]);

    let msg = InstantiateMsg {
        owner: "owner".to_string(),
        whitelist: vec!["minter0000".to_string()],
    };

    let info = mock_info("addr0000", &[]);
    let _res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

    // unauthorized: only owner allowed to execute
    let msg = ExecuteMsg::RemoveMinter {
        minter: "minter0000".to_string(),
    };

    let info = mock_info("addr0000", &[]);
    let res = execute(deps.as_mut(), mock_env(), info, msg.clone());
    match res {
        Err(ContractError::Unauthorized {}) => assert_eq!(true, true),
        _ => panic!("DO NOT ENTER HERE"),
    }

    // minter not found error
    let msg = ExecuteMsg::RemoveMinter {
        minter: "minter0001".to_string(),
    };

    let info = mock_info("owner", &[]);
    let res = execute(deps.as_mut(), mock_env(), info, msg.clone());
    match res {
        Err(ContractError::MinterNotFound {}) => assert_eq!(true, true),
        _ => panic!("DO NOT ENTER HERE"),
    }

    //proper execution
    let msg = ExecuteMsg::RemoveMinter {
        minter: "minter0000".to_string(),
    };

    let info = mock_info("owner", &[]);
    let _res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();

    assert_eq!(
        from_binary::<ConfigResponse>(
            &query(deps.as_ref(), mock_env(), QueryMsg::Config {}).unwrap()
        )
        .unwrap(),
        ConfigResponse {
            owner: "owner".to_string(),
            bbro_token: "".to_string(),
            whitelist: vec![],
        },
    );
}

#[test]
fn mint() {
    let mut deps = mock_dependencies(&[]);

    let msg = InstantiateMsg {
        owner: "owner".to_string(),
        whitelist: vec!["minter0000".to_string()],
    };

    let info = mock_info("addr0000", &[]);
    let _res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

    // bro_token address not set error
    let msg = ExecuteMsg::Mint {
        recipient: "addr0001".to_string(),
        amount: Uint128::from(100u128),
    };

    let info = mock_info("minter0001", &[]);
    let res = execute(deps.as_mut(), mock_env(), info, msg.clone());
    match res {
        Err(ContractError::BbroContractAddressIsNotSet {}) => assert_eq!(true, true),
        _ => panic!("DO NOT ENTER HERE"),
    }

    // set bbro_token address
    let msg = ExecuteMsg::UpdateConfig {
        owner: None,
        bbro_token: Some("bbro_token".to_string()),
    };

    let info = mock_info("owner", &[]);
    let _res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();

    // minter is not whitelisted error
    let msg = ExecuteMsg::Mint {
        recipient: "addr0001".to_string(),
        amount: Uint128::from(100u128),
    };

    let info = mock_info("minter0001", &[]);
    let res = execute(deps.as_mut(), mock_env(), info, msg.clone());
    match res {
        Err(ContractError::Unauthorized {}) => assert_eq!(true, true),
        _ => panic!("DO NOT ENTER HERE"),
    }

    // proper execution
    let msg = ExecuteMsg::Mint {
        recipient: "addr0001".to_string(),
        amount: Uint128::from(100u128),
    };

    let info = mock_info("minter0000", &[]);
    let res = execute(deps.as_mut(), mock_env(), info, msg.clone()).unwrap();

    let mint_msg = res.messages.get(0).expect("no message");
    assert_eq!(
        mint_msg,
        &SubMsg::new(CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: "bbro_token".to_string(),
            funds: vec![],
            msg: to_binary(&Cw20ExecuteMsg::Mint {
                recipient: "addr0001".to_string(),
                amount: Uint128::from(100u128),
            })
            .unwrap(),
        })),
    )
}

#[test]
fn burn() {
    let mut deps = mock_dependencies(&[]);

    let msg = InstantiateMsg {
        owner: "owner".to_string(),
        whitelist: vec!["minter0000".to_string()],
    };

    let info = mock_info("addr0000", &[]);
    let _res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

    // bro_token address not set error
    let msg = ExecuteMsg::Mint {
        recipient: "addr0001".to_string(),
        amount: Uint128::from(100u128),
    };

    let info = mock_info("minter0001", &[]);
    let res = execute(deps.as_mut(), mock_env(), info, msg.clone());
    match res {
        Err(ContractError::BbroContractAddressIsNotSet {}) => assert_eq!(true, true),
        _ => panic!("DO NOT ENTER HERE"),
    }

    // set bbro_token address
    let msg = ExecuteMsg::UpdateConfig {
        owner: None,
        bbro_token: Some("bbro_token".to_string()),
    };

    let info = mock_info("owner", &[]);
    let _res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();

    // minter is not whitelisted error
    let msg = ExecuteMsg::Mint {
        recipient: "addr0001".to_string(),
        amount: Uint128::from(100u128),
    };

    let info = mock_info("minter0001", &[]);
    let res = execute(deps.as_mut(), mock_env(), info, msg.clone());
    match res {
        Err(ContractError::Unauthorized {}) => assert_eq!(true, true),
        _ => panic!("DO NOT ENTER HERE"),
    }

    // proper execution
    let msg = ExecuteMsg::Burn {
        owner: "addr0001".to_string(),
        amount: Uint128::from(100u128),
    };

    let info = mock_info("minter0000", &[]);
    let res = execute(deps.as_mut(), mock_env(), info, msg.clone()).unwrap();

    let mint_msg = res.messages.get(0).expect("no message");
    assert_eq!(
        mint_msg,
        &SubMsg::new(CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: "bbro_token".to_string(),
            funds: vec![],
            msg: to_binary(&Cw20ExecuteMsg::BurnFrom {
                owner: "addr0001".to_string(),
                amount: Uint128::from(100u128),
            })
            .unwrap(),
        })),
    )
}
