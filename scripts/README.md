# scripts/

- `quote-dry.sh` (S-4): a 1Click `dry: true` quote from NEAR USDC to Hyperliquid USDC. Asset ids come from the 1Click docs. Verify them with `GET /v0/tokens`.
- `deploy-testnet.sh`: author-only testnet deploy, `register_trader`, and one relayed `submit_pnl`. It reads `NEAR_PRIVATE_KEY` from the environment (`ed25519:` or `ml-dsa-65:`) and does not print it. Signing uses the near-cli legacy keychain, which already stores the public key. Judges do not run it.
- `outlayer-publish.sh` (A-5): build the worker, `outlayer upload`, seal `HL_ADDRESS` for that wasm hash, then `set_attestor_code_hash` and `set_attestor_source`. The secret row belongs to the `outlayer whoami` account (outlayer-cli signs with ed25519 only), and only runs signed by the contract account may read it. With `OUTLAYER_REQUEST=1` it calls `request_pnl_update`. It reads `HL_ADDRESS` from the environment or `.env` and does not print it. Judges do not run it.
- `build-worker.sh`: reproducible worker build. Pinned toolchain 1.99.1, `--locked`, and remapped paths, so the wasm sha256 is the same on any machine. `outlayer-publish.sh` calls it.
- `upgrade-contract.sh`: author-only. Rebuilds the contract and redeploys it without an init call. State is kept. Judges do not run it.
- Planned: `attest-https.ts` (A-7), `fresh-clone-check.sh` (H-4).

Scripts read secrets from env and must never echo them.
