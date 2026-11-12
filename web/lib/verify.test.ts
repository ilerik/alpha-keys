import { describe, expect, it } from "vitest";
import { flipOneByte, PUBLISHED_DIGEST, recordDigest, RELAYED_CANONICAL, verificationVerdict } from "./verify";

const PUBLISHED = "c77fa93b9c667bfefa078b647e55333a42266d4f0b2dc000a45500e0b52881d7";

describe("relayed record verification", () => {
  it("recomputes the published digest of the fixture record", async () => {
    expect(RELAYED_CANONICAL).toBe("7d|8000000|10|108098|2|1008000000|relayed-demo|fixture");
    expect(PUBLISHED_DIGEST).toBe(PUBLISHED);
    expect(await recordDigest(RELAYED_CANONICAL)).toBe(PUBLISHED);
    expect(verificationVerdict(PUBLISHED, PUBLISHED)).toBe("VERIFIED");
  });

  it("prints NOT VERIFIED after one byte of the record changes", async () => {
    const flipped = flipOneByte(RELAYED_CANONICAL);
    expect(flipped).toBe("7d|8000000|10|108098|2|1008000000|relayed-demo|fixturd");
    const digest = await recordDigest(flipped);
    expect(digest).not.toBe(PUBLISHED);
    expect(verificationVerdict(digest, PUBLISHED)).toBe("NOT VERIFIED");
  });
});
