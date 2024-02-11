use crate::state::{self, FeeConfig, CONFIG};
use cosmwasm_std::{Addr, Decimal, DepsMut, StdResult};
use cw_dex::astroport::AstroportStaking;
use cw_storage_plus::Item;
use locked_astroport_vault_0_2_0::state::Config as Config0_2_0;

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

pub fn migrate_from_0_3_0_to_0_4_0(deps: DepsMut, incentives_contract: Addr) -> StdResult<()> {
    let old_staking = locked_astroport_vault_0_2_0::state::STAKING.load(deps.storage)?;

    let staking = AstroportStaking {
        lp_token_addr: old_staking.lp_token_addr,
        incentives: incentives_contract,
    };

    state::STAKING.save(deps.storage, &staking)?;

    Ok(())
}
