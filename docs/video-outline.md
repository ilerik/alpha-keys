# Demo video outline

2-3 minutes. Record the local Quickstart. The hosted Vercel URL, once it exists, can open the leaderboard and the verify check. The feed moment is localhost.

1. Problem. One sentence: the account stays hidden, the track record is public, the feed is a key. Show the leaderboard: Alpha One, PnL 8 USD, drawdown 10 bps, Relayed badge.
2. Dry quote. Open Alpha One. Click **Buy key → Pay from another chain**. Show `dry: true`, an `amountOut`, and `depositAddress: null`.
3. Locked feed. Open Feed. Click **Open feed**. Show `403 no viewing key`.
4. Unlock. Click **Mint demo key**. Show `Trades are streaming.` and the ETH lines.
5. Verify. Open Verify. Click **Check record**. Show `VERIFIED` and digest `c77fa93b9c667bfefa078b647e55333a42266d4f0b2dc000a45500e0b52881d7`. Click **Flip one byte**. Show `NOT VERIFIED`.
6. Optional, only if it is already on screen: the copier dry-run line `coin_not_allowed:DOGE` from `make demo-seed`. Do not show a live order.

Say these limits out loud: the record is relayed from a fixture, the quote is dry, and the demo mint is in the local gateway.

Submit today, Thu Nov 12, as soon as the GitHub repo is public. Do not wait for Saturday. Near Hunt shows the project only after the form is sent.

## X post (Ilya posts this himself, then submits)

```
built at NEAR HACKATHON

Alpha Keys: a viewing-key NFT for a trader who never publishes the account. The PnL is on NEAR. The feed stays locked until you hold the key. Flip one byte of the record and it shows NOT VERIFIED.

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

`security` stays off. The published record is relayed. `agents` stays off. The copier dry-run does not spend.

Idea:

Alpha Keys is a viewing-key NFT for a trader who never publishes the account. The Hyperliquid address stays off the chain. A relayed PnL record stores profit, drawdown, and Sharpe from a public fixture, and anyone can recompute them with make demo-seed. The demo gateway returns 403 until a key is minted, then it streams delayed, size-bucketed trades. The verify page hashes the fixture line and shows NOT VERIFIED after one flipped byte. 1Click quotes are dry. Judges clone the repo and follow the README with no secrets and no funds.

Save the 201 body to `docs/submission-receipt.md` after the form accepts it. This file is the outline, not the receipt. The form has not been sent.

