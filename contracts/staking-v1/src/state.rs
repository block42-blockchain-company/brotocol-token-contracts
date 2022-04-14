use std::str::FromStr;

use cosmwasm_std::{BlockInfo, CanonicalAddr, Decimal, StdError, StdResult, Storage, Uint128};
use cw20::Expiration;
use cw_storage_plus::{Item, Map};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::math::{decimal_mul_in_256, decimal_sub_in_256, decimal_sum_in_256};

use services::epoch_manager::EpochInfoResponse;

/// ## Description
/// Stores config struct of type [`Config`] at the given key
static CONFIG: Item<Config> = Item::new("config");

/// ## Description
/// Stores state struct of type [`State`] at the given key
static STATE: Item<State> = Item::new("state");

/// ## Description
/// A map which stores stakers info with [`CanonicalAddr`] type as key and [`StakerInfo`] type as value
static STAKERS: Map<&[u8], StakerInfo> = Map::new("stakers");

/// ## Description
/// A map which stores staker withdrawals info with [`CanonicalAddr`] type as key and [`Vec<WithdrawalInfo>`] type as value
static WITHDRAWALS: Map<&[u8], Vec<WithdrawalInfo>> = Map::new("withdrawals");

/// ## Description
/// This structure describes the main control config of staking contract.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Config {
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
    /// community bonding address,
    /// if value is set to none
    /// than option to stake from community bonding contract is disabled
    pub community_bonding_contract: Option<CanonicalAddr>,
    /// vesting period for withdrawal
    pub unstake_period_blocks: u64,
    /// minimum staking amount
    pub min_staking_amount: Uint128,
    /// lockup config
    pub lockup_config: LockupConfig,
}

impl Config {
    pub fn validate(&self) -> StdResult<()> {
        let min_base_rate = Decimal::from_str("0.0001")?;
        if self.lockup_config.base_rate < min_base_rate {
            return Err(StdError::generic_err(
                "base_rate must be higher than min_base_rate",
            ));
        }

        let max_base_rate = Decimal::from_str("0.0005")?;
        if self.lockup_config.base_rate > max_base_rate {
            return Err(StdError::generic_err(
                "base_rate must be smaller than max_base_rate",
            ));
        }

        let min_linear_growth = Decimal::from_str("0.0004")?;
        if self.lockup_config.linear_growth < min_linear_growth {
            return Err(StdError::generic_err(
                "linear_growth must be higher than min_linear_growth",
            ));
        }

        let max_linear_growth = Decimal::from_str("0.0015")?;
        if self.lockup_config.linear_growth > max_linear_growth {
            return Err(StdError::generic_err(
                "linear_growth must be smaller than max_linear_growth",
            ));
        }

        let min_exponential_growth = Decimal::from_str("0.000001")?;
        if self.lockup_config.exponential_growth < min_exponential_growth {
            return Err(StdError::generic_err(
                "exponential_growth must be higher than min_exponential_growth",
            ));
        }

        let max_exponential_growth = Decimal::from_str("0.000015")?;
        if self.lockup_config.exponential_growth > max_exponential_growth {
            return Err(StdError::generic_err(
                "exponential_growth must be higher than max_exponential_growth",
            ));
        }

        if self.lockup_config.min_lockup_period_epochs > self.lockup_config.max_lockup_period_epochs
        {
            return Err(StdError::generic_err(
                "min_lockup_period_epochs must be less then or equal to max_lockup_period_epochs",
            ));
        }

        Ok(())
    }
}

/// ## Description
/// This structure describes the lockup config of staking contract.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct LockupConfig {
    /// min lockup period
    pub min_lockup_period_epochs: u64,
    /// max lockup period
    pub max_lockup_period_epochs: u64,
    /// base rate for bbro premium reward calculation
    pub base_rate: Decimal,
    /// linear growth for bbro premium reward calculation
    pub linear_growth: Decimal,
    /// exponential growth for bbro premium reward calculation
    pub exponential_growth: Decimal,
}

impl LockupConfig {
    /// ## Description
    /// Validates that passed lockup period is valid
    pub fn valid_lockup_period(&self, epochs_locked: u64) -> bool {
        if epochs_locked < self.min_lockup_period_epochs
            || epochs_locked > self.max_lockup_period_epochs
        {
            return false;
        }

        true
    }
}

/// ## Description
/// This structure describes the state of staking contract.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct State {
    /// global reward index for BRO staking rewards
    pub global_reward_index: Decimal,
    /// total amount of staked BRO tokens by all stakers
    pub total_stake_amount: Uint128,
    /// last rewards distribution block
    pub last_distribution_block: u64,
}

