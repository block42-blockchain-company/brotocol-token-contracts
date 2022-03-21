use cosmwasm_std::StdError;
use thiserror::Error;

use services::ownership_proposal::OwnershipProposalError;

/// ## Description
/// This enum describes airdrop contract errors
#[derive(Error, Debug)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("{0}")]
    OwnershipProposal(#[from] OwnershipProposalError),

    #[error("Unauthorized")]
    Unauthorized {},

    #[error("Invalid receive hook msg")]
    InvalidHookData {},

    #[error("Invalid hex encoded merkle root")]
    InvalidHexMerkle {},

    #[error("Invalid hex encoded proof")]
    InvalidHexProof {},

    #[error("Merkle verification failed")]
    MerkleVerification {},

    #[error("Already claimed")]
    AlreadyClaimed {},
}
