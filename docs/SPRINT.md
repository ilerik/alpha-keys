# Sprint: remaining days to submission

Status: written Tue Nov 10, 2026, 22:00 Yekaterinburg (UTC+5). The remote repo is still the Nov 5 skeleton (commit `cf1d78d`). This file replaces the calendar in `PLAN.md` for the rest of the hackathon. Task IDs still refer to `docs/TASKS.md`. Paths are repo-relative.

Ilya builds in Cursor, solo. Do not submit, post, pay, or make the repo public without his explicit go-ahead. The Saturday checklist names those steps and who does them.

## Clock

| Milestone | Yekaterinburg (UTC+5) | UTC |
|---|---|---|
| Now / sprint start | Tue Nov 10, 22:00 | 17:00 |
| TEE decision | Wed Nov 11, 23:00 | 18:00 |
| Feature freeze | Fri Nov 13, 17:00 | 12:00 |
| X post + video | Sat Nov 14, 20:00 | 15:00 |
| Internal hard stop (submit, repo public) | Sat Nov 14, 22:00 | 17:00 |
| Official deadline | Sun Nov 15, 04:59 | Sat Nov 14, 23:59 |

Judging is Nov 16-18. A hosted demo should stay up through Nov 25.

## The winning slice

Ship a demo a judge (or an AI screener) can finish in a few minutes:

1. A real Hyperliquid account is trading, or has recent fills. The address never appears in the repo, logs, or job output.
2. PnL, max drawdown, and Sharpe are computed in `attestor/pnl-core` and a record is stored on the `alpha-keys` contract on NEAR testnet, with a hash a judge can check.
3. A follower buys one Viewing-key NFT (NEP-171). In the judge path, payment is a 1Click `dry: true` quote plus a testnet "Mint demo key" (or a small NEAR `buy_key`).
4. With the key, the feed unlocks (SSE or a simple poll). Without it, the gateway returns 403.
5. On a verify page or CLI: change one byte of the attestation or hash. The result is NOT VERIFIED.

Everything else is polish or a cut.

### Must (in the video and the README Quickstart)

- Contract on testnet: trader registry, one minted key, `has_access`, at least one PnL record (`contracts/alpha-keys`).
- Deterministic `pnl-core` metrics on a fixture and on a real account (`attestor/pnl-core`).
- An attestation path: OutLayer or the signed-hash fallback (see TEE decision).
- A 1Click dry quote, visible in the UI or via `scripts/quote-dry.sh`.
- A gated feed: holder sees sanitized trades, non-holder gets 403 (`gateway/`).
- A verify step with the flip-one-byte moment (`web/` or a script plus the README).
- A README with numbered "do X, see Y" steps that work from a fresh clone with no secrets and no funds.

### Stretch (only after the must slice is recordable)

- Copier `--dry-run` against the feed (`copier/`, K-1 and K-2).
- OutLayer feed job (F-2), if the TEE path is GO.
- Hosted Vercel demo (W-6).
- Verify-page polish (W-5) and leak tests (H-1).
- Confidential 1Click funding of a fresh HL account (S-5), only if the jurisdiction question is answered and the must slice is already green.

### Cut by default

Do not start these unless the must slice is green and that day's go/no-go says there is spare time:

- Live copier executor, liquidation-guard loop, Agent API (K-3 to K-6).
- 1Click live minter relayer and the NEP-141 USDC path (P-2, P-3).
- Feed watermarking (F-5).
- NEAR AI Cloud trade explainer.
- Guardian / Never-Liquidated pivot (G-*). Too late unless the whole Alpha Keys slice is abandoned before Wednesday night, which is not the plan.

## Cut lines

Stop the current side path at the first line you hit. Write the decision in `docs/DECISIONS.md` that same night.

