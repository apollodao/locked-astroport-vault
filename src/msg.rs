use apollo_cw_asset::AssetInfoUnchecked;
use cosmwasm_schema::cw_serde;
use cw_dex::astroport::{AstroportPool, AstroportStaking};
use cw_dex::traits::{Pool, Staking};

#[cw_serde]
pub struct InstantiateMsg {
    /// Contract owner
    pub owner: String,
    /// Denom of vault base token
    pub base_token_addr: String,
    /// Vault token sub-denom
    pub vault_token_subdenom: String,
    /// Cw20-adaptor contract address.
    pub cw20_adaptor: Option<String>,
    /// Type implementing [`cw_dex::traits::Pool`]
    pub pool: AstroportPool,
    /// Type implementing [`cw_dex::traits::Staking`]
    pub staking: AstroportStaking,
    /// Lock duration in seconds
    pub lock_duration: u64,
    /// Reward tokens
    pub reward_tokens: Vec<AssetInfoUnchecked>,
}

pub type ExecuteMsg = cw_vault_standard::VaultStandardExecuteMsg;
pub type QueryMsg = cw_vault_standard::VaultStandardQueryMsg;
