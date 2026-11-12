# Alpha Keys: build plan

All times are written **UTC first, then Moscow time (MSK, UTC+3) in parentheses**.

| Milestone | When |
|---|---|
| Code window opened (only work since then is judged) | Oct 19, 2026 |
| Today / D0 | Thu Nov 5, 2026 |
| **Go/no-go gate: TEE attestation** | **Sun Nov 8 19:00 UTC (22:00 MSK)** |
| Feature freeze | Fri Nov 13 12:00 UTC (15:00 MSK) |
| X demo post published | by Sat Nov 14 15:00 UTC (18:00 MSK) |
| Internal hard stop: submission sent | Sat Nov 14 17:00 UTC (20:00 MSK) |
| **Official deadline: repo public + submitted** | **Sat Nov 14 23:59 UTC (Sun Nov 15 02:59 MSK)** |
| Judging | Nov 16 to Nov 18 (results by Nov 18 23:59 UTC / Nov 19 02:59 MSK) |
| Hosted demo stays up | until at least Nov 25 |

Granular tasks with acceptance criteria are in [docs/TASKS.md](docs/TASKS.md); the IDs below (e.g. `C-1`) refer to it.

---

## Day-by-day

### D0: Thu Nov 5: setup and first contact with every API
- Tasks
  - Repo skeleton, plan, and agent rules (this commit). Tag `pre-hackathon` on the first commit that imports any baseline fleet code (S-2).
  - Accounts: NEAR testnet dev account (`alpha-keys-dev.testnet`) plus a mainnet payout account. Optionally get a 1Click partner JWT at partners.near-intents.org (S-3). Claim NEAR AI credits (stretch only).
  - Toolchain: `rustup target add wasm32-unknown-unknown wasm32-wasip2`, `cargo-near`, `near` CLI (near-cli-rs), `outlayer` CLI, Node 22+, pnpm, uv.
  - `scripts/quote-dry.sh`: one `dry: true` 1Click quote from NEAR USDC to Hyperliquid USDC (S-4).
  - **Ask the organizers** (Discord/Telegram) about payout/KYC checks and whether 1Click is usable from Russia (see Open questions). Decide on the funding path based on the answer.
- Deliverable: private repo pushed; toolchain installed; dry quote JSON saved to `fixtures/quotes/`.
- Done-check: `make check` runs (stubs only) and `scripts/quote-dry.sh` prints a quote with `amountOut`.

### D1: Fri Nov 6: NEAR contract core
- Tasks: C-1 to C-6. Trader registry, Viewing-Key NFT (NEP-171/177/178/181/199) with tiers, NEAR-native `buy_key`, minter role, `PnlRecord` storage with an authorized-attestor check, view methods. Unit tests plus `near-workspaces` sandbox tests. Deploy to testnet.
- Deliverable: `alpha-keys.<dev>.testnet` deployed and initialized.
- Done-check: `near view … get_trader` returns a trader, and `near view … nft_tokens_for_owner` shows a minted key.

### D2: Sat Nov 7: PnL engine plus the real trader account
- Tasks: A-1 to A-3 (`pnl-core`: fills to PnL, max drawdown, Sharpe, deterministic and fixture-tested). Import the baseline HL client and list it in the README disclosure (S-2). Fund a fresh HL account via a tiny **confidential** 1Click swap, **only if the jurisdiction answer allows it**. Otherwise use a pre-existing test account and say so (S-5). Start one fleet bot on it with minimal size.
- Deliverable: `cargo run -p pnl-core --example report -- fixtures/fills/sample.json` prints the metrics JSON. A live account is trading.
- Done-check: the report for the real account matches HL's UI PnL within rounding.

### D3: Sun Nov 8: TEE attestation (gate day)
- Tasks: A-4 to A-7. Port `pnl-core` into the OutLayer WASM job (`wasm32-wasip2`); store the HL address as an OutLayer secret (whitelisted to the contract/project); publish the WASM by sha256; the contract's `request_pnl_update` calls `outlayer.testnet` `request_execution` and the `on_pnl_result` callback stores the record (wasm hash, input hash, output).
- Deliverable: an attested PnL record on testnet, visible through `get_pnl_history`.
- Done-check: a tx on the testnet explorer shows `request_execution` and then the `on_pnl_result` callback, and the stored `code_hash` equals the published WASM sha256.
- **GO/NO-GO at Sun Nov 8 19:00 UTC (22:00 MSK)**, see below.

