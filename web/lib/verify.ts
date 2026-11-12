/** Canonical bytes of the relayed fixture record. A judge can hash this string with sha256sum. */
export const RELAYED_CANONICAL = "7d|8000000|10|108098|2|1008000000|relayed-demo|fixture";

/** sha256 of RELAYED_CANONICAL. Flipping one byte of the record must not produce this digest. */
export const PUBLISHED_DIGEST = "c77fa93b9c667bfefa078b647e55333a42266d4f0b2dc000a45500e0b52881d7";

export function flipOneByte(value: string): string {
  const bytes = new Uint8Array(new TextEncoder().encode(value));
  bytes[bytes.length - 1] ^= 1;
  return new TextDecoder().decode(bytes);
}

export async function recordDigest(value: string): Promise<string> {
  const digest = await crypto.subtle.digest("SHA-256", new TextEncoder().encode(value));
  return [...new Uint8Array(digest)].map((byte) => byte.toString(16).padStart(2, "0")).join("");
}

export function verificationVerdict(recomputed: string, published: string): "VERIFIED" | "NOT VERIFIED" {
  return recomputed === published ? "VERIFIED" : "NOT VERIFIED";
}
