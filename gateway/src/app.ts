import { Hono } from "hono";
import { cors } from "hono/cors";
import { streamSSE } from "hono/streaming";
import { randomBytes } from "node:crypto";
import type { TradeEvent } from "./types.ts";
import { verifyNep413 } from "./nep413.ts";
import { issueSession, readSession } from "./session.ts";

export interface Leader {
  id: string;
  owner: string;
  displayName: string;
  priceLiveYocto: string;
  priceDelayedYocto: string;
  royaltyBps: number;
  maxKeys: number;
  keysSold: number;
  active: boolean;
  latest: {
    window: string;
    pnlUsdE6: number;
    maxDdBps: number;
    sharpeE4: number;
    nTrades: number;
    equityUsdE6: number;
    codeHash: string;
    inputHash: string;
    source: "relayed" | "outlayer";
  };
}

export interface AppOptions {
  demo: boolean;
  secret: string;
  feed: TradeEvent[];
  leader: Leader;
  copier?: unknown[];
  now?: () => number;
}

const RECIPIENT = "alpha-keys";

export function createApp(options: AppOptions) {
  const now = options.now ?? Date.now;
  const nonces = new Map<string, number>();
  const holders = new Set<string>();
  const app = new Hono();
  app.use("*", cors());

  app.get("/health", (c) => c.json({ ok: true, demo: options.demo }));

  app.get("/demo/copier", (c) => {
    if (!options.demo) return c.text("demo mode is off", 404);
    return c.json({ decisions: options.copier ?? [] });
  });

  app.get("/demo/leaderboard", (c) => {
    if (!options.demo) return c.text("demo mode is off", 404);
    return c.json({ traders: [options.leader] });
  });

  app.post("/demo/session", async (c) => {
    if (!options.demo) return c.text("demo mode is off", 404);
    const body = await c.req.json<{ accountId?: string }>();
    if (!body.accountId) return c.text("accountId", 400);
    return c.json({ token: issueSession(body.accountId, options.secret, now()) });
  });

  app.post("/demo/mint", async (c) => {
    if (!options.demo) return c.text("demo mode is off", 404);
    const session = sessionOf(c.req.header("authorization"), options.secret, now);
    if (!session) return c.text("unauthorized", 401);
    holders.add(session.accountId);
    return c.json({ tier: "Delayed60", accountId: session.accountId });
  });

  app.post("/auth/challenge", (c) => {
    const nonce = randomBytes(32).toString("base64url");
    nonces.set(nonce, now() + 5 * 60 * 1000);
    return c.json({ nonce, recipient: RECIPIENT });
  });

  app.post("/auth/verify", async (c) => {
    const body = await c.req.json<{
      accountId?: string;
      publicKey?: string;
      signature?: string;
      message?: string;
      nonce?: string;
      recipient?: string;
    }>();
    if (!body.accountId || !body.publicKey || !body.signature || !body.message || !body.nonce) {
      return c.text("missing field", 400);
    }
    if (body.recipient !== RECIPIENT) return c.text("recipient", 400);
    if (!body.message.includes(body.accountId)) return c.text("account", 400);
    const expiry = nonces.get(body.nonce);
    if (!expiry || expiry <= now()) return c.text("nonce", 401);
    nonces.delete(body.nonce);
    const ok = verifyNep413(
      body.message,
      Buffer.from(body.nonce, "base64url"),
      RECIPIENT,
      Buffer.from(body.signature, "base64url"),
      Buffer.from(body.publicKey, "hex"),
    );
    if (!ok) return c.text("signature", 401);
    return c.json({ token: issueSession(body.accountId, options.secret, now()) });
  });

  app.get("/feed/:traderId", (c) => {
    const session = sessionOf(c.req.header("authorization"), options.secret, now);
    if (!session) return c.text("unauthorized", 401);
    if (!holders.has(session.accountId)) return c.text("no viewing key", 403);
    const traderId = c.req.param("traderId");
    const events = options.feed.filter((event) => event.traderId === traderId);
    return streamSSE(c, async (stream) => {
      for (const event of events) {
        await stream.writeSSE({ data: JSON.stringify(event) });
      }
    });
  });

  return app;
}

function sessionOf(header: string | undefined, secret: string, now: () => number) {
  if (!header?.startsWith("Bearer ")) return null;
  return readSession(header.slice("Bearer ".length), secret, now());
}

export const DEMO_LEADER: Leader = {
  id: "alpha-1",
  owner: "trader.testnet",
  displayName: "Alpha One",
  priceLiveYocto: "1000000000000000000000000",
  priceDelayedYocto: "500000000000000000000000",
  royaltyBps: 1000,
  maxKeys: 100,
  keysSold: 1,
  active: true,
  latest: {
    window: "7d",
    pnlUsdE6: 8_000_000,
    maxDdBps: 10,
    sharpeE4: 108_098,
    nTrades: 2,
    equityUsdE6: 1_008_000_000,
    codeHash: "relayed-demo",
    inputHash: "fixture",
    source: "relayed",
  },
};
