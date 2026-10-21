use crate::AlertRegistry;
use crate::ContractError;
use crate::AlertRegistryClient;
use crate::DataKey;
use soroban_sdk::{testutils::Address as _, vec, Address, Env, String, Vec};

fn setup() -> (Env, AlertRegistryClient<'static>) {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(AlertRegistry, ());
    let client = AlertRegistryClient::new(&env, &contract_id);
    (env, client)
}

fn str(env: &Env, s: &str) -> String {
    String::from_str(env, s)
}

/// Build a Soroban String of `n` repetitions of ASCII char `ch`.
/// Uses a fixed 8192-byte stack buffer — sufficient for the Soroban max.
fn str_repeat(env: &Env, ch: char, n: usize) -> String {
    assert!(n <= 8192, "str_repeat: n exceeds Soroban String max");
    let byte = ch as u8;
    let mut buf = [0u8; 8192];
    for b in buf.iter_mut().take(n) {
        *b = byte;
    }
    let s = core::str::from_utf8(&buf[..n]).unwrap();
    String::from_str(env, s)
}

// 1. Happy path — register and retrieve
#[test]
fn test_register_and_get_alert() {
    let (env, client) = setup();
    let owner = Address::generate(&env);
    let target = Address::generate(&env);

    let id = client.register_alert(
        &owner,
        &target,
        &str(&env, "My Alert"),
        &str(&env, "hash123"),
        &vec![&env, str(&env, "rule:transfer")],
    );

    let cfg = client.get_alert(&id).unwrap();
    assert_eq!(cfg.label, str(&env, "My Alert"));
    assert_eq!(cfg.owner, owner);
    assert!(cfg.active);
}

// 2. Happy path — update alert
#[test]
fn test_update_alert() {
    let (env, client) = setup();
    let owner = Address::generate(&env);
    let target = Address::generate(&env);

    let id = client.register_alert(
        &owner,
        &target,
        &str(&env, "Alert"),
        &str(&env, "hash"),
        &vec![&env, str(&env, "rule:transfer")],
    );

    assert_eq!(
        client
            .try_update_alert(&owner, &id, &vec![&env, str(&env, "rule:mint")], &false)
            .unwrap(),
        Ok(())
    );

    let cfg = client.get_alert(&id).unwrap();
    assert!(!cfg.active);
    assert_eq!(cfg.rules.get(0).unwrap(), str(&env, "rule:mint"));
}

// 3. Happy path — remove alert
#[test]
fn test_remove_alert() {
    let (env, client) = setup();
    let owner = Address::generate(&env);
    let target = Address::generate(&env);

    let id = client.register_alert(
        &owner,
        &target,
        &str(&env, "Alert"),
        &str(&env, "hash"),
        &vec![&env],
    );

    assert_eq!(client.try_remove_alert(&owner, &id).unwrap(), Ok(()));
    assert!(client.get_alert(&id).is_none());
}

// 4. Unauthorized update rejected
#[test]
fn test_update_unauthorized() {
    let (env, client) = setup();
    let owner = Address::generate(&env);
    let attacker = Address::generate(&env);
    let target = Address::generate(&env);

    let id = client.register_alert(
        &owner,
        &target,
        &str(&env, "Alert"),
        &str(&env, "hash"),
        &vec![&env],
    );

    assert_eq!(
        client
            .try_update_alert(&attacker, &id, &vec![&env], &false)
            .unwrap_err()
            .unwrap(),
        ContractError::Unauthorized
    );
}

#[test]
#[should_panic(expected = "invalid rule descriptor")]
fn test_register_alert_rejects_invalid_rules() {
    let (env, client) = setup();
    let owner = Address::generate(&env);
    let target = Address::generate(&env);

    client.register_alert(
        &owner,
        &target,
        &str(&env, "Alert"),
        &str(&env, "hash"),
        &vec![&env, str(&env, "rule:unknown")],
    );
}

#[test]
#[should_panic(expected = "invalid rule descriptor")]
fn test_update_alert_rejects_invalid_rules() {
    let (env, client) = setup();
    let owner = Address::generate(&env);
    let target = Address::generate(&env);

    let id = client.register_alert(
        &owner,
        &target,
        &str(&env, "Alert"),
        &str(&env, "hash"),
        &vec![&env, str(&env, "rule:transfer")],
    );

    client.update_alert(&owner, &id, &vec![&env, str(&env, "rule:bogus")], &true);
}

