import { createHmac, timingSafeEqual } from "node:crypto";

export interface Session {
  accountId: string;
  exp: number;
}

export function issueSession(accountId: string, secret: string, nowMs = Date.now(), ttlMs = 15 * 60 * 1000): string {
  const header = b64({ alg: "HS256", typ: "JWT" });
  const payload = b64({ sub: accountId, exp: nowMs + ttlMs });
  const body = `${header}.${payload}`;
  const sig = createHmac("sha256", secret).update(body).digest("base64url");
  return `${body}.${sig}`;
}

export function readSession(token: string, secret: string, nowMs = Date.now()): Session | null {
  const [header, payload, sig] = token.split(".");
  if (!header || !payload || !sig) return null;
  const body = `${header}.${payload}`;
  const expected = createHmac("sha256", secret).update(body).digest("base64url");
  const a = Buffer.from(sig);
  const b = Buffer.from(expected);
  if (a.length !== b.length || !timingSafeEqual(a, b)) return null;
  const parsed = JSON.parse(Buffer.from(payload, "base64url").toString()) as { sub?: string; exp?: number };
  if (!parsed.sub || typeof parsed.exp !== "number" || parsed.exp <= nowMs) return null;
  return { accountId: parsed.sub, exp: parsed.exp };
}

function b64(value: unknown): string {
  return Buffer.from(JSON.stringify(value)).toString("base64url");
}
