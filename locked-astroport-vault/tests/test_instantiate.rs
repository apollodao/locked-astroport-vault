use std::str::FromStr;

use cosmwasm_std::{coin, Coins};
use cw_it::{test_tube::Account, traits::CwItRunner, TestRunner};
use cw_ownable::Ownership;
use locked_astroport_vault_test_helpers::robot::{LockedAstroportVaultRobot, DEFAULT_COINS};

pub const UNOPTIMIZED_PATH: &str = "../target/wasm32-unknown-unknown/release";

fn get_runner<'a>() -> TestRunner<'a> {
    TestRunner::from_str(
        &std::env::var("TEST_RUNNER").unwrap_or_else(|_| "osmosis-test-app".into()),
    )
    .unwrap()
}

#[test]
fn test_instantiation() {
    let runner = get_runner();
    let vault_contract = LockedAstroportVaultRobot::local_contract(Some(UNOPTIMIZED_PATH));
    let admin = runner
        .init_account(&Coins::from_str(DEFAULT_COINS).unwrap().to_vec())
        .unwrap();
    let treasury_addr = runner.init_account(&[]).unwrap();
    let token_factory_fee = coin(10_000_000, "uosmo");
    let robot = LockedAstroportVaultRobot::new_wsteth_eth_vault(
        &runner,
        vault_contract,
        token_factory_fee,
        treasury_addr.address(),
        None,
        &admin,
    );

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
}
