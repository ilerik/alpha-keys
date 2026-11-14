import { DEMO_TRADER } from "../../lib/demo";
import { AttestedRecord } from "./attested-record";
import { FlipByte } from "./flip";

export default function VerifyPage() {
  const record = DEMO_TRADER.latest;
  const contractId = process.env.NEXT_PUBLIC_CONTRACT_ID ?? "";
  const tx = process.env.NEXT_PUBLIC_PNL_TX ?? "";
  return (
    <>
      <h1 className="manifesto">Verify</h1>
      <AttestedRecord />
      <p>
        The fixture record below is <strong>relayed</strong>. The judge path stores the same numbers from the fixture through <code>submit_pnl</code>, so anyone can recompute them.
      </p>
      {contractId ? (
        <p>
          Contract <code>{contractId}</code>
          {tx ? (
            <>
              {" "}
              · <a href={`https://testnet.nearblocks.io/txns/${tx}`}>explorer transaction</a>
            </>
          ) : null}
        </p>
      ) : (
        <p className="limit">No contract id is set. Judges do not deploy.</p>
      )}
      <div className="stats">
        <div><span>window</span><strong>{record.window}</strong></div>
        <div><span>pnl_usd_e6</span><strong>{record.pnlUsdE6}</strong></div>
        <div><span>max_dd_bps</span><strong>{record.maxDdBps}</strong></div>
        <div><span>sharpe_e4</span><strong>{record.sharpeE4}</strong></div>
        <div><span>n_trades</span><strong>{record.nTrades}</strong></div>
        <div><span>equity_usd_e6</span><strong>{record.equityUsdE6}</strong></div>
      </div>
      <p>code hash <code>{record.codeHash}</code></p>
      <p>input hash <code>{record.inputHash}</code></p>
      <FlipByte />
      <h2>Reproduce the metrics</h2>
      <pre>make demo-seed</pre>
      <p>You see the same pnl, drawdown, and Sharpe printed as JSON, then the delayed events.</p>
      <h2>Reproduce a WASM hash</h2>
      <pre>./scripts/build-worker.sh</pre>
      <p>
        You see <code>wasm sha256 cb9c3130c6858cf4b460950281cff3468eabc0d79f1652fb05061c7fe41978dd</code>.
        That is the third field of <code>get_config</code>. The relayed fixture hash is the label <code>relayed-demo</code>, not a WASM digest.
      </p>
    </>
  );
}
