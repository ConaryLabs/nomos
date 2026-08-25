// Stages `dist/` from published artifacts, then scans it and fails closed.
//
// `docs/review/nomos-viewer.md` section 6 is the design. Two things live here
// because they are one decision: what may enter the public artifact, and what
// the public artifact may contain. A build that stages something the scan
// refuses is a build that fails, not a directory someone publishes anyway.
//
// Usage:
//   node apps/nomos-viewer/build.mjs --from target/executable-gaol --out apps/nomos-viewer/dist

import { createHash } from "node:crypto";
import {
  copyFileSync,
  mkdirSync,
  readFileSync,
  readdirSync,
  rmSync,
  statSync,
  writeFileSync,
} from "node:fs";
import { dirname, join, relative, resolve } from "node:path";
import { fileURLToPath, pathToFileURL } from "node:url";

import { decodeCollection, decodePlan } from "./src/plan.mjs";

const here = dirname(fileURLToPath(import.meta.url));

export class BuildError extends Error {
  constructor(rule, message) {
    super(`${rule}: ${message}`);
    this.name = "BuildError";
    this.rule = rule;
  }
}

const APP_MODULES = ["plan.mjs", "catalog.mjs", "play.mjs", "render.mjs", "ui.mjs"];
const VENDOR_FILES = ["three/three.module.min.js", "three/three.core.min.js", "three/LICENSE"];

const sha256 = (bytes) => createHash("sha256").update(bytes).digest("hex");

// ---------------------------------------------------------------------------
// Staging
// ---------------------------------------------------------------------------

/// Stages `out` from the published artifacts in `from` plus the app and vendor
/// trees. Returns the staged file list with byte counts.
export function stage({ from, out, app = here }) {
  const source = resolve(from);
  const target = resolve(out);
  rmSync(target, { recursive: true, force: true });
  mkdirSync(join(target, "areas"), { recursive: true });
  mkdirSync(join(target, "src"), { recursive: true });
  mkdirSync(join(target, "vendor", "three"), { recursive: true });

  // The collection is decoded before anything is staged: an artifact the viewer
  // could not read must never reach the public directory.
  const collectionBytes = readFileSync(join(source, "areas.json"));
  const collection = decodeCollection(JSON.parse(collectionBytes.toString("utf8")), "areas.json");
  writeFileSync(join(target, "areas.json"), collectionBytes);

  for (const area of collection.areas) {
    const planBytes = readFileSync(join(source, "areas", area.id, "rendering-plan.json"));
    const plan = decodePlan(JSON.parse(planBytes.toString("utf8")), area.plan);
    if (plan.area.id !== area.id) {
      throw new BuildError("staging", `${area.plan} carries area \`${plan.area.id}\``);
    }
    writeFileSync(join(target, area.plan), planBytes);
  }

  copyFileSync(join(app, "index.html"), join(target, "index.html"));
  for (const name of APP_MODULES) {
    copyFileSync(join(app, "src", name), join(target, "src", name));
  }

  // The vendored files are re-checked against their manifest as they are
  // copied, so a tampered working tree cannot be published even once.
  const manifest = JSON.parse(readFileSync(join(app, "vendor", "MANIFEST.json"), "utf8"));
  const recorded = new Map(
    manifest.packages.flatMap((one) => one.files).map((one) => [one.path, one]),
  );
  for (const name of VENDOR_FILES) {
    const entry = recorded.get(name);
    if (!entry) throw new BuildError("vendor", `${name} is not recorded in vendor/MANIFEST.json`);
    const bytes = readFileSync(join(app, "vendor", name));
    if (bytes.length !== entry.bytes || sha256(bytes) !== entry.sha256) {
      throw new BuildError("vendor", `${name} does not match its recorded digest`);
    }
    writeFileSync(join(target, "vendor", name), bytes);
  }

  return listFiles(target).map((path) => ({
    path: relative(target, path),
    bytes: statSync(path).size,
  }));
}

