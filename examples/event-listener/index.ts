/**
 * Listens for AlertRegistry events from Soroban RPC and prints each one (#258).
 *
 * Run with:
 *   ALERT_REGISTRY_ID=<C...> npx tsx index.ts
 *
 * The loop keeps a cursor, so a restart resumes where it stopped. Handlers must be idempotent,
 * because a crash between handling and saving the cursor replays a few events.
 */

import { SorobanRpc, nativeToScVal, scValToNative } from "@stellar/stellar-sdk";

const RPC_URL = process.env.SOROBAN_RPC_URL ?? "https://soroban-testnet.stellar.org";
const CONTRACT_ID = process.env.ALERT_REGISTRY_ID;
const POLL_MS = 5_000;

/** Base64 XDR of each topic symbol, the form the RPC filter expects. */
const sym = (name: string) => nativeToScVal(name, { type: "symbol" }).toXDR("base64");

if (!CONTRACT_ID) {
  throw new Error("Set ALERT_REGISTRY_ID to the AlertRegistry contract address");
}

const server = new SorobanRpc.Server(RPC_URL);

let cursor: string | undefined;

/** Prints one event. Replace the body with a real handler. */
function handle(ev: SorobanRpc.Api.EventResponse): void {
  const topics = ev.topic.map((t) => String(scValToNative(t)));
  const data = scValToNative(ev.value);
  console.log(JSON.stringify({ ledger: ev.ledger, topics, data }));
}

async function poll(): Promise<void> {
  const page = await server.getEvents({
    startLedger: cursor ? undefined : 0,
    cursor,
    filters: [
      {
        type: "contract",
        contractIds: [CONTRACT_ID!],
        topics: [[sym("alert"), sym("register")]],
      },
    ],
    limit: 100,
  });

  for (const ev of page.events) {
    handle(ev);
    cursor = ev.pagingToken;
  }
}

async function main(): Promise<void> {
  // The first call uses startLedger; later calls use the cursor.
  for (;;) {
    try {
      await poll();
    } catch (err) {
      console.error("poll failed, retrying", err);
    }
    await new Promise((resolve) => setTimeout(resolve, POLL_MS));
  }
}

void main();
