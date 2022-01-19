use cosmwasm_std::{OverflowError, StdError};
use thiserror::Error;

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

    #[error("ust_bonding_reward_ratio must be less than 1.0 and non-negative")]
    InvalidUstBondRatio {},

    #[error("Invalid funds input")]
    InvalidFundsInput {},

    #[error("Insufficient BRO balance for bond payout")]
    NotEnoughForBondPayout {},

    #[error("Nothing to claim, wait for vesting period")]
    NothingToClaim {},
}
