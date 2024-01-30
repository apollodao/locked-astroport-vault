use crate::claims::Claims;
use apollo_cw_asset::{Asset, AssetInfoBase, AssetList};
use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, CosmosMsg, Decimal, Deps, Env, StdError, StdResult, Uint128};
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

#[cw_serde]
#[derive(Default)]
/// A struct that contains a fee configuration (fee rate and recipients).
pub struct FeeConfig<T: AddressLike> {
    /// The fraction of the tokens that are taken as a fee.
    pub fee_rate: Decimal,
    /// The addresses of the recipients of the fee. Each address in the vec is
    /// paired with a Decimal, which represents the percentage of the fee
    /// that should be sent to that address. The sum of all decimals must be
    /// 1.
    pub fee_recipients: Vec<(T, Decimal)>,
}

impl FeeConfig<String> {
    /// Validates the fee config and returns a `FeeConfig<Addr>`.
    pub fn check(&self, deps: &Deps) -> StdResult<FeeConfig<Addr>> {
        // Fee rate must be between 0 and 100%
        if self.fee_rate > Decimal::one() {
            return Err(StdError::generic_err("Fee rate can't be higher than 100%"));
        }
        // If fee rate is not zero, then there must be some fee recipients and their
        // weights must sum to 100%
        if !self.fee_rate.is_zero()
            && self.fee_recipients.iter().map(|(_, p)| p).sum::<Decimal>() != Decimal::one()
        {
            return Err(StdError::generic_err(
                "Sum of fee recipient percentages must be 100%",
            ));
        }
        // Fee recipients should not contain zero weights
        if self.fee_recipients.iter().any(|(_, p)| p.is_zero()) {
            return Err(StdError::generic_err(
                "Fee recipient percentages must be greater than zero",
            ));
        }
        Ok(FeeConfig {
            fee_rate: self.fee_rate,
            fee_recipients: self
                .fee_recipients
                .iter()
                .map(|(addr, percentage)| Ok((deps.api.addr_validate(addr)?, *percentage)))
                .collect::<StdResult<Vec<_>>>()?,
        })
    }
}

impl FeeConfig<Addr> {
    /// Creates messages to transfer an `AssetList` of assets to the fee
    /// recipients.
    pub fn transfer_assets_msgs(&self, assets: &AssetList, env: &Env) -> StdResult<Vec<CosmosMsg>> {
        if self.fee_rate.is_zero() {
            return Ok(vec![]);
        }
        Ok(self
            .fee_recipients
            .iter()
            // Filter out the contract address because it's unnecessary to send fees to ourselves
            .filter(|(addr, _)| addr != env.contract.address)
            .map(|(addr, percentage)| {
                let assets: AssetList = assets
                    .iter()
                    .map(|asset| Asset::new(asset.info.clone(), asset.amount * *percentage))
                    .collect::<Vec<_>>()
                    .into();
                assets.transfer_msgs(addr)
            })
            .collect::<StdResult<Vec<_>>>()?
            .into_iter()
            .flatten()
            .collect())
    }

    /// Calculates the fee from the input assets and returns messages to send
    /// them to the fee recipients.
    pub fn fee_msgs_from_assets(&self, assets: &AssetList, env: &Env) -> StdResult<Vec<CosmosMsg>> {
        // Take fee from input assets
        let assets: AssetList = assets
            .iter()
            .map(|asset| Asset::new(asset.info.clone(), asset.amount * self.fee_rate))
            .collect::<Vec<_>>()
            .into();
        // Send fee to fee recipients
        self.transfer_assets_msgs(&assets, env)
    }

    /// Calculates the fee from the input asset and returns messages to send it
    /// to the fee recipients.
    pub fn fee_msgs_from_asset(&self, asset: Asset, env: &Env) -> StdResult<Vec<CosmosMsg>> {
        self.fee_msgs_from_assets(&AssetList::from(vec![asset]), env)
    }
}

impl From<FeeConfig<Addr>> for FeeConfig<String> {
    fn from(value: FeeConfig<Addr>) -> Self {
        Self {
            fee_rate: value.fee_rate,
            fee_recipients: value
                .fee_recipients
                .into_iter()
                .map(|(addr, percentage)| (addr.to_string(), percentage))
                .collect(),
        }
    }
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
    /// The router contract address
    pub router: CwDexRouterBase<T>,
    /// The asset to which we should swap reward_assets into before providing
    /// liquidity. Should be one of the assets in the pool.
    pub reward_liquidation_target: AssetInfoBase<T>,
    /// Helper for providing liquidity with unbalanced assets.
    pub liquidity_helper: LiquidityHelperBase<T>,
    /// The fee that is taken on rewards accrued
    pub performance_fee: FeeConfig<T>,
    /// A fee that is taken on deposits
    pub deposit_fee: FeeConfig<T>,
    /// A fee that is taken on withdrawals
    pub withdrawal_fee: FeeConfig<T>,
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

