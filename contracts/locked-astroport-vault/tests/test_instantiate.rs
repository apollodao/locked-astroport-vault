use std::str::FromStr;

use apollo_cw_asset::AssetInfo;
use cosmwasm_std::{Addr, Coins};
use cw_dex::astroport::AstroportStaking;
use cw_it::traits::CwItRunner;
use cw_ownable::Ownership;
use cw_vault_standard::VaultStandardInfoResponse;
use locked_astroport_vault::state::StateResponse;
use locked_astroport_vault_test_helpers::robot::{LockedAstroportVaultRobot, DEFAULT_COINS};

pub mod common;
pub use common::get_test_runner;

use crate::common::{default_instantiate, DEPS_PATH};

#[test]
fn test_instantiation() {
    let owned_runner = get_test_runner();
    let runner = owned_runner.as_ref();
    let admin = runner
        .init_account(&Coins::from_str(DEFAULT_COINS).unwrap().to_vec())
        .unwrap();
    let dependencies = LockedAstroportVaultRobot::instantiate_deps(&runner, &admin, DEPS_PATH);
    let (robot, base_pool, _treasury) = default_instantiate(&runner, &admin, &dependencies);

    // Query ownership to confirm
    let ownership = robot.query_ownership();
    assert!(matches!(
        ownership,
        Ownership {
            owner: Some(_),
            pending_owner: None,
            pending_expiry: None,
        },
    ));

    // Query contract version
    let version = robot.query_contract_version();
    assert_eq!(
        version,
        cw2::ContractVersion {
            contract: locked_astroport_vault::contract::CONTRACT_NAME.to_string(),
            version: locked_astroport_vault::contract::CONTRACT_VERSION.to_string(),
        }
    );

    // Query vault standard info
    let vault_standard_info = robot.query_vault_standard_info();
    assert_eq!(
        vault_standard_info,
        VaultStandardInfoResponse {
            version: 0,
            extensions: vec![
                "internal".to_string(),
                "lockup".to_string(),
                "force-unlock".to_string(),
                "apollo".to_string(),
                "update-ownership".to_string(),
            ]
        }
    );

    // Query vault's non-configurable state
    let state = robot.query_state();
    assert_eq!(
        state,
        StateResponse {
            base_token: base_pool.lp_token_addr.clone(),
            pool: base_pool.clone(),
            staked_base_tokens: 0u128.into(),
            vault_token_supply: 0u128.into(),
            staking: AstroportStaking {
                lp_token_addr: base_pool.lp_token_addr,
                generator_addr: Addr::unchecked(
                    &dependencies.astroport_contracts.generator.address
                ),
                astro_token: AssetInfo::native("uastro"),
            },
            vault_token_denom: format!("factory/{}/testVaultToken", robot.vault_addr),
        }
    )
}
