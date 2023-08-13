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
    astroport::utils::{create_astroport_pair, AstroportContracts},
    robot::TestRobot,
    test_tube::{Account, Module, SigningAccount, Wasm},
    traits::CwItRunner,
    Artifact, ContractType, TestRunner,
};
use cw_vault_standard_test_helpers::traits::{
    force_unlock::ForceUnlockVaultRobot, lockup::LockedVaultRobot, CwVaultStandardRobot,
};
use liquidity_helper::LiquidityHelperUnchecked;
use locked_astroport_vault::msg::InstantiateMsg;

use crate::router::CwDexRouterRobot;

const CW_DEX_ROUTER_WASM_PATH: &str = "artifacts/cw_dex_router.wasm";
const ASTROPORT_LIQUIDITY_HELPER_WASM_PATH: &str = "artifacts/astroport_liquidity_helper.wasm";
const ASTROPORT_ARTIFACTS_DIR: &str = "artifacts/astroport-artifacts";

const TWO_WEEKS_IN_SECS: u64 = 1_209_600;

const WSTETH_DENOM: &str = "uwsteth";
const ETH_DENOM: &str = "ueth";
const ASTRO_DENOM: &str = "uastro";
const USDC_DENOM: &str = "uusdc";
const AXL_DENOM: &str = "uaxl";
const NTRN_DENOM: &str = "untrn";

#[cw_serde]
struct AstroportLiquidityHelperInstantiateMsg {
    astroport_factory: String,
}

pub struct LockedAstroportVaultRobot<'a> {
    pub runner: &'a TestRunner<'a>,
    pub vault_addr: String,
}

impl<'a> LockedAstroportVaultRobot<'a> {
    pub fn new_wsteth_eth_vault(
        runner: &'a TestRunner<'a>,
        vault_contract: ContractType,
        token_factory_fee: Coin,
        treasury_addr: String,
        signer: &SigningAccount,
    ) -> Self {
        let wsteth = AssetInfo::native(WSTETH_DENOM.to_string());
        let eth = AssetInfo::native(ETH_DENOM.to_string());
        let astro = AssetInfo::native(ASTRO_DENOM.to_string());
        let usdc = AssetInfo::native(USDC_DENOM.to_string());
        let axl = AssetInfo::native(AXL_DENOM.to_string());
        let ntrn = AssetInfo::native(NTRN_DENOM.to_string());

        // Deploy astroport contracts
        let astroport_contracts = AstroportContracts::new_from_local_contracts(
            runner,
            &Some(ASTROPORT_ARTIFACTS_DIR),
            false,
            &None,
            signer,
        );

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

        let (axl_usdc_pair, axl_usdc_lp) = create_astroport_pair(
            runner,
            &astroport_contracts.factory.address,
            PairType::Xyk {},
            [axl.clone().into(), usdc.clone().into()],
            None,
            signer,
            Some([
                Uint128::from(1_000_000_000u128),
                Uint128::from(1_000_000_000u128),
            ]),
        );

        // Create CwDexRouterRobot
        // TODO: Support multi-test
        let cw_dex_router_robot = CwDexRouterRobot::new(
            runner,
            ContractType::Artifact(Artifact::Local(CW_DEX_ROUTER_WASM_PATH.to_string())),
            signer,
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

        // NTRN <-> USDC
        cw_dex_router_robot.set_path(
            ntrn.clone().into(),
            usdc.clone().into(),
            SwapOperationsListUnchecked::new(vec![swap_operation(
                &ntrn_usdc_pair,
                &ntrn_usdc_lp,
                &ntrn,
                &usdc,
            )]),
            true,
            signer,
        );

        // ASTRO <-> USDC
        cw_dex_router_robot.set_path(
            astro.clone().into(),
            usdc.clone().into(),
            SwapOperationsListUnchecked::new(vec![swap_operation(
                &astro_usdc_pair,
                &astro_usdc_lp,
                &astro,
                &usdc,
            )]),
            true,
            signer,
        );

        // ETH <-> USDC
        cw_dex_router_robot.set_path(
            eth.clone().into(),
            usdc.clone().into(),
            SwapOperationsListUnchecked::new(vec![swap_operation(
                &eth_usdc_pair,
                &eth_usdc_lp,
                &eth,
                &usdc,
            )]),
            true,
            signer,
        );

        // AXL <-> USDC
        cw_dex_router_robot.set_path(
            axl.clone().into(),
            usdc.clone().into(),
            SwapOperationsListUnchecked::new(vec![swap_operation(
                &axl_usdc_pair,
                &axl_usdc_lp,
                &axl,
                &usdc,
            )]),
            true,
            signer,
        );

        let wasm = Wasm::new(runner);
        let code_id = runner
            .store_code(
                ContractType::Artifact(Artifact::Local(
                    ASTROPORT_LIQUIDITY_HELPER_WASM_PATH.to_string(),
                )),
                signer,
            )
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

        let init_msg = InstantiateMsg {
            owner: signer.address().to_string(),
            vault_token_subdenom: "testVaultToken".to_string(),
            lock_duration: TWO_WEEKS_IN_SECS,
            reward_tokens: vec![astro.into(), usdc.into(), ntrn.into()],
            deposits_enabled: true,
            treasury: treasury_addr,
            performance_fee: Decimal::percent(3),
            router: cw_dex_router_robot.into(),
            reward_liquidation_target: eth.into(),
            pool_addr: wsteth_eth_pair,
            astro_token: apollo_cw_asset::AssetInfoUnchecked::native("uastro"),
            astroport_generator: astroport_contracts.generator.address,
            liquidity_helper: LiquidityHelperUnchecked::new(liquidity_helper_addr),
        };

        Self::new_with_instantiate_msg(runner, vault_contract, token_factory_fee, &init_msg, signer)
    }

    pub fn new_with_instantiate_msg(
        runner: &'a TestRunner<'a>,
        vault_contract: ContractType,
        token_factory_fee: Coin,
        instantiate_msg: &InstantiateMsg,
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

        Self { runner, vault_addr }
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

// TODO: figure out how to refactor to get around the need to create the AsrtoportPool like this.
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