/// ## Description
/// This structure describes the lockup info of staking contract.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct LockupInfo {
    /// locked amount
    pub amount: Uint128,
    /// block at whick locup was created
    pub locked_at_block: u64,
    /// amount of epochs until lockup will be unlocked
    pub epochs_locked: u64,
}

/// ## Description
/// This structure describes the staker info of staking contract.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct StakerInfo {
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
    pub lockups: Vec<LockupInfo>,
}

impl StakerInfo {
    /// ## Description
    /// Returns total staked amount(unlocked+locked)
    pub fn total_staked(&self) -> StdResult<Uint128> {
        Ok(self
            .unlocked_stake_amount
            .checked_add(self.locked_stake_amount)?)
    }

    /// ## Description
    /// Computes bro staking reward and adds it to pending_reward
    pub fn compute_bro_reward(&mut self, state: &State) -> StdResult<()> {
        let stake_amount = self.total_staked()?;
        let pending_bro_reward = (stake_amount * state.global_reward_index)
            .checked_sub(stake_amount * self.reward_index)?;

        self.reward_index = state.global_reward_index;
        self.pending_bro_reward = self.pending_bro_reward.checked_add(pending_bro_reward)?;
        Ok(())
    }

    /// ## Description
    /// Computes normal bbro reward for staked BRO
    pub fn compute_normal_bbro_reward(
        &mut self,
        epoch_info: &EpochInfoResponse,
        state: &State,
        current_block: u64,
    ) -> StdResult<()> {
        let stake_amount = self.total_staked()?;

        if stake_amount.is_zero() || state.last_distribution_block < self.last_balance_update {
            return Ok(());
        }

        let epochs_staked = Uint128::from(state.last_distribution_block - self.last_balance_update)
            .checked_div(Uint128::from(epoch_info.epoch))?;

        let bbro_per_epoch_reward =
            stake_amount.checked_div(epoch_info.epochs_per_year())? * epoch_info.bbro_emission_rate;

        let bbro_reward = bbro_per_epoch_reward.checked_mul(epochs_staked)?;
        self.pending_bbro_reward = self.pending_bbro_reward.checked_add(bbro_reward)?;
        self.last_balance_update = current_block;

        Ok(())
    }

    /// ## Description
    /// Computes premium bbro reward when locking staked BRO using next formula:
    /// ((base_rate+linear_growth*epochs_locked+exponential_growth*epochs_locked^2)-0.0005)*bro_locked_amount
    pub fn compute_premium_bbro_reward(
        &self,
        lockup_config: &LockupConfig,
        epochs_locked: u64,
        amount: Uint128,
    ) -> Uint128 {
        let epochs_locked: u128 = epochs_locked.into();

        // epochs_locked * linear_growth
        let linear_growth = decimal_mul_in_256(
            lockup_config.linear_growth,
            Decimal::from_ratio(Uint128::from(epochs_locked), Uint128::from(1u128)),
        );

        // epochs_locked^2 * exponential_growth
        let exponential_growth = decimal_mul_in_256(
            lockup_config.exponential_growth,
            Decimal::from_ratio(
                Uint128::from((epochs_locked * epochs_locked) as u128),
                Uint128::from(1u128),
            ),
        );

        // (base_rate + linear_growth * epochs_locked + exponential_growth * epochs_locked^2) - 0.0005
        let bbro_rate = decimal_sub_in_256(
            decimal_sum_in_256(
                lockup_config.base_rate,
                decimal_sum_in_256(linear_growth, exponential_growth),
            ),
            Decimal::from_ratio(Uint128::from(5u128), Uint128::from(10000u128)),
        );

        // (...) * bro_locked_amount
        bbro_rate * amount
    }

    /// ## Description
    /// Adds new lockup period for staker
    pub fn add_lockup(
        &mut self,
        current_block: u64,
        amount: Uint128,
        epochs_locked: u64,
    ) -> StdResult<()> {
        self.locked_stake_amount = self.locked_stake_amount.checked_add(amount)?;
        self.lockups.push(LockupInfo {
            amount,
            locked_at_block: current_block,
            epochs_locked,
        });

        Ok(())
    }

    /// ## Description
    /// Removes passed lockups from list and updates balances
    pub fn unlock_expired_lockups(
        &mut self,
        current_block: &BlockInfo,
        epoch_info: &EpochInfoResponse,
    ) -> StdResult<()> {
        let mut unlocked_amount = Uint128::zero();
        let lockups: Vec<LockupInfo> = self
            .lockups
            .clone()
            .into_iter()
            .filter(|l| {
                let unlocked_at_block = l.locked_at_block + (l.epochs_locked * epoch_info.epoch);
                if current_block.height >= unlocked_at_block {
                    unlocked_amount += l.amount;
                    false
                } else {
                    true
                }
            })
            .collect();

        self.locked_stake_amount = self.locked_stake_amount.checked_sub(unlocked_amount)?;
        self.unlocked_stake_amount = self.unlocked_stake_amount.checked_add(unlocked_amount)?;
        self.lockups = lockups;

        Ok(())
    }

