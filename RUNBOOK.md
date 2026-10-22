# Operator runbook

For contract operators. Each procedure lists the steps, what to check, and what to tell users.
Commands assume the Stellar CLI and the `scripts/` helpers in this repo.

## 1. Initial deployment

1. Build the WASM: `cargo build --release --target wasm32-unknown-unknown`.
2. Deploy with `scripts/deploy.sh`. It prints the contract addresses.
3. Initialise each contract with its admin address (`initialize`). A second call fails with
   `AlreadyInitialized`.
4. Record the addresses and the WASM hash in `DEPLOYMENTS.md`, and commit the change.
5. Check: `get_admin` on `AlertRegistry` returns the admin you set, and `get_admins` on
   `WatcherRegistry` returns one admin.

## 2. Contract upgrade

1. Build and deploy the new WASM as above.
2. Confirm that a stored alert still reads back with `get_alert`. Check `scripts/upgrade.sh` for
   any steps it expects before you run it.
3. Update `DEPLOYMENTS.md` with the new WASM hash.

Stored data is kept across an upgrade only when the storage keys did not change. Do not change a
`DataKey` variant in place.

## 3. Admin key rotation

- **AlertRegistry:** `transfer_admin(admin, new_admin)`, signed by the current admin. Check
  `get_admin` returns `new_admin`. The old key is refused from then on. The transfer emits
  `admin/transfer`.
- **WatcherRegistry:** `add_admin` for the new key first, then `remove_admin` for the old one. Add
  before removing, so the set never has zero admins. `remove_admin` refuses the last admin.

## 4. Watcher onboarding

1. The admin calls `register_watcher(admin, watcher)` for the new node.
2. Check: `is_authorized(watcher)` returns `true`.
3. Check the watcher can read through `AlertRegistry` once the gate is set: the call succeeds
   with the watcher as `querier`.
4. Keep `get_watcher_count` matching the number of nodes you expect.

## 5. Emergency response

The compiled contract has **no pause function**. The tools that exist:

- `remove_alert_by_admin(admin, config_id)` removes one alert.
- `deactivate_all_alerts(caller)` deactivates the alerts the caller owns.
- `set_watcher_registry` with the zero address disables watcher gating, if gating is causing the
  incident. The registry is then open to reads.
- `clear_all_watchers(admin)` removes every watcher, which stops all gated reads.

Before using any of these, tell users what is happening and what they must do, because an alert
removed or deactivated cannot be undone by the contract.

## 6. Storage recovery

If an alert has expired and reads as absent, follow [`docs/storage-recovery.md`](docs/storage-recovery.md).
Keep a keeper running so this is rare.