1. **Wed Nov 11, 23:00 YEKT (18:00 UTC): TEE decision.** If an OutLayer job has not produced a verifiable record on testnet (or a clear path to one on Thursday morning), switch to the signed-hash fallback and do not spend Thursday morning on OutLayer.
2. **Thu Nov 12, 23:00 YEKT (18:00 UTC): demo is recordable, or cut the web.** If a judge cannot buy (or mint) a key and see the feed 403/unlock plus the flip-one-byte check, freeze a CLI + minimal page and spend Friday on the README and the video, not on more UI.
3. **Fri Nov 13, 17:00 YEKT (12:00 UTC): feature freeze.** After this, only README, video, bugfixes, secrets scan, and submission. No copier, no funding, no new endpoints.
4. **Copier.** Start K-1/K-2 only if Thursday's go/no-go is green and Friday morning has at least three spare hours. Otherwise the demo mentions the policy stub and stops.
5. **Confidential funding.** Stay on a pre-existing HL account and say so in the README, unless jurisdiction is cleared and the must slice is already recorded.
6. **Sat Nov 14, 22:00 YEKT (17:00 UTC): internal hard stop.** Submit and make the repo public even if polish is missing. Do not keep coding toward 04:59 YEKT.

## TEE decision (Wednesday night)

Two acceptable paths. Pick one at Wed Nov 11, 23:00 YEKT (18:00 UTC) and record it in `docs/DECISIONS.md`.

**Path A: OutLayer (preferred).** `attestor/outlayer-job` runs in Intel TDX, `HL_ADDRESS` is an OutLayer secret, and the contract callback `on_pnl_result` (A-6) or the HTTPS fallback `submit_pnl` (A-7) stores the record with `code_hash`. Flip-one-byte: mutate the stored hash or the attestation blob, and verification fails.

**Path B: signed-hash fallback.** A local or script attestor (same `pnl-core`) writes metrics plus `sha256(output)` and a signature from a pinned attestor key. The contract accepts `submit_pnl` only from that key and stores `code_hash` / `input_hash` / output. The verify step recomputes the hash and checks the signature. Flip-one-byte still works. The UI and README must say this is weaker than a TEE attestation: it trusts the pinned key, not Intel TDX.

Do not invent a Path C that posts raw PnL with no hash. The NOT VERIFIED moment is part of the winning slice.

## How to use this in Cursor

One task at a time. Prefer the paths and done-when lines below over the old D0-D9 dates in `PLAN.md`. Commit often. Never commit `.env`. If you import baseline Hyperliquid fleet code, do it as its own commit, tag `pre-hackathon`, and list the files in the README (S-2) before anything else.

Daily go/no-go: at the end of each day, spend 10 minutes on three questions. (1) Does today's done-when hold? (2) Is the must slice still on track for Thursday night? (3) What gets cut tomorrow? Log the answer in `docs/DECISIONS.md`.

---

## Tue Nov 10, 22:00 to Wed 01:00 YEKT (17:00-20:00 UTC) - Foundation

**Goal.** The contract and the PnL engine can stand up on their own: a trader exists, a key can be minted, and metrics can be computed from fills.

**Hours.** About 3. If you stop earlier, start Wednesday with the leftover, not with the TEE.

**Tasks**

- `contracts/alpha-keys/src/lib.rs` (C-2, C-3, C-4 minter/`buy_key`, C-5, C-6 storage): `register_trader`, Viewing-key NFT (NEP-171 is enough; royalties can stay thin), `mint_key` for the demo path, `has_access`, `PnlRecord` + `submit_pnl` + `get_pnl_history`. No Hyperliquid address in state. Unit tests.
- `attestor/pnl-core` (A-1, A-2): parse a fills fixture, compute PnL, max drawdown, and Sharpe as fixed-point ints. Keep `max_drawdown_bps`. Add `fixtures/fills/sample.json` with the address stripped.
- If time: `scripts/deploy-testnet.sh` and a testnet deploy (C-6). If not, that is the first job Wednesday morning.
- Do not start OutLayer, the gateway, or the web tonight.

**Done when**

- `cargo test -p alpha-keys` and `cargo test -p pnl-core` pass.
- You can mint a key in unit tests and `has_access` returns the tier.
- `cargo run -p pnl-core --example report -- fixtures/fills/sample.json` (or the equivalent) prints metrics JSON.

**Go/no-go.** If the contract still does not mint a key by 01:00, sleep and finish it before any TEE work on Wednesday. That is the blocker.

---

## Wed Nov 11 - Attest, key, and feed (core day)

**Goal.** The must slice works on testnet without a pretty UI: real (or fixture-backed real) PnL is on-chain, one key gates a feed, and verification fails on a flipped byte. The TEE path is chosen at night.

**Hours.** About 9-10 (morning: finish contract/PnL and deploy; afternoon: attestation and feed; evening: decision).

