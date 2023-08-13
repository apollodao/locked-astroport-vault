use cosmwasm_std::{coin, to_binary, Addr, Api, CosmosMsg, Decimal, Deps, DepsMut, Env, Uint128};
use cw_utils::Duration;
use osmosis_std::types::osmosis::tokenfactory::v1beta1::{MsgBurn, MsgMint};

use crate::error::ContractResult;
use crate::state::{CONFIG, STATE};
use cosmwasm_schema::serde::Serialize;

use cosmwasm_std::{Coin, MessageInfo, StdError, StdResult};

/// Return the `Coin` from `info.funds` if it is the only denom in the funds.
/// Otherwise, return an error.
pub fn one_coin(info: &MessageInfo, denom: &str) -> StdResult<Coin> {
    if info.funds.len() != 1 && info.funds[0].denom != denom {
        Err(StdError::generic_err("Must deposit exactly one token"))
    } else {
        Ok(info.funds[0].clone())
    }
}

/// Return the `Coin` from `info.funds` if it is the only denom in the funds
/// and the amount is exactly `amount`. Otherwise, return an error.
pub fn correct_funds(info: &MessageInfo, denom: &str, amount: Uint128) -> StdResult<Coin> {
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

/// Converts an `Option<String>` to an `Addr` by unwrapping the string and verifying the address,
/// or using the sender address if the supplied Option is `None`.
pub fn unwrap_recipient(
    recipient: Option<String>,
    info: &MessageInfo,
    api: &dyn Api,
) -> StdResult<Addr> {
    recipient.map_or(Ok(info.sender.clone()), |x| api.addr_validate(&x))
}

/// Returns the number of vault tokens that will be minted for `base_token_amount`
/// base tokens.
pub(crate) fn convert_to_shares(deps: Deps, base_token_amount: Uint128) -> Uint128 {
    let state = STATE.load(deps.storage).unwrap();
    Decimal::from_ratio(base_token_amount, state.staked_base_tokens) * state.vault_token_supply
}

/// Returns the number of base tokens that will be released for `vault_token_amount`
/// vault tokens.
pub(crate) fn convert_to_assets(deps: Deps, vault_token_amount: Uint128) -> Uint128 {
    let state = STATE.load(deps.storage).unwrap();
    Decimal::from_ratio(vault_token_amount, state.vault_token_supply) * state.staked_base_tokens
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

    let mint_amount = convert_to_shares(deps.as_ref(), deposit_amount);

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
    env: &Env,
    burn_amount: Uint128,
) -> ContractResult<(CosmosMsg, Uint128)> {
    let mut state = STATE.load(deps.storage)?;
    let cfg = CONFIG.load(deps.storage)?;

    let release_amount = convert_to_assets(deps.as_ref(), burn_amount);

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

/// A trait to convert a type into a `CosmosMsg` Execute variant that calls the contract itself.
pub trait IntoInternalCall {
    fn into_internal_call(&self, env: &Env) -> StdResult<CosmosMsg>;
}

/// Implement the trait for any type that implements `Serialize`.
impl<T> IntoInternalCall for T
where
    T: Serialize,
{
    fn into_internal_call(&self, env: &Env) -> StdResult<CosmosMsg> {
        Ok(CosmosMsg::Wasm(cosmwasm_std::WasmMsg::Execute {
            contract_addr: env.contract.address.to_string(),
            msg: to_binary(self)?,
            funds: vec![],
        }))
    }
}

/// A trait to check if a type is zero.
pub trait IsZero {
    fn is_zero(&self) -> bool;
}

/// Implement the trait `IsZero` for `Duration`.
impl IsZero for Duration {
    fn is_zero(&self) -> bool {
        match self {
            Duration::Time(t) => t == &0,
            Duration::Height(h) => h == &0,
        }
    }
}