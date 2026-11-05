# gateway/

The feed gateway (TypeScript, Hono, SSE). Followers authenticate with a NEP-413 signed message and get a short-lived session. The gateway then checks `has_access` on the contract and streams sanitized `TradeEvent`s from the OutLayer feed job. It returns **401** without a session and **403** without a viewing key. `src/minter.ts` mints keys after 1Click `SUCCESS`.

The gateway never sees the trader's HL address. Interfaces are in [../docs/ARCHITECTURE.md §4](../docs/ARCHITECTURE.md). Tasks are F-1, F-3 to F-5, and P-2.
