#!/usr/bin/env bash
# Redeploy the alpha-keys contract code on testnet without re-initializing.
# State layout is unchanged, so traders, keys and the fixture PnL row stay.
# Signs with the near-cli legacy keychain. Reads and prints no secrets.
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
ACCOUNT="${NEAR_ACCOUNT_ID:-alpha-keys.testnet}"
TARGET="${CARGO_TARGET_DIR:-$ROOT/target}"
WASM="$TARGET/wasm32-unknown-unknown/release/alpha_keys.wasm"

if ! command -v near >/dev/null 2>&1; then
  echo "Install near-cli-rs and retry. https://github.com/near/near-cli-rs" >&2
  exit 2
fi

echo "Building the contract wasm."
cargo build --locked -p alpha-keys --release --target wasm32-unknown-unknown --manifest-path "$ROOT/Cargo.toml"
echo "contract sha256 $(sha256sum "$WASM" | awk '{print $1}')"

if [[ "${UPGRADE_CHECK_ONLY:-}" == 1 ]]; then
  echo "check only: built, nothing deployed"
  exit 0
fi

near contract deploy "$ACCOUNT" use-file "$WASM" without-init-call \
  network-config testnet sign-with-legacy-keychain send

echo "Reading get_config to confirm the state survived."
near contract call-function as-read-only "$ACCOUNT" get_config json-args '{}' network-config testnet now
