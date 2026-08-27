import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import { lstatSync, readFileSync, readdirSync } from "node:fs";
import { join, relative } from "node:path";
import test from "node:test";

import { app } from "./helpers.mjs";

const sha256 = (bytes) => createHash("sha256").update(bytes).digest("hex");
const files = (directory) => readdirSync(directory, { withFileTypes: true }).flatMap((entry) => {
  if (entry.name === "dist") return [];
  const path = join(directory, entry.name);
  const info = lstatSync(path);
  assert.equal(info.isSymbolicLink(), false, path);
  if (info.isDirectory()) return files(path);
  assert.equal(info.isFile(), true, path);
  return [relative(app, path).split("\\").join("/")];
});

test("SOURCE_MANIFEST is a closed digest allowlist outside generated dist", () => {
  const rows = readFileSync(join(app, "SOURCE_MANIFEST"), "utf8").trimEnd().split("\n");
  const recorded = rows.map((row) => {
    const match = /^([0-9a-f]{64})  (.+)$/.exec(row);
    assert.ok(match, row);
    return { digest: match[1], path: match[2] };
  });
  const expected = files(app).filter((path) => path !== "SOURCE_MANIFEST").sort();
  assert.deepEqual(recorded.map((row) => row.path), expected);
  for (const row of recorded) assert.equal(sha256(readFileSync(join(app, row.path))), row.digest, row.path);
});

test("PUBLIC_FILES is an exact sorted source-manifest subset", () => {
  const source = new Set(readFileSync(join(app, "SOURCE_MANIFEST"), "utf8").match(/  (.+)$/gm).map((row) => row.slice(2)));
  const publicFiles = readFileSync(join(app, "PUBLIC_FILES"), "utf8").trimEnd().split("\n");
  assert.deepEqual(publicFiles, [...publicFiles].sort());
  assert.equal(new Set(publicFiles).size, publicFiles.length);
  for (const path of publicFiles) {
    assert.ok(source.has(path), path);
    assert.match(path, /\.(?:html|css|mjs)$/);
  }
});

test("the app has no package manager or unmanifested dependency surface", () => {
  const names = files(app);
  for (const forbidden of ["package.json", "package-lock.json", ".npmrc", "npm-shrinkwrap.json"]) {
    assert.equal(names.includes(forbidden), false);
  }
});
