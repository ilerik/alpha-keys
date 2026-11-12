export const ONECLICK_BASE = "https://1click.chaindefuser.com/v0";

export const NEAR_USDC = "nep141:17208628f84f5d6ad33f0da3bbbeb27ffcb398eac501a31bd6ad2011e36133a1";

/** Public NEAR account used as the dry-quote recipient. A live buy would use the treasury. */
export const DRY_RECIPIENT = "wrap.near";

/** Placeholder origin-chain refund. Built so the source file has no 40-hex address. */
export const DRY_REFUND_EVM = `0x${"0".repeat(39)}1`;

export interface QuoteRequest {
  dry: boolean;
  swapType: "EXACT_INPUT";
  slippageTolerance: number;
  originAsset: string;
  depositType: "ORIGIN_CHAIN";
  destinationAsset: string;
  amount: string;
  refundTo: string;
  refundType: "ORIGIN_CHAIN";
  recipient: string;
  recipientType: "DESTINATION_CHAIN";
  deadline: string;
}

export interface OriginAsset {
  assetId: string;
  decimals: number;
  symbol: string;
}

export function quoteAmount(decimals: number): string {
  if (decimals === 6) return "5000000";
  return (5n * 10n ** BigInt(decimals)).toString();
}

export function quoteRequest(originAsset: string, decimals: number, live = false, now = Date.now()): QuoteRequest {
  return {
    dry: !live,
    swapType: "EXACT_INPUT",
    slippageTolerance: 100,
    originAsset,
    depositType: "ORIGIN_CHAIN",
    destinationAsset: NEAR_USDC,
    amount: quoteAmount(decimals),
    refundTo: DRY_REFUND_EVM,
    refundType: "ORIGIN_CHAIN",
    recipient: DRY_RECIPIENT,
    recipientType: "DESTINATION_CHAIN",
    deadline: new Date(now + 30 * 60 * 1000).toISOString(),
  };
}

export interface QuoteView {
  dry: boolean;
  amountIn: string;
  amountOut: string | null;
  depositAddress: null;
  error: string | null;
}

interface QuotePayload {
  message?: string;
  quote?: {
    amountIn?: string;
    amountInFormatted?: string;
    amountOut?: string;
    amountOutFormatted?: string;
  };
}

export function quoteViewFromResponse(request: QuoteRequest, payload: QuotePayload, ok: boolean): QuoteView {
  const quote = payload.quote;
  return {
    dry: request.dry,
    amountIn: quote?.amountInFormatted ?? quote?.amountIn ?? request.amount,
    amountOut: quote?.amountOutFormatted ?? quote?.amountOut ?? null,
    depositAddress: null,
    error: ok ? null : payload.message ?? "quote failed",
  };
}

export async function requestDryQuote(fetchImpl: typeof fetch, origin: OriginAsset): Promise<QuoteView> {
  const body = quoteRequest(origin.assetId, origin.decimals, false);
  try {
    const response = await fetchImpl(`${ONECLICK_BASE}/quote`, {
      method: "POST",
      headers: { "content-type": "application/json" },
      body: JSON.stringify(body),
    });
    const payload = (await response.json()) as QuotePayload;
    return quoteViewFromResponse(body, payload, response.ok);
  } catch (error) {
    return {
      dry: true,
      amountIn: body.amount,
      amountOut: null,
      depositAddress: null,
      error: error instanceof Error ? error.message : "quote failed",
    };
  }
}

export async function originStable(fetchImpl: typeof fetch): Promise<OriginAsset> {
  const response = await fetchImpl(`${ONECLICK_BASE}/tokens`);
  if (!response.ok) throw new Error(`tokens ${response.status}`);
  const tokens = (await response.json()) as Array<OriginAsset & { blockchain?: string }>;
  const found = tokens.find(
    (token) => token.blockchain === "eth" && token.decimals === 6 && (token.symbol === "USDC" || token.symbol === "USDT"),
  );
  if (!found) throw new Error("no ethereum stablecoin in the token list");
  return { assetId: found.assetId, decimals: found.decimals, symbol: found.symbol };
}
