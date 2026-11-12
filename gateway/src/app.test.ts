import { generateKeyPairSync } from "node:crypto";
import { readFileSync } from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { describe, expect, it } from "vitest";
import { createApp, DEMO_LEADER } from "./app.ts";
import { signNep413 } from "./nep413.ts";
import { quoteBody } from "./oneclick.ts";
import type { TradeEvent } from "./types.ts";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const feed: TradeEvent[] = readFileSync(path.join(root, "fixtures/feed/alpha-1.jsonl"), "utf8")
  .trim()
  .split("\n")
  .map((line) => JSON.parse(line) as TradeEvent);

function app() {
  return createApp({
    demo: true,
    secret: "test-secret",
    feed,
    leader: DEMO_LEADER,
    copier: [{ type: "reject", decision: { reason: "coin_not_allowed:DOGE" } }],
    now: () => 1_700_000_000_000,
  });
}

describe("gateway", () => {
  it("keeps 1Click quotes dry unless ONECLICK_LIVE=1", () => {
    expect(quoteBody({ amount: "1" }, {})).toMatchObject({ dry: true });
    expect(quoteBody({ amount: "1" }, { ONECLICK_LIVE: "1" })).toMatchObject({ dry: false });
  });

  it("rejects a replayed nonce and a bad signature", async () => {
    const gateway = app();
    const { publicKey, privateKey } = generateKeyPairSync("ed25519");
    const raw = publicKey.export({ format: "der", type: "spki" }).subarray(-32);
    const challenge = await gateway.request("/auth/challenge", { method: "POST" });
    const { nonce } = (await challenge.json()) as { nonce: string };
    const message = "alpha-keys login demo.alpha-keys.testnet";
    const nonceBytes = Buffer.from(nonce, "base64url");
    const signature = signNep413(message, nonceBytes, "alpha-keys", privateKey).toString("base64url");
    const body = {
      accountId: "demo.alpha-keys.testnet",
      publicKey: raw.toString("hex"),
      signature,
      message,
      nonce,
      recipient: "alpha-keys",
    };
    const ok = await gateway.request("/auth/verify", { method: "POST", body: JSON.stringify(body) });
    expect(ok.status).toBe(200);
    const replay = await gateway.request("/auth/verify", { method: "POST", body: JSON.stringify(body) });
    expect(replay.status).toBe(401);
    const again = await gateway.request("/auth/challenge", { method: "POST" });
    const next = (await again.json()) as { nonce: string };
    const bad = await gateway.request("/auth/verify", {
      method: "POST",
      body: JSON.stringify({ ...body, nonce: next.nonce, signature: Buffer.alloc(64).toString("base64url") }),
    });
    expect(bad.status).toBe(401);
  });

  it("returns 401, then 403, then a delayed stream after a demo key", async () => {
    const gateway = app();
    const closed = await gateway.request("/feed/alpha-1");
    expect(closed.status).toBe(401);
    const session = await gateway.request("/demo/session", {
      method: "POST",
      body: JSON.stringify({ accountId: "demo.alpha-keys.testnet" }),
    });
    const { token } = (await session.json()) as { token: string };
    const headers = { authorization: `Bearer ${token}` };
    const denied = await gateway.request("/feed/alpha-1", { headers });
    expect(denied.status).toBe(403);
    expect(await denied.text()).toBe("no viewing key");
    const minted = await gateway.request("/demo/mint", { method: "POST", headers });
    expect(minted.status).toBe(200);
    const stream = await gateway.request("/feed/alpha-1", { headers });
    expect(stream.status).toBe(200);
    const text = await stream.text();
    expect(text).toContain("alpha-1");
    expect(text).toContain("ETH");
    const schema = JSON.parse(readFileSync(path.join(root, "docs/schemas/trade-event.json"), "utf8")) as {
      required: string[];
    };
    for (const key of schema.required) expect(feed[0]).toHaveProperty(key);
  });

  it("serves the dry-run copier transcript only in demo mode", async () => {
    const open = await app().request("/demo/copier");
    expect(open.status).toBe(200);
    const body = (await open.json()) as { decisions: Array<{ decision: { reason?: string } }> };
    expect(body.decisions[0].decision.reason).toBe("coin_not_allowed:DOGE");
    const closed = createApp({ demo: false, secret: "test-secret", feed, leader: DEMO_LEADER });
    expect((await closed.request("/demo/copier")).status).toBe(404);
  });
});
