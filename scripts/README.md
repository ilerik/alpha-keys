# scripts/

- `quote-dry.sh` (S-4): a 1Click `dry: true` quote from NEAR USDC to Hyperliquid USDC. Asset ids come from the 1Click docs. Verify them with `GET /v0/tokens`.
- Planned: `deploy-testnet.sh` (C-6), `outlayer-publish.sh` (A-5), `attest-https.ts` (A-7), `fresh-clone-check.sh` (H-4).

Scripts read secrets from env and must never echo them.
