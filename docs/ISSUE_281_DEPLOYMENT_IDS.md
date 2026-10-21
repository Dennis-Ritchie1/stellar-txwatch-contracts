# Issue #281: Testnet contract IDs in `DEPLOYMENTS.md` (blocked)

**Status:** blocked. The real contract IDs come from running `scripts/deploy.sh` against testnet, which
was not done here.

## Why it is blocked

`DEPLOYMENTS.md` holds placeholder addresses (`CXXXX…`). The repository has no real testnet IDs, and an
address cannot be made up. Testnet also resets periodically, so any IDs recorded now have a limited life.

## What is needed

1. Run `scripts/deploy.sh` on testnet for `AlertRegistry` and `WatcherRegistry`.
2. Record for each: the contract ID, network, deploy date, WASM hash, admin address, and deployer.
3. Replace the placeholder rows in `DEPLOYMENTS.md` with those values, and commit.

## Not done

Placeholders left as they are. No values were invented.
