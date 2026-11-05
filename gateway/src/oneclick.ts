// 1Click client stub (P-1). Base: https://1click.chaindefuser.com/v0 . dry:true by default.
export const ONECLICK_BASE = "https://1click.chaindefuser.com/v0";

export function isLive(env: Record<string, string | undefined> = process.env): boolean {
  return env.ONECLICK_LIVE === "1";
}

export function quoteBody(req: Record<string, unknown>, env = process.env): Record<string, unknown> {
  return { ...req, dry: !isLive(env) };
}
// TODO(P-1): fetch(`${ONECLICK_BASE}/quote`, { headers: { "X-API-Key": env.ONECLICK_JWT } ... }), /tokens, /status
