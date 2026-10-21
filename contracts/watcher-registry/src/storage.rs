use soroban_sdk::{Address, Env, Vec};

use crate::types::{ContractError, DataKey, WatcherInfo};

pub fn load_admins(env: &Env) -> Vec<Address> {
    env.storage()
        .instance()
        .get(&DataKey::Admins)
        .unwrap_or_else(|| Vec::new(env))
}

pub fn save_admins(env: &Env, admins: &Vec<Address>) {
    env.storage().instance().set(&DataKey::Admins, admins);
}

pub fn load_watchers(env: &Env) -> Vec<WatcherInfo> {
    env.storage()
        .instance()
        .get(&DataKey::Watchers)
        .unwrap_or_else(|| Vec::new(env))
}

pub fn save_watchers(env: &Env, watchers: &Vec<WatcherInfo>) {
    env.storage().instance().set(&DataKey::Watchers, watchers);
}

pub fn assert_admin(env: &Env, caller: &Address) -> Result<(), ContractError> {
    let admins = load_admins(env);
    if !admins.iter().any(|a| &a == caller) {
        return Err(ContractError::Unauthorized);
    }
    Ok(())
}

pub fn load_watcher_count(env: &Env) -> u32 {
    env.storage()
        .instance()
        .get(&DataKey::WatcherCount)
        .unwrap_or(0)
}

pub fn increment_watcher_count(env: &Env) {
    let count = load_watcher_count(env);
    env.storage()
        .instance()
        .set(&DataKey::WatcherCount, &(count + 1));
}

pub fn decrement_watcher_count(env: &Env) {
    let count = load_watcher_count(env);
    let next = count.saturating_sub(1);
    env.storage()
        .instance()
        .set(&DataKey::WatcherCount, &next);
}
