#!/usr/bin/env bash
# scripts/verify.sh — Verify deployed contract state and WASM hash
# Usage: ./scripts/verify.sh --contract alert-registry|watcher-registry --contract-id <ID> [--network testnet|mainnet]
#        ./scripts/verify.sh --network testnet|mainnet   (full post-deploy verification)
set -euo pipefail

NETWORK="${NETWORK:-testnet}"
CONTRACT=""
CONTRACT_ID=""
ALERT_CONTRACT_ID="${ALERT_CONTRACT_ID:-}"
WATCHER_CONTRACT_ID="${WATCHER_CONTRACT_ID:-}"

while [[ $# -gt 0 ]]; do
  case "$1" in
    --network)            NETWORK="$2";            shift 2 ;;
    --contract)           CONTRACT="$2";           shift 2 ;;
    --contract-id)        CONTRACT_ID="$2";        shift 2 ;;
    --alert-contract-id)  ALERT_CONTRACT_ID="$2";  shift 2 ;;
    --watcher-contract-id) WATCHER_CONTRACT_ID="$2"; shift 2 ;;
    *) echo "Unknown argument: $1"; exit 1 ;;
  esac
done

case "$NETWORK" in
  testnet)
    RPC_URL="https://soroban-testnet.stellar.org"
    NETWORK_PASSPHRASE="Test SDF Network ; September 2015"
    ;;
  mainnet)
    RPC_URL="${MAINNET_RPC_URL:?MAINNET_RPC_URL must be set for mainnet}"
    NETWORK_PASSPHRASE="Public Global Stellar Network ; September 2015"
    ;;
  *)
    echo "Unsupported network: $NETWORK"; exit 1 ;;
esac

echo "==> Network: $NETWORK"

# ---------------------------------------------------------------------------
# Single-contract WASM hash verification mode (legacy behaviour)
# ---------------------------------------------------------------------------
if [[ -n "$CONTRACT" || -n "$CONTRACT_ID" ]]; then
  [[ -z "$CONTRACT" ]]    && { echo "Error: --contract required (alert-registry|watcher-registry)"; exit 1; }
  [[ -z "$CONTRACT_ID" ]] && { echo "Error: --contract-id required"; exit 1; }

  WASM_NAME="${CONTRACT//-/_}"
  WASM="target/wasm32-unknown-unknown/release/${WASM_NAME}.wasm"

  echo "==> Building contracts..."
  cargo build --release --target wasm32-unknown-unknown

  LOCAL_HASH=$(sha256sum "$WASM" | awk '{print $1}')
  echo "==> Local WASM hash:    $LOCAL_HASH"

  DEPLOYED_HASH=$(stellar contract info \
    --id "$CONTRACT_ID" \
    --network "$NETWORK" \
    --rpc-url "$RPC_URL" \
    --network-passphrase "$NETWORK_PASSPHRASE" \
    | grep -i "wasm hash" | awk '{print $NF}' | tr -d '"')
  echo "==> Deployed WASM hash: $DEPLOYED_HASH"

  if [[ "$LOCAL_HASH" == "$DEPLOYED_HASH" ]]; then
    echo "==> MATCH: deployed contract matches local build."
  else
    echo "==> MISMATCH: deployed contract does NOT match local build!"
    exit 1
  fi
  exit 0
fi

# ---------------------------------------------------------------------------
# Full post-deploy verification mode
# ---------------------------------------------------------------------------
[[ -z "$ALERT_CONTRACT_ID" ]]   && { echo "Error: --alert-contract-id required (or set ALERT_CONTRACT_ID)"; exit 1; }
[[ -z "$WATCHER_CONTRACT_ID" ]] && { echo "Error: --watcher-contract-id required (or set WATCHER_CONTRACT_ID)"; exit 1; }

FAILED=0
pass() { echo "  [PASS] $1"; }
fail() { echo "  [FAIL] $1"; FAILED=1; }

invoke() {
  local id="$1"; shift
  stellar contract invoke \
    --id "$id" \
    --network "$NETWORK" \
    --rpc-url "$RPC_URL" \
    --network-passphrase "$NETWORK_PASSPHRASE" \
    -- "$@" 2>/dev/null
}

# 1. Both contracts deployed and accessible
echo "==> Check 1: contracts deployed and accessible"
if invoke "$ALERT_CONTRACT_ID" get_admin >/dev/null; then
  pass "AlertRegistry reachable ($ALERT_CONTRACT_ID)"
else
  fail "AlertRegistry not reachable ($ALERT_CONTRACT_ID)"
