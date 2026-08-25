// Every external origin named anywhere under `apps/`, accounted for.
//
// RUNTIME.md section 5 R1-4: "the CDN import at
// `experiments/executable-gaol/src/webgl-renderer.mjs:1` appears nowhere in the
// accepted tree", and the artifact carries no external-origin script, style,
// module, or fetch target. Issue #148's acceptance spells that as a grep whose
// only hits are license text and documentation comments.
//
// This is that grep, made mechanical, in two tiers:
//
//  * what ships - `index.html` and `src/` - carries no URL at all, in code or
//    in prose;
//  * everywhere else under `apps/`, each occurrence must be one of four things
//    that are not an origin the app can reach: a loopback address, recorded
//    provenance in the vendor manifest, the smoke lane's declared
//    negative-control probe, or a violation a test plants in order to prove it
//    is refused.
//
// The vendored files are excluded here and covered by `vendor.test.mjs`, which
// counts their two documentation strings against `MANIFEST.json`.

import test from "node:test";
import assert from "node:assert/strict";
import { readFileSync, readdirSync } from "node:fs";
import { extname, join, relative } from "node:path";
import { fileURLToPath } from "node:url";

import { stripComments } from "../build.mjs";

const app = fileURLToPath(new URL("..", import.meta.url));
const URL_PATTERN = /https?:\/\/[^\s"'`,)\\]+/g;
const LOOPBACK = /^https?:\/\/(127\.0\.0\.1|localhost)(:|\/|$)/;
const PROBE = "https://example.invalid/nomos-viewer-probe";
const PLANTED = /^https:\/\/(cdn\.jsdelivr\.net|fonts\.example\.invalid|example\.invalid)(\/|$)/;

const walk = (dir) =>
  readdirSync(dir, { withFileTypes: true }).flatMap((entry) => {
    const path = join(dir, entry.name);
    if (entry.isDirectory()) return entry.name === "dist" ? [] : walk(path);
    return [path];
  });

const files = walk(app)
  .map((path) => relative(app, path).split("\\").join("/"))
  .filter((path) => !path.startsWith("vendor/three/"))
  .sort();

const read = (path) => readFileSync(join(app, path), "utf8");

test("nothing that ships names a URL", () => {
  const shipped = files.filter((path) => path === "index.html" || path.startsWith("src/"));
  assert.ok(shipped.length >= 6, "the shipped file set was not found");
  for (const path of shipped) {
    const found = read(path).match(URL_PATTERN);
    assert.equal(found, null, `${path} names ${found?.[0]}`);
  }
});

test("the CDN import appears nowhere under apps", () => {
  for (const path of files) {
    const text = read(path);
    for (const cdn of ["cdn.jsdelivr.net/npm/three", "unpkg.com/three"]) {
      // The one permitted mention is a test that plants the study's import in a
      // staged tree and requires the scan to refuse it.
      const planted = path.startsWith("test/") && /refuses an external origin/.test(text);
      assert.equal(
        text.includes(cdn) && !planted,
        false,
        `${path} carries the CDN import ${cdn}`,
      );
    }
  }
});

test("every external URL under apps is accounted for", () => {
  const unaccounted = [];
  for (const path of files) {
    const text = read(path);
    const code = extname(path) === ".json" ? text : stripComments(text);
    for (const match of text.match(URL_PATTERN) ?? []) {
      if (LOOPBACK.test(match)) continue;
      if (!code.includes(match)) continue; // prose, not code
      if (path === "vendor/MANIFEST.json") continue; // recorded provenance
      if (path === "smoke/smoke.mjs" && match === PROBE) continue; // the negative control
      if (path.startsWith("test/") && PLANTED.test(match)) continue; // a planted violation
      unaccounted.push(`${path}: ${match}`);
    }
  }
  assert.deepEqual(unaccounted, [], "an external URL under apps/ is not accounted for");
});

test("the negative control is a domain that cannot resolve", () => {
  // `.invalid` is reserved by RFC 2606 and resolves nowhere, so the probe fails
  // for a reason that is true with or without the host-resolver rule; what the
  // lane measures is that it fails *before* leaving the machine.
  assert.match(PROBE, /\.invalid\//);
  assert.ok(read("smoke/smoke.mjs").includes(`const PROBE = "${PROBE}"`));
});
