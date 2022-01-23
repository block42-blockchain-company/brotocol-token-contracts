use crate::contract::{execute, instantiate, query};
use crate::error::ContractError;
use cosmwasm_std::testing::{mock_env, mock_info, MOCK_CONTRACT_ADDR};
use cosmwasm_std::{from_binary, to_binary, BankMsg, Coin, CosmosMsg, SubMsg, Uint128, WasmMsg};
use cw20::Cw20ExecuteMsg;
use terraswap::asset::AssetInfo;

use crate::mock_querier::mock_dependencies;

use services::treasury::{ConfigResponse, ExecuteMsg, InstantiateMsg, QueryMsg};

#[test]
fn proper_initialization() {
    let mut deps = mock_dependencies(&[]);

    let msg = InstantiateMsg {};

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
}

#[test]
fn spend() {
    let mut deps = mock_dependencies(&[Coin {
        denom: "uusd".to_string(),
        amount: Uint128::zero(),
    }]);

    let msg = InstantiateMsg {};

    let info = mock_info("addr0000", &[]);
    let _res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

    // unauthorized: only owner allowed to execute
    let msg = ExecuteMsg::Spend {
        asset_info: AssetInfo::NativeToken {
            denom: "uusd".to_string(),
        },
        recipient: "addr0004".to_string(),
    };

    let info = mock_info("addr0001", &[]);
    let res = execute(deps.as_mut(), mock_env(), info, msg.clone());
    match res {
        Err(ContractError::Unauthorized {}) => assert_eq!(true, true),
        _ => panic!("DO NOT ENTER HERE"),
    }

    // insufficient funds error
    let msg = ExecuteMsg::Spend {
        asset_info: AssetInfo::NativeToken {
            denom: "uusd".to_string(),
        },
        recipient: "addr0004".to_string(),
    };

    let info = mock_info("addr0000", &[]);
    let res = execute(deps.as_mut(), mock_env(), info, msg.clone());
    match res {
        Err(ContractError::InsufficientFunds {}) => assert_eq!(true, true),
        _ => panic!("DO NOT ENTER HERE"),
    }

    // proper execution
    let mut deps = mock_dependencies(&[Coin {
        denom: "uusd".to_string(),
        amount: Uint128::from(1000u128),
    }]);

    deps.querier.with_token_balances(&[(
        &"bro_token".to_string(),
        &[(&MOCK_CONTRACT_ADDR.to_string(), &Uint128::from(12300u128))],
    )]);

    let msg = InstantiateMsg {};

    let info = mock_info("addr0000", &[]);
    let _res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

    // native token transfer
    let msg = ExecuteMsg::Spend {
        asset_info: AssetInfo::NativeToken {
            denom: "uusd".to_string(),
        },
        recipient: "addr0004".to_string(),
    };

    let info = mock_info("addr0000", &[]);
    let res = execute(deps.as_mut(), mock_env(), info, msg.clone()).unwrap();

    let native_transfer_msg = res.messages.get(0).expect("no message");
    assert_eq!(
        native_transfer_msg,
        &SubMsg::new(CosmosMsg::Bank(BankMsg::Send {
            to_address: "addr0004".to_string(),
            amount: vec![Coin {
                denom: "uusd".to_string(),
                amount: Uint128::from(1000u128),
            }]
        })),
    );

    // custom token transfer
    let msg = ExecuteMsg::Spend {
        asset_info: AssetInfo::Token {
            contract_addr: "bro_token".to_string(),
        },
        recipient: "addr0004".to_string(),
    };

    let info = mock_info("addr0000", &[]);
    let res = execute(deps.as_mut(), mock_env(), info, msg.clone()).unwrap();

    let token_transfer_msg = res.messages.get(0).expect("no message");
    assert_eq!(
        token_transfer_msg,
        &SubMsg::new(CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: "bro_token".to_string(),
            funds: vec![],
            msg: to_binary(&Cw20ExecuteMsg::Transfer {
                recipient: "addr0004".to_string(),
                amount: Uint128::from(12300u128),
            })
            .unwrap(),
        })),
    );
}
