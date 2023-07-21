use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, Uint128};

#[cw_serde]
pub struct Config {
    pub base_token_denom: String,
    pub vault_token_denom: String,
    pub cw20_adaptor: Option<Addr>,
}

#[cw_serde]
pub struct VaultState {
    pub vault_token_supply: Uint128,
    pub staked_base_tokens: Uint128,
}
