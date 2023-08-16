use apollo_cw_asset::AssetInfo;
use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, Coin, Decimal, Uint128};
use cw20::Cw20QueryMsg;
use cw_dex::{
    astroport::{astroport::factory::PairType, AstroportPool},
    Pool,
};
use cw_dex_router::operations::{SwapOperationUnchecked, SwapOperationsListUnchecked};
use cw_it::{
    astroport::{
        robot::AstroportTestRobot,
        utils::{create_astroport_pair, AstroportContracts},
    },
    cw_multi_test::ContractWrapper,
    robot::TestRobot,
    test_tube::{Account, Module, SigningAccount, Wasm},
    traits::CwItRunner,
    Artifact, ContractType, TestRunner,
};
use cw_ownable::Ownership;
use cw_vault_standard_test_helpers::traits::{
    force_unlock::ForceUnlockVaultRobot, lockup::LockedVaultRobot, CwVaultStandardRobot,
};
use liquidity_helper::LiquidityHelperUnchecked;
use locked_astroport_vault;
use locked_astroport_vault::msg::{
    ApolloExtensionExecuteMsg, ApolloExtensionQueryMsg, ExecuteMsg, ExtensionExecuteMsg,
    ExtensionQueryMsg, InstantiateMsg, QueryMsg,
};
use locked_astroport_vault::state::ConfigUpdates;

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
    pub dependencies: &'a LockedVaultDependencies<'a>,
}

impl<'a> LockedAstroportVaultRobot<'a> {
    /// Returns a `ContractType` representing a local wasm file of the contract.
    /// If `artifacts_dir` is `None`, the default path of `artifacts` will be used.
    pub fn wasm_contract(artifacts_dir: Option<&str>) -> ContractType {
        let dir = artifacts_dir.unwrap_or_else(|| "artifacts").to_string();
        let path = format!("{}/{}", dir, LOCKED_ASTROPORT_VAULT_WASM_NAME);
        println!("Loading contract from {}", path);
        ContractType::Artifact(Artifact::Local(path))
    }

    /// Returns a `ContractType` representing a multi-test contract of the contract.
    pub fn multitest_contract() -> ContractType {
        ContractType::MultiTestContract(Box::new(
            ContractWrapper::new_with_empty(
                locked_astroport_vault::contract::execute,
                locked_astroport_vault::contract::instantiate,
                locked_astroport_vault::contract::query,
            )
            .with_reply(locked_astroport_vault::contract::reply),
        ))
    }

    /// Returns a `ContractType` representing the contract to use for the given `TestRunner`.
    pub fn contract(runner: &'a TestRunner<'a>, _artifacts_dir: Option<&str>) -> ContractType {
        match runner {
            #[cfg(feature = "osmosis-test-tube")]
            TestRunner::OsmosisTestApp(_) => Self::wasm_contract(_artifacts_dir),
            TestRunner::MultiTest(_) => Self::multitest_contract(),
            _ => panic!("Unsupported runner"),
        }
    }

    // Uploads and instantiates dependencies for the LockedAstroportVaultRobot.
    pub fn instantiate_deps(
        runner: &'a TestRunner<'a>,
        signer: &SigningAccount,
        artifacts_dir: Option<&str>,
    ) -> LockedVaultDependencies<'a> {
        let artifacts_dir = artifacts_dir.unwrap_or_else(|| "tests/test_artifacts");

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
            CwDexRouterRobot::contract(&runner, Some(artifacts_dir)),
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

    /// Creates a new `LockedAstroportVaultRobot` from the given `InstantiateMsg`.
    pub fn new_with_instantiate_msg(
        runner: &'a TestRunner<'a>,
        vault_contract: ContractType,
        token_factory_fee: Coin,
        instantiate_msg: &InstantiateMsg,
        dependencies: &'a LockedVaultDependencies<'a>,
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

        Self {
            runner,
            vault_addr,
            dependencies,
        }
    }