#[test]
fn test_admin_remove_any_alert() {
    let (env, client) = setup();
    let admin = Address::generate(&env);
    client.initialize(&admin);

    let owner = Address::generate(&env);
    let target = Address::generate(&env);
    let id = client.register_alert(
        &owner,
        &target,
        &str(&env, "Alert"),
        &str(&env, "hash"),
        &vec![&env, str(&env, "rule:mint")],
    );

    assert_eq!(client.remove_alert_by_admin(&admin, &id), ());
    assert!(client.get_alert(&id).is_none());
}

#[test]
#[should_panic(expected = "owner alert limit exceeded")]
fn test_admin_set_per_owner_alert_limit() {
    let (env, client) = setup();
    let admin = Address::generate(&env);
    client.initialize(&admin);
    client.set_per_owner_alert_limit(&admin, &1u32);

    let owner = Address::generate(&env);
    let target = Address::generate(&env);
    client.register_alert(
        &owner,
        &target,
        &str(&env, "Alert1"),
        &str(&env, "hash1"),
        &vec![&env, str(&env, "rule:transfer")],
    );
    client.register_alert(
        &owner,
        &target,
        &str(&env, "Alert2"),
        &str(&env, "hash2"),
        &vec![&env, str(&env, "rule:mint")],
    );
}

#[test]
fn test_admin_transfer_admin() {
    let (env, client) = setup();
    let admin = Address::generate(&env);
    client.initialize(&admin);
    let new_admin = Address::generate(&env);

    assert_eq!(client.transfer_admin(&admin, &new_admin), ());
    let owner = Address::generate(&env);
    let target = Address::generate(&env);
    let id = client.register_alert(
        &owner,
        &target,
        &str(&env, "Alert"),
        &str(&env, "hash"),
        &vec![&env, str(&env, "rule:transfer")],
    );
    assert_eq!(client.remove_alert_by_admin(&new_admin, &id), ());
}

// 5. Unauthorized remove rejected
#[test]
fn test_remove_unauthorized() {
    let (env, client) = setup();
    let owner = Address::generate(&env);
    let attacker = Address::generate(&env);
    let target = Address::generate(&env);

    let id = client.register_alert(
        &owner,
        &target,
        &str(&env, "Alert"),
        &str(&env, "hash"),
        &vec![&env],
    );

    assert_eq!(
        client
            .try_remove_alert(&attacker, &id)
            .unwrap_err()
            .unwrap(),
        ContractError::Unauthorized
    );
}

// 6. Edge case — get nonexistent alert returns None
#[test]
fn test_get_nonexistent_alert() {
    let (_env, client) = setup();
    assert!(client.get_alert(&999u64).is_none());
}

// 7. Edge case — get alerts for contract with no alerts returns empty vec
#[test]
fn test_get_alerts_for_contract_empty() {
    let (env, client) = setup();
    let querier = Address::generate(&env);
    let target = Address::generate(&env);
    assert_eq!(client.get_alerts_for_contract(&querier, &target).unwrap().len(), 0);
}

// 8. Index queries
#[test]
fn test_index_queries() {
    let (env, client) = setup();
    let querier = Address::generate(&env);
    let owner = Address::generate(&env);
    let target = Address::generate(&env);

    client.register_alert(&owner, &target, &str(&env, "A1"), &str(&env, "h1"), &vec![&env]);
    client.register_alert(&owner, &target, &str(&env, "A2"), &str(&env, "h2"), &vec![&env]);

    assert_eq!(client.get_alerts_for_contract(&querier, &target).unwrap().len(), 2);
    assert_eq!(client.get_alerts_by_owner(&querier, &owner).unwrap().len(), 2);
}

// 9. get_alert_count is monotonic
#[test]
fn test_get_alert_count() {
    let (env, client) = setup();
    let owner = Address::generate(&env);
    let target = Address::generate(&env);

    assert_eq!(client.get_alert_count(), 0);
    let id = client.register_alert(&owner, &target, &str(&env, "A"), &str(&env, "h"), &vec![&env]);
    assert_eq!(client.get_alert_count(), 1);
    client.register_alert(&owner, &target, &str(&env, "B"), &str(&env, "h"), &vec![&env]);
    assert_eq!(client.get_alert_count(), 2);
    client.remove_alert(&owner, &id);
    assert_eq!(client.get_alert_count(), 2);
}

// get_active_alert_count decreases after remove
#[test]
fn test_get_active_alert_count() {
    let (env, client) = setup();
    let owner = Address::generate(&env);
    let target = Address::generate(&env);

    assert_eq!(client.get_active_alert_count(&owner), 0);
    let id1 = client.register_alert(&owner, &target, &str(&env, "A"), &str(&env, "h"), &vec![&env]);
    let _id2 = client.register_alert(&owner, &target, &str(&env, "B"), &str(&env, "h"), &vec![&env]);
    assert_eq!(client.get_active_alert_count(&owner), 2);
    client.remove_alert(&owner, &id1);
    assert_eq!(client.get_active_alert_count(&owner), 1);
}

