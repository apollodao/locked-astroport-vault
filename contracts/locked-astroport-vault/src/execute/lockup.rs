use apollo_cw_asset::Asset;
use cosmwasm_std::{DepsMut, Env, MessageInfo, Response, StdError, Uint128};

use crate::error::{ContractError, ContractResponse};
use crate::helpers::unwrap_recipient;
use crate::state::{self, BASE_TOKEN, FORCE_WITHDRAW_WHITELIST, STAKING};

use cw_dex::traits::Unstake;

pub fn execute_withdraw_unlocked(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    recipient: Option<String>,
    lockup_id: u64,
) -> ContractResponse {
    let base_token = BASE_TOKEN.load(deps.storage)?;
    let recipient = unwrap_recipient(recipient, &info, deps.api)?;

    // Calculate amount of LP tokens available to claim
    let claim_amount = state::claims().claim_tokens(deps.storage, &env.block, &info, lockup_id)?;

    // Unstake LP tokens
    let staking = STAKING.load(deps.storage)?;
    let res = staking.unstake(deps.as_ref(), &env, claim_amount)?;

    // Send LP tokens to recipient
    let send_msg = Asset::cw20(base_token, claim_amount).transfer_msg(recipient)?;

    Ok(res.add_message(send_msg))
}

pub fn execute_update_force_withdraw_whitelist(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    add_addresses: Vec<String>,
    remove_addresses: Vec<String>,
) -> ContractResponse {
    if !cw_ownable::is_owner(deps.storage, &info.sender)? {
        return Err(ContractError::Unauthorized {});
    }

    for addr in add_addresses {
        FORCE_WITHDRAW_WHITELIST.insert(deps.storage, &deps.api.addr_validate(&addr)?)?;
    }

    for addr in remove_addresses {
        let addr = deps.api.addr_validate(&addr)?;
        let was_removed = FORCE_WITHDRAW_WHITELIST.remove(deps.storage, &addr)?;
        if !was_removed {
            return Err(ContractError::Std(StdError::generic_err(
                "Address not in whitelist",
            )));
        }
    }

    Ok(Response::new())
}

pub fn execute_force_withdraw_unlocking(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    amount: Option<Uint128>,
    recipient: Option<String>,
    lockup_id: u64,
) -> ContractResponse {
    let base_token = BASE_TOKEN.load(deps.storage)?;
    let recipient = unwrap_recipient(recipient, &info, deps.api)?;

    // Check that the sender is whitelisted
    if !FORCE_WITHDRAW_WHITELIST.contains(deps.storage, &info.sender) {
        return Err(ContractError::Unauthorized {});
    }

    // Get the claimed amount and update the claim in storage, deleting it if
    // all of the tokens are claimed, or updating it with the remaining amount.
    let claimed_amount = state::claims().force_claim(deps.storage, &info, lockup_id, amount)?;

    // Unstake LP tokens
    let staking = STAKING.load(deps.storage)?;
    let res = staking.unstake(deps.as_ref(), &env, claimed_amount)?;

    // Send LP tokens to recipient
    let send_msg = Asset::cw20(base_token, claimed_amount).transfer_msg(recipient)?;

    Ok(res.add_message(send_msg))
}
