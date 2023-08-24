use apollo_cw_asset::{Asset, AssetInfo, AssetList};
use cosmwasm_std::{attr, to_binary, DepsMut, Env, Event, Response, StdError, Uint128};

use crate::error::ContractResponse;
use crate::helpers::IntoInternalCall;
use crate::msg::InternalMsg;
use crate::state::{BASE_TOKEN, CONFIG, POOL, STAKING, STATE};

use cw_dex::traits::{Rewards, Stake};

pub fn execute_compound(deps: DepsMut, env: Env) -> ContractResponse {
    let staking = STAKING.load(deps.storage)?;

    // Claim any pending rewards
    let claim_rewards_res = staking.claim_rewards(deps.as_ref(), &env)?;

    // Sell rewards
    let sell_msg = InternalMsg::SellTokens {}.into_internal_call(&env, vec![])?;

    // Provide Liquidity
    let provide_msg = InternalMsg::ProvideLiquidity {}.into_internal_call(&env, vec![])?;

    // Stake LP tokens
    let stake_msg = InternalMsg::StakeLps {}.into_internal_call(&env, vec![])?;

    Ok(claim_rewards_res
        .add_message(sell_msg)
        .add_message(provide_msg)
        .add_message(stake_msg))
}

pub fn execute_sell_tokens(deps: DepsMut, env: Env) -> ContractResponse {
    let cfg = CONFIG.load(deps.storage)?;

    // Calculate performance fees and tokens to sell
    let mut performance_fees = AssetList::new();
    let mut tokens_to_sell = AssetList::new();
    for asset_info in cfg.reward_tokens.into_iter() {
        let balance = asset_info.query_balance(&deps.querier, &env.contract.address)?;
        let fee = balance * cfg.performance_fee;
        performance_fees.add(&Asset::new(asset_info.clone(), fee))?;
        // Don't add to list of tokens to sell if it is the reward liquidation target
        if asset_info != cfg.reward_liquidation_target {
            tokens_to_sell.add(&Asset::new(asset_info, balance - fee))?;
        }
    }

    // Create msgs to transfer performance fees to treasury
    let mut msgs = performance_fees.transfer_msgs(cfg.treasury)?;

    // Create event
    let mut event = Event::new("apollo/vaults/execute_compound")
        .add_attributes(vec![attr("action", "sell_tokens")]);
    for fee in performance_fees.iter() {
        event = event.add_attributes(vec![attr("performance_fee", fee.to_string())]);
    }
    for token in tokens_to_sell.iter() {
        event = event.add_attributes(vec![attr("sold_token", token.to_string())]);
    }

    // Add msg to sell reward tokens
    if tokens_to_sell.len() > 0 {
        msgs.append(&mut cfg.router.basket_liquidate_msgs(
            tokens_to_sell,
            &cfg.reward_liquidation_target,
            None,
            None,
        )?);
    }

    Ok(Response::default().add_messages(msgs).add_event(event))
}

pub fn execute_provide_liquidity(deps: DepsMut, env: Env) -> ContractResponse {
    let cfg = CONFIG.load(deps.storage)?;

    let pool = POOL.load(deps.storage)?;
    let pool_asset_balances = AssetList::query_asset_info_balances(
        pool.pool_assets.clone(),
        &deps.querier,
        &env.contract.address,
    )?;

    // Return with no messages if there are no assets to provide liquidity with
    if pool_asset_balances.len() == 0 {
        return Ok(Response::default());
    }

    let event = Event::new("apollo/vaults/execute_compound")
        .add_attribute("action", "provide_liquidity")
        .add_attribute("pool_asset_balances", pool_asset_balances.to_string());

    let provide_liquidity_msgs = cfg.liquidity_helper.balancing_provide_liquidity(
        pool_asset_balances,
        Uint128::zero(), // TODO: Set slippage?
        to_binary(&pool)?,
        None,
    )?;

    Ok(Response::new()
        .add_messages(provide_liquidity_msgs)
        .add_event(event))
}

pub fn execute_stake_lps(deps: DepsMut, env: Env) -> ContractResponse {
    let base_token = BASE_TOKEN.load(deps.storage)?;
    let staking = STAKING.load(deps.storage)?;

    // Query LP token balance
    let lp_token_balance =
        AssetInfo::cw20(base_token).query_balance(&deps.querier, &env.contract.address)?;

    // Return with no messages if there are no LP tokens to stake
    if lp_token_balance.is_zero() {
        return Ok(Response::default());
    }

    // Stake LP tokens
    let staking_res = staking.stake(deps.as_ref(), &env, lp_token_balance)?;

    // Update total staked amount
    STATE.update(deps.storage, |mut state| {
        state.staked_base_tokens += lp_token_balance;
        Ok::<_, StdError>(state)
    })?;

    let event = Event::new("apollo/vaults/execute_compound")
        .add_attribute("action", "stake_lps")
        .add_attribute("amount_to_stake", lp_token_balance.to_string());

    Ok(staking_res.add_event(event))
}
