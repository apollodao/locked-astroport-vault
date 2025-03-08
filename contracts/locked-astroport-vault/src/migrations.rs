use crate::state::{self, FeeConfig, BASE_TOKEN, CONFIG, STAKING, STATE};
use apollo_utils::responses::merge_responses;
use cosmwasm_std::{Addr, Decimal, DepsMut, Env, Response, StdResult, Uint128};
use cw_dex::traits::{Rewards, Stake, Unstake};
use cw_dex_astroport::AstroportStaking;
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

    Ok(())
}

#[allow(deprecated)]
pub fn migrate_from_0_3_0_to_v0_4_1(
    deps: DepsMut,
    env: Env,
    incentives_contract: Addr,
) -> StdResult<Response> {
    let old_staking = locked_astroport_vault_0_2_0::state::STAKING.load(deps.storage)?;

    let staking = AstroportStaking {
        #[allow(deprecated)]
        lp_token_addr: old_staking.lp_token_addr.clone(),
        incentives: incentives_contract,
    };

    state::STAKING.save(deps.storage, &staking)?;

    // Read total staked amount
    let state = STATE.load(deps.storage).unwrap();
    if state.staked_base_tokens.is_zero() {
        return Ok(Response::default());
    }

    // Claim all pending rewards from old staking contract
    let claim_res = old_staking.claim_rewards(deps.as_ref(), &env)?;

    // Unstake entire balance from old staking contract
    let unstake_res = old_staking.unstake(deps.as_ref(), &env, state.staked_base_tokens)?;

    // Stake entire balance in new staking contract
    let stake_res = staking.stake(deps.as_ref(), &env, state.staked_base_tokens)?;

    Ok(merge_responses(vec![claim_res, unstake_res, stake_res]))
}

/// Migrates the contract from v0.4.1 to v0.4.4.
///
/// Withdraws any remaining assets from the old generator contract and migrates
/// to the new generator contract. Some funds were left in the old generator
/// contract after the migration from 0.3.0 to 0.4.1, so we need to withdraw
/// them and deposit them into the new generator contract.
pub fn migrate_from_v0_4_1_to_v0_4_4(
    deps: DepsMut,
    env: Env,
    old_generator: Addr,
    amount: Uint128,
) -> StdResult<Response> {
    let base_token = BASE_TOKEN.load(deps.storage)?;
    let old_staking = AstroportStaking {
        lp_token_addr: base_token.clone(),
        incentives: old_generator,
    };
    let unstake_res = old_staking.unstake(deps.as_ref(), &env, amount)?;

    let new_staking = STAKING.load(deps.storage)?;
    let stake_res = new_staking.stake(deps.as_ref(), &env, amount)?;

    Ok(merge_responses(vec![unstake_res, stake_res]))
}
