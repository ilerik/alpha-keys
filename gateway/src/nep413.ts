import { createPublicKey, sign, verify, type KeyObject } from "node:crypto";

const TAG = 2_147_484_061;
const ED25519_SPKI_PREFIX = Buffer.from("302a300506032b6570032100", "hex");

export function encodeNep413(message: string, nonce: Uint8Array, recipient: string): Buffer {
  if (nonce.length !== 32) throw new Error("nonce must be 32 bytes");
  return Buffer.concat([
    u32(TAG),
    encodeString(message),
    Buffer.from(nonce),
    encodeString(recipient),
    Buffer.from([0]),
  ]);
}

export function signNep413(message: string, nonce: Uint8Array, recipient: string, privateKey: KeyObject): Buffer {
  return sign(null, encodeNep413(message, nonce, recipient), privateKey);
}

export function verifyNep413(
  message: string,
  nonce: Uint8Array,
  recipient: string,
  signature: Uint8Array,
  publicKey: Uint8Array,
): boolean {
  if (publicKey.length !== 32) return false;
  const key = createPublicKey({
    key: Buffer.concat([ED25519_SPKI_PREFIX, Buffer.from(publicKey)]),
    format: "der",
    type: "spki",
  });
  return verify(null, encodeNep413(message, nonce, recipient), key, signature);
}

function encodeString(value: string): Buffer {
  const bytes = Buffer.from(value, "utf8");
  return Buffer.concat([u32(bytes.length), bytes]);
}

function u32(value: number): Buffer {
  const out = Buffer.alloc(4);
  out.writeUInt32LE(value);
  return out;
}
