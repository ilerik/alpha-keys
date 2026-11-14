# Handoff for the submission bot

Start at `main` commit `05e1df0` or later. The repo is still private. Do not open `.env`, print `HL_ADDRESS`, or show a private key.

## Name

Keep the name **Alpha Keys**. A search on 14 Nov 2026 found no NEAR, copy-trading, or NFT product with that name. Nearby names are different things: a typing-practice site called AlphaKeys, Unity gradient alpha keys, and game whitelist "Alpha Key" passes. This project's names are `alpha-keys.testnet`, `github.com/ilerik/alpha-keys`, and `alpha-keys.vercel.app`.

## What is already true

- Leaderboard row is an OutLayer callback, not the fixture. Tx `ECyHsbAmWfuo19Hp8GFdUn5C2wNpxri97ABrHJzihapa`. Worker hash `cb9c3130c6858cf4b460950281cff3468eabc0d79f1652fb05061c7fe41978dd`.
- On screen: Attested, about 27.92 USD PnL, 10000 bps drawdown, Sharpe 0.00, 114 trades. Equity is dust. The curve counts a withdrawal as drawdown. Do not say "top trader".
- The fixture row stays for `make demo-seed`. Flip-one-byte still uses that fixture.
- Tracks: `nfts`, `privacy`, `defi`. Leave `security` and `agents` off.
- One form only, at https://nearhackathon.com/#apply. Never fill `website`. `demoUrl` is the X post, not the Vercel URL.
- The idea paragraph in `docs/video-outline.md` matches the live callback. Use that text on the form.

## What you do

Follow `docs/video-outline.md` for the recording, then `docs/submission/CHECKLIST.md` from step 2. Wait until https://alpha-keys.vercel.app/ shows the Attested badge before you record. Do not commit `docs/demo.webm`.

GitHub About text:

Viewing-key NFTs for a hidden trader. PnL is computed in an OutLayer TEE (Intel TDX) and attested on NEAR; the account stays sealed.
