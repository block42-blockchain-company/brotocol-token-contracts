use cosmwasm_std::{OverflowError, StdError};
use thiserror::Error;

use services::ownership_proposal::OwnershipProposalError;

/// ## Description
/// This enum describes bonding contract errors
#[derive(Error, Debug)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("{0}")]
    OverflowError(#[from] OverflowError),

    #[error("{0}")]
    OwnershipProposal(#[from] OwnershipProposalError),

    #[error("Migration error")]
    MigrationError {},

    #[error("Unauthorized")]
    Unauthorized {},

    #[error("Invalid lockup period for Community bonding mode")]
    InvalidLockupPeriodForCommunityBondingMode {},

    #[error("Invalid receive hook msg")]
    InvalidHookData {},

    #[error("Invalid funds input")]
    InvalidFundsInput {},

    #[error("BRO bonding payout is too low")]
    BondPayoutIsTooLow {},

    #[error("Insufficient BRO balance for bond payout")]
    NotEnoughForBondPayout {},

    #[error("Nothing to claim, wait for vesting period")]
    NothingToClaim {},

    #[error("LP Token bonding disabled")]
    LpBondingDisabled {},
}
