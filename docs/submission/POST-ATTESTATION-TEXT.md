# Text changes once Alpha One shows Attested

Apply only after TEE-RUNBOOK.md step 8 passes. `<TX4>` is the request_pnl_update transaction id. `<HASH>` is cb9c3130c6858cf4b460950281cff3468eabc0d79f1652fb05061c7fe41978dd unless your build printed another.

## README.md (on main, after merging tee-live)

Line 3, replace with:

> The PnL row for Alpha One is live TEE output. An OutLayer worker in Intel TDX holds the Hyperliquid address as a sealed secret, computes 7d PnL, drawdown, and Sharpe, and returns them to `alpha-keys.testnet` through yield/resume. The contract stores the row only if its code hash equals the pinned WASM (`<HASH>`). Transaction: https://testnet.nearblocks.io/txns/<TX4>. The fixture row from the first deploy stays on chain for the offline Quickstart.

Tracks section, replace the `security` line with:

> `security` stays off this submission: the attestation is live, but the viewing key is an app-layer right, not an audited security product. `agents` stays off: the copier dry-run does not spend.

(Or add `security` to the tracks if you want it. The callback is a real TEE proof now. I'd keep three tracks.)

Quickstart intro, replace the sentence "The judge PnL row is **relayed** from a fixture (`submit_pnl`)." with:

> The leaderboard row is a live OutLayer callback. The fixture row (`submit_pnl`) is what `make demo-seed` reproduces offline.

Quickstart step 6, replace with:

> 6. In another, `pnpm --filter web dev`, then open http://localhost:3000. You see Alpha One with the **Attested** badge and code hash `<HASH>`. That badge appears only for an OutLayer callback whose code hash equals `get_config`'s pinned hash.

Add a step after 9:

> 10. Reproduce the worker hash: `./scripts/build-worker.sh`. You see `wasm sha256 <HASH>`, the same value as the third field of `get_config` on `alpha-keys.testnet`.

Hosted demo line: replace `Demo video: _TBD (X post)._` with `Demo video: <X post URL>`.

## GitHub About text

    gh repo edit ilerik/alpha-keys --description "Viewing-key NFTs for a hidden trader. PnL is computed in an OutLayer TEE (Intel TDX) and attested on NEAR; the account stays sealed."

## Site (already automatic on main)

The leaderboard flips by itself: badge Attested, the real numbers, and the line "The Attested row is an OutLayer callback whose code hash matches the pinned WASM. The fixture row stays on chain. A public order book can still be fingerprinted." The Verify panel needs the tee-live merge. Optional Vercel env change: set `NEXT_PUBLIC_PNL_TX` to `<TX4>` so the explorer link on Verify points at the attested run, then redeploy.

## form-answers.md idea (1087 characters)

Alpha Keys is a viewing-key NFT for a trader who never publishes the account. The Hyperliquid address stays off the chain. An OutLayer worker in Intel TDX holds the address as a sealed secret, fetches fills and equity from Hyperliquid, and returns only 7d profit, drawdown, and Sharpe to a NEAR contract through yield/resume. The contract stores the row only if the worker's code hash equals the pinned WASM, and the build is reproducible, so anyone can rebuild the wasm and compare hashes. The live leaderboard on alpha-keys.testnet shows that attested row. Followers buy a NEP-171 Viewing Key (royalty on resale) and see a NEAR Intents 1Click dry quote for paying from another chain. The demo gateway returns 403 until a key is minted, then streams delayed, size-bucketed trades. The verify page shows the attested record and a relayed fixture that anyone can recompute with make demo-seed; flip one byte and it shows NOT VERIFIED. Hosted demo: https://alpha-keys.vercel.app. The feed unlock runs on localhost, as the README says. Judges follow the README with no secrets and no funds.

## x-post.txt

    built at NEAR HACKATHON

    Alpha Keys: a viewing-key NFT for a trader who never publishes the account. PnL is computed inside a TEE and attested on NEAR. The feed stays locked until you hold the key.

    https://alpha-keys.vercel.app

    @NEARProtocol #NEARHackathon

## Video (v3)

- Recapture the leaderboard shot: ATTESTED badge, real numbers, code hash. The RELAYED badge in v2 is then wrong.
- Final slide caveat, replace "PnL shown is a sample record on NEAR testnet. Live attested feeds are coming next." with "PnL attested by an OutLayer TEE worker on NEAR testnet. Feed unlock shown from the local demo gateway."
- Optional new 4 s beat before the pitch: the Verify page attested panel with code hash = pinned hash. Voice: "The numbers come from code running in a secure enclave. Anyone can check the hash."
- The real 7d PnL is a small loss. If you keep it on screen, the voice line "a trader you can verify" still works; avoid "top trader" wording.

## docs/video-outline.md

Replace "The badge on screen is Relayed. The numbers are the fixture: PnL 8 USD, drawdown 10 bps, code hash `relayed-demo`." with "The badge on screen is Attested. The code hash is `<HASH>`." and drop "A claim that the on-screen row is a live enclave callback" from Never show.
