use cosmwasm_std::StdError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("Unauthorized")]
    Unauthorized {},

    #[error("Cannot spend more than spend_limit")]
    SpendLimitReached {},

    #[error("Distributor already registered")]
    DistributorAlreadyRegistered {},

    #[error("Distributor not found")]
    DistributorNotFound {},
}