// 10. update_webhook changes the hash
#[test]
fn test_update_webhook() {
    let (env, client) = setup();
    let owner = Address::generate(&env);
    let target = Address::generate(&env);

    let id = client.register_alert(&owner, &target, &str(&env, "A"), &str(&env, "old-hash"), &vec![&env]);
    assert_eq!(
        client.try_update_webhook(&owner, &id, &str(&env, "new-hash")).unwrap(),
        Ok(())
    );
    assert_eq!(client.get_alert(&id).unwrap().webhook_hash, str(&env, "new-hash"));
}

// 11. update_webhook unauthorized
#[test]
fn test_update_webhook_unauthorized() {
    let (env, client) = setup();
    let owner = Address::generate(&env);
    let attacker = Address::generate(&env);
    let target = Address::generate(&env);

    let id = client.register_alert(&owner, &target, &str(&env, "A"), &str(&env, "hash"), &vec![&env]);
    assert_eq!(
        client
            .try_update_webhook(&attacker, &id, &str(&env, "evil-hash"))
            .unwrap_err()
            .unwrap(),
        ContractError::Unauthorized
    );
}

#[test]
fn test_update_alert_missing_returns_not_found() {
    let (env, client) = setup();
    let attacker = Address::generate(&env);

    assert_eq!(
        client
            .try_update_alert(&attacker, &999u64, &vec![&env], &false)
            .unwrap_err()
            .unwrap(),
        ContractError::AlertNotFound
    );
}

#[test]
fn test_remove_alert_nonexistent_returns_not_found() {
    let (env, client) = setup();
    let owner = Address::generate(&env);

    assert_eq!(
        client
            .try_remove_alert(&owner, &999u64)
            .unwrap_err()
            .unwrap(),
        ContractError::AlertNotFound
    );
}

// 12. active defaults to true on registration
#[test]
fn test_active_defaults_to_true() {
    let (env, client) = setup();
    let owner = Address::generate(&env);
    let target = Address::generate(&env);

    let id = client.register_alert(&owner, &target, &str(&env, "Alert"), &str(&env, "hash"), &vec![&env]);
    assert!(client.get_alert(&id).unwrap().active);
}

// 13. register_alert rejects more than 50 rules
#[test]
#[should_panic(expected = "too many rules: maximum is 50")]
fn test_register_alert_too_many_rules() {
    let (env, client) = setup();
    let owner = Address::generate(&env);
    let target = Address::generate(&env);

    let mut rules: Vec<String> = vec![&env];
    for _ in 0..51u32 {
        rules.push_back(str(&env, "rule"));
    }
    client.register_alert(&owner, &target, &str(&env, "A"), &str(&env, "h"), &rules);
}

// 14. update_alert rejects more than 50 rules
#[test]
#[should_panic(expected = "too many rules: maximum is 50")]
fn test_update_alert_too_many_rules() {
    let (env, client) = setup();
    let owner = Address::generate(&env);
    let target = Address::generate(&env);

    let id = client.register_alert(&owner, &target, &str(&env, "A"), &str(&env, "h"), &vec![&env]);

    let mut rules: Vec<String> = vec![&env];
    for _ in 0..51u32 {
        rules.push_back(str(&env, "rule"));
    }
    client.update_alert(&owner, &id, &rules, &true);
}

// 15. exactly 50 rules is accepted
#[test]
fn test_register_alert_exactly_50_rules() {
    let (env, client) = setup();
    let owner = Address::generate(&env);
    let target = Address::generate(&env);

    let mut rules: Vec<String> = vec![&env];
    for i in 0..50u32 {
        rules.push_back(str(&env, if i % 2 == 0 { "rule:transfer" } else { "rule:mint" }));
    }
    let id = client.register_alert(&owner, &target, &str(&env, "A"), &str(&env, "h"), &rules);
    assert_eq!(client.get_alert(&id).unwrap().rules.len(), 50);
}

// 16. Label exceeding 128 bytes is rejected
#[test]
#[should_panic(expected = "label exceeds 128 bytes")]
fn test_label_too_long() {
    let (env, client) = setup();
    let owner = Address::generate(&env);
    let target = Address::generate(&env);
    let long_label = str(&env, &"a".repeat(129));
    client.register_alert(&owner, &target, &long_label, &str(&env, "hash"), &vec![&env]);
}

