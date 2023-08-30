use apollo_cw_asset::AssetInfoUnchecked;
use common::{default_instantiate, get_test_runner, DEPS_PATH};
use cosmwasm_std::Decimal;
use cw_it::helpers::Unwrap;
use cw_utils::Duration;
use locked_astroport_vault::state::ConfigUpdates;
use locked_astroport_vault_test_helpers::robot::LockedAstroportVaultRobot;

pub mod common;

#[test]
fn config_validation_fails_if_reward_liquidation_target_not_in_pool_assets() {
    let owned_runner = get_test_runner();
    let runner = owned_runner.as_ref();
    let admin = LockedAstroportVaultRobot::new_admin(&runner);
    let dependencies = LockedAstroportVaultRobot::instantiate_deps(&runner, &admin, DEPS_PATH);
    let (robot, _base_pool, _treasury) = default_instantiate(&runner, &admin, &dependencies);

    let updates = ConfigUpdates {
        reward_liquidation_target: Some(AssetInfoUnchecked::native("random_token")),
        ..Default::default()
    };
    robot.update_config(updates, Unwrap::Err("is not in the pool assets"), &admin);
}

#[test]
fn config_validation_fails_if_lock_duration_is_in_blocks() {
    let owned_runner = get_test_runner();
    let runner = owned_runner.as_ref();
    let admin = LockedAstroportVaultRobot::new_admin(&runner);
    let dependencies = LockedAstroportVaultRobot::instantiate_deps(&runner, &admin, DEPS_PATH);
    let (robot, _base_pool, _treasury) = default_instantiate(&runner, &admin, &dependencies);

    let updates = ConfigUpdates {
        lock_duration: Some(Duration::Height(420)),
        ..Default::default()
    };
    robot.update_config(
        updates,
        Unwrap::Err("lock_duration must be specified in seconds"),
        &admin,
    );
}

#[test]
fn config_validation_fails_if_performance_fee_exceeds_100_percent() {
    let owned_runner = get_test_runner();
    let runner = owned_runner.as_ref();
    let admin = LockedAstroportVaultRobot::new_admin(&runner);
    let dependencies = LockedAstroportVaultRobot::instantiate_deps(&runner, &admin, DEPS_PATH);
    let (robot, _base_pool, _treasury) = default_instantiate(&runner, &admin, &dependencies);

    let updates = ConfigUpdates {
        performance_fee: Some(Decimal::percent(101)),
        ..Default::default()
    };
    robot.update_config(
        updates,
        Unwrap::Err("Performance fee can't be higher than 100%"),
        &admin,
    );
}

#[test]
fn config_validation_fails_if_route_not_found_in_cw_dex_router() {
    let owned_runner = get_test_runner();
    let runner = owned_runner.as_ref();
    let admin = LockedAstroportVaultRobot::new_admin(&runner);
    let dependencies = LockedAstroportVaultRobot::instantiate_deps(&runner, &admin, DEPS_PATH);
    let (robot, _base_pool, _treasury) = default_instantiate(&runner, &admin, &dependencies);

    let updates = ConfigUpdates {
        reward_tokens: Some(vec![AssetInfoUnchecked::native("random_token")]),
        ..Default::default()
    };
    robot.update_config(
        updates,
        Unwrap::Err("Could not read path in cw-dex-router for"),
        &admin,
    );
}
