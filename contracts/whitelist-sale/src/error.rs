use cosmwasm_std::{OverflowError, StdError};
use thiserror::Error;

use services::ownership_proposal::OwnershipProposalError;

/// ## Description
/// This enum describes whitelist sale contract errors
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

    #[error("Invalid receive hook msg")]
    InvalidHookData {},

    #[error("Accounts for sale is not registered")]
    AccountsIsNotRegistered {},

    #[error("Sale was already registered")]
    SaleWasAlreadyRegistered {},

    #[error("Invalid sale period")]
    InvalidSalePeriod {},

    #[error("Received amount must be higher or equal then required amount for sale")]
    ReceivedAmountMustBeHigherThenRequiredAmountForSale {},

    #[error("Invalid funds input")]
    InvalidFundsInput {},

    #[error("Address is not whitelisted")]
    AddressIsNotWhitelisted {},

    #[error("Sale is not live")]
    SaleIsNotLive {},

    #[error("Purchase amount is too high")]
    PurchaseAmountIsTooHigh {},

    #[error("Sale is not finished yet")]
    SaleIsNotFinishedYet {},
}
