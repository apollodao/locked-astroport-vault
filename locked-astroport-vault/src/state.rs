use std::collections::HashSet;

use crate::claims::Claims;
use apollo_cw_asset::AssetInfo;
use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, Decimal, Uint128};
use cw_dex::astroport::{AstroportPool, AstroportStaking};
use cw_dex_router::helpers::CwDexRouter;
use cw_storage_plus::Item;
use cw_utils::Duration;
use liquidity_helper::LiquidityHelper;

pub const CONFIG: Item<Config> = Item::new("config");
pub const POOL: Item<AstroportPool> = Item::new("pool");
pub const STAKING: Item<AstroportStaking> = Item::new("staking");
pub const STATE: Item<VaultState> = Item::new("state");

pub fn claims() -> Claims<'static> {
    Claims::new("claims", "claims_index", "num_claims")
}

#[cw_serde]
pub struct Config {
    pub base_token: Addr,
    pub vault_token_denom: String,
    pub lock_duration: Duration,
    pub reward_tokens: Vec<AssetInfo>,
    pub force_withdraw_whitelist: HashSet<Addr>,
    /// Whether or not deposits are enabled
    pub deposits_enabled: bool,
    /// The treasury address to send fees to
    pub treasury: Addr,
    /// The fee that is taken on rewards accrued
    pub performance_fee: Decimal,
    /// The router contract address
    pub router: CwDexRouter,
    /// The asset to which we should swap reward_assets into before providing
    /// liquidity. Should be one of the assets in the pool.
    pub reward_liquidation_target: AssetInfo,
    /// Helper for providing liquidity with unbalanced assets.
    pub liquidity_helper: LiquidityHelper,
}

#[cw_serde]
pub struct VaultState {
    pub staked_base_tokens: Uint128,
    pub vault_token_supply: Uint128,
}
