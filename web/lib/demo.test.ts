import { describe, expect, it } from "vitest";
import { isLoopbackUrl } from "./demo";

describe("loopback gateway", () => {
  it("treats localhost and 127.0.0.1 as local", () => {
    expect(isLoopbackUrl("http://localhost:8787")).toBe(true);
    expect(isLoopbackUrl("http://127.0.0.1:8787/feed")).toBe(true);
    expect(isLoopbackUrl("http://[::1]:8787")).toBe(true);
  });

  it("leaves a public gateway alone", () => {
    expect(isLoopbackUrl("https://alpha-keys.vercel.app")).toBe(false);
    expect(isLoopbackUrl("https://rpc.testnet.fastnear.com")).toBe(false);
  });
});
