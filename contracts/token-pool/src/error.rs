use cosmwasm_std::StdError;
use thiserror::Error;

/// ## Description
/// This enum describes token pool contract errors
#[derive(Error, Debug)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("Unauthorized")]
    Unauthorized {},
}