const listFiles = (dir) =>
  readdirSync(dir, { withFileTypes: true }).flatMap((entry) => {
    const path = join(dir, entry.name);
    return entry.isDirectory() ? listFiles(path) : [path];
  });

// ---------------------------------------------------------------------------
// The scan
// ---------------------------------------------------------------------------

/// Removes `//` and block comments, so a rule about code is not a rule about
/// prose. Shared with the tests, which make the same distinction.
export function stripComments(text) {
  return text.replace(/\/\*[\s\S]*?\*\/|(^|[^:\\])\/\/[^\n]*/g, (match, prefix) =>
    prefix === undefined ? " " : prefix,
  );
}

const EXPECTED_FILES = [
  "areas.json",
  "index.html",
  "src/catalog.mjs",
  "src/plan.mjs",
  "src/play.mjs",
  "src/render.mjs",
  "src/ui.mjs",
  "vendor/three/LICENSE",
  "vendor/three/three.core.min.js",
  "vendor/three/three.module.min.js",
];

// A repo-relative citation of the source a claim came from, carried by
// `entities[].provenance[].source.path`. RUNTIME.md section 5 R1-4 forbids
// `.nomos` *source* in the artifact; a path is not source, and the design
// record's finding 1 records the owner ruling that this exact shape is the only
// permitted occurrence.
const PROVENANCE_PATH = /^experiments\/executable-gaol\/areas\/[a-z0-9-]+\/world\.nomos$/;

const FORBIDDEN_INPUTS = [
  "world-ir",
  "world_ir",
  "compiler-receipts",
  "compiler_receipts",
  "final-state.json",
  "command-log",
  "entity-catalog.json",
];

// The four projection member names. A plan records which members it was
// compiled against — `projection_digests[].file` — and that is metadata, not
// the member. Anywhere else, one of these names means a projection has been
// copied into the public artifact.
const PROJECTION_FILES = [
  "simulation.json",
  "navigation.json",
  "persistence.json",
  "diagnostics.json",
];

