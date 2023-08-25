use common::{default_instantiate, get_test_runner, DEPS_PATH};
use cw_it::test_tube::Account;
use cw_vault_standard_test_helpers::traits::force_unlock::ForceUnlockVaultRobot;
use locked_astroport_vault_test_helpers::robot::LockedAstroportVaultRobot;

pub mod common;

#[test]
#[should_panic(expected = "Cannot add and remove the same address")]
fn cannot_add_and_remove_the_same_address_to_force_withdraw_whitelist() {
    let runner = get_test_runner();
    let admin = LockedAstroportVaultRobot::new_admin(&runner);
    let dependencies = LockedAstroportVaultRobot::instantiate_deps(&runner, &admin, DEPS_PATH);
    let (robot, _treasury) = default_instantiate(&runner, &admin, &dependencies);

    let user = robot.new_user(&admin);

    robot.update_force_withdraw_whitelist(&admin, vec![user.address()], vec![user.address()]);
}
