use astroport::asset::AssetInfo;
use cosmwasm_std::testing::{MockApi, MockQuerier, MockStorage};
use cosmwasm_std::{
    from_binary, from_slice, to_binary, Addr, Coin, ContractResult, Decimal, OwnedDeps, Querier,
    QuerierResult, QueryRequest, SystemError, SystemResult, Uint128, WasmQuery,
};
use services::staking::LockupConfigResponse;
use std::collections::HashMap;
use std::str::FromStr;

use astroport::{asset::PairInfo, factory::QueryMsg as FactoryQueryMsg};
use cw20::{BalanceResponse as Cw20BalanceResponse, Cw20QueryMsg, TokenInfoResponse};
use services::{
    oracle::{ConsultPriceResponse, QueryMsg as OracleQueryMsg},
    staking::{ConfigResponse as StakingConfigResponse, QueryMsg as StakingQueryMsg},
};
use terra_cosmwasm::{TaxCapResponse, TaxRateResponse, TerraQuery, TerraQueryWrapper, TerraRoute};

pub const MOCK_ASTRO_FACTORY_ADDR: &str = "astrofactory";
pub const MOCK_BRO_UST_PAIR_ADDR: &str = "bro_ust_pair";
pub const MOCK_LP_TOKEN_ADDR: &str = "bro_ust_lp";
pub const MOCK_BRO_TOKEN_ADDR: &str = "bro_token";
pub const MOCK_ORACLE_ADDR: &str = "oracle";
pub const MOCK_STAKING_ADDR: &str = "bro_staking";

/// mock_dependencies is a drop-in replacement for cosmwasm_std::testing::mock_dependencies
/// this uses our CustomQuerier.
pub fn mock_dependencies(
    balances: &[(&str, &[Coin])],
) -> OwnedDeps<MockStorage, MockApi, WasmMockQuerier> {
    let custom_querier: WasmMockQuerier = WasmMockQuerier::new(MockQuerier::new(balances));

    OwnedDeps {
        storage: MockStorage::default(),
        api: MockApi::default(),
        querier: custom_querier,
    }
}

pub struct WasmMockQuerier {
    base: MockQuerier<TerraQueryWrapper>,
    token_querier: TokenQuerier,
    tax_querier: TaxQuerier,
}

#[derive(Clone, Default)]
pub struct TokenQuerier {
    // this lets us iterate over all pairs that match the first string
    balances: HashMap<String, HashMap<String, Uint128>>,
}

impl TokenQuerier {
    pub fn new(balances: &[(&String, &[(&String, &Uint128)])]) -> Self {
        TokenQuerier {
            balances: balances_to_map(balances),
        }
    }
}

pub(crate) fn balances_to_map(
    balances: &[(&String, &[(&String, &Uint128)])],
) -> HashMap<String, HashMap<String, Uint128>> {
    let mut balances_map: HashMap<String, HashMap<String, Uint128>> = HashMap::new();
    for (contract_addr, balances) in balances.iter() {
        let mut contract_balances_map: HashMap<String, Uint128> = HashMap::new();
        for (addr, balance) in balances.iter() {
            contract_balances_map.insert(addr.to_string(), **balance);
        }

        balances_map.insert(contract_addr.to_string(), contract_balances_map);
    }
    balances_map
}

#[derive(Clone, Default)]
pub struct TaxQuerier {
    rate: Decimal,
    // this lets us iterate over all pairs that match the first string
    caps: HashMap<String, Uint128>,
}

impl Querier for WasmMockQuerier {
    fn raw_query(&self, bin_request: &[u8]) -> QuerierResult {
        // MockQuerier doesn't support Custom, so we ignore it completely here
        let request: QueryRequest<TerraQueryWrapper> = match from_slice(bin_request) {
            Ok(v) => v,
            Err(e) => {
                return SystemResult::Err(SystemError::InvalidRequest {
                    error: format!("Parsing query request: {}", e),
                    request: bin_request.into(),
                })
            }
        };
        self.handle_query(&request)
    }
}

