pub mod common;

mod test {

    use std::str::FromStr;

    use crate::common::{get_test_runner, DENOM_CREATION_FEE, DEPS_PATH, UNOPTIMIZED_PATH};
    use cosmwasm_std::{to_json_binary, Addr, Coin, Decimal, Empty};
    use cw_it::osmosis_std::types::cosmwasm::wasm::v1::{
        MsgMigrateContract, MsgMigrateContractResponse,
    };
    use cw_it::robot::TestRobot;
    use cw_it::test_tube::{Account, Runner};
    use cw_it::traits::CwItRunner;
    use cw_it::TestRunner;
    use locked_astroport_vault::msg::{
        ApolloExtensionQueryMsg, ExtensionQueryMsg, MigrateMsg, QueryMsg,
    };
    use locked_astroport_vault::state::{Config, FeeConfig, StateResponse};
    use locked_astroport_vault_test_helpers::robot::LockedAstroportVaultRobot;

    use locked_astroport_vault_test_helpers_0_2_0::robot::LockedAstroportVaultRobot as LockedAstroportVaultRobot_0_2_0;

    #[cfg(feature = "osmosis-test-tube")]
    use cw_it::{Artifact, ContractType};

    #[test]
    fn test_migrate_from_0_2_0_to_0_4_0() {
        let owned_runner = get_test_runner();
        let runner = owned_runner.as_ref();
        let admin = LockedAstroportVaultRobot::new_admin(&runner);
        let dependencies =
            LockedAstroportVaultRobot_0_2_0::instantiate_deps(&runner, &admin, DEPS_PATH);
        let token_factory_fee = Coin::from_str(DENOM_CREATION_FEE).unwrap();
        let performance_fee = Decimal::percent(10);
        let treasury = runner.init_account(&[]).unwrap();

        // Instantiate v0.2.0 vault
        let old_contract = match runner {
            #[cfg(feature = "osmosis-test-tube")]
            TestRunner::OsmosisTestApp(_) => {
                let path = format!("{}/{}", DEPS_PATH, "locked_astroport_vault_0_2_0.wasm");
                ContractType::Artifact(Artifact::Local(path))
            }
            TestRunner::MultiTest(_) => LockedAstroportVaultRobot_0_2_0::multitest_contract(),
            _ => panic!("Unsupported runner"),
        };
        let (robot, _) = LockedAstroportVaultRobot_0_2_0::new_wsteth_eth_vault(
            &runner,
            old_contract,
            token_factory_fee,
            treasury.address(),
            performance_fee,
            &dependencies,
            &admin,
        );

        let old_staking = robot.query_state().staking;

        // Upload new contract
        let new_contract = LockedAstroportVaultRobot::<'_>::contract(&runner, UNOPTIMIZED_PATH);
        let new_code_id = runner.store_code(new_contract, &admin).unwrap();

        // Migrate
        runner
            .execute::<_, MsgMigrateContractResponse>(
                MsgMigrateContract {
                    sender: admin.address(),
                    contract: robot.vault_addr.clone(),
                    code_id: new_code_id,
                    msg: to_json_binary(&MigrateMsg {
                        incentives_contract: Addr::unchecked("incentives"),
                    })
                    .unwrap()
                    .0,
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

        // Query state from contract
        let state: StateResponse = robot
            .wasm()
            .query(
                &robot.vault_addr,
                &QueryMsg::VaultExtension(ApolloExtensionQueryMsg::State {}),
            )
            .unwrap();

        // Check that the staking struct was migrated correctly
        assert_eq!(state.staking.lp_token_addr, old_staking.lp_token_addr);
        assert_eq!(state.staking.incentives, Addr::unchecked("incentives"));
    }
}
