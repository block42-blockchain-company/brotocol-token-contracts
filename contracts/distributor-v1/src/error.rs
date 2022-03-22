use cosmwasm_std::{OverflowError, StdError};
use thiserror::Error;

use services::ownership_proposal::OwnershipProposalError;

/// ## Description
/// This enum describes distributor contract errors
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

    #[error("Distribution is not started yet")]
    DistributionIsNotStartedYet {},

    #[error("No rewards, epoch didn't pass yet")]
    NoRewards {},

    #[error("Rewards pool balance is lower than distribution amount")]
    NotEnoughBalanceForRewards {},

    #[error("Contract is paused")]
    ContractIsPaused {},
}
