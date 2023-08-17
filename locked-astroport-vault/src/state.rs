use crate::claims::Claims;
use apollo_cw_asset::AssetInfoBase;
use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, Decimal, Deps, StdError, StdResult, Uint128};
use cw_address_like::AddressLike;
use cw_dex::astroport::{AstroportPool, AstroportStaking};
use cw_dex_router::helpers::CwDexRouterBase;
use cw_item_set::Set;
use cw_storage_plus::Item;
use cw_utils::Duration;
use liquidity_helper::LiquidityHelperBase;
use optional_struct::{optional_struct, Applyable};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// Stores the configurable values for the contract.
pub const CONFIG: Item<Config> = Item::new("config");

/// Stores the Astroport pool that the vault compounds.
pub const POOL: Item<AstroportPool> = Item::new("pool");

/// Stores the Astroport staking contract that the vault uses to stake the LP
/// tokens.
pub const STAKING: Item<AstroportStaking> = Item::new("staking");

/// The base token that is accepted for deposits and that the vault accrues more
/// of over time. In this case it is an Astroport CW20 LP token.
pub const BASE_TOKEN: Item<Addr> = Item::new("base_token");

/// The denom of the native vault token that represents shares of the vault.
pub const VAULT_TOKEN_DENOM: Item<String> = Item::new("vault_token_denom");

/// Stores the state of the vault.
pub const STATE: Item<VaultState> = Item::new("state");

/// Stores a set of addresses that are allowed to force withdraw. This is used
/// in the case of liquidations when the vault tokens are used as collateral in
/// lending protocols such as Mars.
pub const FORCE_WITHDRAW_WHITELIST: Set<&Addr> = Set::new("whitelist");

/// Stores unlocking positions that are created upon redeeming vault tokens if
/// the vault has a lockup duration.
pub fn claims() -> Claims<'static> {
    Claims::new("claims", "claims_index", "num_claims")
}

#[optional_struct(ConfigUpdates)]
#[derive(Serialize, Deserialize, JsonSchema, Clone, Debug, PartialEq)]
pub struct ConfigBase<T: AddressLike> {
    /// The duration of the lock period for deposits. This can be set to zero
    /// seconds to disable locking.
    pub lock_duration: Duration,
    /// The tokens that are compounded into more base tokens. This can be
    /// updated if more tokens are available on either the Astroport
    /// generator or just transfered to the vault directly.
    pub reward_tokens: Vec<AssetInfoBase<T>>,
    /// Whether or not deposits are enabled
    pub deposits_enabled: bool,
    /// The treasury address to send fees to
    pub treasury: T,
    /// The fee that is taken on rewards accrued
    pub performance_fee: Decimal,
    /// The router contract address
    pub router: CwDexRouterBase<T>,
    /// The asset to which we should swap reward_assets into before providing
    /// liquidity. Should be one of the assets in the pool.
    pub reward_liquidation_target: AssetInfoBase<T>,
    /// Helper for providing liquidity with unbalanced assets.
    pub liquidity_helper: LiquidityHelperBase<T>,
}

pub type Config = ConfigBase<Addr>;
pub type ConfigUnchecked = ConfigBase<String>;

impl ConfigUnchecked {
    /// Checks that the all the values on the `ConfigUnchecked` are valid and
    /// returns a `Config`.
    pub fn check(&self, deps: Deps) -> StdResult<Config> {
        let api = deps.api;

        // Check that the lock duration is specified in seconds
        match self.lock_duration {
            Duration::Time(_) => {}
            _ => {
                return Err(StdError::generic_err(
                    "lock_duration must be specified in seconds",
                ))
            }
        }

        // Validate performance fee
        if self.performance_fee > Decimal::one() {
            return Err(StdError::generic_err(
                "Performance fee can't be higher than 100%",
            ));
        }

        // Validate reward tokens
        let reward_tokens = self
            .reward_tokens
            .iter()
            .map(|asset_info| asset_info.check(api))
            .collect::<StdResult<Vec<_>>>()?;
        let reward_liquidation_target = self.reward_liquidation_target.check(api)?;
        let router = self.router.check(api)?;

        // Check that the router can route between all reward assets and the
        // reward liquidation target. We discard the actual path because we
        // don't need it here. We just need to make sure the paths exist.
        for asset in &reward_tokens {
            // We skip the reward liquidation target because we don't need to
            // route from it.
            if asset == &reward_liquidation_target {
                continue;
            }
            // We map the error here because the error coming from the router is
            // not passed along into the query error, and thus we will otherwise
            // just see "Querier contract error" and no more information.
            router
                .query_path_for_pair(&deps.querier, asset, &reward_liquidation_target)
                .map_err(|_| {
                    StdError::generic_err(format!(
                        "Could not read path in cw-dex-router for {:?} -> {:?}",
                        asset, reward_liquidation_target
                    ))
                })?;
        }

        Ok(Config {
            lock_duration: self.lock_duration,
            reward_tokens,
            deposits_enabled: self.deposits_enabled,
            treasury: api.addr_validate(&self.treasury)?,
            performance_fee: self.performance_fee,
            router,
            reward_liquidation_target,
            liquidity_helper: self.liquidity_helper.check(api)?,
        })
    }
}

impl From<Config> for ConfigUnchecked {
    fn from(value: Config) -> Self {
        Self {
            lock_duration: value.lock_duration,
            reward_tokens: value.reward_tokens.into_iter().map(Into::into).collect(),
            deposits_enabled: value.deposits_enabled,
            treasury: value.treasury.to_string(),
            performance_fee: value.performance_fee,
            router: value.router.into(),
            reward_liquidation_target: value.reward_liquidation_target.into(),
            liquidity_helper: value.liquidity_helper.into(),
        }
    }
}

#[cw_serde]
/// A struct that represents the state of the vault.
pub struct VaultState {
    /// The total amount of base tokens staked in the vault.
    pub staked_base_tokens: Uint128,
    /// The total amount of vault tokens that have been minted.
    pub vault_token_supply: Uint128,
}
