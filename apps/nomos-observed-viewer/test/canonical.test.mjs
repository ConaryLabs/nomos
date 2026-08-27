import assert from "node:assert/strict";
import test from "node:test";

import { canonicalBytes, encodeCanonical, parseCanonical } from "../src/canonical.mjs";
import { capture, planBytes } from "./helpers.mjs";

test("committed canonical plan bytes round trip exactly", () => {
  const bytes = planBytes();
  assert.deepEqual(Buffer.from(canonicalBytes(parseCanonical(bytes))), bytes);
});

test("canonical encoding preserves integer and UTF-8 spellings", () => {
  assert.equal(encodeCanonical({ z: "café", a: -2n }), '{"a":-2,"z":"café"}');
});

test("parser distinguishes malformed, duplicate, and noncanonical bytes", () => {
  assert.equal(capture(() => parseCanonical(Uint8Array.of(0xff))).kind, "utf8");
  assert.equal(capture(() => parseCanonical("{" )).kind, "json");
  assert.equal(capture(() => parseCanonical('{"a":1,"a":2}')).kind, "duplicate");
  assert.equal(capture(() => parseCanonical(' {"a":1}')).kind, "canonical");
  assert.equal(capture(() => parseCanonical('{"a":1.0}')).kind, "canonical");
  assert.equal(capture(() => parseCanonical('{"a":18446744073709551616}')).kind, "json");
  assert.equal(capture(() => parseCanonical('{"a":-9223372036854775809}')).kind, "json");
  assert.equal(parseCanonical('{"a":18446744073709551615}').a, 18446744073709551615n);
});