// 17. Label at exactly 128 bytes is accepted
#[test]
fn test_label_max_length_accepted() {
    let (env, client) = setup();
    let owner = Address::generate(&env);
    let target = Address::generate(&env);
    let max_label = str(&env, &"a".repeat(128));
    client.register_alert(&owner, &target, &max_label, &str(&env, "hash"), &vec![&env]);
}

// ── Soroban string-length boundary tests ─────────────────────────────────────
//
// Soroban's String type supports up to 8 192 bytes.  The contract enforces
// its own tighter 128-byte limit on `label`, so any string longer than 128
// bytes must be rejected by the contract guard long before the Soroban
// limit is reached.

// 18. Label of 8 192 bytes (Soroban max) is rejected by the app guard.
#[test]
#[should_panic(expected = "label exceeds 128 bytes")]
fn test_label_at_soroban_max_rejected_by_app_guard() {
    let (env, client) = setup();
    let owner = Address::generate(&env);
    let target = Address::generate(&env);
    let label = str_repeat(&env, 'a', 8192);
    client.register_alert(&owner, &target, &label, &str(&env, "hash"), &vec![&env]);
}

// 19. Label of 8 191 bytes (one below Soroban max) is also rejected by the app guard.
#[test]
#[should_panic(expected = "label exceeds 128 bytes")]
fn test_label_one_below_soroban_max_rejected_by_app_guard() {
    let (env, client) = setup();
    let owner = Address::generate(&env);
    let target = Address::generate(&env);
    let label = str_repeat(&env, 'b', 8191);
    client.register_alert(&owner, &target, &label, &str(&env, "hash"), &vec![&env]);
}

// 20. A Soroban String of exactly 8 192 bytes can be constructed without panicking.
#[test]
fn test_soroban_string_8192_bytes_is_constructible() {
    let (env, _client) = setup();
    let s = str_repeat(&env, 'x', 8192);
    assert_eq!(s.len(), 8192);
}

// ── Feature: renew_alert_ttl ──────────────────────────────────────────────────

// renew_alert_ttl — happy path: owner can renew without changing data
#[test]
fn test_renew_alert_ttl_happy_path() {
    let (env, client) = setup();
    let owner = Address::generate(&env);
    let target = Address::generate(&env);

    let id = client.register_alert(
        &owner,
        &target,
        &str(&env, "Alert"),
        &str(&env, "hash"),
        &vec![&env, str(&env, "rule:transfer")],
    );

    let before = client.get_alert(&id).unwrap();

    // Advance time — renew should NOT change updated_at
    env.ledger().with_mut(|li| li.timestamp += 100);

    assert_eq!(client.try_renew_alert_ttl(&owner, &id).unwrap(), Ok(()));

    let after = client.get_alert(&id).unwrap();

    // Data must be completely unchanged
    assert_eq!(after.label, before.label);
    assert_eq!(after.webhook_hash, before.webhook_hash);
    assert_eq!(after.rules, before.rules);
    assert_eq!(after.active, before.active);
    assert_eq!(after.updated_at, before.updated_at);
    assert_eq!(after.created_at, before.created_at);
}

// renew_alert_ttl — unauthorized caller is rejected
#[test]
fn test_renew_alert_ttl_unauthorized() {
    let (env, client) = setup();
    let owner = Address::generate(&env);
    let attacker = Address::generate(&env);
    let target = Address::generate(&env);

    let id = client.register_alert(
        &owner,
        &target,
        &str(&env, "Alert"),
        &str(&env, "hash"),
        &vec![&env],
    );

    assert_eq!(
        client
            .try_renew_alert_ttl(&attacker, &id)
            .unwrap_err()
            .unwrap(),
        ContractError::Unauthorized
    );
}

// renew_alert_ttl — nonexistent alert returns AlertNotFound
#[test]
fn test_renew_alert_ttl_not_found() {
    let (env, client) = setup();
    let caller = Address::generate(&env);

    assert_eq!(
        client
            .try_renew_alert_ttl(&caller, &999u64)
            .unwrap_err()
            .unwrap(),
        ContractError::AlertNotFound
    );
}

// ── Feature: propose_webhook / confirm_webhook ────────────────────────────────

// propose_webhook — happy path: pending hash is stored, live hash unchanged
#[test]
fn test_propose_webhook_stores_pending_hash() {
    let (env, client) = setup();
    let owner = Address::generate(&env);
    let target = Address::generate(&env);

    let id = client.register_alert(
        &owner,
        &target,
        &str(&env, "Alert"),
        &str(&env, "old-hash"),
        &vec![&env],
    );

    assert_eq!(
        client
            .try_propose_webhook(&owner, &id, &str(&env, "new-hash"))
            .unwrap(),
        Ok(())
    );

    let cfg = client.get_alert(&id).unwrap();
    // Live hash must still be the original
    assert_eq!(cfg.webhook_hash, str(&env, "old-hash"));
    // Pending hash must be set
    assert_eq!(cfg.pending_webhook_hash, Some(str(&env, "new-hash")));
}

