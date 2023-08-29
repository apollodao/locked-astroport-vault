use common::{default_instantiate, instantiate_vault, VaultSetup, DEPS_PATH};
use cosmwasm_std::{Decimal, Uint128};
use cw_it::helpers::Unwrap;
use cw_it::robot::TestRobot;
use cw_it::test_tube::Account;
use cw_vault_standard_test_helpers::traits::CwVaultStandardRobot;
use locked_astroport_vault::helpers::INITIAL_VAULT_TOKENS_PER_BASE_TOKEN;
use locked_astroport_vault_test_helpers::robot::LockedAstroportVaultRobot;
use test_case::test_case;

pub mod common;
use common::compound::test_compound_vault;
use common::get_test_runner;

#[test_case(VaultSetup::WstEth, Decimal::percent(5); "Compound wsteth_eth vault, 5% fee.")]
#[test_case(VaultSetup::AxlrNtrn, Decimal::percent(5); "Compound axlr_ntrn vault, 5% fee.")]
#[test_case(VaultSetup::WstEth, Decimal::zero(); "Compound wsteth_eth vault, 0% fee.")]
#[test_case(VaultSetup::AxlrNtrn, Decimal::zero(); "Compound axlr_ntrn vault, 0% fee.")]
#[test_case(VaultSetup::WstEth, Decimal::percent(100); "Compound wsteth_eth vault, 100% fee.")]
#[test_case(VaultSetup::AxlrNtrn, Decimal::percent(100); "Compound axlr_ntrn vault, 100% fee.")]
fn compound_vault(setup: VaultSetup, fee: Decimal) {
    let owned_runner = get_test_runner();
    let runner = owned_runner.as_ref();
    let admin = LockedAstroportVaultRobot::new_admin(&runner);
    let dependencies = LockedAstroportVaultRobot::instantiate_deps(&runner, &admin, DEPS_PATH);
    let (robot, _treasury) = instantiate_vault(&runner, &admin, setup, fee, &dependencies);
    let user = robot.new_user(&admin);

    let base_token_balance = robot.query_base_token_balance(user.address());
    let deposit_amount = base_token_balance / Uint128::new(2);

    test_compound_vault(&robot, deposit_amount, fee, &user, &admin)
}

#[test]
fn rewards_are_compounded_when_in_vault_before_first_deposit() {
    let owned_runner = get_test_runner();
    let runner = owned_runner.as_ref();
    let admin = LockedAstroportVaultRobot::new_admin(&runner);
    let dependencies = LockedAstroportVaultRobot::instantiate_deps(&runner, &admin, DEPS_PATH);
    let (robot, _treasury) = default_instantiate(&runner, &admin, &dependencies);
    let user = robot.new_user(&admin);

    // Donate some reward tokens to the vault
    let config = robot.query_config();
    let reward_token = &config.reward_tokens[0];
    let donation_amount = 1000000u128;
    robot.send_native_tokens(
        &admin,
        &robot.vault_addr,
        donation_amount,
        reward_token.to_string(),
    );

    let deposit_amount = Uint128::new(1_000u128);

    // User's base token balance in vault should be greater than their deposit due
    // to compounding. User's vault token balance should be greated than their
    // deposit times the multiplier, due to the extra tokens from compounding.
    robot
        .deposit_cw20(deposit_amount, None, Unwrap::Ok, &user)
        .assert_native_token_balance_gt(
            user.address(),
            robot.vault_token(),
            deposit_amount * INITIAL_VAULT_TOKENS_PER_BASE_TOKEN,
        )
        .assert_vt_balance_converted_to_assets_gt(user.address(), deposit_amount);
}
