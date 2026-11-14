# Demo video and public handoff

Record a new video. Do not commit `docs/demo.webm`. Ilya posts the X text and sends the form. One application only.

After `origin/main` updates, wait until https://alpha-keys.vercel.app/ shows this footer: "Token 0 is a Delayed60 key on alpha-keys.testnet. Attested means an OutLayer callback whose code hash matches the pinned WASM." Then record.

## Never show

- `.env`, `HL_ADDRESS`, a private key, a seed phrase, or the trader account
- Terminal scrollback
- `scripts/outlayer-publish.sh` output

The badge on screen is Attested. The code hash is `cb9c3130c6858cf4b460950281cff3468eabc0d79f1652fb05061c7fe41978dd`. The live 7d row is about 27.92 USD PnL, 10000 bps drawdown, Sharpe 0.00, and 114 trades. Current equity is dust, and the curve counts a withdrawal as drawdown. Do not say "top trader".

## Record

2-3 minutes. Leaderboard and Verify can be the hosted site. The feed moment is localhost.

1. Problem. One sentence: the account stays hidden, the track record is public, the feed is a key. Show the leaderboard: Alpha One, Attested badge, code hash `cb9c3130c685`, about 27.92 USD PnL, 10000 bps drawdown.
2. Dry quote. Open Alpha One. Click **Buy key → Pay from another chain**. Show `dry: true`, an `amountOut`, and `depositAddress: null`.
3. Locked feed. In two terminals, from a fresh clone: `pnpm --filter gateway start`, then `pnpm --filter web dev`. Open http://localhost:3000/feed/alpha-1. Click **Open feed**. Show `403 no viewing key`.
4. Unlock. Click **Mint demo key**. Show `Trades are streaming.` and the ETH lines.
5. Verify. Open Verify. Click **Check record**. Show `VERIFIED` and digest `c77fa93b9c667bfefa078b647e55333a42266d4f0b2dc000a45500e0b52881d7`. Click **Flip one byte**. Show `NOT VERIFIED`.
6. Optional, only if it is already on screen: the copier dry-run line `coin_not_allowed:DOGE` from `make demo-seed`. Do not show a live order.

Say these limits out loud: the PnL row is an OutLayer callback, the 10000 bps drawdown includes withdrawals, the quote is dry, and the demo mint is in the local gateway.

Hosted pages, no login:

- https://alpha-keys.vercel.app/
- https://alpha-keys.vercel.app/t/alpha-1
- https://alpha-keys.vercel.app/verify
- Attested tx: https://testnet.nearblocks.io/txns/ECyHsbAmWfuo19Hp8GFdUn5C2wNpxri97ABrHJzihapa
- Fixture tx: https://testnet.nearblocks.io/txns/6uDLJJ5s9URhR79Gt6P97r9p54HPdgSEWWdgEuYpyekF
- Token 0: https://testnet.nearblocks.io/nft-token/alpha-keys.testnet/0

## Go public

Confirm `git ls-files` has no `.env` and `make check` prints `leak check: no address-shaped hex`. Then make https://github.com/ilerik/alpha-keys public.

GitHub About text:

Viewing-key NFTs for a hidden trader. PnL is computed in an OutLayer TEE (Intel TDX) and attested on NEAR; the account stays sealed.

## X post (Ilya posts this himself, then submits)

```
built at NEAR HACKATHON

Alpha Keys: a viewing-key NFT for a trader who never publishes the account. PnL is computed inside a TEE and attested on NEAR. The feed stays locked until you hold the key.

https://alpha-keys.vercel.app

@NEARProtocol #NEARHackathon
```

Attach the video. `demoUrl` on the submission form is this post, not the Vercel URL.

## Submission fields

Ilya sends one application at https://nearhackathon.com/#apply. Do not send a second one.

- projectName: Alpha Keys
- size: 1
- tracks: nfts, privacy, defi
- projectUrl: https://github.com/ilerik/alpha-keys
- demoUrl: the X post. If the post is not up yet, submit with the public repo URL and correct `demoUrl` once through https://nearhackathon.com/request-changes
- nearAddress: his mainnet account, lowercase
- idea: the paragraph below
- consent: true
- no `website` field

`security` stays off. The viewing key is an app-layer right, not an audited security product. `agents` stays off. The copier dry-run does not spend.

Idea:

Alpha Keys is a viewing-key NFT for a trader who never publishes the account. The Hyperliquid address stays off the chain. An OutLayer worker in Intel TDX holds the address as a sealed secret, fetches fills and equity from Hyperliquid, and returns only 7d profit, drawdown, and Sharpe to a NEAR contract through yield/resume. The contract stores the row only if the worker's code hash equals the pinned WASM, and the build is reproducible, so anyone can rebuild the wasm and compare hashes. The live leaderboard on alpha-keys.testnet shows that attested row. Followers buy a NEP-171 Viewing Key (royalty on resale) and see a NEAR Intents 1Click dry quote for paying from another chain. The demo gateway returns 403 until a key is minted, then streams delayed, size-bucketed trades. The verify page shows the attested record and a relayed fixture that anyone can recompute with make demo-seed; flip one byte and it shows NOT VERIFIED. Hosted demo: https://alpha-keys.vercel.app. The feed unlock runs on localhost, as the README says. Judges follow the README with no secrets and no funds.

Save the 201 body to `docs/submission-receipt.md` after the form accepts it. This file is the outline, not the receipt. The form has not been sent.
