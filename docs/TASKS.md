# Task backlog

How to use this file: hand **one task at a time** to a Cursor agent. Paste the task block and say "follow AGENTS.md". Each task is one small PR. A task is done only when its acceptance criteria (AC) pass **and** `make check` is green. Times are UTC (MSK).

Legend: 🔴 critical path · 🟡 important · ⚪ stretch. `deps:` lists tasks that must merge first.

---

## M0: Setup (D0, Thu Nov 5)

**S-1 🔴 Toolchain and Makefile**
- Make `make check` run: `cargo test --workspace` (contracts and attestor), `pnpm -r test`, and `uv run pytest` in `copier/`.
- AC: on a clean machine with the README prerequisites, `make check` exits 0 with the stub tests.

**S-2 🔴 Baseline import and disclosure**
- Copy the needed HL fleet modules into `copier/src/copier/vendor/` (and `attestor/pnl-core/src/vendor/` if needed), with a `# baseline: <origin repo>@<commit>` header. Commit that alone, then tag it `pre-hackathon`. List the files in the README "Pre-hackathon code disclosure" section.
- AC: `git tag` shows `pre-hackathon`; the README lists every vendored file; no secrets are in the vendored code.

**S-3 🟡 Accounts and env**
- Create the testnet dev account and fill your local `.env` (never committed). Optionally add a 1Click JWT.
- AC: `.env.example` lists every variable used anywhere (CI grep: each `process.env.X` / `os.environ["X"]` / `env::var("X")` appears in `.env.example`).

**S-4 🔴 1Click dry-quote script**
- `scripts/quote-dry.sh` (curl + jq) or `scripts/quote-dry.ts`: a NEAR USDC → HL USDC dry quote and a NEAR → NEAR-USDC dry quote. It writes `fixtures/quotes/*.json` and **refuses to run if `dry` isn't true** unless `ONECLICK_LIVE=1`.
- AC: the script prints `amountOut`; a test asserts `dry: true` in the request body by default.

**S-5 🟡 Trader account funding runbook** (deps: jurisdiction answer)
- `docs/RUNBOOK-funding.md`: the exact confidential quote request (`confidentiality: "basic"`) to HL USDC, the amounts (≥ 0.5 USDC plus the 0.2 fee), and the checks. Ilya runs it manually.
- AC: the runbook exists; nothing in the repo automates live funding.

---

## M1: Contract (D1, Fri Nov 6)

**C-1 🔴 Contract scaffold** (deps: S-1)
- `cargo near new`-style layout in `contracts/alpha-keys`, `near-sdk = "5.30"`, `near-contract-standards = "5.30"`, `new()` init, owner, `outlayer_id`, `attestor_code_hash`.
- AC: `cargo near build non-reproducible-wasm` succeeds; a unit test checks the init state.

**C-2 🔴 Trader registry**
- `register_trader`, `update_trader`, `get_trader`, `list_traders(from, limit)`. Storage deposit handling. No field accepts an EVM address.
- AC: unit tests cover registration, a duplicate id (panics), updates by the owner only, and pagination.

**C-3 🔴 Viewing-Key NFT**
- Use `near_contract_standards::non_fungible_token` (core, metadata, approval, enumeration) plus NEP-199 `nft_payout` with `royalty_bps` to the trader owner. `KeyMeta { trader_id, tier, expires_at_ns, purchase_ref }`.
- AC: sandbox test: mint, `nft_tokens_for_owner`, transfer, and `nft_payout` splits correctly.

**C-4 🔴 Buy paths**
- `buy_key` (NEAR attached ≥ price; excess refunded; proceeds to the trader owner), `ft_on_transfer` for USDC with `msg`, and minter-only `mint_key(receiver, trader_id, tier, purchase_ref)`. Enforce `max_keys`.
- AC: tests cover underpayment rejected, overpayment refunded, a non-minter `mint_key` panicking, `max_keys` enforced, and `purchase_ref` stored.

**C-5 🔴 Access view**
- `has_access(account_id, trader_id) -> Option<Tier>`. Returns the best tier and ignores expired keys.
- AC: unit tests cover none, delayed, live, expired, and transferred-away keys.

**C-6 🔴 PnL records plus testnet deploy**
- `PnlRecord`, `get_pnl_history`, owner-only `set_attestor_code_hash`, relayer-only `submit_pnl` (fallback). `scripts/deploy-testnet.sh`.
- AC: deployed on testnet; the README step 4 command works; the contract id is in `.env.example` as a public value.

