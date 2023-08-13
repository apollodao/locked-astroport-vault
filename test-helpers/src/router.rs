use apollo_cw_asset::{Asset, AssetInfo, AssetInfoUnchecked};
use cosmwasm_std::{Addr, Coin};
use cw_dex_router::{
    helpers::{CwDexRouter, CwDexRouterUnchecked},
    msg::InstantiateMsg,
    operations::SwapOperationsListUnchecked,
};
use cw_it::{
    test_tube::{Account, Module, SigningAccount, Wasm},
    traits::CwItRunner,
    ContractType, TestRunner,
};

pub struct CwDexRouterRobot<'a> {
    pub runner: &'a TestRunner<'a>,
    pub cw_dex_router: CwDexRouter,
}

impl<'a> CwDexRouterRobot<'a> {
    pub fn new(
        runner: &'a TestRunner<'a>,
        contract: ContractType,
        signer: &SigningAccount,
    ) -> Self {
        let code_id = runner.store_code(contract, signer).unwrap();

        let wasm = Wasm::new(runner);
        let router_addr = wasm
            .instantiate(
                code_id,
                &InstantiateMsg {},
                Some(&signer.address()),
                Some("cw_dex_router"),
                &[],
                signer,
            )
            .unwrap()
            .data
            .address;

        let cw_dex_router = CwDexRouter::new(&Addr::unchecked(router_addr));

        Self {
            runner,
            cw_dex_router,
        }
    }

    pub fn set_path(
        &self,
        from: AssetInfoUnchecked,
        to: AssetInfoUnchecked,
        path: SwapOperationsListUnchecked,
        bidirectional: bool,
        signer: &SigningAccount,
    ) {
        let wasm = Wasm::new(self.runner);
        wasm.execute(
            &self.cw_dex_router.0.to_string(),
            &cw_dex_router::msg::ExecuteMsg::SetPath {
                offer_asset: from,
                ask_asset: to,
                path,
                bidirectional,
            },
            &[],
            signer,
        )
        .unwrap();
    }
}

impl<'a> From<CwDexRouterRobot<'a>> for CwDexRouter {
    fn from(value: CwDexRouterRobot) -> Self {
        value.cw_dex_router
    }
}

impl<'a> From<CwDexRouterRobot<'a>> for CwDexRouterUnchecked {
    fn from(value: CwDexRouterRobot) -> Self {
        value.cw_dex_router.into()
    }
}