### D4: Mon Nov 9: gated feed plus the buy flow
- Tasks: F-1 to F-5 (feed job inside OutLayer: verify the NFT via NEAR RPC, apply the tier delay and size jitter, return sanitized trades; gateway: NEP-413 auth, SSE stream, 403 for non-holders). P-1 to P-3 (1Click buy: dry quote in the UI; live path pays a treasury and the minter relayer mints on `SUCCESS`).
- Deliverable: a holder sees trades and a non-holder gets 403.
- Done-check: `pnpm --filter gateway test` passes; a manual curl with and without a key gives 200 SSE and 403.

### D5: Tue Nov 10: copier agent
- Tasks: K-1 to K-5. Subscribe to the feed, size orders to the follower's equity, enforce the policy (max notional, max leverage, daily loss cap, liquidation-distance guard, kill switch), keep a dry-run mode, and keep an audit log. Optional: Agent API policy wallet (K-6) if the demo keys work.
- Deliverable: one copied trade on a second HL account (tiny size) plus a dry-run transcript.
- Done-check: `copier --dry-run` on the fixture stream shows accepted and rejected orders with their reasons; the tests cover every limit.

### D6: Wed Nov 11: frontend plus hosting
- Tasks: W-1 to W-6. Next.js app: leaderboard with an "Attested" badge (links to the tx and WASM hash), trader page with the PnL chart, buy key (NEAR or any chain through 1Click), live feed panel, copier settings, "verify attestation" panel. Deploy to Vercel against testnet.
- Deliverable: a hosted demo URL.
- Done-check: a fresh browser with a testnet wallet can complete the flow: leaderboard, mint demo key, feed.

### D7: Thu Nov 12: hardening plus the judge README
- Tasks: H-1 to H-5. Threat-model pass (delay/jitter defaults, key-sharing limits, attestation verification doc). Failure states (1Click `REFUNDED`/`FAILED`, OutLayer timeout). A `demo-seed` simulation mode. Finalize the README Quickstart ("do X, see Y"). Run the Quickstart from a fresh clone with Claude Code or Codex.
- Deliverable: a README that an AI screener can reproduce.
- Done-check: a fresh-clone run in a clean container passes every Quickstart step with no secrets set.

### D8: Fri Nov 13: video, post draft, go public
- Tasks: V-1 to V-4. Record a 2–3 min video: problem, attested leaderboard, buy key, feed unlock, copy fires, verify attestation. Draft the X post. Run a secrets scan over the **full git history** (gitleaks or trufflehog). Tag `hackathon-submission` (provisional). **Make the repo public** (Ilya does this himself).
- Deliverable: video file, post draft, public clean repo.
- Done-check: `gh repo view --json visibility` returns PUBLIC and the history scan is clean.

### D9: Sat Nov 14: ship
- Tasks: final fixes only (no features). Ilya publishes the X post by **15:00 UTC (18:00 MSK)**. It says "built at NEAR HACKATHON", tags @NEARProtocol and #NEARHackathon (optionally @NearLegion and @NearBuilders), and attaches the video. Submit via the form or `POST /api/applications` by **17:00 UTC (20:00 MSK)**. Re-tag `hackathon-submission`.
- Deliverable: a 201 response saved to `docs/submission-receipt.md` (no secrets).
- Done-check: the receipt is saved, the hosted demo is up, and the repo is public.

---

## Go/no-go gate: Sun Nov 8 19:00 UTC (22:00 MSK)

