use apollo_cw_asset::Asset;
use cosmwasm_std::{DepsMut, Env, MessageInfo, Response, StdError, Uint128};

use crate::error::{ContractError, ContractResponse};
use crate::helpers::{self, burn_vault_tokens, unwrap_recipient, IntoInternalCall};
use crate::msg::InternalMsg;
use crate::state::{self, CONFIG, STAKING};

use cw_dex::traits::{Rewards, Unstake};

pub fn execute_compound(deps: DepsMut, env: Env) -> ContractResponse {
    let staking = STAKING.load(deps.storage)?;

    // Claim any pending rewards
    let claim_rewards_res = staking.claim_rewards(deps.as_ref(), &env)?;

    // Sell rewards
    let sell_msg = InternalMsg::SellTokens {}.into_internal_call(&env)?;

    // Provide Liquidity
    let provide_msg = InternalMsg::ProvideLiquidity {}.into_internal_call(&env)?;

    // Stake LP tokens
    let stake_msg = InternalMsg::StakeLps {}.into_internal_call(&env)?;

    Ok(claim_rewards_res
        .add_message(sell_msg)
        .add_message(provide_msg)
        .add_message(stake_msg))
}

pub fn execute_withdraw_unlocked(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    recipient: Option<String>,
    lockup_id: u64, // TODO: use lockup_id
) -> ContractResponse {
    let cfg = CONFIG.load(deps.storage)?;
    let recipient = unwrap_recipient(recipient, &info, deps.api)?;

    // Calculate amount of LP tokens available to claim
    let claim_amount = state::claims().claim_tokens(deps.storage, &env.block, &info, lockup_id)?;

    // Unstake LP tokens
    let staking = STAKING.load(deps.storage)?;
    let res = staking.unstake(deps.as_ref(), &env, claim_amount)?;

    // Send LP tokens to recipient
    let send_msg = Asset::cw20(cfg.base_token, claim_amount).transfer_msg(&recipient)?;

    Ok(res.add_message(send_msg))
}

pub fn execute_update_whitelist(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    add_addresses: Vec<String>,
    remove_addresses: Vec<String>,
) -> ContractResponse {
    if !cw_ownable::is_owner(deps.storage, &info.sender)? {
        return Err(ContractError::Unauthorized {});
    }

    let cfg = CONFIG.load(deps.storage)?;
    let mut whitelist = cfg.force_withdraw_whitelist;

    for addr in add_addresses {
        whitelist.insert(deps.api.addr_validate(&addr)?);
    }

    for addr in remove_addresses {
        let addr = deps.api.addr_validate(&addr)?;
        if !whitelist.contains(&addr) {
            return Err(ContractError::Std(StdError::generic_err(
                "Address not in whitelist",
            )));
        }
        whitelist.remove(&addr);
    }

    Ok(Response::new())
}

pub fn execute_force_redeem(
    mut deps: DepsMut,
    env: Env,
    info: MessageInfo,
    amount: Uint128,
    recipient: Option<String>,
) -> ContractResponse {
    let cfg = CONFIG.load(deps.storage)?;
    let recipient = unwrap_recipient(recipient, &info, deps.api)?;

    // Check that the sender is whitelisted
    if !cfg.force_withdraw_whitelist.contains(&info.sender) {
        return Err(ContractError::Unauthorized {});
    }

    // Check that only vault tokens were sent and that the amount is correct
    // TODO: Check for amount == zero?
    let unlock_amount = helpers::correct_funds(&info, &cfg.vault_token_denom, amount)?;

    // Calculate claim amount and create msg to burn vault tokens
    let (burn_msg, release_amount) = burn_vault_tokens(deps.branch(), &env, unlock_amount.amount)?;

    // Unstake LP tokens
    let staking = STAKING.load(deps.storage)?;
    let staking_res = staking.unstake(deps.as_ref(), &env, release_amount)?;

    // Send LP tokens to recipient
    let send_msg = Asset::cw20(cfg.base_token, release_amount).transfer_msg(&recipient)?;

    Ok(staking_res.add_message(burn_msg).add_message(send_msg))
}

pub fn execute_force_withdraw_unlocking(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    amount: Option<Uint128>,
    recipient: Option<String>,
    lockup_id: u64,
) -> ContractResponse {
    let cfg = CONFIG.load(deps.storage)?;
    let recipient = unwrap_recipient(recipient, &info, deps.api)?;

    // Check that the sender is whitelisted
    if !cfg.force_withdraw_whitelist.contains(&info.sender) {
        return Err(ContractError::Unauthorized {});
    }

    // Get the claimed amount and update the claim in storage, deleting it if
    // all of the tokens are claimed, or updating it with the remaining amount.
    let claimed_amount = state::claims().force_claim(deps.storage, &info, lockup_id, amount)?;

    // Unstake LP tokens
    let staking = STAKING.load(deps.storage)?;
    let res = staking.unstake(deps.as_ref(), &env, claimed_amount)?;

    // Send LP tokens to recipient
    let send_msg = Asset::cw20(cfg.base_token, claimed_amount).transfer_msg(&recipient)?;

    Ok(res.add_message(send_msg))
}
