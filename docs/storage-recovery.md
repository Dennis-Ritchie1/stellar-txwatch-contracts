# Recovering archived alert storage

Soroban archives persistent entries whose TTL has run out. An archived alert is not deleted,
but reads no longer return it until it is restored. This page is the operator procedure for
finding archived entries and restoring them. The TTL rules are in [`ttl.md`](./ttl.md).

## 1. Detect archived entries

An entry is archived when its TTL has passed and no one has extended it. From an operator
machine:

1. Note the alert's config ID and the `AlertRegistry` contract ID.
2. Call `get_alert` with the config ID. A `None` result is either a removed alert or an
   archived one, so check the next step before concluding anything.
3. Read the entry's ledger key from Horizon or Soroban RPC. An archived entry is reported
   as archived, not as missing.

## 2. RestoreFootprintOp

`RestoreFootprintOp` is the Soroban operation that moves archived entries back into live state.
It has a fee, which depends on the size of the restored entries and the number of ledgers they
are extended for. Restoring costs more than a normal write, so do it only for alerts you need.

Restore before an alert is archived where you can: a keeper that bumps TTL (see section 5) avoids
the fee entirely.

## 3. Restore with the Stellar CLI

```sh
stellar contract restore \
  --id <ALERT_REGISTRY_CONTRACT_ID> \
  --key <ENTRY_KEY> \
  --network testnet \
  --source <OPERATOR_KEY_NAME>
```

Check `stellar contract restore --help` for the flags in your CLI version. Run the restore on
testnet first.

## 4. Restore with the JavaScript SDK

Build a transaction that contains a `RestoreFootprintOp` over the archived keys, simulate it to
get the resource fee, then sign and submit it:

1. Build the transaction with the keys in its footprint.
2. Simulate it with the RPC server. The simulation returns the restore fee.
3. Add the fee and sign with the operator key.
4. Submit, then poll until the transaction is confirmed.
5. Call `get_alert` again to confirm the alert is readable.

The exact builder calls depend on the SDK version. Follow the SDK's own examples for
`RestoreFootprintOp`.

## 5. Prevention

Archival is avoided, not repaired, by keeping TTL up:

- Run a keeper that calls `bump_alert` for alerts before their TTL expires.
- Renew long-lived alerts through `renew_alert_ttl`.
- Monitor expiry with the procedure in the keeper's own runbook, and alert on any alert within
  one day of expiry.

## 6. Checklist

- Confirm an entry is archived before restoring it.
- Test the restore on testnet first.
- Verify with `get_alert` afterwards.
- Keep a keeper running so restores are rare.
