# Alpha Keys: live TEE attestation runbook

Goal: a real OutLayer (Intel TDX) callback row on `alpha-keys.testnet`, so https://alpha-keys.vercel.app shows **Attested**. Run everything below on your own machine, from the repo root. Nothing here prints your Hyperliquid address or a private key.

Branch: `tee-live` (commit 9775046). Do not merge to main until step 9 passes.

Expected worker hash (reproducible build): `cb9c3130c6858cf4b460950281cff3468eabc0d79f1652fb05061c7fe41978dd`

Time: about 30 to 45 minutes, most of it installs and waiting for the callback.

## What you sign

On `alpha-keys.testnet` (near-cli, your existing ml-dsa-65 keychain), 4 transactions:

| # | Transaction | What it does | Cost |
|---|---|---|---|
| 1 | `deploy` (code only, no init) | Ships the gas fix and the 10-minute window check. State is kept. | gas only, under 0.01 NEAR |
| 2 | `set_attestor_code_hash` | Pins the worker sha256. From now on only that exact wasm counts as Attested. The old fixture row turns back to Relayed on the site. | gas only |
| 3 | `set_attestor_source` | Stores the FastFS wasm URL, the secrets profile `pnl` and the secret owner account. | gas only |
| 4 | `request_pnl_update` | Asks OutLayer to run the worker in TDX for the last 7 days. The result comes back in the same transaction through yield/resume and is stored by `on_pnl_result`. | 0.5 NEAR attached, about 0.007 NEAR used, the rest is refunded |

On your OutLayer login account (ed25519, signed by outlayer-cli): FastFS upload transactions (they fail on purpose, the indexer keeps the bytes), `outlayer deploy` (0.1 NEAR storage, excess refunded), `outlayer secrets set` (small storage deposit).

Testnet NEAR needed: `alpha-keys.testnet` already has about 9.99 NEAR and the script refuses below 2 NEAR. The OutLayer account needs about 1 NEAR.

## 0. Prerequisites (once)

```bash
rustup --version                      # Rust via rustup
near --version                        # near-cli-rs 0.31 or newer (ml-dsa-65 support)
curl -fsSL https://raw.githubusercontent.com/out-layer/outlayer-cli/main/install.sh | sh
outlayer about
python3 --version
```

Your near-cli legacy keychain for `alpha-keys.testnet` (from the first deploy) must still be in `~/.near-credentials/testnet/`.

## 1. Get the branch and check the build

```bash
git fetch origin && git checkout tee-live && git pull --ff-only
make check                                  # contract sandbox tests did not run on my box; please run them here
./scripts/build-worker.sh                   # installs toolchain 1.99.1 if missing
```

Expect the stderr line `wasm sha256 cb9c3130c6858cf4b460950281cff3468eabc0d79f1652fb05061c7fe41978dd`. If yours differs, stop and send me the line. It still works, but the "reproduce the hash" claim needs the two to match.

## 2. Ship the contract fix (tx 1)

The deployed `request_pnl_update` cannot run: it reserves almost all prepaid gas for OutLayer and the runtime refuses it (GasExceeded, reproduced in a unit test).

```bash
UPGRADE_CHECK_ONLY=1 ./scripts/upgrade-contract.sh     # build only
./scripts/upgrade-contract.sh                          # signs tx 1
```

Expect `get_config` to print `["alpha-keys.testnet","outlayer.testnet","relayed-demo"]` afterwards.

## 3. Log in to OutLayer with an ed25519 testnet account

outlayer-cli only signs with ed25519, and `alpha-keys.testnet` has only an ml-dsa-65 key. Use a separate testnet account. It owns the sealed secret; only transactions signed by `alpha-keys.testnet` can read it.

If you do not have an ed25519 testnet account, create one from the faucet (pick any free name):

```bash
near account create-account sponsor-by-faucet-service YOURNAME-outlayer.testnet \
  autogenerate-new-keypair save-to-legacy-keychain network-config testnet create
```

Then log in. It prompts for the account id and the `ed25519:...` private key. The key is in `~/.near-credentials/testnet/YOURNAME-outlayer.testnet.json` (field `private_key`). Paste it only into this prompt.

```bash
outlayer login testnet
outlayer whoami          # Account: YOURNAME-outlayer.testnet, Network: testnet
```

## 4. Put the address where only the script reads it

Env var names (values never go in chat, git or a screenshot):

- `HL_ADDRESS`: your Hyperliquid main account address. Put it in the repo's `.env` (gitignored), one line `HL_ADDRESS=0x...`.
- `NEAR_ACCOUNT_ID`: optional, defaults to `alpha-keys.testnet`.
- `OUTLAYER_ACCOUNT_ID`: optional, defaults to the `outlayer whoami` account.
- `OUTLAYER_DEPOSIT`: optional, defaults to `0.5 NEAR`.

```bash
git status --short .env        # must print nothing (ignored)
git check-ignore .env          # must print .env
```

## 5. Dry run

```bash
OUTLAYER_PUBLISH_CHECK_ONLY=1 ./scripts/outlayer-publish.sh
```

Expect `wasm sha256 cb9c31...` and `check only: wasm built, nothing published`.

## 6. Publish and pin, without the run (tx 2 and 3)

```bash
OUTLAYER_REQUEST=0 ./scripts/outlayer-publish.sh
```

Expected output, in order:

