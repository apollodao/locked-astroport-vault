use apollo_cw_asset::{Asset, AssetList};
use cosmwasm_std::{attr, to_json_binary, DepsMut, Env, Event, Response, StdError, Uint128};

use crate::error::ContractResponse;
use crate::helpers::IntoInternalCall;
use crate::msg::InternalMsg;
use crate::state::{BASE_TOKEN, CONFIG, POOL, STAKING, STATE};

use cw_dex::traits::{Rewards, Stake};

pub fn execute_compound(
    deps: DepsMut,
    env: Env,
    discount_deposit: Option<Asset>,
) -> ContractResponse {
    let staking = STAKING.load(deps.storage)?;
    // Claim any pending rewards
    let claim_rewards_res = staking
        .claim_rewards(deps.as_ref(), &env)
        .unwrap_or_default(); // Astroport throws an error on query pending rewards if we have never
                              // staked before, so we just ignore the error here.

    // Sell rewards
    let sell_msg = InternalMsg::SellTokens {}.into_internal_call(&env, vec![])?;

    // Provide Liquidity
    let provide_msg = InternalMsg::ProvideLiquidity {}.into_internal_call(&env, vec![])?;

    // Stake LP tokens
    let stake_msg = InternalMsg::StakeLps { discount_deposit }.into_internal_call(&env, vec![])?;

    Ok(claim_rewards_res
        .add_message(sell_msg)
        .add_message(provide_msg)
        .add_message(stake_msg))
}

pub fn execute_sell_tokens(deps: DepsMut, env: Env) -> ContractResponse {
    let cfg = CONFIG.load(deps.storage)?;

    // Create event
    let mut event = Event::new("apollo/vaults/execute_compound")
        .add_attributes(vec![attr("action", "sell_tokens")]);

    // Calculate performance fees and tokens to sell
    let mut performance_fees = AssetList::new();
    let mut tokens_to_sell = AssetList::new();
    for asset_info in cfg.reward_tokens.into_iter() {
        let balance = asset_info.query_balance(&deps.querier, &env.contract.address)?;
        let fee_amount = balance * cfg.performance_fee.fee_rate;
        let amount_to_sell = balance - fee_amount;

        if fee_amount > Uint128::zero() {
            let fee_asset = Asset::new(asset_info.clone(), fee_amount);
            performance_fees.add(&fee_asset)?;
            event = event.add_attributes(vec![attr("fee", fee_asset.to_string())]);
        }

        if amount_to_sell > Uint128::zero() && asset_info != cfg.reward_liquidation_target {
            let token = Asset::new(asset_info.clone(), amount_to_sell);
            tokens_to_sell.add(&token)?;
            event = event.add_attributes(vec![attr("sold_token", token.to_string())]);
        }
    }

    // Create msgs to transfer performance fees to treasury
    let mut msgs = cfg
        .performance_fee
        .transfer_assets_msgs(&performance_fees, &env)?;

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
        Uint128::zero(),
        to_json_binary(&pool)?,
        None,
    )?;

    Ok(Response::new()
        .add_messages(provide_liquidity_msgs)
        .add_event(event))
}

pub fn execute_stake_lps(
    deps: DepsMut,
    env: Env,
    discount_deposit: Option<Asset>,
) -> ContractResponse {
    let base_token = BASE_TOKEN.load(deps.storage)?;
    let staking = STAKING.load(deps.storage)?;

    // Query LP token balance
    let lp_token_balance = base_token.query_balance(&deps.querier, &env.contract.address)?;

    let stake_amount = if let Some(discount_deposit) = discount_deposit {
        lp_token_balance.saturating_sub(discount_deposit.amount)
    } else {
        lp_token_balance
    };

    // Return with no messages if there are no LP tokens to stake
    if stake_amount.is_zero() {
        return Ok(Response::default());
    }

    // Stake LP tokens
    let staking_res = staking.stake(deps.as_ref(), &env, stake_amount)?;

    // Update total staked amount
    STATE.update(deps.storage, |mut state| {
        state.staked_base_tokens += lp_token_balance;
        Ok::<_, StdError>(state)
    })?;

    let state = STATE.load(deps.storage)?;
    let event = Event::new("apollo/vaults/execute_compound")
        .add_attribute("action", "stake_lps")
        .add_attribute("amount_to_stake", lp_token_balance.to_string())
        .add_attribute("staked_base_tokens_after_action", state.staked_base_tokens)
        .add_attribute("vault_token_supply_after_action", state.vault_token_supply);

    Ok(staking_res.add_event(event))
}
