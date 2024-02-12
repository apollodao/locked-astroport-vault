use common::{default_instantiate, get_test_runner, DEPS_PATH};
use cosmwasm_std::Uint128;
use cw_it::helpers::Unwrap;
use cw_it::test_tube::Account;
use cw_it::traits::CwItRunner;
use cw_vault_standard_test_helpers::traits::force_unlock::ForceUnlockVaultRobot;
use cw_vault_standard_test_helpers::traits::lockup::LockedVaultRobot;
use cw_vault_standard_test_helpers::traits::CwVaultStandardRobot;
use locked_astroport_vault::helpers::INITIAL_VAULT_TOKENS_PER_BASE_TOKEN;
use locked_astroport_vault_test_helpers::robot::LockedAstroportVaultRobot;

pub mod common;

#[test]
fn update_force_withdraw_whitelist_works_correctly() {
    let owned_runner = get_test_runner();
    let runner = owned_runner.as_ref();
    let admin = LockedAstroportVaultRobot::new_admin(&runner);
    let dependencies = LockedAstroportVaultRobot::instantiate_deps(&runner, &admin, DEPS_PATH);
    let (robot, _base_pool, _treasury) = default_instantiate(&runner, &admin, &dependencies);

    let user = robot.new_user(&admin);

    robot
        .update_force_withdraw_whitelist(vec![user.address()], vec![], Unwrap::Ok, &admin)
        .assert_force_withdraw_whitelist_eq(&[&user.address()])
        .update_force_withdraw_whitelist(vec![], vec![user.address()], Unwrap::Ok, &admin)
        .assert_force_withdraw_whitelist_eq(&[])
        .update_force_withdraw_whitelist(
            vec![],
            vec![user.address()],
            Unwrap::Err("Address not in whitelist"),
            &admin,
        );
}

#[test]
fn cannot_add_and_remove_the_same_address_to_force_withdraw_whitelist() {
    let owned_runner = get_test_runner();
    let runner = owned_runner.as_ref();
    let admin = LockedAstroportVaultRobot::new_admin(&runner);
    let dependencies = LockedAstroportVaultRobot::instantiate_deps(&runner, &admin, DEPS_PATH);
    let (robot, _base_pool, _treasury) = default_instantiate(&runner, &admin, &dependencies);

    let user = robot.new_user(&admin);

    robot.update_force_withdraw_whitelist(
        vec![user.address()],
        vec![user.address()],
        Unwrap::Err("Cannot add and remove the same address"),
        &admin,
    );
}

#[test]
fn query_force_withdraw_whitelist_pagination_works() {
    let owned_runner = get_test_runner();
    let runner = owned_runner.as_ref();
    let admin = LockedAstroportVaultRobot::new_admin(&runner);
    let dependencies = LockedAstroportVaultRobot::instantiate_deps(&runner, &admin, DEPS_PATH);
    let (robot, _base_pool, _treasury) = default_instantiate(&runner, &admin, &dependencies);

    // Instantiate and whitelist 15 addresses
    let accs = runner.init_accounts(&[], 15).unwrap();
    let mut addrs: Vec<String> = accs.iter().map(|a| a.address()).collect();
    addrs.sort();
    robot.update_force_withdraw_whitelist(addrs.clone(), vec![], Unwrap::Ok, &admin);

    // Query with no pagination args
    let res = robot.query_force_withdraw_whitelist(None, None);
    assert_eq!(res.len(), 10); // Default limit of 10
    assert_eq!(res, addrs[0..10]);

    // Query starting after the first address
    let res = robot.query_force_withdraw_whitelist(Some(addrs[0].clone()), None);
    assert_eq!(res.len(), 10);
    assert_eq!(res, addrs[1..11]);

    // Query starting after the last address
    let res = robot.query_force_withdraw_whitelist(Some(addrs[14].clone()), None);
    assert_eq!(res.len(), 0);

    // Query with a limit of 5
    let res = robot.query_force_withdraw_whitelist(None, Some(5));
    assert_eq!(res.len(), 5);
    assert_eq!(res, addrs[0..5]);

    // Query with a limit of 5 and starting after the first address
    let res = robot.query_force_withdraw_whitelist(Some(addrs[0].clone()), Some(5));
    assert_eq!(res.len(), 5);
    assert_eq!(res, addrs[1..6]);

    // Query with limit of 15
    let res = robot.query_force_withdraw_whitelist(None, Some(15));
    assert_eq!(res.len(), 15);
    assert_eq!(res, addrs[0..15]);
}

