use apollo_cw_asset::AssetInfoUnchecked;
use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Decimal, Uint128};
use cw_dex::astroport::{AstroportPool, AstroportStaking};
use cw_dex_router::helpers::CwDexRouterUnchecked;
use cw_ownable::Action as OwnerAction;
use cw_vault_standard::extensions::{
    force_unlock::ForceUnlockExecuteMsg, lockup::LockupExecuteMsg,
};

#[cw_serde]
pub struct InstantiateMsg {
    /// Contract owner
    pub owner: String,
    /// Denom of vault base token
    pub base_token_addr: String,
    /// Vault token sub-denom
    pub vault_token_subdenom: String,
    /// Type implementing [`cw_dex::traits::Pool`]
    pub pool: AstroportPool,
    /// Type implementing [`cw_dex::traits::Staking`]
    pub staking: AstroportStaking,
    /// Lock duration in seconds
    pub lock_duration: u64,
    /// Reward tokens
    pub reward_tokens: Vec<AssetInfoUnchecked>,
    /// Whether or not deposits are enabled
    pub deposits_enabled: bool,
    /// The treasury address to send fees to
    pub treasury: String,
    /// The fee that is taken on rewards accrued
    pub performance_fee: Decimal,
    /// The router contract address
    pub router: CwDexRouterUnchecked,
    /// The asset to which we should swap reward_assets into before providing
    /// liquidity. Should be one of the assets in the pool.
    pub reward_liquidation_target: AssetInfoUnchecked,
}

#[cw_serde]
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
        /// The optional recipient of the vault token. If not set, the caller
        /// address will be used instead.
        recipient: Option<String>,
    },
    Redeem {
        /// An optional field containing which address should receive the
        /// withdrawn base tokens. If not set, the caller address will be
        /// used instead.
        recipient: Option<String>,
        /// The amount of vault tokens sent to the contract.
        amount: Uint128,
    },
}

/// Apollo extension messages define functionality that is part of all apollo
/// vaults, but not part of the vault standard.
#[cw_serde]
pub enum ApolloExtensionExecuteMsg {
    /// Update the configuration of the vault.
    UpdateConfig {
        // The config updates.
        // updates: ConfigUpdates,
    },
    /// Compounds the vault
    Compound {},
}

#[cw_serde]
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

pub type ExecuteMsg = cw_vault_standard::VaultStandardExecuteMsg<ExtensionExecuteMsg>;
pub type QueryMsg = cw_vault_standard::VaultStandardQueryMsg;
