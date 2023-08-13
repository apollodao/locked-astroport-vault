use std::collections::HashSet;

use cosmwasm_std::{
    entry_point, to_binary, Binary, CosmosMsg, Decimal, Deps, DepsMut, Env, MessageInfo, Reply,
    Response, StdResult, SubMsg,
};
use cw_utils::Duration;
use cw_vault_standard::extensions::force_unlock::ForceUnlockExecuteMsg;
use cw_vault_standard::extensions::lockup::LockupExecuteMsg;
use osmosis_std::types::osmosis::tokenfactory::v1beta1::MsgCreateDenom;

use crate::error::{ContractError, ContractResponse};
use crate::execute::{
    execute_compound, execute_force_redeem, execute_force_withdraw_unlocking,
    execute_update_whitelist, execute_withdraw_unlocked,
};
use crate::execute_internal::{self};
use crate::helpers::{self, IntoInternalCall, IsZero};
use crate::msg::{
    ApolloExtensionExecuteMsg, ExecuteMsg, ExtensionExecuteMsg, InstantiateMsg, InternalMsg,
    QueryMsg,
};
use crate::query::{
    query_unlocking_position, query_unlocking_positions, query_vault_info,
    query_vault_standard_info,
};
use crate::state::{Config, CONFIG, POOL, STAKING, STATE};

pub const CONTRACT_NAME: &str = "crates.io:my-contract";
pub const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[entry_point]
pub fn instantiate(
    deps: DepsMut,
    env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> ContractResponse {
    cw2::set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
    cw_ownable::initialize_owner(deps.storage, deps.api, Some(&msg.owner))?;

    let vault_token_denom = format!(
        "factory/{}/{}",
        env.contract.address, msg.vault_token_subdenom
    );

    // Validate performance fee
    if msg.performance_fee > Decimal::one() {
        return Err(ContractError::PerformanceFeeTooHigh {});
    }

    let config = Config {
        vault_token_denom,
        base_token: deps.api.addr_validate(&msg.base_token_addr)?,
        lock_duration: Duration::Time(msg.lock_duration),
        reward_tokens: msg
            .reward_tokens
            .iter()
            .map(|asset_info| asset_info.check(deps.api))
            .collect::<StdResult<Vec<_>>>()?,
        force_withdraw_whitelist: HashSet::new(),
        deposits_enabled: msg.deposits_enabled,
        treasury: deps.api.addr_validate(&msg.treasury)?,
        performance_fee: msg.performance_fee,
        router: msg.router.check(deps.api)?,
        reward_liquidation_target: msg.reward_liquidation_target.check(deps.api)?,
    };

    CONFIG.save(deps.storage, &config)?;
    POOL.save(deps.storage, &msg.pool)?;
    STAKING.save(deps.storage, &msg.staking)?;

    // Create vault token
    let create_denom_msg: CosmosMsg = MsgCreateDenom {
        sender: env.contract.address.to_string(),
        subdenom: msg.vault_token_subdenom,
    }
    .into();

    Ok(Response::new().add_message(create_denom_msg))
}

#[entry_point]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    let cfg = CONFIG.load(deps.storage)?;
    match msg {
        ExecuteMsg::Deposit { amount, recipient } => {
            // Call contract itself first to compound, but as a SubMsg so that we can still deposit
            // if the compound fails
            let compound_msg = SubMsg::reply_always(
                ApolloExtensionExecuteMsg::Compound {}.into_internal_call(&env)?,
                0,
            );

            let deposit_msg =
                InternalMsg::Deposit { amount, recipient }.into_internal_call(&env)?;

            Ok(Response::new()
                .add_submessage(compound_msg)
                .add_message(deposit_msg))
        }
        ExecuteMsg::Redeem { recipient, amount } => {
            // Call contract itself first to compound, but as a SubMsg so that we can still redeem
            // if the compound fails
            let compound_msg = SubMsg::reply_always(
                ApolloExtensionExecuteMsg::Compound {}.into_internal_call(&env)?,
                0,
            );

            let redeem_msg = InternalMsg::Redeem { amount, recipient }.into_internal_call(&env)?;

            Ok(Response::new()
                .add_submessage(compound_msg)
                .add_message(redeem_msg))
        }
        ExecuteMsg::VaultExtension(msg) => match msg {
            ExtensionExecuteMsg::Lockup(msg) => {
                if cfg.lock_duration.is_zero() {
                    return Err(ContractError::LockupDisabled {});
                }

                match msg {
                    LockupExecuteMsg::Unlock { amount } => {
                        // TODO: Remove Unlock in favor of Redeem?
                        let recipient = Some(info.sender.to_string());
                        execute_internal::redeem(deps, env, info, amount, recipient)
                    }
                    LockupExecuteMsg::EmergencyUnlock { amount } => {
                        let recipient = Some(info.sender.to_string()); // TODO: Add recipient field to LockupExecuteMsg?
                        execute_internal::redeem(deps, env, info, amount, recipient)
                    }
                    LockupExecuteMsg::WithdrawUnlocked {
                        recipient,
                        lockup_id,
                    } => execute_withdraw_unlocked(deps, env, info, recipient, lockup_id),
                }
            }
            ExtensionExecuteMsg::ForceUnlock(msg) => {
                if cfg.lock_duration.is_zero() {
                    return Err(ContractError::LockupDisabled {});
                }

                match msg {
                    ForceUnlockExecuteMsg::ForceRedeem { recipient, amount } => {
                        execute_force_redeem(deps, env, info, amount, recipient)
                    }
                    ForceUnlockExecuteMsg::ForceWithdrawUnlocking {
                        lockup_id,
                        amount,
                        recipient,
                    } => execute_force_withdraw_unlocking(
                        deps, env, info, amount, recipient, lockup_id,
                    ),
                    ForceUnlockExecuteMsg::UpdateForceWithdrawWhitelist {
                        add_addresses,
                        remove_addresses,
                    } => execute_update_whitelist(deps, env, info, add_addresses, remove_addresses),
                }
            }
            ExtensionExecuteMsg::Internal(msg) => {
                // Assert that only the contract itself can call internal messages
                if info.sender != env.contract.address {
                    return Err(ContractError::Unauthorized {});
                }

                match msg {
                    InternalMsg::SellTokens {} => execute_internal::sell_tokens(deps, env),
                    InternalMsg::ProvideLiquidity {} => {
                        execute_internal::provide_liquidity(deps, env)
                    }
                    InternalMsg::StakeLps {} => execute_internal::stake_lps(deps, env),
                    InternalMsg::Deposit { amount, recipient } => {
                        execute_internal::deposit(deps, env, info, amount, recipient)
                    }
                    InternalMsg::Redeem { recipient, amount } => {
                        execute_internal::redeem(deps, env, info, amount, recipient)
                    }
                }
            }
            ExtensionExecuteMsg::UpdateOwnership(action) => {
                let ownership =
                    cw_ownable::update_ownership(deps, &env.block, &info.sender, action)?;
                Ok(Response::new().add_attributes(ownership.into_attributes()))
            }
            ExtensionExecuteMsg::Apollo(msg) => match msg {
                ApolloExtensionExecuteMsg::UpdateConfig {} => todo!(),
                ApolloExtensionExecuteMsg::Compound {} => execute_compound(deps, env),
            },
        },
    }
}

