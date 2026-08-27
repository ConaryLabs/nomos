import assert from "node:assert/strict";
import test from "node:test";

import { summarizeDurations } from "../smoke/smoke.mjs";

test("timing summary uses the frozen even median and nearest-rank p95", () => {
  const ten = ["10", "1", "9", "2", "8", "3", "7", "4", "6", "5"];
  assert.deepEqual(summarizeDurations(ten), {
    median_denominator: 2,
    median_numerator_ns: "11",
    p95_ns: "10",
  });
  const twenty = Array.from({ length: 20 }, (_, index) => String(20 - index));
  assert.deepEqual(summarizeDurations(twenty), {
    median_denominator: 2,
    median_numerator_ns: "21",
    p95_ns: "19",
  });
});
