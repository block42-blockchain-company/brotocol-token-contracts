use cosmwasm_std::{OverflowError, StdError};
use thiserror::Error;

/// ## Description
/// This enum describes staking contract errors
#[derive(Error, Debug)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("{0}")]
    OverflowError(#[from] OverflowError),

    #[error("Unauthorized")]
    Unauthorized {},

    #[error("Invalid receive hook msg")]
    InvalidHookData {},

    #[error("Staking amount must be higher than zero")]
    StakingAmountMustBeHigherThanZero {},

    #[error("Cannot unstake more than unlocked staked amount")]
    ForbiddenToUnstakeMoreThanUnlocked {},

    #[error("Nothing to claim")]
    NothingToClaim {},

    #[error("Invalid lockup period")]
    InvalidLockupPeriod {},

    #[error("Forbidden to lockup more than unlocked")]
    ForbiddenToLockupMoreThanUnlocked {},
}