    /// Creates a AXL/NTRN pool and a new LockedAstroportVaultRobot for the pool with no lockup.
    pub fn new_unlocked_axlr_ntrn_vault(
        runner: &'a TestRunner<'a>,
        vault_contract: ContractType,
        token_factory_fee: Coin,
        treasury_addr: String,
        dependencies: &'a LockedVaultDependencies<'a>,
        signer: &SigningAccount,
    ) -> Self {
        let axl = AssetInfo::native(AXL_DENOM.to_string());
        let ntrn = AssetInfo::native(NTRN_DENOM.to_string());
        let astro = AssetInfo::native(ASTRO_DENOM.to_string());

        // Create AXL/NTRN astroport pool
        let (axl_ntrn_pair, _axl_ntrn_lp) = create_astroport_pair(
            runner,
            &dependencies.astroport_contracts.factory.address,
            PairType::Xyk {},
            [axl.clone().into(), ntrn.clone().into()],
            None,
            signer,
            Some([
                Uint128::from(1_000_000_000u128),
                Uint128::from(1_000_000_000u128),
            ]),
        );

        let instantiate_msg = InstantiateMsg {
            owner: signer.address().to_string(),
            vault_token_subdenom: "testVaultToken".to_string(),
            lock_duration: 0u64,
            reward_tokens: vec![astro.into(), axl.into(), ntrn.clone().into()],
            deposits_enabled: true,
            treasury: treasury_addr,
            performance_fee: Decimal::percent(3),
            router: dependencies
                .cw_dex_router_robot
                .cw_dex_router
                .clone()
                .into(),
            reward_liquidation_target: ntrn.into(),
            pool_addr: axl_ntrn_pair,
            astro_token: apollo_cw_asset::AssetInfoUnchecked::native("uastro"),
            astroport_generator: dependencies.astroport_contracts.generator.address.clone(),
            liquidity_helper: LiquidityHelperUnchecked::new(
                dependencies.liquidity_helper_addr.clone(),
            ),
        };

        Self::new_with_instantiate_msg(
            runner,
            vault_contract,
            token_factory_fee,
            &instantiate_msg,
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
        treasury_addr: String,
        dependencies: &'a LockedVaultDependencies<'a>,
        signer: &SigningAccount,
    ) -> Self {
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

        // Create astroport pairs
        let (wsteth_eth_pair, wsteth_eth_lp) = create_astroport_pair(
            runner,
            &astroport_contracts.factory.address,
            PairType::Xyk {},
            [wsteth.clone().into(), eth.clone().into()],
            None,
            signer,
            Some([
                Uint128::from(1_000_000_000u128),
                Uint128::from(1_000_000_000u128),
            ]),
        );
        let (astro_usdc_pair, astro_usdc_lp) = create_astroport_pair(
            runner,
            &astroport_contracts.factory.address,
            PairType::Xyk {},
            [astro.clone().into(), usdc.clone().into()],
            None,
            signer,
            Some([
                Uint128::from(1_000_000_000u128),
                Uint128::from(1_000_000_000u128),
            ]),
        );
        let (ntrn_usdc_pair, ntrn_usdc_lp) = create_astroport_pair(
            runner,
            &astroport_contracts.factory.address,
            PairType::Xyk {},
            [ntrn.clone().into(), usdc.clone().into()],
            None,
            signer,
            Some([
                Uint128::from(1_000_000_000u128),
                Uint128::from(1_000_000_000u128),
            ]),
        );
        let (eth_usdc_pair, eth_usdc_lp) = create_astroport_pair(
            runner,
            &astroport_contracts.factory.address,
            PairType::Xyk {},
            [eth.clone().into(), usdc.clone().into()],
            None,
            signer,
            Some([
                Uint128::from(1_000_000_000u128),
                Uint128::from(1_000_000_000u128),
            ]),
        );
        let (axl_ntrn_pair, axl_ntrn_lp) = create_astroport_pair(
            runner,
            &astroport_contracts.factory.address,
            PairType::Xyk {},
            [axl.clone().into(), ntrn.clone().into()],
            None,
            signer,
            Some([
                Uint128::from(1_000_000_000u128),
                Uint128::from(1_000_000_000u128),
            ]),
        );

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
            )]),
            true,
            signer,
        );
        // NTRN <-> USDC <-> ETH
        cw_dex_router_robot.set_path(
            ntrn.clone().into(),
            eth.clone().into(),
            SwapOperationsListUnchecked::new(vec![
                swap_operation(&ntrn_usdc_pair, &ntrn_usdc_lp, &ntrn, &usdc),
                swap_operation(&eth_usdc_pair, &eth_usdc_lp, &usdc, &eth),
            ]),
            true,
            signer,
        );
        // ASTRO <-> USDC <-> ETH
        cw_dex_router_robot.set_path(
            astro.clone().into(),
            eth.clone().into(),
            SwapOperationsListUnchecked::new(vec![
                swap_operation(&astro_usdc_pair, &astro_usdc_lp, &astro, &usdc),
                swap_operation(&eth_usdc_pair, &eth_usdc_lp, &usdc, &eth),
            ]),
            true,
            signer,
        );
        // AXL <-> USDC <-> ETH
        cw_dex_router_robot.set_path(
            axl.clone().into(),
            eth.clone().into(),
            SwapOperationsListUnchecked::new(vec![
                swap_operation(&axl_ntrn_pair, &axl_ntrn_lp, &axl, &usdc),
                swap_operation(&eth_usdc_pair, &eth_usdc_lp, &usdc, &eth),
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
            treasury: treasury_addr,
            performance_fee: Decimal::percent(3),
            router: dependencies
                .cw_dex_router_robot
                .cw_dex_router
                .clone()
                .into(),
            reward_liquidation_target: eth.into(),
            pool_addr: wsteth_eth_pair,
            astro_token: apollo_cw_asset::AssetInfoUnchecked::native("uastro"),
            astroport_generator: astroport_contracts.generator.address.clone(),
            liquidity_helper: LiquidityHelperUnchecked::new(liquidity_helper_addr.clone()),
        };

        Self::new_with_instantiate_msg(
            runner,
            vault_contract,
            token_factory_fee,
            &init_msg,
            dependencies,
            signer,
        )
    }

    /// Increase CW20 allowance and deposit into the vault.
    pub fn deposit_cw20(
        &self,
        amount: Uint128,
        recipient: Option<String>,
        signer: &SigningAccount,
    ) -> &Self {
        self.increase_cw20_allowance(&self.base_token(), &self.vault_addr, amount, &signer)
            .deposit(amount, recipient, &[], signer)
    }

    /// Update the config of the vault and return a reference to the robot.
    pub fn update_config(&self, updates: ConfigUpdates<String>, signer: &SigningAccount) -> &Self {
        self.wasm()
            .execute(
                &self.vault_addr,
                &ExecuteMsg::VaultExtension(ExtensionExecuteMsg::Apollo(
                    ApolloExtensionExecuteMsg::UpdateConfig { updates },
                )),
                &[],
                signer,
            )
            .unwrap();
        self
    }

    // Queries //

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
}

