use crate::contract::{execute, instantiate, query};
use crate::error::ContractError;
use astroport::asset::{Asset, AssetInfo};
use cosmwasm_std::testing::{mock_env, mock_info};
use cosmwasm_std::{from_binary, Addr, Attribute, Uint128};
use cw20::Expiration;
use services::ownership_proposal::OwnershipProposalResponse;

use crate::mock_querier::{mock_dependencies, MOCK_FACTORY_ADDR, MOCK_PAIR_ADDR};

use services::oracle::{
    ConfigResponse, ConsultPriceResponse, ExecuteMsg, InstantiateMsg, QueryMsg,
};

#[test]
fn proper_initialization() {
    let mut deps = mock_dependencies(&[]);
    let env = mock_env();

    let bro_asset_info = AssetInfo::Token {
        contract_addr: Addr::unchecked("bro"),
    };
    let ust_asset_info = AssetInfo::NativeToken {
        denom: "uusd".to_string(),
    };

    deps.querier.set_cumulative_price(
        Addr::unchecked(MOCK_PAIR_ADDR),
        [
            Asset {
                info: bro_asset_info.clone(),
                amount: Uint128::zero(),
            },
            Asset {
                info: ust_asset_info.clone(),
                amount: Uint128::zero(),
            },
        ],
        Uint128::zero(),
        Uint128::from(10_000000u128),
        Uint128::from(100000u128),
    );

    let msg = InstantiateMsg {
        owner: "owner".to_string(),
        factory_contract: MOCK_FACTORY_ADDR.to_string(),
        asset_infos: [bro_asset_info.clone(), ust_asset_info.clone()],
        price_update_interval: 120,
    };

    let info = mock_info("addr0000", &[]);
    let _res = instantiate(deps.as_mut(), env.clone(), info, msg).unwrap();

    assert_eq!(
        from_binary::<ConfigResponse>(
            &query(deps.as_ref(), mock_env(), QueryMsg::Config {}).unwrap()
        )
        .unwrap(),
        ConfigResponse {
            owner: "owner".to_string(),
            factory: MOCK_FACTORY_ADDR.to_string(),
            asset_infos: [bro_asset_info.clone(), ust_asset_info.clone()],
            price_update_interval: 120,
        },
    );
}

#[test]
fn update_config() {
    let mut deps = mock_dependencies(&[]);
    let env = mock_env();

    let bro_asset_info = AssetInfo::Token {
        contract_addr: Addr::unchecked("bro"),
    };
    let ust_asset_info = AssetInfo::NativeToken {
        denom: "uusd".to_string(),
    };

    deps.querier.set_cumulative_price(
        Addr::unchecked(MOCK_PAIR_ADDR),
        [
            Asset {
                info: bro_asset_info.clone(),
                amount: Uint128::zero(),
            },
            Asset {
                info: ust_asset_info.clone(),
                amount: Uint128::zero(),
            },
        ],
        Uint128::zero(),
        Uint128::from(10_000000u128),
        Uint128::from(100000u128),
    );

    let msg = InstantiateMsg {
        owner: "owner".to_string(),
        factory_contract: MOCK_FACTORY_ADDR.to_string(),
        asset_infos: [bro_asset_info.clone(), ust_asset_info.clone()],
        price_update_interval: 120,
    };

    let info = mock_info("owner", &[]);
    let _res = instantiate(deps.as_mut(), env.clone(), info, msg).unwrap();

    // unauthorized: only owner allowed to execute
    let msg = ExecuteMsg::UpdateConfig {
        price_update_interval: Some(130),
    };

    let info = mock_info("addr0000", &[]);
    let res = execute(deps.as_mut(), mock_env(), info, msg.clone());
    match res {
        Err(ContractError::Unauthorized {}) => assert!(true),
        _ => panic!("DO NOT ENTER HERE"),
    }

    // proper execution
    let info = mock_info("owner", &[]);
    let res = execute(deps.as_mut(), mock_env(), info, msg.clone()).unwrap();

    assert_eq!(res.attributes[0], Attribute::new("action", "update_config"));
    assert_eq!(
        res.attributes[1],
        Attribute::new("price_update_interval_changed", "130")
    );

    assert_eq!(
        from_binary::<ConfigResponse>(
            &query(deps.as_ref(), mock_env(), QueryMsg::Config {}).unwrap()
        )
        .unwrap(),
        ConfigResponse {
            owner: "owner".to_string(),
            factory: MOCK_FACTORY_ADDR.to_string(),
            asset_infos: [bro_asset_info.clone(), ust_asset_info.clone()],
            price_update_interval: 130,
        },
    );
}

