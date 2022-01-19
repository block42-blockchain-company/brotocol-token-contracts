use cosmwasm_std::{OverflowError, StdError};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("{0}")]
    OverflowError(#[from] OverflowError),

    #[error("Unauthorized")]
    Unauthorized {},

    #[error("Invalid receive hook msg")]
    InvalidHookData {},

    #[error("Cannot unbond more than bond amount")]
    ForbiddenToUnbondMoreThanBonded {},

    #[error("Nothing to claim, wait for unbonding period")]
    NothingToClaim {},
}
