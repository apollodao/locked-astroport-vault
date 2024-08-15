use crate::common::compound::test_compound_vault;
use crate::common::{get_test_runner, instantiate_vault, VaultSetup, DEPS_PATH};
use cosmwasm_std::{Decimal, Uint128};
use cw_it::test_tube::Account;
use cw_vault_standard::VaultInfoResponse;
use cw_vault_standard_test_helpers::traits::CwVaultStandardRobot;
use locked_astroport_vault_test_helpers::robot::LockedAstroportVaultRobot;
use proptest::prelude::*;
use proptest::proptest;

pub mod common;

fn vault_setup() -> impl Strategy<Value = VaultSetup> {
    prop_oneof![Just(VaultSetup::WstEth), Just(VaultSetup::AxlrNtrn)]
}

proptest! {
    #![proptest_config(ProptestConfig {
        cases: 16,
        max_local_rejects: 100000,
        max_global_rejects: 100000,
        max_shrink_iters: 512,
        ..ProptestConfig::default()
    })]


    /// Tests the property that when equal amounts of base tokens are deposited between compounds, smaller
    /// amounts of vault tokens are minted.
    #[test]
    fn minted_vault_token_amount_decreases_as_rewards_compound(setup in vault_setup(), fee in 0..99u64) {
        let owned_runner = get_test_runner();
        let runner = owned_runner.as_ref();
        println!("before new_admin");
        let admin = LockedAstroportVaultRobot::new_admin(&runner);
        println!("after new_admin");
        let dependencies = LockedAstroportVaultRobot::instantiate_deps(&runner, &admin, DEPS_PATH);
        println!("after deps");
        let fee = Decimal::percent(fee);
        let (robot, _treasury) = instantiate_vault(&runner, &admin, setup, fee, &dependencies);
        println!("after instantiate_vault");
        let contract_info = robot.query_vault_info();
        println!("contract_info: {:?}", contract_info);
        let deposit_amount = Uint128::new(1_000_000u128);
        let query_vault_standard_info = robot.query_vault_standard_info();
        println!("query_vault_standard_info: {:?}", query_vault_standard_info);

        let mut last_vault_token_amount_received = Uint128::MAX;
        for _ in 0..10 {
            let user = robot.new_user(&admin);
            println!("before test_compound_vault");
            let user_base_token_balance = robot.query_base_token_balance(user.address());
            println!("user_base_token_balance : {:?}", user_base_token_balance);

            test_compound_vault(&robot, deposit_amount, fee, &user, &admin);
            println!("after test_compound_vault");
            let vault_token_amount_received = robot.query_vault_token_balance(user.address());
            assert!(vault_token_amount_received < last_vault_token_amount_received);

            last_vault_token_amount_received = vault_token_amount_received;
        }
    }
}
