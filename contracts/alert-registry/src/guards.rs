//! Shared guard helpers for the alert registry contract.
//!
//! This module centralises the authorization and limit checks used across
//! [`crate::AlertRegistry`]. Keeping them here improves discoverability as the
//! contract grows and makes it easy to add future guards (e.g.
//! `assert_not_paused`, `assert_watcher_authorized`) in one place.

use soroban_sdk::{symbol_short, Address, Env};

use crate::{AlertConfig, ContractError, DataKey};

/// Assert that `caller` is the owner of the alert described by `config`.
///
/// Returns [`ContractError::Unauthorized`] if the addresses do not match.
pub fn assert_owner(config: &AlertConfig, caller: &Address) -> Result<(), ContractError> {
    if config.owner != *caller {
        return Err(ContractError::Unauthorized);
    }
    Ok(())
}

/// Assert that `caller` is the currently configured admin.
///
/// Returns [`ContractError::NotInitialized`] if no admin has been set and
/// [`ContractError::Unauthorized`] if `caller` is not the admin.
pub fn assert_admin(env: &Env, caller: &Address) -> Result<(), ContractError> {
    let admin: Address = env
        .storage()
        .instance()
        .get(&symbol_short!("ADMIN"))
        .ok_or(ContractError::NotInitialized)?;
    if admin != *caller {
        return Err(ContractError::Unauthorized);
    }
    Ok(())
}

/// Assert that `owner` has not exceeded the configured per-owner active alert
/// limit.
///
/// A configured limit of `0` means no limit is enforced. Returns
/// [`ContractError::OwnerAlertLimitExceeded`] when the owner already has at
/// least `limit` active alerts.
pub fn assert_per_owner_limit(env: &Env, owner: &Address) -> Result<(), ContractError> {
    let limit: u32 = env
        .storage()
        .instance()
        .get(&symbol_short!("LIMIT"))
        .unwrap_or(0u32);
    if limit == 0 {
        return Ok(());
    }

    let ids: soroban_sdk::Vec<u64> = env
        .storage()
        .persistent()
        .get(&DataKey::OwnerIndex(owner.clone()))
        .unwrap_or_else(|| soroban_sdk::Vec::new(env));

    let mut active: u32 = 0;
    for id in ids.iter() {
        let is_active: bool = env
            .storage()
            .persistent()
            .get(&DataKey::AlertActive(id))
            .unwrap_or(false);
        if is_active {
            active += 1;
        }
    }

    if active >= limit {
        return Err(ContractError::OwnerAlertLimitExceeded);
    }
    Ok(())
}
