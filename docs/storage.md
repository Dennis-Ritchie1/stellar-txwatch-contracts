# Storage Reference

This document describes every storage key used by both contracts, its value type, storage tier (instance vs persistent), and TTL behavior.

---

## AlertRegistry

Source: `contracts/alert-registry/src/lib.rs`

### Storage Keys

| Key | Tier | Value Type | Description |
|---|---|---|---|
| `DataKey::Alert(id: u64)` | Persistent | `AlertConfig` | A single alert configuration, keyed by its numeric ID. The `active` flag is stored inside `AlertConfig` |
| `DataKey::OwnerIndex(addr: Address)` | Persistent | `Vec<u64>` | List of alert IDs owned by a given address |
| `DataKey::ContractIndex(addr: Address)` | Persistent | `Vec<u64>` | List of alert IDs watching a given contract address |
| `symbol_short!("NEXT_ID")` | Instance | `u64` | Monotonic counter used to generate unique alert IDs (uses `checked_add(1)`, returning `ContractError::CounterOverflow` on `u64::MAX`) |
| `symbol_short!("ADMIN")` | Instance | `Address` | Optional admin address that may remove alerts and set owner limits |
| `symbol_short!("LIMIT")` | Instance | `u32` | Optional per-owner active alert limit |
| `symbol_short!("WATCHREG")` | Instance | `Address` | Optional `WatcherRegistry` contract address; when set, read queries are gated to registered watchers |

### AlertConfig Fields

| Field | Type | Description |
|---|---|---|
| `label` | `String` | Human-readable name for the alert (max 128 bytes) |
| `webhook_hash` | `String` | SHA-256 hex digest of the webhook URL |
| `rules` | `Vec<String>` | Rule descriptor strings (e.g. `"rule:transfer"`) |
| `owner` | `Address` | Address that owns and may mutate this alert |
| `target_contract` | `Address` | Contract address being watched |
| `created_at` | `u64` | Ledger timestamp at registration |
| `updated_at` | `u64` | Ledger timestamp of the most recent update |
| `active` | `bool` | Whether the alert is currently active |
| `pending_webhook_hash` | `Option<String>` | Pending webhook hash proposed via `propose_webhook`, not yet confirmed. `None` when no rotation is in progress. |

### TTL Behavior

All three persistent key variants (`Alert`, `OwnerIndex`, `ContractIndex`) are extended by **100 ledgers** (≈ 8 minutes at 5 s/ledger) on every write that touches them.

| Function | Keys Extended |
|---|---|
| `register_alert` | `Alert(id)`, `OwnerIndex(owner)`, `ContractIndex(target)` |
| `update_alert` | `Alert(id)` |
| `update_webhook` | `Alert(id)` |
| `propose_webhook` | `Alert(id)` |
| `confirm_webhook` | `Alert(id)` |
| `renew_alert_ttl` | `Alert(id)`, `OwnerIndex(owner)`, `ContractIndex(target)` — data unchanged |
| `remove_alert` | Entries deleted — no TTL extension |

Read-only functions (`get_alert`, `get_alerts_for_contract`, `get_alerts_by_owner`, paginated variants, `get_alert_count`) do **not** extend any TTL.

The `NEXT_ID` instance key has no explicit TTL management — its lifetime is tied to the contract instance itself.

> See [docs/ttl.md](ttl.md) for implications of the 100-ledger setting and recommended production values.

---

## WatcherRegistry

Source: `contracts/watcher-registry/src/lib.rs`

### Storage Keys
| Key | Tier | Value Type | Description |
|---|---|---|---|
| `symbol_short!("ADMIN")` | Instance | `Address` | Current admin address |
| `symbol_short!("WATCHERS")` | Instance | `Vec<Address>` | List of authorized watcher node addresses |

### TTL Behavior

WatcherRegistry uses **instance storage exclusively**. To prevent archival during low-activity periods, the contract explicitly extends the instance TTL on every mutating function and on the hot read path `is_authorized`.

The extension uses the `Default_TTL` constant defined in `contracts/watcher-registry/src/storage.rs` (currently 518,400 ledgers, ~30 days at 5 s/ledger) and is applied via `env: storage().instance().extend_ttl(DEFAULT_TTL, DEFAULT_TTL)`.

| Function | TTL Extended |
|---|---|
| `is_authorized` | Yes |
| `register_watcher` | Yes |
| `remove_watcher` | Yes |
| `add_admin` | Yes |
| `remove_admin` | Yes |
| `set_admin` | Yes |

There are no persistent storage entries in WatcherRegistry.

---

## Storage Tier Summary

| Contract | Tier | Keys | TTL Managed By |
|---|---|---|---|
| AlertRegistry | Persistent | `Alert`, `OwnerIndex`, `ContractIndex` | Contract (`extend_ttl`, 100 ledgers) |
| AlertRegistry | Instance | `NEXT_ID` | Network |
| WatcherRegistry | Instance | `ADMIN`, `WATCHERS` | Contract (`extend_ttl`, `DEFAULT_TTL` ledgers) |

## Source of truth for keys

`AlertRegistry` keys are defined in `contracts/alert-registry/src/lib.rs`, the only module the
contract compiles from `lib.rs`. `AlertActive(id)` is the separate active flag, and `NEXT_ID`
is the instance counter. The files `types.rs`, `storage.rs` and `contract.rs` are not declared in
`lib.rs`, and their older `DataKey` definitions do not apply to the deployed contract. Read this
document against `lib.rs`, not against those files.