// confirm_webhook — happy path: pending hash is promoted to live hash
#[test]
fn test_confirm_webhook_promotes_pending_hash() {
    let (env, client) = setup();
    let owner = Address::generate(&env);
    let target = Address::generate(&env);

    let id = client.register_alert(
        &owner,
        &target,
        &str(&env, "Alert"),
        &str(&env, "old-hash"),
        &vec![&env],
    );

    client.propose_webhook(&owner, &id, &str(&env, "new-hash")).unwrap();

    assert_eq!(client.try_confirm_webhook(&owner, &id).unwrap(), Ok(()));

    let cfg = client.get_alert(&id).unwrap();
    // Live hash must now be the new one
    assert_eq!(cfg.webhook_hash, str(&env, "new-hash"));
    // Pending hash must be cleared
    assert!(cfg.pending_webhook_hash.is_none());
}

// confirm_webhook — returns NoPendingWebhook when no rotation is in progress
#[test]
fn test_confirm_webhook_no_pending_returns_error() {
    let (env, client) = setup();
    let owner = Address::generate(&env);
    let target = Address::generate(&env);

    let id = client.register_alert(
        &owner,
        &target,
        &str(&env, "Alert"),
        &str(&env, "hash"),
        &vec![&env],
    );

    assert_eq!(
        client
            .try_confirm_webhook(&owner, &id)
            .unwrap_err()
            .unwrap(),
        ContractError::NoPendingWebhook
    );
}

// propose_webhook — unauthorized caller is rejected
#[test]
fn test_propose_webhook_unauthorized() {
    let (env, client) = setup();
    let owner = Address::generate(&env);
    let attacker = Address::generate(&env);
    let target = Address::generate(&env);

    let id = client.register_alert(
        &owner,
        &target,
        &str(&env, "Alert"),
        &str(&env, "hash"),
        &vec![&env],
    );

    assert_eq!(
        client
            .try_propose_webhook(&attacker, &id, &str(&env, "evil-hash"))
            .unwrap_err()
            .unwrap(),
        ContractError::Unauthorized
    );
}

// confirm_webhook — unauthorized caller is rejected
#[test]
fn test_confirm_webhook_unauthorized() {
    let (env, client) = setup();
    let owner = Address::generate(&env);
    let attacker = Address::generate(&env);
    let target = Address::generate(&env);

    let id = client.register_alert(
        &owner,
        &target,
        &str(&env, "Alert"),
        &str(&env, "hash"),
        &vec![&env],
    );

    client.propose_webhook(&owner, &id, &str(&env, "new-hash")).unwrap();

    assert_eq!(
        client
            .try_confirm_webhook(&attacker, &id)
            .unwrap_err()
            .unwrap(),
        ContractError::Unauthorized
    );
}

// propose_webhook — nonexistent alert returns AlertNotFound
#[test]
fn test_propose_webhook_not_found() {
    let (env, client) = setup();
    let caller = Address::generate(&env);

    assert_eq!(
        client
            .try_propose_webhook(&caller, &999u64, &str(&env, "hash"))
            .unwrap_err()
            .unwrap(),
        ContractError::AlertNotFound
    );
}

// confirm_webhook — nonexistent alert returns AlertNotFound
#[test]
fn test_confirm_webhook_not_found() {
    let (env, client) = setup();
    let caller = Address::generate(&env);

    assert_eq!(
        client
            .try_confirm_webhook(&caller, &999u64)
            .unwrap_err()
            .unwrap(),
        ContractError::AlertNotFound
    );
}

// propose_webhook — calling propose twice overwrites the pending hash
#[test]
fn test_propose_webhook_overwrites_previous_pending() {
    let (env, client) = setup();
    let owner = Address::generate(&env);
    let target = Address::generate(&env);

    let id = client.register_alert(
        &owner,
        &target,
        &str(&env, "Alert"),
        &str(&env, "original"),
        &vec![&env],
    );

    client.propose_webhook(&owner, &id, &str(&env, "first-pending")).unwrap();
    client.propose_webhook(&owner, &id, &str(&env, "second-pending")).unwrap();

    let cfg = client.get_alert(&id).unwrap();
    assert_eq!(cfg.pending_webhook_hash, Some(str(&env, "second-pending")));
    // Live hash still unchanged
    assert_eq!(cfg.webhook_hash, str(&env, "original"));
}

