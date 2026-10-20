# Architecture

This page explains how the two contracts are built and why. For function signatures see
[`docs/alert-registry.md`](docs/alert-registry.md) and [`docs/watcher-registry.md`](docs/watcher-registry.md).

## Module structure

Each contract is one compiled module, `contracts/<name>/src/lib.rs`. It holds the `#[contract]`
struct, the `#[contractimpl]` entry points, the data types, and the storage keys.

`contracts/alert-registry/src/` also contains `contract.rs`, `storage.rs` and `types.rs`. Those
files are **not compiled**: `lib.rs` does not declare them as modules. They hold an older split
of the same code, and some of their definitions (webhook propose and confirm, `renew_alert_ttl`)
are not in the deployed contract. Read the compiled `lib.rs` as the source of truth. The tests in
`tests.rs` are compiled, as `contract_tests`.

## Storage tiers

| Contract | Data | Tier | Why |
|---|---|---|---|
| AlertRegistry | `Alert`, `AlertActive`, `OwnerIndex`, `ContractIndex` | Persistent | Entries can grow and must be able to expire. Each has its own TTL, extended on writes and by `bump_alert`. |
| AlertRegistry | `ADMIN`, `LIMIT`, `WATCHREG`, `NEXT_ID` | Instance | Small, fixed settings and a counter. They live as long as the contract instance. |
| WatcherRegistry | `Admins`, `Watchers`, `W_CNT` | Instance | The whole set is read on most calls, and the set is small. |

An expired persistent entry reads as absent, so reads and counts ignore it. An alert is not
removed from the owner index when it expires, which is why the count helpers check that the alert
still exists.

## Admin model

- **AlertRegistry** has one admin, stored under `ADMIN`. `transfer_admin` replaces that one address.
- **WatcherRegistry** has a set of admins. `add_admin` and `remove_admin` change the set, and
  `remove_admin` refuses to remove the last admin (`LastAdmin`).

The two designs differ because the watcher registry gates who may read alerts, and an operator set
needs more than one key so that losing one key does not lock the registry.

## Access gating

When `set_watcher_registry` has been called, the read functions in `AlertRegistry` make a
cross-contract call to `WatcherRegistry::is_watcher_authorized` before returning data. A caller that
is not a registered watcher gets `NotAWatcher`. With no registry set, reads are open.

## Events

Events are published as a two-symbol topic (`namespace`, `action`) and a small data tuple. The
data is kept small so that indexers can decode it cheaply. Namespaces group related events: `alert`
for alert lifecycle, `admin` for role changes, and `watcher` for watcher membership. The shapes are
in [`docs/events.md`](docs/events.md).

## Errors

Each contract returns a typed `ContractError` with a fixed numeric code, so callers can match on the
code rather than on message text. Codes are never reused.