    /// ## Description
    /// Checks if staker info can be deleted or not
    pub fn can_be_removed(&self) -> StdResult<bool> {
        Ok(self.total_staked()?.is_zero()
            && self.pending_bro_reward.is_zero()
            && self.pending_bbro_reward.is_zero())
    }
}

/// ## Description
/// This structure describes withdrawal info of staking contract.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct WithdrawalInfo {
    /// amount to withdraw
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

/// ## Description
/// Returns config struct of type [`Config`]
/// ## Params
/// * **storage** is an object of type [`Storage`]
pub fn load_config(storage: &dyn Storage) -> StdResult<Config> {
    CONFIG.load(storage)
}

/// ## Description
/// Updates owner field in [`Config`] object
/// ## Params
/// * **storage** is an object of type [`Storage`]
///
/// * **new_owner** is an object of type [`CanonicalAddr`]
pub fn update_owner(storage: &mut dyn Storage, new_owner: CanonicalAddr) -> StdResult<()> {
    CONFIG.update::<_, StdError>(storage, |mut c| {
        c.owner = new_owner;
        Ok(c)
    })?;

    Ok(())
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
/// Saves or updates changes in [`STAKERS`] map for specified key of type [`CanonicalAddr`] and value of type [`StakerInfo`]
/// ## Params
/// * **storage** is an object of type [`Storage`]
///
/// * **staker** is an object of type [`CanonicalAddr`]
///
/// * **info** is an object of type [`StakerInfo`]
pub fn store_staker_info(
    storage: &mut dyn Storage,
    staker: &CanonicalAddr,
    info: &StakerInfo,
) -> StdResult<()> {
    STAKERS.save(storage, staker.as_slice(), info)
}

/// ## Description
/// Returns staker info object of type [`StakerInfo`] by specified key of type [`CanonicalAddr`] from map [`STAKERS`]
/// ## Params
/// * **storage** is an object of type [`Storage`]
///
/// * **staker** is an object of type [`CanonicalAddr`]
///
/// * **current_block** is a field of type [`u64`]
pub fn read_staker_info(
    storage: &dyn Storage,
    staker: &CanonicalAddr,
    current_block: u64,
) -> StdResult<StakerInfo> {
    let res: Option<StakerInfo> = STAKERS.may_load(storage, staker.as_slice())?;

    match res {
        Some(info) => Ok(info),
        None => Ok(StakerInfo {
            reward_index: Decimal::zero(),
            unlocked_stake_amount: Uint128::zero(),
            locked_stake_amount: Uint128::zero(),
            pending_bro_reward: Uint128::zero(),
            pending_bbro_reward: Uint128::zero(),
            last_balance_update: current_block,
            lockups: vec![],
        }),
    }
}

/// ## Description
/// Removes staker info object of type [`StakerInfo`] by specified key of type [`CanonicalAddr`] from map [`STAKERS`]
/// ## Params
/// * **storage** is an object of type [`Storage`]
///
/// * **staker** is an object of type [`CanonicalAddr`]
pub fn remove_staker_info(storage: &mut dyn Storage, staker: &CanonicalAddr) {
    STAKERS.remove(storage, staker.as_slice())
}

/// ## Description
/// Saves or updates changes in [`WITHDRAWALS`] map for specified key of type [`CanonicalAddr`] and value of type [`Vec<WithdrawalInfo>`]
/// ## Params
/// * **storage** is an object of type [`Storage`]
///
/// * **staker** is an object of type [`CanonicalAddr`]
///
/// * **claims** is an [`Vec`] of type [`WithdrawalInfo`]
#[allow(clippy::ptr_arg)]
pub fn store_withdrawals(
    storage: &mut dyn Storage,
    staker: &CanonicalAddr,
    claims: &Vec<WithdrawalInfo>,
) -> StdResult<()> {
    WITHDRAWALS.save(storage, staker.as_slice(), claims)
}

/// ## Description
/// Returns stakers withdrawal info object of type [`Vec<WithdrawalInfo>`] by specified key of type [`CanonicalAddr`] from map [`WITHDRAWALS`]
/// ## Params
/// * **storage** is an object of type [`Storage`]
///
/// * **staker** is an object of type [`CanonicalAddr`]
pub fn load_withdrawals(
    storage: &dyn Storage,
    staker: &CanonicalAddr,
) -> StdResult<Vec<WithdrawalInfo>> {
    WITHDRAWALS
        .may_load(storage, staker.as_slice())
        .map(|res| res.unwrap_or_default())
}
