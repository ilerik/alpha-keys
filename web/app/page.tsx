"use client";

import Link from "next/link";
import { useEffect, useState } from "react";
import { attestedLatest, attestationBadge, pinnedHashFromConfig, type LatestView } from "../lib/attested";
import { DEMO_TRADER, GATEWAY, isLoopbackUrl } from "../lib/demo";
import { usdFromE6 } from "../lib/format";
import { viewConfig, viewPnlHistory, viewTrader } from "../lib/near";
import { useLoopback } from "./loopback";

type Trader = Omit<typeof DEMO_TRADER, "latest"> & { latest: LatestView };

const BEATS = [
  ["Sealed address", "The Hyperliquid account lives as a TEE secret. The chain never stores it."],
  ["Attested PnL", "A job inside the enclave publishes PnL, drawdown, and Sharpe. Anyone can check the numbers."],
  ["Viewing-key NFT", "A NEP-171 key unlocks the feed. Live is scarce. Delayed is sixty seconds. Resale pays the trader."],
  ["Capped copier", "An opt-in agent mirrors size into the follower's own account, inside limits the follower sets."],
];

function withCallback(trader: Trader, callback: LatestView | null): Trader {
  if (!callback || trader.id !== "alpha-1") return trader;
  return { ...trader, latest: callback };
}

export default function LeaderboardPage() {
  const [board, setBoard] = useState<{ traders: Trader[]; pinnedHash: string }>({
    traders: [DEMO_TRADER],
    pinnedHash: "",
  });
  const { allowed } = useLoopback();

  useEffect(() => {
    const contractId = process.env.NEXT_PUBLIC_CONTRACT_ID;
    const rpcUrl = process.env.NEXT_PUBLIC_NEAR_RPC_URL;
    const chain =
      contractId && rpcUrl
        ? Promise.all([
            viewTrader(rpcUrl, contractId, "alpha-1").catch(() => null),
            viewPnlHistory(rpcUrl, contractId, "alpha-1").catch(() => null),
            viewConfig(rpcUrl, contractId).catch(() => null),
          ])
        : Promise.resolve([null, null, null] as const);
    const gateway = !isLoopbackUrl(GATEWAY) || allowed ? fetch(`${GATEWAY}/demo/leaderboard`) : Promise.resolve(null);
    Promise.all([
      gateway
        .then((response) => (response && response.ok ? response.json() : Promise.reject(response?.status)))
        .catch(() => null),
      chain,
    ]).then(([body, chainResult]) => {
      const [, history, config] = chainResult;
      const pinnedHash = pinnedHashFromConfig(config);
      const callback = attestedLatest(history, pinnedHash);
      const base: Trader[] = body?.traders ?? [DEMO_TRADER];
      setBoard({ traders: base.map((trader) => withCallback(trader, callback)), pinnedHash });
    });
  }, [allowed]);

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
        {board.traders.some((trader) => attestationBadge(trader.latest.source, trader.latest.codeHash, board.pinnedHash) === "Attested")
          ? "The Attested row is an OutLayer callback whose code hash matches the pinned WASM. The fixture row stays on chain. A public order book can still be fingerprinted."
          : "This demo record is relayed from a fixture, not a live enclave callback. A public order book can still be fingerprinted."}
      </p>
      <h2>Leaderboard</h2>
      {board.traders.map((trader) => (
        <article className="card" key={trader.id}>
          <div className="row">
            <h2>{trader.displayName}</h2>
            <span className="badge">{attestationBadge(trader.latest.source, trader.latest.codeHash, board.pinnedHash)}</span>
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