const SOURCE_MARKERS = [/^\s*schema\s+nomos\./m, /^\s*entity\s+[a-z_]+\s*\{/m, /^\s*catalog\s*\{/m];

const CREDENTIALS = [
  /-----BEGIN [A-Z ]*PRIVATE KEY-----/,
  /\bAKIA[0-9A-Z]{16}\b/,
  /\bghp_[A-Za-z0-9]{20,}/,
  /\bgithub_pat_[A-Za-z0-9_]{20,}/,
  /\bxox[baprs]-[A-Za-z0-9-]{10,}/,
  /\bAIza[0-9A-Za-z_-]{35}\b/,
  /\bBearer\s+[A-Za-z0-9._-]{16,}/,
  /\b(password|secret|api[_-]?key|access[_-]?token)\s*[:=]\s*["'][^"']{8,}["']/i,
];

const MACHINE_PATHS = [
  /(^|[^\w])\/home\//,
  /(^|[^\w])\/Users\//,
  /(^|[^\w])\/root\//,
  /(^|[^\w])\/work\//,
  /\/github\/workspace/,
  /(^|[^\w])\/runner\//,
  /(^|[^\w])\/private\/var\//,
  /(^|[^\w])\/tmp\//,
  /\b[A-Za-z]:\\\\?/,
];

const ABSOLUTE_SPECIFIER = /^(?:[a-z][a-z0-9+.-]*:|\/\/)/i;

// Every position in a module that names something to load.
const SPECIFIER_PATTERNS = [
  /\bfrom\s*["']([^"']+)["']/g,
  /\bimport\s*["']([^"']+)["']/g,
  /\bimport\s*\(\s*["']([^"']+)["']/g,
  /\bfetch\s*\(\s*["']([^"']+)["']/g,
  /\bnew\s+URL\s*\(\s*["']([^"']+)["']/g,
  /\bnew\s+Worker\s*\(\s*["']([^"']+)["']/g,
  /\bnew\s+EventSource\s*\(\s*["']([^"']+)["']/g,
  /\bnew\s+WebSocket\s*\(\s*["']([^"']+)["']/g,
  /\bimportScripts\s*\(\s*["']([^"']+)["']/g,
];

const HTML_ATTRIBUTES = /\b(?:src|href|action|poster|srcset|data|formaction)\s*=\s*["']([^"']*)["']/gi;
const CSS_URLS = /url\(\s*["']?([^"')]+)["']?\s*\)/gi;

// The one absolute reference the page is allowed: an empty data icon, which
// makes the browser issue no favicon request at all.
const ALLOWED_ABSOLUTE = new Set(["data:,"]);

const COLOUR_LITERALS = [/#[0-9a-fA-F]{3,8}\b/, /\b0x[0-9a-fA-F]{6}\b/, /\brgba?\(/, /\bhsla?\(/];

const refuse = (rule, message) => {
  throw new BuildError(rule, message);
};

const checkSpecifiers = (rule, where, code) => {
  for (const pattern of SPECIFIER_PATTERNS) {
    for (const match of code.matchAll(pattern)) {
      const specifier = match[1];
      if (ABSOLUTE_SPECIFIER.test(specifier) && !ALLOWED_ABSOLUTE.has(specifier)) {
        refuse(rule, `${where} loads \`${specifier}\`, which is not a relative path`);
      }
    }
  }
};

/// Scans a staged directory. Throws a `BuildError` naming the rule that refused.
export function scanDist(dir) {
  const root = resolve(dir);
  const files = listFiles(root)
    .map((path) => relative(root, path).split("\\").join("/"))
    .sort();

  // 8. Shape: exactly the staged layout, plus one plan per declared area.
  const collection = JSON.parse(readFileSync(join(root, "areas.json"), "utf8"));
  const expected = [
    ...EXPECTED_FILES,
    ...collection.areas.map((one) => one.plan),
  ].sort();
  if (files.join("\n") !== expected.join("\n")) {
    refuse(
      "shape",
      `the staged tree is not the declared layout\n  staged:   ${files.join(", ")}\n  expected: ${expected.join(", ")}`,
    );
  }

  const manifest = JSON.parse(
    readFileSync(join(here, "vendor", "MANIFEST.json"), "utf8"),
  );
  const vendorDigests = new Map(
    manifest.packages.flatMap((one) => one.files).map((one) => [`vendor/${one.path}`, one]),
  );

  for (const name of files) {
    const bytes = readFileSync(join(root, name));
    const text = bytes.toString("utf8");
    const vendored = vendorDigests.get(name);

    // 3. The vendored files are pinned by digest rather than read for content.
    if (vendored) {
      if (bytes.length !== vendored.bytes || sha256(bytes) !== vendored.sha256) {
        refuse("vendor", `${name} does not match its recorded digest`);
      }
      const specifiers = [...text.matchAll(/\bfrom\s*["']([^"']+)["']/g)].map((one) => one[1]);
      for (const specifier of specifiers) {
        if (ABSOLUTE_SPECIFIER.test(specifier)) {
          refuse("external-origin", `${name} imports \`${specifier}\``);
        }
      }
      continue;
    }

    const code = stripComments(text);

    // 1 and 2. External origins in a live position, and any absolute URL in the
    // app's own code once its comments are gone.
    if (name.endsWith(".mjs")) {
      checkSpecifiers("external-origin", name, code);
      const found = code.match(/https?:\/\/[^\s"'`)]+/);
      if (found) refuse("external-origin", `${name} carries the URL ${found[0]}`);
    }
    if (name.endsWith(".html")) {
      for (const match of text.matchAll(HTML_ATTRIBUTES)) {
        const value = match[1].trim();
        if (value && ABSOLUTE_SPECIFIER.test(value) && !ALLOWED_ABSOLUTE.has(value)) {
          refuse("external-origin", `${name} references \`${value}\``);
        }
      }
      for (const match of text.matchAll(CSS_URLS)) {
        const value = match[1].trim();
        if (ABSOLUTE_SPECIFIER.test(value) && !ALLOWED_ABSOLUTE.has(value)) {
          refuse("external-origin", `${name} loads \`${value}\` from a stylesheet`);
        }
      }
      for (const script of text.matchAll(/<script[^>]*>([\s\S]*?)<\/script>/gi)) {
        checkSpecifiers("external-origin", `${name} inline module`, stripComments(script[1]));
      }
      const found = stripComments(text.replace(/<!--[\s\S]*?-->/g, " ")).match(
        /https?:\/\/[^\s"'`)]+/,
      );
      if (found) refuse("external-origin", `${name} carries the URL ${found[0]}`);
    }

    // 4. Forbidden inputs, and `.nomos` outside its one permitted shape.
    for (const forbidden of FORBIDDEN_INPUTS) {
      if (text.includes(forbidden)) refuse("forbidden-input", `${name} carries \`${forbidden}\``);
    }
    for (const marker of SOURCE_MARKERS) {
      if (marker.test(text)) refuse("forbidden-input", `${name} carries .nomos source`);
    }
    for (const match of text.matchAll(/[^"'\s]*\.nomos/g)) {
      const occurrence = match[0];
      const before = text.slice(Math.max(0, match.index - 10), match.index);
      if (!/"path"\s*:\s*"$/.test(before) || !PROVENANCE_PATH.test(occurrence)) {
        refuse(
          "forbidden-input",
          `${name} carries \`${occurrence}\` outside a provenance source path`,
        );
      }
    }
    for (const projection of PROJECTION_FILES) {
      for (const match of text.matchAll(new RegExp(projection.replace(".", "\\."), "g"))) {
        const before = text.slice(Math.max(0, match.index - 10), match.index);
        if (!/"file"\s*:\s*"$/.test(before)) {
          refuse("forbidden-input", `${name} carries \`${projection}\` outside a recorded digest`);
        }
      }
    }

    // 5 and 6. Credentials, and paths from the machine that built this.
    for (const pattern of CREDENTIALS) {
      const found = text.match(pattern);
      if (found) refuse("credential", `${name} carries something shaped like a credential`);
    }
    for (const pattern of MACHINE_PATHS) {
      const found = text.match(pattern);
      if (found) refuse("build-path", `${name} carries the path ${found[0].trim()}`);
    }

    // 7. Colours live in the catalog, and nowhere else.
    if (name === "index.html" || (name.endsWith(".mjs") && !name.endsWith("catalog.mjs"))) {
      for (const pattern of COLOUR_LITERALS) {
        const found = code.match(pattern);
        if (found) refuse("colour-literal", `${name} carries the colour ${found[0]}`);
      }
    }
  }

  return { files: files.length, bytes: files.reduce((sum, name) => sum + statSync(join(root, name)).size, 0) };
}

// ---------------------------------------------------------------------------
// Entry
// ---------------------------------------------------------------------------

const parseArguments = (argv) => {
  const options = { from: "target/executable-gaol", out: join(here, "dist") };
  for (let index = 0; index < argv.length; index += 1) {
    if (argv[index] === "--from") options.from = argv[index + 1];
    else if (argv[index] === "--out") options.out = argv[index + 1];
    else if (argv[index].startsWith("--")) throw new BuildError("usage", `unknown option ${argv[index]}`);
  }
  return options;
};

// Node 22 has no `import.meta.main`, and CI pins Node 22.
if (process.argv[1] && pathToFileURL(process.argv[1]).href === import.meta.url) {
  const options = parseArguments(process.argv.slice(2));
  const staged = stage(options);
  const report = scanDist(options.out);
  const bytes = staged.reduce((sum, one) => sum + one.bytes, 0);
  process.stdout.write(
    `NOMOS_VIEWER_BUILD PASS files=${report.files} bytes=${bytes} out=${relative(process.cwd(), resolve(options.out)) || "."}\n`,
  );
  for (const one of staged.sort((left, right) => right.bytes - left.bytes)) {
    process.stdout.write(`  ${String(one.bytes).padStart(8)}  ${one.path}\n`);
  }
}
