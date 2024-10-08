use cosmwasm_std::{Coin, OverflowError, StdError};
use cw_dex_astroport::cw_dex::CwDexError;
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

    #[error(transparent)]
    Semver(#[from] semver::Error),

    #[error("Unauthorized")]
    Unauthorized {},

    #[error("Deposits are disabled")]
    DepositsDisabled {},

    #[error("Unknown reply ID: {id}")]
    UnknownReplyId { id: u64 },

    #[error("Lockup is disabled for this vault")]
    LockupDisabled {},

    #[error("Unexpected funds sent. Expected: {expected:?}, Actual: {actual:?}")]
    UnexpectedFunds {
        expected: Vec<Coin>,
        actual: Vec<Coin>,
    },
}

pub type ContractResult<T> = Result<T, ContractError>;
pub type ContractResponse = ContractResult<cosmwasm_std::Response>;
