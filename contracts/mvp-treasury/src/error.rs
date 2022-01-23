use cosmwasm_std::StdError;
use thiserror::Error;

/// ## Description
/// This enum describes mvp treasury contract errors
#[derive(Error, Debug)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("Unauthorized")]
    Unauthorized {},

    #[error("Insufficient funds")]
    InsufficientFunds {},
}
