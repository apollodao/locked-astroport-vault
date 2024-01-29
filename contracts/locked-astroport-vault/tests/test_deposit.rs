use std::str::FromStr;

use common::{instantiate_wsteth_eth_vault, DEPS_PATH};
use cosmwasm_std::{Coins, Decimal, Uint128};
use cw_it::helpers::Unwrap;
use cw_it::test_tube::Account;
use cw_it::traits::CwItRunner;
use cw_vault_standard_test_helpers::traits::CwVaultStandardRobot;
use locked_astroport_vault::helpers::INITIAL_VAULT_TOKENS_PER_BASE_TOKEN;
use locked_astroport_vault::state::{ConfigUpdates, FeeConfig};
use locked_astroport_vault_test_helpers::robot::{LockedAstroportVaultRobot, DEFAULT_COINS};

pub mod common;
pub use common::get_test_runner;

use crate::common::default_instantiate;

#[test]
fn test_deposit() {
    let owned_runner = get_test_runner();
    let runner = owned_runner.as_ref();
    let admin = runner
        .init_account(&Coins::from_str(DEFAULT_COINS).unwrap().to_vec())
        .unwrap();
    let dependencies = LockedAstroportVaultRobot::instantiate_deps(&runner, &admin, DEPS_PATH);
    let (robot, _base_pool, _treasury) = default_instantiate(&runner, &admin, &dependencies);
    let user = robot.new_user(&admin);

    // Deposit some funds and assert the vault token balance is correct
    let base_token_balance = robot.query_base_token_balance(user.address());
    let deposit_amount = base_token_balance / Uint128::new(2);
    robot
        .deposit_cw20(deposit_amount, None, Unwrap::Ok, &user)
        .assert_vault_token_balance_eq(
            user.address(),
            deposit_amount * INITIAL_VAULT_TOKENS_PER_BASE_TOKEN,
        )
        .assert_total_vault_token_supply_eq(deposit_amount * INITIAL_VAULT_TOKENS_PER_BASE_TOKEN)
        .assert_total_vault_assets_eq(deposit_amount);
}

#[test]
fn can_only_deposit_when_despoits_enabled() {
    let owned_runner = get_test_runner();
    let runner = owned_runner.as_ref();
    let admin = runner
        .init_account(&Coins::from_str(DEFAULT_COINS).unwrap().to_vec())
        .unwrap();
    let dependencies = LockedAstroportVaultRobot::instantiate_deps(&runner, &admin, DEPS_PATH);
    let (robot, _base_pool, _treasury) = default_instantiate(&runner, &admin, &dependencies);
    let user = robot.new_user(&admin);

    // Deposit, should work
    let deposit_amount = Uint128::new(100);
    robot
        .deposit_cw20(deposit_amount, None, Unwrap::Ok, &user)
        .assert_vault_token_balance_eq(
            user.address(),
            deposit_amount * INITIAL_VAULT_TOKENS_PER_BASE_TOKEN,
        );

    // Disable deposits
    robot.update_config(
        ConfigUpdates {
            deposits_enabled: Some(false),
            ..Default::default()
        },
        Unwrap::Ok,
        &admin,
    );

    //Deposit, should fail
    robot.deposit_cw20(
        deposit_amount,
        None,
        Unwrap::Err("Deposits are disabled"),
        &user,
    );
}

#[test]
fn deposit_fee_works() {
    let owned_runner = get_test_runner();
    let runner = owned_runner.as_ref();
    let admin = runner
        .init_account(&Coins::from_str(DEFAULT_COINS).unwrap().to_vec())
        .unwrap();
    let dependencies = LockedAstroportVaultRobot::instantiate_deps(&runner, &admin, DEPS_PATH);
    let treasury = runner.init_account(&[]).unwrap();
    let fee_rate = Decimal::percent(1);
    let deposit_fee = Some(FeeConfig {
        fee_rate,
        fee_recipients: vec![(treasury.address(), Decimal::percent(100))],
    });
    let robot =
        instantiate_wsteth_eth_vault(&runner, &admin, None, deposit_fee, None, &dependencies);
    let user = robot.new_user(&admin);

    // Deposit some funds and assert the vault token balance is correct
    let base_token_balance = robot.query_base_token_balance(user.address());
    let deposit_amount = base_token_balance / Uint128::new(2);
    let expected_amount =
        deposit_amount * (Decimal::one() - fee_rate) * INITIAL_VAULT_TOKENS_PER_BASE_TOKEN;
    robot
        .deposit_cw20(deposit_amount, None, Unwrap::Ok, &user)
        .assert_vault_token_balance_eq(user.address(), expected_amount)
        .assert_total_vault_token_supply_eq(expected_amount)
        .assert_total_vault_assets_eq(deposit_amount * (Decimal::one() - fee_rate));
}
