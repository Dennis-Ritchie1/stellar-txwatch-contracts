#!/usr/bin/env bash
#
# scripts/upgrade.sh — Upgrade a deployed contract to a new WASM build.
#
# Safety checks (issue #303):
#   1. --network is REQUIRED; the script refuses to run without it.
#   2. Mainnet upgrades require an interactive "Type YES" confirmation.
#   3. The current on-chain admin key is verified against DEPLOYMENTS.md.
#   4. Old vs. new WASM hashes are printed before the upgrade.
#   5. scripts/verify.sh is run after the upgrade to confirm functionality.
#
# Usage:
#   bash scripts/upgrade.sh --network testnet --contract <CONTRACT_ID> --wasm <PATH>
#   bash scripts/upgrade.sh --network mainnet --contract <CONTRACT_ID> --wasm <PATH>
#
# Examples:
#   # Upgrade the testnet alert registry to a freshly built WASM
#   bash scripts/upgrade.sh --network testnet \
#     --contract CXXXX... --wasm target/wasm32-unknown-unknown/release/alert_registry.wasm
#
#   # Upgrade mainnet (prompts for confirmation)
#   bash scripts/upgrade.sh --network mainnet \
#     --contract CXXXX... --wasm target/wasm32-unknown-unknown/release/alert_registry.wasm
#
set -euo pipefail

NETWORK=""
CONTRACT=""
WASM=""

usage() {
  echo "Usage: $0 --network <testnet|mainnet> --contract <CONTRACT_ID> --wasm <PATH>" >&2
  exit 1
}

while [[ $# -gt 0 ]]; do
  case "$1" in
    --network) NETWORK="${2:-}"; shift 2 ;;
    --contract) CONTRACT="${2:-}"; shift 2 ;;
    --wasm) WASM="${2:-}"; shift 2 ;;
    -h|--help) usage ;;
    *) echo "Unknown argument: $1" >&2; usage ;;
  esac
done

# 1. Require explicit --network flag.
if [[ -z "$NETWORK" ]]; then
  echo "ERROR: --network is required (testnet|mainnet)." >&2
  usage
fi
if [[ "$NETWORK" != "testnet" && "$NETWORK" != "mainnet" ]]; then
  echo "ERROR: --network must be 'testnet' or 'mainnet' (got '$NETWORK')." >&2
  exit 1
fi
if [[ -z "$CONTRACT" || -z "$WASM" ]]; then
  echo "ERROR: --contract and --wasm are required." >&2
  usage
fi
if [[ ! -f "$WASM" ]]; then
  echo "ERROR: WASM file not found: $WASM" >&2
  exit 1
fi

case "$NETWORK" in
  testnet)
    RPC_URL="https://soroban-testnet.stellar.org"
    NETWORK_PASSPHRASE="Test SDF Network ; September 2015"
    ;;
  mainnet)
    RPC_URL="${MAINNET_RPC_URL:-https://mainnet.stellar.validationcloud.io/v1/${STELLAR_API_KEY:-}}"
    NETWORK_PASSPHRASE="Public Global Stellar Network ; September 2015"
    ;;
esac

# 2. Interactive confirmation for mainnet.
if [[ "$NETWORK" == "mainnet" ]]; then
  echo "WARNING: You are about to upgrade a MAINNET contract: $CONTRACT"
  read -r -p "Type YES to upgrade mainnet contracts: " CONFIRM
  if [[ "$CONFIRM" != "YES" ]]; then
    echo "Aborted: mainnet confirmation not given." >&2
    exit 1
  fi
fi

# 3. Verify the current admin key matches DEPLOYMENTS.md.
DEPLOYMENTS_FILE="$(dirname "$0")/../DEPLOYMENTS.md"
if [[ ! -f "$DEPLOYMENTS_FILE" ]]; then
  echo "ERROR: DEPLOYMENTS.md not found at $DEPLOYMENTS_FILE" >&2
  exit 1
fi

EXPECTED_ADMIN="$(awk -F'|' -v net="$NETWORK" '
  tolower($0) ~ tolower(net) && $0 ~ /G[A-Z0-9]{55}/ {
    gsub(/[ `]/, "", $3); print $3; exit
  }' "$DEPLOYMENTS_FILE")"

CURRENT_ADMIN="$(stellar contract invoke \
  --id "$CONTRACT" \
  --rpc-url "$RPC_URL" \
  --network-passphrase "$NETWORK_PASSPHRASE" \
  -- get_admin 2>/dev/null || true)"

if [[ -z "$CURRENT_ADMIN" ]]; then
  echo "ERROR: could not read current admin key from $CONTRACT." >&2
  exit 1
fi
if [[ -n "$EXPECTED_ADMIN" && "$CURRENT_ADMIN" != "$EXPECTED_ADMIN" ]]; then
  echo "ERROR: admin key mismatch for $NETWORK." >&2
  echo "  on-chain:    $CURRENT_ADMIN" >&2
  echo "  DEPLOYMENTS: $EXPECTED_ADMIN" >&2
  exit 1
fi

# 4. Compare old and new WASM hashes before upgrading.
OLD_HASH="$(stellar contract fetch \
  --id "$CONTRACT" \
  --rpc-url "$RPC_URL" \
  --network-passphrase "$NETWORK_PASSPHRASE" 2>/dev/null | sha256sum | awk '{print $1}' || true)"
NEW_HASH="$(sha256sum "$WASM" | awk '{print $1}')"

echo "WASM hash comparison:"
echo "  old (on-chain): ${OLD_HASH:-<unavailable>}"
echo "  new (local):    $NEW_HASH"
if [[ -n "$OLD_HASH" && "$OLD_HASH" == "$NEW_HASH" ]]; then
  echo "No change: new WASM matches the deployed WASM. Nothing to upgrade."
  exit 0
fi

# Perform the upgrade.
stellar contract upload \
  --wasm "$WASM" \
  --rpc-url "$RPC_URL" \
  --network-passphrase "$NETWORK_PASSPHRASE"

stellar contract invoke \
  --id "$CONTRACT" \
  --rpc-url "$RPC_URL" \
  --network-passphrase "$NETWORK_PASSPHRASE" \
  -- upgrade --new_wasm_hash "$NEW_HASH"

echo "Upgrade submitted for $CONTRACT on $NETWORK."

# 5. Post-upgrade verification.
VERIFY_SCRIPT="$(dirname "$0")/verify.sh"
if [[ -x "$VERIFY_SCRIPT" || -f "$VERIFY_SCRIPT" ]]; then
  echo "Running post-upgrade verification..."
  bash "$VERIFY_SCRIPT" --network "$NETWORK" --contract "$CONTRACT"
else
  echo "WARNING: scripts/verify.sh not found; skipping post-upgrade verification." >&2
fi
