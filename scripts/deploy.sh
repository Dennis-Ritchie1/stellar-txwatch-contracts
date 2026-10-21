#!/usr/bin/env bash
# scripts/deploy.sh — Deploy both contracts to Stellar (testnet or mainnet)
#
# Usage: ./scripts/deploy.sh [--network testnet|mainnet]
#        NETWORK=mainnet ./scripts/deploy.sh
#
# Required environment variables:
#   NETWORK              Target network: testnet (default) or mainnet.
#   MAINNET_RPC_URL      RPC endpoint; required when NETWORK=mainnet.
#   STELLAR_IDENTITY     Stellar CLI identity to deploy from (default: deployer).
#
# Optional environment variables (idempotency):
#   ALERT_CONTRACT_ID    Existing Alert Registry contract ID; if set and the
#                        contract is already deployed, upload/deploy is skipped.
#   WATCHER_CONTRACT_ID  Existing Watcher Registry contract ID; same behavior.
#
# Rollback: if a deploy partially fails, the contracts already deployed remain
# on-chain. Re-run this script with ALERT_CONTRACT_ID / WATCHER_CONTRACT_ID set
# to the successfully deployed IDs to skip re-uploading them and resume from the
# failed step. To fully roll back, redeploy fresh contracts and update
# DEPLOYMENTS.md with the new IDs.
set -euo pipefail

# --- Network selection ---
NETWORK="${NETWORK:-testnet}"

while [[ $# -gt 0 ]]; do
  case "$1" in
    --network)
      NETWORK="$2"; shift 2 ;;
    *)
      echo "Unknown argument: $1"; exit 1 ;;
  esac
done

case "$NETWORK" in
  testnet)
    RPC_URL="https://soroban-testnet.stellar.org"
    NETWORK_PASSPHRASE="Test SDF Network ; September 2015"
    ;;
  mainnet)
    RPC_URL="${MAINNET_RPC_URL:?MAINNET_RPC_URL must be set for mainnet deployments}"
    NETWORK_PASSPHRASE="Public Global Stellar Network ; September 2015"
    ;;
  *)
    echo "Unsupported network: $NETWORK (use testnet or mainnet)"; exit 1 ;;
  esac

IDENTITY="${STELLAR_IDENTITY:-deployer}"

echo "==> Network: $NETWORK"
echo "==> Checking Stellar CLI..."
stellar --version

if [[ "$NETWORK" == "testnet" ]]; then
  echo "==> Funding account on testnet..."
  stellar keys generate --overwrite "$IDENTITY" --network "$NETWORK" 2>/dev/null || true
  stellar keys fund "$IDENTITY" --network "$NETWORK"
fi

# Returns 0 if the given contract ID is already deployed and reachable.
contract_exists() {
  local id="$1"
  [[ -n "$id" ]] || return 1
  stellar contract info \
    --id "$id" \
    --network "$NETWORK" \
    --rpc-url "$RPC_URL" \
    --network-passphrase "$NETWORK_PASSPHRASE" >/dev/null 2>&1
}

echo "==> Building contracts..."
cargo build --release --target wasm32-unknown-unknown

ALERT_WASM="target/wasm32-unknown-unknown/release/alert_registry.wasm"
WATCHER_WASM="target/wasm32-unknown-unknown/release/watcher_registry.wasm"

if contract_exists "${ALERT_CONTRACT_ID:-}"; then
  ALERT_ID="$ALERT_CONTRACT_ID"
  echo "==> Alert Registry already deployed ($ALERT_ID), skipping upload"
else
  echo "==> Deploying Alert Registry..."
  ALERT_ID=$(stellar contract deploy \
    --wasm "$ALERT_WASM" \
    --source "$IDENTITY" \
    --network "$NETWORK" \
    --rpc-url "$RPC_URL" \
    --network-passphrase "$NETWORK_PASSPHRASE")
  echo "Alert Registry deployed: $ALERT_ID"
fi

if contract_exists "${WATCHER_CONTRACT_ID:-}"; then
  WATCHER_ID="$WATCHER_CONTRACT_ID"
  echo "==> Watcher Registry already deployed ($WATCHER_ID), skipping upload"
else
  echo "==> Deploying Watcher Registry..."
  WATCHER_ID=$(stellar contract deploy \
    --wasm "$WATCHER_WASM" \
    --source "$IDENTITY" \
    --network "$NETWORK" \
    --rpc-url "$RPC_URL" \
    --network-passphrase "$NETWORK_PASSPHRASE")
  echo "Watcher Registry deployed: $WATCHER_ID"
fi

ADMIN_ADDRESS=$(stellar keys address "$IDENTITY")

echo "==> Initializing Watcher Registry..."
stellar contract invoke \
  --id "$WATCHER_ID" \
  --source "$IDENTITY" \
  --network "$NETWORK" \
  --rpc-url "$RPC_URL" \
  --network-passphrase "$NETWORK_PASSPHRASE" \
  -- initialize \
  --admin "$ADMIN_ADDRESS"

echo "==> Verifying Watcher Registry is callable..."
stellar contract invoke \
  --id "$WATCHER_ID" \
  --source "$IDENTITY" \
  --network "$NETWORK" \
  --rpc-url "$RPC_URL" \
  --network-passphrase "$NETWORK_PASSPHRASE" \
  -- get_admin

echo ""
echo "==> Deployment complete ($NETWORK). Update DEPLOYMENTS.md with:"
echo "    Alert Registry:   $ALERT_ID"
echo "    Watcher Registry: $WATCHER_ID"
