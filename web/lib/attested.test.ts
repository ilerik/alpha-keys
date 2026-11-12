import { describe, expect, it } from "vitest";
import { attestationBadge, attestedLatest, pinnedHashFromConfig } from "./attested";

const PIN = "abc123";

const callback = {
  window: "7d",
  pnl_usd_e6: -4_200_000,
  max_dd_bps: 745,
  sharpe_e4: 0,
  n_trades: 57,
  equity_usd_e6: 318_000_000,
  code_hash: PIN,
  input_hash: "input",
  source: { OutLayerCallback: { request_id: null } },
};

const relayed = {
  ...callback,
  pnl_usd_e6: 8_000_000,
  code_hash: "relayed-demo",
  source: { OutLayerHttps: { attestation_ref: "relayed-demo", relayer: "alpha-keys.testnet" } },
};

describe("attestation badge", () => {
  it("shows Attested only for a callback whose hash matches the pin", () => {
    expect(attestationBadge(callback.source, PIN, PIN)).toBe("Attested");
    expect(attestationBadge(relayed.source, "relayed-demo", "relayed-demo")).toBe("Relayed");
    expect(attestationBadge(callback.source, PIN, "other")).toBe("Relayed");
    expect(attestationBadge("relayed", PIN, PIN)).toBe("Relayed");
    expect(attestationBadge("outlayer", PIN, PIN)).toBe("Relayed");
  });

  it("keeps the fixture when the callback hash does not match", () => {
    expect(attestedLatest([relayed, { ...callback, code_hash: "stale" }], PIN)).toBeNull();
    expect(pinnedHashFromConfig(["alpha-keys.testnet", "outlayer.testnet", PIN])).toBe(PIN);
  });

  it("uses the last matching callback and leaves earlier relayed rows in place", () => {
    const latest = attestedLatest([relayed, callback], PIN);
    expect(latest?.source).toBe("callback");
    expect(latest?.pnlUsdE6).toBe(-4_200_000);
    expect(latest?.codeHash).toBe(PIN);
    expect(latest?.nTrades).toBe(57);
  });
});
