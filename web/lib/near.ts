import { JsonRpcProvider } from "near-api-js";

function providerFor(rpcUrl: string): JsonRpcProvider {
  return new JsonRpcProvider({ url: rpcUrl });
}

export async function viewTrader(rpcUrl: string, contractId: string, traderId: string): Promise<unknown> {
  return providerFor(rpcUrl).callFunction({
    contractId,
    method: "get_trader",
    args: { trader_id: traderId },
  });
}

export async function viewConfig(rpcUrl: string, contractId: string): Promise<unknown> {
  return providerFor(rpcUrl).callFunction({
    contractId,
    method: "get_config",
    args: {},
  });
}

export async function viewPnlHistory(rpcUrl: string, contractId: string, traderId: string): Promise<unknown> {
  return providerFor(rpcUrl).callFunction({
    contractId,
    method: "get_pnl_history",
    args: { trader_id: traderId, from: 0, limit: 200 },
  });
}