- `OutLayer login YOURNAME-outlayer.testnet. Secrets are stored under that account.`
- `Uploading to FastFS...` then `FastFS URL: https://test.fastfs.io/YOURNAME-outlayer.testnet/outlayer.testnet/cb9c31....wasm`
- an `outlayer deploy` result (if it fails the script says so and carries on; the contract uses the URL directly)
- `Sealing HL_ADDRESS for this wasm hash. The value is not printed.`
- two near-cli transactions, then `Pinned code hash cb9c31...`

The secret value is passed to outlayer-cli as a process argument for a moment, on your machine only. Run it in your own terminal, not a shared one.

Check the pin:

```bash
near contract call-function as-read-only alpha-keys.testnet get_config json-args '{}' network-config testnet now
```

Expect the third value to be `cb9c3130...78dd`.

## 7. Run the attested update (tx 4)

Step 6 already pinned everything, so send only tx 4:

```bash
NOW_MS=$(python3 -c 'import time; print(int(time.time()*1000))')
near contract call-function as-transaction alpha-keys.testnet request_pnl_update \
  json-args "{\"trader_id\":\"alpha-1\",\"window\":\"7d\",\"now_ms\":$NOW_MS}" \
  prepaid-gas '300.0 Tgas' attached-deposit '0.5 NEAR' \
  sign-as alpha-keys.testnet network-config testnet sign-with-legacy-keychain send
```

The worker takes about 5 seconds inside TDX; the yield/resume callback lands in the same transaction within a minute or two. near-cli prints the transaction id. Save it for the README and the form.

(`OUTLAYER_REQUEST=1 ./scripts/outlayer-publish.sh` does steps 6 and 7 in one go, but repeats the upload, the secret and txs 2 and 3. Use it only if you skipped step 6.)

## 8. Verify on chain (anyone can run this)

```bash
curl -s https://rpc.testnet.near.org -H 'content-type: application/json' -d '{"jsonrpc":"2.0","id":1,"method":"query","params":{"request_type":"call_function","finality":"final","account_id":"alpha-keys.testnet","method_name":"get_pnl_history","args_base64":"'$(printf '{"trader_id":"alpha-1","from":0,"limit":20}' | base64 | tr -d '\n')'"}}' \
 | python3 -c 'import sys,json; r=json.load(sys.stdin)["result"]["result"]; [print(x["source"], x["code_hash"][:12], x["pnl_usd_e6"], x["max_dd_bps"], x["n_trades"]) for x in json.loads(bytes(r))]'
```

Pass: the last row has source `{'OutLayerCallback': {'request_id': None}}` and code hash `cb9c3130c685`, the same as `get_config`.

Explorer: https://testnet.nearblocks.io/address/alpha-keys.testnet (tx 4 shows `request_pnl_update`, a receipt to `outlayer.testnet`, and `on_pnl_result`).

## 9. Verify on the site

Open https://alpha-keys.vercel.app. Main already reads the chain, so without any merge you see Alpha One with the **Attested** badge, code hash `cb9c31...`, and the real 7d numbers. The text under the beats changes to "The Attested row is an OutLayer callback...".

The Verify page shows the attested record only after `tee-live` is merged to main (new panel). Merge with:

```bash
git checkout main && git merge --ff-only tee-live && git push origin main
```

Vercel redeploys in about a minute.

## If it goes wrong

- **Tx 4 fails before OutLayer** (`attach at least 200 Tgas`, `now_ms must be within 10 minutes`, `attestor wasm url not set`): rerun with 300 Tgas, a fresh `NOW_MS`, or redo step 6.
- **Callback stored nothing.** Look in tx 4's logs for `EVENT_JSON ... pnl_rejected` and its `reason`:
  - `execution failed` or `empty output`: the worker errored. Most likely the secret did not open (`missing HL_ADDRESS`). Check `outlayer secrets list` shows profile `pnl` for hash `cb9c31...` with access `whitelist:alpha-keys.testnet`, and that step 6 ran while `outlayer whoami` showed the same account (it stores that account as `secrets_account_id`; rerun step 6 if unsure). The OutLayer dashboard at https://app.outlayer.ai shows the execution error.
  - `code hash`: the pin changed between request and callback. Rerun tx 4.
  - `malformed output`: send me the dashboard output (it holds only metrics).
  - Nothing is stored on a rejection and the deposit is refunded except the base fee, so you can retry.
- **Numbers look wrong.** Rows are append-only. To take the badge off at once:

  ```bash
  near contract call-function as-transaction alpha-keys.testnet set_attestor_code_hash \
    json-args '{"code_hash":"relayed-demo"}' prepaid-gas '30.0 Tgas' attached-deposit '0 NEAR' \
    sign-as alpha-keys.testnet network-config testnet sign-with-legacy-keychain send
  ```

  The site falls back to the relayed fixture right away (the callback row no longer matches the pin). Fix, rebuild, rerun from step 6; a new hash makes the old row stale.
- **Contract upgrade broke something**: `git checkout 8a04448 && ./scripts/upgrade-contract.sh` restores the old code (state is the same layout).

## Known limits (say them if asked)

- PnL is realized PnL of fills in the window minus fees, plus current unrealized PnL of open positions (which may predate the window). Deposits and withdrawals in the window are not separated out.
- Equity (perps account value plus spot USDC) is stored on chain in the record. It is not shown on the site.
- `get_pnl_history` is read with limit 200; past 200 rows the site would miss newer ones.
