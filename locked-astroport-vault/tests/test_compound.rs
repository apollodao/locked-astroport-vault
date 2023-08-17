use std::str::FromStr;

use cosmwasm_std::{Coins, Uint128};
use cw_it::robot::TestRobot;
use cw_it::test_tube::Account;
use cw_it::traits::CwItRunner;
use cw_vault_standard_test_helpers::traits::CwVaultStandardRobot;
use locked_astroport_vault_test_helpers::robot::{LockedAstroportVaultRobot, DEFAULT_COINS};

pub mod common;
pub use common::{get_test_runner, UNOPTIMIZED_PATH};

use crate::common::default_instantiate;

#[test]
fn test_compound_vault() {
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
        .assert_vault_token_balance_eq(user.address(), deposit_amount)
        .assert_vt_balance_converted_to_assets_eq(user.address(), deposit_amount);

    // Donate some reward tokens to the vault to simulate rewards accruing, then
    // compound and check that the base token amount corresponding to the users
    // vault token balance has increased.
    let config = robot.query_config();
    let reward_tokens = config.reward_tokens;
    let mut base_token_balance_in_vault = deposit_amount;
    for token in reward_tokens {
        println!("Donating {}", token);
        let amount = Uint128::new(1_000_000);
        base_token_balance_in_vault = robot
            .send_native_tokens(&admin, &robot.vault_addr, amount, token.to_string())
            .assert_vt_balance_converted_to_assets_eq(user.address(), base_token_balance_in_vault)
            .compound_vault(&user)
            .assert_vt_balance_converted_to_assets_gt(user.address(), base_token_balance_in_vault)
            .query_convert_to_assets(deposit_amount);
    }
}
