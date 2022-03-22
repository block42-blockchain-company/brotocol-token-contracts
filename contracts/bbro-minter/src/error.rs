use cosmwasm_std::StdError;
use thiserror::Error;

use services::ownership_proposal::OwnershipProposalError;

/// ## Description
/// This enum describes bbro-minter contract errors
#[derive(Error, Debug)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("{0}")]
    OwnershipProposal(#[from] OwnershipProposalError),

    #[error("Unauthorized")]
    Unauthorized {},

    #[error("Initial balances for token creation must be empty")]
    InitialBalancesMustBeEmpty {},

    #[error("Initial minter info for token creation must be empty")]
    InitialMinterInfoMustBeEmpty {},

    #[error("bBRO token contract address is not set")]
    BbroContractAddressIsNotSet {},

    #[error("Minter already registered")]
    MinterAlreadyRegistered {},

    #[error("Minter not found")]
    MinterNotFound {},
}
