export const DEMO_TRADER = {
  id: "alpha-1",
  owner: "trader.testnet",
  displayName: "Alpha One",
  priceLiveYocto: "1000000000000000000000000",
  priceDelayedYocto: "500000000000000000000000",
  royaltyBps: 1000,
  maxKeys: 100,
  keysSold: 1,
  active: true,
  latest: {
    window: "7d",
    pnlUsdE6: 8_000_000,
    maxDdBps: 10,
    sharpeE4: 108_098,
    nTrades: 2,
    equityUsdE6: 1_008_000_000,
    codeHash: "relayed-demo",
    inputHash: "fixture",
    source: "relayed" as const,
  },
};

export const GATEWAY = process.env.NEXT_PUBLIC_GATEWAY_URL ?? "http://localhost:8787";
