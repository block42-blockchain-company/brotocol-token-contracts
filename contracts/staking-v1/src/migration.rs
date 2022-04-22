use cosmwasm_std::{CanonicalAddr, StdResult, Storage, Uint128};

use cw_storage_plus::Item;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::state::LockupConfig;

/// ## Description
/// Stores outdated config struct of type [`ConfigV100`] at the given key
static CONFIGV100: Item<ConfigV100> = Item::new("config");

/// ## Description
/// This structure describes a contract migration message.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct MigrationMsgV100 {
    /// community bonding address
    pub community_bonding_contract: Option<String>,
    /// amount of blocks in epoch
    pub prev_epoch_blocks: u64,
}

/// ## Description
/// This structure describes the main control config of staking contract.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct ConfigV100 {
    /// contract/multisig address that allowed to control settings
    pub owner: CanonicalAddr,
    /// defines whether the contract is paused or not
    pub paused: bool,
    /// bro token address
    pub bro_token: CanonicalAddr,
    /// rewards pool address
    pub rewards_pool_contract: CanonicalAddr,
    /// bbro minter address
    pub bbro_minter_contract: CanonicalAddr,
    /// epoch manager contract address
    pub epoch_manager_contract: CanonicalAddr,
    /// vesting period for withdrawal
    pub unstake_period_blocks: u64,
    /// minimum staking amount
    pub min_staking_amount: Uint128,
    /// lockup config
    pub lockup_config: LockupConfig,
}

/// ## Description
/// Returns outdated config struct of type [`ConfigV100`]
/// ## Params
/// * **storage** is an object of type [`Storage`]
pub fn load_config_v100(storage: &dyn Storage) -> StdResult<ConfigV100> {
    CONFIGV100.load(storage)
}
