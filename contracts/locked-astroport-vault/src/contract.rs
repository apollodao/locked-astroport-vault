use apollo_cw_asset::{Asset, AssetInfo};
use apollo_utils::assets::receive_asset;
#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;

use cosmwasm_std::{
    to_json_binary, Binary, CosmosMsg, Deps, DepsMut, Env, Event, MessageInfo, QueryRequest, Reply,
    Response, StdResult, SubMsg, Uint128, WasmQuery,
};
use cw2::ensure_from_older_version;
use cw_dex_astroport::{astroport, AstroportPool, AstroportStaking};
use cw_utils::Duration;
use cw_vault_standard::extensions::force_unlock::ForceUnlockExecuteMsg;
use cw_vault_standard::extensions::lockup::LockupExecuteMsg;
use osmosis_std::types::osmosis::tokenfactory::v1beta1::MsgCreateDenom;

use crate::error::{ContractError, ContractResponse};
use crate::helpers::{self, IntoInternalCall};
use crate::msg::{
    ApolloExtensionExecuteMsg, ApolloExtensionQueryMsg, ExecuteMsg, ExtensionExecuteMsg,
    ExtensionQueryMsg, InstantiateMsg, InternalMsg, MigrateMsg, QueryMsg,
};
use crate::query::{
    query_force_withdraw_whitelist, query_state, query_unlocking_position,
    query_unlocking_positions, query_vault_info, query_vault_standard_info,
};
use crate::state::{
    ConfigUnchecked, VaultState, BASE_TOKEN, CONFIG, FORCE_WITHDRAW_WHITELIST, POOL, STAKING,
    STATE, VAULT_TOKEN_DENOM,
};
use crate::{execute, query};

pub const CONTRACT_NAME: &str = env!("CARGO_PKG_NAME");
pub const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

/// The ID used in the reply entrypoint for SubMsgs that compound the vault
pub const COMPOUND_REPLY_ID: u64 = 4018u64;

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> ContractResponse {
    cw2::set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
    cw_ownable::initialize_owner(deps.storage, deps.api, Some(&msg.owner))?;

    // Query pair info from astroport pair
    let pair_info = deps
        .querier
        .query::<astroport_v5::asset::PairInfo>(&QueryRequest::Wasm(WasmQuery::Smart {
            contract_addr: msg.pool_addr.clone(),
            msg: to_json_binary(&astroport::pair::QueryMsg::Pair {})?,
        }))?;

    // Store pool info
    let liquidity_manager = msg
        .astroport_liquidity_manager
        .map(|x| deps.api.addr_validate(&x))
        .transpose()?;
    let pool = AstroportPool::new(
        deps.as_ref(),
        deps.api.addr_validate(&msg.pool_addr)?,
        liquidity_manager,
    )?;
    POOL.save(deps.storage, &pool)?;

    // Create, validate and store config
    let config = ConfigUnchecked {
        lock_duration: Duration::Time(msg.lock_duration),
        reward_tokens: msg.reward_tokens,
        deposits_enabled: msg.deposits_enabled,
        router: msg.router,
        reward_liquidation_target: msg.reward_liquidation_target,
        liquidity_helper: msg.liquidity_helper,
        performance_fee: msg.performance_fee.unwrap_or_default(),
        deposit_fee: msg.deposit_fee.unwrap_or_default(),
        withdrawal_fee: msg.withdrawal_fee.unwrap_or_default(),
    }
    .check(deps.as_ref())?;
    CONFIG.save(deps.storage, &config)?;

    // Store base token and vault token denom
    let vault_token_denom = format!(
        "factory/{}/{}",
        env.contract.address, msg.vault_token_subdenom
    );
    let base_token = AssetInfo::from_str(deps.api, &pair_info.liquidity_token);

    BASE_TOKEN.save(deps.storage, &base_token)?;
    VAULT_TOKEN_DENOM.save(deps.storage, &vault_token_denom)?;
    STATE.save(
        deps.storage,
        &VaultState {
            staked_base_tokens: Uint128::zero(),
            vault_token_supply: Uint128::zero(),
        },
    )?;

    // Store staking info
    let staking = AstroportStaking {
        lp_token: base_token.clone(),
        incentives: deps.api.addr_validate(&msg.astroport_incentives_addr)?,
    };
    STAKING.save(deps.storage, &staking)?;

    // Create vault token
    let create_denom_msg: CosmosMsg = MsgCreateDenom {
        sender: env.contract.address.to_string(),
        subdenom: msg.vault_token_subdenom,
    }
    .into();

    Ok(Response::new().add_message(create_denom_msg))
}

