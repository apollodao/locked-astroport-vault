use apollo_cw_asset::AssetInfoUnchecked;
use common::{default_instantiate, get_test_runner, DEPS_PATH};
use locked_astroport_vault::state::ConfigUpdates;
use locked_astroport_vault_test_helpers::helpers::Unwrap;
use locked_astroport_vault_test_helpers::robot::LockedAstroportVaultRobot;

pub mod common;

#[test]
fn config_cannot_be_validated_if_reward_liquidation_target_not_in_pool_assets() {
    let runner = get_test_runner();
    let admin = LockedAstroportVaultRobot::new_admin(&runner);
    let dependencies = LockedAstroportVaultRobot::instantiate_deps(&runner, &admin, DEPS_PATH);
    let (robot, _treasury) = default_instantiate(&runner, &admin, &dependencies);

    let updates = ConfigUpdates {
        reward_liquidation_target: Some(AssetInfoUnchecked::native("random_token")),
        ..Default::default()
    };
    robot.update_config(updates, Unwrap::Err("is not in the pool assets"), &admin);
}
