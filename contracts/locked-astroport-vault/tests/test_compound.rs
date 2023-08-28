use common::{instantiate_vault, VaultSetup, DEPS_PATH};
use cosmwasm_std::{Decimal, Uint128};
use cw_it::test_tube::Account;
use cw_vault_standard_test_helpers::traits::CwVaultStandardRobot;
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
