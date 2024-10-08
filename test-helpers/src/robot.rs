use std::collections::HashSet;
use std::str::FromStr;

use apollo_cw_asset::AssetInfo;
use cosmwasm_schema::cw_serde;
use cosmwasm_std::{coin, coins, to_json_binary, Addr, Coin, Coins, Decimal, Empty, Uint128};
use cw20::{Cw20ExecuteMsg, Cw20QueryMsg};
use cw_dex_astroport::astroport::asset::AssetInfo as AstroAssetInfo;
use cw_dex_astroport::astroport::factory::PairType;

use cw_dex_astroport::AstroportPool;
use cw_dex_router::operations::{SwapOperationUnchecked, SwapOperationsListUnchecked};
use cw_it::astroport::robot::AstroportTestRobot;
use cw_it::astroport::utils::{create_astroport_pair, AstroportContracts};
use cw_it::cw_multi_test::ContractWrapper;
use cw_it::helpers::Unwrap;
use cw_it::osmosis_std::types::cosmwasm::wasm::v1::{
    MsgMigrateContract, MsgMigrateContractResponse,
};
use cw_it::robot::TestRobot;
use cw_it::test_tube::{Account, Module, Runner, SigningAccount, Wasm};
use cw_it::traits::CwItRunner;
use cw_it::{Artifact, ContractType, TestRunner};
use cw_ownable::Ownership;
use cw_utils::Duration;
use cw_vault_standard::extensions::lockup::{LockupQueryMsg, UnlockingPosition};
use cw_vault_standard::{VaultInfoResponse, VaultStandardInfoResponse, VaultStandardQueryMsg};
use cw_vault_standard_test_helpers::traits::force_unlock::ForceUnlockVaultRobot;
use cw_vault_standard_test_helpers::traits::lockup::LockedVaultRobot;
use cw_vault_standard_test_helpers::traits::CwVaultStandardRobot;
use liquidity_helper::LiquidityHelperUnchecked;
use locked_astroport_vault::msg::{
    ApolloExtensionExecuteMsg, ApolloExtensionQueryMsg, ExecuteMsg, ExtensionExecuteMsg,
    ExtensionQueryMsg, InstantiateMsg, QueryMsg,
};
use locked_astroport_vault::state::{Config, ConfigBase, ConfigUpdates, FeeConfig, StateResponse};

use crate::router::CwDexRouterRobot;

pub const LOCKED_ASTROPORT_VAULT_WASM_NAME: &str = "locked_astroport_vault.wasm";
pub const ASTROPORT_LIQUIDITY_HELPER_WASM_NAME: &str = "astroport_liquidity_helper.wasm";
pub const ASTROPORT_ARTIFACTS_DIR: &str = "astroport-artifacts";

pub const TWO_WEEKS_IN_SECS: u64 = 1_209_600;

pub const WSTETH_DENOM: &str = "uwsteth";
pub const ETH_DENOM: &str = "ueth";
pub const ASTRO_DENOM: &str = "uastro";
pub const USDC_DENOM: &str = "uusdc";
pub const AXL_DENOM: &str = "uaxl";
pub const NTRN_DENOM: &str = "untrn";

/// The default coins to fund new accounts with
pub const DEFAULT_COINS: &str = "1000000000000000000uosmo,1000000000000000000uwsteth,1000000000000000000ueth,1000000000000000000uastro,1000000000000000000uusdc,1000000000000000000uaxl,1000000000000000000untrn";

pub const INITIAL_LIQ: u128 = 1_000_000_000_000_000u128;

pub const DENOM_CREATION_FEE: &str = "10000000uosmo";

/// Contracts that are required for the LockedAstroportVaultRobot to function.
pub struct LockedVaultDependencies<'a> {
    pub astroport_contracts: AstroportContracts,
    pub cw_dex_router_robot: CwDexRouterRobot<'a>,
    pub liquidity_helper_addr: String,
}

#[cw_serde]
struct AstroportLiquidityHelperInstantiateMsg {
    astroport_factory: String,
}

pub struct LockedAstroportVaultRobot<'a> {
    pub runner: &'a TestRunner<'a>,
    pub vault_addr: String,
    pub astroport_contracts: AstroportContracts,
    pub base_token_is_cw20: bool,
}

impl<'a> LockedAstroportVaultRobot<'a> {
    /// Returns a `ContractType` representing a local wasm file of the contract.
    /// If `artifacts_dir` is `None`, the default path of `artifacts` will be
    /// used.
    pub fn wasm_contract(artifacts_dir: &str) -> ContractType {
        let path = format!("{}/{}", artifacts_dir, LOCKED_ASTROPORT_VAULT_WASM_NAME);
        println!("Loading contract from {}", path);
        ContractType::Artifact(Artifact::Local(path))
    }

    /// Returns a `ContractType` representing a multi-test contract of the
    /// contract.
    pub fn multitest_contract() -> ContractType {
        ContractType::MultiTestContract(Box::new(
            ContractWrapper::new_with_empty(
                locked_astroport_vault::contract::execute,
                locked_astroport_vault::contract::instantiate,
                locked_astroport_vault::contract::query,
            )
            .with_migrate(locked_astroport_vault::contract::migrate)
            .with_reply(locked_astroport_vault::contract::reply),
        ))
    }

