use std::str::FromStr;

use common::{instantiate_wsteth_eth_vault, DEPS_PATH};
use cosmwasm_std::{coins, Coins, Decimal, Uint128};
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
    let funds = coins(deposit_amount.u128(), robot.base_token());
    robot
        .deposit_native(deposit_amount, None, Unwrap::Ok, &user, &funds)
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
    let funds = coins(deposit_amount.u128(), robot.base_token());
    robot
        .deposit_native(deposit_amount, None, Unwrap::Ok, &user, &funds)
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

    let funds = coins(deposit_amount.u128(), robot.base_token());
    //Deposit, should fail
    robot.deposit_native(
        deposit_amount,
        None,
        Unwrap::Err("Deposits are disabled"),
        &user,
        &funds,
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
    let funds = coins(deposit_amount.u128(), robot.base_token());
    robot
        .deposit_native(deposit_amount, None, Unwrap::Ok, &user, &funds)
        .assert_vault_token_balance_eq(user.address(), expected_amount)
        .assert_total_vault_token_supply_eq(expected_amount)
        .assert_total_vault_assets_eq(deposit_amount * (Decimal::one() - fee_rate))
        .assert_base_token_balance_eq(treasury.address(), deposit_amount * fee_rate);
}

#[test]
fn deposit_fee_works_with_multiple_recipients() {
    let owned_runner = get_test_runner();
    let runner = owned_runner.as_ref();
    let admin = runner
        .init_account(&Coins::from_str(DEFAULT_COINS).unwrap().to_vec())
        .unwrap();
    let dependencies = LockedAstroportVaultRobot::instantiate_deps(&runner, &admin, DEPS_PATH);
    let treasury_one = runner.init_account(&[]).unwrap();
    let treasury_two = runner.init_account(&[]).unwrap();
    let fee_rate = Decimal::percent(5);
    let deposit_fee = Some(FeeConfig {
        fee_rate,
        fee_recipients: vec![
            (treasury_one.address(), Decimal::percent(80)),
            (treasury_two.address(), Decimal::percent(20)),
        ],
    });
    let robot =
        instantiate_wsteth_eth_vault(&runner, &admin, None, deposit_fee, None, &dependencies);
    let user = robot.new_user(&admin);

    // Deposit some funds and assert the vault token balance is correct
    let base_token_balance = robot.query_base_token_balance(user.address());
    let deposit_amount = base_token_balance / Uint128::new(2);
    let expected_amount =
        deposit_amount * (Decimal::one() - fee_rate) * INITIAL_VAULT_TOKENS_PER_BASE_TOKEN;
    let funds = coins(deposit_amount.u128(), robot.base_token());
    robot
        .deposit_native(deposit_amount, None, Unwrap::Ok, &user, &funds)
        .assert_vault_token_balance_eq(user.address(), expected_amount)
        .assert_total_vault_token_supply_eq(expected_amount)
        .assert_total_vault_assets_eq(deposit_amount * (Decimal::one() - fee_rate))
        .assert_base_token_balance_eq(
            treasury_one.address(),
            deposit_amount * fee_rate * Decimal::percent(80),
        )
        .assert_base_token_balance_eq(
            treasury_two.address(),
            deposit_amount * fee_rate * Decimal::percent(20),
        );
}

#[test]
fn deposit_fee_works_with_vault_as_recipient() {
    let owned_runner = get_test_runner();
    let runner = owned_runner.as_ref();
    let admin = runner
        .init_account(&Coins::from_str(DEFAULT_COINS).unwrap().to_vec())
        .unwrap();
    let dependencies = LockedAstroportVaultRobot::instantiate_deps(&runner, &admin, DEPS_PATH);
    let fee_rate = Decimal::percent(1);
    let robot = instantiate_wsteth_eth_vault(&runner, &admin, None, None, None, &dependencies);
    let user = robot.new_user(&admin);

    // Update deposit fee to include vault as recipient
    let deposit_fee = Some(FeeConfig {
        fee_rate,
        fee_recipients: vec![(robot.vault_addr.clone(), Decimal::percent(100))],
    });
    robot.update_config(
        ConfigUpdates {
            deposit_fee,
            ..Default::default()
        },
        Unwrap::Ok,
        &admin,
    );

    // Deposit some funds and assert the vault token balance is correct
    let base_token_balance = robot.query_base_token_balance(user.address());
    let deposit_amount = base_token_balance / Uint128::new(2);
    let expected_amount =
        deposit_amount * (Decimal::one() - fee_rate) * INITIAL_VAULT_TOKENS_PER_BASE_TOKEN;
    let funds = coins(deposit_amount.u128(), robot.base_token());
    robot
        .assert_base_token_balance_eq(robot.vault_addr.clone(), Uint128::zero())
        .deposit_native(deposit_amount, None, Unwrap::Ok, &user, &funds)
        .assert_vault_token_balance_eq(user.address(), expected_amount)
        .assert_total_vault_token_supply_eq(expected_amount)
        .assert_total_vault_assets_eq(deposit_amount * (Decimal::one() - fee_rate))
        .assert_base_token_balance_eq(robot.vault_addr.clone(), deposit_amount * fee_rate); // Vault should have just the fee, as rest would be staked

    // Compound the vault, the fees in the vault should be staked
    robot
        .compound_vault(&admin)
        .assert_base_token_balance_eq(robot.vault_addr.clone(), Uint128::zero())
        .assert_total_vault_token_supply_eq(expected_amount)
        .assert_total_vault_assets_eq(deposit_amount);
}
