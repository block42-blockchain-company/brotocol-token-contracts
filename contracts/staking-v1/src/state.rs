use cosmwasm_std::{Addr, CanonicalAddr, Decimal, QuerierWrapper, StdResult, Storage, Uint128};
use cw20::Expiration;
use cw_storage_plus::{Item, Map};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use services::querier::query_epoch_info;

static CONFIG: Item<Config> = Item::new("config");
static STATE: Item<State> = Item::new("state");
static STAKERS: Map<&[u8], StakerInfo> = Map::new("stakers");
static WITHDRAWALS: Map<&[u8], Vec<WithdrawalInfo>> = Map::new("withdrawals");

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Config {
    pub bro_token: CanonicalAddr,
    pub rewards_pool_contract: CanonicalAddr,
    pub bbro_minter_contract: CanonicalAddr,
    pub epoch_manager_contract: CanonicalAddr,
    pub unbond_period_blocks: u64,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct State {
    pub global_reward_index: Decimal,
    pub total_bond_amount: Uint128,
    pub last_distribution_block: u64,
}

impl State {
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

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct StakerInfo {
    pub reward_index: Decimal,
    pub bond_amount: Uint128,
    pub pending_reward: Uint128,
    pub last_balance_update: u64,
}

impl StakerInfo {
    pub fn compute_staker_reward(&mut self, state: &State) -> StdResult<()> {
        let pending_reward = (self.bond_amount * state.global_reward_index)
            .checked_sub(self.bond_amount * self.reward_index)?;

        self.reward_index = state.global_reward_index;
        self.pending_reward += pending_reward;
        Ok(())
    }

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

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct WithdrawalInfo {
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

pub fn store_staker_info(
    storage: &mut dyn Storage,
    staker: &CanonicalAddr,
    info: &StakerInfo,
) -> StdResult<()> {
    STAKERS.save(storage, staker.as_slice(), info)
}

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

pub fn remove_staker_info(storage: &mut dyn Storage, staker: &CanonicalAddr) {
    STAKERS.remove(storage, staker.as_slice())
}

#[allow(clippy::ptr_arg)]
pub fn store_withdrawals(
    storage: &mut dyn Storage,
    staker: &CanonicalAddr,
    claims: &Vec<WithdrawalInfo>,
) -> StdResult<()> {
    WITHDRAWALS.save(storage, staker.as_slice(), claims)
}

pub fn load_withdrawals(
    storage: &dyn Storage,
    staker: &CanonicalAddr,
) -> StdResult<Vec<WithdrawalInfo>> {
    WITHDRAWALS
        .may_load(storage, staker.as_slice())
        .map(|res| res.unwrap_or_default())
}