#[test]
fn force_redeem_works() {
    let owned_runner = get_test_runner();
    let runner = owned_runner.as_ref();
    let admin = LockedAstroportVaultRobot::new_admin(&runner);
    let dependencies = LockedAstroportVaultRobot::instantiate_deps(&runner, &admin, DEPS_PATH);
    let (robot, _base_pool, _treasury) = default_instantiate(&runner, &admin, &dependencies);

    let user = robot.new_user(&admin);

    // Deposit from user, whitelist user for force redeem, then force redeem
    let balance_before_deposit = robot.query_base_token_balance(user.address());
    let deposit_amount = Uint128::new(100);
    robot
        .deposit_cw20(deposit_amount, None, Unwrap::Ok, &user)
        .assert_base_token_balance_eq(user.address(), balance_before_deposit - deposit_amount)
        .assert_vault_token_balance_eq(
            user.address(),
            deposit_amount * INITIAL_VAULT_TOKENS_PER_BASE_TOKEN,
        )
        .update_force_withdraw_whitelist(vec![user.address()], vec![], Unwrap::Ok, &admin)
        .force_redeem_all(None, Unwrap::Ok, &user)
        .assert_base_token_balance_eq(user.address(), balance_before_deposit)
        .assert_vault_token_balance_eq(user.address(), 0u128);
}

#[test]
fn force_redeem_to_recipient_works() {
    let owned_runner = get_test_runner();
    let runner = owned_runner.as_ref();
    let admin = LockedAstroportVaultRobot::new_admin(&runner);
    let dependencies = LockedAstroportVaultRobot::instantiate_deps(&runner, &admin, DEPS_PATH);
    let (robot, _base_pool, _treasury) = default_instantiate(&runner, &admin, &dependencies);

    let user = robot.new_user(&admin);
    let recipient = runner.init_account(&[]).unwrap();

    // Deposit from user, whitelist user for force redeem, then force redeem
    let balance_before_deposit = robot.query_base_token_balance(user.address());
    let deposit_amount = Uint128::new(100);
    robot
        .deposit_cw20(deposit_amount, None, Unwrap::Ok, &user)
        .assert_base_token_balance_eq(user.address(), balance_before_deposit - deposit_amount)
        .assert_vault_token_balance_eq(
            user.address(),
            deposit_amount * INITIAL_VAULT_TOKENS_PER_BASE_TOKEN,
        )
        .update_force_withdraw_whitelist(vec![user.address()], vec![], Unwrap::Ok, &admin)
        .force_redeem_all(Some(recipient.address()), Unwrap::Ok, &user)
        .assert_base_token_balance_eq(user.address(), balance_before_deposit - deposit_amount)
        .assert_vault_token_balance_eq(user.address(), 0u128)
        .assert_base_token_balance_eq(recipient.address(), deposit_amount)
        .assert_vault_token_balance_eq(recipient.address(), 0u128);
}

#[test]
fn force_withdraw_unlocking_works() {
    let owned_runner = get_test_runner();
    let runner = owned_runner.as_ref();
    let admin = LockedAstroportVaultRobot::new_admin(&runner);
    let dependencies = LockedAstroportVaultRobot::instantiate_deps(&runner, &admin, DEPS_PATH);
    let (robot, _base_pool, _treasury) = default_instantiate(&runner, &admin, &dependencies);

    let user = robot.new_user(&admin);

    // Deposit from user, begin unlocking, whitelist user, then call force withdraw
    // unlocking
    let balance_before_deposit = robot.query_base_token_balance(user.address());
    let deposit_amount = Uint128::new(100);
    robot
        .deposit_cw20(deposit_amount, None, Unwrap::Ok, &user)
        .assert_base_token_balance_eq(user.address(), balance_before_deposit - deposit_amount)
        .assert_vault_token_balance_eq(
            user.address(),
            deposit_amount * INITIAL_VAULT_TOKENS_PER_BASE_TOKEN,
        )
        .unlock_all(Unwrap::Ok, &user)
        .assert_base_token_balance_eq(user.address(), balance_before_deposit - deposit_amount)
        .assert_vault_token_balance_eq(user.address(), 0u128)
        .update_force_withdraw_whitelist(vec![user.address()], vec![], Unwrap::Ok, &admin)
        .force_withdraw_unlocking(0, None::<Uint128>, None, Unwrap::Ok, &user)
        .assert_base_token_balance_eq(user.address(), balance_before_deposit)
        .assert_vault_token_balance_eq(user.address(), 0u128)
        .force_withdraw_unlocking(0, None::<Uint128>, None, Unwrap::Err("not found"), &user);
}

