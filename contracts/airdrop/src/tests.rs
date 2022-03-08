use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info};
use cosmwasm_std::{attr, from_binary, to_binary, Attribute, CosmosMsg, SubMsg, Uint128, WasmMsg};
use cw20::{Cw20ExecuteMsg, Cw20ReceiveMsg};

use crate::contract::{execute, instantiate, query};
use crate::error::ContractError;
use services::airdrop::{
    ConfigResponse, Cw20HookMsg, ExecuteMsg, InstantiateMsg, IsClaimedResponse,
    LatestStageResponse, MerkleRootResponse, QueryMsg,
};

#[test]
fn proper_initialization() {
    let mut deps = mock_dependencies(&[]);

    let msg = InstantiateMsg {
        owner: "owner0000".to_string(),
        bro_token: "bro0000".to_string(),
    };

    let info = mock_info("addr0000", &[]);

    // we can just call .unwrap() to assert this was a success
    let _res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

    // it worked, let's query the state
    let res = query(deps.as_ref(), mock_env(), QueryMsg::Config {}).unwrap();
    let config: ConfigResponse = from_binary(&res).unwrap();
    assert_eq!("owner0000", config.owner.as_str());
    assert_eq!("bro0000", config.bro_token.as_str());

    let res = query(deps.as_ref(), mock_env(), QueryMsg::LatestStage {}).unwrap();
    let latest_stage: LatestStageResponse = from_binary(&res).unwrap();
    assert_eq!(0u8, latest_stage.latest_stage);
}

#[test]
fn update_config() {
    let mut deps = mock_dependencies(&[]);

    let msg = InstantiateMsg {
        owner: "owner0000".to_string(),
        bro_token: "bro0000".to_string(),
    };

    let info = mock_info("addr0000", &[]);
    let _res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

    // update owner
    let info = mock_info("owner0000", &[]);
    let msg = ExecuteMsg::UpdateConfig {
        owner: Some("owner0001".to_string()),
    };

    let res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();
    assert_eq!(0, res.messages.len());

    assert_eq!(res.attributes[0], Attribute::new("action", "update_config"));
    assert_eq!(
        res.attributes[1],
        Attribute::new("owner_changed", "owner0001")
    );

    // it worked, let's query the state
    let res = query(deps.as_ref(), mock_env(), QueryMsg::Config {}).unwrap();
    let config: ConfigResponse = from_binary(&res).unwrap();
    assert_eq!("owner0001", config.owner.as_str());

    // Unauthorzied err
    let info = mock_info("owner0000", &[]);
    let msg = ExecuteMsg::UpdateConfig { owner: None };

    let res = execute(deps.as_mut(), mock_env(), info, msg);
    match res {
        Err(ContractError::Unauthorized {}) => {}
        _ => panic!("Must return unauthorized error"),
    }
}

#[test]
fn register_merkle_root() {
    let mut deps = mock_dependencies(&[]);

    let msg = InstantiateMsg {
        owner: "owner0000".to_string(),
        bro_token: "bro0000".to_string(),
    };

    let info = mock_info("addr0000", &[]);
    let _res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

    // error: unauthorized (info.sender must be bro token addr)
    let msg = ExecuteMsg::Receive(Cw20ReceiveMsg {
        sender: "addr0000".to_string(),
        amount: Uint128::from(100u128),
        msg: to_binary(&Cw20HookMsg::RegisterMerkleRoot {
            merkle_root: "34e1e5510ffa861485d6d4712cb297b60fbad7114f01aeeedd426947cf88c689"
                .to_string(),
        })
        .unwrap(),
    });

    let info = mock_info("addr0000", &[]);
    let res = execute(deps.as_mut(), mock_env(), info, msg.clone());
    match res {
        Err(ContractError::Unauthorized {}) => {}
        _ => panic!("Must return unauthorized error"),
    }

    // error: unauthorized (cw20_msg.sender must be contract owner)
    let info = mock_info("bro0000", &[]);
    let res = execute(deps.as_mut(), mock_env(), info, msg);
    match res {
        Err(ContractError::Unauthorized {}) => {}
        _ => panic!("Must return unauthorized error"),
    }

    // register new merkle root
    let msg = ExecuteMsg::Receive(Cw20ReceiveMsg {
        sender: "owner0000".to_string(),
        amount: Uint128::from(100u128),
        msg: to_binary(&Cw20HookMsg::RegisterMerkleRoot {
            merkle_root: "34e1e5510ffa861485d6d4712cb297b60fbad7114f01aeeedd426947cf88c689"
                .to_string(),
        })
        .unwrap(),
    });

    let info = mock_info("bro0000", &[]);
    let res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();

    assert_eq!(
        res.attributes,
        vec![
            attr("action", "register_merkle_root"),
            attr("stage", "1"),
            attr(
                "merkle_root",
                "34e1e5510ffa861485d6d4712cb297b60fbad7114f01aeeedd426947cf88c689"
            )
        ]
    );

    let res = query(deps.as_ref(), mock_env(), QueryMsg::LatestStage {}).unwrap();
    let latest_stage: LatestStageResponse = from_binary(&res).unwrap();
    assert_eq!(1u8, latest_stage.latest_stage);

    let res = query(
        deps.as_ref(),
        mock_env(),
        QueryMsg::MerkleRoot {
            stage: latest_stage.latest_stage,
        },
    )
    .unwrap();
    let merkle_root: MerkleRootResponse = from_binary(&res).unwrap();
    assert_eq!(
        "34e1e5510ffa861485d6d4712cb297b60fbad7114f01aeeedd426947cf88c689".to_string(),
        merkle_root.merkle_root
    );
}

