# web/

The Next.js (App Router, TypeScript) frontend. Wallet login uses `@hot-labs/near-connect` (https://docs.near.org/tools/near-connect) and contract calls use `near-api-js` 7.x.

Pages: `/` leaderboard, `/t/[id]` trader page and dry quote, `/feed/[id]` 403 then stream, `/verify` with the flip-one-byte check.

Local run: `pnpm --filter web dev`, then open http://localhost:3000. You see Alpha One. The feed page needs `pnpm --filter gateway start` in another terminal.

## Vercel

Author-only. The dashboard project uses these settings:

- Root directory: `web`
- Framework: Next.js
- Node.js 22
- Package manager: pnpm. The lockfile is at the repo root, so include source files outside the root directory.

Environment variables are public values only:

- `NEXT_PUBLIC_NEAR_NETWORK` = `testnet`
- `NEXT_PUBLIC_NEAR_RPC_URL` = `https://rpc.testnet.fastnear.com`
- `NEXT_PUBLIC_CONTRACT_ID` = `alpha-keys.testnet`
- `NEXT_PUBLIC_PNL_TX` = `6uDLJJ5s9URhR79Gt6P97r9p54HPdgSEWWdgEuYpyekF`

Leave `NEXT_PUBLIC_GATEWAY_URL` unset. The feed unlock stays on localhost.
