#!/usr/bin/env bash
# S-4: 1Click DRY quote (no funds move). NEAR USDC -> Hyperliquid USDC (perps balance).
# Refuses live mode unless ONECLICK_LIVE=1. Never echoes ONECLICK_JWT.
set -euo pipefail
BASE="${ONECLICK_BASE_URL:-https://1click.chaindefuser.com/v0}"
DRY=true; [[ "${ONECLICK_LIVE:-0}" == "1" ]] && DRY=false
RECIPIENT="${HL_DEMO_RECIPIENT:-0x0000000000000000000000000000000000000001}"  # placeholder, dry only
REFUND="${NEAR_ACCOUNT_ID:-alpha-keys-dev.testnet}"
DEADLINE="$(date -u -d '+30 min' +%Y-%m-%dT%H:%M:%S.000Z)"
AUTH=(); [[ -n "${ONECLICK_JWT:-}" ]] && AUTH=(-H "X-API-Key: ${ONECLICK_JWT}")
mkdir -p fixtures/quotes
curl -sS -X POST "$BASE/quote" -H 'Content-Type: application/json' "${AUTH[@]}" -d @- <<JSON | tee fixtures/quotes/near-usdc-to-hl.json
{
  "dry": $DRY,
  "swapType": "EXACT_INPUT",
  "slippageTolerance": 100,
  "originAsset": "nep141:17208628f84f5d6ad33f0da3bbbeb27ffcb398eac501a31bd6ad2011e36133a1",
  "destinationAsset": "1cs_v1:hypercore:hip1:0x6d1e7cde53ba9467b783cb7c530ce054",
  "amount": "5000000",
  "depositType": "ORIGIN_CHAIN",
  "recipient": "$RECIPIENT",
  "recipientType": "DESTINATION_CHAIN",
  "refundTo": "$REFUND",
  "refundType": "ORIGIN_CHAIN",
  "deadline": "$DEADLINE"
}
JSON
echo