#[test]
fn claim() {
    let mut deps = mock_dependencies(&[]);

    let msg = InstantiateMsg {
        owner: "owner0000".to_string(),
        bro_token: "bro0000".to_string(),
    };

    let info = mock_info("addr0000", &[]);
    let _res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

    // register merkle roots
    let info = mock_info("bro0000", &[]);
    let msg = ExecuteMsg::Receive(Cw20ReceiveMsg {
        sender: "owner0000".to_string(),
        amount: Uint128::from(100u128),
        msg: to_binary(&Cw20HookMsg::RegisterMerkleRoot {
            merkle_root: "6379fc64b595aaed9ad87abb9c3acac7d29bea4546a35fafac4fc98269d4b615"
                .to_string(),
        })
        .unwrap(),
    });
    let _res = execute(deps.as_mut(), mock_env(), info.clone(), msg).unwrap();

    let msg = ExecuteMsg::Receive(Cw20ReceiveMsg {
        sender: "owner0000".to_string(),
        amount: Uint128::from(100u128),
        msg: to_binary(&Cw20HookMsg::RegisterMerkleRoot {
            merkle_root: "374d30a1b573a7a328ea39cbaf507c4c246f5744396b740cd2b332666a5ed733"
                .to_string(),
        })
        .unwrap(),
    });
    let _res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();

    let msg = ExecuteMsg::Claim {
        amount: Uint128::new(41241u128),
        stage: 1u8,
        proof: vec![
            "f10fdeda8d9df3bb96e6386f0294b093ee6aef0838b95f5a47d6b8ace7bbaa4e".to_string(),
            "319085115986ec53b8fc0543cd54362b08d996b73a42c44837ac24144115697c".to_string(),
        ],
    };

    let info = mock_info("terra16g488h5ywmpmc4uudhl023m97qnugr0kc9tekv", &[]);
    let res = execute(deps.as_mut(), mock_env(), info.clone(), msg.clone()).unwrap();
    assert_eq!(
        res.messages,
        vec![SubMsg::new(CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: "bro0000".to_string(),
            msg: to_binary(&Cw20ExecuteMsg::Transfer {
                recipient: "terra16g488h5ywmpmc4uudhl023m97qnugr0kc9tekv".to_string(),
                amount: Uint128::new(41241u128),
            })
            .unwrap(),
            funds: vec![]
        }))]
    );

    assert_eq!(
        res.attributes,
        vec![
            attr("action", "claim"),
            attr("stage", "1"),
            attr("address", "terra16g488h5ywmpmc4uudhl023m97qnugr0kc9tekv"),
            attr("amount", "41241")
        ]
    );

    assert!(
        from_binary::<IsClaimedResponse>(
            &query(
                deps.as_ref(),
                mock_env(),
                QueryMsg::IsClaimed {
                    stage: 1,
                    address: "terra16g488h5ywmpmc4uudhl023m97qnugr0kc9tekv".to_string(),
                }
            )
            .unwrap()
        )
        .unwrap()
        .is_claimed
    );

    let res = execute(deps.as_mut(), mock_env(), info, msg);
    match res {
        Err(ContractError::AlreadyClaimed {}) => {}
        _ => panic!("DO NOT ENTER HERE"),
    }

    // Claim next airdrop
    let msg = ExecuteMsg::Claim {
        amount: Uint128::new(4134134u128),
        stage: 2u8,
        proof: vec![
            "0ceeee6895428d3dfa5fabe848e76734e9f9ed0d70adff33f4e9fd576d125d2e".to_string(),
            "8b1eef385c9e1c9002bb8366329b429750df45e6515f00454f3e03f241c1c945".to_string(),
            "8eb811cc4a74bb39f942a856074a76e857b73cce5bfbba0a4fff4d768d9e932b".to_string(),
        ],
    };

    let info = mock_info("terra1nn8r2nq9p0ce32zp9mc3cmha04kztpxm89zkpy", &[]);
    let res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();

    assert_eq!(
        res.messages,
        vec![SubMsg::new(CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: "bro0000".to_string(),
            msg: to_binary(&Cw20ExecuteMsg::Transfer {
                recipient: "terra1nn8r2nq9p0ce32zp9mc3cmha04kztpxm89zkpy".to_string(),
                amount: Uint128::new(4134134u128),
            })
            .unwrap(),
            funds: vec![]
        }))]
    );

    assert_eq!(
        res.attributes,
        vec![
            attr("action", "claim"),
            attr("stage", "2"),
            attr("address", "terra1nn8r2nq9p0ce32zp9mc3cmha04kztpxm89zkpy"),
            attr("amount", "4134134")
        ]
    );
}
