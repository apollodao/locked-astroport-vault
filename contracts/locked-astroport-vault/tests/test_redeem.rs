use std::str::FromStr;

use common::instantiate_axlr_ntrn_vault;
use cosmwasm_std::{Coins, Decimal, Uint128};
use cw_it::helpers::Unwrap;
use cw_it::test_tube::Account;
use cw_it::traits::CwItRunner;
use cw_vault_standard_test_helpers::traits::CwVaultStandardRobot;
use locked_astroport_vault::helpers::INITIAL_VAULT_TOKENS_PER_BASE_TOKEN;
use locked_astroport_vault_test_helpers::robot::{LockedAstroportVaultRobot, DEFAULT_COINS};

pub mod common;
pub use common::get_test_runner;

use crate::common::{default_instantiate, DEPS_PATH};

#[test]
fn test_redeem_with_lockup() {
    let owned_runner = get_test_runner();
    let runner = owned_runner.as_ref();
    let admin = runner
        .init_account(&Coins::from_str(DEFAULT_COINS).unwrap().to_vec())
        .unwrap();
    let dependencies = LockedAstroportVaultRobot::instantiate_deps(&runner, &admin, DEPS_PATH);
    let (robot, _treasury) = default_instantiate(&runner, &admin, &dependencies);
    let user = robot.new_user(&admin);

    // Try redeeming without first depositing, should fail
    robot.redeem(
        Uint128::one(),
        None,
        Unwrap::Err("Unexpected funds sent"),
        Some(vec![]),
        &user,
    );

    // Deposit some funds, assert that vt and bt balances are correct, redeem, and
    // assert again. After redeeming the base_token_balance should not be the
    // same as before depositing, since we have a lockup on this vault and
    // instead a claim has been created.
    let base_token_balance = robot.query_base_token_balance(user.address());
    let deposit_amount = Uint128::new(100);
    robot
        .deposit_cw20(deposit_amount, None, Unwrap::Ok, &user)
        .assert_vault_token_balance_eq(
            user.address(),
            deposit_amount * INITIAL_VAULT_TOKENS_PER_BASE_TOKEN,
        )
        .assert_base_token_balance_eq(user.address(), base_token_balance - deposit_amount)
        .redeem(
            deposit_amount * INITIAL_VAULT_TOKENS_PER_BASE_TOKEN,
            None,
            Unwrap::Ok,
            None,
            &user,
        )
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
    let owned_runner = get_test_runner();
    let runner = owned_runner.as_ref();
    let admin = runner
        .init_account(&Coins::from_str(DEFAULT_COINS).unwrap().to_vec())
        .unwrap();
    let dependencies = LockedAstroportVaultRobot::instantiate_deps(&runner, &admin, DEPS_PATH);
    let (robot, _treasury) =
        instantiate_axlr_ntrn_vault(&runner, &admin, Decimal::percent(5), &dependencies);
    let user = robot.new_user(&admin);

    // Try redeeming without first depositing, should fail
    robot.redeem(
        Uint128::one(),
        None,
        Unwrap::Err("Unexpected funds sent"),
        Some(vec![]),
        &user,
    );

    // Deposit some funds, assert that vt and bt balances are correct, redeem, and
    // assert again. After redeeming the base_token_balance should be the same
    // as before depositing, since we have no lockup on this vault.
    let base_token_balance = robot.query_base_token_balance(user.address());
    let deposit_amount = Uint128::new(100);
    robot
        .deposit_cw20(deposit_amount, None, Unwrap::Ok, &user)
        .assert_vault_token_balance_eq(
            user.address(),
            deposit_amount * INITIAL_VAULT_TOKENS_PER_BASE_TOKEN,
        )
        .assert_base_token_balance_eq(user.address(), base_token_balance - deposit_amount)
        .redeem(
            deposit_amount * INITIAL_VAULT_TOKENS_PER_BASE_TOKEN,
            None,
            Unwrap::Ok,
            None,
            &user,
        )
        .assert_base_token_balance_eq(user.address(), base_token_balance)
        .assert_vault_token_balance_eq(user.address(), Uint128::zero());
}
