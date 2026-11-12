export type LatestView = {
  window: string;
  pnlUsdE6: number;
  maxDdBps: number;
  sharpeE4: number;
  nTrades: number;
  equityUsdE6: number;
  codeHash: string;
  inputHash: string;
  source: string;
};

export function isOutLayerCallback(source: unknown): boolean {
  return typeof source === "object" && source !== null && "OutLayerCallback" in source;
}

/** Attested is only an OutLayer callback whose code hash is the pinned WASM digest. */
export function attestationBadge(source: unknown, codeHash: string, pinnedHash: string): "Attested" | "Relayed" {
  if (pinnedHash.length === 0 || codeHash !== pinnedHash) return "Relayed";
  if (isOutLayerCallback(source) || source === "callback") return "Attested";
  return "Relayed";
}

function field(record: Record<string, unknown>, snake: string, camel: string): unknown {
  return record[snake] ?? record[camel];
}

function asNumber(value: unknown): number | null {
  if (typeof value === "number" && Number.isFinite(value)) return value;
  if (typeof value === "string" && value.trim() !== "" && Number.isFinite(Number(value))) return Number(value);
  return null;
}

function asString(value: unknown): string {
  return typeof value === "string" ? value : "";
}

/** The last callback whose code hash equals the contract's current pin. */
export function attestedLatest(history: unknown, pinnedHash: string): LatestView | null {
  if (!Array.isArray(history) || pinnedHash.length === 0) return null;
  let found: LatestView | null = null;
  for (const row of history) {
    if (typeof row !== "object" || row === null) continue;
    const record = row as Record<string, unknown>;
    const codeHash = asString(field(record, "code_hash", "codeHash"));
    if (attestationBadge(record.source, codeHash, pinnedHash) !== "Attested") continue;
    const pnlUsdE6 = asNumber(field(record, "pnl_usd_e6", "pnlUsdE6"));
    const maxDdBps = asNumber(field(record, "max_dd_bps", "maxDdBps"));
    const sharpeE4 = asNumber(field(record, "sharpe_e4", "sharpeE4"));
    const nTrades = asNumber(field(record, "n_trades", "nTrades"));
    const equityUsdE6 = asNumber(field(record, "equity_usd_e6", "equityUsdE6"));
    if (pnlUsdE6 === null || maxDdBps === null || sharpeE4 === null || nTrades === null || equityUsdE6 === null) {
      continue;
    }
    found = {
      window: asString(record.window),
      pnlUsdE6,
      maxDdBps,
      sharpeE4,
      nTrades,
      equityUsdE6,
      codeHash,
      inputHash: asString(field(record, "input_hash", "inputHash")),
      source: "callback",
    };
  }
  return found;
}

export function pinnedHashFromConfig(config: unknown): string {
  if (!Array.isArray(config) || typeof config[2] !== "string") return "";
  return config[2];
}