// Full rotation flow: propose → confirm → propose again → confirm again
#[test]
fn test_webhook_rotation_full_cycle() {
    let (env, client) = setup();
    let owner = Address::generate(&env);
    let target = Address::generate(&env);

    let id = client.register_alert(
        &owner,
        &target,
        &str(&env, "Alert"),
        &str(&env, "v1"),
        &vec![&env],
    );

    // First rotation
    client.propose_webhook(&owner, &id, &str(&env, "v2")).unwrap();
    client.confirm_webhook(&owner, &id).unwrap();
    let cfg = client.get_alert(&id).unwrap();
    assert_eq!(cfg.webhook_hash, str(&env, "v2"));
    assert!(cfg.pending_webhook_hash.is_none());

    // Second rotation
    client.propose_webhook(&owner, &id, &str(&env, "v3")).unwrap();
    client.confirm_webhook(&owner, &id).unwrap();
    let cfg = client.get_alert(&id).unwrap();
    assert_eq!(cfg.webhook_hash, str(&env, "v3"));
    assert!(cfg.pending_webhook_hash.is_none());
}

// propose_webhook emits a wh_prop event
#[test]
fn test_propose_webhook_emits_event() {
    let (env, client) = setup();
    let owner = Address::generate(&env);
    let target = Address::generate(&env);

    let id = client.register_alert(
        &owner,
        &target,
        &str(&env, "Alert"),
        &str(&env, "hash"),
        &vec![&env],
    );

    client.propose_webhook(&owner, &id, &str(&env, "new-hash")).unwrap();

    assert!(!env.events().all().is_empty());
}

// confirm_webhook emits a wh_conf event
#[test]
fn test_confirm_webhook_emits_event() {
    let (env, client) = setup();
    let owner = Address::generate(&env);
    let target = Address::generate(&env);

    let id = client.register_alert(
        &owner,
        &target,
        &str(&env, "Alert"),
        &str(&env, "hash"),
        &vec![&env],
    );

    client.propose_webhook(&owner, &id, &str(&env, "new-hash")).unwrap();
    client.confirm_webhook(&owner, &id).unwrap();

    assert!(!env.events().all().is_empty());
}

// pending_webhook_hash is None on fresh registration
#[test]
fn test_pending_webhook_hash_none_on_registration() {
    let (env, client) = setup();
    let owner = Address::generate(&env);
    let target = Address::generate(&env);

    let id = client.register_alert(
        &owner,
        &target,
        &str(&env, "Alert"),
        &str(&env, "hash"),
        &vec![&env],
    );

    let cfg = client.get_alert(&id).unwrap();
    assert!(cfg.pending_webhook_hash.is_none());
}

// #64 — 10 alerts from the same owner watching the same contract
//
// Registers 10 alerts from a single owner all targeting the same contract.
// Verifies that both the OwnerIndex and the ContractIndex contain exactly
// 10 entries after all registrations.
#[test]
fn test_ten_alerts_same_owner_same_contract() {
    let (env, client) = setup();
    let owner = Address::generate(&env);
    let target = Address::generate(&env);
    // webhook_hash must be exactly 64 characters
    let webhook_hash = str(&env, "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa");
    // 10 distinct labels
    let labels = [
        "Alert 0", "Alert 1", "Alert 2", "Alert 3", "Alert 4",
        "Alert 5", "Alert 6", "Alert 7", "Alert 8", "Alert 9",
    ];

    for label in labels {
        client.register_alert(
            &owner,
            &target,
            &str(&env, label),
            &webhook_hash,
            &vec![&env],
        );
    }

    // Both indexes must contain exactly 10 entries
    assert_eq!(
        client.get_alerts_by_owner(&owner, &owner).unwrap().len(),
        10,
        "owner index must contain exactly 10 entries"
    );
    assert_eq!(
        client.get_alerts_for_contract(&owner, &target).unwrap().len(),
        10,
        "contract index must contain exactly 10 entries"
    );
}

// Counter overflow guard: set counter to u64::MAX and verify CounterOverflow returned
#[test]
fn test_counter_overflow_guard() {
    let (env, client) = setup();
    let owner = Address::generate(&env);
    let target = Address::generate(&env);

    // Set counter to u64::MAX across both storage locations
    env.storage().instance().set(&soroban_sdk::symbol_short!("NEXT_ID"), &u64::MAX);
    env.storage().persistent().set(&DataKey::NextId, &u64::MAX);

    let res = client.try_register_alert(
        &owner,
        &target,
        &str(&env, "alert-overflow"),
        &str(&env, "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"),
        &vec![&env],
    );

    assert_eq!(res.unwrap_err().unwrap(), ContractError::CounterOverflow);
}