#[allow(deprecated)]
#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::Deposit { amount, recipient } => {
            let base_token = BASE_TOKEN.load(deps.storage)?;
            let deposited = Asset::new(base_token.clone(), amount);
            let transfer_res = receive_asset(&info, &env, &deposited)?;

            // Call contract itself first to compound, but as a SubMsg so that we can still
            // deposit if the compound fails
            let compound_msg = SubMsg::reply_on_error(
                InternalMsg::Compound {
                    discount_deposit: amount,
                }
                .into_internal_call(&env, vec![])?,
                COMPOUND_REPLY_ID,
            );

            let recipient = helpers::unwrap_recipient(recipient, &info, deps.api)?;
            let deposit_msg =
                InternalMsg::Deposit { amount, recipient }.into_internal_call(&env, vec![])?;

            Ok(transfer_res
                .add_submessage(compound_msg)
                .add_message(deposit_msg))
        }
        ExecuteMsg::Redeem { recipient, amount } => {
            // Call contract itself first to compound, but as a SubMsg so that we can still
            // redeem if the compound fails
            let compound_msg = SubMsg::reply_on_error(
                ApolloExtensionExecuteMsg::Compound {}.into_internal_call(&env, vec![])?,
                COMPOUND_REPLY_ID,
            );

            let recipient = helpers::unwrap_recipient(recipient, &info, deps.api)?;
            let redeem_msg =
                InternalMsg::Redeem { amount, recipient }.into_internal_call(&env, info.funds)?;

            Ok(Response::new()
                .add_submessage(compound_msg)
                .add_message(redeem_msg))
        }
        ExecuteMsg::VaultExtension(msg) => match msg {
            ExtensionExecuteMsg::Lockup(msg) => match msg {
                LockupExecuteMsg::Unlock { amount } => {
                    let recipient = info.sender.clone();
                    execute::basic_vault::execute_redeem(deps, env, info, amount, recipient, false)
                }
                LockupExecuteMsg::EmergencyUnlock { amount } => {
                    let recipient = info.sender.clone();
                    execute::basic_vault::execute_redeem(deps, env, info, amount, recipient, false)
                }
                LockupExecuteMsg::WithdrawUnlocked {
                    recipient,
                    lockup_id,
                } => execute::lockup::execute_withdraw_unlocked(
                    deps, env, info, recipient, lockup_id,
                ),
            },
            ExtensionExecuteMsg::ForceUnlock(msg) => match msg {
                ForceUnlockExecuteMsg::ForceRedeem { recipient, amount } => {
                    // Check that the sender is whitelisted, then call redeem handler with
                    // force_redeem=true
                    if !FORCE_WITHDRAW_WHITELIST.contains(deps.storage, &info.sender) {
                        return Err(ContractError::Unauthorized {});
                    }
                    let recipient = helpers::unwrap_recipient(recipient, &info, deps.api)?;

                    execute::basic_vault::execute_redeem(deps, env, info, amount, recipient, true)
                }
                ForceUnlockExecuteMsg::ForceWithdrawUnlocking {
                    lockup_id,
                    amount,
                    recipient,
                } => execute::lockup::execute_force_withdraw_unlocking(
                    deps, env, info, amount, recipient, lockup_id,
                ),
                ForceUnlockExecuteMsg::UpdateForceWithdrawWhitelist {
                    add_addresses,
                    remove_addresses,
                } => execute::lockup::execute_update_force_withdraw_whitelist(
                    deps,
                    env,
                    info,
                    add_addresses,
                    remove_addresses,
                ),
            },
            ExtensionExecuteMsg::Internal(msg) => {
                // Assert that only the contract itself can call internal messages
                if info.sender != env.contract.address {
                    return Err(ContractError::Unauthorized {});
                }

                match msg {
                    InternalMsg::Compound { discount_deposit } => {
                        execute::compound::execute_compound(deps, env, discount_deposit)
                    }
                    InternalMsg::SellTokens {} => execute::compound::execute_sell_tokens(deps, env),
                    InternalMsg::ProvideLiquidity {} => {
                        execute::compound::execute_provide_liquidity(deps, env)
                    }
                    InternalMsg::StakeLps { discount_tokens } => {
                        execute::compound::execute_stake_lps(deps, env, discount_tokens)
                    }
                    InternalMsg::Deposit { amount, recipient } => {
                        execute::basic_vault::execute_deposit(deps, env, amount, recipient)
                    }
                    InternalMsg::Redeem { recipient, amount } => {
                        execute::basic_vault::execute_redeem(
                            deps, env, info, amount, recipient, false,
                        )
                    }
                }
            }
            ExtensionExecuteMsg::UpdateOwnership(action) => {
                let ownership =
                    cw_ownable::update_ownership(deps, &env.block, &info.sender, action)?;
                Ok(Response::new().add_attributes(ownership.into_attributes()))
            }
            ExtensionExecuteMsg::Apollo(msg) => match msg {
                ApolloExtensionExecuteMsg::UpdateConfig { updates } => {
                    execute::basic_vault::execute_update_config(deps, info, updates)
                }
                ApolloExtensionExecuteMsg::Compound {} => {
                    execute::compound::execute_compound(deps, env, Uint128::zero())
                }
            },
        },
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::VaultStandardInfo {} => to_json_binary(&query_vault_standard_info(deps)?),
        QueryMsg::Info {} => to_json_binary(&query_vault_info(deps)?),
        #[allow(deprecated)]
        QueryMsg::PreviewDeposit { .. } => unimplemented!("Cannot reliably preview deposit"),
        #[allow(deprecated)]
        QueryMsg::PreviewRedeem { .. } => unimplemented!("Cannot reliably preview redeem"),
        QueryMsg::TotalAssets {} => {
            let state = STATE.load(deps.storage)?;
            to_json_binary(&state.staked_base_tokens)
        }
        QueryMsg::TotalVaultTokenSupply {} => {
            let state = STATE.load(deps.storage)?;
            to_json_binary(&state.vault_token_supply)
        }
        QueryMsg::ConvertToShares { amount } => {
            to_json_binary(&helpers::convert_to_shares(deps, amount))
        }
        QueryMsg::ConvertToAssets { amount } => {
            to_json_binary(&helpers::convert_to_assets(deps, amount))
        }
        QueryMsg::VaultTokenExchangeRate { quote_denom } => {
            to_json_binary(&query::vault_token_exchange_rate(deps, quote_denom)?)
        }
        QueryMsg::VaultExtension(ext_msg) => match ext_msg {
            ExtensionQueryMsg::Lockup(lockup_msg) => match lockup_msg {
                cw_vault_standard::extensions::lockup::LockupQueryMsg::UnlockingPositions {
                    owner,
                    start_after,
                    limit,
                } => to_json_binary(&query_unlocking_positions(deps, owner, start_after, limit)?),
                cw_vault_standard::extensions::lockup::LockupQueryMsg::UnlockingPosition {
                    lockup_id,
                } => to_json_binary(&query_unlocking_position(deps, lockup_id)?),
                cw_vault_standard::extensions::lockup::LockupQueryMsg::LockupDuration {} => {
                    let cfg = CONFIG.load(deps.storage)?;
                    to_json_binary(&cfg.lock_duration)
                }
            },
            ExtensionQueryMsg::Apollo(msg) => match msg {
                ApolloExtensionQueryMsg::Config {} => {
                    let cfg = CONFIG.load(deps.storage)?;
                    to_json_binary(&cfg)
                }
                ApolloExtensionQueryMsg::Ownership {} => {
                    let ownership = cw_ownable::get_ownership(deps.storage)?;
                    to_json_binary(&ownership)
                }
                ApolloExtensionQueryMsg::ContractVersion {} => {
                    let version = cw2::get_contract_version(deps.storage)?;
                    to_json_binary(&version)
                }
                ApolloExtensionQueryMsg::ForceWithdrawWhitelist { start_after, limit } => {
                    let whitelist = query_force_withdraw_whitelist(deps, start_after, limit)?;
                    to_json_binary(&whitelist)
                }
                ApolloExtensionQueryMsg::State {} => {
                    let state_res = query_state(deps)?;
                    to_json_binary(&state_res)
                }
            },
        },
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn reply(_deps: DepsMut, _env: Env, msg: Reply) -> Result<Response, ContractError> {
    // If this reply is triggered by a compound SubMsg::ReplyOnError, we add an
    // event so it can be seen in the transaction logs. Unfortunately we can't
    // add the error, because error messages are still redacted
    // (https://github.com/CosmWasm/wasmd/issues/1160).
    if msg.id == COMPOUND_REPLY_ID {
        let event = Event::new("apollo/vaults/execute_compound")
            .add_attribute("action", "reply on compound failed");
        return Ok(Response::new().add_event(event));
    }
    Ok(Response::new())
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn migrate(deps: DepsMut, _env: Env, _msg: MigrateMsg) -> Result<Response, ContractError> {
    ensure_from_older_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    cw2::set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    Ok(Response::new())
}
