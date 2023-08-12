use cosmwasm_std::{Coin, Uint128};
use cw20::Cw20QueryMsg;
use cw_it::{
    robot::TestRobot,
    test_tube::{Account, Module, SigningAccount, Wasm},
    traits::CwItRunner,
    ContractType, TestRunner,
};
use cw_vault_standard_test_helpers::traits::CwVaultStandardRobot;
use locked_astroport_vault::msg::InstantiateMsg;

pub struct LockedAstroportVaultRobot<'a> {
    pub runner: &'a TestRunner<'a>,
    pub vault_addr: String,
}

impl<'a> LockedAstroportVaultRobot<'a> {
    pub fn new(
        runner: &'a TestRunner<'a>,
        vault_contract: ContractType,
        token_factory_fee: Coin,
        signer: &SigningAccount,
    ) -> Self {
        let code_id = runner.store_code(vault_contract, signer).unwrap();

        let wasm = Wasm::new(runner);
        let vault_addr = wasm
            .instantiate(
                code_id,
                &InstantiateMsg {
                    owner: signer.address().to_string(),
                    base_token_addr: todo!(),
                    vault_token_subdenom: todo!(),
                    pool: todo!(),
                    staking: todo!(),
                    lock_duration: todo!(),
                    reward_tokens: todo!(),
                    deposits_enabled: todo!(),
                    treasury: todo!(),
                    performance_fee: todo!(),
                    router: todo!(),
                    reward_liquidation_target: todo!(),
                },
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
