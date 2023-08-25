use cosmwasm_std::{coin, coins, Addr, Api, CosmosMsg, Deps, DepsMut, Env, Uint128};
use cw_utils::Duration;
use osmosis_std::types::osmosis::tokenfactory::v1beta1::{MsgBurn, MsgMint};

use crate::error::{ContractError, ContractResult};
use crate::state::STATE;

use cosmwasm_std::{Coin, MessageInfo, StdResult};

pub const INITIAL_VAULT_TOKENS_PER_BASE_TOKEN: Uint128 = Uint128::new(1_000_000);

/// Asserts that exactly `amount` of `denom` is sent to the contract, with no
/// extra funds.
pub fn assert_correct_funds(
    info: &MessageInfo,
    denom: &str,
    amount: Uint128,
) -> ContractResult<()> {
    if info.funds.len() != 1 || info.funds[0].denom != denom || info.funds[0].amount != amount {
        return Err(ContractError::UnexpectedFunds {
            expected: coins(amount.u128(), denom),
            actual: info.funds.clone(),
        });
    }
    Ok(())
}

/// Converts an `Option<String>` to an `Addr` by unwrapping the string and
/// verifying the address, or using the sender address if the supplied Option is
/// `None`.
pub fn unwrap_recipient(
    recipient: Option<String>,
    info: &MessageInfo,
    api: &dyn Api,
) -> StdResult<Addr> {
    recipient.map_or(Ok(info.sender.clone()), |x| api.addr_validate(&x))
}

/// Returns the number of vault tokens that will be minted for
/// `base_token_amount` base tokens.
pub(crate) fn convert_to_shares(deps: Deps, base_token_amount: Uint128) -> Uint128 {
    let state = STATE.load(deps.storage).unwrap();
    if state.staked_base_tokens.is_zero() {
        return base_token_amount * INITIAL_VAULT_TOKENS_PER_BASE_TOKEN;
    }
    state
        .vault_token_supply
        .multiply_ratio(base_token_amount, state.staked_base_tokens)
}

/// Returns the number of base tokens that will be released for
/// `vault_token_amount` vault tokens.
pub(crate) fn convert_to_assets(deps: Deps, vault_token_amount: Uint128) -> Uint128 {
    let state = STATE.load(deps.storage).unwrap();
    if state.vault_token_supply.is_zero() {
        return vault_token_amount / INITIAL_VAULT_TOKENS_PER_BASE_TOKEN;
    }
    state
        .staked_base_tokens
        .multiply_ratio(vault_token_amount, state.vault_token_supply)
}

/// Return a token factory mint message to mint `amount` of vault tokens to
/// `env.contract.address`.
pub(crate) fn mint_vault_tokens(
    deps: DepsMut,
    env: Env,
    deposit_amount: Uint128,
    vault_token_denom: &str,
) -> ContractResult<(CosmosMsg, Uint128)> {
    let mut state = STATE.load(deps.storage)?;

    let mint_amount = convert_to_shares(deps.as_ref(), deposit_amount);

    state.staked_base_tokens = state.staked_base_tokens.checked_add(deposit_amount)?;
    state.vault_token_supply = state.vault_token_supply.checked_add(mint_amount)?;

    STATE.save(deps.storage, &state)?;

    Ok((
        MsgMint {
            sender: env.contract.address.to_string(),
            amount: Some(coin(mint_amount.u128(), vault_token_denom).into()),
        }
        .into(),
        mint_amount,
    ))
}

/// Return a token factory burn message to burn `amount` of vault tokens from\
/// `env.contract.address` in a tuple together with the amount of base tokens
/// that should be released.
pub(crate) fn burn_vault_tokens(
    deps: DepsMut,
    env: &Env,
    burn_amount: Uint128,
    vault_token_denom: &str,
) -> ContractResult<(CosmosMsg, Uint128)> {
    let mut state = STATE.load(deps.storage)?;

    let release_amount = convert_to_assets(deps.as_ref(), burn_amount);

    state.staked_base_tokens = state.staked_base_tokens.checked_sub(release_amount)?;
    state.vault_token_supply = state.vault_token_supply.checked_sub(burn_amount)?;

    STATE.save(deps.storage, &state)?;

    Ok((
        MsgBurn {
            sender: env.contract.address.to_string(),
            amount: Some(coin(burn_amount.u128(), vault_token_denom).into()),
        }
        .into(),
        release_amount,
    ))
}

/// A trait to convert a type into a `CosmosMsg` Execute variant that calls the
/// contract itself.
pub trait IntoInternalCall {
    fn into_internal_call(self, env: &Env, funds: Vec<Coin>) -> StdResult<CosmosMsg>;
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
