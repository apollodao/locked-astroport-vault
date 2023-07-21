use std::time::Instant;

use apollo_utils::responses::merge_responses;
use cosmwasm_schema::serde::{de::DeserializeOwned, Serialize};
use cosmwasm_std::{
    coin, to_binary, CosmosMsg, Decimal, DepsMut, Env, MessageInfo, Response, Uint128, WasmMsg,
};
use cw20::Cw20ExecuteMsg;
use cw_storage_plus::Item;
use osmosis_std::types::osmosis::tokenfactory::v1beta1::{MsgBurn, MsgMint};

use crate::{
    error::{ContractResponse, ContractResult},
    msg::InstantiateMsg,
    state::{self, Config, VaultState, CONFIG, STAKING, STATE},
    utils,
};

use cw_dex::traits::{Pool, Stake, Staking};

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

fn mint_vault_tokens(
    deps: DepsMut,
    env: Env,
    deposit_amount: Uint128,
) -> ContractResult<CosmosMsg> {
    let mut state = STATE.load(deps.storage)?;
    let cfg = CONFIG.load(deps.storage)?;

    let mint_amount =
        Decimal::from_ratio(deposit_amount, state.staked_base_tokens) * state.vault_token_supply;

    state.staked_base_tokens = state.staked_base_tokens.checked_add(deposit_amount)?;
    state.vault_token_supply = state.vault_token_supply.checked_add(mint_amount)?;

    STATE.save(deps.storage, &state)?;

    Ok(MsgMint {
        sender: env.contract.address.to_string(),
        amount: Some(coin(mint_amount.u128(), &cfg.vault_token_denom).into()),
    }
    .into())
}

pub fn execute_unlock(deps: DepsMut, env: Env, info: MessageInfo) -> ContractResponse {
    let cfg = CONFIG.load(deps.storage)?;
    let unlock_amount = utils::one_coin(info, cfg.vault_token_denom)?;
}
