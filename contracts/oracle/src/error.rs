use cosmwasm_std::StdError;
use thiserror::Error;

/// ## Description
/// This enum describes oracle contract errors
#[derive(Error, Debug)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("Unauthorized")]
    Unauthorized {},

    #[error("Price cannot be updated because update interval has not passed")]
    UpdatePriceIntervalError {},
}