impl<'a> TestRobot<'a, TestRunner<'a>> for LockedAstroportVaultRobot<'a> {
    fn runner(&self) -> &'a TestRunner<'a> {
        &self.runner
    }
}

impl<'a> CwVaultStandardRobot<'a, TestRunner<'a>> for LockedAstroportVaultRobot<'a> {
    fn vault_addr(&self) -> String {
        self.vault_addr.clone()
    }

    fn query_base_token_balance(&self, address: impl Into<String>) -> Uint128 {
        self.wasm()
            .query::<_, cw20::BalanceResponse>(
                &self.base_token(),
                &Cw20QueryMsg::Balance {
                    address: address.into(),
                },
            )
            .unwrap()
            .balance
    }
}

impl<'a> LockedVaultRobot<'a, TestRunner<'a>> for LockedAstroportVaultRobot<'a> {}

impl<'a> ForceUnlockVaultRobot<'a, TestRunner<'a>> for LockedAstroportVaultRobot<'a> {}

impl<'a> AstroportTestRobot<'a, TestRunner<'a>> for LockedAstroportVaultRobot<'a> {
    fn astroport_contracts(&self) -> &AstroportContracts {
        &self.dependencies.astroport_contracts
    }
}

// TODO: figure out how to refactor to get around the need to create the AstroportPool like this.
// should probably take an unchecked pool as argument, which would mean we need to change the
// cw-dex-router message to take an unchecked pool as well.
fn swap_operation(
    pair_addr: &str,
    lp_addr: &str,
    from: &AssetInfo,
    to: &AssetInfo,
) -> SwapOperationUnchecked {
    SwapOperationUnchecked::new(
        Pool::Astroport(AstroportPool {
            pair_addr: Addr::unchecked(pair_addr),
            lp_token_addr: Addr::unchecked(lp_addr),
            pool_assets: vec![from.clone(), to.clone()],
            pair_type: PairType::Xyk {},
        }),
        from.into(),
        to.into(),
    )
}
