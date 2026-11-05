# web/

The Next.js (App Router, TypeScript) frontend. Wallet login uses `@hot-labs/near-connect` (https://docs.near.org/tools/near-connect) and contract calls use `near-api-js` 7.x.

Pages: `/` leaderboard (Attested badge), `/t/[id]` trader page and buy, `/feed/[id]`, `/copier`, `/verify`. It deploys to Vercel against testnet. Tasks W-1 to W-6 and P-1/P-3.

Scaffold in W-1 with `pnpm create next-app web --ts --app` (keep `package.json` name `web`).
