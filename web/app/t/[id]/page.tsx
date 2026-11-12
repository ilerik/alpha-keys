"use client";

import { useEffect, useState } from "react";
import { DEMO_TRADER, GATEWAY } from "../../../lib/demo";
import { originStable, requestDryQuote, type QuoteView } from "../../../lib/oneclick";

interface DecisionLine {
  type: string;
  decision: { reason?: string; coin?: string; side?: string; size?: number };
}

export default function TraderPage() {
  const trader = DEMO_TRADER;
  const [quote, setQuote] = useState<QuoteView | null>(null);
  const [busy, setBusy] = useState(false);
  const [decisions, setDecisions] = useState<DecisionLine[]>([]);

  useEffect(() => {
    fetch(`${GATEWAY}/demo/copier`)
      .then((response) => (response.ok ? response.json() : Promise.reject(response.status)))
      .then((body: { decisions: DecisionLine[] }) => setDecisions(body.decisions))
      .catch(() => setDecisions([]));
  }, []);

  async function dryQuote() {
    setBusy(true);
    try {
      const origin = await originStable(fetch);
      setQuote(await requestDryQuote(fetch, origin));
    } catch (error) {
      setQuote({
        dry: true,
        amountIn: "5.0",
        amountOut: null,
        depositAddress: null,
        error: error instanceof Error ? error.message : "quote failed",
      });
    } finally {
      setBusy(false);
    }
  }

  return (
    <>
      <h1 className="manifesto">{trader.displayName}</h1>
      <p>Live tier is 1 NEAR. Delayed tier is 0.5 NEAR. Royalty {trader.royaltyBps / 100}% to the trader.</p>
      <section className="card">
        <h2>The key is an NFT</h2>
        <p>
          The delayed tier waits 60 seconds. Token <code>0</code> on <code>alpha-keys.testnet</code> is a NEP-171 key.
          Its owner can transfer it with <code>nft_transfer</code>. A resale pays the trader 10% through <code>nft_payout</code>.
        </p>
        <p>
          <a href="https://testnet.nearblocks.io/nft-token/alpha-keys.testnet/0">Token 0 on the testnet explorer</a>
        </p>
      </section>
      <section className="card">
        <h2>Pay with testnet NEAR</h2>
        <p>
          Call <code>buy_key</code> on the contract with the tier attached. A testnet wallet can do that from the
          connect button. The judge path does not need a funded account.
        </p>
      </section>
      <section className="card">
        <h2>Pay from another chain</h2>
        <button type="button" onClick={dryQuote} disabled={busy}>
          {busy ? "Quoting" : "Buy key → Pay from another chain"}
        </button>
        {quote ? (
          <pre>
            {JSON.stringify(
              { dry: quote.dry, amountIn: quote.amountIn, amountOut: quote.amountOut, depositAddress: quote.depositAddress, error: quote.error },
              null,
              2,
            )}
          </pre>
        ) : null}
      </section>
      <section>
        <h2>Copier dry-run</h2>
        <div className="tape">
          {decisions.length === 0 ? "waiting for the transcript" : decisions.map((line, index) => (
            <div key={index}>
              {line.type === "reject"
                ? `REJECT  ${line.decision.reason}`
                : `ACCEPT  ${line.decision.side} ${line.decision.coin} ${line.decision.size}`}
            </div>
          ))}
        </div>
      </section>
    </>
  );
}