// ── Tests for issues #202, #203, #204, #205 ───────────────────────────────────

// #202 — get_alert_count reads the same key as next_id() (instance "NEXT_ID").
// Registers N alerts and asserts get_alert_count == N; removing alerts must not
// decrement the counter (it is monotonic).
#[test]
fn test_202_get_alert_count_matches_registered_count() {
    let (env, client) = setup();
    let owner = Address::generate(&env);
    let target = Address::generate(&env);
    let hash = str(&env, "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa");

    assert_eq!(client.get_alert_count(), 0u64, "#202: count must start at 0");

    let id0 = client.register_alert(&owner, &target, &str(&env, "A"), &hash, &vec![&env]);
    assert_eq!(client.get_alert_count(), 1u64, "#202: count must be 1 after first register");

    client.register_alert(&owner, &target, &str(&env, "B"), &hash, &vec![&env]);
    assert_eq!(client.get_alert_count(), 2u64, "#202: count must be 2 after second register");

    client.register_alert(&owner, &target, &str(&env, "C"), &hash, &vec![&env]);
    assert_eq!(client.get_alert_count(), 3u64, "#202: count must be 3 after third register");

    // Removing an alert must NOT decrement the monotonic counter.
    client.remove_alert(&owner, &id0);
    assert_eq!(client.get_alert_count(), 3u64, "#202: monotonic counter must not decrease after remove");
}

// #203 — removing the last alert owned by an address must delete OwnerIndex,
// not leave an empty vec in storage.
#[test]
fn test_203_owner_index_deleted_when_last_alert_removed() {
    let (env, client) = setup();
    let owner = Address::generate(&env);
    let target = Address::generate(&env);
    let hash = str(&env, "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa");

    let id = client.register_alert(&owner, &target, &str(&env, "Only"), &hash, &vec![&env]);
    // Sanity: owner has one alert.
    assert_eq!(client.get_active_alert_count(&owner), 1u32);

    client.remove_alert(&owner, &id);

    // After removal the OwnerIndex key must not exist in storage (#203).
    let key_present = env
        .storage()
        .persistent()
        .has(&DataKey::OwnerIndex(owner.clone()));
    assert!(!key_present, "#203: OwnerIndex must be deleted when the last alert is removed");
}

// #203 — removing the last alert watching a contract must delete ContractIndex,
// not leave an empty vec in storage.
#[test]
fn test_203_contract_index_deleted_when_last_alert_removed() {
    let (env, client) = setup();
    let owner = Address::generate(&env);
    let target = Address::generate(&env);
    let hash = str(&env, "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa");

    let id = client.register_alert(&owner, &target, &str(&env, "Only"), &hash, &vec![&env]);
    // Sanity: contract has one alert.
    assert_eq!(client.get_alert_count_for_contract(&target), 1u32);

    client.remove_alert(&owner, &id);

    // After removal the ContractIndex key must not exist in storage (#203).
    let key_present = env
        .storage()
        .persistent()
        .has(&DataKey::ContractIndex(target.clone()));
    assert!(!key_present, "#203: ContractIndex must be deleted when the last alert is removed");
}

// #203 — partial removal (owner still has remaining alerts) must keep the index.
#[test]
fn test_203_index_kept_when_alerts_remain() {
    let (env, client) = setup();
    let owner = Address::generate(&env);
    let target = Address::generate(&env);
    let hash = str(&env, "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa");

    let id1 = client.register_alert(&owner, &target, &str(&env, "First"), &hash, &vec![&env]);
    client.register_alert(&owner, &target, &str(&env, "Second"), &hash, &vec![&env]);

    client.remove_alert(&owner, &id1);

    // One alert remains — index must still be present.
    let key_present = env
        .storage()
        .persistent()
        .has(&DataKey::OwnerIndex(owner.clone()));
    assert!(key_present, "#203: OwnerIndex must stay when alerts remain");
    assert_eq!(client.get_active_alert_count(&owner), 1u32);
}

// #204 — confirm_webhook must emit (Symbol("alert"), Symbol("webhook")) — not wh_conf.
#[test]
fn test_204_confirm_webhook_emits_correct_event() {
    let (env, client) = setup();
    let owner = Address::generate(&env);
    let target = Address::generate(&env);

    let id = client.register_alert(
        &owner,
        &target,
        &str(&env, "Alert"),
        &str(&env, "old-hash"),
        &vec![&env],
    );

    client.propose_webhook(&owner, &id, &str(&env, "new-hash")).unwrap();
    client.confirm_webhook(&owner, &id).unwrap();

    // Verify that an event with topics ("alert", "webhook") was emitted.
    let want_a = soroban_sdk::Symbol::new(&env, "alert");
    let want_b = soroban_sdk::Symbol::new(&env, "webhook");
    let found = env.events().all().iter().any(|(_contract, topics, _data)| {
        use soroban_sdk::TryFromVal;
        topics.len() == 2
            && soroban_sdk::Symbol::try_from_val(&env, &topics.get(0).unwrap()).ok() == Some(want_a.clone())
            && soroban_sdk::Symbol::try_from_val(&env, &topics.get(1).unwrap()).ok() == Some(want_b.clone())
    });
    assert!(found, "#204: confirm_webhook must emit (alert, webhook) — not (alert, wh_conf)");
}

