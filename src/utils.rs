use cosmwasm_std::{Coin, MessageInfo, StdError, StdResult};

/// Return the `denom` from `info.funds` if it is the only coin in the funds.
/// Otherwise, return an error.
pub fn one_coin(info: MessageInfo, denom: String) -> StdResult<Coin> {
    if info.funds.len() != 1 && info.funds[0].denom != denom {
        Err(StdError::generic_err("Must deposit exactly one token"))
    } else {
        Ok(info.funds[0].clone())
    }
}
