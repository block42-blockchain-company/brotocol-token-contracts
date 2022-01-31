use cosmwasm_std::{Addr, CanonicalAddr, Decimal, QuerierWrapper, StdResult, Storage, Uint128};
use cw20::Expiration;
use cw_storage_plus::{Item, Map};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
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
    /// bro token address
    pub bro_token: CanonicalAddr,
    /// rewards pool address
    pub rewards_pool_contract: CanonicalAddr,
    /// bbro minter address
    pub bbro_minter_contract: CanonicalAddr,
    /// epoch manager contract address
    pub epoch_manager_contract: CanonicalAddr,
    /// vesting period for withdrawal
    pub unbond_period_blocks: u64,
}

/// ## Description
/// This structure describes state of staking contract.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct State {
    /// global reward index for BRO staking rewards
    pub global_reward_index: Decimal,
    /// total amount of bonded BRO tokens by all stakers
    pub total_bond_amount: Uint128,
    /// last rewards distribution block
    pub last_distribution_block: u64,
}

impl State {
    /// ## Description
    /// Increases total bond amount and staker bond amount
    pub fn increase_bond_amount(
        &mut self,
        staker_info: &mut StakerInfo,
        amount: Uint128,
        current_block: u64,
    ) {
        self.total_bond_amount += amount;
        staker_info.bond_amount += amount;
        staker_info.last_balance_update = current_block;
    }

    /// ## Description
    /// Decreases total bond amount and staker bond amount
    pub fn decrease_bond_amount(
        &mut self,
        staker_info: &mut StakerInfo,
        amount: Uint128,
        current_block: u64,
    ) -> StdResult<()> {
        self.total_bond_amount = self.total_bond_amount.checked_sub(amount)?;
        staker_info.bond_amount = staker_info.bond_amount.checked_sub(amount)?;
        staker_info.last_balance_update = current_block;

        Ok(())
    }
}

/// ## Description
/// This structure describes staker info of staking contract.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct StakerInfo {
    /// reward index of staker
    pub reward_index: Decimal,
    /// amount of BRO tokens bonded by staker
    pub bond_amount: Uint128,
    /// amount of pending rewards of staker
    pub pending_reward: Uint128,
    /// last balance update(bond, unbond) block
    pub last_balance_update: u64,
}

impl StakerInfo {
    /// ## Description
    /// Computes staking reward and adds it to pending_reward
    pub fn compute_staker_reward(&mut self, state: &State) -> StdResult<()> {
        let pending_reward = (self.bond_amount * state.global_reward_index)
            .checked_sub(self.bond_amount * self.reward_index)?;

        self.reward_index = state.global_reward_index;
        self.pending_reward += pending_reward;
        Ok(())
    }

    /// ## Description
    /// Computes staker bbro reward
    pub fn compute_staker_bbro_reward(
        &self,
        querier: &QuerierWrapper,
        epoch_manager_contract: Addr,
        state: &State,
    ) -> StdResult<Uint128> {
        if self.bond_amount.is_zero() || state.last_distribution_block < self.last_balance_update {
            return Ok(Uint128::zero());
        }

        let epoch_info = query_epoch_info(querier, epoch_manager_contract)?;

        let epochs_staked = Uint128::from(state.last_distribution_block - self.last_balance_update)
            .checked_div(Uint128::from(epoch_info.epoch))?;

        let bbro_per_epoch_reward = self.bond_amount.checked_div(epoch_info.epochs_per_year())?
            * epoch_info.bbro_emission_rate;

        let bbro_reward = bbro_per_epoch_reward.checked_mul(epochs_staked)?;
        Ok(bbro_reward)
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
            bond_amount: Uint128::zero(),
            pending_reward: Uint128::zero(),
            last_balance_update: current_block,
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
