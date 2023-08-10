use apollo_cw_asset::AssetInfoUnchecked;
use cosmwasm_schema::cw_serde;
use cosmwasm_std::Decimal;
use cw_dex::astroport::{AstroportPool, AstroportStaking};
use cw_dex::traits::{Pool, Staking};
use cw_dex_router::helpers::CwDexRouterUnchecked;

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

pub type ExecuteMsg = cw_vault_standard::VaultStandardExecuteMsg;
pub type QueryMsg = cw_vault_standard::VaultStandardQueryMsg;
