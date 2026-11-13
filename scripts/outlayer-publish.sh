#!/usr/bin/env bash
# Publish the PnL worker and pin it on alpha-keys.testnet.
# Reads HL_ADDRESS from the environment or from .env. Never prints that value.
# Judges do not run this. OUTLAYER_PUBLISH_CHECK_ONLY=1 builds the wasm and stops.
# OUTLAYER_REQUEST=0 pins the worker and skips request_pnl_update.
set -euo pipefail

if [[ $# -ne 0 ]]; then
  echo "This script takes no arguments. Put HL_ADDRESS in the environment or .env, then run it." >&2
  exit 2
fi

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
ACCOUNT="${NEAR_ACCOUNT_ID:-alpha-keys.testnet}"
PROFILE="${OUTLAYER_SECRETS_PROFILE:-pnl}"
REQUEST="${OUTLAYER_REQUEST:-1}"
DEPOSIT="${OUTLAYER_DEPOSIT:-0.5 NEAR}"
export OUTLAYER_NETWORK="${OUTLAYER_NETWORK:-testnet}"
export ROOT ACCOUNT PROFILE

echo "Building the PnL worker (scripts/build-worker.sh, pinned toolchain, remapped paths)."
WASM="$("$ROOT/scripts/build-worker.sh" | tail -n 1)"
if [[ ! -f "$WASM" ]]; then
  echo "Wasm was not found after the build." >&2
  exit 1
fi
HASH="$(sha256sum "$WASM" | awk '{print $1}')"
echo "wasm sha256 ${HASH}"

if [[ "${OUTLAYER_PUBLISH_CHECK_ONLY:-}" == 1 ]]; then
  echo "check only: wasm built, nothing published"
  exit 0
fi

if ! python3 - <<'PY'
import os, re, sys
from pathlib import Path

def address() -> str:
    value = os.environ.get("HL_ADDRESS", "").strip().strip('"').strip("'")
    if value:
        return value
    env_path = Path(os.environ["ROOT"]) / ".env"
    if not env_path.is_file():
        return ""
    for line in env_path.read_text().splitlines():
        if line.startswith("HL_ADDRESS="):
            return line.split("=", 1)[1].strip().strip('"').strip("'")
    return ""

value = address()
if not re.fullmatch(r"0x[0-9a-fA-F]{40}", value):
    print("HL_ADDRESS must be a 0x address of 40 hex characters. The value was not printed.", file=sys.stderr)
    sys.exit(2)
PY
then
  exit 2
fi

if ! command -v near >/dev/null 2>&1; then
  echo "Install near-cli-rs and retry. https://github.com/near/near-cli-rs" >&2
  exit 2
fi

if ! command -v outlayer >/dev/null 2>&1; then
  echo "Install outlayer-cli and log in on testnet, then retry. The address was not printed." >&2
  exit 2
fi

# outlayer-cli signs with an ed25519 key only, so its login can be a different
# account from the contract owner (alpha-keys.testnet holds an ml-dsa-65 key).
# The secret row belongs to the outlayer login, so secrets_account_id must be
# that account. The run is admitted by the SIGNER of request_pnl_update.
SECRETS_OWNER="${OUTLAYER_ACCOUNT_ID:-$(outlayer whoami 2>/dev/null | awk '/^Account:/ {print $2}')}"
if [[ -z "$SECRETS_OWNER" ]]; then
  echo "outlayer is not logged in. Run: outlayer login testnet" >&2
  exit 2
fi
if ! outlayer whoami 2>/dev/null | grep -q '^Network: *testnet'; then
  echo "outlayer is not on testnet. Run: outlayer login testnet (or export OUTLAYER_NETWORK=testnet)" >&2
  exit 2
fi
echo "OutLayer login ${SECRETS_OWNER}. Secrets are stored under that account."
export SECRETS_OWNER

echo "Uploading the wasm. The indexer can take a minute before the URL serves it."
UPLOAD="$(outlayer upload "$WASM" 2>&1)"
printf '%s\n' "$UPLOAD"
URL="$(printf '%s\n' "$UPLOAD" | python3 -c 'import re,sys; text=sys.stdin.read(); match=re.search(r"https://[^[:space:]]+\.wasm", text); print(match.group(0) if match else "")')"
if [[ -z "$URL" ]]; then
  echo "Upload did not print a wasm URL. Nothing was pinned." >&2
  exit 1
fi
echo "wasm url ${URL}"

for _ in 1 2 3 4 5 6 7 8 9 10 11 12; do
  if curl -fsS --max-time 20 -o /dev/null "$URL"; then
    break
  fi
  sleep 15
done
if ! curl -fsS --max-time 20 -o /dev/null "$URL"; then
  echo "The wasm URL is not reachable yet. Nothing was pinned." >&2
  exit 1
fi

if ! OUTLAYER_NETWORK="$OUTLAYER_NETWORK" PROJECT_NAME="${OUTLAYER_PROJECT:-alpha-keys-pnl}" outlayer deploy "${OUTLAYER_PROJECT:-alpha-keys-pnl}" "$URL" --hash "$HASH"; then
  echo "Project deploy did not succeed. The contract still uses the wasm URL directly."
fi

echo "Sealing HL_ADDRESS for this wasm hash. The value is not printed."
HASH="$HASH" URL="$URL" python3 - <<'PY'
import json, os, re, subprocess, sys
from pathlib import Path

def address() -> str:
    value = os.environ.get("HL_ADDRESS", "").strip().strip('"').strip("'")
    if value:
        return value
    env_path = Path(os.environ["ROOT"]) / ".env"
    for line in env_path.read_text().splitlines():
        if line.startswith("HL_ADDRESS="):
            return line.split("=", 1)[1].strip().strip('"').strip("'")
    return ""

secret = address()
payload = json.dumps({"HL_ADDRESS": secret})
command = [
    "outlayer",
    "secrets",
    "set",
    payload,
    "--wasm-hash",
    os.environ["HASH"],
    "--profile",
    os.environ["PROFILE"],
    "--access",
    "whitelist:" + os.environ["ACCOUNT"],
]
completed = subprocess.run(command, text=True, capture_output=True)
redacted = secret
def scrub(text: str) -> str:
    return text.replace(redacted, "[redacted]")
sys.stdout.write(scrub(completed.stdout))
sys.stderr.write(scrub(completed.stderr))
if completed.returncode != 0:
    sys.exit(completed.returncode)
if re.search(r"0x[0-9a-fA-F]{40}", scrub(completed.stdout + completed.stderr)):
    print("A command printed an address-shaped value after redaction. Stop.", file=sys.stderr)
    sys.exit(1)
PY

call() {
  local method="$1"
  local args="$2"
  local deposit="$3"
  local gas="$4"
  near contract call-function as-transaction "$ACCOUNT" "$method" json-args "$args" \
    prepaid-gas "$gas" attached-deposit "$deposit" \
    sign-as "$ACCOUNT" network-config testnet sign-with-legacy-keychain send
}

call set_attestor_code_hash "$(printf '{"code_hash":"%s"}' "$HASH")" '0 NEAR' '30.0 Tgas'
call set_attestor_source "$(printf '{"wasm_url":"%s","secrets_profile":"%s","secrets_account_id":"%s"}' "$URL" "$PROFILE" "$SECRETS_OWNER")" '0 NEAR' '30.0 Tgas'
echo "Pinned code hash ${HASH}"
echo "Pinned wasm url ${URL}"
echo "Secrets profile ${PROFILE} owned by ${SECRETS_OWNER}, readable by ${ACCOUNT}"

if [[ "$REQUEST" != 1 ]]; then
  echo "OUTLAYER_REQUEST is not 1, so request_pnl_update was not called."
  exit 0
fi

python3 - "$ACCOUNT" <<'PY'
import json, sys, urllib.request
account = sys.argv[1]
body = json.dumps({
    "jsonrpc": "2.0",
    "id": "balance",
    "method": "query",
    "params": {"request_type": "view_account", "finality": "final", "account_id": account},
}).encode()
request = urllib.request.Request(
    "https://rpc.testnet.fastnear.com",
    data=body,
    headers={"content-type": "application/json"},
)
with urllib.request.urlopen(request, timeout=30) as response:
    payload = json.load(response)
amount = int(payload["result"]["amount"])
# Keep a reserve so the account can still pay storage after the refund.
if amount < 2_000_000_000_000_000_000_000_000:
    print("Account balance is below 2 NEAR. request_pnl_update was not sent.", file=sys.stderr)
    sys.exit(2)
print("balance check passed")
PY

NOW_MS="$(python3 -c 'import time; print(int(time.time() * 1000))')"
echo "Calling request_pnl_update for alpha-1 window 7d now_ms ${NOW_MS}"
call request_pnl_update "$(printf '{"trader_id":"alpha-1","window":"7d","now_ms":%s}' "$NOW_MS")" "$DEPOSIT" '300.0 Tgas'

echo "Reading get_pnl_history. A callback can land after this transaction."
near contract call-function as-read-only "$ACCOUNT" get_pnl_history \
  json-args '{"trader_id":"alpha-1","from":0,"limit":20}' \
  network-config testnet now | python3 -c '
import re, sys
text = sys.stdin.read()
if re.search(r"0x[0-9a-fA-F]{40}", text):
    print("Refusing to print history because it contains an address-shaped hex.", file=sys.stderr)
    sys.exit(1)
sys.stdout.write(text)
'
echo "Explorer: https://testnet.nearblocks.io/address/${ACCOUNT}"
