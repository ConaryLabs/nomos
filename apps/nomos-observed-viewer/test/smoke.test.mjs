import assert from "node:assert/strict";
import test from "node:test";

import { summarizeDurations, verifyLaunchEvidence } from "../smoke/smoke.mjs";

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

const expectedCounts = (offset) => ({
  actions: 2 + offset,
  actors: 5 + offset,
  controlled_markers: 2,
  hostile_outlines: 2,
  protection_rings: 2,
  terrain_cells: 12 + offset,
  terrain_layers: 3,
});

const rawFixture = () => {
  const port = 4321;
  const samplesPerScene = 2;
  const plans = [0, 1].map((index) => ({
    bytes: 100 + index,
    expected_counts: expectedCounts(index),
    path: `plans/scene_${index === 0 ? "one" : "two"}.json`,
    sha256: String(index + 1).repeat(64),
  }));
  const launches = Array.from({ length: plans.length * samplesPerScene }, (_, launchOrdinal) => {
    const sceneOrdinal = Math.floor(launchOrdinal / samplesPerScene);
    const sampleOrdinal = launchOrdinal % samplesPerScene;
    const profile = `/proof/tmp/nomos-observed-chrome-${launchOrdinal}`;
    return {
      browser_product: "HeadlessChrome/test",
      cache_disabled: true,
      chrome_flags: [
        "--host-resolver-rules=MAP * ~NOTFOUND, EXCLUDE localhost",
        `--user-data-dir=${profile}`,
      ],
      closure: { after_result_ms: 1, duration_ms: 1, exit_code: null, signal: "SIGTERM" },
      console_errors: [],
      elapsed_ns: String(100_000_000 + launchOrdinal),
      exceptions: [],
      frame: {
        consequence_counts: { ...plans[sceneOrdinal].expected_counts },
        plan_sha256: plans[sceneOrdinal].sha256,
        viewport: { height: 720, width: 1280 },
      },
      launch_ordinal: launchOrdinal,
      network_negative_control: "blocked",
      profile,
      requests: [
        `http://localhost:${port}/?scene=${sceneOrdinal}`,
        `http://localhost:${port}/${plans[sceneOrdinal].path}`,
      ],
      sample_ordinal: sampleOrdinal,
      scene_ordinal: sceneOrdinal,
      screenshot: sampleOrdinal === 0 ? `scene_${sceneOrdinal + 1}.png` : null,
      webgl2: true,
    };
  });
  return { launches, plans, port, samplesPerScene };
};

test("raw launch evidence binds every launch to its browser facts and closure", () => {
  assert.equal(verifyLaunchEvidence(rawFixture()), true);
});

test("raw launch evidence plants fail closed", async (t) => {
  const plants = [
    ["missing field", (value) => { delete value.launches[0].webgl2; }, /fields differ/],
    ["WebGL2 false", (value) => { value.launches[0].webgl2 = false; }, /did not prove WebGL2/],
    ["cache enabled", (value) => { value.launches[0].cache_disabled = false; }, /did not disable cache/],
    ["duplicate profile", (value) => { value.launches[1].profile = value.launches[0].profile; value.launches[1].chrome_flags[1] = value.launches[0].chrome_flags[1]; }, /reused a browser profile/],
    ["profile flag drift", (value) => { value.launches[0].chrome_flags[1] = "--user-data-dir=/elsewhere"; }, /profile flag differs/],
    ["plan drift", (value) => { value.launches[0].frame.plan_sha256 = "f".repeat(64); }, /plan digest differs/],
    ["consequence drift", (value) => { value.launches[0].frame.consequence_counts.actors += 1; }, /consequence counts differ/],
    ["missing navigation request", (value) => { value.launches[0].requests.shift(); }, /navigation request is absent/],
    ["external request", (value) => { value.launches[0].requests.push("https://example.invalid/"); }, /made an external request/],
    ["launch ordinal drift", (value) => { value.launches[0].launch_ordinal = 1; }, /ordinal differs/],
    ["closure overflow", (value) => { value.launches[0].closure.after_result_ms = 2_001; }, /closure exceeded 2000 ms/],
  ];
  for (const [name, mutate, expected] of plants) {
    await t.test(name, () => {
      const fixture = rawFixture();
      mutate(fixture);
      assert.throws(() => verifyLaunchEvidence(fixture), expected);
    });
  }
});
