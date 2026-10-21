# Event listener example

Prints `AlertRegistry` events from Soroban RPC as they are emitted. Start from here when you
build an integration that reacts to alert changes.

## Run

```sh
npm install @stellar/stellar-sdk tsx
ALERT_REGISTRY_ID=<C...> npx tsx index.ts
```

`SOROBAN_RPC_URL` defaults to the testnet RPC. Set it to point at another node.

## What it filters

The filter matches events from `ALERT_REGISTRY_ID` whose first topic is `alert` and second topic
is `register`. Change the second topic to listen for another action, for example `remove` or
`bump`. The full list is in [`docs/events.md`](../../docs/events.md).

The topic values are built with `nativeToScVal(..., { type: "symbol" })` from the SDK, so the filter
matches the same XDR the network emits.

## Cursor and restarts

The example keeps the paging token of the last event in memory. A restart begins from ledger 0,
so store the cursor to a file or database before using it in production. See
[`docs/event-indexing.md`](../../docs/event-indexing.md) for the full pattern.

## Limits

- The handler only prints. It does not verify the event against the ledger.
- The base64 topic values are not checked against a live contract.
- Not tested against a running testnet deployment.