    /// Returns a `ContractType` representing the contract to use for the given
    /// `TestRunner`.
    pub fn contract(runner: &'a TestRunner<'a>, _artifacts_dir: &str) -> ContractType {
        match runner {
            #[cfg(feature = "osmosis-test-tube")]
            TestRunner::OsmosisTestApp(_) => Self::wasm_contract(_artifacts_dir),
            TestRunner::MultiTest(_) => Self::multitest_contract(),
            _ => panic!("Unsupported runner"),
        }
    }

    /// Creates a new account with the default coins.
    pub fn new_admin(runner: &'a TestRunner<'a>) -> SigningAccount {
        runner
            .init_account(&Coins::from_str(DEFAULT_COINS).unwrap().to_vec())
            .unwrap()
    }

    // Uploads and instantiates dependencies for the LockedAstroportVaultRobot.
    pub fn instantiate_deps(
        runner: &'a TestRunner<'a>,
        signer: &SigningAccount,
        artifacts_dir: &str,
    ) -> LockedVaultDependencies<'a> {
        // Upload and instantiate astroport contracts
        let astroport_contracts = AstroportContracts::new_from_local_contracts(
            runner,
            &Some(&format!("{}/{}", artifacts_dir, ASTROPORT_ARTIFACTS_DIR)),
            false,
            &None,
            signer,
        );

        // Create CwDexRouterRobot
        let cw_dex_router_robot = CwDexRouterRobot::new(
            runner,
            CwDexRouterRobot::contract(runner, artifacts_dir),
            signer,
        );

        // Upload and instantiate liquidity helper
        let liquidity_helper_contract = match runner {
            #[cfg(feature = "osmosis-test-tube")]
            TestRunner::OsmosisTestApp(_) => ContractType::Artifact(Artifact::Local(format!(
                "{}/{}",
                artifacts_dir, ASTROPORT_LIQUIDITY_HELPER_WASM_NAME
            ))),
            TestRunner::MultiTest(_) => {
                ContractType::MultiTestContract(Box::new(ContractWrapper::new_with_empty(
                    astroport_liquidity_helper::contract::execute,
                    astroport_liquidity_helper::contract::instantiate,
                    astroport_liquidity_helper::contract::query,
                )))
            }
            _ => panic!("Unsupported runner"),
        };
        let wasm = Wasm::new(runner);
        let code_id = runner
            .store_code(liquidity_helper_contract, signer)
            .unwrap();
        let liquidity_helper_addr = wasm
            .instantiate(
                code_id,
                &AstroportLiquidityHelperInstantiateMsg {
                    astroport_factory: astroport_contracts.factory.address.to_string(),
                },
                Some(&signer.address()),
                Some("astroport_liquidity_helper"),
                &[],
                signer,
            )
            .unwrap()
            .data
            .address;

