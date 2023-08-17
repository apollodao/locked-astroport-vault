use cosmwasm_std::{Deps, StdResult};
use cw_vault_standard::extensions::lockup::UnlockingPosition;
use cw_vault_standard::{VaultInfoResponse, VaultStandardInfoResponse};
use strum::VariantNames;

use crate::msg::ExtensionExecuteMsg;
use crate::state::{self, BASE_TOKEN, VAULT_TOKEN_DENOM};

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
