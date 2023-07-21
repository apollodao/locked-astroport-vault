use cosmwasm_std::{
    entry_point, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdError, StdResult,
};
use cw_dex::{
    astroport::{AstroportPool, AstroportStaking},
    traits::{Pool, Staking},
};
use cw_ownable::OwnershipError;
use cw_vault_standard::{
    extensions::{force_unlock::ForceUnlockExecuteMsg, lockup::LockupExecuteMsg},
    ExtensionExecuteMsg,
};
use osmosis_std::types::osmosis::tokenfactory::v1beta1::MsgCreateDenom;

use crate::{
    error::{ContractError, ContractResponse},
    execute::execute_deposit,
    msg::{ExecuteMsg, InstantiateMsg, QueryMsg},
    state::{Config, CONFIG, POOL, STAKING},
};

pub const CONTRACT_NAME: &str = "crates.io:my-contract";
pub const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[entry_point]
pub fn instantiate(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> ContractResponse {
    cw2::set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
    cw_ownable::initialize_owner(deps.storage, deps.api, Some(&msg.owner))?;

    let vault_token_denom = format!(
        "factory/{}/{}",
        env.contract.address, msg.vault_token_subdenom
    );

    let cw20_adaptor = match msg.cw20_adaptor {
        Some(adaptor) => Some(deps.api.addr_validate(&adaptor)?),
        None => None,
    };

    let config = Config {
        base_token_denom: msg.base_token_denom,
        vault_token_denom,
        cw20_adaptor,
    };

    CONFIG.save(deps.storage, &config)?;
    POOL.save(deps.storage, &msg.pool)?;
    STAKING.save(deps.storage, &msg.staking)?;

    Ok(Response::new())
}

#[entry_point]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::Deposit { amount, recipient } => execute_deposit(deps, env, info, amount),
        ExecuteMsg::Redeem { recipient, amount } => todo!(),
        ExecuteMsg::VaultExtension(msg) => match msg {
            ExtensionExecuteMsg::Lockup(msg) => match msg {
                LockupExecuteMsg::Unlock { amount } => {
                    todo!()
                }
                LockupExecuteMsg::EmergencyUnlock { amount } => todo!(),
                LockupExecuteMsg::WithdrawUnlocked {
                    recipient,
                    lockup_id,
                } => todo!(),
            },
            ExtensionExecuteMsg::ForceUnlock(msg) => match msg {
                ForceUnlockExecuteMsg::ForceRedeem { recipient, amount } => todo!(),
                ForceUnlockExecuteMsg::ForceWithdrawUnlocking {
                    lockup_id,
                    amount,
                    recipient,
                } => todo!(),
                ForceUnlockExecuteMsg::UpdateForceWithdrawWhitelist {
                    add_addresses,
                    remove_addresses,
                } => todo!(),
            },
        },
    }
}

#[entry_point]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::VaultStandardInfo {} => todo!(),
        QueryMsg::Info {} => todo!(),
        QueryMsg::PreviewDeposit { amount } => todo!(),
        QueryMsg::PreviewRedeem { amount } => todo!(),
        QueryMsg::TotalAssets {} => todo!(),
        QueryMsg::TotalVaultTokenSupply {} => todo!(),
        QueryMsg::ConvertToShares { amount } => todo!(),
        QueryMsg::ConvertToAssets { amount } => todo!(),
        QueryMsg::VaultExtension(_) => todo!(),
    }
}

// ----------------------------------- Tests -----------------------------------

#[cfg(test)]
mod tests {
    use cosmwasm_std::{
        testing::{mock_dependencies, mock_env, mock_info},
        Addr,
    };
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
                base_token_denom: todo!(),
                vault_token_subdenom: todo!(),
                pool: todo!(),
                staking: todo!(),
                cw20_adaptor: todo!(),
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
