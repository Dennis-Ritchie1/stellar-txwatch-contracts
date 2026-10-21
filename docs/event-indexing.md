# Indexing contract events

This guide is for off-chain services that consume `alert-registry` and `watcher-registry`
events. The event shapes are in [`events.md`](./events.md). This page explains how to
subscribe, filter, page and handle duplicates.

## 1. Where the events come from

Contract events are read from Soroban RPC (`getEvents`) or from Horizon's Soroban event
endpoint. Pick one source per consumer and keep the cursor for that source.

- **Soroban RPC `getEvents`** returns events for a ledger range and a filter.
- **Horizon** exposes the same events through its Soroban endpoints. Check the current Horizon
  reference for the exact path and parameters before deploying, since the endpoint names have
  changed between releases.

## 2. Filtering by topic

Every event has topic 0 (a namespace such as `alert`, `watcher` or `admin`) and topic 1 (the
action). Filter on the contract ID and both topics, so unrelated events are not fetched:

```text
contract: <AlertRegistry contract ID>
topics:   ["alert", "register"]
```

Use `*` in a topic position to match any value there.

## 3. Cursor-based pagination

Each event carries a paging token. Store the token of the last event you have fully handled,
and pass it back as the cursor on the next request. Do not store the ledger number alone: several
events can share one ledger.

```text
loop:
  page   = fetch_events(filter, cursor)
  for ev in page.events:
      handle(ev)
      cursor = ev.paging_token
  save(cursor)
  if page is empty: sleep(poll interval)
```

Save the cursor only after the events in the page are handled. A crash between the two then
replays a few events, which the idempotency rule below absorbs.

## 4. Idempotency

A replay can deliver an event twice. Key every write on the event's transaction hash and its
index within the transaction, and treat a repeat as a no-op:

```text
key = (tx_hash, event_index)
if key in processed: skip
else: write the change, record the key, in one transaction
```

Events for an alert can be applied in any order only if the handler reads the current state
first. Where order matters (for example `alert.register` then `alert.remove` for the same ID),
process events in ledger order.

## 5. Example: Node.js subscriber

This sketch uses the `@stellar/stellar-sdk` RPC client. Check the SDK version you install, since
method names differ between major releases.

```ts
import { SorobanRpc } from "@stellar/stellar-sdk";

const server = new SorobanRpc.Server("https://soroban-testnet.stellar.org");
const contractId = process.env.ALERT_REGISTRY_ID!;

let cursor: string | undefined;

async function poll(): Promise<void> {
  const page = await server.getEvents({
    filters: [{ type: "contract", contractIds: [contractId], topics: [["alert", "register"]] }],
    cursor,
    limit: 100,
  });

  for (const ev of page.events) {
    await handle(ev.txHash, ev.id, ev.value); // handle() must be idempotent
    cursor = ev.pagingToken;
  }
}

setInterval(() => void poll(), 5_000);
```

## 6. Checklist

- Filter on the contract ID and the topics you need.
- Store the paging token after each handled page.
- Key writes on `(tx_hash, event_index)`.
- Process in ledger order when one entity has several events.
