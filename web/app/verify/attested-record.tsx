"use client";

import { useEffect, useState } from "react";
import { attestedLatest, pinnedHashFromConfig, type LatestView } from "../../lib/attested";
import { viewConfig, viewPnlHistory } from "../../lib/near";

/** Shows the newest OutLayer callback whose code hash equals the contract pin. Renders nothing until one exists. */
export function AttestedRecord() {
  const [state, setState] = useState<{ record: LatestView | null; pinnedHash: string }>({ record: null, pinnedHash: "" });

  useEffect(() => {
    const contractId = process.env.NEXT_PUBLIC_CONTRACT_ID;
    const rpcUrl = process.env.NEXT_PUBLIC_NEAR_RPC_URL;
    if (!contractId || !rpcUrl) return;
    Promise.all([
      viewPnlHistory(rpcUrl, contractId, "alpha-1").catch(() => null),
      viewConfig(rpcUrl, contractId).catch(() => null),
    ]).then(([history, config]) => {
      const pinnedHash = pinnedHashFromConfig(config);
      setState({ record: attestedLatest(history, pinnedHash), pinnedHash });
    });
  }, []);

  const { record, pinnedHash } = state;
  if (!record) return null;
  const contractId = process.env.NEXT_PUBLIC_CONTRACT_ID ?? "";
  return (
    <section>
      <h2>Attested record</h2>
      <p>
        This row is an <strong>OutLayer callback</strong>. The worker ran inside Intel TDX with the trader account as a sealed
        secret. Its code hash equals the hash pinned on <code>{contractId}</code>, so the contract accepted it.
      </p>
      <div className="stats">
        <div><span>window</span><strong>{record.window}</strong></div>
        <div><span>pnl_usd_e6</span><strong>{record.pnlUsdE6}</strong></div>
        <div><span>max_dd_bps</span><strong>{record.maxDdBps}</strong></div>
        <div><span>sharpe_e4</span><strong>{record.sharpeE4}</strong></div>
        <div><span>n_trades</span><strong>{record.nTrades}</strong></div>
      </div>
      <p>code hash <code>{record.codeHash}</code></p>
      <p>pinned hash <code>{pinnedHash}</code></p>
      <p>input hash <code>{record.inputHash}</code></p>
      <p>
        <a href="https://testnet.nearblocks.io/txns/ECyHsbAmWfuo19Hp8GFdUn5C2wNpxri97ABrHJzihapa">Callback transaction</a>
        {" · "}
        <a href={`https://testnet.nearblocks.io/address/${contractId}`}>Contract on the explorer</a>
      </p>
      <h2>Relayed fixture record</h2>
    </section>
  );
}
