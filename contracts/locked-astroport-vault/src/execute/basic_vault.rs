use apollo_cw_asset::Asset;
use apollo_utils::responses::merge_responses;
use cosmwasm_std::{
    coins, Addr, BankMsg, CosmosMsg, DepsMut, Env, Event, MessageInfo, Response, Uint128,
};
use cw_vault_standard::extensions::lockup::{
    UNLOCKING_POSITION_ATTR_KEY, UNLOCKING_POSITION_CREATED_EVENT_TYPE,
};
use optional_struct::Applyable;

use crate::error::{ContractError, ContractResponse};
use crate::helpers::{self, burn_vault_tokens, mint_vault_tokens, IsZero};
use crate::state::{
    self, ConfigUnchecked, ConfigUpdates, BASE_TOKEN, CONFIG, STAKING, VAULT_TOKEN_DENOM,
};

use cw_dex::traits::{Stake, Unstake};

pub fn execute_deposit(
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
    let (mint_msg, mint_amount) = mint_vault_tokens(deps, env, amount, &vault_token_denom)?;

    // Send minted vault tokens to recipient
    let send_msg: CosmosMsg = BankMsg::Send {
        to_address: recipient.to_string(),
        amount: coins(mint_amount.u128(), vault_token_denom),
    }
    .into();

    let event = Event::new("apollo/vaults/execute_deposit")
        .add_attribute("deposit_amount", amount)
        .add_attribute("vault_tokens_minted", mint_amount);

    Ok(merge_responses(vec![transfer_from_res, staking_res])
        .add_message(mint_msg)
        .add_message(send_msg)
        .add_event(event))
}

pub fn execute_redeem(
    mut deps: DepsMut,
    env: Env,
    info: MessageInfo,
    amount: Uint128,
    recipient: Addr,
    force_redeem: bool,
) -> ContractResponse {
    let cfg = CONFIG.load(deps.storage)?;
    let base_token = BASE_TOKEN.load(deps.storage)?;
    let vt_denom = VAULT_TOKEN_DENOM.load(deps.storage)?;

    // Check that only vault tokens were sent and that the amount is correct
    helpers::assert_correct_funds(&info, &vt_denom, amount)?;

    // Calculate claim amount and create msg to burn vault tokens
    let (burn_msg, claim_amount) = burn_vault_tokens(deps.branch(), &env, amount, &vt_denom)?;

    // If lock duration is zero or this is a force redeem, unstake LP tokens and
    // send them to recipient, else create a claim for recipient so they can
    // call `WithdrawUnlocked` later.
    let res = if cfg.lock_duration.is_zero() || force_redeem {
        // Unstake LP tokens
        let staking = STAKING.load(deps.storage)?;
        let res = staking.unstake(deps.as_ref(), &env, claim_amount)?;

        // Send LP tokens to recipient
        let send_msg = Asset::cw20(base_token, claim_amount).transfer_msg(&recipient)?;

        res.add_message(send_msg)
    } else {
        // Create claim for recipient
        let claim = state::claims().create_claim(
            deps.storage,
            &recipient,
            claim_amount,
            cfg.lock_duration.after(&env.block),
        )?;
        let event = Event::new(UNLOCKING_POSITION_CREATED_EVENT_TYPE)
            .add_attribute(UNLOCKING_POSITION_ATTR_KEY, format!("{}", claim.id));
        Response::new().add_event(event)
    };

    let event = Event::new("apollo/vaults/execute_redeem")
        .add_attribute("is_force_redeem", format!("{}", force_redeem))
        .add_attribute("vault_tokens_redeemed", amount)
        .add_attribute("lp_tokens_claimed", claim_amount);

    Ok(res.add_message(burn_msg).add_event(event))
}

pub fn execute_update_config(
    deps: DepsMut,
    info: MessageInfo,
    updates: ConfigUpdates<String>,
) -> ContractResponse {
    cw_ownable::assert_owner(deps.storage, &info.sender)?;

    let event = Event::new("apollo/vaults/execute_update_config")
        .add_attribute("updates", format!("{:?}", updates));

    let mut config: ConfigUnchecked = CONFIG.load(deps.storage)?.into();

    updates.apply_to(&mut config);

    let config = config.check(deps.as_ref())?;

    CONFIG.save(deps.storage, &config)?;

    Ok(Response::new().add_event(event))
}
