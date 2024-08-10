use cosmwasm_std::coins;
use cosmwasm_std::{Decimal, Uint128};
use cw_it::helpers::Unwrap;
use cw_it::robot::TestRobot;
use cw_it::test_tube::{Account, SigningAccount};
use cw_vault_standard_test_helpers::traits::CwVaultStandardRobot;
use locked_astroport_vault_test_helpers::robot::LockedAstroportVaultRobot;

/// Desposits `deposit_amount` base tokens into the vault, donates some of each
/// reward token to the vault, then compounds the vault and asserts that the
/// base token balance corresponding to the users vault token balance has
/// increased.
pub fn test_compound_vault(
    robot: &LockedAstroportVaultRobot,
    deposit_amount: Uint128,
    fee: Decimal,
    user: &SigningAccount,
    admin: &SigningAccount,
) {
    // Check vault token balance and this amount converted to base tokens before
    // deposit
    let vt_balance_before_deposit = robot.query_vault_token_balance(user.address());
    let bt_balance_in_vault_before_deposit =
        robot.query_convert_to_assets(vt_balance_before_deposit);

    let funds = coins(deposit_amount.u128(), robot.base_token());

    // Deposit some funds and assert the vault token balance is correct
    let vt_balance_after_deposit = robot
        .deposit_native(deposit_amount, None, Unwrap::Ok, user, &funds)
        .assert_vt_balance_converted_to_assets_eq(
            user.address(),
            bt_balance_in_vault_before_deposit + deposit_amount,
        )
        .query_vault_token_balance(user.address());
    // Donate some reward tokens to the vault to simulate rewards accruing, then
    // compound and check that the base token amount corresponding to the users
    // vault token balance has increased.
    let config = robot.query_config();
    let treasury = &config.performance_fee.fee_recipients[0].0;
    let reward_tokens = config.reward_tokens;
    let mut base_token_balance_in_vault = bt_balance_in_vault_before_deposit + deposit_amount;
    for token in reward_tokens {
        println!("Donating {}", token);
        let donation_amount = Uint128::new(1_000_000);

        let amount_in_treasury_before =
            robot.query_native_token_balance(treasury.to_string(), token.to_string());

        robot
            .send_native_tokens(admin, &robot.vault_addr, donation_amount, token.to_string())
            .assert_vt_balance_converted_to_assets_eq(user.address(), base_token_balance_in_vault)
            .compound_vault(user)
            .assert_native_token_balance_eq(
                treasury.to_string(),
                token.to_string(),
                amount_in_treasury_before + donation_amount * fee,
            );

        // If fee is less than 100% then the users base token balance in the vault
        // should have increased
        if fee < Decimal::percent(100) {
            robot.assert_vt_balance_converted_to_assets_gt(
                user.address(),
                base_token_balance_in_vault,
            );
        }

        base_token_balance_in_vault = robot.query_convert_to_assets(vt_balance_after_deposit);
    }
}
