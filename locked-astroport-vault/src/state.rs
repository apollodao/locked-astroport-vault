use crate::claims::Claims;
use apollo_cw_asset::AssetInfo;
use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, Decimal, Uint128};
use cw_dex::astroport::{AstroportPool, AstroportStaking};
use cw_dex_router::helpers::CwDexRouter;
use cw_item_set::Set;
use cw_storage_plus::Item;
use cw_utils::Duration;
use liquidity_helper::LiquidityHelper;

/// Stores the configurable values for the contract.
pub const CONFIG: Item<Config> = Item::new("config");

/// Stores the Astroport pool that the vault compounds.
pub const POOL: Item<AstroportPool> = Item::new("pool");

/// Stores the Astroport staking contract that the vault uses to stake the LP tokens.
pub const STAKING: Item<AstroportStaking> = Item::new("staking");

/// The base token that is accepted for deposits and that the vault accrues more of over time.
/// In this case it is an Astroport CW20 LP token.
pub const BASE_TOKEN: Item<Addr> = Item::new("base_token");

/// The denom of the native vault token that represents shares of the vault.
pub const VAULT_TOKEN_DENOM: Item<String> = Item::new("vault_token_denom");

/// Stores the state of the vault.
pub const STATE: Item<VaultState> = Item::new("state");

/// Stores a set of addresses that are allowed to force withdraw. This is used in the case of
/// liquidations when the vault tokens are used as collateral in lending protocols such as Mars.
pub const FORCE_WITHDRAW_WHITELIST: Set<&Addr> = Set::new("whitelist");

/// Stores unlocking positions that are created upon redeeming vault tokens if the vault has a
/// lockup duration.
pub fn claims() -> Claims<'static> {
    Claims::new("claims", "claims_index", "num_claims")
}

#[cw_serde]
pub struct Config {
    /// The duration of the lock period for deposits. This can be set to zero seconds to disable locking.
    pub lock_duration: Duration,
    /// The tokens that are compounded into more base tokens. This can be updated if more tokens are
    /// available on either the Astroport generator or just transfered to the vault directly.
    pub reward_tokens: Vec<AssetInfo>,
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
