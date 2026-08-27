import assert from "node:assert/strict";
import { existsSync, mkdirSync, readFileSync, writeFileSync } from "node:fs";
import { mkdtempSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import test from "node:test";

import { build, scanDistribution } from "../build.mjs";
import { parseIntegrity } from "../src/runtime.mjs";
import { root } from "./helpers.mjs";

const plans = () => ["scene_one.json", "scene_two.json"]
  .map((name) => join(root, "fixtures/r2/plans", name))
  .filter(existsSync);

const tree = (receipt) => receipt.files.map(({ path, bytes, sha256 }) => ({ path, bytes, sha256 }));

test("two clean builds are byte-identical and remain below the ceiling", () => {
  const temporary = mkdtempSync(join(tmpdir(), "nomos-observed-build-test-"));
  const first = build({ plans: plans(), out: join(temporary, "first"), receipt: join(temporary, "first.json") });
  const second = build({ plans: plans(), out: join(temporary, "second"), receipt: join(temporary, "second.json") });
  assert.deepEqual(tree(first), tree(second));
  assert.ok(first.total_bytes <= 2_000_000);
  const integrity = parseIntegrity(readFileSync(join(temporary, "first/ARTIFACTS.sha256"), "utf8"));
  assert.equal(integrity.plans.length, plans().length);
});

test("distribution scan refuses an extra file", () => {
  const temporary = mkdtempSync(join(tmpdir(), "nomos-observed-scan-test-"));
  const out = join(temporary, "dist");
  build({ plans: plans(), out, receipt: join(temporary, "receipt.json") });
  writeFileSync(join(out, "extra.js"), "export {};\n");
  assert.throws(() => scanDistribution(out, plans().map((path) => `plans/${path.split("/").at(-1)}`)), /distribution shape differs/);
});

test("build refuses to replace an output or receipt", () => {
  const temporary = mkdtempSync(join(tmpdir(), "nomos-observed-immutable-build-test-"));
  const output = join(temporary, "output");
  const receipt = join(temporary, "receipt.json");
  mkdirSync(output);
  assert.throws(() => build({ plans: plans(), out: output, receipt }), /output path must not exist/);
  writeFileSync(receipt, "preserve\n");
  assert.throws(() => build({ plans: plans(), out: join(temporary, "new-output"), receipt }), /receipt path must not exist/);
  assert.equal(readFileSync(receipt, "utf8"), "preserve\n");
});

test("integrity index errors have the exact non-schema envelope", () => {
  let failure;
  try { parseIntegrity("bad\n"); } catch (error) { failure = error; }
  assert.deepEqual(Object.keys(failure), ["artifact", "code", "message", "path"]);
  assert.equal(failure.code, "OV0101");
});
