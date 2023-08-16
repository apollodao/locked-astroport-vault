use std::str::FromStr;

use cosmwasm_std::Coin;
use cw_it::{
    cw_multi_test::{StargateKeeper, StargateMessageHandler},
    multi_test::{modules::TokenFactory, MultiTestRunner},
    osmosis_std::types::osmosis::tokenfactory::v1beta1::MsgCreateDenom,
    test_tube::{Account, SigningAccount},
    traits::CwItRunner,
    TestRunner,
};
use locked_astroport_vault_test_helpers::robot::{
    LockedAstroportVaultRobot, LockedVaultDependencies,
};

pub const UNOPTIMIZED_PATH: &str = "../target/wasm32-unknown-unknown/release";

pub const DENOM_CREATION_FEE: &str = "10000000uosmo";

const TOKEN_FACTORY: &TokenFactory =
    &TokenFactory::new("factory", 32, 16, 59 + 16, DENOM_CREATION_FEE);

pub fn get_test_runner<'a>() -> TestRunner<'a> {
    match option_env!("TEST_RUNNER_TYPE").unwrap_or("multi-test") {
        "multi-test" => {
            let type_url = MsgCreateDenom::TYPE_URL;
            println!("type_url: {}", type_url);

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
    let vault_contract = LockedAstroportVaultRobot::contract(&runner, Some(UNOPTIMIZED_PATH));
    let treasury_addr = runner.init_account(&[]).unwrap();
    let token_factory_fee = Coin::from_str(DENOM_CREATION_FEE).unwrap();

    let robot = LockedAstroportVaultRobot::new_wsteth_eth_vault(
        &runner,
        vault_contract,
        token_factory_fee,
        treasury_addr.address(),
        &dependencies,
        &admin,
    );

    (robot, treasury_addr)
}
