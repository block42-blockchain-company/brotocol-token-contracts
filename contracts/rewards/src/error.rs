use cosmwasm_std::StdError;
use thiserror::Error;

use services::ownership_proposal::OwnershipProposalError;

/// ## Description
/// This enum describes rewards-pool contract errors
#[derive(Error, Debug)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("{0}")]
    OwnershipProposal(#[from] OwnershipProposalError),

    #[error("Unauthorized")]
    Unauthorized {},

    #[error("Cannot spend more than spend_limit")]
    SpendLimitReached {},

    #[error("Distributor already registered")]
    DistributorAlreadyRegistered {},

    #[error("Distributor not found")]
    DistributorNotFound {},
}
