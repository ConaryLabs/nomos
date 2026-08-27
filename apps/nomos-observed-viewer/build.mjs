import { createHash } from "node:crypto";
import {
  copyFileSync,
  existsSync,
  lstatSync,
  mkdirSync,
  readFileSync,
  readdirSync,
  statSync,
  writeFileSync,
} from "node:fs";
import { basename, dirname, join, relative, resolve } from "node:path";
import { fileURLToPath } from "node:url";

import { decodePlanBytes } from "./src/plan.mjs";

const app = dirname(fileURLToPath(import.meta.url));
const root = resolve(app, "../..");
const vendorRoot = join(root, "apps/nomos-viewer/vendor");
const vendorFiles = ["three/LICENSE", "three/three.core.min.js", "three/three.module.min.js"];

export class BuildFailure extends Error {
  constructor(rule, message) {
    super(`${rule}: ${message}`);
    this.name = "BuildFailure";
    this.rule = rule;
  }
}

const fail = (rule, message) => {
  throw new BuildFailure(rule, message);
};

export const sha256 = (bytes) => createHash("sha256").update(bytes).digest("hex");

const regularFiles = (directory) =>
  readdirSync(directory, { withFileTypes: true }).flatMap((entry) => {
    const path = join(directory, entry.name);
    if (entry.isSymbolicLink()) fail("file-type", `${path} is a symlink`);
    if (entry.isDirectory()) return regularFiles(path);
    if (!entry.isFile()) fail("file-type", `${path} is not regular`);
    return [path];
  });

const lines = (path) => {
  const text = readFileSync(path, "utf8");
  if (!text.endsWith("\n") || text.includes("\r") || text.slice(0, -1).includes("\n\n")) {
    fail("manifest", `${path} has noncanonical lines`);
  }
  return text.slice(0, -1).split("\n");
};

const safeRelative = (path) =>
  !path.startsWith("/") && !path.includes("\\") && !path.split("/").includes("..") && path !== "";

const readPublicFiles = () => {
  const paths = lines(join(app, "PUBLIC_FILES"));
  if ([...paths].sort().some((path, index) => path !== paths[index]) || new Set(paths).size !== paths.length) {
    fail("public-files", "PUBLIC_FILES is not sorted and unique");
  }
  for (const path of paths) {
    if (!safeRelative(path) || !/\.(?:html|css|mjs)$/.test(path)) fail("public-files", `unsafe public path ${path}`);
    const source = join(app, path);
    if (!lstatSync(source).isFile()) fail("public-files", `${path} is not a regular source file`);
  }
  return paths;
};

const vendorRecords = () => {
  const manifest = JSON.parse(readFileSync(join(vendorRoot, "MANIFEST.json"), "utf8"));
  return new Map(manifest.packages.flatMap((one) => one.files).map((one) => [one.path, one]));
};

const stageFile = (source, destination) => {
  mkdirSync(dirname(destination), { recursive: true });
  copyFileSync(source, destination);
  const before = readFileSync(source);
  const after = readFileSync(destination);
  if (before.length !== after.length || sha256(before) !== sha256(after)) fail("staging", `${source} did not copy exactly`);
};

