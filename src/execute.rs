use std::time::Instant;

use cosmwasm_schema::serde::{de::DeserializeOwned, Serialize};
use cosmwasm_std::{coin, CosmosMsg, Decimal, DepsMut, Env, MessageInfo, Response, Uint128};
use cw_storage_plus::Item;
use osmosis_std::types::osmosis::tokenfactory::v1beta1::{MsgBurn, MsgMint};

use crate::{
    error::{ContractResponse, ContractResult},
    msg::InstantiateMsg,
    state::{self, Config, VaultState},
    utils,
};

use cw_dex::traits::{Pool, Staking};

pub struct VaultContract<'a, P, S> {
    pub config: Item<'a, Config>,
    pub vault_token_supply: Item<'a, Uint128>,
    pub staked_base_tokens: Item<'a, Uint128>,
    pub pool: Item<'a, P>,
    pub staking: Item<'a, S>,
}

impl<P, S> Default for VaultContract<'_, P, S> {
    fn default() -> Self {
        Self {
            config: Item::new("config"),
            vault_token_supply: Item::new("vault_token_supply"),
            staked_base_tokens: Item::new("staked_base_tokens"),
            pool: Item::new("pool"),
            staking: Item::new("staking"),
        }
    }
}

impl<'a, P: Pool + Serialize + DeserializeOwned, S: Staking + Serialize + DeserializeOwned>
    VaultContract<'a, P, S>
{
    pub fn instantiate(
        &self,
        deps: DepsMut,
        env: Env,
        info: MessageInfo,
        msg: InstantiateMsg<P, S>,
    ) -> ContractResponse {
        cw_ownable::initialize_owner(deps.storage, deps.api, Some(&msg.owner))?;

        let vault_token_denom = format!(
            "factory/{}/{}",
            env.contract.address, msg.vault_token_subdenom
        );

        let cw20_adaptor = match msg.cw20_adaptor {
            Some(adaptor) => Some(deps.api.addr_validate(&adaptor)?),
            None => None,
        };

        let config = Config {
            base_token_denom: msg.base_token_denom,
            vault_token_denom,
            cw20_adaptor,
        };

        self.config.save(deps.storage, &config)?;
        self.pool.save(deps.storage, &msg.pool)?;
        self.staking.save(deps.storage, &msg.staking)?;

        Ok(Response::new())
    }

    pub fn execute_deposit(&self, deps: DepsMut, env: Env, info: MessageInfo) -> ContractResponse {
        let cfg = self.config.load(deps.storage)?;

        let deposit_amount = utils::one_coin(info, cfg.base_token_denom)?.amount;

        let staking = self.staking.load(deps.storage)?;

        Ok(staking
            .stake(deps.as_ref(), &env, deposit_amount)?
            .add_message(self.mint_vault_tokens(deps, env, deposit_amount)?))
    }

    fn mint_vault_tokens(
        &self,
        deps: DepsMut,
        env: Env,
        deposit_amount: Uint128,
    ) -> ContractResult<CosmosMsg> {
        let staked_base_tokens = self.staked_base_tokens.load(deps.storage)?;
        let vault_token_supply = self.vault_token_supply.load(deps.storage)?;
        let cfg = self.config.load(deps.storage)?;

        let mint_amount =
            Decimal::from_ratio(deposit_amount, staked_base_tokens) * vault_token_supply;

        self.staked_base_tokens.save(
            deps.storage,
            &staked_base_tokens.checked_add(deposit_amount)?,
        )?;
        self.vault_token_supply
            .save(deps.storage, &vault_token_supply.checked_add(mint_amount)?)?;

        Ok(MsgMint {
            sender: env.contract.address.to_string(),
            amount: Some(coin(mint_amount.u128(), &cfg.vault_token_denom).into()),
        }
        .into())
    }
}
