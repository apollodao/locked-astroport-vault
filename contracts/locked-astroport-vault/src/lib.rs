/// Contains the contract's entrypoints.
pub mod contract;

/// Contains the vaults API and related messages.
pub mod msg;

/// Contains ExecuteMsg handler functions.
pub mod execute;

/// Contains QueryMsg handlers.
pub mod query;

/// Contains structs and helpers for the state of the vault.
pub mod state;

/// Contains the contracts error type.
pub mod error;

/// Contains helper and utility functions and traits.
pub mod helpers;

/// Contains the Claims struct for storing claims on locked LP tokens
/// (UnlockingPositions).
pub mod claims;

/// Contains functions for migrating the contract state from older to newer
/// versions.
pub mod migrations;
