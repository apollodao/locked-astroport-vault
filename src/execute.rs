use apollo_cw_asset::{Asset, AssetList};
use apollo_utils::responses::merge_responses;
use cosmwasm_std::{
    coin, coins, to_binary, BankMsg, Coins, CosmosMsg, DepsMut, Empty, Env, MessageInfo, Response,
    StdError, Uint128, WasmMsg,
};
use cw20::Cw20ExecuteMsg;

use crate::error::{ContractError, ContractResponse};
use crate::helpers::{self, burn_vault_tokens, mint_vault_tokens, unwrap_recipient};
use crate::state::{CLAIMS, CONFIG, POOL, STAKING};

use cw_dex::traits::{Rewards, Stake, Unstake};

pub fn execute_compound(deps: DepsMut, env: Env, info: MessageInfo) -> ContractResponse {
    let cfg = CONFIG.load(deps.storage)?;
    let staking = STAKING.load(deps.storage)?;

    // Claim any pending rewards
    let claim_rewards_res = staking.claim_rewards(deps.as_ref(), &env)?;

    // Sell rewards
    let performance_fee = cfg.performance_fee;
    let pool = POOL.load(deps.storage)?;
    let reward_tokens: AssetList = cfg.reward_tokens.into();

    // Query balances of reward tokens of the contract
    let mut reward_token_balances =
        reward_tokens.query_balances(&deps.querier, &env.contract.address)?;

    // Calculate performance fees and transfer them to treasury
    let mut performance_fees = reward_token_balances.clone();
    performance_fees.apply(|x| x.amount = x.amount * performance_fee);
    let mut msgs = performance_fees.transfer_msgs(cfg.treasury)?;

    // Deduct performance fees from reward token balances
    let tokens_to_sell = reward_token_balances.deduct_many(&performance_fees)?;

    // Sell reward tokens for base tokens
    if tokens_to_sell.len() > 0 {
        msgs.append(&mut cfg.router.basket_liquidate_msgs(
            tokens_to_sell,
            &cfg.reward_liquidation_target,
            None,
            None,
        )?);
    }

    // Provide Liquidity

    // Stake LP tokens

    // let res = staking.compound(deps.as_ref(), &env)?;

    Ok(Response::default().add_messages(msgs))
}

pub fn execute_deposit(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    amount: Uint128,
    recipient: Option<String>,
) -> ContractResponse {
    let cfg = CONFIG.load(deps.storage)?;
    let recipient = helpers::unwrap_recipient(recipient, &info, deps.api)?;

    // Check that deposits are enabled
    if !cfg.deposits_enabled {
        return Err(ContractError::DepositsDisabled {});
    }

    // Transfer LP tokens from sender
    let transfer_from_res = Response::new().add_message(WasmMsg::Execute {
        contract_addr: cfg.base_token.to_string(),
        msg: to_binary(&Cw20ExecuteMsg::TransferFrom {
            owner: info.sender.to_string(),
            recipient: env.contract.address.to_string(),
            amount,
        })?,
        funds: vec![],
    });

    // Stake deposited LP tokens
    let staking = STAKING.load(deps.storage)?;
    let staking_res = staking.stake(deps.as_ref(), &env, amount)?;

    // Mint vault tokens
    let mint_res = Response::new().add_message(mint_vault_tokens(deps, env, amount)?);

    // Send minted vault tokens to recipient
    let send_msg: CosmosMsg = BankMsg::Send {
        to_address: recipient.to_string(),
        amount: coins(amount.u128(), cfg.vault_token_denom),
    }
    .into();

    Ok(merge_responses(vec![transfer_from_res, staking_res, mint_res]).add_message(send_msg))
}

pub fn execute_redeem(
    mut deps: DepsMut,
    env: Env,
    info: MessageInfo,
    amount: Uint128,
    recipient: Option<String>,
    emergency: bool,
) -> ContractResponse {
    let cfg = CONFIG.load(deps.storage)?;
    let recipient = unwrap_recipient(recipient, &info, deps.api)?;

    // Check that only vault tokens were sent and that the amount is correct
    // TODO: Check for amount == zero?
    let unlock_amount = helpers::correct_funds(&info, &cfg.vault_token_denom, amount)?;

    // Calculate claim amount and create msg to burn vault tokens
    let (burn_msg, release_amount) = burn_vault_tokens(deps.branch(), &env, unlock_amount.amount)?;

    // Create claim for recipient
    CLAIMS.create_claim(
        deps.storage,
        &recipient,
        release_amount,
        cfg.lock_duration.after(&env.block),
    )?;

    // If emergency, only burn vault tokens, otherwise also claim rewards
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
