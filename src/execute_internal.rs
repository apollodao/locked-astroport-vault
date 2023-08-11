use apollo_cw_asset::{Asset, AssetInfo, AssetList};
use apollo_utils::responses::merge_responses;
use cosmwasm_std::{
    coins, to_binary, BankMsg, CosmosMsg, DepsMut, Env, MessageInfo, Response, StdResult, Uint128,
    WasmMsg,
};
use cw20::Cw20ExecuteMsg;

use crate::error::{ContractError, ContractResponse};
use crate::helpers::{self, burn_vault_tokens, mint_vault_tokens, unwrap_recipient};
use crate::state::{CLAIMS, CONFIG, POOL, STAKING};

use cw_dex::traits::{Pool, Rewards, Stake};

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
    let pool = POOL.load(deps.storage)?;
    let pool_asset_balances: AssetList = pool
        .pool_assets
        .clone()
        .into_iter()
        .map(|asset_info| {
            Ok(Asset::new(
                asset_info.clone(),
                asset_info.query_balance(&deps.querier, &env.contract.address)?,
            ))
        })
        .collect::<StdResult<Vec<Asset>>>()
        .map(Into::into)?;

    let provide_res =
        pool.provide_liquidity(deps.as_ref(), &env, pool_asset_balances, Uint128::zero())?; // TODO: Set slippage?

    Ok(provide_res)
}

pub fn stake_lps(deps: DepsMut, env: Env) -> ContractResponse {
    let cfg = CONFIG.load(deps.storage)?;
    let staking = STAKING.load(deps.storage)?;

    // Query LP token balance
    let lp_token_balance =
        AssetInfo::cw20(cfg.base_token).query_balance(&deps.querier, &env.contract.address)?;

    // Stake LP tokens
    let staking_res = staking.stake(deps.as_ref(), &env, lp_token_balance)?;

    Ok(staking_res)
}

pub fn deposit(
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

pub fn redeem(
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