#[entry_point]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::VaultStandardInfo {} => to_binary(&query_vault_standard_info(deps)?),
        QueryMsg::Info {} => to_binary(&query_vault_info(deps)?),
        QueryMsg::PreviewDeposit { .. } => unimplemented!("Cannot reliably preview deposit"),
        QueryMsg::PreviewRedeem { .. } => unimplemented!("Cannot reliably preview redeem"),
        QueryMsg::TotalAssets {} => {
            let state = STATE.load(deps.storage)?;
            to_binary(&state.staked_base_tokens)
        }
        QueryMsg::TotalVaultTokenSupply {} => {
            let state = STATE.load(deps.storage)?;
            to_binary(&state.vault_token_supply)
        }
        QueryMsg::ConvertToShares { amount } => {
            to_binary(&helpers::convert_to_shares(deps, amount))
        }
        QueryMsg::ConvertToAssets { amount } => {
            to_binary(&helpers::convert_to_assets(deps, amount))
        }
        QueryMsg::VaultExtension(ext_msg) => match ext_msg {
            cw_vault_standard::ExtensionQueryMsg::Lockup(lockup_msg) => match lockup_msg {
                cw_vault_standard::extensions::lockup::LockupQueryMsg::UnlockingPositions {
                    owner,
                    start_after,
                    limit,
                } => to_binary(&query_unlocking_positions(deps, owner, start_after, limit)?),
                cw_vault_standard::extensions::lockup::LockupQueryMsg::UnlockingPosition {
                    lockup_id,
                } => to_binary(&query_unlocking_position(deps, lockup_id)?),
                cw_vault_standard::extensions::lockup::LockupQueryMsg::LockupDuration {} => {
                    let cfg = CONFIG.load(deps.storage)?;
                    to_binary(&cfg.lock_duration)
                }
            },
        },
    }
}

#[entry_point]
pub fn reply(_deps: DepsMut, _env: Env, _msg: Reply) -> Result<Response, ContractError> {
    Ok(Response::new())
}

// ----------------------------------- Tests -----------------------------------

#[cfg(test)]
mod tests {
    use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info};
    use cosmwasm_std::Addr;
    use cw2::ContractVersion;
    use cw_ownable::Ownership;

    use super::*;

    #[test]
    fn proper_initialization() {
        let mut deps = mock_dependencies();

        // run instantiation logic
        instantiate(
            deps.as_mut(),
            mock_env(),
            mock_info("larry", &[]),
            InstantiateMsg {
                owner: "pumpkin".into(),
                vault_token_subdenom: todo!(),
                pool: todo!(),
                staking: todo!(),
                base_token_addr: todo!(),
                lock_duration: todo!(),
                reward_tokens: todo!(),
                deposits_enabled: todo!(),
                treasury: todo!(),
                performance_fee: todo!(),
                router: todo!(),
                reward_liquidation_target: todo!(),
            },
        )
        .unwrap();

        // correct cw2 version info should have been stored
        let version = cw2::get_contract_version(deps.as_ref().storage).unwrap();
        assert_eq!(
            version,
            ContractVersion {
                contract: CONTRACT_NAME.into(),
                version: CONTRACT_VERSION.into(),
            },
        );

        // correct ownership info should have been stored
        let ownership = cw_ownable::get_ownership(deps.as_ref().storage).unwrap();
        assert_eq!(
            ownership,
            Ownership {
                owner: Some(Addr::unchecked("pumpkin")),
                pending_owner: None,
                pending_expiry: None,
            },
        );
    }
}