use crate::state::{self, FeeConfig, CONFIG};
use apollo_cw_asset::AssetInfoBase;
use cosmwasm_std::{Addr, Decimal, DepsMut, StdResult};
use cw_dex_router::helpers::CwDexRouterBase;
use cw_storage_plus::Item;
use cw_utils::Duration;
use liquidity_helper::LiquidityHelperBase;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, JsonSchema, Clone, Debug, PartialEq)]
/// The Config of the contract v0.2.0
pub struct Config0_2_0 {
    /// The duration of the lock period for deposits. This can be set to zero
    /// seconds to disable locking.
    pub lock_duration: Duration,
    /// The tokens that are compounded into more base tokens. This can be
    /// updated if more tokens are available on either the Astroport
    /// generator or just transfered to the vault directly.
    pub reward_tokens: Vec<AssetInfoBase<Addr>>,
    /// Whether or not deposits are enabled
    pub deposits_enabled: bool,
    /// The treasury address to send fees to
    pub treasury: Addr,
    /// The fee that is taken on rewards accrued
    pub performance_fee: Decimal,
    /// The router contract address
    pub router: CwDexRouterBase<Addr>,
    /// The asset to which we should swap reward_assets into before providing
    /// liquidity. Should be one of the assets in the pool.
    pub reward_liquidation_target: AssetInfoBase<Addr>,
    /// Helper for providing liquidity with unbalanced assets.
    pub liquidity_helper: LiquidityHelperBase<Addr>,
}

pub fn migrate_from_0_2_0_to_0_3_0(deps: DepsMut) -> StdResult<()> {
    // Load the old config
    let old_config: Config0_2_0 = Item::new("config").load(deps.storage)?;

    // Create the new config
    let config = state::Config {
        lock_duration: old_config.lock_duration,
        reward_tokens: old_config.reward_tokens,
        deposits_enabled: old_config.deposits_enabled,
        router: old_config.router,
        reward_liquidation_target: old_config.reward_liquidation_target,
        liquidity_helper: old_config.liquidity_helper,
        performance_fee: FeeConfig {
            fee_rate: old_config.performance_fee,
            fee_recipients: vec![(old_config.treasury, Decimal::one())],
        },
        deposit_fee: FeeConfig::default().check(&deps.as_ref())?,
        withdrawal_fee: FeeConfig::default().check(&deps.as_ref())?,
    };

    // Store the new config
    CONFIG.save(deps.storage, &config)?;

    Ok(())
}
