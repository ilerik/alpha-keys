"use client";

import Link from "next/link";
import { useEffect, useState } from "react";
import { DEMO_TRADER, GATEWAY } from "../lib/demo";
import { usdFromE6 } from "../lib/format";
import { viewTrader } from "../lib/near";

type Trader = typeof DEMO_TRADER;

const BEATS = [
  ["Sealed address", "The Hyperliquid account lives as a TEE secret. The chain never stores it."],
  ["Attested PnL", "A job inside the enclave publishes PnL, drawdown, and Sharpe. Anyone can check the numbers."],
  ["Viewing-key NFT", "A NEP-171 key unlocks the feed. Live is scarce. Delayed is sixty seconds. Resale pays the trader."],
  ["Capped copier", "An opt-in agent mirrors size into the follower's own account, inside limits the follower sets."],
];

export default function LeaderboardPage() {
  const [traders, setTraders] = useState<Trader[]>([DEMO_TRADER]);

  useEffect(() => {
    const contractId = process.env.NEXT_PUBLIC_CONTRACT_ID;
    const rpcUrl = process.env.NEXT_PUBLIC_NEAR_RPC_URL;
    const chain = contractId && rpcUrl ? viewTrader(rpcUrl, contractId, "alpha-1").catch(() => null) : Promise.resolve(null);
    Promise.all([
      fetch(`${GATEWAY}/demo/leaderboard`)
        .then((response) => (response.ok ? response.json() : Promise.reject(response.status)))
        .catch(() => null),
      chain,
    ]).then(([body]) => {
      if (body?.traders) setTraders(body.traders);
    });
  }, []);

  return (
    <>
      <h1 className="manifesto">The account stays hidden. The track record is public. The feed is a key.</h1>
      <p className="lede">
        Good traders get front-run when the account is visible. Followers get screenshots instead of a record.
        Alpha Keys sells access to the trades, not the address.
      </p>
      <div className="beats">
        {BEATS.map(([title, text]) => (
          <p className="beat" key={title}>
            <strong>{title}.</strong> {text}
          </p>
        ))}
      </div>
      <p className="limit">
        This demo record is relayed from a fixture, not a live enclave callback. A public order book can still be fingerprinted.
      </p>
      <h2>Leaderboard</h2>
      {traders.map((trader) => (
        <article className="card" key={trader.id}>
          <div className="row">
            <h2>{trader.displayName}</h2>
            <span className="badge">{trader.latest.source === "relayed" ? "Relayed" : "Attested"}</span>
          </div>
          <div className="stats">
            <div><span>PnL</span><strong>{usdFromE6(trader.latest.pnlUsdE6)} USD</strong></div>
            <div><span>drawdown</span><strong>{trader.latest.maxDdBps} bps</strong></div>
            <div><span>Sharpe</span><strong>{(trader.latest.sharpeE4 / 10_000).toFixed(2)}</strong></div>
            <div><span>trades</span><strong>{trader.latest.nTrades}</strong></div>
          </div>
          <p className="muted">code hash <code>{trader.latest.codeHash}</code></p>
          <Link href={`/t/${trader.id}`}>Open {trader.displayName}</Link>
        </article>
      ))}
    </>
  );
}
