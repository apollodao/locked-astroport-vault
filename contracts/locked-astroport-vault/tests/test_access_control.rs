use cosmwasm_std::{Addr, Uint128};
use cw_it::helpers::Unwrap;
use cw_it::robot::TestRobot;
use cw_it::test_tube::Account;

use cw_vault_standard_test_helpers::traits::force_unlock::ForceUnlockVaultRobot;
use locked_astroport_vault_test_helpers::robot::LockedAstroportVaultRobot;

use common::{default_instantiate, get_test_runner, DEPS_PATH};
use locked_astroport_vault::msg::{ExecuteMsg, ExtensionExecuteMsg, InternalMsg};
use locked_astroport_vault::state::ConfigUpdates;
use strum::EnumCount;

pub mod common;

#[test]
fn update_ownership_can_only_be_called_by_admin() {
    let owned_runner = get_test_runner();
    let runner = owned_runner.as_ref();
    let admin = LockedAstroportVaultRobot::new_admin(&runner);
    let dependencies = LockedAstroportVaultRobot::instantiate_deps(&runner, &admin, DEPS_PATH);
    let (robot, _treasury) = default_instantiate(&runner, &admin, &dependencies);
    let user = robot.new_user(&admin);

    let action = cw_ownable::Action::TransferOwnership {
        new_owner: admin.address(),
        expiry: None,
    };

    // Try calling update_ownership as non-admin, should fail. Then try calling as
    // admin, should work.
    robot
        .update_ownership(
            action.clone(),
            Unwrap::Err("Caller is not the contract's current owner"),
            &user,
        )
        .update_ownership(action, Unwrap::Ok, &admin);
}

#[test]
fn update_config_can_only_be_called_by_admin() {
    let owned_runner = get_test_runner();
    let runner = owned_runner.as_ref();
    let admin = LockedAstroportVaultRobot::new_admin(&runner);
    let dependencies = LockedAstroportVaultRobot::instantiate_deps(&runner, &admin, DEPS_PATH);
    let (robot, _treasury) = default_instantiate(&runner, &admin, &dependencies);
    let user = robot.new_user(&admin);

    let config_updates = ConfigUpdates {
        treasury: Some(user.address()),
        ..Default::default()
    };

    // Try calling update_config as non-admin, should fail. Then try calling as
    // admin, should work.
    robot
        .update_config(
            config_updates.clone(),
            Unwrap::Err("Caller is not the contract's current owner"),
            &user,
        )
        .update_config(config_updates, Unwrap::Ok, &admin);
}

#[test]
#[should_panic(expected = "Caller is not the contract's current owner")]
fn update_force_withdraw_whitelist_can_only_be_called_by_admin() {
    let owned_runner = get_test_runner();
    let runner = owned_runner.as_ref();
    let admin = LockedAstroportVaultRobot::new_admin(&runner);
    let dependencies = LockedAstroportVaultRobot::instantiate_deps(&runner, &admin, DEPS_PATH);
    let (robot, _treasury) = default_instantiate(&runner, &admin, &dependencies);
    let user = robot.new_user(&admin);

    robot.update_force_withdraw_whitelist(vec![user.address()], vec![], Unwrap::Ok, &user);
}

#[test]
fn internal_msg_can_only_be_called_by_contract() {
    let owned_runner = get_test_runner();
    let runner = owned_runner.as_ref();
    let admin = LockedAstroportVaultRobot::new_admin(&runner);
    let dependencies = LockedAstroportVaultRobot::instantiate_deps(&runner, &admin, DEPS_PATH);
    let (robot, _treasury) = default_instantiate(&runner, &admin, &dependencies);
    let user = robot.new_user(&admin);

    let msgs: [InternalMsg; InternalMsg::COUNT] = [
        InternalMsg::StakeLps {},
        InternalMsg::ProvideLiquidity {},
        InternalMsg::SellTokens {},
        InternalMsg::Deposit {
            depositor: Addr::unchecked(user.address()),
            recipient: Addr::unchecked(user.address()),
            amount: Uint128::new(420),
        },
        InternalMsg::Redeem {
            recipient: Addr::unchecked(user.address()),
            amount: Uint128::new(420),
        },
    ];

    for msg in msgs {
        let err = robot
            .wasm()
            .execute(
                &robot.vault_addr,
                &ExecuteMsg::VaultExtension(ExtensionExecuteMsg::Internal(msg)),
                &[],
                &admin,
            )
            .unwrap_err();
        assert!(err.to_string().contains("Unauthorized"));
    }
}
