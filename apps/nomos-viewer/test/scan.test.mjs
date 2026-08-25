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
import {
  collectionDocument,
  hallPlan,
  publishedSemanticsBytes,
  yardPlan,
} from "./fixtures.mjs";

const app = dirname(fileURLToPath(new URL("../build.mjs", import.meta.url)));

const workspace = () => mkdtempSync(join(tmpdir(), "nomos-viewer-scan-"));

/// The published runtime a build stages beside the app.
///
/// Eight bytes: the WebAssembly magic number and the version word. The build
/// pins the binary by a digest it computed and the scan checks that magic
/// rather than reading the file as text, so a real module would prove nothing
/// more here — what the module does is `crates/nomos-play`'s own tests to make.
const WASM_HEADER = Buffer.from([0x00, 0x61, 0x73, 0x6d, 0x01, 0x00, 0x00, 0x00]);

/// Writes the published artifacts a build stages from.
function publish(root) {
  const from = join(root, "published");
  mkdirSync(from, { recursive: true });
  writeFileSync(join(from, "areas.json"), `${JSON.stringify(collectionDocument(), null, 2)}\n`);
  for (const [id, plan] of [
    ["test-hall", hallPlan()],
    ["test-yard", yardPlan()],
  ]) {
    mkdirSync(join(from, "areas", id, "world"), { recursive: true });
    writeFileSync(join(from, "areas", id, "rendering-plan.json"), `${JSON.stringify(plan)}\n`);
    // The executable semantics, under the name the compiler publishes it as.
    // The plan beside it carries the digest of exactly these bytes, which is
    // what lets the build refuse a pair that does not belong together.
    writeFileSync(join(from, "areas", id, "world", "simulation.json"), publishedSemanticsBytes(id));
  }
  return from;
}

const publishRuntime = (root) => {
  const path = join(root, "nomos_play.wasm");
  writeFileSync(path, WASM_HEADER);
  return path;
};

const build = (root) => {
  const from = publish(root);
  const out = join(root, "dist");
  const staged = stage({ from, out, wasm: publishRuntime(root), app });
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
    // The executable semantics, staged beside the plan that publishes its
    // digest. It is the one projection member the artifact carries, because it
    // is what makes the browser able to run a kernel transaction at all.
    "areas/test-hall.simulation.json",
    "areas/test-yard.json",
    "areas/test-yard.simulation.json",
    "index.html",
    "nomos_play.wasm",
    "src/catalog.mjs",
    "src/plan.mjs",
    "src/play.mjs",
    "src/render.mjs",
    "src/runtime.mjs",
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
  stage({ from, out, wasm: publishRuntime(root), app });
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
  assert.equal(report.files, 16);
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

test("the scan refuses a runtime that is not a WebAssembly module", (t) => {
  const root = workspace();
  t.after(() => rmSync(root, { recursive: true, force: true }));
  const { out } = build(root);
  // The binary is the one staged file the scan pins rather than reads, so what
  // is worth planting is a file that is not the module at all. An empty or
  // truncated stage fails here, where it is cheap to say so, rather than in the
  // browser as a failed instantiation.
  writeFileSync(join(out, "nomos_play.wasm"), "not a module\n");
  const error = refuses("binary", () => scanDist(out));
  assert.match(error.message, /not a WebAssembly module/);
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
  assert.throws(() => stage({ from, out: join(root, "dist"), wasm: publishRuntime(root), app }), /NV0102/);
});

test("staging refuses a plan whose bytes are not the ones the collection names", (t) => {
  const root = workspace();
  t.after(() => rmSync(root, { recursive: true, force: true }));
  const from = publish(root);
  // Re-serialized, not edited: the document still decodes and still carries the
  // same area, and its bytes are no longer the published ones. That is exactly
  // what the collection's SHA-256 is for.
  writeFileSync(
    join(from, "areas", "test-hall", "rendering-plan.json"),
    `${JSON.stringify(hallPlan(), null, 2)}\n`,
  );
  refuses("plan-digest", () => stage({ from, out: join(root, "dist"), wasm: publishRuntime(root), app }));
});

test("staging refuses semantics the plan did not publish", (t) => {
  const root = workspace();
  t.after(() => rmSync(root, { recursive: true, force: true }));
  const from = publish(root);
  // A plan and its simulation projection are published as a pair, and the plan
  // carries the digest that says which bytes are its half. The yard's semantics
  // are a well-formed projection and the wrong one, which is exactly the shape a
  // stale or mismatched build takes; an artifact that could not be played must
  // not reach the public directory.
  writeFileSync(
    join(from, "areas", "test-hall", "world", "simulation.json"),
    publishedSemanticsBytes("test-yard"),
  );
  refuses("semantics-digest", () =>
    stage({ from, out: join(root, "dist"), wasm: publishRuntime(root), app }),
  );
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
    () => stage({ from, out: join(root, "dist"), wasm: publishRuntime(root), app: tampered }),
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
