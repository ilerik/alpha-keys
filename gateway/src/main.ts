import { createServer } from "node:http";
import { readFileSync } from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { createApp, DEMO_LEADER } from "./app.ts";
import type { TradeEvent } from "./types.ts";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const feed: TradeEvent[] = readFileSync(path.join(root, "fixtures/feed/alpha-1.jsonl"), "utf8")
  .split("\n")
  .filter((line) => line.trim().length > 0)
  .map((line) => JSON.parse(line) as TradeEvent);
const copier = readFileSync(path.join(root, "fixtures/feed/transcript.jsonl"), "utf8")
  .split("\n")
  .filter((line) => line.trim().length > 0)
  .map((line) => JSON.parse(line) as unknown);

const secret = process.env.GATEWAY_JWT_SECRET ?? "demo-only-not-a-secret";
const demo = process.env.DEMO_FEED !== "0";
const port = Number(process.env.GATEWAY_PORT ?? 8787);
const app = createApp({ demo, secret, feed, leader: DEMO_LEADER, copier });

createServer(async (req, res) => {
  const chunks: Buffer[] = [];
  for await (const chunk of req) chunks.push(chunk as Buffer);
  const url = `http://127.0.0.1:${port}${req.url ?? "/"}`;
  const headers = new Headers();
  for (const [key, value] of Object.entries(req.headers)) {
    if (typeof value === "string") headers.set(key, value);
  }
  const response = await app.fetch(
    new Request(url, { method: req.method, headers, body: chunks.length ? Buffer.concat(chunks) : undefined }),
  );
  res.statusCode = response.status;
  response.headers.forEach((value, key) => res.setHeader(key, value));
  res.end(Buffer.from(await response.arrayBuffer()));
}).listen(port, () => {
  console.log(`gateway listening on ${port} demo=${demo}`);
});
