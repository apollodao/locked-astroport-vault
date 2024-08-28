use std::str::FromStr;

use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Coin, Decimal};
use cw_dex_astroport::AstroportPool;
use cw_it::cw_multi_test::{StargateKeeper, StargateMessageHandler};
use cw_it::multi_test::modules::TokenFactory;
use cw_it::multi_test::MultiTestRunner;
use cw_it::test_tube::{Account, SigningAccount};
use cw_it::traits::CwItRunner;
use cw_it::{OwnedTestRunner, TestRunner};
use locked_astroport_vault::state::FeeConfig;
use locked_astroport_vault_test_helpers::robot::{
    LockedAstroportVaultRobot, LockedVaultDependencies, DENOM_CREATION_FEE,
};

#[cfg(feature = "osmosis-test-tube")]
use cw_it::osmosis_test_tube::OsmosisTestApp;

pub mod compound;

pub const UNOPTIMIZED_PATH: &str = "../../target/wasm32-unknown-unknown/release";
pub const DEPS_PATH: &str = "tests/test_artifacts";

const TOKEN_FACTORY: &TokenFactory =
    &TokenFactory::new("factory", 32, 16, 59 + 16, DENOM_CREATION_FEE);

/// An enum to represent the different default vault setups
#[cw_serde]
pub enum VaultSetup {
    WstEth,
    AxlrNtrn,
}

pub fn get_test_runner<'a>() -> OwnedTestRunner<'a> {
    let test_runner = match option_env!("TEST_RUNNER").unwrap_or("multi-test") {
        "multi-test" => {
            let mut stargate_keeper = StargateKeeper::new();
            TOKEN_FACTORY.register_msgs(&mut stargate_keeper);

            OwnedTestRunner::MultiTest(MultiTestRunner::new_with_stargate("osmo", stargate_keeper))
        }
        #[cfg(feature = "osmosis-test-tube")]
        "osmosis-test-app" => OwnedTestRunner::OsmosisTestApp(OsmosisTestApp::new()),
        _ => panic!("Unsupported test runner type"),
    };
    println!("Using test runner: {:?}", test_runner.to_string());
    test_runner
}

pub fn default_instantiate<'a>(
    runner: &'a TestRunner<'a>,
    admin: &SigningAccount,
    dependencies: &'a LockedVaultDependencies<'a>,
) -> (LockedAstroportVaultRobot<'a>, AstroportPool, SigningAccount) {
    let vault_contract = LockedAstroportVaultRobot::contract(runner, UNOPTIMIZED_PATH);
    let treasury_addr = runner.init_account(&[]).unwrap();
    let token_factory_fee = Coin::from_str(DENOM_CREATION_FEE).unwrap();

    let (robot, wsteth_eth_pool) = LockedAstroportVaultRobot::new_wsteth_eth_vault(
        runner,
        vault_contract,
        token_factory_fee,
        Some(FeeConfig {
            fee_rate: Decimal::percent(5),
            fee_recipients: vec![(treasury_addr.address(), Decimal::percent(100))],
        }),
        None,
        None,
        dependencies,
        admin,
    );

    (robot, wsteth_eth_pool, treasury_addr)
}

pub fn instantiate_wsteth_eth_vault<'a>(
    runner: &'a TestRunner<'a>,
    admin: &SigningAccount,
    performance_fee: Option<FeeConfig<String>>,
    deposit_fee: Option<FeeConfig<String>>,
    withdrawal_fee: Option<FeeConfig<String>>,
    dependencies: &'a LockedVaultDependencies<'a>,
) -> LockedAstroportVaultRobot<'a> {
    let vault_contract = LockedAstroportVaultRobot::contract(runner, UNOPTIMIZED_PATH);
    let token_factory_fee = Coin::from_str(DENOM_CREATION_FEE).unwrap();

    let (robot, _) = LockedAstroportVaultRobot::new_wsteth_eth_vault(
        runner,
        vault_contract,
        token_factory_fee,
        performance_fee,
        deposit_fee,
        withdrawal_fee,
        dependencies,
        admin,
    );

    robot
}

pub fn instantiate_axlr_ntrn_vault<'a>(
    runner: &'a TestRunner<'a>,
    admin: &SigningAccount,
    performance_fee: Option<FeeConfig<String>>,
    deposit_fee: Option<FeeConfig<String>>,
    withdrawal_fee: Option<FeeConfig<String>>,
    dependencies: &'a LockedVaultDependencies<'a>,
) -> LockedAstroportVaultRobot<'a> {
    let vault_contract = LockedAstroportVaultRobot::contract(runner, UNOPTIMIZED_PATH);
    let token_factory_fee = Coin::from_str(DENOM_CREATION_FEE).unwrap();

    let (robot, _axl_ntrn_pool, _astro_ntrn_pool) =
        LockedAstroportVaultRobot::new_unlocked_axlr_ntrn_vault(
            runner,
            vault_contract,
            token_factory_fee,
            performance_fee,
            deposit_fee,
            withdrawal_fee,
            dependencies,
            admin,
        );

    robot
}

pub fn instantiate_vault<'a>(
    runner: &'a TestRunner<'a>,
    admin: &SigningAccount,
    setup: VaultSetup,
    performance_fee: Decimal,
    dependencies: &'a LockedVaultDependencies<'a>,
) -> (LockedAstroportVaultRobot<'a>, SigningAccount) {
    let treasury_addr = runner.init_account(&[]).unwrap();
    let performance_fee = Some(FeeConfig {
        fee_rate: performance_fee,
        fee_recipients: vec![(treasury_addr.address(), Decimal::percent(100))],
    });

    let robot = match setup {
        VaultSetup::WstEth => {
            instantiate_wsteth_eth_vault(runner, admin, performance_fee, None, None, dependencies)
        }
        VaultSetup::AxlrNtrn => {
            instantiate_axlr_ntrn_vault(runner, admin, performance_fee, None, None, dependencies)
        }
    };

    (robot, treasury_addr)
}
