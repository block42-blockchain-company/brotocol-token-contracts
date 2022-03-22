use cosmwasm_std::{OverflowError, StdError};
use thiserror::Error;

use services::ownership_proposal::OwnershipProposalError;

/// ## Description
/// This enum describes staking contract errors
#[derive(Error, Debug)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("{0}")]
    OverflowError(#[from] OverflowError),

    #[error("{0}")]
    OwnershipProposal(#[from] OwnershipProposalError),

    #[error("Unauthorized")]
    Unauthorized {},

    #[error("Contract is paused")]
    ContractIsPaused {},

    #[error("Invalid receive hook msg")]
    InvalidHookData {},

    #[error("Staking amount must be higher than min amount")]
    StakingAmountMustBeHigherThanMinAmount {},

    #[error("Cannot unstake more than unlocked staked amount")]
    ForbiddenToUnstakeMoreThanUnlocked {},

    #[error("Nothing to claim")]
    NothingToClaim {},

    #[error("Invalid lockup period")]
    InvalidLockupPeriod {},

    #[error("Lockup premium reward is zero")]
    LockupPremiumRewardIsZero {},

    #[error("Forbidden to lockup more than unlocked")]
    ForbiddenToLockupMoreThanUnlocked {},
}
