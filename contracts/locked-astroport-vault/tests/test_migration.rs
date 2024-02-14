pub mod common;

mod test {

    use std::str::FromStr;

    use crate::common::{get_test_runner, DENOM_CREATION_FEE, DEPS_PATH, UNOPTIMIZED_PATH};
    use apollo_cw_asset::AssetInfoUnchecked;
    use cosmwasm_std::{coin, to_json_binary, Addr, Coin, Decimal, Uint128};
    use cw_dex_astroport::astroport_v3::incentives;
    use cw_it::cw_multi_test::ContractWrapper;
    use cw_it::osmosis_std::types::cosmwasm::wasm::v1::{
        MsgMigrateContract, MsgMigrateContractResponse,
    };
    use cw_it::robot::TestRobot;
    use cw_it::test_tube::{Account, Module, Runner, Wasm};
    use cw_it::traits::CwItRunner;
    use cw_it::{ContractType, TestRunner};
    use cw_vault_standard_test_helpers::traits::CwVaultStandardRobot;
    use locked_astroport_vault::msg::{
        ApolloExtensionQueryMsg, ExtensionQueryMsg, MigrateMsg, QueryMsg,
    };
    use locked_astroport_vault::state::{Config, FeeConfig, StateResponse};
    use locked_astroport_vault_test_helpers::robot::LockedAstroportVaultRobot;

    #[cfg(feature = "osmosis-test-tube")]
    use cw_it::Artifact;

