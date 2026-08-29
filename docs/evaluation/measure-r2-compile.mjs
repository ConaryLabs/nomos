#!/usr/bin/env node

import { strict as assert } from "node:assert";
import { createHash } from "node:crypto";
import { readdirSync, readFileSync, statSync, writeFileSync } from "node:fs";
import { resolve, join } from "node:path";
import { spawnSync } from "node:child_process";
import { cpus, hostname, platform, release, arch } from "node:os";

const usage = "usage: node docs/evaluation/measure-r2-compile.mjs --binary <release-binary> --fixture <maximum-scene> --output <empty-directory>";
const arguments_ = process.argv.slice(2);
assert.equal(arguments_.length, 6, usage);
assert.equal(arguments_[0], "--binary", usage);
assert.equal(arguments_[2], "--fixture", usage);
assert.equal(arguments_[4], "--output", usage);

const binary = resolve(arguments_[1]);
const fixture = resolve(arguments_[3]);
const output = resolve(arguments_[5]);
assert(statSync(binary).isFile(), "release binary is not a regular file");
assert(statSync(fixture).isFile(), "maximum fixture is not a regular file");
assert(statSync(output).isDirectory(), "output is not a directory");
assert.deepEqual(readdirSync(output), [], "output directory is not empty");

const digest = (bytes) => createHash("sha256").update(bytes).digest("hex");
const run = (kind, ordinal, recorded) => {
  const destination = join(output, `${kind}-${String(ordinal).padStart(3, "0")}.json`);
  const started = process.hrtime.bigint();
  const result = spawnSync(
    binary,
    ["compile", "--input", fixture, "--out", destination],
    { encoding: null, stdio: "pipe" },
  );
  const elapsed = process.hrtime.bigint() - started;
  assert.equal(result.status, 0, `${kind} ${ordinal} exited ${result.status}`);
  assert.deepEqual(result.stdout, Buffer.alloc(0), `${kind} ${ordinal} wrote stdout`);
  assert.deepEqual(result.stderr, Buffer.alloc(0), `${kind} ${ordinal} wrote stderr`);
  const bytes = readFileSync(destination);
  return {
    digest: digest(bytes),
    elapsed: recorded ? elapsed : null,
    path: destination,
    size: bytes.length,
  };
};

const warmups = Array.from({ length: 10 }, (_, index) => run("warmup", index, false));
const samples = Array.from({ length: 100 }, (_, index) => run("sample", index, true));
const outputDigests = new Set([...warmups, ...samples].map((row) => `${row.size}:${row.digest}`));
assert.equal(outputDigests.size, 1, "maximum-scene outputs are not byte-identical");

const sorted = samples.map((row) => row.elapsed).sort((left, right) => (left < right ? -1 : left > right ? 1 : 0));
const medianNumerator = sorted[49] + sorted[50];
const p95 = sorted[Math.ceil(0.95 * sorted.length) - 1];

writeFileSync(
  join(output, "samples.tsv"),
  `ordinal\telapsed_ns\tbytes\tsha256\tpath\n${samples
    .map((row, index) => `${index}\t${row.elapsed}\t${row.size}\t${row.digest}\t${row.path}`)
    .join("\n")}\n`,
  { flag: "wx" },
);
writeFileSync(
  join(output, "summary.json"),
  `${JSON.stringify({
    architecture: arch(),
    binary,
    binary_sha256: digest(readFileSync(binary)),
    cpu_count: cpus().length,
    fixture,
    fixture_sha256: digest(readFileSync(fixture)),
    hostname: hostname(),
    measurement_role: "recorded_observation",
    median_denominator: 2,
    median_numerator_ns: medianNumerator.toString(),
    node: process.version,
    output_digest: [...outputDigests][0],
    p95_ns: p95.toString(),
    platform: platform(),
    release: release(),
    recorded_samples: 100,
    warmups: 10,
  })}\n`,
  { flag: "wx" },
);

console.log(`r2 compile latency: median ${medianNumerator}/2 ns; p95 ${p95} ns; RECORDED`);
