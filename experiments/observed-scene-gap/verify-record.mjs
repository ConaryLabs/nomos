#!/usr/bin/env node

import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import { readFileSync, statSync } from "node:fs";
import { dirname, join, relative, resolve } from "node:path";
import { fileURLToPath } from "node:url";
import { spawnSync } from "node:child_process";

const experimentDir = dirname(fileURLToPath(import.meta.url));
const repoRoot = resolve(experimentDir, "../..");
const record = JSON.parse(readFileSync(join(experimentDir, "record.json"), "utf8"));

function sha256(path) {
  return createHash("sha256").update(readFileSync(path)).digest("hex");
}

function git(...args) {
  const result = spawnSync("git", args, {
    cwd: repoRoot,
    encoding: "utf8",
    stdio: ["ignore", "pipe", "pipe"],
  });
  assert.equal(result.status, 0, `git ${args.join(" ")} failed: ${result.stderr}`);
  return result.stdout.trim();
}

assert.equal(record.schema, "nomos.experiment.observed_scene_gap_record@1");
assert.equal(record.issue, 189);
assert.equal(
  git("rev-parse", `${record.accepted_surface.commit}^{tree}`),
  record.accepted_surface.tree,
);
assert.equal(
  git("rev-parse", `${record.experiment_source.commit}^{tree}`),
  record.experiment_source.tree,
);

const changedAtSource = git(
  "diff",
  "--name-only",
  `${record.accepted_surface.commit}..${record.experiment_source.commit}`,
)
  .split("\n")
  .filter(Boolean);
assert.equal(changedAtSource.length > 0, true);
for (const path of changedAtSource) {
  assert.equal(
    path.startsWith(record.experiment_source.changed_root),
    true,
    `experiment source escapes quarantine: ${path}`,
  );
}

for (const schema of record.schemas) {
  assert.equal(
    sha256(join(repoRoot, schema.owner_path)),
    schema.owner_sha256,
    `${schema.identity} owner bytes changed`,
  );
}

const fixturePath = join(repoRoot, record.fixture.path);
assert.equal(statSync(fixturePath).size, record.fixture.bytes);
assert.equal(sha256(fixturePath), record.fixture.sha256);
assert.equal(record.fixture.adopter_payload, false);
assert.equal(record.fixture.raw_transform_or_final_pixels, false);
assert.equal(record.fixture.gameplay_rules, false);
assert.equal(record.fixture.facts_already_resolved, true);

const resultPath = join(repoRoot, record.proof.result_path);
assert.equal(statSync(resultPath).size, record.proof.result_bytes);
assert.equal(sha256(resultPath), record.proof.result_sha256);
const result = JSON.parse(readFileSync(resultPath, "utf8"));
assert.equal(result.status, "pass");
assert.equal(result.fixture_sha256, record.fixture.sha256);
assert.equal(result.positive_control.status, "completed");
assert.equal(
  result.positive_control.plan_sha256,
  record.proof.positive_control.plan_sha256,
);

for (const diagnostic of record.proof.diagnostics) {
  const expectedPath = join(experimentDir, "expected", `${diagnostic.probe}.json`);
  assert.equal(sha256(expectedPath), diagnostic.expected_sha256);
  const expected = JSON.parse(readFileSync(expectedPath, "utf8"));
  assert.equal(expected.code, diagnostic.code);
  assert.equal(expected.message, diagnostic.message);
  assert.equal(expected.status, diagnostic.status);
  assert.deepEqual(result.probes[diagnostic.probe].diagnostic, expected);
  assert.equal(
    result.probes[diagnostic.probe].diagnostic_sha256,
    diagnostic.expected_sha256,
  );
}

assert.equal(record.proof.outcome, "pass");
assert.equal(record.classification.already_representable_honestly, false);
assert.equal(record.classification.adopter_owned_mapping_only, false);
assert.equal(record.classification.reusable_nomos_capability, "candidate");
assert.equal(record.classification.presenter_implementation_exists, false);
assert.equal(record.classification.presenter_resolves_gameplay_fact, false);

process.stdout.write(
  `${JSON.stringify({
    command: relative(repoRoot, fileURLToPath(import.meta.url)),
    diagnostics: record.proof.diagnostics.length,
    fixture_sha256: record.fixture.sha256,
    status: "pass",
  })}\n`,
);