impl WasmMockQuerier {
    pub fn handle_query(&self, request: &QueryRequest<TerraQueryWrapper>) -> QuerierResult {
        match &request {
            QueryRequest::Custom(TerraQueryWrapper { route, query_data }) => {
                if route == &TerraRoute::Treasury {
                    match query_data {
                        TerraQuery::TaxRate {} => {
                            let res = TaxRateResponse {
                                rate: self.tax_querier.rate,
                            };
                            SystemResult::Ok(ContractResult::from(to_binary(&res)))
                        }
                        TerraQuery::TaxCap { denom } => {
                            let cap = self
                                .tax_querier
                                .caps
                                .get(denom)
                                .copied()
                                .unwrap_or_default();
                            let res = TaxCapResponse { cap };
                            SystemResult::Ok(ContractResult::from(to_binary(&res)))
                        }
                        _ => panic!("DO NOT ENTER HERE"),
                    }
                } else {
                    panic!("DO NOT ENTER HERE")
                }
            }
            QueryRequest::Wasm(WasmQuery::Smart { contract_addr, msg }) => {
                if contract_addr == MOCK_ASTRO_FACTORY_ADDR {
                    match from_binary(msg).unwrap() {
                        FactoryQueryMsg::Pair { .. } => SystemResult::Ok(ContractResult::Ok(
                            to_binary(&PairInfo {
                                asset_infos: [
                                    AssetInfo::Token {
                                        contract_addr: Addr::unchecked(MOCK_BRO_TOKEN_ADDR),
                                    },
                                    AssetInfo::NativeToken {
                                        denom: "uusd".to_string(),
                                    },
                                ],
                                contract_addr: Addr::unchecked(MOCK_BRO_UST_PAIR_ADDR),
                                liquidity_token: Addr::unchecked(MOCK_LP_TOKEN_ADDR),
                                pair_type: astroport::factory::PairType::Xyk {},
                            })
                            .unwrap(),
                        )),
                        _ => panic!("DO NOT ENTER HERE"),
                    }
                } else if contract_addr == MOCK_ORACLE_ADDR {
                    match from_binary(msg).unwrap() {
                        OracleQueryMsg::ConsultPrice { amount, .. } => {
                            SystemResult::Ok(ContractResult::Ok(
                                to_binary(&ConsultPriceResponse {
                                    amount: amount.checked_div(Uint128::from(10u128)).unwrap(),
                                })
                                .unwrap(),
                            ))
                        }
                        OracleQueryMsg::IsReadyToTrigger {} => {
                            SystemResult::Ok(ContractResult::Ok(to_binary(&true).unwrap()))
                        }
                        _ => panic!("DO NOT ENTER HERE"),
                    }
                } else if contract_addr == MOCK_STAKING_ADDR {
                    match from_binary(msg).unwrap() {
                        StakingQueryMsg::Config {} => SystemResult::Ok(ContractResult::Ok(
                            to_binary(&StakingConfigResponse {
                                owner: "owner".to_string(),
                                paused: false,
                                bro_token: "bro_token".to_string(),
                                rewards_pool_contract: "rewards".to_string(),
                                bbro_minter_contract: "bbro_minter".to_string(),
                                epoch_manager_contract: "epoch_manager".to_string(),
                                community_bonding_contract: Some(
                                    "community_bonding0000".to_string(),
                                ),
                                unstake_period_blocks: 10,
                                min_staking_amount: Uint128::from(1u128),
                                lockup_config: LockupConfigResponse {
                                    min_lockup_period_epochs: 2,
                                    max_lockup_period_epochs: 725,
                                    base_rate: Decimal::from_str("0.1").unwrap(),
                                    linear_growth: Decimal::from_str("0.1").unwrap(),
                                    exponential_growth: Decimal::from_str("0.1").unwrap(),
                                },
                            })
                            .unwrap(),
                        )),
                        _ => panic!("DO NOT ENTER HERE"),
                    }
                } else {
                    match from_binary(msg).unwrap() {
                        Cw20QueryMsg::TokenInfo {} => SystemResult::Ok(ContractResult::Ok(
                            to_binary(&TokenInfoResponse {
                                name: "LP".to_string(),
                                symbol: "LP".to_string(),
                                decimals: 6,
                                total_supply: Uint128::from(1_000_000000u128),
                            })
                            .unwrap(),
                        )),
                        Cw20QueryMsg::Balance { address } => {
                            let balances: &HashMap<String, Uint128> =
                                match self.token_querier.balances.get(contract_addr) {
                                    Some(balances) => balances,
                                    None => {
                                        return SystemResult::Err(SystemError::InvalidRequest {
                                            error: format!(
                                                "No balance info exists for the contract {}",
                                                contract_addr
                                            ),
                                            request: msg.as_slice().into(),
                                        })
                                    }
                                };
                            let balance = match balances.get(&address) {
                                Some(v) => *v,
                                None => {
                                    return SystemResult::Ok(ContractResult::Ok(
                                        to_binary(&Cw20BalanceResponse {
                                            balance: Uint128::zero(),
                                        })
                                        .unwrap(),
                                    ));
                                }
                            };
                            SystemResult::Ok(ContractResult::Ok(
                                to_binary(&Cw20BalanceResponse { balance }).unwrap(),
                            ))
                        }
                        _ => panic!("DO NOT ENTER HERE"),
                    }
                }
            }
            _ => self.base.handle_query(request),
        }
    }
}

impl WasmMockQuerier {
    pub fn new(base: MockQuerier<TerraQueryWrapper>) -> Self {
        WasmMockQuerier {
            base,
            token_querier: TokenQuerier::default(),
            tax_querier: TaxQuerier::default(),
        }
    }

    // configure the mint whitelist mock querier
    pub fn with_token_balances(&mut self, balances: &[(&String, &[(&String, &Uint128)])]) {
        self.token_querier = TokenQuerier::new(balances);
    }
}