        // Validate reward tokens
        let reward_tokens = self
            .reward_tokens
            .iter()
            .map(|asset_info| asset_info.check(api))
            .collect::<StdResult<Vec<_>>>()?;
        let reward_liquidation_target = self.reward_liquidation_target.check(api)?;
        let router = self.router.check(api)?;

        // Reward liquidation target must be one of the pool assets
        let pool_assets = POOL.load(deps.storage)?.pool_assets;
        if !pool_assets.contains(&reward_liquidation_target) {
            return Err(StdError::generic_err(format!(
                "Reward liquidation target {:?} is not in the pool assets {:?}",
                reward_liquidation_target, pool_assets
            )));
        }

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
            router,
            reward_liquidation_target,
            liquidity_helper: self.liquidity_helper.check(api)?,
            performance_fee: self.performance_fee.check(&deps)?,
            deposit_fee: self.deposit_fee.check(&deps)?,
            withdrawal_fee: self.withdrawal_fee.check(&deps)?,
        })
    }
}

impl From<Config> for ConfigUnchecked {
    fn from(value: Config) -> Self {
        Self {
            lock_duration: value.lock_duration,
            reward_tokens: value.reward_tokens.into_iter().map(Into::into).collect(),
            deposits_enabled: value.deposits_enabled,
            router: value.router.into(),
            reward_liquidation_target: value.reward_liquidation_target.into(),
            liquidity_helper: value.liquidity_helper.into(),
            performance_fee: value.performance_fee.into(),
            deposit_fee: value.deposit_fee.into(),
            withdrawal_fee: value.withdrawal_fee.into(),
        }
    }
}

#[cw_serde]
/// A struct that represents the state of the vault.
pub struct VaultState {
    /// The total amount of base tokens staked in the vault.
    pub staked_base_tokens: Uint128,
    /// The total amount of vault tokens in circulation.
    pub vault_token_supply: Uint128,
}

#[cw_serde]
pub struct StateResponse {
    /// The total amount of base tokens staked in the vault.
    pub staked_base_tokens: Uint128,
    /// The total amount of vault tokens in circulation.
    pub vault_token_supply: Uint128,
    /// The CW20 token address of the base token.
    pub base_token: Addr,
    //// The denom of the native vault token that represents shares of the vault.
    pub vault_token_denom: String,
    /// The address of the Astroport pool that this vault is compounding rewards
    /// into.
    pub pool: AstroportPool,
    /// The AstroportStaking object config.
    pub staking: AstroportStaking,
}

#[cfg(test)]
pub mod tests {
    use cosmwasm_std::{testing::mock_dependencies, Decimal};

    #[test]
    fn fee_config_rate_cannot_be_larger_than_one() {
        let deps = mock_dependencies();

        let fee_config = super::FeeConfig {
            fee_rate: Decimal::one() + Decimal::percent(1),
            fee_recipients: vec![],
        };
        assert!(fee_config
            .check(&deps.as_ref())
            .unwrap_err()
            .to_string()
            .contains("Fee rate can't be higher than 100%"));
    }

    #[test]
    fn fee_config_recipients_must_sum_to_one() {
        let deps = mock_dependencies();

        let fee_config = super::FeeConfig {
            fee_rate: Decimal::percent(1),
            fee_recipients: vec![
                ("addr1".to_string(), Decimal::percent(20)),
                ("addr2".to_string(), Decimal::percent(50)),
            ],
        };
        assert!(fee_config
            .check(&deps.as_ref())
            .unwrap_err()
            .to_string()
            .contains("Sum of fee recipient percentages must be 100%"));
    }

    #[test]
    fn fee_config_recipient_weights_must_be_greater_than_zero() {
        let deps = mock_dependencies();

        let fee_config = super::FeeConfig {
            fee_rate: Decimal::percent(1),
            fee_recipients: vec![
                ("addr1".to_string(), Decimal::percent(100)),
                ("addr2".to_string(), Decimal::zero()),
            ],
        };
        assert!(fee_config
            .check(&deps.as_ref())
            .unwrap_err()
            .to_string()
            .contains("Fee recipient percentages must be greater than zero"));
    }
}
