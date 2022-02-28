use cosmwasm_std::{
    Addr, BlockInfo, CanonicalAddr, Decimal, QuerierWrapper, StdResult, Storage, Uint128,
};
use cw20::Expiration;
use cw_storage_plus::{Item, Map};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::math::{decimal_mul_in_256, decimal_sub_in_256, decimal_sum_in_256};

use services::querier::query_epoch_info;

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
    /// block at which amount will be unlocked
    pub unlocked_at: Expiration,
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
    /// Computes staking reward and adds it to pending_reward
    pub fn compute_staking_reward(&mut self, state: &State) -> StdResult<()> {
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
        querier: &QuerierWrapper,
        epoch_manager_contract: Addr,
        state: &State,
        current_block: u64,
    ) -> StdResult<()> {
        let stake_amount = self.total_staked()?;

        if stake_amount.is_zero() || state.last_distribution_block < self.last_balance_update {
            return Ok(());
        }

        let epoch_info = query_epoch_info(querier, epoch_manager_contract)?;

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
        querier: &QuerierWrapper,
        epoch_manager_contract: Addr,
        current_block: u64,
        amount: Uint128,
        epochs_locked: u64,
    ) -> StdResult<()> {
        let epoch_blocks = query_epoch_info(querier, epoch_manager_contract)?.epoch;
        let unlocked_at_block = current_block + (epoch_blocks * epochs_locked);

        self.locked_stake_amount = self.locked_stake_amount.checked_add(amount)?;
        self.lockups.push(LockupInfo {
            amount,
            unlocked_at: Expiration::AtHeight(unlocked_at_block),
        });

        Ok(())
    }

    /// ## Description
    /// Removes passed lockups from list and updates balances
    pub fn unlock_expired_lockups(&mut self, current_block: &BlockInfo) -> StdResult<()> {
        let mut unlocked_amount = Uint128::zero();
        let lockups: Vec<LockupInfo> = self
            .lockups
            .clone()
            .into_iter()
            .filter(|l| {
                if l.unlocked_at.is_expired(current_block) {
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
