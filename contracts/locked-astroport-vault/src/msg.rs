use apollo_cw_asset::AssetInfoUnchecked;
use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::{to_json_binary, Addr, Coin, CosmosMsg, Env, StdResult, Uint128};
use cw_dex_router::helpers::CwDexRouterUnchecked;
use cw_ownable::Action as OwnerAction;
use cw_vault_standard::extensions::force_unlock::ForceUnlockExecuteMsg;
use cw_vault_standard::extensions::lockup::{LockupExecuteMsg, LockupQueryMsg};
use liquidity_helper::LiquidityHelperUnchecked;
use strum::{EnumCount, EnumVariantNames};

use crate::helpers::IntoInternalCall;
use crate::state::{ConfigUpdates, FeeConfig};

#[cw_serde]
pub struct InstantiateMsg {
    /// Contract owner
    pub owner: String,
    /// Vault token sub-denom
    pub vault_token_subdenom: String,
    /// Address of the pool.
    pub pool_addr: String,
    /// Astroport token
    pub astro_token: AssetInfoUnchecked,
    /// Astroport generator address
    pub astroport_generator: String,
    /// Lock duration in seconds
    pub lock_duration: u64,
    /// Reward tokens
    pub reward_tokens: Vec<AssetInfoUnchecked>,
    /// Whether or not deposits are enabled
    pub deposits_enabled: bool,
    /// The router contract address
    pub router: CwDexRouterUnchecked,
    /// The asset to which we should swap reward_assets into before providing
    /// liquidity. Should be one of the assets in the pool.
    pub reward_liquidation_target: AssetInfoUnchecked,
    /// Helper for providing liquidity with unbalanced assets.
    pub liquidity_helper: LiquidityHelperUnchecked,
    /// The address of the astroport liquidity manager contract.
    pub astroport_liquidity_manager: String,
    /// The fee that is taken on rewards accrued
    pub performance_fee: Option<FeeConfig<String>>,
    /// A fee that is taken on deposits
    pub deposit_fee: Option<FeeConfig<String>>,
    /// A fee that is taken on withdrawals
    pub withdrawal_fee: Option<FeeConfig<String>>,
}

#[cw_serde]
#[derive(EnumCount)]
pub enum InternalMsg {
    /// Sell reward tokens
    SellTokens {},
    /// Provide liquidity to the pool
    ProvideLiquidity {},
    /// Stake LP tokens
    StakeLps {},
    /// Deposit into the vault after compounding
    Deposit {
        /// The amount of base tokens to deposit.
        amount: Uint128,
        /// The original caller of the contract. We can't use info.sender, since
        /// this is an internal call, so that would be the contract
        /// itself.
        depositor: Addr,
        /// The recipient of the vault token.
        recipient: Addr,
    },
    Redeem {
        /// The address which should receive the withdrawn base tokens.
        recipient: Addr,
        /// The amount of vault tokens sent to the contract.
        amount: Uint128,
    },
}

impl IntoInternalCall for InternalMsg {
    fn into_internal_call(self, env: &Env, funds: Vec<Coin>) -> StdResult<CosmosMsg> {
        ExtensionExecuteMsg::into_internal_call(ExtensionExecuteMsg::Internal(self), env, funds)
    }
}

/// Apollo extension messages define functionality that is part of all apollo
/// vaults, but not part of the vault standard.
#[cw_serde]
pub enum ApolloExtensionExecuteMsg {
    /// Update the configuration of the vault.
    UpdateConfig {
        // The config updates.
        updates: ConfigUpdates<String>,
    },
    /// Compounds the vault
    Compound {},
}

impl IntoInternalCall for ApolloExtensionExecuteMsg {
    fn into_internal_call(self, env: &Env, funds: Vec<Coin>) -> StdResult<CosmosMsg> {
        ExtensionExecuteMsg::into_internal_call(ExtensionExecuteMsg::Apollo(self), env, funds)
    }
}

#[cw_serde]
#[derive(EnumVariantNames)]
#[strum(serialize_all = "kebab-case")]
pub enum ExtensionExecuteMsg {
    /// Execute an internal message (can only be called by the contract itself
    Internal(InternalMsg),

    /// Execute a message from the lockup extension.
    Lockup(LockupExecuteMsg),

    /// Execute a message from the force unlock extension.
    ForceUnlock(ForceUnlockExecuteMsg),

    /// Execute an Apollo extension message.
    Apollo(ApolloExtensionExecuteMsg),

    /// Execute an Owner extension message.
    UpdateOwnership(OwnerAction),
}

impl IntoInternalCall for ExtensionExecuteMsg {
    fn into_internal_call(self, env: &Env, funds: Vec<Coin>) -> StdResult<CosmosMsg> {
        Ok(CosmosMsg::Wasm(cosmwasm_std::WasmMsg::Execute {
            contract_addr: env.contract.address.to_string(),
            msg: to_json_binary(&ExecuteMsg::VaultExtension(self))?,
            funds,
        }))
    }
}

#[cw_ownable::cw_ownable_query]
#[cw_serde]
#[derive(QueryResponses)]
pub enum ApolloExtensionQueryMsg {
    /// Returns the current config.
    #[returns(crate::state::Config)]
    Config {},

    /// Returns the current version of the contract.
    #[returns(cw2::ContractVersion)]
    ContractVersion {},

    /// Returns the list of addresses that are whitelisted for force withdrawal.
    #[returns(Vec<Addr>)]
    ForceWithdrawWhitelist {
        start_after: Option<String>,
        limit: Option<u32>,
    },

    /// Returns the non-configurable state of the contract.
    #[returns(crate::state::StateResponse)]
    State {},
}

#[cw_serde]
pub enum ExtensionQueryMsg {
    /// Execute an Apollo extension query.
    Apollo(ApolloExtensionQueryMsg),

    /// Execute a message from the lockup extension.
    Lockup(LockupQueryMsg),
}

pub type ExecuteMsg = cw_vault_standard::VaultStandardExecuteMsg<ExtensionExecuteMsg>;

pub type QueryMsg = cw_vault_standard::VaultStandardQueryMsg<ExtensionQueryMsg>;