**GO** if all three hold:
1. The OutLayer WASM job runs on testnet with the HL address taken from an OutLayer secret (it isn't in the input or in the code).
2. The `on_pnl_result` callback stores a record on the contract with `code_hash` and the output.
3. A second run reproduces the same output for the same fills snapshot.

**Partial GO:** OutLayer runs, but the on-chain yield/resume path fails. Use the OutLayer **HTTPS API** instead: a relayer posts the result plus the OutLayer attestation to `submit_pnl`, and the contract accepts it only from the registered attestor key. Write down the weaker trust model.

**NO-GO:** pivot to **"Never-Liquidated guardian"**. It reuses D1–D2 plumbing (contract skeleton, HL client, `pnl-core` risk math, 1Click dry quotes):
- Contract: replace the NFT key with a **Guardian Policy NFT** (NEP-171, so the `nfts` track still fits) that encodes the bands (min margin ratio, max top-up per day, allowed source chains).
- Agent: a monitor that reads the HL margin ratio (and optionally an Aave or Rhea health factor). When a band is crossed, it plans a top-up via a 1Click quote from the cheapest source chain (`dry: true` for judges, one tiny real top-up if allowed).
- A simulation mode that replays a historical crash against a fake position.
- Tracks become `defi`, `security`, `agents` (+ `nfts`). Days D4–D9 keep the same shape: D4 is the planner, D5 the executor, D6 the UI, D7 the hardening.
- Record the decision and its reason in `docs/DECISIONS.md` that night.

---

## Submission checklist
- [ ] Repo **public** on Thu Nov 12, not Saturday night. Near Hunt lists the project only after the form is sent, and votes need the time before the deadline.
- [ ] `LICENSE` (MIT) present.
- [ ] README: what it is, the problem, numbered **"do X, see Y"** Quickstart, NEAR tech used, libraries, AI assistance, pre-hackathon disclosure with `pre-hackathon` and `hackathon-submission` tags and a compare link.
- [ ] Quickstart passes from a fresh clone with **no secrets and no funds** (dry quotes, testnet).
- [ ] Hosted demo up (Vercel) through Nov 25.
- [ ] Demo video, 2–3 min.
- [ ] X post by Ilya: "built at NEAR HACKATHON", @NEARProtocol, #NEARHackathon, video attached.
- [ ] Full-history secrets scan is clean; `.env` was never committed.
- [ ] Submission fields: projectName "Alpha Keys", leadName, email, size `1`, tracks `["defi","privacy","nfts","agents"]`, nearAddress (mainnet, lowercase), projectUrl (repo), demoUrl (X post), idea (20–2000 chars), consent=true, a requestId UUID reused on retry, **no `website` field**.
- [ ] Submit **once**. Corrections go through nearhackathon.com/request-changes.
- [ ] If Ilya submits a second project, each README names the other.

---

## Open questions (resolve early, log answers in docs/DECISIONS.md)
1. **Jurisdiction / 1Click terms.** The 1Click ToS (updated 2026-10-07) bars "Restricted Persons" and anyone located in a "Restricted Jurisdiction", which includes any place Intents Technology designates. No list is published, and VPN workarounds are forbidden. **Is Russia covered?** Until this is answered: use `dry: true` quotes only, make no live 1Click swaps from Russia, and don't fund the trader account through 1Click. Ask the organizers or NEAR Intents support. Also ask whether hackathon payouts involve KYC or sanctions screening. (Not legal advice.)
2. **Multi-track winning.** You can enter several tracks, but the rules don't say whether one project can win in more than one. Ask in Discord. It doesn't change the build, only the framing and the order of tracks.
3. **OutLayer vs NEAR AI Cloud for the TEE part.** Plan: OutLayer is primary. It runs arbitrary WASM, has sealed secrets, has a NEAR yield/resume callback, and produces a per-execution TDX attestation. NEAR AI Cloud is private *inference*, so it fits only an optional "trade explainer" and is a weak fallback for attesting arithmetic. Decide at the gate.
4. **Agent API key rotation.** Agent API keys come from the demo environment (`api.demo.agentsonintents.com`) and "rotate after the demo period", with no date given. Anything that depends on it must be optional, must degrade gracefully, and must stay out of the judge Quickstart.
5. Does OutLayer's testnet support outbound HTTPS to `api.hyperliquid.xyz` from WASI with the resource limits we need? Verify on D3 morning.
6. Is the 1Click "deposit to treasury, then mint on SUCCESS" trust model acceptable, or should we do a NEP-141 `ft_transfer_call` buy (USDC on NEAR) as the trustless path and keep 1Click as the convenience path? Current plan: build both, with the trustless one first.