**Tasks**

- Morning: finish Tuesday leftover. Deploy `contracts/alpha-keys` to testnet. Keep the contract id in local `.env`; mention it in the README once it is stable. Register one trader. Compute metrics for a real HL account on your machine (`HL_ADDRESS` in `.env`, never printed). Store a record with `submit_pnl` so the contract path is alive even before OutLayer (A-3, C-6).
- Attestation, Path A attempt (A-4, A-5, then A-6 or A-7): build `attestor/outlayer-job` to `wasm32-wasip2`, keep `HL_ADDRESS` sealed, print metrics on stdout with no address. Time-box this. If the yield/resume callback is sticky, use the HTTPS + `submit_pnl` variant and label it in the UI.
- Feed, thin path (F-1, F-3 if quick, F-4): `gateway/` checks `has_access` and serves SSE. The gateway must not see the HL address. Use a local publisher script (author machine, `.env` only) that sanitizes fills (60 s delay, no address) and pushes events into the gateway, unless the OutLayer feed job (F-2) is already running. Tests: holder 200, non-holder 403. NEP-413 is ideal; a documented demo session is acceptable for Wednesday if NEP-413 slips.
- Buy path, thin (P-1, C-4): `scripts/quote-dry.sh` or `gateway/src/oneclick.ts` shows a 1Click `dry: true` quote. A "Mint demo key" call on testnet (minter role) or `buy_key` with testnet NEAR actually mints the NFT. Live 1Click payment is out of scope.
- A verify script or a README snippet: given a record, recompute `sha256`, flip one byte, print NOT VERIFIED. This can move to a page on Thursday.

**Done when**

- On testnet, `get_trader` and `get_pnl_history` return data, and `nft_tokens_for_owner` shows one key.
- Curl or a test: no key -> 403; with a key -> at least one sanitized trade event.
- Flip one byte of the hash or attestation -> NOT VERIFIED.
- `docs/DECISIONS.md` has a TEE decision, Path A or Path B, with a one-line reason.

**Go/no-go (23:00 YEKT / 18:00 UTC).** See the TEE cut line. If the feed is not gating tonight, that is Thursday's first job, ahead of any UI chrome.

---

## Thu Nov 12 - Close the loop in a UI

**Goal.** A judge can click through the story: attested leaderboard, buy or mint a key, feed unlocks, flip one byte.

**Hours.** About 8-10.

**Tasks**

- `web/` (W-1 to W-4, then W-5): Next.js, `@hot-labs/near-connect` on testnet, leaderboard with an Attested badge (tx + `code_hash`), trader page with the dry 1Click quote and Mint demo key / `buy_key`, feed page that handles 403, verify page with the flip-one-byte control.
- Wire whichever attestation path you chose on Wednesday. Do not reopen the other path unless Path A arrives with little extra work.
- W-6: deploy to Vercel if the app boots. If deploy fights you past about two hours, record a local demo and move on.
- Optional only if the click-through is already recordable: start copier policy tests (K-1), not the live executor.

**Done when**

- From a testnet wallet (or a documented demo mint): leaderboard -> buy/mint -> feed plays; a second window without a key sees 403; verify shows VERIFIED, then NOT VERIFIED after one flipped byte.
- You can capture this in a screen recording, even if the styling is plain.

**Go/no-go (23:00 YEKT / 18:00 UTC).** If the click-through is not recordable, cut further UI and lock a CLI + thin page for the video. Do not start the copier.

---

## Fri Nov 13 - Harden, README, freeze

**Goal.** An AI screener can clone the repo and follow the README. The demo is boringly reliable. Features stop at 17:00 YEKT.

**Hours.** About 8. Feature freeze at 17:00 YEKT (12:00 UTC).

**Tasks (before freeze)**

- Only leftover must-slice bugs.
- Copier K-1/K-2 (`copier/`, `--dry-run`, fixture transcript) only if Thursday was green and you have a clear three-hour block before freeze. Otherwise cut.
- `make demo-seed` (H-3) for the offline path: fixture fills -> `pnl-core` -> sample feed. Include the copier only if K-2 landed.

**Tasks (all day, including after freeze)**

