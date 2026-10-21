# @tx-wat/watcher-registry

[![npm version](https://img.shields.io/npm/v/@tx-wat/watcher-registry.svg)](https://www.npmjs.com/package/@tx-wat/watcher-registry)

TypeScript bindings for the WatcherRegistry Soroban smart contract.

## Install

```sh
npm install @tx-wat/watcher-registry
```

## Testnet contract ID

The testnet contract ID is exposed via `networks.testnet.contractId` and is
overlaid at publish time from `DEPLOYMENTS.md`. See that file for the current
deployed address.

## Usage

All examples below import from the package root, initialize a `Client` against
testnet, perform a typed contract call, and read the result. Every call is
wrapped in `try/catch` to surface the generated error types.

### 1. Initialize the registry

```ts
import { Client, networks } from "@tx-wat/watcher-registry";

const client = new Client({
  contractId: networks.testnet.contractId,
  networkPassphrase: networks.testnet.networkPassphrase,
  rpcUrl: networks.testnet.rpcUrl,
});

try {
  const admin = "G...ADMIN_ADDRESS";
  const tx = await client.initialize({ admin });
  const result = await tx.signAndSend();
  console.log("initialized:", result);
} catch (err) {
  console.error("initialize failed:", err);
}
```

### 2. Add / remove an admin

```ts
import { Client, networks } from "@tx-wat/watcher-registry";

const client = new Client({
  contractId: networks.testnet.contractId,
  networkPassphrase: networks.testnet.networkPassphrase,
  rpcUrl: networks.testnet.rpcUrl,
});

try {
  const admin = "G...NEW_ADMIN_ADDRESS";
  const addTx = await client.add_admin({ admin });
  const addResult = await addTx.signAndSend();
  console.log("admin added:", addResult);

  const removeTx = await client.remove_admin({ admin });
  const removeResult = await removeTx.signAndSend();
  console.log("admin removed:", removeResult);
} catch (err) {
  console.error("admin update failed:", err);
}
```

### 3. Register and remove a watcher

```ts
import { Client, networks } from "@tx-wat/watcher-registry";

const client = new Client({
  contractId: networks.testnet.contractId,
  networkPassphrase: networks.testnet.networkPassphrase,
  rpcUrl: networks.testnet.rpcUrl,
});

try {
  const watcher = "G...WATCHER_ADDRESS";
  const registerTx = await client.register_watcher({ watcher });
  const registerResult = await registerTx.signAndSend();
  console.log("watcher registered:", registerResult);

  const removeTx = await client.remove_watcher({ watcher });
  const removeResult = await removeTx.signAndSend();
  console.log("watcher removed:", removeResult);
} catch (err) {
  console.error("watcher update failed:", err);
}
```

### 4. Check authorization (is_authorized)

```ts
import { Client, networks } from "@tx-wat/watcher-registry";

const client = new Client({
  contractId: networks.testnet.contractId,
  networkPassphrase: networks.testnet.networkPassphrase,
  rpcUrl: networks.testnet.rpcUrl,
});

try {
  const watcher = "G...WATCHER_ADDRESS";
  const authorized = await client.is_authorized({ watcher });
  console.log("authorized:", authorized); // boolean
} catch (err) {
  console.error("is_authorized failed:", err);
}
```

### 5. List all watchers (get_all_watchers)

```ts
import { Client, networks } from "@tx-wat/watcher-registry";

const client = new Client({
  contractId: networks.testnet.contractId,
  networkPassphrase: networks.testnet.networkPassphrase,
  rpcUrl: networks.testnet.rpcUrl,
});

try {
  const watchers = await client.get_all_watchers();
  console.log("watchers:", watchers); // string[]
} catch (err) {
  console.error("get_all_watchers failed:", err);
}
```

### 6. Get watcher count

```ts
import { Client, networks } from "@tx-wat/watcher-registry";

const client = new Client({
  contractId: networks.testnet.contractId,
  networkPassphrase: networks.testnet.networkPassphrase,
  rpcUrl: networks.testnet.rpcUrl,
});

try {
  const count = await client.get_watcher_count();
  console.log("watcher count:", count); // number
} catch (err) {
  console.error("get_watcher_count failed:", err);
}
```
