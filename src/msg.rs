use cosmwasm_schema::cw_serde;
use cw_dex::{
    astroport::{AstroportPool, AstroportStaking},
    traits::{Pool, Staking},
};

#[cw_serde]
pub struct InstantiateMsg {
    /// Contract owner
    pub owner: String,
    /// Denom of vault base token
    pub base_token_denom: String,
    /// Vault token sub-denom
    pub vault_token_subdenom: String,
    /// Cw20-adaptor contract address.
    pub cw20_adaptor: Option<String>,
    /// Type implementing [`cw_dex::traits::Pool`]
    pub pool: AstroportPool,
    /// Type implementing [`cw_dex::traits::Staking`]
    pub staking: AstroportStaking,
}

pub type ExecuteMsg = cw_vault_standard::VaultStandardExecuteMsg;
pub type QueryMsg = cw_vault_standard::VaultStandardQueryMsg;
