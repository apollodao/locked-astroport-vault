use std::str::FromStr;

use cosmwasm_std::{Coins, Uint128};
use cw_it::{astroport::robot::AstroportTestRobot, test_tube::Account, traits::CwItRunner};
use cw_vault_standard_test_helpers::traits::CwVaultStandardRobot;
use locked_astroport_vault::state::ConfigUpdates;
use locked_astroport_vault_test_helpers::robot::{LockedAstroportVaultRobot, DEFAULT_COINS};

pub mod common;
pub use common::{get_test_runner, UNOPTIMIZED_PATH};

use crate::common::default_instantiate;

#[test]
fn test_deposit() {
    let runner = get_test_runner();
    let admin = runner
        .init_account(&Coins::from_str(DEFAULT_COINS).unwrap().to_vec())
        .unwrap();
    let dependencies = LockedAstroportVaultRobot::instantiate_deps(&runner, &admin, None);
    let (robot, _treasury) = default_instantiate(&runner, &admin, &dependencies);
    let user = robot.new_user(&admin);

    // Deposit some funds and assert the vault token balance is correct
    let base_token_balance = robot.query_base_token_balance(user.address());
    let deposit_amount = base_token_balance / Uint128::new(2);
    robot
        .deposit_cw20(deposit_amount, None, &user)
        .assert_vault_token_balance_eq(user.address().to_string(), deposit_amount)
        .assert_total_vault_token_supply_eq(deposit_amount)
        .assert_total_vault_assets_eq(deposit_amount);
}

#[test]
#[should_panic(expected = "Deposits are disabled")]
fn can_only_deposit_when_despoits_enabled() {
    let runner = get_test_runner();
    let admin = runner
        .init_account(&Coins::from_str(DEFAULT_COINS).unwrap().to_vec())
        .unwrap();
    let dependencies = LockedAstroportVaultRobot::instantiate_deps(&runner, &admin, None);
    let (robot, _treasury) = default_instantiate(&runner, &admin, &dependencies);
    let user = robot.new_user(&admin);

    // Deposit, should work
    let deposit_amount = Uint128::new(100);
    robot
        .deposit_cw20(deposit_amount, None, &user)
        .assert_vault_token_balance_eq(user.address(), deposit_amount);

    // Disable deposits
    robot.update_config(
        ConfigUpdates {
            deposits_enabled: Some(false),
            ..Default::default()
        },
        &admin,
    );

    //Deposit, should fail
    robot
        .increase_cw20_allowance(
            &robot.base_token(),
            &robot.vault_addr,
            deposit_amount,
            &user,
        )
        .deposit(deposit_amount, None, &[], &user);
}
