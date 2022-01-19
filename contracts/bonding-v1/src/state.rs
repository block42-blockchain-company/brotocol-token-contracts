use cosmwasm_std::{Storage, StdResult, CanonicalAddr, Uint128, Decimal};
use cw20::Expiration;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use cw_storage_plus::{Item, Map};

static CONFIG: Item<Config> = Item::new("config");
static STATE: Item<State> = Item::new("state");
static CLAIMS: Map<&[u8], Vec<ClaimInfo>> = Map::new("claims");

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Config {
    pub owner: CanonicalAddr,
    pub bro_token: CanonicalAddr,
    pub lp_token: CanonicalAddr,
    pub treasury_contract: CanonicalAddr,
    pub astroport_factory: CanonicalAddr,
    pub ust_bonding_reward_ratio: Decimal,
    pub ust_bonding_discount: Decimal,
    pub lp_bonding_discount: Decimal,
    pub vesting_period_blocks: u64,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct State {
    pub ust_bonding_balance: Uint128,
    pub lp_bonding_balance: Uint128,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum BondType {
    UstBond,
    LpBond,
}

impl ToString for BondType {
    fn to_string(&self) -> String {
        match self {
            BondType::UstBond => "ust_bond".to_string(),
            BondType::LpBond => "lp_bond".to_string(),
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct ClaimInfo {
    pub bond_type: BondType,
    pub amount: Uint128,
    pub claimable_at: Expiration,
}

pub fn store_config(storage: &mut dyn Storage, config: &Config) -> StdResult<()> {
    CONFIG.save(storage, config)
}

pub fn load_config(storage: &dyn Storage) -> StdResult<Config> {
    CONFIG.load(storage)
}

pub fn store_state(storage: &mut dyn Storage, state: &State) -> StdResult<()> {
    STATE.save(storage, state)
}

pub fn load_state(storage: &dyn Storage) -> StdResult<State> {
    STATE.load(storage)
}

pub fn store_claims(storage: &mut dyn Storage, account: &CanonicalAddr, claims: &Vec<ClaimInfo>) -> StdResult<()> {
    CLAIMS.save(storage, &account.as_slice(), claims)
}

pub fn load_claims(storage: &dyn Storage, account: &CanonicalAddr) -> StdResult<Vec<ClaimInfo>> {
    CLAIMS.may_load(storage, account.as_slice()).map(|res| res.unwrap_or_default())
}