// #204 — confirm_webhook promotes pending to live and clears pending.
#[test]
fn test_204_confirm_webhook_promotes_pending_hash() {
    let (env, client) = setup();
    let owner = Address::generate(&env);
    let target = Address::generate(&env);

    let id = client.register_alert(
        &owner,
        &target,
        &str(&env, "Alert"),
        &str(&env, "original"),
        &vec![&env],
    );

    client.propose_webhook(&owner, &id, &str(&env, "rotated")).unwrap();
    client.confirm_webhook(&owner, &id).unwrap();

    let cfg = client.get_alert(&id).unwrap();
    assert_eq!(cfg.webhook_hash, str(&env, "rotated"), "#204: live hash must be promoted");
    assert!(cfg.pending_webhook_hash.is_none(), "#204: pending hash must be cleared after confirm");
}

// #204 — confirm_webhook with no pending hash returns NoPendingWebhook.
#[test]
fn test_204_confirm_webhook_no_pending_returns_error() {
    let (env, client) = setup();
    let owner = Address::generate(&env);
    let target = Address::generate(&env);

    let id = client.register_alert(
        &owner,
        &target,
        &str(&env, "Alert"),
        &str(&env, "hash"),
        &vec![&env],
    );

    let result = client.try_confirm_webhook(&owner, &id);
    assert_eq!(
        result.unwrap_err().unwrap(),
        ContractError::NoPendingWebhook,
        "#204: must return NoPendingWebhook when no rotation is in progress"
    );
}

// #205 — stale/expired IDs are pruned from the owner index during limit check.
// Simulates expiry by manually removing the Alert entry from storage, then
// verifies the owner can register a new alert without hitting the limit.
#[test]
fn test_205_stale_ids_pruned_from_owner_index_during_limit_check() {
    let (env, client) = setup();
    let admin = Address::generate(&env);
    let owner = Address::generate(&env);
    let target = Address::generate(&env);
    let hash = str(&env, "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa");

    client.initialize(&admin);
    // Limit of 1: owner may only have 1 live alert at a time.
    client.set_per_owner_alert_limit(&admin, &1u32);

    let id = client.register_alert(&owner, &target, &str(&env, "First"), &hash, &vec![&env]);

    // Simulate TTL expiry by directly removing the Alert entry from persistent storage,
    // while leaving the OwnerIndex intact (as would happen on ledger archival).
    env.storage().persistent().remove(&DataKey::Alert(id));

    // The index still contains the stale ID, but the alert is gone.
    // assert_per_owner_limit must prune it and allow the new registration.
    let result = client.try_register_alert(
        &owner,
        &target,
        &str(&env, "Second"),
        &hash,
        &vec![&env],
    );
    assert!(result.is_ok(), "#205: registration must succeed after stale IDs are pruned from owner index");
}

// #205 — after pruning, the owner index should only contain live IDs.
#[test]
fn test_205_index_is_compacted_after_stale_prune() {
    let (env, client) = setup();
    let admin = Address::generate(&env);
    let owner = Address::generate(&env);
    let target = Address::generate(&env);
    let hash = str(&env, "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa");

    client.initialize(&admin);
    client.set_per_owner_alert_limit(&admin, &3u32);

    // Register 2 alerts.
    let id0 = client.register_alert(&owner, &target, &str(&env, "A"), &hash, &vec![&env]);
    let id1 = client.register_alert(&owner, &target, &str(&env, "B"), &hash, &vec![&env]);

    // Simulate expiry of both.
    env.storage().persistent().remove(&DataKey::Alert(id0));
    env.storage().persistent().remove(&DataKey::Alert(id1));

    // Register a new alert — this triggers pruning of the 2 stale IDs.
    client.register_alert(&owner, &target, &str(&env, "C"), &hash, &vec![&env]);

    // Active count must now be exactly 1 (the new alert).
    assert_eq!(
        client.get_active_alert_count(&owner),
        1u32,
        "#205: after pruning, only 1 live alert must remain"
    );
}