        LockedVaultDependencies {
            astroport_contracts,
            cw_dex_router_robot,
            liquidity_helper_addr,
        }
    }

    /// Creates a new `LockedAstroportVaultRobot` from the given
    /// `InstantiateMsg`.
    pub fn new_with_instantiate_msg(
        runner: &'a TestRunner<'a>,
        vault_contract: ContractType,
        token_factory_fee: Coin,
        instantiate_msg: &InstantiateMsg,
        dependencies: &LockedVaultDependencies<'a>,
        signer: &SigningAccount,
    ) -> Self {
        let code_id = runner.store_code(vault_contract, signer).unwrap();

        let wasm = Wasm::new(runner);
        let vault_addr = wasm
            .instantiate(
                code_id,
                instantiate_msg,
                Some(&signer.address()),
                Some("Locked Astroport Vault"),
                &[token_factory_fee],
                signer,
            )
            .unwrap()
            .data
            .address;

        let vault_info: VaultInfoResponse = wasm
            .query(&vault_addr, &VaultStandardQueryMsg::<Empty>::Info {})
            .unwrap();
        let base_token = vault_info.base_token;
        let base_token_is_cw20 =
            base_token.starts_with(signer.prefix()) || base_token.starts_with("contract");

        Self {
            runner,
            vault_addr,
            astroport_contracts: dependencies.astroport_contracts.clone(),
            base_token_is_cw20,
        }
    }

    /// Creates a AXL/NTRN pool and a new LockedAstroportVaultRobot for the pool
    /// with the specified lockup duration.
    pub fn new_axlr_ntrn_vault(
        runner: &'a TestRunner<'a>,
        vault_contract: ContractType,
        token_factory_fee: Coin,
        performance_fee: Option<FeeConfig<String>>,
        deposit_fee: Option<FeeConfig<String>>,
        withdrawal_fee: Option<FeeConfig<String>>,
        lock_duration: u64,
        dependencies: &LockedVaultDependencies<'a>,
        signer: &SigningAccount,
    ) -> (Self, AstroportPool, AstroportPool) {
        let axl = AssetInfo::native(AXL_DENOM.to_string());
        let ntrn = AssetInfo::native(NTRN_DENOM.to_string());
        let astro = AssetInfo::native(ASTRO_DENOM.to_string());
        let liq_manager =
            Addr::unchecked(&dependencies.astroport_contracts.liquidity_manager.address);

        // Create AXL/NTRN astroport pool
        let (axl_ntrn_pair, axl_ntrn_lp) = create_astroport_pair(
            runner,
            &dependencies.astroport_contracts.factory.address,
            PairType::Xyk {},
            // PairTypeV2::Xyk {},
            [
                asset_info_to_astroport_v5_assetinfo(&axl),
                asset_info_to_astroport_v5_assetinfo(&ntrn),
                // axl.clone().into(),
                // ntrn.clone().into(),
            ],
            None,
            signer,
            Some([Uint128::from(INITIAL_LIQ), Uint128::from(INITIAL_LIQ)]),
            &[Coin::from_str(DENOM_CREATION_FEE).unwrap()],
        );
        let axl_ntrn_lp = asset_info_from_str(&axl_ntrn_lp, signer.prefix());
        let axl_ntrn_pool = AstroportPool {
            lp_token: axl_ntrn_lp.clone(),
            pair_addr: Addr::unchecked(&axl_ntrn_pair),
            pair_type: PairType::Xyk {},
            pool_assets: [axl.clone(), ntrn.clone()].to_vec(),
            liquidity_manager: if axl_ntrn_lp.is_native() {
                None
            } else {
                Some(liq_manager.clone())
            },
        };

        // Create ASTRO/NTRN astroport pool
        let (astro_ntrn_pair, astro_ntrn_lp) = create_astroport_pair(
            runner,
            &dependencies.astroport_contracts.factory.address,
            PairType::Xyk {},
            // PairTypeV2::Xyk {},
            [
                asset_info_to_astroport_v5_assetinfo(&astro),
                asset_info_to_astroport_v5_assetinfo(&ntrn),
                // astro.clone().into(),
                // ntrn.clone().into(),
            ],
            None,
            signer,
            Some([Uint128::from(INITIAL_LIQ), Uint128::from(INITIAL_LIQ)]),
            &[Coin::from_str(DENOM_CREATION_FEE).unwrap()],
        );
        let astro_ntrn_lp = asset_info_from_str(&astro_ntrn_lp, signer.prefix());
        let astro_ntrn_pool = AstroportPool {
            lp_token: astro_ntrn_lp.clone(),
            pair_addr: Addr::unchecked(&astro_ntrn_pair),
            pair_type: PairType::Xyk {},
            pool_assets: [astro.clone(), ntrn.clone()].to_vec(),
            liquidity_manager: if astro_ntrn_lp.is_native() {
                None
            } else {
                Some(liq_manager.clone())
            },
        };

        // Set routes in cw-dex-router
        // AXL <-> NTRN
        dependencies.cw_dex_router_robot.set_path(
            axl.clone().into(),
            ntrn.clone().into(),
            SwapOperationsListUnchecked::new(vec![swap_operation(
                &axl_ntrn_pair,
                &axl_ntrn_lp,
                &axl,
                &ntrn,
                liq_manager.as_str(),
            )]),
            true,
            signer,
        );

        // ASTRO <-> NTRN
        dependencies.cw_dex_router_robot.set_path(
            astro.clone().into(),
            ntrn.clone().into(),
            SwapOperationsListUnchecked::new(vec![swap_operation(
                &astro_ntrn_pair,
                &astro_ntrn_lp,
                &astro,
                &ntrn,
                liq_manager.as_str(),
            )]),
            true,
            signer,
        );

        // ASTRO <-> NTRN <-> AXL
        dependencies.cw_dex_router_robot.set_path(
            astro.clone().into(),
            axl.clone().into(),
            SwapOperationsListUnchecked::new(vec![
                swap_operation(
                    &astro_ntrn_pair,
                    &astro_ntrn_lp,
                    &astro,
                    &ntrn,
                    liq_manager.as_str(),
                ),
                swap_operation(
                    &axl_ntrn_pair,
                    &axl_ntrn_lp,
                    &ntrn,
                    &axl,
                    liq_manager.as_str(),
                ),
            ]),
            true,
            signer,
        );

        let instantiate_msg = InstantiateMsg {
            owner: signer.address().to_string(),
            vault_token_subdenom: "testVaultToken".to_string(),
            lock_duration,
            reward_tokens: vec![astro.into(), axl.into(), ntrn.clone().into()],
            deposits_enabled: true,
            performance_fee,
            deposit_fee,
            withdrawal_fee,
            router: dependencies
                .cw_dex_router_robot
                .cw_dex_router
                .clone()
                .into(),
            reward_liquidation_target: ntrn.into(),
            pool_addr: axl_ntrn_pair,
            liquidity_helper: LiquidityHelperUnchecked::new(
                dependencies.liquidity_helper_addr.clone(),
            ),
            astroport_liquidity_manager: if axl_ntrn_lp.is_native() {
                None
            } else {
                Some(liq_manager.to_string())
            },
            astroport_incentives_addr: dependencies.astroport_contracts.incentives.address.clone(),
        };

        (
            Self::new_with_instantiate_msg(
                runner,
                vault_contract,
                token_factory_fee,
                &instantiate_msg,
                dependencies,
                signer,
            ),
            axl_ntrn_pool,
            astro_ntrn_pool,
        )
    }

    /// Creates a AXL/NTRN pool and a new LockedAstroportVaultRobot for the pool
    /// with no lockup.
    pub fn new_unlocked_axlr_ntrn_vault(
        runner: &'a TestRunner<'a>,
        vault_contract: ContractType,
        token_factory_fee: Coin,
        performance_fee: Option<FeeConfig<String>>,
        deposit_fee: Option<FeeConfig<String>>,
        withdrawal_fee: Option<FeeConfig<String>>,
        dependencies: &LockedVaultDependencies<'a>,
        signer: &SigningAccount,
    ) -> (Self, AstroportPool, AstroportPool) {
        Self::new_axlr_ntrn_vault(
            runner,
            vault_contract,
            token_factory_fee,
            performance_fee,
            deposit_fee,
            withdrawal_fee,
            0,
            dependencies,
            signer,
        )
    }

    /// Creates a new wstETH/ETH vault with a lockup of 2 weeks.
    /// The vault has the reward assets with the following swap paths:
    /// ASTRO -> USDC -> ETH
    /// AXL -> NTRN -> USDC -> ETH
    /// NTRN -> USDC -> ETH
    pub fn new_wsteth_eth_vault(
        runner: &'a TestRunner<'a>,
        vault_contract: ContractType,
        token_factory_fee: Coin,
        performance_fee: Option<FeeConfig<String>>,
        deposit_fee: Option<FeeConfig<String>>,
        withdrawal_fee: Option<FeeConfig<String>>,
        dependencies: &'a LockedVaultDependencies<'a>,
        signer: &SigningAccount,
    ) -> (Self, AstroportPool) {
        let wsteth = AssetInfo::native(WSTETH_DENOM.to_string());
        let eth = AssetInfo::native(ETH_DENOM.to_string());
        let astro = AssetInfo::native(ASTRO_DENOM.to_string());
        let usdc = AssetInfo::native(USDC_DENOM.to_string());
        let axl = AssetInfo::native(AXL_DENOM.to_string());
        let ntrn = AssetInfo::native(NTRN_DENOM.to_string());

        let LockedVaultDependencies {
            astroport_contracts,
            cw_dex_router_robot,
            liquidity_helper_addr,
        } = dependencies;

        let liq_manager =
            Addr::unchecked(&dependencies.astroport_contracts.liquidity_manager.address);

        // Create astroport pairs
        let (wsteth_eth_pair, wsteth_eth_lp) = create_astroport_pair(
            runner,
            &astroport_contracts.factory.address,
            PairType::Xyk {},
            // PairTypeV2::Xyk {},
            [
                asset_info_to_astroport_v5_assetinfo(&wsteth),
                asset_info_to_astroport_v5_assetinfo(&eth),
                // wsteth.clone().into(),
                // eth.clone().into(),
            ],
            None,
            signer,
            Some([Uint128::from(INITIAL_LIQ), Uint128::from(INITIAL_LIQ)]),
            &[Coin::from_str(DENOM_CREATION_FEE).unwrap()],
        );
        let wsteth_eth_lp = asset_info_from_str(&wsteth_eth_lp, signer.prefix());
        let wsteth_eth_pool = AstroportPool {
            lp_token: wsteth_eth_lp.clone(),
            pair_addr: Addr::unchecked(&wsteth_eth_pair),
            pair_type: PairType::Xyk {},
            pool_assets: [wsteth.clone(), eth.clone()].to_vec(),
            liquidity_manager: if wsteth_eth_lp.is_native() {
                None
            } else {
                Some(liq_manager.clone())
            },
        };

        let (astro_usdc_pair, astro_usdc_lp) = create_astroport_pair(
            runner,
            &astroport_contracts.factory.address,
            PairType::Xyk {},
            // PairTypeV2::Xyk {},
            [
                asset_info_to_astroport_v5_assetinfo(&astro),
                asset_info_to_astroport_v5_assetinfo(&usdc),
                // astro.clone().into(),
                // usdc.clone().into(),
            ],
            None,
            signer,
            Some([Uint128::from(INITIAL_LIQ), Uint128::from(INITIAL_LIQ)]),
            &[Coin::from_str(DENOM_CREATION_FEE).unwrap()],
        );
        let astro_usdc_lp = asset_info_from_str(&astro_usdc_lp, signer.prefix());

        let (ntrn_usdc_pair, ntrn_usdc_lp) = create_astroport_pair(
            runner,
            &astroport_contracts.factory.address,
            PairType::Xyk {},
            // PairTypeV2::Xyk {},
            [
                asset_info_to_astroport_v5_assetinfo(&ntrn),
                asset_info_to_astroport_v5_assetinfo(&usdc),
                // ntrn.clone().into(),
                // usdc.clone().into(),
            ],
            None,
            signer,
            Some([Uint128::from(INITIAL_LIQ), Uint128::from(INITIAL_LIQ)]),
            &[Coin::from_str(DENOM_CREATION_FEE).unwrap()],
        );
        let ntrn_usdc_lp = asset_info_from_str(&ntrn_usdc_lp, signer.prefix());

        let (eth_usdc_pair, eth_usdc_lp) = create_astroport_pair(
            runner,
            &astroport_contracts.factory.address,
            PairType::Xyk {},
            // PairTypeV2::Xyk {},
            [
                asset_info_to_astroport_v5_assetinfo(&eth),
                asset_info_to_astroport_v5_assetinfo(&usdc),
                // eth.clone().into(),
                // usdc.clone().into(),
            ],
            None,
            signer,
            Some([Uint128::from(INITIAL_LIQ), Uint128::from(INITIAL_LIQ)]),
            &[Coin::from_str(DENOM_CREATION_FEE).unwrap()],
        );
        let eth_usdc_lp = asset_info_from_str(&eth_usdc_lp, signer.prefix());

        let (axl_ntrn_pair, axl_ntrn_lp) = create_astroport_pair(
            runner,
            &astroport_contracts.factory.address,
            PairType::Xyk {},
            // PairTypeV2::Xyk {},
            [
                asset_info_to_astroport_v5_assetinfo(&axl),
                asset_info_to_astroport_v5_assetinfo(&ntrn),
                // axl.clone().into(),
                // ntrn.clone().into(),
            ],
            None,
            signer,
            Some([Uint128::from(INITIAL_LIQ), Uint128::from(INITIAL_LIQ)]),
            &[Coin::from_str(DENOM_CREATION_FEE).unwrap()],
        );
        let axl_ntrn_lp = asset_info_from_str(&axl_ntrn_lp, signer.prefix());

        // Set routes in cw-dex-router
        // WSTETH <-> ETH
        cw_dex_router_robot.set_path(
            eth.clone().into(),
            wsteth.clone().into(),
            SwapOperationsListUnchecked::new(vec![swap_operation(
                &wsteth_eth_pair,
                &wsteth_eth_lp,
                &eth,
                &wsteth,
                liq_manager.as_ref(),
            )]),
            true,
            signer,
        );
        // NTRN <-> USDC <-> ETH
        cw_dex_router_robot.set_path(
            ntrn.clone().into(),
            eth.clone().into(),
            SwapOperationsListUnchecked::new(vec![
                swap_operation(
                    &ntrn_usdc_pair,
                    &ntrn_usdc_lp,
                    &ntrn,
                    &usdc,
                    liq_manager.as_ref(),
                ),
                swap_operation(
                    &eth_usdc_pair,
                    &eth_usdc_lp,
                    &usdc,
                    &eth,
                    liq_manager.as_ref(),
                ),
            ]),
            true,
            signer,
        );
        // ASTRO <-> USDC <-> ETH
        cw_dex_router_robot.set_path(
            astro.clone().into(),
            eth.clone().into(),
            SwapOperationsListUnchecked::new(vec![
                swap_operation(
                    &astro_usdc_pair,
                    &astro_usdc_lp,
                    &astro,
                    &usdc,
                    liq_manager.as_ref(),
                ),
                swap_operation(
                    &eth_usdc_pair,
                    &eth_usdc_lp,
                    &usdc,
                    &eth,
                    liq_manager.as_ref(),
                ),
            ]),
            true,
            signer,
        );
        // AXL <-> USDC <-> ETH
        cw_dex_router_robot.set_path(
            axl.clone().into(),
            eth.clone().into(),
            SwapOperationsListUnchecked::new(vec![
                swap_operation(
                    &axl_ntrn_pair,
                    &axl_ntrn_lp,
                    &axl,
                    &ntrn,
                    liq_manager.as_ref(),
                ),
                swap_operation(
                    &ntrn_usdc_pair,
                    &ntrn_usdc_lp,
                    &ntrn,
                    &usdc,
                    liq_manager.as_ref(),
                ),
                swap_operation(
                    &eth_usdc_pair,
                    &eth_usdc_lp,
                    &usdc,
                    &eth,
                    liq_manager.as_ref(),
                ),
            ]),
            true,
            signer,
        );

        let init_msg = InstantiateMsg {
            owner: signer.address(),
            vault_token_subdenom: "testVaultToken".to_string(),
            lock_duration: TWO_WEEKS_IN_SECS,
            reward_tokens: vec![astro.into(), axl.into(), ntrn.into()],
            deposits_enabled: true,
            performance_fee,
            deposit_fee,
            withdrawal_fee,
            router: dependencies
                .cw_dex_router_robot
                .cw_dex_router
                .clone()
                .into(),
            reward_liquidation_target: eth.into(),
            pool_addr: wsteth_eth_pair,
            liquidity_helper: LiquidityHelperUnchecked::new(liquidity_helper_addr.clone()),
            astroport_liquidity_manager: if wsteth_eth_lp.is_native() {
                None
            } else {
                Some(liq_manager.to_string())
            },
            astroport_incentives_addr: astroport_contracts.incentives.address.clone(),
        };

        (
            Self::new_with_instantiate_msg(
                runner,
                vault_contract,
                token_factory_fee,
                &init_msg,
                dependencies,
                signer,
            ),
            wsteth_eth_pool,
        )
    }

    pub fn migrate(&self, new_code_id: u64, signer: &SigningAccount) -> &Self {
        self.runner
            .execute::<_, MsgMigrateContractResponse>(
                MsgMigrateContract {
                    sender: signer.address(),
                    contract: self.vault_addr.clone(),
                    code_id: new_code_id,
                    msg: to_json_binary(&Empty {}).unwrap().0,
                },
                "/cosmwasm.wasm.v1.MsgMigrateContract",
                signer,
            )
            .unwrap();
        self
    }

    pub fn send_cw20(
        &self,
        amount: Uint128,
        token_addr: &str,
        recipient: &str,
        signer: &SigningAccount,
    ) -> &Self {
        self.wasm()
            .execute(
                token_addr,
                &Cw20ExecuteMsg::Transfer {
                    recipient: recipient.to_string(),
                    amount,
                },
                &[],
                signer,
            )
            .unwrap();
        self
    }

    /// Create a new testing account with some base token balance.
    pub fn new_user(&self, admin: &SigningAccount) -> SigningAccount {
        let user = self
            .runner
            .init_account(&[coin(1_000_000_000_000_000, "uosmo")])
            .unwrap();

        if self.base_token_is_cw20 {
            self.send_cw20(
                Uint128::new(1_000_000),
                &self.base_token(),
                &user.address(),
                admin,
            );
        } else {
            self.send_native_tokens(
                admin,
                user.address(),
                Uint128::new(1_000_000),
                self.base_token(),
            );
        }

        user
    }

    /// Deposit base tokens into the vault
    pub fn deposit(
        &self,
        amount: Uint128,
        recipient: Option<String>,
        unwrap_choice: Unwrap,
        signer: &SigningAccount,
    ) -> &Self {
        if self.base_token_is_cw20 {
            self.increase_cw20_allowance(&self.base_token(), &self.vault_addr, amount, signer)
                .deposit_with_funds(amount, recipient, &[], unwrap_choice, signer)
        } else {
            CwVaultStandardRobot::deposit(self, amount, recipient, unwrap_choice, signer);
            self
        }
    }

    /// Increase CW20 allowance and deposit into the vault.
    pub fn deposit_cw20(
        &self,
        amount: Uint128,
        recipient: Option<String>,
        unwrap_choice: Unwrap,
        signer: &SigningAccount,
    ) -> &Self {
        self.increase_cw20_allowance(&self.base_token(), &self.vault_addr, amount, signer)
            .deposit_with_funds(amount, recipient, &[], unwrap_choice, signer)
    }

    /// Update the config of the vault and return a reference to the robot.
    pub fn update_config(
        &self,
        updates: ConfigUpdates<String>,
        unwrap_choice: Unwrap,
        signer: &SigningAccount,
    ) -> &Self {
        unwrap_choice.unwrap(self.wasm().execute(
            &self.vault_addr,
            &ExecuteMsg::VaultExtension(ExtensionExecuteMsg::Apollo(
                ApolloExtensionExecuteMsg::UpdateConfig { updates },
            )),
            &[],
            signer,
        ));
        self
    }

    /// Calls `ExecuteMsg::Redeem` to redeem vault tokens from the vault
    pub fn redeem(
        &self,
        amount: Uint128,
        recipient: Option<String>,
        unwrap_choice: Unwrap,
        funds: Option<Vec<Coin>>,
        signer: &SigningAccount,
    ) -> &Self {
        unwrap_choice.unwrap(self.wasm().execute(
            &self.vault_addr,
            &ExecuteMsg::Redeem { amount, recipient },
            &funds.unwrap_or_else(|| coins(amount.u128(), self.vault_token())),
            signer,
        ));
        self
    }

    /// Compounds the rewards in the vault
    pub fn compound_vault(&self, signer: &SigningAccount) -> &Self {
        self.wasm()
            .execute(
                &self.vault_addr,
                &ExecuteMsg::VaultExtension(ExtensionExecuteMsg::Apollo(
                    ApolloExtensionExecuteMsg::Compound {},
                )),
                &[],
                signer,
            )
            .unwrap();
        self
    }

    /// Updates the contract's ownership
    pub fn update_ownership(
        &self,
        action: cw_ownable::Action,
        unwrap_choice: Unwrap,
        signer: &SigningAccount,
    ) -> &Self {
        let msg = ExecuteMsg::VaultExtension(ExtensionExecuteMsg::UpdateOwnership(action));
        unwrap_choice.unwrap(self.wasm().execute(&self.vault_addr, &msg, &[], signer));
        self
    }

    /// Increases the test runner's block time by the given number of seconds
    pub fn increase_time(&self, seconds: u64) -> &Self {
        self.runner.increase_time(seconds).unwrap();
        self
    }

    pub fn increase_time_by_lockup_duration(&self) -> &Self {
        let seconds = match self.query_lockup_duration() {
            Duration::Time(s) => s,
            Duration::Height(_) => panic!("Expected time duration, got height duration"),
        };
        self.increase_time(seconds);
        self
    }

    // Queries //

    /// Queries the ownership info of the vault
    pub fn query_ownership(&self) -> Ownership<Addr> {
        self.wasm()
            .query::<_, cw_ownable::Ownership<Addr>>(
                &self.vault_addr,
                &QueryMsg::VaultExtension(ExtensionQueryMsg::Apollo(
                    ApolloExtensionQueryMsg::Ownership {},
                )),
            )
            .unwrap()
    }

    /// Queries the contract version info of the vault
    pub fn query_contract_version(&self) -> cw2::ContractVersion {
        self.wasm()
            .query::<_, cw2::ContractVersion>(
                &self.vault_addr,
                &QueryMsg::VaultExtension(ExtensionQueryMsg::Apollo(
                    ApolloExtensionQueryMsg::ContractVersion {},
                )),
            )
            .unwrap()
    }

    /// Queries all the unlocking positions of the given owner
    pub fn query_unlocking_positions(&self, owner: &str) -> Vec<UnlockingPosition> {
        self.wasm()
            .query::<_, Vec<UnlockingPosition>>(
                &self.vault_addr,
                &QueryMsg::VaultExtension(ExtensionQueryMsg::Lockup(
                    LockupQueryMsg::UnlockingPositions {
                        owner: owner.to_string(),
                        start_after: None,
                        limit: None,
                    },
                )),
            )
            .unwrap()
    }

    /// Queries the config of the vault
    pub fn query_config(&self) -> ConfigBase<Addr> {
        self.wasm()
            .query::<_, Config>(
                &self.vault_addr,
                &QueryMsg::VaultExtension(ExtensionQueryMsg::Apollo(
                    ApolloExtensionQueryMsg::Config {},
                )),
            )
            .unwrap()
    }

    /// Queries the ConvertToShares query to convert an amount of base tokens to
    /// vault tokens
    pub fn query_convert_to_shares(&self, amount: impl Into<Uint128>) -> Uint128 {
        self.wasm()
            .query::<_, Uint128>(
                &self.vault_addr,
                &QueryMsg::ConvertToShares {
                    amount: amount.into(),
                },
            )
            .unwrap()
    }

    /// Queries the ConvertToAssets query to convert an amount of vault tokens
    /// to base tokens
    pub fn query_convert_to_assets(&self, amount: impl Into<Uint128>) -> Uint128 {
        self.wasm()
            .query::<_, Uint128>(
                &self.vault_addr,
                &QueryMsg::ConvertToAssets {
                    amount: amount.into(),
                },
            )
            .unwrap()
    }

    /// Queries the current block time in seconds since the UNIX epoch
    pub fn query_block_time_seconds(&self) -> u64 {
        self.runner.query_block_time_nanos() / 1_000_000_000
    }

    /// Queries the total supply of vault tokens
    pub fn query_total_vault_token_supply(&self) -> Uint128 {
        self.wasm()
            .query::<_, Uint128>(&self.vault_addr, &QueryMsg::TotalVaultTokenSupply {})
            .unwrap()
    }

    /// Queries the total assets (base tokens) held by the vault
    pub fn query_total_vault_assets(&self) -> Uint128 {
        self.wasm()
            .query::<_, Uint128>(&self.vault_addr, &QueryMsg::TotalAssets {})
            .unwrap()
    }

    pub fn query_force_withdraw_whitelist(
        &self,
        start_after: Option<String>,
        limit: Option<u32>,
    ) -> Vec<String> {
        self.wasm()
            .query::<_, Vec<String>>(
                &self.vault_addr,
                &QueryMsg::VaultExtension(ExtensionQueryMsg::Apollo(
                    ApolloExtensionQueryMsg::ForceWithdrawWhitelist { start_after, limit },
                )),
            )
            .unwrap()
    }

    pub fn query_vault_standard_info(&self) -> VaultStandardInfoResponse {
        self.wasm()
            .query::<_, VaultStandardInfoResponse>(
                &self.vault_addr,
                &QueryMsg::VaultStandardInfo {},
            )
            .unwrap()
    }

    pub fn query_state(&self) -> StateResponse {
        self.wasm()
            .query::<_, StateResponse>(
                &self.vault_addr,
                &QueryMsg::VaultExtension(ExtensionQueryMsg::Apollo(
                    ApolloExtensionQueryMsg::State {},
                )),
            )
            .unwrap()
    }

    pub fn query_vault_token_exchange_rate(&self, quote_denom: impl Into<String>) -> Decimal {
        let quote_denom = quote_denom.into();
        self.wasm()
            .query::<_, Decimal>(
                &self.vault_addr,
                &QueryMsg::VaultTokenExchangeRate { quote_denom },
            )
            .unwrap()
    }

    // Assertions //

    /// Asserts that value a and b are equal, or off by only one.
    pub fn assert_eq_or_off_by_one(a: impl Into<Uint128>, b: impl Into<Uint128>) {
        let a = a.into();
        let b = b.into();

        if a != b && a.abs_diff(b) != Uint128::new(1) {
            panic!("assert_eq_or_off_by_one failed. {} != {}", a, b);
        }
    }

    /// Asserts that the vault token balance of the given address, when
    /// converted to an amount of base tokens using the current exchange
    /// rate, is equal to the given amount.
    pub fn assert_vt_balance_converted_to_assets_eq(
        &self,
        address: impl Into<String>,
        amount: impl Into<Uint128>,
    ) -> &Self {
        let vault_token_balance = self.query_vault_token_balance(address);
        let assets = self.query_convert_to_assets(vault_token_balance);
        Self::assert_eq_or_off_by_one(assets, amount);
        self
    }

    /// Asserts that the vault token balance of the given address, when
    /// converted to an amount of base tokens using the current exchange
    /// rate, is greater than the given amount.
    pub fn assert_vt_balance_converted_to_assets_gt(
        &self,
        address: impl Into<String>,
        amount: impl Into<Uint128>,
    ) -> &Self {
        let assets = self.query_convert_to_assets(self.query_vault_token_balance(address));
        assert!(assets > amount.into());
        self
    }

    /// Asserts that the total vault token supply is equal to the given amount
    pub fn assert_total_vault_token_supply_eq(&self, amount: impl Into<Uint128>) -> &Self {
        assert_eq!(self.query_total_vault_token_supply(), amount.into());
        self
    }

    /// Asserts that the total amount of base tokens help by the vault is equal
    /// to the given amount
    pub fn assert_total_vault_assets_eq(&self, amount: impl Into<Uint128>) -> &Self {
        assert_eq!(self.query_total_vault_assets(), amount.into());
        self
    }

    /// Asserts that the unlocking position at the given id `lockup_id` has the
    /// properties `owner`, and `base_token_amount`.
    pub fn assert_unlocking_position_has_props(
        &self,
        lockup_id: u64,
        owner: &str,
        base_token_amount: impl Into<Uint128>,
    ) -> &Self {
        let unlocking_position = self.query_unlocking_position(lockup_id);
        assert_eq!(unlocking_position.id, lockup_id);
        assert_eq!(unlocking_position.owner, owner.to_string());
        assert_eq!(
            unlocking_position.base_token_amount,
            base_token_amount.into()
        );
        self
    }

    pub fn assert_force_withdraw_whitelist_eq(&self, expected: &[&str]) -> &Self {
        let actual: HashSet<String> = self
            .query_force_withdraw_whitelist(None, None)
            .into_iter()
            .collect();
        let expected: HashSet<String> = expected.iter().map(|s| s.to_string()).collect();
        assert_eq!(actual, expected);
        self
    }
}

