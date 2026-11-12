#!/usr/bin/env bash
# Deploy alpha-keys to NEAR testnet and store the demo PnL as a relayed record.
# Reads NEAR_ACCOUNT_ID and NEAR_PRIVATE_KEY from the environment. Never prints the key.
# near-cli-rs 0.31 may create an ed25519 key or an ml-dsa-65 key. Both are accepted.
# The keychain file is not rewritten when it already holds this secret and its public key.
# Judges do not run this. There is no contract id until this script succeeds.
set -euo pipefail

if [[ $# -ne 0 ]]; then
  echo "This script takes no arguments. Export NEAR_ACCOUNT_ID and NEAR_PRIVATE_KEY, then run it." >&2
  exit 2
fi

ACCOUNT="${NEAR_ACCOUNT_ID:-}"
KEY="${NEAR_PRIVATE_KEY:-}"
if [[ -z "$ACCOUNT" || -z "$KEY" ]]; then
  echo "Set NEAR_ACCOUNT_ID and NEAR_PRIVATE_KEY in the environment. Do not pass them as arguments." >&2
  exit 2
fi
case "$KEY" in
  ed25519:*|ml-dsa-65:*) ;;
  *)
    echo "NEAR_PRIVATE_KEY must start with ed25519: or ml-dsa-65:. The value was not printed." >&2
    exit 2
    ;;
esac
if ! command -v near >/dev/null 2>&1; then
  echo "Install near-cli-rs and retry. https://github.com/near/near-cli-rs" >&2
  exit 2
fi

CRED_DIR="${HOME}/.near-credentials/testnet"
CRED_FILE="$CRED_DIR/${ACCOUNT}.json"
KEY_DIR="$CRED_DIR/${ACCOUNT}"
if [[ ! -f "$CRED_FILE" && ! -d "$KEY_DIR" ]]; then
  echo "No keychain file for ${ACCOUNT}. Import the account with near-cli so the file includes the public key, then rerun. The key was not printed." >&2
  exit 2
fi
# near-cli-rs stores {public_key, private_key}. Replacing that with {account_id, private_key}
# drops the public key, and an ml-dsa-65 public key is not the secret with a different prefix.
if ! NEAR_ACCOUNT_ID="$ACCOUNT" NEAR_PRIVATE_KEY="$KEY" CRED_FILE="$CRED_FILE" KEY_DIR="$KEY_DIR" python3 - <<'PY'
import json, os, sys
from pathlib import Path
secret = os.environ["NEAR_PRIVATE_KEY"]
paths = []
cred = Path(os.environ["CRED_FILE"])
if cred.is_file():
    paths.append(cred)
key_dir = Path(os.environ["KEY_DIR"])
if key_dir.is_dir():
    paths.extend(sorted(key_dir.glob("*.json")))
for path in paths:
    try:
        data = json.loads(path.read_text())
    except json.JSONDecodeError:
        continue
    if isinstance(data, dict) and data.get("private_key") == secret and data.get("public_key"):
        sys.exit(0)
sys.exit(3)
PY
then
  echo "The keychain for ${ACCOUNT} does not contain this NEAR_PRIVATE_KEY. The key was not printed." >&2
  exit 2
fi
unset KEY
unset NEAR_PRIVATE_KEY
echo "Using the existing keychain for ${ACCOUNT}. The key file was not rewritten."

# DEPLOY_TESTNET_CHECK_ONLY=1 stops after the key check. It does not deploy.
if [[ "${DEPLOY_TESTNET_CHECK_ONLY:-}" == 1 ]]; then
  echo "key type accepted for ${ACCOUNT}"
  exit 0
fi

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
TARGET="${CARGO_TARGET_DIR:-$ROOT/target}"
WASM="$TARGET/wasm32-unknown-unknown/release/alpha_keys.wasm"

echo "Building the contract wasm."
cargo build -p alpha-keys --release --target wasm32-unknown-unknown --manifest-path "$ROOT/Cargo.toml"
if [[ ! -f "$WASM" ]]; then
  echo "Wasm was not found after the build." >&2
  exit 1
fi

near contract deploy "$ACCOUNT" use-file "$WASM" without-init-call \
  network-config testnet sign-with-legacy-keychain send

call() {
  local method="$1"
  local args="$2"
  local deposit="$3"
  near contract call-function as-transaction "$ACCOUNT" "$method" json-args "$args" \
    prepaid-gas '100.0 Tgas' attached-deposit "$deposit" \
    sign-as "$ACCOUNT" network-config testnet sign-with-legacy-keychain send
}

call new "$(printf '{"owner_id":"%s","outlayer_id":"outlayer.testnet","attestor_code_hash":"relayed-demo"}' "$ACCOUNT")" '0 NEAR'
call register_trader '{"id":"alpha-1","display_name":"Alpha One","price_live":"1000000000000000000000000","price_delayed":"500000000000000000000000","royalty_bps":1000,"max_keys":100}' '2 NEAR'
call add_minter "$(printf '{"account_id":"%s"}' "$ACCOUNT")" '1 NEAR'
call submit_pnl '{"trader_id":"alpha-1","window":"7d","pnl_usd_e6":8000000,"max_dd_bps":10,"sharpe_e4":108098,"n_trades":2,"equity_usd_e6":1008000000,"code_hash":"relayed-demo","input_hash":"fixture","attestation_ref":"relayed-demo"}' '0 NEAR'
call mint_key "$(printf '{"receiver_id":"%s","trader_id":"alpha-1","tier":"Delayed60","purchase_ref":"demo","expires_at_ns":null}' "$ACCOUNT")" '1 NEAR'

echo
echo "CONTRACT_ID=${ACCOUNT}"
echo "Put that public account in .env.example as CONTRACT_ID and NEXT_PUBLIC_CONTRACT_ID."
echo "Put the submit_pnl transaction id in NEXT_PUBLIC_PNL_TX."
echo "Explorer: https://testnet.nearblocks.io/address/${ACCOUNT}"
echo "Do not commit the credentials file or the private key."
