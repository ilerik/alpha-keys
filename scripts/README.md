# scripts/

- `quote-dry.sh` (S-4): a 1Click `dry: true` quote from NEAR USDC to Hyperliquid USDC. Asset ids come from the 1Click docs. Verify them with `GET /v0/tokens`.
- `deploy-testnet.sh`: author-only testnet deploy, `register_trader`, and one relayed `submit_pnl`. It reads `NEAR_PRIVATE_KEY` from the environment (`ed25519:` or `ml-dsa-65:`) and does not print it. Signing uses the near-cli legacy keychain, which already stores the public key. Judges do not run it.
- Planned: `outlayer-publish.sh` (A-5), `attest-https.ts` (A-7), `fresh-clone-check.sh` (H-4).

Scripts read secrets from env and must never echo them.