    #[allow(deprecated)]
    #[test]
    fn test_migrate_from_0_2_0_to_0_4_1() {
        let owned_runner = get_test_runner();
        let runner = owned_runner.as_ref();
        let admin = LockedAstroportVaultRobot::new_admin(&runner);
        let token_factory_fee = Coin::from_str(DENOM_CREATION_FEE).unwrap();
        let performance_fee = Decimal::percent(10);
        let treasury = runner.init_account(&[]).unwrap();

        // Construct multitest contract for v0.2.0 vault
        let old_contract = match runner {
            #[cfg(feature = "osmosis-test-tube")]
            TestRunner::OsmosisTestApp(_) => {
                let path = format!("{}/{}", DEPS_PATH, "locked_astroport_vault_0_2_0.wasm");
                ContractType::Artifact(Artifact::Local(path))
            }
            TestRunner::MultiTest(_) => {
                ContractType::MultiTestContract(Box::new(ContractWrapper::new_with_empty(
                    locked_astroport_vault_0_2_0::contract::execute,
                    locked_astroport_vault_0_2_0::contract::instantiate,
                    locked_astroport_vault_0_2_0::contract::query,
                )))
            }
            _ => panic!("Unsupported runner"),
        };

        // Upload v0.2.0 code
        let code_id_v0_2_0 = runner.store_code(old_contract, &admin).unwrap();

        // Use new Vault robot to setup a new vault. This also sets up the same pools
        // etc. with astroport as the v0.2.0 robot
        let dependencies = LockedAstroportVaultRobot::instantiate_deps(&runner, &admin, DEPS_PATH);
        let (robot, wsteth_eth_pool) = LockedAstroportVaultRobot::new_wsteth_eth_vault(
            &runner,
            LockedAstroportVaultRobot::<'_>::contract(&runner, UNOPTIMIZED_PATH),
            token_factory_fee,
            Some(FeeConfig {
                fee_rate: Decimal::percent(5),
                fee_recipients: vec![(treasury.address(), Decimal::percent(100))],
            }),
            None,
            None,
            &dependencies,
            &admin,
        );

        // Instantiate v0.2.0 contract
        let init_msg_v0_2_0 = locked_astroport_vault_0_2_0::msg::InstantiateMsg {
            owner: admin.address(),
            vault_token_subdenom: "vt".to_string(),
            pool_addr: wsteth_eth_pool.pair_addr.to_string(),
            astro_token: AssetInfoUnchecked::cw20(
                dependencies.astroport_contracts.astro_token.address.clone(),
            ),
            astroport_generator: dependencies.astroport_contracts.generator.address.clone(),
            lock_duration: 14 * 7 * 24 * 3600,
            reward_tokens: vec![],
            deposits_enabled: true,
            treasury: treasury.address(),
            performance_fee,
            router: dependencies
                .cw_dex_router_robot
                .cw_dex_router
                .0
                .to_string()
                .into(),
            reward_liquidation_target: wsteth_eth_pool.pool_assets[0].clone().into(),
            liquidity_helper: dependencies.liquidity_helper_addr.clone().into(),
            astroport_liquidity_manager: dependencies
                .astroport_contracts
                .liquidity_manager
                .address
                .clone(),
        };
        let wasm = Wasm::new(&runner);
        let contract_addr = wasm
            .instantiate(
                code_id_v0_2_0,
                &init_msg_v0_2_0,
                Some(&admin.address()),
                Some("old contract"),
                &[coin(10_000_000u128, "uosmo")],
                &admin,
            )
            .unwrap()
            .data
            .address;

        // Deposit some base tokens
        let deposit_amount = Uint128::new(1_000_000);
        robot
            .wasm()
            .execute(
                &robot.base_token(),
                &cw20::Cw20ExecuteMsg::IncreaseAllowance {
                    spender: contract_addr.clone(),
                    amount: deposit_amount,
                    expires: None,
                },
                &[],
                &admin,
            )
            .unwrap();
        robot
            .wasm()
            .execute(
                &contract_addr,
                &locked_astroport_vault_0_2_0::msg::ExecuteMsg::Deposit {
                    amount: deposit_amount,
                    recipient: None,
                },
                &[],
                &admin,
            )
            .unwrap();

        // Query state from v0.2.0 contract
        let old_staking = wasm
            .query::<_, locked_astroport_vault_0_2_0::state::StateResponse>(
                &contract_addr,
                &locked_astroport_vault_0_2_0::msg::QueryMsg::VaultExtension(
                    locked_astroport_vault_0_2_0::msg::ExtensionQueryMsg::Apollo(
                        locked_astroport_vault_0_2_0::msg::ApolloExtensionQueryMsg::State {},
                    ),
                ),
            )
            .unwrap()
            .staking;
        #[allow(deprecated)]
        let lp_token_addr = old_staking.lp_token_addr.clone();

        // Upload v0.4.1 code
        let new_contract = LockedAstroportVaultRobot::<'_>::contract(&runner, UNOPTIMIZED_PATH);
        let code_id_v0_4_1 = runner.store_code(new_contract, &admin).unwrap();

        // Migrate
        runner
            .execute::<_, MsgMigrateContractResponse>(
                MsgMigrateContract {
                    sender: admin.address(),
                    contract: contract_addr.clone(),
                    code_id: code_id_v0_4_1,
                    msg: to_json_binary(&MigrateMsg {
                        incentives_contract: dependencies
                            .astroport_contracts
                            .incentives
                            .address
                            .clone(),
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
                &contract_addr,
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
                &contract_addr,
                &QueryMsg::VaultExtension(ExtensionQueryMsg::Apollo(
                    ApolloExtensionQueryMsg::State {},
                )),
            )
            .unwrap();

        // Check that the staking struct was migrated correctly
        assert_eq!(state.staking.lp_token_addr, lp_token_addr);
        assert_eq!(
            state.staking.incentives.to_string(),
            dependencies.astroport_contracts.incentives.address
        );

        // Check that the staked amount was migrated correctly
        let staked_amount = state.staked_base_tokens;
        assert_eq!(staked_amount, deposit_amount);

        // Query generator contract staked balance
        let generator_staked = robot
            .wasm()
            .query::<_, Uint128>(
                &dependencies.astroport_contracts.generator.address,
                &cw_dex::astroport::astroport::generator::QueryMsg::Deposit {
                    lp_token: robot.base_token(),
                    user: contract_addr.clone(),
                },
            )
            .unwrap();
        assert_eq!(generator_staked, Uint128::zero());

        // Query incentive contract staked balance
        let incentives_staked = robot
            .wasm()
            .query::<_, Uint128>(
                &dependencies.astroport_contracts.incentives.address,
                &incentives::QueryMsg::Deposit {
                    lp_token: robot.base_token(),
                    user: contract_addr,
                },
            )
            .unwrap();
        assert_eq!(staked_amount, incentives_staked);
    }
}
