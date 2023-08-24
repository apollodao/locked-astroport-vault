use std::str::FromStr;

use cosmwasm_std::{Coin, Decimal};
use cw_it::cw_multi_test::{StargateKeeper, StargateMessageHandler};
use cw_it::multi_test::modules::TokenFactory;
use cw_it::multi_test::MultiTestRunner;
use cw_it::test_tube::{Account, SigningAccount};
use cw_it::traits::CwItRunner;
use cw_it::TestRunner;
use locked_astroport_vault_test_helpers::robot::{
    LockedAstroportVaultRobot, LockedVaultDependencies,
};

pub const UNOPTIMIZED_PATH: &str = "../target/wasm32-unknown-unknown/release";
pub const DEPS_PATH: &str = "tests/test_artifacts";

pub const DENOM_CREATION_FEE: &str = "10000000uosmo";

const TOKEN_FACTORY: &TokenFactory =
    &TokenFactory::new("factory", 32, 16, 59 + 16, DENOM_CREATION_FEE);

/// An enum to represent the different default vault setups
pub enum VaultSetup {
    WstEth,
    AxlrNtrn,
}

pub fn get_test_runner<'a>() -> TestRunner<'a> {
    match option_env!("TEST_RUNNER").unwrap_or("multi-test") {
        "multi-test" => {
            let mut stargate_keeper = StargateKeeper::new();
            TOKEN_FACTORY.register_msgs(&mut stargate_keeper);

            TestRunner::MultiTest(MultiTestRunner::new_with_stargate("osmo", stargate_keeper))
        }
        #[cfg(feature = "osmosis-test-tube")]
        "osmosis-test-app" => {
            TestRunner::OsmosisTestApp(cw_it::osmosis_test_tube::OsmosisTestApp::new())
        }
        _ => panic!("Unsupported test runner type"),
    }
}

pub fn default_instantiate<'a>(
    runner: &'a TestRunner<'a>,
    admin: &SigningAccount,
    dependencies: &'a LockedVaultDependencies<'a>,
) -> (LockedAstroportVaultRobot<'a>, SigningAccount) {
    let vault_contract = LockedAstroportVaultRobot::contract(runner, UNOPTIMIZED_PATH);
    let treasury_addr = runner.init_account(&[]).unwrap();
    let token_factory_fee = Coin::from_str(DENOM_CREATION_FEE).unwrap();

    let robot = LockedAstroportVaultRobot::new_wsteth_eth_vault(
        runner,
        vault_contract,
        token_factory_fee,
        treasury_addr.address(),
        Decimal::percent(5),
        dependencies,
        admin,
    );

    (robot, treasury_addr)
}

pub fn instantiate_wsteth_eth_vault<'a>(
    runner: &'a TestRunner<'a>,
    admin: &SigningAccount,
    performance_fee: Decimal,
    dependencies: &'a LockedVaultDependencies<'a>,
) -> (LockedAstroportVaultRobot<'a>, SigningAccount) {
    let vault_contract = LockedAstroportVaultRobot::contract(runner, UNOPTIMIZED_PATH);
    let treasury_addr = runner.init_account(&[]).unwrap();
    let token_factory_fee = Coin::from_str(DENOM_CREATION_FEE).unwrap();

    let robot = LockedAstroportVaultRobot::new_wsteth_eth_vault(
        runner,
        vault_contract,
        token_factory_fee,
        treasury_addr.address(),
        performance_fee,
        dependencies,
        admin,
    );

    (robot, treasury_addr)
}

pub fn instantiate_axlr_ntrn_vault<'a>(
    runner: &'a TestRunner<'a>,
    admin: &SigningAccount,
    performance_fee: Decimal,
    dependencies: &'a LockedVaultDependencies<'a>,
) -> (LockedAstroportVaultRobot<'a>, SigningAccount) {
    let vault_contract = LockedAstroportVaultRobot::contract(runner, UNOPTIMIZED_PATH);
    let treasury_addr = runner.init_account(&[]).unwrap();
    let token_factory_fee = Coin::from_str(DENOM_CREATION_FEE).unwrap();

    let (robot, _axl_ntrn_pool, _astro_ntrn_pool) =
        LockedAstroportVaultRobot::new_unlocked_axlr_ntrn_vault(
            runner,
            vault_contract,
            token_factory_fee,
            treasury_addr.address(),
            performance_fee,
            dependencies,
            admin,
        );

    (robot, treasury_addr)
}

pub fn instantiate_vault<'a>(
    runner: &'a TestRunner<'a>,
    admin: &SigningAccount,
    setup: VaultSetup,
    performance_fee: Decimal,
    dependencies: &'a LockedVaultDependencies<'a>,
) -> (LockedAstroportVaultRobot<'a>, SigningAccount) {
    match setup {
        VaultSetup::WstEth => {
            instantiate_wsteth_eth_vault(runner, admin, performance_fee, dependencies)
        }
        VaultSetup::AxlrNtrn => {
            instantiate_axlr_ntrn_vault(runner, admin, performance_fee, dependencies)
        }
    }
}
