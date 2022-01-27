use cosmwasm_std::{CanonicalAddr, Decimal, StdResult, Storage, Uint128};
use cw20::Expiration;
use cw_storage_plus::{Item, Map};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// ## Description
/// Stores config struct of type [`Config`] at the given key
static CONFIG: Item<Config> = Item::new("config");

/// ## Description
/// Stores state struct of type [`State`] at the given key
static STATE: Item<State> = Item::new("state");

/// ## Description
/// A map which stores bonder claims info with [`CanonicalAddr`] type as key and [`Vec<ClaimInfo>`] type as value
static CLAIMS: Map<&[u8], Vec<ClaimInfo>> = Map::new("claims");

/// ## Description
/// This structure describes the main control config of bonding contract.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Config {
    /// contract/multisig address that allowed to control settings
    pub owner: CanonicalAddr,
    /// bro token address
    pub bro_token: CanonicalAddr,
    /// bro/ust lp token address
    pub lp_token: CanonicalAddr,
    /// treasury contract address
    pub treasury_contract: CanonicalAddr,
    /// astroport factory contract address
    pub astroport_factory: CanonicalAddr,
    /// distributed reward percentage for ust bonding balance
    pub ust_bonding_reward_ratio: Decimal,
    /// discount percentage for ust bonding
    pub ust_bonding_discount: Decimal,
    /// discount percentage for lp bonding
    pub lp_bonding_discount: Decimal,
    /// minimum amount of bro to receive via bonding
    pub min_bro_payout: Uint128,
    /// vesting period for withdrawal
    pub vesting_period_blocks: u64,
}

/// ## Description
/// This structure describes state of bonding contract.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct State {
    /// available bro balance for ust bonding
    pub ust_bonding_balance: Uint128,
    /// available bro balance for lp token bonding
    pub lp_bonding_balance: Uint128,
}

/// ## Description
/// This structure describes bond type.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum BondType {
    /// ust bond type
    UstBond,
    /// lp token bond type
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

/// ## Description
/// This structure describes claim info of bonding contract.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct ClaimInfo {
    /// bond type
    pub bond_type: BondType,
    /// amount to claim
    pub amount: Uint128,
    /// block at which amount can be claimed
    pub claimable_at: Expiration,
}

/// ## Description
/// Saves changes of [`Config`] struct in [`CONFIG`] storage
/// ## Params
/// * **storage** is an object of type [`Storage`]
///
/// * **config** updated config struct of type [`Config`]
pub fn store_config(storage: &mut dyn Storage, config: &Config) -> StdResult<()> {
    CONFIG.save(storage, config)
}

// ## Description
/// Returns config struct of type [`Config`]
/// ## Params
/// * **storage** is an object of type [`Storage`]
pub fn load_config(storage: &dyn Storage) -> StdResult<Config> {
    CONFIG.load(storage)
}

/// ## Description
/// Saves changes of [`State`] struct in [`STATE`] storage
/// ## Params
/// * **storage** is an object of type [`Storage`]
///
/// * **state** updated config struct of type [`State`]
pub fn store_state(storage: &mut dyn Storage, state: &State) -> StdResult<()> {
    STATE.save(storage, state)
}

/// ## Description
/// Returns state struct of type [`State`]
/// ## Params
/// * **storage** is an object of type [`Storage`]
pub fn load_state(storage: &dyn Storage) -> StdResult<State> {
    STATE.load(storage)
}

/// ## Description
/// Saves or updates changes in [`CLAIMS`] map for specified key of type [`CanonicalAddr`] and value of type [`Vec<ClaimInfo>`]
/// ## Params
/// * **storage** is an object of type [`Storage`]
///
/// * **staker** is an object of type [`CanonicalAddr`]
///
/// * **claims** is an [`Vec`] of type [`ClaimInfo`]
#[allow(clippy::ptr_arg)]
pub fn store_claims(
    storage: &mut dyn Storage,
    account: &CanonicalAddr,
    claims: &Vec<ClaimInfo>,
) -> StdResult<()> {
    CLAIMS.save(storage, account.as_slice(), claims)
}

/// ## Description
/// Returns bonder claims info object of type [`Vec<WithdrawalInfo>`] by specified key of type [`CanonicalAddr`] from map [`CLAIMS`]
/// ## Params
/// * **storage** is an object of type [`Storage`]
///
/// * **staker** is an object of type [`CanonicalAddr`]
pub fn load_claims(storage: &dyn Storage, account: &CanonicalAddr) -> StdResult<Vec<ClaimInfo>> {
    CLAIMS
        .may_load(storage, account.as_slice())
        .map(|res| res.unwrap_or_default())
}