#[test]
fn force_withdraw_unlocking_to_recipient_works() {
    let owned_runner = get_test_runner();
    let runner = owned_runner.as_ref();
    let admin = LockedAstroportVaultRobot::new_admin(&runner);
    let dependencies = LockedAstroportVaultRobot::instantiate_deps(&runner, &admin, DEPS_PATH);
    let (robot, _base_pool, _treasury) = default_instantiate(&runner, &admin, &dependencies);

    let user = robot.new_user(&admin);
    let recipient = runner.init_account(&[]).unwrap();

    // Deposit from user, begin unlocking, whitelist user, then call force withdraw
    // unlocking
    let balance_before_deposit = robot.query_base_token_balance(user.address());
    let deposit_amount = Uint128::new(100);
    robot
        .deposit_cw20(deposit_amount, None, Unwrap::Ok, &user)
        .assert_base_token_balance_eq(user.address(), balance_before_deposit - deposit_amount)
        .assert_vault_token_balance_eq(
            user.address(),
            deposit_amount * INITIAL_VAULT_TOKENS_PER_BASE_TOKEN,
        )
        .unlock_all(Unwrap::Ok, &user)
        .assert_base_token_balance_eq(user.address(), balance_before_deposit - deposit_amount)
        .assert_vault_token_balance_eq(user.address(), 0u128)
        .update_force_withdraw_whitelist(vec![user.address()], vec![], Unwrap::Ok, &admin)
        .force_withdraw_unlocking(
            0,
            None::<Uint128>,
            Some(recipient.address()),
            Unwrap::Ok,
            &user,
        )
        .assert_base_token_balance_eq(user.address(), balance_before_deposit - deposit_amount)
        .assert_vault_token_balance_eq(user.address(), 0u128)
        .assert_base_token_balance_eq(recipient.address(), deposit_amount)
        .assert_vault_token_balance_eq(recipient.address(), 0u128)
        .force_withdraw_unlocking(0, None::<Uint128>, None, Unwrap::Err("not found"), &user);
}

#[test]
fn force_withdraw_unlocking_with_partial_amount_works() {
    let owned_runner = get_test_runner();
    let runner = owned_runner.as_ref();
    let admin = LockedAstroportVaultRobot::new_admin(&runner);
    let dependencies = LockedAstroportVaultRobot::instantiate_deps(&runner, &admin, DEPS_PATH);
    let (robot, _base_pool, _treasury) = default_instantiate(&runner, &admin, &dependencies);

    let user = robot.new_user(&admin);

    // Deposit from user, begin unlocking, whitelist user, then call force withdraw
    // unlocking
    let balance_before_deposit = robot.query_base_token_balance(user.address());
    let deposit_amount = Uint128::new(100);
    robot
        .deposit_cw20(deposit_amount, None, Unwrap::Ok, &user)
        .assert_base_token_balance_eq(user.address(), balance_before_deposit - deposit_amount)
        .assert_vault_token_balance_eq(
            user.address(),
            deposit_amount * INITIAL_VAULT_TOKENS_PER_BASE_TOKEN,
        )
        .unlock_all(Unwrap::Ok, &user)
        .assert_base_token_balance_eq(user.address(), balance_before_deposit - deposit_amount)
        .assert_vault_token_balance_eq(user.address(), 0u128)
        .update_force_withdraw_whitelist(vec![user.address()], vec![], Unwrap::Ok, &admin)
        .force_withdraw_unlocking(
            0,
            Some(deposit_amount / Uint128::new(2)),
            None,
            Unwrap::Ok,
            &user,
        )
        .assert_base_token_balance_eq(
            user.address(),
            balance_before_deposit - deposit_amount / Uint128::new(2),
        )
        .assert_vault_token_balance_eq(user.address(), 0u128)
        .force_withdraw_unlocking(0, None::<Uint128>, None, Unwrap::Ok, &user)
        .assert_base_token_balance_eq(user.address(), balance_before_deposit)
        .assert_vault_token_balance_eq(user.address(), 0u128)
        .force_withdraw_unlocking(0, None::<Uint128>, None, Unwrap::Err("not found"), &user);
}

#[test]
fn cannot_force_withdraw_unlocking_more_than_position_amount() {
    let owned_runner = get_test_runner();
    let runner = owned_runner.as_ref();
    let admin = LockedAstroportVaultRobot::new_admin(&runner);
    let dependencies = LockedAstroportVaultRobot::instantiate_deps(&runner, &admin, DEPS_PATH);
    let (robot, _base_pool, _treasury) = default_instantiate(&runner, &admin, &dependencies);

    let user = robot.new_user(&admin);

    // Deposit from user, begin unlocking, whitelist user, then call force withdraw
    // unlocking
    let deposit_amount = Uint128::new(100);
    robot
        .deposit_cw20(deposit_amount, None, Unwrap::Ok, &user)
        .assert_vault_token_balance_eq(
            user.address(),
            deposit_amount * INITIAL_VAULT_TOKENS_PER_BASE_TOKEN,
        )
        .unlock_all(Unwrap::Ok, &user)
        .update_force_withdraw_whitelist(vec![user.address()], vec![], Unwrap::Ok, &admin)
        .force_withdraw_unlocking(
            0,
            Some(deposit_amount * INITIAL_VAULT_TOKENS_PER_BASE_TOKEN + Uint128::one()),
            None,
            Unwrap::Err("Claim amount is greater than the claimable amount"),
            &user,
        );
}
