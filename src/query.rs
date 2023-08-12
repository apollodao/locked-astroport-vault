use cosmwasm_std::{Deps, StdError, StdResult};
use cw_vault_standard::{
    extensions::lockup::UnlockingPosition, VaultInfoResponse, VaultStandardInfoResponse,
};
use strum::VariantNames;

use crate::{
    helpers::mint_vault_tokens,
    msg::ExtensionExecuteMsg,
    state::{self, CONFIG},
};

pub fn query_vault_standard_info(_deps: Deps) -> StdResult<VaultStandardInfoResponse> {
    Ok(VaultStandardInfoResponse {
        version: 0,
        extensions: ExtensionExecuteMsg::VARIANTS
            .iter()
            .map(|s| s.to_string())
            .collect(),
    })
}

pub fn query_vault_info(deps: Deps) -> StdResult<VaultInfoResponse> {
    let cfg = CONFIG.load(deps.storage)?;

    Ok(VaultInfoResponse {
        base_token: cfg.base_token.to_string(),
        vault_token: cfg.vault_token_denom,
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

    Ok(claims
        .into_iter()
        .map(|(id, claim)| UnlockingPosition {
            id,
            owner: claim.owner,
            release_at: claim.release_at,
            base_token_amount: claim.base_token_amount,
        })
        .collect())
}

pub fn query_unlocking_position(deps: Deps, id: u64) -> StdResult<UnlockingPosition> {
    let claim = state::claims().query_claim_by_id(deps, id)?;

    Ok(UnlockingPosition {
        id,
        owner: claim.owner,
        release_at: claim.release_at,
        base_token_amount: claim.base_token_amount,
    })
}
