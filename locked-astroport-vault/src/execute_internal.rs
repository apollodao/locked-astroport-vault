use apollo_cw_asset::{Asset, AssetInfo, AssetList};
use apollo_utils::responses::merge_responses;
use cosmwasm_std::{
    coins, to_binary, Addr, BankMsg, CosmosMsg, DepsMut, Env, MessageInfo, Response, Uint128,
};

use crate::error::{ContractError, ContractResponse};
use crate::helpers::{self, burn_vault_tokens, mint_vault_tokens, IsZero};
use crate::state::{self, BASE_TOKEN, CONFIG, POOL, STAKING, VAULT_TOKEN_DENOM};

use cw_dex::traits::{Stake, Unstake};

pub fn sell_tokens(deps: DepsMut, env: Env) -> ContractResponse {
    let cfg = CONFIG.load(deps.storage)?;

    // Calculate performance fees and tokens to sell
    let mut performance_fees = AssetList::new();
    let mut tokens_to_sell = AssetList::new();
    for asset_info in cfg.reward_tokens.into_iter() {
        let balance = asset_info.query_balance(&deps.querier, &env.contract.address)?;
        let fee = balance * cfg.performance_fee;
        performance_fees.add(&Asset::new(asset_info.clone(), fee))?;
        tokens_to_sell.add(&Asset::new(asset_info, balance - fee))?;
    }

    // Create msgs to transfer performance fees to treasury
    let mut msgs = performance_fees.transfer_msgs(cfg.treasury)?;

    // Add msg to sell reward tokens
    if tokens_to_sell.len() > 0 {
        msgs.append(&mut cfg.router.basket_liquidate_msgs(
            tokens_to_sell,
            &cfg.reward_liquidation_target,
            None,
            None,
        )?);
    }

    Ok(Response::default().add_messages(msgs))
}

pub fn provide_liquidity(deps: DepsMut, env: Env) -> ContractResponse {
    let cfg = CONFIG.load(deps.storage)?;

    let pool = POOL.load(deps.storage)?;
    let pool_asset_balances = AssetList::query_asset_info_balances(
        pool.pool_assets.clone(),
        &deps.querier,
        &env.contract.address,
    )?;

    let provide_liquidity_msgs = cfg.liquidity_helper.balancing_provide_liquidity(
        pool_asset_balances,
        Uint128::zero(), // TODO: Set slippage?
        to_binary(&pool)?,
        None,
    )?;

    Ok(Response::new().add_messages(provide_liquidity_msgs))
}

pub fn stake_lps(deps: DepsMut, env: Env) -> ContractResponse {
    let base_token = BASE_TOKEN.load(deps.storage)?;
    let staking = STAKING.load(deps.storage)?;

    // Query LP token balance
    let lp_token_balance =
        AssetInfo::cw20(base_token).query_balance(&deps.querier, &env.contract.address)?;

    // Stake LP tokens
    let staking_res = staking.stake(deps.as_ref(), &env, lp_token_balance)?;

    Ok(staking_res)
}

pub fn deposit(
    deps: DepsMut,
    env: Env,
    amount: Uint128,
    depositor: Addr,
    recipient: Addr,
) -> ContractResponse {
    let cfg = CONFIG.load(deps.storage)?;
    let base_token = BASE_TOKEN.load(deps.storage)?;
    let vault_token_denom = VAULT_TOKEN_DENOM.load(deps.storage)?;

    // Check that deposits are enabled
    if !cfg.deposits_enabled {
        return Err(ContractError::DepositsDisabled {});
    }

    // Transfer LP tokens from sender
    let transfer_from_res = Response::new().add_message(
        Asset::cw20(base_token, amount).transfer_from_msg(depositor, &env.contract.address)?,
    );

    // Stake deposited LP tokens
    let staking = STAKING.load(deps.storage)?;
    let staking_res = staking.stake(deps.as_ref(), &env, amount)?;

    // Mint vault tokens
    let mint_msg = mint_vault_tokens(deps, env, amount, &vault_token_denom)?;

    // Send minted vault tokens to recipient
    let send_msg: CosmosMsg = BankMsg::Send {
        to_address: recipient.to_string(),
        amount: coins(amount.u128(), vault_token_denom),
    }
    .into();

    Ok(merge_responses(vec![transfer_from_res, staking_res])
        .add_message(mint_msg)
        .add_message(send_msg))
}

pub fn redeem(
    mut deps: DepsMut,
    env: Env,
    info: MessageInfo,
    amount: Uint128,
    recipient: Addr,
) -> ContractResponse {
    let cfg = CONFIG.load(deps.storage)?;
    let base_token = BASE_TOKEN.load(deps.storage)?;
    let vt_denom = VAULT_TOKEN_DENOM.load(deps.storage)?;

    // Check that only vault tokens were sent and that the amount is correct
    let unlock_amount = helpers::correct_funds(&info, &vt_denom, amount)?;

    // Calculate claim amount and create msg to burn vault tokens
    let (burn_msg, claim_amount) =
        burn_vault_tokens(deps.branch(), &env, unlock_amount.amount, &vt_denom)?;

    // If lock duration is zero, unstake LP tokens and send them to recipient,
    // else create a claim for recipient so they can call `WithdrawUnlocked` later.
    let res = if cfg.lock_duration.is_zero() {
        // Unstake LP tokens
        let staking = STAKING.load(deps.storage)?;
        let res = staking.unstake(deps.as_ref(), &env, claim_amount)?;

        // Send LP tokens to recipient
        let send_msg = Asset::cw20(base_token, claim_amount).transfer_msg(&recipient)?;

        res.add_message(send_msg)
    } else {
        // Create claim for recipient
        state::claims().create_claim(
            deps.storage,
            &recipient,
            claim_amount,
            cfg.lock_duration.after(&env.block),
        )?;
        Response::new()
    };

    Ok(res.add_message(burn_msg))
}
