use cosmwasm_std::{to_binary, CosmosMsg, DepsMut, Env, MessageInfo, Response, StdError, WasmMsg};
use cw20::Cw20ExecuteMsg;

use crate::error::{ContractError, ContractResponse};
use crate::helpers::{unwrap_recipient, IntoInternalCall};
use crate::msg::InternalMsg;
use crate::state::{CLAIMS, CONFIG, STAKING};

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
    let claim_amount = CLAIMS.claim_tokens(deps.storage, &info.sender, &env.block, None)?;

    // Unstake LP tokens
    let staking = STAKING.load(deps.storage)?;
    let res = staking.unstake(deps.as_ref(), &env, claim_amount)?;

    // Send LP tokens to recipient
    let send_msg: CosmosMsg = WasmMsg::Execute {
        contract_addr: cfg.base_token.to_string(),
        msg: to_binary(&Cw20ExecuteMsg::Transfer {
            recipient: recipient.to_string(),
            amount: claim_amount,
        })?,
        funds: vec![],
    }
    .into();

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
