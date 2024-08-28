use cosmwasm_std::{Addr, Decimal, Deps, Order, StdError, StdResult};
use cw_storage_plus::Bound;
use cw_vault_standard::extensions::lockup::UnlockingPosition;
use cw_vault_standard::{VaultInfoResponse, VaultStandardInfoResponse};
use strum::VariantNames;

use crate::helpers::INITIAL_VAULT_TOKENS_PER_BASE_TOKEN;
use crate::msg::ExtensionExecuteMsg;
use crate::state::{
    self, StateResponse, BASE_TOKEN, FORCE_WITHDRAW_WHITELIST, POOL, STAKING, STATE,
    VAULT_TOKEN_DENOM,
};

/// The default limit for pagination
pub const DEFAULT_LIMIT: u32 = 10;

pub fn query_vault_standard_info(_deps: Deps) -> StdResult<VaultStandardInfoResponse> {
    Ok(VaultStandardInfoResponse {
        version: "0.4.1-rc.1".to_string(),
        extensions: ExtensionExecuteMsg::VARIANTS
            .iter()
            .map(|s| s.to_string())
            .collect(),
    })
}

pub fn query_vault_info(deps: Deps) -> StdResult<VaultInfoResponse> {
    let base_token = BASE_TOKEN.load(deps.storage)?;
    let vault_token_denom = VAULT_TOKEN_DENOM.load(deps.storage)?;

    Ok(VaultInfoResponse {
        base_token: base_token.to_string(),
        vault_token: vault_token_denom,
    })
}

pub fn query_unlocking_positions(
    deps: Deps,
    owner: String,
    start_after: Option<u64>,
    limit: Option<u32>,
) -> StdResult<Vec<UnlockingPosition>> {
    let claims = state::claims().query_claims_for_owner(
        deps,
        &deps.api.addr_validate(&owner)?,
        start_after,
        limit,
    )?;

    Ok(claims.into_iter().map(|(_id, claim)| claim).collect())
}

pub fn query_unlocking_position(deps: Deps, id: u64) -> StdResult<UnlockingPosition> {
    let claim = state::claims().query_claim_by_id(deps, id)?;
    Ok(claim)
}

pub fn query_force_withdraw_whitelist(
    deps: Deps,
    start_after: Option<String>,
    limit: Option<u32>,
) -> StdResult<Vec<Addr>> {
    let limit = limit.unwrap_or(DEFAULT_LIMIT) as usize;
    let start_after = start_after
        .map(|s| deps.api.addr_validate(&s))
        .transpose()?;
    let min = start_after.as_ref().map(Bound::exclusive);

    let whitelist = FORCE_WITHDRAW_WHITELIST
        .items(deps.storage, min, None, Order::Ascending)
        .take(limit)
        .collect::<StdResult<_>>()?;

    Ok(whitelist)
}

pub fn query_state(deps: Deps) -> StdResult<StateResponse> {
    let state = STATE.load(deps.storage)?;
    Ok(StateResponse {
        base_token: BASE_TOKEN.load(deps.storage)?,
        vault_token_denom: VAULT_TOKEN_DENOM.load(deps.storage)?,
        pool: POOL.load(deps.storage)?,
        staked_base_tokens: state.staked_base_tokens,
        vault_token_supply: state.vault_token_supply,
        staking: STAKING.load(deps.storage)?,
    })
}

pub fn vault_token_exchange_rate(deps: Deps, quote_denom: String) -> StdResult<Decimal> {
    let state = STATE.load(deps.storage)?;
    let base_token = BASE_TOKEN.load(deps.storage)?;

    if quote_denom == base_token.to_string() {
        if state.vault_token_supply.is_zero() {
            Ok(Decimal::from_ratio(
                1u128,
                INITIAL_VAULT_TOKENS_PER_BASE_TOKEN.u128(),
            ))
        } else {
            Ok(Decimal::from_ratio(
                state.staked_base_tokens,
                state.vault_token_supply,
            ))
        }
    } else {
        Err(StdError::generic_err("Locked Astroport Vault only supports vault token exchange rate quoted in the base token"))
    }
}
