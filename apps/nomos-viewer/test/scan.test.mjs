// Staging, and the scan over what was staged.
//
// The scan is proved the way `xtask/src/planted.rs` proves the boundary
// checker: one planted violation per rule, each of which must be refused by the
// rule that should catch it.

import test from "node:test";
import assert from "node:assert/strict";
import { cpSync, mkdirSync, mkdtempSync, readFileSync, readdirSync, rmSync, statSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { dirname, join, relative } from "node:path";
import { fileURLToPath } from "node:url";

import { BuildError, scanDist, stage, stripComments } from "../build.mjs";
import { collectionDocument, hallPlan, yardPlan } from "./fixtures.mjs";

const app = dirname(fileURLToPath(new URL("../build.mjs", import.meta.url)));

const workspace = () => mkdtempSync(join(tmpdir(), "nomos-viewer-scan-"));

/// Writes the published artifacts a build stages from.
function publish(root) {
  const from = join(root, "published");
  mkdirSync(join(from, "areas", "test-hall"), { recursive: true });
  mkdirSync(join(from, "areas", "test-yard"), { recursive: true });
  writeFileSync(join(from, "areas.json"), `${JSON.stringify(collectionDocument(), null, 2)}\n`);
  writeFileSync(join(from, "areas", "test-hall", "rendering-plan.json"), `${JSON.stringify(hallPlan())}\n`);
  writeFileSync(join(from, "areas", "test-yard", "rendering-plan.json"), `${JSON.stringify(yardPlan())}\n`);
  return from;
}

const build = (root) => {
  const from = publish(root);
  const out = join(root, "dist");
  const staged = stage({ from, out, app });
  return { from, out, staged };
};

const listing = (dir) =>
  readdirSync(dir, { withFileTypes: true })
    .flatMap((entry) =>
      entry.isDirectory() ? listing(join(dir, entry.name)) : [join(dir, entry.name)],
    )
    .sort();

const refuses = (rule, run) => {
  let error = null;
  try {
    run();
  } catch (caught) {
    error = caught;
  }
  assert.ok(error instanceof BuildError, `expected a BuildError, got ${error}`);
  assert.equal(error.rule, rule, `expected ${rule}, got ${error.rule}: ${error.message}`);
  return error;
};

test("build stages only published artifacts", (t) => {
  const root = workspace();
  t.after(() => rmSync(root, { recursive: true, force: true }));
  const { out, staged } = build(root);
  const files = listing(out).map((path) => relative(out, path).split("\\").join("/"));
  assert.deepEqual(files, [
    "areas.json",
    "areas/test-hall.json",
    "areas/test-yard.json",
    "index.html",
    "src/catalog.mjs",
    "src/plan.mjs",
    "src/play.mjs",
    "src/render.mjs",
    "src/ui.mjs",
    "vendor/three/LICENSE",
    "vendor/three/three.core.min.js",
    "vendor/three/three.module.min.js",
  ]);
  assert.equal(staged.length, files.length);
  // The plans are copied, not re-serialized: the published bytes are what the
  // compiler emitted.
  assert.equal(
    readFileSync(join(out, "areas/test-hall.json"), "utf8"),
    `${JSON.stringify(hallPlan())}\n`,
  );
  // Nothing from the app's own tooling is staged.
  for (const absent of ["build.mjs", "smoke", "test", "README.md", "vendor/MANIFEST.json"]) {
    assert.equal(files.includes(absent), false, `${absent} was staged`);
  }
});

test("building twice is byte identical", (t) => {
  const root = workspace();
  t.after(() => rmSync(root, { recursive: true, force: true }));
  const { from, out } = build(root);
  const first = listing(out).map((path) => [relative(out, path), readFileSync(path)]);
  stage({ from, out, app });
  const second = listing(out).map((path) => [relative(out, path), readFileSync(path)]);
  assert.equal(first.length, second.length);
  for (let index = 0; index < first.length; index += 1) {
    assert.equal(first[index][0], second[index][0]);
    assert.ok(first[index][1].equals(second[index][1]), `${first[index][0]} differs`);
  }
});

test("the scan passes on a staged tree", (t) => {
  const root = workspace();
  t.after(() => rmSync(root, { recursive: true, force: true }));
  const { out } = build(root);
  const report = scanDist(out);
  assert.equal(report.files, 12);
  assert.ok(report.bytes > 700_000, "the vendored renderer is most of the artifact");
  // And the provenance path the plans legitimately carry survives it: this is
  // the one `.nomos` occurrence the design record's finding 1 permits.
  assert.match(
    readFileSync(join(out, "areas/test-hall.json"), "utf8"),
    /"path":"experiments\/executable-gaol\/areas\/test-hall\/world\.nomos"/,
  );
});

test("the scan refuses an external origin in a module", (t) => {
  const root = workspace();
  t.after(() => rmSync(root, { recursive: true, force: true }));
  const { out } = build(root);
  const path = join(out, "src/ui.mjs");
  writeFileSync(path, `import * as three from "https://cdn.jsdelivr.net/npm/three@0.185.1/build/three.module.min.js";\n${readFileSync(path, "utf8")}`);
  const error = refuses("external-origin", () => scanDist(out));
  assert.match(error.message, /src\/ui\.mjs/);
});

test("the scan refuses an external origin in the page", (t) => {
  const root = workspace();
  t.after(() => rmSync(root, { recursive: true, force: true }));
  const { out } = build(root);
  const path = join(out, "index.html");
  writeFileSync(
    path,
    readFileSync(path, "utf8").replace(
      "<title>",
      '<link rel="stylesheet" href="https://fonts.example.invalid/x.css"><title>',
    ),
  );
  refuses("external-origin", () => scanDist(out));
});

test("the scan refuses a fetch to an origin", (t) => {
  const root = workspace();
  t.after(() => rmSync(root, { recursive: true, force: true }));
  const { out } = build(root);
  const path = join(out, "src/plan.mjs");
  writeFileSync(path, `${readFileSync(path, "utf8")}\nexport const beacon = () => fetch("https://example.invalid/ping");\n`);
  refuses("external-origin", () => scanDist(out));
});

test("the scan refuses a vendored file that does not match its digest", (t) => {
  const root = workspace();
  t.after(() => rmSync(root, { recursive: true, force: true }));
  const { out } = build(root);
  const path = join(out, "vendor/three/three.module.min.js");
  writeFileSync(path, `${readFileSync(path, "utf8")}\n// one byte more\n`);
  refuses("vendor", () => scanDist(out));
});

test("the scan refuses .nomos outside a provenance path", (t) => {
  const root = workspace();
  t.after(() => rmSync(root, { recursive: true, force: true }));
  const { out } = build(root);
  const path = join(out, "index.html");
  writeFileSync(path, readFileSync(path, "utf8").replace("</body>", "<!-- built from world.nomos --></body>"));
  const error = refuses("forbidden-input", () => scanDist(out));
  assert.match(error.message, /world\.nomos/);
});

test("the scan refuses .nomos source content", (t) => {
  const root = workspace();
  t.after(() => rmSync(root, { recursive: true, force: true }));
  const { out } = build(root);
  writeFileSync(
    join(out, "index.html"),
    `${readFileSync(join(out, "index.html"), "utf8")}\nentity north_gate {\n  primitive: iron\n}\n`,
  );
  refuses("forbidden-input", () => scanDist(out));
});

test("the scan refuses a projection member outside a recorded digest", (t) => {
  const root = workspace();
  t.after(() => rmSync(root, { recursive: true, force: true }));
  const { out } = build(root);
  const path = join(out, "src/play.mjs");
  writeFileSync(path, `${readFileSync(path, "utf8")}\nexport const source = "navigation.json";\n`);
  const error = refuses("forbidden-input", () => scanDist(out));
  assert.match(error.message, /navigation\.json/);
});

test("the scan refuses World IR and receipts", (t) => {
  const root = workspace();
  t.after(() => rmSync(root, { recursive: true, force: true }));
  const { out } = build(root);
  const path = join(out, "src/play.mjs");
  writeFileSync(path, `${readFileSync(path, "utf8")}\nexport const ir = "world_ir";\n`);
  refuses("forbidden-input", () => scanDist(out));
});

test("the scan refuses something shaped like a credential", (t) => {
  const root = workspace();
  t.after(() => rmSync(root, { recursive: true, force: true }));
  const { out } = build(root);
  const path = join(out, "src/play.mjs");
  writeFileSync(path, `${readFileSync(path, "utf8")}\nconst token = "ghp_0123456789abcdefghijklmnopqrstuvwxyz";\n`);
  refuses("credential", () => scanDist(out));
});

test("the scan refuses a path from the build machine", (t) => {
  const root = workspace();
  t.after(() => rmSync(root, { recursive: true, force: true }));
  const { out } = build(root);
  const path = join(out, "src/play.mjs");
  writeFileSync(path, `${readFileSync(path, "utf8")}\nexport const built = "/home/runner/work/nomos";\n`);
  const error = refuses("build-path", () => scanDist(out));
  assert.match(error.message, /home/);
});

test("the scan refuses a colour outside the catalog", (t) => {
  const root = workspace();
  t.after(() => rmSync(root, { recursive: true, force: true }));
  const { out } = build(root);
  const path = join(out, "src/ui.mjs");
  writeFileSync(path, `${readFileSync(path, "utf8")}\nconst accent = "#8ee6e3";\n`);
  refuses("colour-literal", () => scanDist(out));
});

test("the scan refuses a file the layout does not declare", (t) => {
  const root = workspace();
  t.after(() => rmSync(root, { recursive: true, force: true }));
  const { out } = build(root);
  writeFileSync(join(out, "notes.txt"), "left behind\n");
  refuses("shape", () => scanDist(out));
});

test("the scan refuses a staged tree missing a declared plan", (t) => {
  const root = workspace();
  t.after(() => rmSync(root, { recursive: true, force: true }));
  const { out } = build(root);
  rmSync(join(out, "areas/test-yard.json"));
  refuses("shape", () => scanDist(out));
});

test("staging refuses an artifact the viewer could not read", (t) => {
  const root = workspace();
  t.after(() => rmSync(root, { recursive: true, force: true }));
  const from = publish(root);
  const broken = structuredClone(hallPlan());
  broken.schema = "nomos.rendering_plan@1";
  writeFileSync(join(from, "areas", "test-hall", "rendering-plan.json"), JSON.stringify(broken));
  assert.throws(() => stage({ from, out: join(root, "dist"), app }), /NV0102/);
});

test("staging refuses a vendored file that does not match its manifest", (t) => {
  const root = workspace();
  t.after(() => rmSync(root, { recursive: true, force: true }));
  const from = publish(root);
  const tampered = join(root, "app");
  cpSync(app, tampered, { recursive: true });
  const vendored = join(tampered, "vendor/three/three.core.min.js");
  writeFileSync(vendored, `${readFileSync(vendored, "utf8")}\n`);
  assert.throws(
    () => stage({ from, out: join(root, "dist"), app: tampered }),
    /does not match its recorded digest/,
  );
});

test("comment stripping keeps code and drops prose", () => {
  assert.equal(stripComments("const a = 1; // https://example.invalid\n").trim(), "const a = 1;");
  assert.equal(stripComments("/* https://example.invalid */const a = 1;"), " const a = 1;");
  assert.match(stripComments('const url = "./areas.json";'), /\.\/areas\.json/);
});

test("the report measures the public artifact", (t) => {
  const root = workspace();
  t.after(() => rmSync(root, { recursive: true, force: true }));
  const { out } = build(root);
  const report = scanDist(out);
  const measured = listing(out).reduce((sum, path) => sum + statSync(path).size, 0);
  assert.equal(report.bytes, measured);
});
