import { readFileSync } from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { describe, expect, it } from "vitest";
import { accessMessage, usdFromE6 } from "./format";
import { quoteRequest, quoteViewFromResponse, requestDryQuote, type QuoteRequest } from "./oneclick";

const recorded = JSON.parse(
  readFileSync(path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../../fixtures/quotes/dry-quote.json"), "utf8"),
) as { quote: { amountIn: string; amountOut: string; amountOutFormatted: string; depositAddress: string } };

describe("web quote", () => {
  it("builds a dry 5-unit quote into NEAR USDC", () => {
    const body = quoteRequest("nep141:origin", 6, false, Date.parse("2026-11-11T05:00:00.000Z"));
    expect(body.dry).toBe(true);
    expect(body.amount).toBe("5000000");
    expect(body.destinationAsset).toContain("nep141:");
    expect(body.recipient).toBe("wrap.near");
    expect(body.refundType).toBe("ORIGIN_CHAIN");
    expect(body.deadline).toBe("2026-11-11T05:30:00.000Z");
    expect(body.refundTo.startsWith("0x")).toBe(true);
    expect(body.refundTo.endsWith("1")).toBe(true);
    expect(body.refundTo.length).toBe(42);
  });

  it("reads amountOut from the recorded dry response and drops the deposit address", async () => {
    const seen: QuoteRequest[] = [];
    const view = await requestDryQuote(async (_url, init) => {
      seen.push(JSON.parse(String(init?.body)) as QuoteRequest);
      return new Response(JSON.stringify(recorded));
    }, { assetId: "nep141:origin", decimals: 6, symbol: "USDT" });
    expect(seen[0].dry).toBe(true);
    expect(view.amountIn).toBe("5.0");
    expect(view.amountOut).toBe("4.994155");
    expect(view.depositAddress).toBeNull();
    expect(view.error).toBeNull();
    const parsed = quoteViewFromResponse(seen[0], recorded, true);
    expect(parsed.amountOut).toBe(recorded.quote.amountOutFormatted);
    expect(accessMessage(403)).toBe("403 no viewing key");
    expect(usdFromE6(8_000_000)).toBe("8");
  });
});
