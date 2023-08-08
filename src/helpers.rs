use cosmwasm_std::{coin, CosmosMsg, Decimal, DepsMut, Env, Uint128};
use cw_dex::traits::Rewards;
use osmosis_std::types::osmosis::tokenfactory::v1beta1::{MsgBurn, MsgMint};

use crate::error::ContractResult;
use crate::state::{CONFIG, STAKING, STATE};

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

/// Return the `denom` from `info.funds` if it is the only coin in the funds
/// and the amount is exactly `amount`. Otherwise, return an error.
pub fn correct_funds(info: MessageInfo, denom: String, amount: Uint128) -> StdResult<Coin> {
    let coin = one_coin(info, denom)?;
    if coin.amount != amount {
        Err(StdError::generic_err(format!(
            "Invalid amount {} expected {}",
            coin.amount, amount
        )))
    } else {
        Ok(coin)
    }
}

/// Return a token factory mint message to mint `amount` of vault tokens to
/// `env.contract.address`.
pub(crate) fn mint_vault_tokens(
    deps: DepsMut,
    env: Env,
    deposit_amount: Uint128,
) -> ContractResult<CosmosMsg> {
    let mut state = STATE.load(deps.storage)?;
    let cfg = CONFIG.load(deps.storage)?;

    let mint_amount =
        Decimal::from_ratio(deposit_amount, state.staked_base_tokens) * state.vault_token_supply;

    state.staked_base_tokens = state.staked_base_tokens.checked_add(deposit_amount)?;
    state.vault_token_supply = state.vault_token_supply.checked_add(mint_amount)?;

    STATE.save(deps.storage, &state)?;

    Ok(MsgMint {
        sender: env.contract.address.to_string(),
        amount: Some(coin(mint_amount.u128(), &cfg.vault_token_denom).into()),
    }
    .into())
}

/// Return a token factory burn message to burn `amount` of vault tokens from\
/// `env.contract.address` in a tuple together with the amount of base tokens
/// that should be released.
pub(crate) fn burn_vault_tokens(
    deps: DepsMut,
    env: Env,
    burn_amount: Uint128,
) -> ContractResult<(CosmosMsg, Uint128)> {
    let mut state = STATE.load(deps.storage)?;
    let cfg = CONFIG.load(deps.storage)?;

    let release_amount =
        Decimal::from_ratio(burn_amount, state.vault_token_supply) * state.staked_base_tokens;

    state.staked_base_tokens = state.staked_base_tokens.checked_sub(release_amount)?;
    state.vault_token_supply = state.vault_token_supply.checked_sub(burn_amount)?;

    STATE.save(deps.storage, &state)?;

    Ok((
        MsgBurn {
            sender: env.contract.address.to_string(),
            amount: Some(coin(burn_amount.u128(), &cfg.vault_token_denom).into()),
        }
        .into(),
        release_amount,
    ))
}