impl<'a> TestRobot<'a, TestRunner<'a>> for LockedAstroportVaultRobot<'a> {
    fn runner(&self) -> &'a TestRunner<'a> {
        self.runner
    }
}

impl<'a> CwVaultStandardRobot<'a, TestRunner<'a>> for LockedAstroportVaultRobot<'a> {
    fn vault_addr(&self) -> String {
        self.vault_addr.clone()
    }

    fn query_base_token_balance(&self, address: impl Into<String>) -> Uint128 {
        if self.base_token_is_cw20 {
            self.wasm()
                .query::<_, cw20::BalanceResponse>(
                    &self.base_token(),
                    &Cw20QueryMsg::Balance {
                        address: address.into(),
                    },
                )
                .unwrap()
                .balance
        } else {
            self.query_native_token_balance(address, self.base_token())
        }
    }
}

impl<'a> LockedVaultRobot<'a, TestRunner<'a>> for LockedAstroportVaultRobot<'a> {}

impl<'a> ForceUnlockVaultRobot<'a, TestRunner<'a>> for LockedAstroportVaultRobot<'a> {}

impl<'a> AstroportTestRobot<'a, TestRunner<'a>> for LockedAstroportVaultRobot<'a> {
    fn astroport_contracts(&self) -> &AstroportContracts {
        &self.astroport_contracts
    }
}

