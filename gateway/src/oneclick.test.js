// Placeholder test runnable with plain `node --test` until vitest is added in F-1.
import { test } from "node:test";
import assert from "node:assert/strict";

test("dry is default (contract of P-1)", () => {
  const live = process.env.ONECLICK_LIVE === "1";
  assert.equal(live, false, "tests must never run with ONECLICK_LIVE=1");
});
