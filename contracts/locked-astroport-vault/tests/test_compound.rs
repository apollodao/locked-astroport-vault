use common::{instantiate_vault, VaultSetup, DEPS_PATH};
use cosmwasm_std::{Decimal, Uint128};
use cw_it::robot::TestRobot;
use cw_it::test_tube::{Account, SigningAccount};
use cw_vault_standard_test_helpers::traits::CwVaultStandardRobot;
use locked_astroport_vault_test_helpers::robot::LockedAstroportVaultRobot;
use test_case::test_case;

pub mod common;
pub use common::{get_test_runner, UNOPTIMIZED_PATH};

fn test_compound_vault(robot: &LockedAstroportVaultRobot, fee: Decimal, admin: &SigningAccount) {
    let user = robot.new_user(admin);

    // Deposit some funds and assert the vault token balance is correct
    let base_token_balance = robot.query_base_token_balance(user.address());
    let deposit_amount = base_token_balance / Uint128::new(2);
    robot
        .deposit_cw20(deposit_amount, None, &user)
        .assert_vault_token_balance_eq(user.address(), deposit_amount)
        .assert_vt_balance_converted_to_assets_eq(user.address(), deposit_amount);

    // Donate some reward tokens to the vault to simulate rewards accruing, then
    // compound and check that the base token amount corresponding to the users
    // vault token balance has increased.
    let config = robot.query_config();
    let treasury = config.treasury;
    let reward_tokens = config.reward_tokens;
    let mut base_token_balance_in_vault = deposit_amount;
    for token in reward_tokens {
        println!("Donating {}", token);
        let amount = Uint128::new(1_000_000);
        robot
            .send_native_tokens(admin, &robot.vault_addr, amount, token.to_string())
            .assert_vt_balance_converted_to_assets_eq(user.address(), base_token_balance_in_vault)
            .compound_vault(&user)
            .assert_native_token_balance_eq(treasury.to_string(), token.to_string(), amount * fee);

        // If fee is less than 100% then the users base token balance in the vault
        // should have increased
        if fee < Decimal::percent(100) {
            robot.assert_vt_balance_converted_to_assets_gt(
                user.address(),
                base_token_balance_in_vault,
            );
        }

        base_token_balance_in_vault = robot.query_convert_to_assets(deposit_amount);
    }
}

#[test_case(VaultSetup::WstEth, Decimal::percent(5); "Compound wsteth_eth vault, 5% fee.")]
#[test_case(VaultSetup::AxlrNtrn, Decimal::percent(5); "Compound axlr_ntrn vault, 5% fee.")]
#[test_case(VaultSetup::WstEth, Decimal::zero(); "Compound wsteth_eth vault, 0% fee.")]
#[test_case(VaultSetup::AxlrNtrn, Decimal::zero(); "Compound axlr_ntrn vault, 0% fee.")]
#[test_case(VaultSetup::WstEth, Decimal::percent(100); "Compound wsteth_eth vault, 100% fee.")]
#[test_case(VaultSetup::AxlrNtrn, Decimal::percent(100); "Compound axlr_ntrn vault, 100% fee.")]
fn compound_vault(setup: VaultSetup, fee: Decimal) {
    let runner = get_test_runner();
    let admin = LockedAstroportVaultRobot::new_admin(&runner);
    let dependencies = LockedAstroportVaultRobot::instantiate_deps(&runner, &admin, DEPS_PATH);
    let (robot, _treasury) = instantiate_vault(&runner, &admin, setup, fee, &dependencies);

    test_compound_vault(&robot, fee, &admin)
}
