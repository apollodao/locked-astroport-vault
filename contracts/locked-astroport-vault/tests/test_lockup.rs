use common::{default_instantiate, get_test_runner, DEPS_PATH};
use cosmwasm_std::{attr, coin, Uint128};
use cw_it::helpers::Unwrap;
use cw_it::robot::TestRobot;
use cw_it::test_tube::Account;
use cw_vault_standard::extensions::lockup::{
    UNLOCKING_POSITION_ATTR_KEY, UNLOCKING_POSITION_CREATED_EVENT_TYPE,
};
use cw_vault_standard_test_helpers::traits::force_unlock::ForceUnlockVaultRobot;
use cw_vault_standard_test_helpers::traits::CwVaultStandardRobot;
use locked_astroport_vault::msg::ExecuteMsg;
use locked_astroport_vault_test_helpers::robot::LockedAstroportVaultRobot;

pub mod common;

#[test]
#[should_panic(expected = "Cannot add and remove the same address")]
fn cannot_add_and_remove_the_same_address_to_force_withdraw_whitelist() {
    let owned_runner = get_test_runner();
    let runner = owned_runner.as_ref();
    let admin = LockedAstroportVaultRobot::new_admin(&runner);
    let dependencies = LockedAstroportVaultRobot::instantiate_deps(&runner, &admin, DEPS_PATH);
    let (robot, _treasury) = default_instantiate(&runner, &admin, &dependencies);

    let user = robot.new_user(&admin);

    robot.update_force_withdraw_whitelist(
        vec![user.address()],
        vec![user.address()],
        Unwrap::Ok,
        &admin,
    );
}

#[test]
fn unlocking_position_event_emitted_when_vault_has_lockup() {
    let owned_runner = get_test_runner();
    let runner = owned_runner.as_ref();
    let admin = LockedAstroportVaultRobot::new_admin(&runner);
    let dependencies = LockedAstroportVaultRobot::instantiate_deps(&runner, &admin, DEPS_PATH);
    let (robot, _treasury) = default_instantiate(&runner, &admin, &dependencies);
    let user = robot.new_user(&admin);

    let deposit_amount = Uint128::new(100);
    let vault_token_balance = robot
        .deposit_cw20(deposit_amount, None, Unwrap::Ok, &user)
        .query_vault_token_balance(user.address());

    let res = robot
        .wasm()
        .execute(
            &robot.vault_addr(),
            &ExecuteMsg::Redeem {
                amount: vault_token_balance,
                recipient: None,
            },
            &[coin(vault_token_balance.u128(), robot.vault_token())],
            &user,
        )
        .unwrap();
    for event in res.events.iter() {
        println!("{:?}", event);
    }
    res.events
        .iter()
        .find(|e| {
            e.ty == format!("wasm-{}", UNLOCKING_POSITION_CREATED_EVENT_TYPE)
                && e.attributes
                    .contains(&attr(UNLOCKING_POSITION_ATTR_KEY, "0"))
        })
        .unwrap();
}
