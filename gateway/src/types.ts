// Shared feed types (F-1). Keep in sync with docs/ARCHITECTURE.md §4 and docs/schemas/.
export type Tier = "Live" | "Delayed60";

export interface TradeEvent {
  traderId: string;
  seq: number;
  tsMs: number;
  coin: string;
  side: "B" | "A";
  sizeBucket: string;
  px: string;
  reduceOnly: boolean;
  kind: "open" | "add" | "reduce" | "close";
}

export const TIER_DELAY_MS: Record<Tier, number> = { Live: 0, Delayed60: 60_000 };