- README Quickstart (H-4): numbered "do X, see Y", no secrets, no funds, testnet + `dry: true`. Disclose AI assistance, libraries, and pre-hackathon code (or state that none was imported). List the NEAR tech actually used.
- Leak check (H-1): no `0x` + 40 hex in job outputs, fixtures, or logs.
- Failure copy (H-2) for the paths you actually shipped (OutLayer timeout and/or 1Click dry-only).
- Draft the video outline and the X post text (V-1, V-2). Do not post.
- If you imported baseline code and have not tagged yet, tag `pre-hackathon` now.

**Done when**

- A fresh clone (ideally `scripts/fresh-clone-check.sh`, or a clean directory) passes every Quickstart step.
- Feature freeze holds: no new features after 17:00 YEKT.
- The video outline is shot-by-shot, including the flip-one-byte moment.

**Go/no-go.** If the Quickstart still needs a hidden `.env` secret, fix that before any filming. If Vercel is down, the README must work locally and the video must not depend on the hosted URL.

---

## Sat Nov 14 - Record, publish, submit

**Goal.** Public repo, demo video, X post, one submission. Stop at the internal hard stop.

**Hours.** About 8, with a hard stop at 22:00 YEKT (17:00 UTC). The official deadline is 04:59 YEKT Sunday (23:59 UTC Saturday). Do not use that buffer for features.

### Morning to 16:00 YEKT (11:00 UTC)

- Record the 2-3 min video (V-1): problem, attested leaderboard, dry quote + key, feed 403 then unlock, flip one byte -> NOT VERIFIED, and optional copier dry-run if it shipped.
- Full-history secrets scan (gitleaks or trufflehog). Fix anything it finds. Confirm `.env` was never committed (V-3).
- Tag `hackathon-submission` (provisional until the form is sent). If baseline code was imported, confirm the `pre-hackathon` tag and add a `compare/pre-hackathon...hackathon-submission` link to the README.
- LICENSE is already MIT. Leave it as is.

### By 20:00 YEKT (15:00 UTC): X post

- Ilya posts from his own account. The post must say "built at NEAR HACKATHON", tag @NEARProtocol and #NEARHackathon (optionally @NearLegion and @NearBuilders), and attach the video. Draft only until he posts.

### By 22:00 YEKT (17:00 UTC): go public and submit

- Ilya makes https://github.com/ilerik/alpha-keys public (V-3).
- Submit once at https://nearhackathon.com/#apply (or the official `POST /api/applications` if he prefers). Corrections go through https://nearhackathon.com/request-changes, not a second application.
- Suggested fields: projectName "Alpha Keys", size `1`, tracks `["defi","privacy","nfts","agents"]`, projectUrl = the public repo, demoUrl = the X post, nearAddress = his mainnet account (lowercase), idea 20-2000 chars, consent=true, a requestId UUID reused on retry, no `website` field.
- Save a 201 receipt to `docs/submission-receipt.md` with no secrets (V-4).

**Done when**

- The repo is public, LICENSE is present, and the README Quickstart is numbered "do X, see Y".
- The X post is live with the required phrase, tags, and video.
- The submission receipt is saved.
- The hosted demo, if you have one, is left up (target: through Nov 25).

**Go/no-go.** At 22:00 YEKT, if anything in the checklist is still open, do only that item. No new features. After submit, walk away.

## Submission checklist (tick on Saturday)

- [ ] `pre-hackathon` tag exists if baseline code was imported; README lists those files and a compare link
- [ ] `hackathon-submission` tag
- [ ] LICENSE (MIT) present
- [ ] README: what it is, the problem, numbered "do X, see Y" Quickstart, NEAR tech used, libraries, AI assistance, honest limitations (public HL book, app-layer viewing key, 1Click `dry: true`, attestation path A or B)
- [ ] Quickstart works from a fresh clone with no secrets and no funds
- [ ] Full-history secrets scan is clean; `.env` was never committed
- [ ] Demo video, 2-3 min, includes flip-one-byte -> NOT VERIFIED
- [ ] X post by Ilya: "built at NEAR HACKATHON", @NEARProtocol, #NEARHackathon, video attached, by 20:00 YEKT (15:00 UTC)
- [ ] Repo made public by Ilya
- [ ] One submission at https://nearhackathon.com/#apply by 22:00 YEKT (17:00 UTC)
- [ ] Receipt saved; hosted demo left up if it exists