---

## M2: PnL engine (D2, Sat Nov 7)

**A-1 🔴 `pnl-core` types and fixtures**
- `Fill`, `Position`, `AccountState`, `Metrics`. Serde matches HL `/info` `userFillsByTime` and `clearinghouseState`. Fixture `fixtures/fills/sample.json` (anonymized: address removed).
- AC: the fixture parses; a round-trip test passes.

**A-2 🔴 Metrics**
- Realized plus unrealized PnL, equity curve, max drawdown (bps), Sharpe on daily returns (e4 fixed point), and trade count. Deterministic, with no floats in the output (fixed-point ints).
- AC: hand-computed expected values for 3 fixtures; a property test checks that drawdown is ≥ 0 and ≤ 10000 bps.

**A-3 🟡 `report` example plus live fetch (native only)**
- `cargo run -p pnl-core --example report -- <file>`. Behind a `native-fetch` feature: fetch from HL using `HL_ADDRESS` from env.
- AC: the output matches the HL UI within $0.01 on a real account (manual check, noted in the PR).

---

## M3: TEE (D3, Sun Nov 8, gate 19:00 UTC / 22:00 MSK)

**A-4 🔴 OutLayer WASM job (pnl action)** (deps: A-2)
- `attestor/outlayer-job`, target `wasm32-wasip2`. It reads the input JSON from stdin and `HL_ADDRESS` from env (OutLayer secret), does the HTTPS fetch, and writes the metrics JSON to stdout.
- AC: runs locally under `wasmtime` with a mocked HTTP fixture; the output has no `0x…40 hex`; the build is reproducible (sha256 printed).

**A-5 🔴 Publish and secret**
- `scripts/outlayer-publish.sh`: `outlayer upload` + `outlayer deploy`, plus `outlayer secrets set '{"HL_ADDRESS":"…"}' --access whitelist:<contract>`. The script **reads the address from env and never echoes it**.
- AC: the project version sha256 is recorded in `docs/DECISIONS.md`.

**A-6 🔴 Contract ↔ OutLayer yield/resume**
- `request_pnl_update` uses `ext_outlayer::request_execution(code_source, resource_limits, input_data, secrets_ref, "Json", payer)`, plus the `on_pnl_result` callback (`#[private]`, `#[callback_result]`). Validate the output schema and code hash, then store.
- AC: a testnet tx shows the full path; `get_pnl_history` returns the record; a malformed output is rejected without panicking the whole tx (logged event).

**A-7 🟡 HTTPS fallback path**
- `scripts/attest-https.ts`: call `POST https://testnet-api.outlayer.ai/call/{owner}/{project}` and `submit_pnl` with an attestation reference.
- AC: works end-to-end; the UI labels it "relayed".

**GATE-1 🔴 Go/no-go decision**: Sun Nov 8 19:00 UTC (22:00 MSK). Record GO, partial, or NO-GO and the reason in `docs/DECISIONS.md`. On NO-GO, switch to the `G-*` tasks at the bottom.

---

## M4: Feed plus buy (D4, Mon Nov 9)

**F-1 🔴 Shared types**: `gateway/src/types.ts` (`TradeEvent`, `Tier`) plus a JSON schema in `docs/schemas/`; the copier imports the schema.
- AC: schema tests run in both TS and Python.

**F-2 🔴 Feed action in the WASM job**: delay per tier, size bucketing, time jitter, sanitized events.
- AC: a fixture-based test checks that no event is newer than now − delay and that sizes fall into buckets.

**F-3 🔴 NEP-413 auth**: `/auth/challenge`, `/auth/verify`, JWT session (secret from env).
- AC: tests cover a valid signature, a wrong account, a replayed nonce, and an expired token.

**F-4 🔴 SSE feed with gating**: `GET /feed/:traderId`; calls `has_access` (cached ≤ 30 s); 401/403 semantics; per-token concurrency of 1.
- AC: an integration test with a mocked RPC covers holder → 200 stream and non-holder → 403.

**F-5 🟡 Watermarking**: per-subscriber deterministic micro-jitter; the `scripts/trace-leak.ts` tool identifies the subscriber from a leaked stream.
- AC: the test identifies the right subscriber out of 10.

**P-1 🔴 1Click quote client**: `web/lib/oneclick.ts` with `dry: true` by default. Gets the token list from `/v0/tokens`. Optional `X-API-Key`.
- AC: a unit test with fetch mocked asserts `dry: true` unless `ONECLICK_LIVE=1`.

