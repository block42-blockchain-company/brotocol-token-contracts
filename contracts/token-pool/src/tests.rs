use crate::contract::{execute, instantiate, query};
use crate::error::ContractError;
use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info};
use cosmwasm_std::{from_binary, to_binary, CosmosMsg, SubMsg, Uint128, WasmMsg};
use cw20::{Cw20ExecuteMsg, Expiration};

use services::ownership_proposal::OwnershipProposalResponse;
use services::{
    bonding::ExecuteMsg as TestExecuteMsg,
    token_pool::{ConfigResponse, ExecuteMsg, InstantiateMsg, QueryMsg},
};

#[test]
fn proper_initialization() {
    let mut deps = mock_dependencies(&[]);

    let msg = InstantiateMsg {
        owner: "owner".to_string(),
        bro_token: "bro0000".to_string(),
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
        },
    );
}

#[test]
fn transfer() {
    let mut deps = mock_dependencies(&[]);

    let msg = InstantiateMsg {
        owner: "owner".to_string(),
        bro_token: "bro0000".to_string(),
    };

    let info = mock_info("addr0000", &[]);
    let _res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

    // error: unauthorized(only owner can execute)
    let msg = ExecuteMsg::Transfer {
        recipient: "addr0000".to_string(),
        amount: Uint128::from(100u128),
    };

    let info = mock_info("addr0001", &[]);
    let res = execute(deps.as_mut(), mock_env(), info, msg.clone());
    match res {
        Err(ContractError::Unauthorized {}) => assert!(true),
        _ => panic!("DO NOT ENTER HERE"),
    }

    // proper execution
    let info = mock_info("owner", &[]);
    let res = execute(deps.as_mut(), mock_env(), info, msg.clone()).unwrap();

    let transfer_bro_message = res.messages.get(0).expect("no message");
    assert_eq!(
        transfer_bro_message,
        &SubMsg::new(CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: "bro0000".to_string(),
            funds: vec![],
            msg: to_binary(&Cw20ExecuteMsg::Transfer {
                recipient: "addr0000".to_string(),
                amount: Uint128::from(100u128),
            })
            .unwrap(),
        }))
    );
}

#[test]
fn send() {
    let mut deps = mock_dependencies(&[]);

    let msg = InstantiateMsg {
        owner: "owner".to_string(),
        bro_token: "bro0000".to_string(),
    };

    let info = mock_info("addr0000", &[]);
    let _res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

    // error: unauthorized(only owner can execute)
    let msg = ExecuteMsg::Send {
        contract: "bonding".to_string(),
        amount: Uint128::from(100u128),
        msg: to_binary(&TestExecuteMsg::Claim {}).unwrap(),
    };

    let info = mock_info("addr0001", &[]);
    let res = execute(deps.as_mut(), mock_env(), info, msg.clone());
    match res {
        Err(ContractError::Unauthorized {}) => assert!(true),
        _ => panic!("DO NOT ENTER HERE"),
    }

    // proper execution
    let info = mock_info("owner", &[]);
    let res = execute(deps.as_mut(), mock_env(), info, msg.clone()).unwrap();

    let send_bro_message = res.messages.get(0).expect("no message");
    assert_eq!(
        send_bro_message,
        &SubMsg::new(CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: "bro0000".to_string(),
            funds: vec![],
            msg: to_binary(&Cw20ExecuteMsg::Send {
                contract: "bonding".to_string(),
                amount: Uint128::from(100u128),
                msg: to_binary(&TestExecuteMsg::Claim {}).unwrap(),
            })
            .unwrap(),
        }))
    );
}

#[test]
fn update_owner() {
    let mut deps = mock_dependencies(&[]);

    let msg = InstantiateMsg {
        owner: "owner0000".to_string(),
        bro_token: "bro0000".to_string(),
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
            bro_token: "bro0000".to_string(),
        },
    );
}
