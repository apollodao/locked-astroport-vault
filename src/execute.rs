use apollo_utils::responses::merge_responses;
use cosmwasm_std::{
    to_binary, CosmosMsg, DepsMut, Env, MessageInfo, Response, StdError, Uint128, WasmMsg,
};
use cw20::Cw20ExecuteMsg;

use crate::error::{ContractError, ContractResponse};
use crate::helpers::{self, burn_vault_tokens, mint_vault_tokens};
use crate::state::{CLAIMS, CONFIG, STAKING};

use cw_dex::traits::{Rewards, Stake, Unstake};

pub fn execute_deposit(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    amount: Uint128,
) -> ContractResponse {
    let cfg = CONFIG.load(deps.storage)?;

    let transfer_from_res = Response::new().add_message(WasmMsg::Execute {
        contract_addr: cfg.base_token.to_string(),
        msg: to_binary(&Cw20ExecuteMsg::TransferFrom {
            owner: info.sender.to_string(),
            recipient: env.contract.address.to_string(),
            amount,
        })?,
        funds: vec![],
    });

    let staking = STAKING.load(deps.storage)?;

    let staking_res = staking.stake(deps.as_ref(), &env, amount)?;

    let mint_res = Response::new().add_message(mint_vault_tokens(deps, env, amount)?);

    Ok(merge_responses(vec![
        transfer_from_res,
        staking_res,
        mint_res,
    ]))
}

pub fn execute_redeem(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    amount: Uint128,
    recipient: Option<String>,
    emergency: bool,
) -> ContractResponse {
    let cfg = CONFIG.load(deps.storage)?;
    let unlock_amount = helpers::correct_funds(info, cfg.vault_token_denom, amount)?;
    let recipient = deps
        .api
        .addr_validate(&recipient.unwrap_or(info.sender.to_string()))?;

    let (burn_msg, release_amount) = burn_vault_tokens(deps, env, unlock_amount.amount)?;

    CLAIMS.create_claim(
        deps.storage,
        &recipient,
        release_amount,
        cfg.lock_duration.after(&env.block),
    );

    let res = if emergency {
        Response::new()
    } else {
        STAKING
            .load(deps.storage)?
            .claim_rewards(deps.as_ref(), &env)?
    };

    Ok(res.add_message(burn_msg))
}

pub fn execute_withdraw_unlocked(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    recipient: Option<String>,
) -> ContractResponse {
    let cfg = CONFIG.load(deps.storage)?;
    let claim_amount = CLAIMS.claim_tokens(deps.storage, &info.sender, &env.block, None)?;

    let staking = STAKING.load(deps.storage)?;
    let res = staking.unstake(deps.as_ref(), &env, claim_amount)?;

    let send_msg: CosmosMsg = WasmMsg::Execute {
        contract_addr: cfg.base_token.to_string(),
        msg: to_binary(&Cw20ExecuteMsg::Transfer {
            recipient: recipient.unwrap_or(info.sender.to_string()),
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