**P-2 🟡 Minter relayer**: polls `/v0/status`; calls `mint_key` on `SUCCESS`; handles `REFUNDED`/`FAILED`; idempotent.
- AC: tests with recorded status responses for each terminal state.

**P-3 🟡 Trustless USDC path**: frontend `ft_transfer_call` to the contract with `msg`.
- AC: a testnet manual check is logged in the PR (testnet USDC from the faucet).

---

## M5: Copier (D5, Tue Nov 10)

**K-1 🔴 Policy module** (pure): `decide(event, account, limits) -> Order | Reject`.
- AC: ≥ 1 test per limit (notional, leverage, daily loss, liquidation distance, allowed coins, kill switch, copy ratio rounding to HL lot size).

**K-2 🔴 SSE client plus dry-run executor**: `copier --dry-run` (the default) logs decisions as JSON lines.
- AC: running against the fixture stream gives the expected transcript (golden file).

**K-3 🟡 Live executor** (follower's own HL API wallet from env; tiny size): reuse the baseline order code.
- AC: one real copied trade on a 2nd account, with the tx hash or order id noted in the PR (no keys).

**K-4 🟡 Liquidation guard loop**: after each fill, compute the distance to liquidation and auto-reduce when it's below the threshold.
- AC: a unit test with a synthetic price path.

**K-5 🟡 Audit log plus web status endpoint**.
- AC: the web `/copier` page shows the last 50 decisions.

**K-6 ⚪ Agent API policy wallet** (demo env, rotating keys).
- AC: behind the `AGENT_API_ENABLED` flag; the Quickstart doesn't depend on it.

---

## M6: Web (D6, Wed Nov 11)

**W-1 🔴 Next.js scaffold plus near-connect login** (testnet). AC: login/logout works; the account id is shown.
**W-2 🔴 Leaderboard**: `list_traders` plus the latest `PnlRecord`, an Attested badge linking to the explorer tx and `code_hash`. AC: renders from testnet.
**W-3 🔴 Trader page plus buy**: tiers, NEAR buy, any-chain buy (dry quote panel), "Mint demo key" (testnet). AC: each button is wired; the dry quote is shown.
**W-4 🔴 Feed page**: SSE consumer; shows 403 nicely. AC: holder vs non-holder demo.
**W-5 🟡 Verify page**: shows the relayed record, the command that reproduces the metrics, and a flip-one-byte control. AC: Check record shows VERIFIED and digest `c77fa93b9c667bfefa078b647e55333a42266d4f0b2dc000a45500e0b52881d7`; Flip one byte shows NOT VERIFIED.
**W-6 🔴 Deploy to Vercel**: env from the dashboard only. AC: the public URL works in an incognito window.

---

## M7: Hardening and README (D7, Thu Nov 12)

**H-1 🔴 Leak tests**: CI fails if `0x[0-9a-fA-F]{40}` shows up in job output fixtures, gateway logs, or web bundles (with an allowlist for known public contract addresses).
**H-2 🔴 Failure states**: OutLayer timeout or refund, 1Click REFUNDED/FAILED, RPC errors, all shown in the UI.
**H-3 🔴 `make demo-seed`**: offline simulation (recorded fills to metrics to feed to copier dry-run) that needs no keys.
**H-4 🔴 Final README Quickstart**: every step is "do X, see Y" and was checked from a fresh clone in Docker (`scripts/fresh-clone-check.sh`).
**H-5 🟡 Threat-model doc update** with the measured delay/jitter defaults.

## M8: Ship (D8-D9, Fri Nov 13 to Sat Nov 14)

**V-1 🔴 Demo video** (2-3 min) · **V-2 🔴 X post draft** (Ilya posts) · **V-3 🔴 Full-history secrets scan plus go public** (Ilya flips visibility) · **V-4 🔴 Submission** by Sat Nov 14 17:00 UTC (20:00 MSK), receipt saved.

---

## Fallback tasks (only on NO-GO at GATE-1): Never-Liquidated guardian

**G-1** Guardian Policy NFT (reuse C-3): bands, max top-up per day, allowed sources.
**G-2** Monitor: HL margin ratio from `clearinghouseState` (reuse A-1/A-2 types).
**G-3** Rescue planner: cheapest source via 1Click **dry** quotes; slippage cap.
**G-4** Executor: author-only tiny real top-up (only if jurisdiction is OK), otherwise simulation.
**G-5** Crash replay simulator plus UI incident log.
