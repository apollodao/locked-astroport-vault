use std::str::FromStr;

use common::instantiate_unlocked_vault;
use cosmwasm_std::{Coins, Uint128};
use cw_it::test_tube::Account;
use cw_it::traits::CwItRunner;
use cw_vault_standard_test_helpers::traits::CwVaultStandardRobot;
use locked_astroport_vault_test_helpers::helpers::Unwrap;
use locked_astroport_vault_test_helpers::robot::{LockedAstroportVaultRobot, DEFAULT_COINS};

pub mod common;
pub use common::{get_test_runner, UNOPTIMIZED_PATH};

use crate::common::default_instantiate;

#[test]
fn test_redeem_with_lockup() {
    let runner = get_test_runner();
    let admin = runner
        .init_account(&Coins::from_str(DEFAULT_COINS).unwrap().to_vec())
        .unwrap();
    let dependencies = LockedAstroportVaultRobot::instantiate_deps(&runner, &admin, None);
    let (robot, _treasury) = default_instantiate(&runner, &admin, &dependencies);
    let user = robot.new_user(&admin);

    // Try redeeming without first depositing, should fail
    robot.redeem(
        Uint128::one(),
        None,
        Unwrap::Err("Cannot Sub with 0 and 1"),
        &user,
    );

    // Deposit some funds, assert that vt and bt balances are correct, redeem, and
    // assert again. After redeeming the base_token_balance should not be the
    // same as before depositing, since we have a lockup on this vault and
    // instead a claim has been created.
    let base_token_balance = robot.query_base_token_balance(user.address());
    let deposit_amount = Uint128::new(100);
    robot
        .deposit_cw20(deposit_amount, None, &user)
        .assert_vault_token_balance_eq(user.address(), deposit_amount)
        .assert_base_token_balance_eq(user.address(), base_token_balance - deposit_amount)
        .redeem(deposit_amount, None, Unwrap::Ok, &user)
        .assert_base_token_balance_eq(user.address(), base_token_balance - deposit_amount)
        .assert_vault_token_balance_eq(user.address(), Uint128::zero());

    // Query the vault to ensure that the claim/unlocking position was created
    // correctly
    let unlocking_positions = robot.query_unlocking_positions(&user.address());
    assert_eq!(unlocking_positions.len(), 1);
    assert_eq!(unlocking_positions[0].owner, user.address());
    assert_eq!(unlocking_positions[0].base_token_amount, deposit_amount);
    assert_eq!(unlocking_positions[0].id, 0u64);
}

#[test]
fn test_redeem_without_lockup() {
    let runner = get_test_runner();
    let admin = runner
        .init_account(&Coins::from_str(DEFAULT_COINS).unwrap().to_vec())
        .unwrap();
    let dependencies = LockedAstroportVaultRobot::instantiate_deps(&runner, &admin, None);
    let (robot, _treasury) = instantiate_unlocked_vault(&runner, &admin, &dependencies);
    let user = robot.new_user(&admin);

    // Try redeeming without first depositing, should fail
    robot.redeem(
        Uint128::one(),
        None,
        Unwrap::Err("Cannot Sub with 0 and 1"),
        &user,
    );

    // Deposit some funds, assert that vt and bt balances are correct, redeem, and
    // assert again. After redeeming the base_token_balance should be the same
    // as before depositing, since we have no lockup on this vault.
    let base_token_balance = robot.query_base_token_balance(user.address());
    let deposit_amount = Uint128::new(100);
    robot
        .deposit_cw20(deposit_amount, None, &user)
        .assert_vault_token_balance_eq(user.address(), deposit_amount)
        .assert_base_token_balance_eq(user.address(), base_token_balance - deposit_amount)
        .redeem(deposit_amount, None, Unwrap::Ok, &user)
        .assert_base_token_balance_eq(user.address(), base_token_balance)
        .assert_vault_token_balance_eq(user.address(), Uint128::zero());
}
