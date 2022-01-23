use cosmwasm_std::StdError;
use thiserror::Error;

/// ## Description
/// This enum describes staking contract errors
#[derive(Error, Debug)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("Unauthorized")]
    Unauthorized {},

    #[error("Invalid hex encoded merkle root")]
    InvalidHexMerkle {},

    #[error("Invalid hex encoded proof")]
    InvalidHexProof {},

    #[error("Merkle verification failed")]
    MerkleVerification {},

    #[error("Already claimed")]
    AlreadyClaimed {},
}
