use crate::state::{self, FeeConfig, CONFIG};
use apollo_cw_asset::AssetInfo;
use cosmwasm_std::{Addr, Decimal, DepsMut, Env, Response, StdResult};
use cw_dex_astroport::{AstroportPool, AstroportStaking};
use cw_storage_plus::Item;
use locked_astroport_vault_0_2_0::state::Config as Config0_2_0;

pub fn migrate_from_0_2_0_to_0_3_0(deps: DepsMut) -> StdResult<Response> {
    // Load the old config
    let old_config: Config0_2_0 = Item::new("config").load(deps.storage)?;

    // Create the new config
    let config = state::Config {
        lock_duration: old_config.lock_duration,
        reward_tokens: old_config.reward_tokens,
        deposits_enabled: old_config.deposits_enabled,
        router: cw_dex_router::helpers::CwDexRouter::new(&old_config.router.0),
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

    Ok(Response::default())
}

#[allow(deprecated)]
pub fn migrate_from_0_3_0_to_0_4_x(
    deps: DepsMut,
    env: Env,
    incentives_contract: Addr,
) -> StdResult<Response> {
    locked_astroport_vault_0_4_2::migrations::migrate_from_0_3_0_to_current(
        deps,
        env,
        incentives_contract,
    )
}

pub fn migrate_from_0_4_x_to_current(deps: DepsMut) -> StdResult<Response> {
    // Migrate lp_token in AstroportStaking from Addr to AssetInfo
    let old_staking = locked_astroport_vault_0_4_2::state::STAKING.load(deps.storage)?;
    let staking = AstroportStaking {
        lp_token: AssetInfo::cw20(old_staking.lp_token_addr),
        incentives: old_staking.incentives,
    };
    state::STAKING.save(deps.storage, &staking)?;

    // Migrate lp_token in AstroportPool from Addr to AssetInfo
    let old_pool = locked_astroport_vault_0_4_2::state::POOL.load(deps.storage)?;
    let pool = AstroportPool {
        lp_token: AssetInfo::cw20(old_pool.lp_token_addr),
        liquidity_manager: old_pool.liquidity_manager,
        pair_addr: old_pool.pair_addr,
        pair_type: old_pool.pair_type,
        pool_assets: old_pool.pool_assets,
    };
    state::POOL.save(deps.storage, &pool)?;

    // Migrate lp token in base token from Addr to AssetInfo
    let old_base_token = locked_astroport_vault_0_4_2::state::BASE_TOKEN.load(deps.storage)?;
    let base_token = AssetInfo::cw20(old_base_token);
    state::BASE_TOKEN.save(deps.storage, &base_token)?;

    Ok(Response::default())
}
