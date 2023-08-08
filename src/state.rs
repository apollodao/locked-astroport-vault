use std::collections::HashSet;

use apollo_cw_asset::AssetInfo;
use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, Uint128};
use cw_controllers::Claims;
use cw_dex::astroport::{AstroportPool, AstroportStaking};
use cw_storage_plus::Item;
use cw_utils::Duration;

pub const CONFIG: Item<Config> = Item::new("config");
pub const POOL: Item<AstroportPool> = Item::new("pool");
pub const STAKING: Item<AstroportStaking> = Item::new("staking");
pub const STATE: Item<VaultState> = Item::new("state");
pub const CLAIMS: Claims = Claims::new("claims");

#[cw_serde]
pub struct Config {
    pub base_token: Addr,
    pub vault_token_denom: String,
    pub cw20_adaptor: Option<Addr>,
    pub lock_duration: Duration,
    pub reward_tokens: Vec<AssetInfo>,
    pub force_withdraw_whitelist: HashSet<Addr>,
}

#[cw_serde]
pub struct VaultState {
    pub staked_base_tokens: Uint128,
    pub vault_token_supply: Uint128,
}
