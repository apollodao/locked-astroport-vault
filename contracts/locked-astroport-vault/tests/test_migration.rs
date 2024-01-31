pub mod common;

#[cfg(feature = "osmosis-test-tube")]
mod test {

    use std::str::FromStr;

    use crate::common::{DENOM_CREATION_FEE, DEPS_PATH, UNOPTIMIZED_PATH};
    use cosmwasm_std::{to_json_binary, Addr, Coin, Decimal, Empty};
    use cw_it::osmosis_std::types::cosmwasm::wasm::v1::{
        MsgMigrateContract, MsgMigrateContractResponse,
    };
    use cw_it::osmosis_test_tube::OsmosisTestApp;
    use cw_it::robot::TestRobot;
    use cw_it::test_tube::{Account, Runner};
    use cw_it::traits::CwItRunner;
    use cw_it::{Artifact, ContractType, OwnedTestRunner};
    use locked_astroport_vault::msg::{ApolloExtensionQueryMsg, ExtensionQueryMsg, QueryMsg};
    use locked_astroport_vault::state::{Config, FeeConfig};
    use locked_astroport_vault_test_helpers::robot::LockedAstroportVaultRobot;

    use locked_astroport_vault_test_helpers_0_2_0::robot::LockedAstroportVaultRobot as LockedAstroportVaultRobot_0_2_0;

    #[test]
    fn test_migrate_from_0_2_0() {
        let owned_runner = OwnedTestRunner::OsmosisTestApp(OsmosisTestApp::new());
        let runner = owned_runner.as_ref();
        let admin = LockedAstroportVaultRobot::new_admin(&runner);
        let dependencies =
            LockedAstroportVaultRobot_0_2_0::instantiate_deps(&runner, &admin, DEPS_PATH);
        let token_factory_fee = Coin::from_str(DENOM_CREATION_FEE).unwrap();
        let performance_fee = Decimal::percent(10);
        let treasury = runner.init_account(&[]).unwrap();

        // Instantiate v0.2.0 vault
        let path = format!("{}/{}", DEPS_PATH, "locked_astroport_vault_0_2_0.wasm");
        let old_contract = ContractType::Artifact(Artifact::Local(path));
        let (robot, _) = LockedAstroportVaultRobot_0_2_0::new_wsteth_eth_vault(
            &runner,
            old_contract,
            token_factory_fee,
            treasury.address(),
            performance_fee,
            &dependencies,
            &admin,
        );

        // Upload new contract
        let new_contract = LockedAstroportVaultRobot::<'_>::wasm_contract(UNOPTIMIZED_PATH);
        let new_code_id = runner.store_code(new_contract, &admin).unwrap();

        // Migrate
        runner
            .execute::<_, MsgMigrateContractResponse>(
                MsgMigrateContract {
                    sender: admin.address(),
                    contract: robot.vault_addr.clone(),
                    code_id: new_code_id,
                    msg: to_json_binary(&Empty {}).unwrap().0,
                },
                "/cosmwasm.wasm.v1.MsgMigrateContract",
                &admin,
            )
            .unwrap();

        // Check that the config was migrated correctly
        let config = robot
            .wasm()
            .query::<_, Config>(
                &robot.vault_addr,
                &QueryMsg::VaultExtension(ExtensionQueryMsg::Apollo(
                    ApolloExtensionQueryMsg::Config {},
                )),
            )
            .unwrap();
        assert_eq!(config.performance_fee.fee_rate, performance_fee);
        assert_eq!(
            config.performance_fee.fee_recipients,
            vec![(Addr::unchecked(treasury.address()), Decimal::one())]
        );
        assert_eq!(
            FeeConfig::<String>::from(config.deposit_fee),
            FeeConfig::default()
        );
        assert_eq!(
            FeeConfig::<String>::from(config.withdrawal_fee),
            FeeConfig::default()
        );
    }
}
