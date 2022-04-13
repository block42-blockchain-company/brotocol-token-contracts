use cosmwasm_std::{CanonicalAddr, Decimal, Env, StdError, StdResult, Storage, Uint128};
use cw20::Expiration;
use cw_storage_plus::{Item, Map};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::state::{store_staker_info, LockupConfig, LockupInfo, StakerInfo};

/// ## Description
/// Stores outdated config struct of type [`ConfigV100`] at the given key
static CONFIGV100: Item<ConfigV100> = Item::new("config");

/// ## Description
/// A map which stores outdated stakers info with [`CanonicalAddr`] type as key and [`StakerInfo`] type as value
static STAKERSV100: Map<&[u8], StakerInfoV100> = Map::new("stakers");

/// ## Description
/// This structure describes a contract migration message.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct MigrationMsgV100 {
    /// community bonding address
    pub community_bonding_contract: Option<String>,
    /// amount of blocks in epoch
    pub current_epoch_blocks: u64,
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
/// This structure describes the outdated lockup info of staking contract.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct LockupInfoV100 {
    /// locked amount
    pub amount: Uint128,
    /// block at which amount will be unlocked
    pub unlocked_at: Expiration,
}

/// ## Description
/// This structure describes the outdated staker info of staking contract.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct StakerInfoV100 {
    /// reward index of staker
    pub reward_index: Decimal,
    /// amount of unlocked BRO tokens staked by staker
    pub unlocked_stake_amount: Uint128,
    /// amount of locked BRO tokens staked by staker
    pub locked_stake_amount: Uint128,
    /// amount of pending bro rewards of staker
    pub pending_bro_reward: Uint128,
    /// amount of pending bbro rewards of staker
    pub pending_bbro_reward: Uint128,
    /// last balance update(stake, unstake) block
    pub last_balance_update: u64,
    /// amounts locked for specified amount of epochs
    pub lockups: Vec<LockupInfoV100>,
}

/// ## Description
/// Returns outdated config struct of type [`ConfigV100`]
/// ## Params
/// * **storage** is an object of type [`Storage`]
pub fn load_config_v100(storage: &dyn Storage) -> StdResult<ConfigV100> {
    CONFIGV100.load(storage)
}

/// ## Description
/// Returns outdated stakers info of type [`Vec<(CanonicalAddr, StakerInfoV100)>`]
/// ## Params
/// * **storage** is an object of type [`Storage`]
fn read_stakers_v100(storage: &dyn Storage) -> StdResult<Vec<(CanonicalAddr, StakerInfoV100)>> {
    STAKERSV100
        .range(storage, None, None, cosmwasm_std::Order::Ascending)
        .map(|item| {
            let (address, info) = item?;
            Ok((address.into(), info))
        })
        .collect()
}

/// ## Description
/// Migrates old staker_info config to the new version
pub fn migrate_stakers_info_to_v110(
    storage: &mut dyn Storage,
    env: Env,
    current_epoch_blocks: u64,
) -> StdResult<()> {
    let stakers_v100 = read_stakers_v100(storage)?;
    for (address, mut staker_info) in stakers_v100 {
        let mut lockups: Vec<LockupInfo> = vec![];
        for lockup_v100 in staker_info.lockups {
            let unlocked_at = match lockup_v100.unlocked_at {
                Expiration::AtHeight(height) => height,
                _ => return Err(StdError::generic_err("expecting Expiration::AtHeight")),
            };

            let remaining_epochs = (unlocked_at - env.block.height) / current_epoch_blocks;
            if lockup_v100.unlocked_at.is_expired(&env.block) || remaining_epochs == 0 {
                staker_info.unlocked_stake_amount += lockup_v100.amount;
                staker_info.locked_stake_amount -= lockup_v100.amount;
            } else {
                lockups.push(LockupInfo {
                    amount: lockup_v100.amount,
                    locked_at_block: env.block.height,
                    epochs_locked: remaining_epochs,
                });
            }
        }

        store_staker_info(
            storage,
            &address,
            &StakerInfo {
                reward_index: staker_info.reward_index,
                unlocked_stake_amount: staker_info.unlocked_stake_amount,
                locked_stake_amount: staker_info.locked_stake_amount,
                pending_bro_reward: staker_info.pending_bro_reward,
                pending_bbro_reward: staker_info.pending_bbro_reward,
                last_balance_update: staker_info.last_balance_update,
                lockups,
            },
        )?;
    }

    Ok(())
}