fi
if invoke "$WATCHER_CONTRACT_ID" get_admins >/dev/null; then
  pass "WatcherRegistry reachable ($WATCHER_CONTRACT_ID)"
else
  fail "WatcherRegistry not reachable ($WATCHER_CONTRACT_ID)"
fi

# 2. AlertRegistry get_admin returns a valid address
echo "==> Check 2: AlertRegistry get_admin returns a valid address"
ADMIN=$(invoke "$ALERT_CONTRACT_ID" get_admin || true)
if [[ -n "$ADMIN" && "$ADMIN" != "null" ]]; then
  pass "get_admin -> $ADMIN"
else
  fail "get_admin returned empty/invalid address"
fi

# 3. WatcherRegistry get_admins returns at least one address
echo "==> Check 3: WatcherRegistry get_admins returns at least one address"
ADMINS=$(invoke "$WATCHER_CONTRACT_ID" get_admins || true)
if [[ -n "$ADMINS" && "$ADMINS" != "[]" && "$ADMINS" != "null" ]]; then
  pass "get_admins -> $ADMINS"
else
  fail "get_admins returned no addresses"
fi

# 4. Round-trip: register an alert, retrieve it, remove it
echo "==> Check 4: alert round-trip (register -> retrieve -> remove)"
ROUNDTRIP_ID="verify-$(date +%s)"
if invoke "$ALERT_CONTRACT_ID" register_alert \
     --alert_id "$ROUNDTRIP_ID" \
     --message "verification round-trip" >/dev/null; then
  pass "register_alert($ROUNDTRIP_ID)"
  RETRIEVED=$(invoke "$ALERT_CONTRACT_ID" get_alert --alert_id "$ROUNDTRIP_ID" || true)
  if [[ -n "$RETRIEVED" && "$RETRIEVED" != "null" ]]; then
    pass "get_alert($ROUNDTRIP_ID) -> $RETRIEVED"
  else
    fail "get_alert($ROUNDTRIP_ID) returned nothing"
  fi
  if invoke "$ALERT_CONTRACT_ID" remove_alert --alert_id "$ROUNDTRIP_ID" >/dev/null; then
    pass "remove_alert($ROUNDTRIP_ID)"
  else
    fail "remove_alert($ROUNDTRIP_ID) failed"
  fi
else
  fail "register_alert($ROUNDTRIP_ID) failed"
fi

# 5. is_authorized returns false for a random address
echo "==> Check 5: is_authorized returns false for a random address"
RANDOM_ADDR="G$(head -c 32 /dev/urandom | base32 | tr -d '=' | head -c 55)"
AUTH=$(invoke "$ALERT_CONTRACT_ID" is_authorized --address "$RANDOM_ADDR" || true)
if [[ "$AUTH" == "false" ]]; then
  pass "is_authorized($RANDOM_ADDR) -> false"
else
  fail "is_authorized($RANDOM_ADDR) -> '$AUTH' (expected false)"
fi

# 6. WASM hash matches expected value from DEPLOYMENTS.md
echo "==> Check 6: WASM hash matches DEPLOYMENTS.md"
DEPLOYMENTS_FILE="$(dirname "$0")/../DEPLOYMENTS.md"
if [[ -f "$DEPLOYMENTS_FILE" ]]; then
  for pair in "alert-registry:$ALERT_CONTRACT_ID" "watcher-registry:$WATCHER_CONTRACT_ID"; do
    name="${pair%%:*}"; id="${pair##*:}"
    EXPECTED=$(grep -i "$name" "$DEPLOYMENTS_FILE" | grep -ioE '[0-9a-f]{64}' | head -n1 || true)
    if [[ -z "$EXPECTED" ]]; then
      fail "no expected WASM hash for $name in DEPLOYMENTS.md"
      continue
    fi
    ACTUAL=$(stellar contract info \
      --id "$id" \
      --network "$NETWORK" \
      --rpc-url "$RPC_URL" \
      --network-passphrase "$NETWORK_PASSPHRASE" \
      | grep -i "wasm hash" | awk '{print $NF}' | tr -d '"' || true)
    if [[ "$EXPECTED" == "$ACTUAL" ]]; then
      pass "$name WASM hash matches ($ACTUAL)"
    else
      fail "$name WASM hash mismatch (expected $EXPECTED, got $ACTUAL)"
    fi
  done
else
  fail "DEPLOYMENTS.md not found at $DEPLOYMENTS_FILE"
fi

echo ""
if [[ "$FAILED" -eq 0 ]]; then
  echo "==> ALL CHECKS PASSED"
  exit 0
else
  echo "==> VERIFICATION FAILED"
  exit 1
fi
