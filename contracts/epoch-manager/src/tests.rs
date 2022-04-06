use std::str::FromStr;

use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info};
use cosmwasm_std::{from_binary, Attribute, Decimal, StdError};
use cw20::Expiration;
use services::ownership_proposal::OwnershipProposalResponse;

use crate::contract::{execute, instantiate, query};
use crate::error::ContractError;
use services::epoch_manager::{
    ConfigResponse, EpochInfoResponse, ExecuteMsg, InstantiateMsg, QueryMsg,
};

#[test]
fn proper_initialization() {
    let mut deps = mock_dependencies(&[]);

    let msg = InstantiateMsg {
        owner: "addr0000".to_string(),
        epoch: 1000,
        blocks_per_year: 6_307_200,
        bbro_emission_rate: Decimal::from_str("1.0").unwrap(),
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
        },
    );

    assert_eq!(
        from_binary::<EpochInfoResponse>(
            &query(deps.as_ref(), mock_env(), QueryMsg::EpochInfo {}).unwrap()
        )
        .unwrap(),
        EpochInfoResponse {
            epoch: 1000,
            blocks_per_year: 6_307_200,
            bbro_emission_rate: Decimal::from_str("1.0").unwrap(),
        },
    );
}

#[test]
fn update_state() {
    let mut deps = mock_dependencies(&[]);

    let msg = InstantiateMsg {
        owner: "addr0000".to_string(),
        epoch: 1000,
        blocks_per_year: 6_307_200,
        bbro_emission_rate: Decimal::from_str("1.0").unwrap(),
    };

    let info = mock_info("addr0000", &[]);
    let _res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

    // unauthorized: only owner allowed to execute
    let msg = ExecuteMsg::UpdateState {
        epoch: Some(2000),
        blocks_per_year: Some(100),
        bbro_emission_rate: Some(Decimal::from_str("0.9").unwrap()),
    };

    let info = mock_info("addr0001", &[]);
    let res = execute(deps.as_mut(), mock_env(), info, msg.clone());
    match res {
        Err(ContractError::Unauthorized {}) => assert_eq!(true, true),
        _ => panic!("DO NOT ENTER HERE"),
    }

    // error: epoch must be higher then zero
    let msg = ExecuteMsg::UpdateState {
        epoch: Some(0),
        blocks_per_year: Some(100),
        bbro_emission_rate: Some(Decimal::from_str("0.9").unwrap()),
    };

    let info = mock_info("addr0000", &[]);
    let res = execute(deps.as_mut(), mock_env(), info, msg.clone()).unwrap_err();
    match res {
        ContractError::Std(StdError::GenericErr { msg, .. }) => {
            assert_eq!(msg, "epoch must be higher than zero".to_string())
        }
        _ => panic!("DO NOT ENTER HERE"),
    }

    // proper execution
    let msg = ExecuteMsg::UpdateState {
        epoch: Some(2000),
        blocks_per_year: Some(100),
        bbro_emission_rate: Some(Decimal::from_str("0.9").unwrap()),
    };

    let info = mock_info("addr0000", &[]);
    let res = execute(deps.as_mut(), mock_env(), info, msg.clone()).unwrap();

    assert_eq!(res.attributes[0], Attribute::new("action", "update_state"));
    assert_eq!(res.attributes[1], Attribute::new("epoch_changed", "2000"));
    assert_eq!(
        res.attributes[2],
        Attribute::new("blocks_per_year_changed", "100")
    );
    assert_eq!(
        res.attributes[3],
        Attribute::new("bbro_emission_rate_changed", "0.9")
    );

    assert_eq!(
        from_binary::<EpochInfoResponse>(
            &query(deps.as_ref(), mock_env(), QueryMsg::EpochInfo {}).unwrap()
        )
        .unwrap(),
        EpochInfoResponse {
            epoch: 2000,
            blocks_per_year: 100,
            bbro_emission_rate: Decimal::from_str("0.9").unwrap(),
        },
    );
}

#[test]
fn update_owner() {
    let mut deps = mock_dependencies(&[]);

    let msg = InstantiateMsg {
        owner: "owner0000".to_string(),
        epoch: 1000,
        blocks_per_year: 6_307_200,
        bbro_emission_rate: Decimal::from_str("1.0").unwrap(),
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
        },
    );
}
