# Decisions log

Times are written as UTC (MSK).

| When | Decision | Why |
|---|---|---|
| Nov 5 | OutLayer is the primary TEE; NEAR AI Cloud is used only for the optional trade explainer | OutLayer runs arbitrary WASM, has sealed secrets, and calls back into NEAR contracts; NEAR AI Cloud attests inference only |
| Nov 5 | 1Click is `dry: true` everywhere until the jurisdiction question is answered | 1Click ToS restricted-jurisdiction clause, no published list |
| Nov 8 19:00 UTC (22:00 MSK) | GATE-1 was still open on that date. The repo was stubs. | The gate could not be decided before the contract and job existed |
| Nov 11 05:30 UTC (08:30 MSK) | Live OutLayer callback is NO-GO for the submission build. The judge record is `submit_pnl` with source `OutLayerHttps`, labeled relayed. `request_pnl_update` stays in the contract. | No NEAR signer and no OutLayer payment key are available here, and a testnet transaction is not signed without the human. The job's HTTPS path returns an error instead of calling Hyperliquid. `execute` is tested with a fake HTTP body and a fixture flag. The pure libraries and the relayed record do not depend on the TEE |
| Nov 12 08:20 UTC (11:20 MSK) | The public demo host is Vercel. Web4 is out of this submission. `CONTRACT_ID` stays empty until `scripts/deploy-testnet.sh` succeeds. | The judge Quickstart does not need the deploy. The account `alpha-keys.testnet` exists, and its key is `ml-dsa-65`. |
| Nov 12 09:00 UTC (12:00 MSK) | Submit today, as soon as `origin/main` is public. Do not wait for Saturday night. | Near Hunt lists a project only after the form is sent. A Saturday-night entry gets almost no upvote time. The judge path already runs from the README. Vercel and a longer video can follow. The form tracks stay `privacy`, `defi`, `nfts`, `security`. `agents` stays off, matching the README. |
