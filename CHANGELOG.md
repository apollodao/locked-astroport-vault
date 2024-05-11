# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

# [Unreleased]

### Changed

- Change base token from `Addr` to `AssetInfo`.
- Bumped `cw-dex-astroport` to `0.2.0-rc.1`.
- Bumped `cw-dex-router` to `0.4.1-rc.1`.


# [0.4.3] - 2024-04-23

### Changed

- Added release profile to `Cargo.toml` with `overflow-checks = true` to prevent wrapping on overflows in release builds.
- Bumped `cosmwasm-std` to `1.5.4`.

# [0.4.2] - 2024-02-26

### Added

- Added event attributes `staked_base_tokens_after_action` and `vault_token_supply_after_action` to `apollo/vaults/execute_deposit`, `apollo/vaults/execute_redeem`, and `apollo/vaults/execute_compound` events.

### Changed

- Bumped `cw-it` to `0.3.1`.

# [0.4.1] - 2024-02-14

### Changed

- Updated migration from `0.2.0` and `0.3.0` to unstake from astroport generator and stake in astroport incentives contract.

# [0.4.0] - 2024-02-13

### Changed

- Use Astroport incentives contract instead of generator for staking rewards.
- Bumped `cw-dex` to `0.5.3`
- Started using `cw-dex-astroport` crate for Pool and Staking implementations.
- Bumped `cw-it` to `0.3.0`.

# [0.3.0] - 2024-02-03

### Added
- Add optional deposit and withdrawal fees

# [0.2.0] - 2023-11-03

### Changed

- Bumped `cw-dex` to `0.5.0`
  - This required adding the field `astroport_liquidity_manager: String` to `InstantiateMsg`.
- Bumped `cw-dex-router` to `0.3.0`
- Bumped `liquidity-helper` and `astroport-liquidity-helper` to `0.3.0`
- Bumped `cosmwasm-std` to `1.5.0`
