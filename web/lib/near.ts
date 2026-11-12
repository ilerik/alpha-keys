import { JsonRpcProvider } from "near-api-js";

export async function viewTrader(rpcUrl: string, contractId: string, traderId: string): Promise<unknown> {
  const provider = new JsonRpcProvider({ url: rpcUrl });
  return provider.callFunction({
    contractId,
    method: "get_trader",
    args: { trader_id: traderId },
  });
}
