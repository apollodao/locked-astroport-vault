use cosmwasm_std::{Decimal, Uint128};
use cw_it::test_tube::Account;
use cw_vault_standard_test_helpers::traits::CwVaultStandardRobot;
use locked_astroport_vault::helpers::INITIAL_VAULT_TOKENS_PER_BASE_TOKEN;
use locked_astroport_vault_test_helpers::robot::LockedAstroportVaultRobot;
use proptest::prelude::*;
use proptest::proptest;

use crate::common::compound::test_compound_vault;
use crate::common::{get_test_runner, instantiate_vault, VaultSetup, DEPS_PATH};

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
        let admin = LockedAstroportVaultRobot::new_admin(&runner);
        let dependencies = LockedAstroportVaultRobot::instantiate_deps(&runner, &admin, DEPS_PATH);
        let fee = Decimal::percent(fee);
        let (robot, _treasury) = instantiate_vault(&runner, &admin, setup, fee, &dependencies);

        let deposit_amount = Uint128::new(1_000_000u128);

        // Assert that initial vault token exchange rate is correct
        let vault_token_exchange_rate = robot.query_vault_token_exchange_rate(robot.base_token());
        assert_eq!(
            vault_token_exchange_rate,
            Decimal::from_ratio(1u128, INITIAL_VAULT_TOKENS_PER_BASE_TOKEN.u128())
        );

        let mut last_vault_token_amount_received = Uint128::MAX;
        for _ in 0..10 {
            let user = robot.new_user(&admin);

            test_compound_vault(&robot, deposit_amount, fee, &user, &admin);

            let vault_token_amount_received = robot.query_vault_token_balance(user.address());
            assert!(vault_token_amount_received < last_vault_token_amount_received);

            last_vault_token_amount_received = vault_token_amount_received;
        }
    }
}
