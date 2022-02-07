use cosmwasm_std::testing::{MockApi, MockQuerier, MockStorage, MOCK_CONTRACT_ADDR};
use cosmwasm_std::{
    from_binary, from_slice, to_binary, Addr, Coin, OwnedDeps, Querier, QuerierResult,
    QueryRequest, SystemError, SystemResult, Uint128, WasmQuery,
};
use std::collections::HashMap;

use terra_cosmwasm::TerraQueryWrapper;

use astroport::{
    asset::{Asset, PairInfo},
    factory::{PairType, QueryMsg as FactoryQueryMsg},
    pair::{CumulativePricesResponse, QueryMsg as PairQueryMsg},
};

pub const MOCK_FACTORY_ADDR: &str = "astrofactory";
pub const MOCK_PAIR_ADDR: &str = "broustpair";

pub fn mock_dependencies(
    contract_balance: &[Coin],
) -> OwnedDeps<MockStorage, MockApi, WasmMockQuerier> {
    let custom_querier: WasmMockQuerier =
        WasmMockQuerier::new(MockQuerier::new(&[(MOCK_CONTRACT_ADDR, contract_balance)]));

    OwnedDeps {
        storage: MockStorage::default(),
        api: MockApi::default(),
        querier: custom_querier,
    }
}

pub struct WasmMockQuerier {
    base: MockQuerier<TerraQueryWrapper>,
    token_querier: TokenQuerier,
}

#[derive(Clone, Default)]
pub struct TokenQuerier {
    // this lets us iterate over all pairs that match the first string
    pairs: HashMap<String, CumulativePricesResponse>,
}

impl TokenQuerier {
    pub fn set(
        &mut self,
        pair: Addr,
        assets: [Asset; 2],
        total: Uint128,
        price0: Uint128,
        price1: Uint128,
    ) {
        self.pairs = HashMap::new();
        self.pairs.insert(
            pair.to_string(),
            CumulativePricesResponse {
                assets,
                total_share: total,
                price0_cumulative_last: price0,
                price1_cumulative_last: price1,
            },
        );
    }
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
            QueryRequest::Wasm(WasmQuery::Smart { contract_addr, msg }) => {
                if contract_addr == MOCK_FACTORY_ADDR {
                    match from_binary(&msg).unwrap() {
                        FactoryQueryMsg::Pair { asset_infos } => SystemResult::Ok(
                            to_binary(&PairInfo {
                                asset_infos,
                                contract_addr: Addr::unchecked(MOCK_PAIR_ADDR),
                                liquidity_token: Addr::unchecked("lp_token"),
                                pair_type: PairType::Xyk {},
                            })
                            .into(),
                        ),
                        _ => panic!("DO NOT ENTER HERE"),
                    }
                } else {
                    match from_binary(&msg).unwrap() {
                        PairQueryMsg::CumulativePrices { .. } => {
                            let balance = match self.token_querier.pairs.get(contract_addr) {
                                Some(v) => v,
                                None => {
                                    return SystemResult::Err(SystemError::Unknown {});
                                }
                            };
                            SystemResult::Ok(to_binary(&balance).into())
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
        }
    }

    pub fn set_cumulative_price(
        &mut self,
        pair: Addr,
        assets: [Asset; 2],
        total: Uint128,
        price0: Uint128,
        price1: Uint128,
    ) {
        self.token_querier.set(pair, assets, total, price0, price1)
    }
}
