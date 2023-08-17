use std::str::FromStr;

use cosmwasm_std::Coins;
use cw_it::traits::CwItRunner;
use cw_ownable::Ownership;
use locked_astroport_vault_test_helpers::robot::{LockedAstroportVaultRobot, DEFAULT_COINS};

pub mod common;
pub use common::{get_test_runner, UNOPTIMIZED_PATH};

use crate::common::default_instantiate;

#[test]
fn test_instantiation() {
    let runner = get_test_runner();
    let admin = runner
        .init_account(&Coins::from_str(DEFAULT_COINS).unwrap().to_vec())
        .unwrap();
    let dependencies = LockedAstroportVaultRobot::instantiate_deps(&runner, &admin, None);
    let (robot, _treasury) = default_instantiate(&runner, &admin, &dependencies);

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
