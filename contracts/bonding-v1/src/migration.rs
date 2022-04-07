use cosmwasm_std::{CanonicalAddr, Decimal, StdResult, Storage, Uint128};
use cw_storage_plus::Item;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use services::bonding::BondingModeMsg;

static CONFIGV100: Item<ConfigV100> = Item::new("config");

/// ## Description
/// This structure describes a contract migration message.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct MigrationMsgV100 {
    pub bonding_mode: BondingModeMsg,
}

/// ## Description
/// This structure describes the outdate config of bonding contract.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct ConfigV100 {
    /// contract/multisig address that allowed to control settings
    pub owner: CanonicalAddr,
    /// bro token address
    pub bro_token: CanonicalAddr,
    /// bro/ust lp token address
    pub lp_token: CanonicalAddr,
    /// rewards pool address
    pub rewards_pool_contract: CanonicalAddr,
    /// treasury contract address
    pub treasury_contract: CanonicalAddr,
    /// astroport factory contract address
    pub astroport_factory: CanonicalAddr,
    /// price oracle contract address
    pub oracle_contract: CanonicalAddr,
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
    /// sets lp bonding option either to enabled or disabled
    pub lp_bonding_enabled: bool,
}

// ## Description
/// Returns outdated config struct of type [`ConfigV100`]
/// ## Params
/// * **storage** is an object of type [`Storage`]
pub fn load_config_v100(storage: &dyn Storage) -> StdResult<ConfigV100> {
    CONFIGV100.load(storage)
}
