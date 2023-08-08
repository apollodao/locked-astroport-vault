use cosmwasm_std::{OverflowError, StdError};
use cw_dex::CwDexError;
use cw_ownable::OwnershipError;

#[derive(Debug, thiserror::Error)]
pub enum ContractError {
    #[error(transparent)]
    Std(#[from] StdError),

    #[error(transparent)]
    Ownership(#[from] OwnershipError),

    #[error(transparent)]
    CwDex(#[from] CwDexError),

    #[error(transparent)]
    Overflow(#[from] OverflowError),

    #[error("Unauthorized")]
    Unauthorized {},
}

pub type ContractResult<T> = Result<T, ContractError>;
pub type ContractResponse = ContractResult<cosmwasm_std::Response>;
