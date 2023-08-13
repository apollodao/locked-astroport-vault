use cosmwasm_std::{Coin, Uint128};
use cw20::Cw20QueryMsg;
use cw_dex::astroport::{self, AstroportPool, AstroportStaking};
use cw_it::{
    astroport::utils::AstroportContracts,
    robot::TestRobot,
    test_tube::{Account, Module, SigningAccount, Wasm},
    traits::CwItRunner,
    ContractType, TestRunner,
};
use cw_vault_standard_test_helpers::traits::CwVaultStandardRobot;
use locked_astroport_vault::msg::InstantiateMsg;

const TWO_WEEKS_IN_SECS: u64 = 1_209_600;

pub struct LockedAstroportVaultRobot<'a> {
    pub runner: &'a TestRunner<'a>,
    pub vault_addr: String,
}

impl<'a> LockedAstroportVaultRobot<'a> {
    pub fn new_default(
        runner: &'a TestRunner<'a>,
        vault_contract: ContractType,
        token_factory_fee: Coin,
        signer: &SigningAccount,
    ) -> Self {
        let astroport_contracts = AstroportContracts::new_from_local_contracts(runner);
        let init_msg = InstantiateMsg {
            owner: signer.address().to_string(),
            base_token_addr: todo!(),
            vault_token_subdenom: "testVaultToken".to_string(),
            pool: AstroportPool {
                pair_addr: todo!(),
                lp_token_addr: todo!(),
                pool_assets: todo!(),
                pair_type: todo!(),
            },
            staking: AstroportStaking {
                lp_token_addr: todo!(),
                generator_addr: todo!(),
                astro_token: todo!(),
            },
            lock_duration: TWO_WEEKS_IN_SECS,
            reward_tokens: todo!(),
            deposits_enabled: todo!(),
            treasury: todo!(),
            performance_fee: todo!(),
            router: todo!(),
            reward_liquidation_target: todo!(),
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