#[allow(deprecated)]
fn swap_operation(
    pair_addr: &str,
    lp_token: &AssetInfo,
    from: &AssetInfo,
    to: &AssetInfo,
    liquidity_manager: &str,
) -> SwapOperationUnchecked {
    let liquidity_manager = if lp_token.is_native() {
        None
    } else {
        Some(Addr::unchecked(liquidity_manager))
    };

    SwapOperationUnchecked::new(
        #[allow(deprecated)]
        cw_dex_router::pool::Pool::Astroport(cw_dex_astroport::AstroportPool {
            pair_addr: Addr::unchecked(pair_addr),
            lp_token: lp_token.clone(),
            pool_assets: vec![from.clone(), to.clone()],
            pair_type: PairType::Xyk {},
            liquidity_manager,
        }),
        from.into(),
        to.into(),
    )
}

pub fn asset_info_to_astroport_v5_assetinfo(asset: &AssetInfo) -> AstroAssetInfo {
    match asset {
        AssetInfo::Native(denom) => AstroAssetInfo::NativeToken {
            denom: denom.clone(),
        },
        AssetInfo::Cw20(contract_addr) => AstroAssetInfo::Token {
            contract_addr: contract_addr.clone(),
        },
    }
}

pub fn asset_info_from_str(asset: &str, addr_prefix: &str) -> AssetInfo {
    if asset.starts_with(addr_prefix) || asset.starts_with("contract") {
        AssetInfo::Cw20(Addr::unchecked(asset))
    } else {
        AssetInfo::Native(asset.to_string())
    }
}
