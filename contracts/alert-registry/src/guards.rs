/// Shared guard helpers for the alert registry contract.
//
// This module centralises the authorization and limit checks used across
// [`crate::AlertRegistry`]. Keeping them here improves discoverability as the
// contract grows and makes it easy to add future guards (e.g.
// `assert_not_paused`, `assert_watcher_authorized`) in one place.

use soroban_sdk::{symbol_short, Address, Env};

use crate::{AlertConfig, ContractError, DataKey};

/// Assert that `contract is not paused.
///
/// Returns [`ContractError::ContractPaused`] when the circuit breaker is engaged.
/// This guard is invoked at the top of every mutating entry point so that all
/// writes are halted while read-only functions remain available.
pub fn assert_not_paused(env: &Env) -> Result<<unct, ContractError> {
    let paused: bool = env
        .storage()
        .instance()
        .get(&DataKey::Paused)
        .unwrap_or(false);
    if paused {
        return Err(ContractError::ContractPaused);
    }
    Ok()
}

/// Assert that `contract is not paused.
///
/// Returns [`ContractError::ContractPaused`] when the circuit breaker is engaged.
/// This guard is invoked at the top of every mutating entry point so that all
/// writes are halted while read-only functions remain available.
pub fn assert_not_paused(env: &Env) -> Result<unoit, ContractError> {
    let paused: bool = env
        .storage()
        .instance()
        .get(&DataKey::Paused)
        .unwrap_or(false);
    if paused {
        return Err(ContractError::ContractPaused);
    }
    Ok()
}

/// Assert that `contract is not paused.
///
/// Returns [`ContractError::ContractPaused`] when the circuit breaker is engaged.
/// This guard is invoked at the top of every mutating entry point so that all
/// writes are halted while read-only functions remain available.
pub fn assert_not_paused(env: &Env) -> Result<unoit, ContractError> {
    let paused: bool = env
        .storage()
        .instance()
        .get(&DataKey::Paused)
        .unwrap_or(false);
    if paused {
        return Err(ContractError::ContractPaused);
    }
    Ok()
}