export const scanDistribution = (distribution, expectedPlans) => {
  const target = resolve(distribution);
  if (!lstatSync(target).isDirectory()) fail("scan", "distribution is not a directory");
  const actual = regularFiles(target)
    .map((path) => relative(target, path).split("\\").join("/"))
    .sort();
  const expected = [
    "ARTIFACTS.sha256",
    ...readPublicFiles(),
    ...expectedPlans,
    ...vendorFiles.map((path) => `vendor/${path}`),
  ].sort();
  if (actual.length !== expected.length || actual.some((path, index) => path !== expected[index])) {
    fail("scan", `distribution shape differs: ${JSON.stringify(actual)}`);
  }
  const forbiddenNames = [
    "compiler-receipts",
    "world-ir",
    "nomos.observed_scene@1",
  ];
  for (const path of actual.filter((name) => !name.startsWith("vendor/"))) {
    const bytes = readFileSync(join(target, path));
    const text = bytes.toString("utf8").toLowerCase();
    for (const forbidden of forbiddenNames) {
      if (text.includes(forbidden)) fail("scan", `${path} contains forbidden token ${forbidden}`);
    }
    if (/\.nomos(?:["'\s]|$)/.test(text)) fail("scan", `${path} contains a source-file token`);
    if (/\b(?:https?|wss?):\/\//i.test(text)) fail("scan", `${path} contains an external origin`);
    if (/\/(?:home|data|work|tmp)\//.test(text)) fail("scan", `${path} contains an absolute build path`);
  }
  return Object.freeze({
    files: actual.map((path) => ({
      bytes: statSync(join(target, path)).size,
      path,
      sha256: sha256(readFileSync(join(target, path))),
    })),
    total_bytes: actual.reduce((sum, path) => sum + statSync(join(target, path)).size, 0),
  });
};

export const build = ({ plans, out, receipt }) => {
  if (!Array.isArray(plans) || plans.length < 1 || plans.length > 2) fail("arguments", "one or two plans are required");
  const names = plans.map((path) => basename(path));
  const allowed = ["scene_one.json", "scene_two.json"];
  if (names.some((name) => !allowed.includes(name)) || new Set(names).size !== names.length) {
    fail("arguments", "plan names must be unique scene_one.json or scene_two.json");
  }
  const ordered = plans.map((path) => resolve(path)).sort((a, b) => basename(a).localeCompare(basename(b)));
  const target = resolve(out);
  if (existsSync(target)) fail("arguments", "output path must not exist");
  if (receipt && existsSync(resolve(receipt))) fail("arguments", "receipt path must not exist");
  mkdirSync(target, { recursive: true });

  for (const path of readPublicFiles()) stageFile(join(app, path), join(target, path));
  const vendor = vendorRecords();
  for (const path of vendorFiles) {
    const record = vendor.get(path);
    const bytes = readFileSync(join(vendorRoot, path));
    if (!record || record.bytes !== bytes.length || record.sha256 !== sha256(bytes)) {
      fail("vendor", `${path} disagrees with the accepted R1 manifest`);
    }
    stageFile(join(vendorRoot, path), join(target, "vendor", path));
  }

  const planRows = ordered.map((path) => {
    const bytes = readFileSync(path);
    decodePlanBytes(bytes, `plans/${basename(path)}`);
    const relativePath = `plans/${basename(path)}`;
    mkdirSync(join(target, "plans"), { recursive: true });
    stageFile(path, join(target, relativePath));
    return { bytes: bytes.length, path: relativePath, sha256: sha256(bytes) };
  });
  const catalogDigest = sha256(readFileSync(join(app, "src/catalog.mjs")));
  const integrity = [
    `catalog_sha256\t${catalogDigest}`,
    ...planRows.map((row) => `${row.sha256}\t${row.bytes}\t${row.path}`),
  ].join("\n") + "\n";
  writeFileSync(join(target, "ARTIFACTS.sha256"), integrity);
  const report = scanDistribution(target, planRows.map((row) => row.path));
  if (report.total_bytes > 2_000_000) fail("budget", `distribution is ${report.total_bytes} bytes`);

  const result = {
    catalog_sha256: catalogDigest,
    files: report.files,
    generated_by: "apps/nomos-observed-viewer/build.mjs",
    node: process.version,
    outcome: "pass",
    plans: planRows,
    total_bytes: report.total_bytes,
  };
  if (receipt) {
    mkdirSync(dirname(resolve(receipt)), { recursive: true });
    writeFileSync(resolve(receipt), `${JSON.stringify(result, null, 2)}\n`);
  }
  return Object.freeze(result);
};

const parseArguments = (argv) => {
  const options = { plans: [] };
  for (let index = 0; index < argv.length; index += 1) {
    const flag = argv[index];
    const value = argv[index + 1];
    if (!value || !["--plan", "--out", "--receipt"].includes(flag)) fail("arguments", `invalid argument ${flag}`);
    if (flag === "--plan") options.plans.push(value);
    else options[flag.slice(2)] = value;
    index += 1;
  }
  if (!options.out || !options.receipt) fail("arguments", "--out and --receipt are required");
  return options;
};

if (process.argv[1] && resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  try {
    const result = build(parseArguments(process.argv.slice(2)));
    process.stdout.write(`NOMOS_OBSERVED_BUILD PASS files=${result.files.length} bytes=${result.total_bytes}\n`);
  } catch (error) {
    process.stderr.write(`${error.message}\n`);
    process.exitCode = 1;
  }
}
