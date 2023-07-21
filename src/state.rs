use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, Uint128};
use cw_dex::astroport::{AstroportPool, AstroportStaking};
use cw_storage_plus::Item;

pub const CONFIG: Item<Config> = Item::new("config");
pub const POOL: Item<AstroportPool> = Item::new("pool");
pub const STAKING: Item<AstroportStaking> = Item::new("staking");
pub const STATE: Item<VaultState> = Item::new("state");

#[cw_serde]
pub struct Config {
    pub base_token: Addr,
    pub vault_token_denom: String,
    pub cw20_adaptor: Option<Addr>,
}

#[cw_serde]
pub struct VaultState {
    pub staked_base_tokens: Uint128,
    pub vault_token_supply: Uint128,
}