#[test]
fn update_price() {
    let mut deps = mock_dependencies(&[]);
    let mut env = mock_env();

    let bro_asset_info = AssetInfo::Token {
        contract_addr: Addr::unchecked("bro"),
    };
    let ust_asset_info = AssetInfo::NativeToken {
        denom: "uusd".to_string(),
    };

    let assets = [
        Asset {
            info: bro_asset_info.clone(),
            amount: Uint128::zero(),
        },
        Asset {
            info: ust_asset_info.clone(),
            amount: Uint128::zero(),
        },
    ];

    deps.querier.set_cumulative_price(
        Addr::unchecked(MOCK_PAIR_ADDR),
        assets.clone(),
        Uint128::zero(),
        Uint128::zero(),
        Uint128::zero(),
    );

    let msg = InstantiateMsg {
        owner: "owner".to_string(),
        factory_contract: MOCK_FACTORY_ADDR.to_string(),
        asset_infos: [bro_asset_info.clone(), ust_asset_info.clone()],
        price_update_interval: 120,
    };

    let info = mock_info("owner", &[]);
    let _res = instantiate(deps.as_mut(), env.clone(), info, msg).unwrap();

    assert_eq!(
        from_binary::<bool>(
            &query(deps.as_ref(), env.clone(), QueryMsg::IsReadyToTrigger {}).unwrap()
        )
        .unwrap(),
        false,
    );

    // update prices
    env.block.time = env.block.time.plus_seconds(121);

    assert_eq!(
        from_binary::<bool>(
            &query(deps.as_ref(), env.clone(), QueryMsg::IsReadyToTrigger {}).unwrap()
        )
        .unwrap(),
        true,
    );

    deps.querier.set_cumulative_price(
        Addr::unchecked(MOCK_PAIR_ADDR),
        assets.clone(),
        Uint128::zero(),
        Uint128::from(10_000000u128),
        Uint128::from(100000u128),
    );

    let msg = ExecuteMsg::UpdatePrice {};
    let info = mock_info("owner", &[]);
    let _res = execute(deps.as_mut(), env.clone(), info, msg).unwrap();

    // consult prices
    assert_eq!(
        from_binary::<ConsultPriceResponse>(
            &query(
                deps.as_ref(),
                mock_env(),
                QueryMsg::ConsultPrice {
                    asset: ust_asset_info.clone(),
                    amount: Uint128::from(10_000000u128),
                }
            )
            .unwrap()
        )
        .unwrap(),
        ConsultPriceResponse {
            amount: Uint128::from(8264u128),
        }
    );
}

#[test]
fn update_owner() {
    let mut deps = mock_dependencies(&[]);
    let env = mock_env();

    let bro_asset_info = AssetInfo::Token {
        contract_addr: Addr::unchecked("bro"),
    };
    let ust_asset_info = AssetInfo::NativeToken {
        denom: "uusd".to_string(),
    };

    deps.querier.set_cumulative_price(
        Addr::unchecked(MOCK_PAIR_ADDR),
        [
            Asset {
                info: bro_asset_info.clone(),
                amount: Uint128::zero(),
            },
            Asset {
                info: ust_asset_info.clone(),
                amount: Uint128::zero(),
            },
        ],
        Uint128::zero(),
        Uint128::from(10_000000u128),
        Uint128::from(100000u128),
    );

    let msg = InstantiateMsg {
        owner: "owner0000".to_string(),
        factory_contract: MOCK_FACTORY_ADDR.to_string(),
        asset_infos: [bro_asset_info.clone(), ust_asset_info.clone()],
        price_update_interval: 120,
    };

    let info = mock_info("addr0000", &[]);
    let _res = instantiate(deps.as_mut(), env.clone(), info, msg).unwrap();

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
            factory: MOCK_FACTORY_ADDR.to_string(),
            asset_infos: [bro_asset_info.clone(), ust_asset_info.clone()],
            price_update_interval: 120,
        },
    );
}
