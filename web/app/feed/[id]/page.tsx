"use client";

import { useState } from "react";
import { GATEWAY } from "../../../lib/demo";
import { accessMessage } from "../../../lib/format";

const ACCOUNT = "demo.alpha-keys.testnet";

function tape(raw: string): string[] {
  return raw
    .split("\n")
    .map((line) => (line.startsWith("data:") ? line.slice(5).trim() : ""))
    .filter((line) => line.length > 0)
    .map((line) => {
      const event = JSON.parse(line) as { tsMs: number; side: string; coin: string; sizeBucket: string; kind: string };
      return `${event.tsMs}  ${event.side}  ${event.coin}  ${event.sizeBucket}  ${event.kind}`;
    });
}

export default function FeedPage() {
  const [status, setStatus] = useState<number | null>(null);
  const [lines, setLines] = useState<string[]>([]);
  const [token, setToken] = useState("");

  async function session(): Promise<string> {
    if (token) return token;
    const response = await fetch(`${GATEWAY}/demo/session`, {
      method: "POST",
      headers: { "content-type": "application/json" },
      body: JSON.stringify({ accountId: ACCOUNT }),
    });
    const body = (await response.json()) as { token: string };
    setToken(body.token);
    return body.token;
  }

  async function openFeed() {
    const bearer = await session();
    const response = await fetch(`${GATEWAY}/feed/alpha-1`, { headers: { authorization: `Bearer ${bearer}` } });
    setStatus(response.status);
    setLines(response.ok ? tape(await response.text()) : []);
  }

  async function mint() {
    const bearer = await session();
    await fetch(`${GATEWAY}/demo/mint`, { method: "POST", headers: { authorization: `Bearer ${bearer}` } });
    await openFeed();
  }

  return (
    <>
      <h1 className="manifesto">Feed</h1>
      <p>Demo account <code>{ACCOUNT}</code>. The stream is the delayed fixture. No Hyperliquid address is involved.</p>
      <div className="row">
        <button type="button" onClick={openFeed}>Open feed</button>
        <button className="ghost" type="button" onClick={mint}>Mint demo key</button>
      </div>
      {status !== null && status !== 200 ? <p className="refused">{accessMessage(status)}</p> : null}
      {status === 200 ? <p>{accessMessage(200)}</p> : null}
      {lines.length > 0 ? <div className="tape">{lines.map((line) => <div key={line}>{line}</div>)}</div> : null}
    </>
  );
}
