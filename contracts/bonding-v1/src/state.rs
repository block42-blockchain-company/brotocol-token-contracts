use std::str::FromStr;

use cosmwasm_std::{
    Api, CanonicalAddr, Decimal, QuerierWrapper, StdError, StdResult, Storage, Uint128,
};
use cw20::Expiration;
use cw_storage_plus::{Item, Map};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::ContractError;

use services::{bonding::BondingModeMsg, querier::query_staking_config};

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
    /// rewards pool address
    pub rewards_pool_contract: CanonicalAddr,
    /// treasury contract address
    pub treasury_contract: CanonicalAddr,
    /// astroport factory contract address
    pub astroport_factory: CanonicalAddr,
    /// price oracle contract address
    pub oracle_contract: CanonicalAddr,
    /// discount percentage for ust bonding
    pub ust_bonding_discount: Decimal,
    /// minimum amount of bro to receive via bonding
    pub min_bro_payout: Uint128,
    /// bonding mode
    pub bonding_mode: BondingMode,
}

impl Config {
    pub fn validate(&self) -> StdResult<()> {
        let one = Decimal::from_str("1.0")?;

        if self.ust_bonding_discount > one || self.ust_bonding_discount <= Decimal::zero() {
            return Err(StdError::generic_err(
                "ust_bonding_discount must be less than 1.0 and non-negative",
            ));
        }

        match self.bonding_mode {
            BondingMode::Normal {
                ust_bonding_reward_ratio,
                lp_bonding_discount,
                vesting_period_blocks,
                ..
            } => {
                if ust_bonding_reward_ratio > one || ust_bonding_reward_ratio <= Decimal::zero() {
                    return Err(StdError::generic_err(
                        "ust_bonding_reward_ratio must be less than 1.0 and non-negative",
                    ));
                }

                if lp_bonding_discount > one || lp_bonding_discount <= Decimal::zero() {
                    return Err(StdError::generic_err(
                        "lp_bonding_discount must be less than 1.0 and non-negative",
                    ));
                }

                if vesting_period_blocks == 0 {
                    return Err(StdError::generic_err(
                        "vesting_period_blocks must be greater than zero",
                    ));
                }
            }
            BondingMode::Community { .. } => {}
        }

        Ok(())
    }
}

/// ## Description
/// This structure describes the bonding contract mode.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum BondingMode {
    /// ## Description
    /// Enables both ust and lp bonding option.
    /// Exchanged bro tokens will become claimable after vesting period.
    Normal {
        /// distributed reward percentage for ust bonding balance
        ust_bonding_reward_ratio: Decimal,
        /// bro/ust lp token address
        lp_token: CanonicalAddr,
        /// discount percentage for lp bonding
        lp_bonding_discount: Decimal,
        /// vesting period for withdrawal
        vesting_period_blocks: u64,
    },
    /// ## Description
    /// Enables only ust bonding option.
    /// Exchanged bro tokens will be locked in staking contract for configured amount of epochs
    /// and then claimable with extra bro/bbro reward from it.
    Community {
        /// staking contract address
        staking_contract: CanonicalAddr,
        /// how many epochs specified amount will be locked
        epochs_locked: u64,
    },
}

impl BondingMode {
    pub fn from_msg(
        mode: BondingModeMsg,
        querier: &QuerierWrapper,
        api: &dyn Api,
    ) -> Result<Self, ContractError> {
        match mode {
            BondingModeMsg::Normal {
                ust_bonding_reward_ratio,
                lp_bonding_discount,
                lp_token,
                vesting_period_blocks,
            } => Ok(BondingMode::Normal {
                ust_bonding_reward_ratio,
                lp_bonding_discount,
                lp_token: api.addr_canonicalize(&lp_token)?,
                vesting_period_blocks,
            }),
            BondingModeMsg::Community {
                staking_contract,
                epochs_locked,
            } => {
                let staking_config =
                    query_staking_config(querier, api.addr_validate(&staking_contract)?)?;
                if epochs_locked < staking_config.lockup_config.min_lockup_period_epochs
                    || epochs_locked > staking_config.lockup_config.max_lockup_period_epochs
                {
                    return Err(ContractError::InvalidLockupPeriodForCommunityBondingMode {});
                }

                Ok(BondingMode::Community {
                    staking_contract: api.addr_canonicalize(&staking_contract)?,
                    epochs_locked,
                })
            }
        }
    }
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

impl Default for State {
    fn default() -> Self {
        State {
            ust_bonding_balance: Uint128::zero(),
            lp_bonding_balance: Uint128::zero(),
        }
    }
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
