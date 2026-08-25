// The vendored dependency is what RUNTIME.md section 4 says it is.
//
// Every number in vendor/MANIFEST.json is recomputed here from the working
// tree. The manifest is a claim; this file is the measurement.

import test from "node:test";
import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import { readFileSync } from "node:fs";

const manifestUrl = new URL("../vendor/MANIFEST.json", import.meta.url);
const manifest = JSON.parse(readFileSync(manifestUrl, "utf8"));
const files = manifest.packages.flatMap((one) => one.files);
const sha256 = (bytes) => createHash("sha256").update(bytes).digest("hex");
const read = (entry, encoding) =>
  readFileSync(new URL(`../vendor/${entry.path}`, import.meta.url), encoding);

test("the manifest records exactly the dependency RUNTIME.md section 4 does", () => {
  assert.equal(manifest.packages.length, 1);
  const [three] = manifest.packages;
  assert.equal(three.name, "three");
  assert.equal(three.version, "0.185.1");
  assert.equal(three.license, "MIT");
  assert.equal(three.entry, "three/three.module.min.js");
  assert.equal(
    three.tarball_integrity,
    "sha512-5aojFCXKwnjBRZvUnt3WFfEcvUJgkN5LlijRFN95hMy8WVkG4I0QNcJE+OuWvuJ0bOdStrbfXn0pkd6/QyiAlg==",
  );
});

test("vendor digests match the manifest", () => {
  for (const entry of files) {
    const bytes = read(entry);
    assert.equal(bytes.length, entry.bytes, `${entry.path} byte count`);
    assert.equal(sha256(bytes), entry.sha256, `${entry.path} sha256`);
  }
});

test("the recorded external-URL count is the measured one", () => {
  // The acceptance criterion is that a grep for an external origin over apps/
  // returns only license text and documentation comments. For the two minified
  // three.js files that is one paper citation in a GLSL comment and one XML
  // namespace identifier, and this is where a version bump that changes either
  // becomes visible.
  for (const entry of files) {
    const found = (read(entry, "utf8").match(/https?:\/\/[^\s"'`)\\]*/g) ?? []).map(
      (one) => one.replace(/[",]+$/, ""),
    );
    assert.equal(found.length, entry.external_url_occurrences, `${entry.path} external URL count`);
    assert.deepEqual([...new Set(found)].sort(), [...entry.external_urls].sort(), entry.path);
  }
});

test("the vendored modules import only their own siblings, by relative path", () => {
  // three@0.185.1's module build re-exports from ./three.core.min.js, so the
  // vendored tree has to carry both files. What must never appear is an
  // absolute specifier: the moment one does, the published page fetches an
  // origin the smoke lane cannot see.
  for (const entry of files.filter((one) => one.path.endsWith(".js"))) {
    const text = read(entry, "utf8");
    const specifiers = [
      ...new Set([...text.matchAll(/\bfrom\s*["']([^"']+)["']/g)].map((match) => match[1])),
    ];
    assert.deepEqual(specifiers.sort(), [...entry.relative_imports].sort(), `${entry.path} imports`);
    for (const specifier of specifiers) {
      assert.match(specifier, /^\.\//, `${entry.path} imports ${specifier}`);
      const sibling = files.find((one) => one.path.endsWith(specifier.slice(2)));
      assert.ok(sibling, `${entry.path} imports ${specifier}, which is not vendored`);
    }
    for (const pattern of [
      /\bimport\s*\(/,
      /\bfetch\s*\(\s*["'`]https?:/,
      /\bimportScripts\s*\(/,
      /\bnew\s+EventSource\s*\(/,
      /\bnew\s+WebSocket\s*\(/,
    ]) {
      assert.equal(pattern.test(text), false, `${entry.path} matches ${pattern}`);
    }
  }
});

test("the license is preserved verbatim", () => {
  const license = read({ path: "three/LICENSE" }, "utf8");
  assert.match(license, /^The MIT License/);
  assert.match(license, /three\.js authors/);
  assert.match(license, /WITHOUT WARRANTY OF ANY KIND/);
});
