use cosmwasm_schema::cw_serde;
use cosmwasm_std::Addr;
use cw_it::test_tube::{Account, Module, SigningAccount, Wasm};
use cw_it::traits::CwItRunner;
use cw_it::{ContractType, TestRunner};
use liquidity_helper::LiquidityHelper;

pub struct AstroportLiquidityHelperRobot<'a> {
    pub runner: &'a TestRunner<'a>,
    pub liquidity_helper: LiquidityHelper,
}

#[cw_serde]
struct AstroportLiquidityHelperInstantiateMsg {
    astroport_factory: String,
}

impl<'a> AstroportLiquidityHelperRobot<'a> {
    pub fn new(
        runner: &'a TestRunner<'a>,
        liquidity_helper_contract: ContractType,
        astroport_factory: String,
        signer: &SigningAccount,
    ) -> Self {
        let code_id = runner
            .store_code(liquidity_helper_contract, signer)
            .unwrap();

        let wasm = Wasm::new(runner);
        let addr = wasm
            .instantiate(
                code_id,
                &AstroportLiquidityHelperInstantiateMsg { astroport_factory },
                Some(&signer.address()),
                Some("astroport_liquidity_helper"),
                &[],
                signer,
            )
            .unwrap()
            .data
            .address;

        let liquidity_helper = LiquidityHelper::new(Addr::unchecked(addr));

        Self {
            runner,
            liquidity_helper,
        }
    }
}
