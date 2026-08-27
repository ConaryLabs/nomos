#!/usr/bin/env node
import { createHash } from "node:crypto";
import { execFileSync } from "node:child_process";
import {
  existsSync,
  lstatSync,
  readFileSync,
  readdirSync,
  realpathSync,
  statSync,
  writeFileSync,
} from "node:fs";
import { dirname, join, relative, resolve, sep } from "node:path";
import { fileURLToPath } from "node:url";
import { auditLiveProcessNamespace } from "./r2-complete-proof-process.mjs";
const CONSTANTS = Object.freeze({
  issue: 199,
  issue_body_sha256: "8ffd30e7a213e991732ea6031743542eb68d9b80fe6d4989ed58052617352dcc",
  r2_contract_sha256: "770740bad1c85cf7ea9dcd16f8c25e01766064d3b59d7f0bb9d438c289a6e638",
  r2_revision_2_authority_sha256: "0356b3918a5c2643c36e16555e8ef78155bf893a8c3c21e4f75263f8289feea0",
  runtime_contract_sha256: "dd6f4b2ce48557f48df61d50cdc25b4ebaf0904331f4fd78d804e3af536db593",
  catalog_sha256: "6259520fbf318ae0393ea4ae69649864acb154db4034d081435416be2ffa9323",
  packet_manifest_sha256: "d5708087cf7967a420667c56a7b02ed052b7058ed8545af06e6771170003c948",
  committed_contact_sheet_sha256: "b76edbd9dd03fce5a99c074200ee7311bf87d5d2e5829c800170c129d00bf576",
  maximum_fixture_sha256: "fe332f711437dab15e4d1315cc3ca57dba6521350ff673941e77feb414585909",
  maximum_fixture_bytes: 98_421,
  plans: Object.freeze({
    scene_one: "717b91f3f35d815bfa9f9cc777b38f8a091f7a6339d786c57360e94ffe4c7699",
    scene_two: "1fd08cfb33d07f93a568e4bb337ebfbe8909a22a973d0f137139c92f0481e905",
  }),
  signatures: Object.freeze({
    scene_one: "ef11771f3f8c210fdd8c9366e780ab720349a49dad88ae8dca969fcbe16c30d2",
    scene_two: "9afb46dc4d7ddb5b79cdcabd63b67d162b3c230aecfde9383e038128572f0f3d",
  }),
  r1_chain_head: "43a1b2164f18bc54738d0402013419659576e2d866c3fca630321a2ca641f143",
  r1_wasm_bytes: 421_195,
  r1_wasm_sha256: "e8e03c125667ad937939f4a628b67df9ff813a88823fecd859784ed241673c97",
  r1_viewer_files: 24,
  r1_viewer_bytes: 1_386_650,
});
const COMMAND_IDS = Object.freeze([
  "workspace-fmt", "workspace-clippy", "workspace-test", "workspace-boundary",
  "r1-gaol-verify", "r1-wasm-build", "r1-native-build", "r1-viewer-mirror",
  "r1-viewer-build", "r1-viewer-tests", "r1-browser-smoke", "r1-native-replay",
  "r1-facts", "r2-schema-ownership", "r2-schema-plants", "r2-source-provenance",
  "r2-source-provenance-plants", "r2-adopter-neutrality", "r2-adopter-neutrality-plants",
  "r2-maximum-fixture", "r2-compiler-tests", "r2-scene-one-repro",
  "r2-scene-two-repro", "r2-scene-signatures", "r2-viewer-tests", "r2-viewer-build",
  "r2-browser-smoke", "clean-release-build", "clean-r1-viewer-build",
  "clean-r2-viewer-build-a", "clean-r2-viewer-build-b", "clean-r2-viewer-compare",
  "maximum-compile-benchmark",
]);

const COMMAND_DISPLAYS = Object.freeze([
  "cargo fmt --all -- --check",
  "cargo clippy --workspace --all-targets --locked --offline -- -D warnings",
  "cargo test --workspace --locked --offline",
  "cargo xtask boundary",
  "experiments/executable-gaol/gaol verify",
  "build R1 wasm, remove its exact target subtree, rebuild, and compare digests",
  "cargo build --locked --offline -p nomos-play",
  "git archive HEAD apps/nomos-viewer and byte-verify output-local mirror",
  "node apps/nomos-viewer/build.mjs in byte-identical output-local mirror",
  "node --test apps/nomos-viewer/test/*.test.mjs (byte-identical mirror)",
  "NOMOS_PLAY_BIN=target/debug/nomos-play NOMOS_PLAY_AREAS=target/executable-gaol/areas node apps/nomos-viewer/smoke/smoke.mjs --dist <output>/r1/viewer-dist --out <output>/r1/viewer-smoke --require-chrome",
  "target/debug/nomos-play replay target/executable-gaol/areas --session <output>/r1/viewer-smoke/session.json",
  "derive and assert accepted R1 facts",
  "docs/evaluation/r2-schema-ownership.sh",
  "three isolated git-archive schema-ownership plants must fail",
  "docs/evaluation/r2-source-provenance.sh",
  "docs/evaluation/r2-source-provenance.test.sh",
  "docs/evaluation/r2-adopter-neutrality.sh",
  "docs/evaluation/r2-adopter-neutrality.test.sh",
  "node docs/evaluation/r2-maximum.test.mjs",
  "compiler tests, release compiler, and exact frozen second-scene packet plants",
  "compile scene_one ten times to unique outputs and compare committed plan",
  "compile scene_two ten times to unique outputs and compare committed plan",
  "node docs/evaluation/r2-scene-signature.mjs scene_one scene_two",
  "node --test apps/nomos-observed-viewer/test/*.test.mjs docs/evaluation/r2-scene-signature.test.mjs docs/evaluation/r2-complete-proof-process.test.mjs docs/evaluation/r2-complete-proof-receipt.test.mjs; docs/evaluation/r2-complete-proof.test.sh",
  "node apps/nomos-observed-viewer/build.mjs --plan scene_one --plan scene_two --out <output>/r2/viewer-proof/dist --receipt <output>/r2/viewer-proof/receipt.json",
  "node apps/nomos-observed-viewer/smoke/smoke.mjs --dist <output>/r2/viewer-proof/dist --out <output>/r2/browser-smoke --samples 10",
  "LC_ALL=C /usr/bin/time -v cargo build --workspace --release --locked --offline (fresh target)",
  "clean R1 viewer build and byte comparison with proof distribution",
  "clean R2 viewer build A",
  "clean R2 viewer build B",
  "compare full regular-file inventories for all clean R2 distributions",
  "node docs/evaluation/measure-r2-compile.mjs --binary <fresh-release>/nomos-observed-scene --fixture maximum --output <output>/r2/compile-benchmark",
]);
const TOOL_LABELS = Object.freeze([
  "git", "realpath", "readlink", "find", "grep", "awk", "sed", "sort", "cmp", "cut",
  "sha256sum", "stat", "date", "du", "jq", "gnu-time", "ar", "basename", "bash", "bwrap",
  "cargo", "cc", "chmod", "cp", "diff", "dirname", "env", "getconf", "head", "id",
  "install", "ionice", "ip", "ld", "ln", "mkdir", "mktemp", "node", "paste", "ps", "rm", "rustc",
  "rustup", "seq", "setpriv", "sh", "sleep", "strings", "sudo", "tar", "timeout", "touch",
  "tr", "uname", "unshare", "wc", "cargo-toolchain", "rustc-toolchain", "rust-lld", "chrome",
]);

const HEX = /^[0-9a-f]{64}$/;
const COMMIT = /^[0-9a-f]{40}$/;
const sha256 = (bytes) => createHash("sha256").update(bytes).digest("hex");
const fail = (message) => { throw new Error(`r2 complete proof receipt: ${message}`); };
const required = (condition, message) => { if (!condition) fail(message); };

const exactKeys = (object, keys, label) => {
  required(object && typeof object === "object" && !Array.isArray(object), `${label} is not an object`);
  const actual = Object.keys(object).sort();
  const expected = [...keys].sort();
  required(JSON.stringify(actual) === JSON.stringify(expected), `${label} fields differ: ${actual.join(",")}`);
};

const readRegular = (path, label = path) => {
  required(existsSync(path), `${label} is missing`);
  const info = lstatSync(path);
  required(info.isFile() && !info.isSymbolicLink(), `${label} is not one regular file`);
  return readFileSync(path);
};

const readText = (path, label = path) => readRegular(path, label).toString("utf8");
const readJson = (path, label = path) => {
  let parsed;
  try { parsed = JSON.parse(readText(path, label)); } catch (error) { fail(`${label} is invalid JSON: ${error.message}`); }
  return parsed;
};

const safeRelative = (path) =>
  typeof path === "string" && path.length > 0 && path.length < 4096 &&
  !path.startsWith("/") && !path.includes("\\") && !path.includes("\0") &&
  !path.split("/").some((part) => part === "" || part === "." || part === "..");

const inside = (parent, child) => child === parent || child.startsWith(`${parent}${sep}`);

const noSymlinkComponents = (path, stop) => {
  let cursor = resolve(path);
  const boundary = resolve(stop);
  while (inside(boundary, cursor)) {
    required(!lstatSync(cursor).isSymbolicLink(), `symlinked path component: ${cursor}`);
    if (cursor === boundary) return;
    cursor = dirname(cursor);
  }
  fail(`${path} is outside ${stop}`);
};

const regularTree = (root) => {
  required(existsSync(root) && lstatSync(root).isDirectory() && !lstatSync(root).isSymbolicLink(), `${root} is not a real directory`);
  const rows = [];
  const visit = (directory) => {
    for (const entry of readdirSync(directory, { withFileTypes: true })) {
      const path = join(directory, entry.name);
      required(!entry.isSymbolicLink(), `evidence contains symlink ${path}`);
      if (entry.isDirectory()) visit(path);
      else {
        required(entry.isFile(), `evidence contains non-regular entry ${path}`);
        const rel = relative(root, path).split(sep).join("/");
        required(safeRelative(rel), `unsafe evidence path ${rel}`);
        const bytes = readFileSync(path);
        rows.push({ path: rel, bytes: bytes.length, sha256: sha256(bytes) });
      }
    }
  };
  visit(root);
  return rows.sort((a, b) => Buffer.from(a.path).compare(Buffer.from(b.path)));
};

const git = (repo, args) => execFileSync("git", ["-C", repo, ...args], { encoding: "utf8" }).trimEnd();

const validateRoots = (repoArgument, outputArgument) => {
  const repo = realpathSync(resolve(repoArgument));
  required(lstatSync(join(repo, ".git")).isDirectory(), "repository .git is not a real local directory");
  required(!existsSync(join(repo, ".git", "commondir")), "repository shares a Git common directory");
  required(!existsSync(join(repo, ".git", "shallow")), "repository is shallow");
  required(!existsSync(join(repo, ".git", "objects", "info", "alternates")), "repository uses object alternates");
  required(realpathSync(git(repo, ["rev-parse", "--show-toplevel"])) === repo, "--repo is not the standalone checkout root");
  const output = realpathSync(resolve(outputArgument));
  required(output !== repo && inside(repo, output), "output is not physically inside the checkout");
  required(output !== join(repo, "target"), "output may not be the target root");
  noSymlinkComponents(output, repo);
  return { repo, output };
};

// `git check-ignore -q` intentionally produces no stdout, so use its status.
const requireIgnored = (repo, output) => {
  try { execFileSync("git", ["-C", repo, "check-ignore", "-q", output], { stdio: "ignore" }); }
  catch { fail("output directory is not Git-ignored"); }
};

const digestFile = (repo, path, expected, label) => {
  const actual = sha256(readRegular(join(repo, path), path));
  required(actual === expected, `${label} digest is ${actual}, expected ${expected}`);
  return actual;
};

const validateSourceBindings = (repo, output, candidate) => {
  const source = readJson(join(output, "metadata/source-tree.json"), "source-tree.json");
  exactKeys(source, [
    "outcome", "commit", "tree", "issue", "issue_body_sha256", "r2_contract_sha256",
    "r2_revision_2_authority_sha256",
    "runtime_contract_sha256", "catalog_sha256", "packet_manifest_sha256",
    "committed_contact_sheet_sha256", "plan_sha256", "scene_signature_sha256",
  ], "source-tree.json");
  exactKeys(source.plan_sha256, ["scene_one", "scene_two"], "source-tree plan digests");
  exactKeys(source.scene_signature_sha256, ["scene_one", "scene_two"], "source-tree signature digests");
  required(source.outcome === "pass", "source-tree outcome is not pass");
  required(source.commit === candidate.commit && source.tree === candidate.tree, "source-tree candidate differs");
  required(source.issue === CONSTANTS.issue && source.issue_body_sha256 === CONSTANTS.issue_body_sha256, "issue authority differs");
  const expected = {
    r2_contract_sha256: CONSTANTS.r2_contract_sha256,
    r2_revision_2_authority_sha256: CONSTANTS.r2_revision_2_authority_sha256,
    runtime_contract_sha256: CONSTANTS.runtime_contract_sha256,
    catalog_sha256: CONSTANTS.catalog_sha256,
    packet_manifest_sha256: CONSTANTS.packet_manifest_sha256,
    committed_contact_sheet_sha256: CONSTANTS.committed_contact_sheet_sha256,
  };
  for (const [key, value] of Object.entries(expected)) required(source[key] === value, `${key} differs`);
  required(JSON.stringify(source.plan_sha256) === JSON.stringify(CONSTANTS.plans), "frozen plan digests differ");
  required(JSON.stringify(source.scene_signature_sha256) === JSON.stringify(CONSTANTS.signatures), "frozen signatures differ");
  digestFile(repo, "R2.md", CONSTANTS.r2_contract_sha256, "R2.md");
  digestFile(repo, "docs/decisions/0024-r2-final-proof-finalization-order.md", CONSTANTS.r2_revision_2_authority_sha256, "R2 revision-2 authority");
  digestFile(repo, "RUNTIME.md", CONSTANTS.runtime_contract_sha256, "RUNTIME.md");
  digestFile(repo, "apps/nomos-observed-viewer/src/catalog.mjs", CONSTANTS.catalog_sha256, "catalog");
  digestFile(repo, "docs/evaluation/r2-second-scene-packet/MANIFEST.sha256", CONSTANTS.packet_manifest_sha256, "second-scene packet");
  const committedSheetPath = join(repo, "docs/evaluation/runs/r2/2026-08-27-issue-197-second-author/evidence/contact-sheet.png");
  digestFile(repo, "docs/evaluation/runs/r2/2026-08-27-issue-197-second-author/evidence/contact-sheet.png", CONSTANTS.committed_contact_sheet_sha256, "committed contact sheet");
  const committedDimensions = pngDimensions(readRegular(committedSheetPath), "committed contact sheet");
  required(committedDimensions.width === 2560 && committedDimensions.height === 720, "committed contact sheet is not 2560x720");
  digestFile(repo, "fixtures/r2/plans/scene_one.json", CONSTANTS.plans.scene_one, "scene-one plan");
  digestFile(repo, "fixtures/r2/plans/scene_two.json", CONSTANTS.plans.scene_two, "scene-two plan");
  const maximum = readRegular(join(repo, "fixtures/r2/maximum-observed-scene.json"));
  required(maximum.length === CONSTANTS.maximum_fixture_bytes && sha256(maximum) === CONSTANTS.maximum_fixture_sha256, "maximum fixture binding differs");
  return source;
};

const validateCandidate = (repo, output, candidate) => {
  required(COMMIT.test(candidate.commit) && COMMIT.test(candidate.tree), "candidate IDs are not full lowercase Git IDs");
  required(git(repo, ["rev-parse", "--verify", "HEAD"]) === candidate.commit, "HEAD differs from candidate commit");
  required(git(repo, ["rev-parse", "HEAD^{tree}"]) === candidate.tree, "HEAD tree differs from candidate tree");
  try {
    git(repo, ["symbolic-ref", "-q", "HEAD"]);
    fail("candidate HEAD is not detached");
  } catch (error) {
    if (error.message?.includes("candidate HEAD is not detached")) throw error;
  }
  for (const name of ["clean-start.json", "clean-end.json"]) {
    const record = readJson(join(output, `metadata/${name}`), name);
    exactKeys(record, ["outcome", "commit", "tree", "porcelain"], name);
    required(record.outcome === "pass" && record.commit === candidate.commit && record.tree === candidate.tree && record.porcelain === "", `${name} is not clean at candidate`);
  }
  required(git(repo, ["status", "--porcelain=v1", "--untracked-files=all"]) === "", "checkout is not clean at receipt verification");
};

const parseLedger = (output) => {
  const path = join(output, "commands.tsv");
  const text = readText(path, "commands.tsv");
  required(text.endsWith("\n") && !text.includes("\r"), "commands.tsv has noncanonical lines");
  const lines = text.slice(0, -1).split("\n");
  required(lines.shift() === "ordinal\tcommand_id\tstarted_ns\tended_ns\texit_code\tstdout_path\tstderr_path\tcommand", "commands.tsv header differs");
  required(lines.length === COMMAND_IDS.length, `command count is ${lines.length}, expected ${COMMAND_IDS.length}`);
  const rows = lines.map((line, index) => {
    const fields = line.split("\t");
    required(fields.length === 8, `command row ${index + 1} does not have eight fields`);
    const [ordinal, id, started, ended, exit, stdout, stderr, command] = fields;
    required(ordinal === String(index + 1), `command ordinal ${ordinal} is not ${index + 1}`);
    required(id === COMMAND_IDS[index], `command ${index + 1} is ${id}, expected ${COMMAND_IDS[index]}`);
    required(/^\d+$/.test(started) && /^\d+$/.test(ended) && BigInt(ended) >= BigInt(started), `command ${id} timestamps are invalid`);
    required(exit === "0", `command ${id} exited ${exit}`);
    for (const [kind, value] of [["stdout", stdout], ["stderr", stderr]]) {
      required(value === `logs/${String(index + 1).padStart(2, "0")}-${id}.${kind}`, `${id} ${kind} path differs`);
      readRegular(join(output, value), `${id} ${kind}`);
    }
    required(stdout !== stderr && command === COMMAND_DISPLAYS[index], `command ${id} display differs`);
    return { ordinal: index + 1, id, started_ns: started, ended_ns: ended, stdout, stderr, command };
  });
  required(new Set(rows.flatMap((row) => [row.stdout, row.stderr])).size === rows.length * 2, "command logs reuse a path");
  return rows;
};

const tapSummary = (text, label) => {
  const value = (name) => {
    const matches = [...text.matchAll(new RegExp(`^# ${name} (\\d+)$`, "gm"))];
    required(matches.length === 1, `${label} has no unique ${name} summary`);
    return Number(matches[0][1]);
  };
  const summary = Object.fromEntries(["tests", "pass", "fail", "cancelled", "skipped", "todo"].map((key) => [key, value(key)]));
  required(summary.tests > 0 && summary.pass === summary.tests && summary.fail === 0 && summary.cancelled === 0 && summary.skipped === 0 && summary.todo === 0, `${label} is not an unskipped pass`);
  return summary;
};

const validateComponentLogs = (output, ledger) => {
  const stdout = (id) => readText(join(output, ledger.find((row) => row.id === id).stdout), `${id} stdout`);
  const markers = {
    "r2-schema-ownership": "R2_SCHEMA_OWNERSHIP PASS",
    "r2-source-provenance": "R2_SOURCE_PROVENANCE PASS",
    "r2-source-provenance-plants": "R2_SOURCE_PROVENANCE_PLANTS PASS",
    "r2-adopter-neutrality": "R2_ADOPTER_NEUTRALITY PASS",
    "r2-adopter-neutrality-plants": "R2_ADOPTER_NEUTRALITY_PLANTS PASS",
    "r2-maximum-fixture": `r2 maximum: ${CONSTANTS.maximum_fixture_bytes} bytes ${CONSTANTS.maximum_fixture_sha256}`,
  };
  for (const [id, marker] of Object.entries(markers)) required(stdout(id).includes(marker), `${id} PASS marker is absent`);
  const proofMarkers = {
    "workspace-boundary": "boundary: clean",
    "r1-viewer-build": "NOMOS_VIEWER_BUILD PASS",
    "r1-browser-smoke": "NOMOS_VIEWER_SMOKE PASS areas=6 moves=65 cost=95",
    "r1-native-replay": "NOMOS_PLAY_REPLAY PASS areas=6 commands=77 receipts=77",
    "r2-compiler-tests": "R2_SECOND_SCENE_PACKET_PLANTS PASS",
    "r2-viewer-tests": "R2_COMPLETE_PROOF_PLANTS PASS",
    "r2-viewer-build": "NOMOS_OBSERVED_BUILD PASS",
    "r2-browser-smoke": "NOMOS_OBSERVED_SMOKE PASS scenes=2 samples=20 external=0",
    "clean-r1-viewer-build": "NOMOS_VIEWER_BUILD PASS",
    "clean-r2-viewer-build-a": "NOMOS_OBSERVED_BUILD PASS",
    "clean-r2-viewer-build-b": "NOMOS_OBSERVED_BUILD PASS",
    "maximum-compile-benchmark": "r2 compile latency:",
  };
  for (const [id, marker] of Object.entries(proofMarkers)) required(stdout(id).includes(marker), `${id} proof marker is absent`);
  required(stdout("maximum-compile-benchmark").includes("; PASS"), "maximum-compile-benchmark PASS marker is absent");
  const schemaPlants = stdout("r2-schema-plants");
  required(schemaPlants === "expected refusal: missing\nexpected refusal: duplicate\nexpected refusal: third\n", "r2-schema-plants output differs");
  const r1GaolTap = tapSummary(stdout("r1-gaol-verify"), "R1 gaol tests");
  const r1Tap = tapSummary(stdout("r1-viewer-tests"), "R1 viewer tests");
  const r2Tap = tapSummary(stdout("r2-viewer-tests"), "R2 viewer tests");
  required(r1Tap.tests === 104 && r1Tap.pass === 104, "R1 viewer test count is not 104/104");
  return { r1_gaol_tests: r1GaolTap, r1_viewer_tests: r1Tap, r2_viewer_tests: r2Tap };
};

const validateIpRows = (addresses, route4, route6, label = "recorded isolation") => {
  required(Array.isArray(addresses) && addresses.length === 1, `${label} does not contain exactly loopback`);
  const lo = addresses[0];
  required(lo.ifname === "lo" && lo.link_type === "loopback" && Array.isArray(lo.flags) && lo.flags.includes("LOOPBACK") && lo.flags.includes("UP") && Array.isArray(lo.addr_info), `${label} loopback is not up`);
  const addressesByFamily = Object.fromEntries(lo.addr_info.map((row) => [row.family, row]));
  required(lo.addr_info.length === 2 && new Set(lo.addr_info.map((row) => row.family)).size === 2 && addressesByFamily.inet?.local === "127.0.0.1" && addressesByFamily.inet?.prefixlen === 8 && addressesByFamily.inet?.scope === "host" && addressesByFamily.inet6?.local === "::1" && addressesByFamily.inet6?.prefixlen === 128 && addressesByFamily.inet6?.scope === "host", `${label} address families are not exact loopback`);
  required(Array.isArray(route4) && route4.length === 3 && Array.isArray(route6) && route6.length === 1, `${label} route counts are not exact loopback`);
  const expected4 = new Map([["127.0.0.0/8", ["local", "host"]], ["127.0.0.1", ["local", "host"]], ["127.255.255.255", ["broadcast", "link"]]]);
  for (const row of route4) {
    const expected = expected4.get(row.dst);
    required(expected && row.type === expected[0] && row.scope === expected[1] && row.dev === "lo" && row.table === "local" && row.protocol === "kernel" && row.prefsrc === "127.0.0.1", `${label} leaks or alters an IPv4 route`);
  }
  const one6 = route6[0];
  required(one6.dst === "::1" && one6.type === "local" && one6.dev === "lo" && one6.table === "local" && one6.protocol === "kernel", `${label} leaks or alters an IPv6 route`);
};

const liveIp = (args) => JSON.parse(execFileSync("ip", ["-j", ...args], { encoding: "utf8" }));

const validateIsolation = (output, liveChecks) => {
  const isolation = readJson(join(output, "metadata/isolation.json"), "isolation.json");
  exactKeys(isolation, ["outcome", "namespace", "external_negative_control", "loopback_only"], "isolation.json");
  required(isolation.outcome === "pass" && isolation.namespace === "fresh" && isolation.external_negative_control === "blocked" && isolation.loopback_only === true, "isolation summary is not a pass");
  const addresses = readJson(join(output, "metadata/ip-address.json"), "ip-address.json");
  const route4 = readJson(join(output, "metadata/ip-route-v4.json"), "ip-route-v4.json");
  const route6 = readJson(join(output, "metadata/ip-route-v6.json"), "ip-route-v6.json");
  validateIpRows(addresses, route4, route6);
  const control = readJson(join(output, "metadata/network-control.json"), "network-control.json");
  exactKeys(control, ["outcome", "destination", "outer_positive", "inner_negative"], "network-control.json");
  exactKeys(control.outer_positive, ["outcome", "exit_code", "stdout", "stderr"], "network outer positive control");
  exactKeys(control.inner_negative, ["outcome", "exit_code", "stdout", "stderr"], "network inner negative control");
  required(control.outcome === "pass" && control.destination === "1.1.1.1:53", "network control identity/result differs");
  required(control.outer_positive.outcome === "connected" && control.outer_positive.exit_code === 0 && control.outer_positive.stdout === "connected\n" && control.outer_positive.stderr === "", "external-connect positive control did not connect");
  required(control.inner_negative.outcome === "blocked" && Number.isInteger(control.inner_negative.exit_code) && control.inner_negative.exit_code > 0 && control.inner_negative.stdout === "" && typeof control.inner_negative.stderr === "string" && control.inner_negative.stderr.length > 0, "external-connect negative control is not a recorded refusal");
  for (const [key, prefix] of [["outer_positive", "network-outer-positive"], ["inner_negative", "network-inner-negative"]]) {
    const rawStdout = readText(join(output, `metadata/${prefix}.stdout`), `${prefix}.stdout`);
    const rawStderr = readText(join(output, `metadata/${prefix}.stderr`), `${prefix}.stderr`);
    required(control[key].stdout === rawStdout && control[key].stderr === rawStderr, `${key.replaceAll("_", "-")} JSON does not match its raw streams`);
  }
  if (liveChecks) validateIpRows(liveIp(["address", "show"]), liveIp(["-4", "route", "show", "table", "all"]), liveIp(["-6", "route", "show", "table", "all"]), "live namespace");
  return isolation;
};

const decodeMountPath = (text) => text.replace(/\\([0-7]{3})/g, (_, octal) => String.fromCharCode(Number.parseInt(octal, 8)));

const validateFilesystemIsolation = (repo, output, liveChecks) => {
  const record = readJson(join(output, "metadata/filesystem-isolation.json"), "filesystem-isolation.json");
  exactKeys(record, ["outcome", "mechanism", "repository_mount", "writable_roots", "negative_control"], "filesystem-isolation.json");
  exactKeys(record.negative_control, ["path", "operation", "exit_code", "stdout", "stderr"], "filesystem-isolation negative control");
  const expectedOutput = relative(repo, output).split(sep).join("/");
  required(record.outcome === "pass" && record.mechanism === "bubblewrap" && record.repository_mount === "read-only", "filesystem isolation is not a bubblewrap read-only pass");
  required(JSON.stringify(record.writable_roots) === JSON.stringify([expectedOutput, "target"]), "filesystem isolation writable roots differ");
  const control = record.negative_control;
  required(control.path === "README.md" && control.operation === "append" && Number.isInteger(control.exit_code) && control.exit_code > 0 && control.stdout === "" && typeof control.stderr === "string" && control.stderr.length > 0, "read-only filesystem negative control is not a recorded refusal");
  const rawStdout = readText(join(output, "metadata/read-only-negative-control.stdout"), "read-only-negative-control.stdout");
  const rawStderr = readText(join(output, "metadata/read-only-negative-control.stderr"), "read-only-negative-control.stderr");
  required(control.stdout === rawStdout && control.stderr === rawStderr, "read-only negative-control JSON does not match its raw streams");
  const mountinfo = readText(join(output, "metadata/mountinfo.txt"), "mountinfo.txt");
  required(mountinfo.endsWith("\n") && !mountinfo.includes("\r"), "mountinfo.txt is empty or noncanonical");
  const mounts = mountinfo.trimEnd().split("\n").map((line) => {
    const fields = line.split(" ");
    const separator = fields.indexOf("-");
    required(separator >= 6 && fields.length >= separator + 4, "mountinfo.txt has a malformed row");
    return { mount: decodeMountPath(fields[4]), options: fields[5].split(",") };
  });
  const requireMode = (mount, mode) => required(mounts.some((row) => row.mount === mount && row.options.includes(mode)), `mountinfo does not record ${mount} as ${mode}`);
  requireMode("/", "ro");
  requireMode(join(repo, "target"), "rw");
  requireMode(output, "rw");
  const writableInRepo = [...new Set(mounts.filter((row) => inside(repo, row.mount) && row.options.includes("rw")).map((row) => row.mount))].sort();
  const expectedWritable = [join(repo, "target"), output].sort();
  required(JSON.stringify(writableInRepo) === JSON.stringify(expectedWritable), "mountinfo records an unexpected writable checkout mount");
  if (liveChecks) required(mountinfo === readFileSync("/proc/self/mountinfo", "utf8"), "recorded mountinfo differs from the live proof namespace");
  return record;
};

const shellCommandPath = (name) => realpathSync(execFileSync("bash", ["-c", "command -v -- \"$1\"", "tool-path", name], { encoding: "utf8" }).trim());

const validateTools = (output, liveChecks) => {
  const text = readText(join(output, "metadata/tools.txt"), "tools.txt");
  required(text.endsWith("\n") && !text.includes("\r"), "tools.txt has noncanonical lines");
  const lines = text.slice(0, -1).split("\n");
  required(lines.shift() === "tool\tpath\tsha256" && lines.length === TOOL_LABELS.length, "tools.txt header/count differs");
  const rows = lines.map((line, index) => {
    const fields = line.split("\t");
    required(fields.length === 3 && fields[0] === TOOL_LABELS[index] && fields[1].startsWith("/") && HEX.test(fields[2]), `tools.txt row ${index + 1} differs`);
    const [tool, path, digest] = fields;
    required(existsSync(path) && realpathSync(path) === path, `tool ${tool} path is absent or noncanonical`);
    const info = lstatSync(path);
    required(info.isFile() && !info.isSymbolicLink() && (info.mode & 0o111) !== 0, `tool ${tool} is not one executable regular file`);
    required(sha256(readFileSync(path)) === digest, `tool ${tool} digest differs`);
    return { tool, path, sha256: digest };
  });
  const versions = readText(join(output, "metadata/tool-versions.txt"), "tool-versions.txt");
  required(versions.endsWith("\n") && versions.trim().length > 0 && !versions.includes("\r"), "tool-versions.txt is empty or noncanonical");
  const versionKeys = ["git", "bash", "rustc", "cargo", "rustup", "node", "jq", "bubblewrap", "cc", "ld", "chrome"];
  const versionLines = versions.slice(0, -1).split("\n");
  required(versionLines.length === versionKeys.length, "tool-versions.txt row count differs");
  versionLines.forEach((line, index) => {
    const equals = line.indexOf("=");
    required(equals > 0 && line.slice(0, equals) === versionKeys[index] && line.slice(equals + 1).length > 0, `tool-versions.txt row ${index + 1} differs`);
  });
  if (liveChecks) {
    const byTool = new Map(rows.map((row) => [row.tool, row.path]));
    const expectedPath = (label) => {
      if (label === "gnu-time") return realpathSync("/usr/bin/time");
      if (label === "cargo-toolchain" || label === "rustc-toolchain") return realpathSync(execFileSync("rustup", ["which", label.slice(0, -"-toolchain".length)], { encoding: "utf8" }).trim());
      if (label === "rust-lld") {
        const rustc = byTool.get("rustc-toolchain");
        const sysroot = execFileSync(rustc, ["--print", "sysroot"], { encoding: "utf8" }).trim();
        const host = /^host: (\S+)$/m.exec(execFileSync(rustc, ["-vV"], { encoding: "utf8" }))?.[1];
        required(host, "rustc toolchain did not report a host triple");
        return realpathSync(join(sysroot, "lib/rustlib", host, "bin/rust-lld"));
      }
      if (label === "chrome") {
        required(typeof process.env.CHROME_BIN === "string" && process.env.CHROME_BIN.length > 0, "CHROME_BIN is absent during tool verification");
        return realpathSync(process.env.CHROME_BIN);
      }
      return shellCommandPath(label);
    };
    rows.forEach((row) => required(row.path === expectedPath(row.tool), `tool ${row.tool} path differs from the live proof environment`));
    const versionSpecs = [
      ["git", ["--version"]], ["bash", ["-c", "printf '%s\\n' \"$BASH_VERSION\""]],
      ["rustc-toolchain", ["--version"]], ["cargo-toolchain", ["--version"]], ["rustup", ["--version"]],
      ["node", ["--version"]], ["jq", ["--version"]], ["bwrap", ["--version"]],
      ["cc", ["--version"]], ["ld", ["--version"]], ["chrome", ["--version"]],
    ];
    const recordedVersions = versionLines.map((line) => line.slice(line.indexOf("=") + 1));
    versionSpecs.forEach(([executable, args], index) => {
      const actual = execFileSync(byTool.get(executable), args, { encoding: "utf8" }).trimEnd().split("\n")[0];
      required(recordedVersions[index] === actual, `tool version ${versionKeys[index]} differs from the live proof environment`);
    });
  }
  return { count: rows.length, tools_sha256: sha256(Buffer.from(text)), versions_sha256: sha256(Buffer.from(versions)) };
};

const validateClosure = (repo, output, liveChecks) => {
  const process = readJson(join(output, "metadata/process-closure.json"), "process-closure.json");
  exactKeys(process, ["outcome", "checked_while_sampler", "checked_after_sampler", "leaked_processes", "namespace_children_before_sampler_stop", "namespace_children"], "process-closure.json");
  required(process.outcome === "pass" && process.checked_while_sampler === true && process.checked_after_sampler === true && Array.isArray(process.leaked_processes) && process.leaked_processes.length === 0 && Array.isArray(process.namespace_children_before_sampler_stop) && process.namespace_children_before_sampler_stop.length === 0 && Array.isArray(process.namespace_children) && process.namespace_children.length === 0, "proof processes did not close before and after sampler stop");
  const renderPids = (rows) => `${rows.map(String).join("\n")}${rows.length ? "\n" : ""}`;
  required(readText(join(output, "metadata/namespace-children-before-sampler-stop.txt"), "namespace-children-before-sampler-stop.txt") === renderPids(process.namespace_children_before_sampler_stop), "pre-stop process-closure raw list differs");
  required(readText(join(output, "metadata/namespace-children.txt"), "namespace-children.txt") === renderPids(process.namespace_children), "final process-closure raw list differs");
  const boundary = readJson(join(output, "metadata/write-boundary.json"), "write-boundary.json");
  exactKeys(boundary, ["outcome", "output_relative", "allowed_roots", "outside_writes", "inputs_unchanged"], "write-boundary.json");
  const expectedOutput = relative(repo, output).split(sep).join("/");
  required(boundary.outcome === "pass" && boundary.output_relative === expectedOutput && boundary.inputs_unchanged === true && Array.isArray(boundary.outside_writes) && boundary.outside_writes.length === 0, "write boundary is not clean");
  required(JSON.stringify(boundary.allowed_roots) === JSON.stringify([expectedOutput, "target"]), "write-boundary allowed roots differ");
  required(readText(join(output, "metadata/environment.txt"), "environment.txt").trim().length > 0, "environment.txt is empty");
  if (liveChecks) { required(globalThis.process.env.NOMOS_R2_PROOF_INNER === "1", "live closure verification is outside the isolated proof"); auditLiveProcessNamespace(); }
  return { process, boundary };
};

const decimalSecondsToNs = (text) => {
  const parts = text.split(":");
  required(parts.length === 2 || parts.length === 3, `invalid GNU time elapsed value ${text}`);
  const secondsText = parts.pop();
  required(/^\d+(?:\.\d{1,9})?$/.test(secondsText) && parts.every((one) => /^\d+$/.test(one)), `invalid GNU time elapsed value ${text}`);
  const [whole, fraction = ""] = secondsText.split(".");
  let seconds = BigInt(whole) + BigInt(parts.pop()) * 60n;
  if (parts.length) seconds += BigInt(parts.pop()) * 3600n;
  return seconds * 1_000_000_000n + BigInt(fraction.padEnd(9, "0"));
};

const validateMeasurements = (output) => {
  const time = readText(join(output, "measurements/clean-release-time.txt"), "GNU time record");
  const elapsed = [...time.matchAll(/^\s*Elapsed \(wall clock\) time \(h:mm:ss or m:ss\):\s*(\S+)\s*$/gm)];
  const exits = [...time.matchAll(/^\s*Exit status:\s*(\d+)\s*$/gm)];
  required(elapsed.length === 1 && exits.length === 1 && exits[0][1] === "0", "GNU time record is incomplete or failed");
  const buildNs = decimalSecondsToNs(elapsed[0][1]);
  required(buildNs <= 60_000_000_000n, `clean release build exceeded 60 s: ${buildNs} ns`);

  const diskText = readText(join(output, "measurements/checkout-disk-samples.tsv"), "disk samples");
  required(diskText.endsWith("\n") && !diskText.includes("\r"), "disk samples have noncanonical lines");
  const diskLines = diskText.slice(0, -1).split("\n");
  required(diskLines.shift() === "ordinal\telapsed_ms\tmebibytes", "disk samples header differs");
  required(diskLines.length >= 2, "disk sampler retained fewer than two rows");
  let previous = -1;
  let maximumGap = 0;
  const rows = diskLines.map((line, index) => {
    const fields = line.split("\t");
    required(fields.length === 3 && fields[0] === String(index) && /^\d+$/.test(fields[1]) && /^\d+$/.test(fields[2]), `disk row ${index} is invalid`);
    const elapsedMs = Number(fields[1]);
    required(elapsedMs > previous, `disk elapsed time is not increasing at ${index}`);
    if (index > 0) {
      maximumGap = Math.max(maximumGap, elapsedMs - previous);
      required(elapsedMs - previous <= 100, `disk sampler gap exceeds 100 ms at ${index}`);
    }
    previous = elapsedMs;
    return { elapsed_ms: elapsedMs, mebibytes: Number(fields[2]) };
  });
  const maximum = Math.max(...rows.map((row) => row.mebibytes));
  required(maximum <= 8_192, `checkout peak disk exceeded 8192 MiB: ${maximum}`);
  const summary = readJson(join(output, "measurements/checkout-disk-summary.json"), "disk summary");
  exactKeys(summary, ["outcome", "interval_ms", "samples", "initial_mib", "final_mib", "maximum_mib", "max_gap_ms", "cpu_priority", "io_priority_class", "concurrency_limit", "du_arguments"], "disk summary");
  required(summary.outcome === "pass" && summary.interval_ms === 50 && summary.samples === rows.length && summary.initial_mib === rows[0].mebibytes && summary.final_mib === rows.at(-1).mebibytes && summary.maximum_mib === maximum && summary.max_gap_ms === maximumGap && summary.cpu_priority === "ordinary" && summary.io_priority_class === "idle" && summary.concurrency_limit === 16 && JSON.stringify(summary.du_arguments) === JSON.stringify(["-sm", "--", "<checkout>"]), "disk summary arithmetic or method differs from raw rows");
  return { clean_release_build_ns: buildNs.toString(), checkout_peak_mib: maximum, disk_samples: rows.length, disk_maximum_gap_ms: maximumGap };
};

const treeInventory = (directory) => regularTree(directory).map(({ path, bytes, sha256: digest }) => ({ path, bytes, sha256: digest }));

const validateBuildReceipt = (repo, directory, expectedPlans) => {
  const receipt = readJson(join(directory, "receipt.json"), `${relative(repo, directory)}/receipt.json`);
  exactKeys(receipt, ["catalog_sha256", "files", "generated_by", "node", "outcome", "plans", "total_bytes"], "R2 build receipt");
  required(receipt.outcome === "pass" && receipt.catalog_sha256 === CONSTANTS.catalog_sha256 && receipt.generated_by === "apps/nomos-observed-viewer/build.mjs" && typeof receipt.node === "string" && receipt.node.length > 0 && Array.isArray(receipt.files), "R2 build receipt identity differs");
  receipt.files.forEach((row) => exactKeys(row, ["bytes", "path", "sha256"], "R2 build file receipt row"));
  const inventory = treeInventory(join(directory, "dist"));
  const recordedFiles = receipt.files.map(({ path, bytes, sha256: digest }) => ({ path, bytes, sha256: digest }))
    .sort((a, b) => Buffer.from(a.path).compare(Buffer.from(b.path)));
  required(JSON.stringify(recordedFiles) === JSON.stringify(inventory), "R2 build receipt does not match its dist tree");
  const total = inventory.reduce((sum, row) => sum + row.bytes, 0);
  required(receipt.total_bytes === total && total <= 2_000_000, `R2 distribution budget/tree differs: ${total}`);
  required(Array.isArray(receipt.plans) && receipt.plans.length === 2, "R2 build receipt does not bind two plans");
  const expectedPlanRows = ["scene_one", "scene_two"].map((name) => {
    const bytes = readRegular(join(repo, `fixtures/r2/plans/${name}.json`));
    return { bytes: bytes.length, path: `plans/${name}.json`, sha256: expectedPlans[name] };
  });
  required(JSON.stringify(receipt.plans) === JSON.stringify(expectedPlanRows), "R2 distribution plan bindings differ");
  for (const row of expectedPlanRows) required(readRegular(join(directory, "dist", row.path)).equals(readRegular(join(repo, `fixtures/r2/${row.path}`))), `R2 distribution ${row.path} differs from its committed plan`);
  const integrity = `catalog_sha256\t${CONSTANTS.catalog_sha256}\n${expectedPlanRows.map((row) => `${row.sha256}\t${row.bytes}\t${row.path}`).join("\n")}\n`;
  required(readText(join(directory, "dist/ARTIFACTS.sha256"), "R2 distribution integrity manifest") === integrity, "R2 distribution integrity manifest differs");
  const publicText = readText(join(repo, "apps/nomos-observed-viewer/PUBLIC_FILES"), "R2 PUBLIC_FILES");
  required(publicText.endsWith("\n") && !publicText.includes("\r") && !publicText.includes("\n\n"), "R2 PUBLIC_FILES is noncanonical");
  const publicFiles = publicText.slice(0, -1).split("\n");
  required(JSON.stringify(publicFiles) === JSON.stringify([...new Set(publicFiles)].sort()), "R2 PUBLIC_FILES is not sorted and unique");
  for (const row of inventory.filter((one) => !one.path.startsWith("vendor/"))) {
    const text = readText(join(directory, "dist", row.path), `R2 dist scan ${row.path}`).toLowerCase();
    for (const forbidden of ["compiler-receipts", "world-ir", "nomos.observed_scene@1", "the-mortal-estate", "mortal_estate", "cairn"]) required(!text.includes(forbidden), `R2 distribution contains forbidden payload ${forbidden} in ${row.path}`);
    required(!/\.nomos(?:["'\s]|$)/.test(text) && !/\b(?:https?|wss?):\/\//.test(text) && !/(?:^|[^\w])\/(?:home|data|work|tmp)\//.test(text), `R2 distribution contains a forbidden source, origin, or machine path in ${row.path}`);
  }
  for (const path of publicFiles) {
    required(safeRelative(path) && /\.(?:html|css|mjs)$/.test(path), `unsafe R2 public path ${path}`);
    required(readRegular(join(directory, "dist", path), `R2 dist ${path}`).equals(readRegular(join(repo, "apps/nomos-observed-viewer", path), `R2 source ${path}`)), `R2 distribution public source differs: ${path}`);
  }
  const vendorManifest = readJson(join(repo, "apps/nomos-viewer/vendor/MANIFEST.json"), "accepted vendor manifest");
  const vendorRows = vendorManifest.packages.flatMap((one) => one.files).filter((row) => ["three/LICENSE", "three/three.core.min.js", "three/three.module.min.js"].includes(row.path));
  required(vendorRows.length === 3 && new Set(vendorRows.map((row) => row.path)).size === 3, "accepted vendor manifest lacks the three R2 files");
  for (const row of vendorRows) {
    const source = readRegular(join(repo, "apps/nomos-viewer/vendor", row.path), `vendor source ${row.path}`);
    required(row.bytes === source.length && row.sha256 === sha256(source) && readRegular(join(directory, "dist/vendor", row.path), `R2 vendor ${row.path}`).equals(source), `R2 vendor binding differs: ${row.path}`);
  }
  const expectedPaths = ["ARTIFACTS.sha256", ...publicFiles, "plans/scene_one.json", "plans/scene_two.json", "vendor/three/LICENSE", "vendor/three/three.core.min.js", "vendor/three/three.module.min.js"].sort();
  required(JSON.stringify(inventory.map((row) => row.path)) === JSON.stringify(expectedPaths), "R2 distribution closed path set differs");
  return { receipt, inventory, total };
};

const canonical = (value) => {
  if (value === null || typeof value === "boolean" || typeof value === "number") return JSON.stringify(value);
  if (typeof value === "string") return JSON.stringify(value);
  if (Array.isArray(value)) return `[${value.map(canonical).join(",")}]`;
  return `{${Object.keys(value).sort((a, b) => Buffer.from(a).compare(Buffer.from(b))).map((key) => `${JSON.stringify(key)}:${canonical(value[key])}`).join(",")}}`;
};

const signature = (scene) => {
  const actorTuples = scene.actors.map(({ cell, controlled, hostile, life_state, protected: protectedValue }) => ({ cell, controlled, hostile, life_state, protected: protectedValue }));
  required(new Set(actorTuples.map(canonical)).size === actorTuples.length, "scene actor tuples are not unique");
  const actors = [...actorTuples].sort((a, b) => canonical(a).localeCompare(canonical(b)));
  const ordinal = new Map(scene.actors.map((actor) => [actor.id, actors.findIndex((row) => canonical(row) === canonical({ cell: actor.cell, controlled: actor.controlled, hostile: actor.hostile, life_state: actor.life_state, protected: actor.protected }))]));
  const terrain = scene.terrain_layers.map(({ cells, role }) => ({ cells, role })).sort((a, b) => canonical(a).localeCompare(canonical(b)));
  const actions = scene.actions.map(({ availability, target_actor }) => ({ availability, target_actor_ordinal: ordinal.get(target_actor) })).sort((a, b) => canonical(a).localeCompare(canonical(b)));
  const normalized = { actions, actors, crop: scene.crop, terrain };
  const axes = Object.fromEntries(Object.entries(normalized).map(([key, value]) => [key, sha256(canonical(value))]));
  return { axis_sha256: axes, normalized, sha256: sha256(canonical(normalized)) };
};

const validateScenes = (repo, output) => {
  for (const [directory, planName] of [["scene-a", "scene_one"], ["scene-b", "scene_two"]]) {
    const expected = readRegular(join(repo, `fixtures/r2/plans/${planName}.json`));
    for (let index = 0; index < 10; index += 1) {
      const actual = readRegular(join(output, `r2/${directory}/plan-${String(index).padStart(2, "0")}.json`));
      required(actual.equals(expected), `${directory} reproduction ${index} differs from committed plan`);
    }
  }
  const signatureRecord = readJson(join(output, "r2/signatures.json"), "R2 signatures");
  exactKeys(signatureRecord, ["outcome", "scenes"], "R2 signatures");
  required(signatureRecord.outcome === "pass" && Array.isArray(signatureRecord.scenes) && signatureRecord.scenes.length === 2, "R2 signature output is not a two-scene pass");
  const scenes = ["scene_one", "scene_two"].map((name) => signature(readJson(join(repo, `fixtures/r2/scenes/${name}.json`))));
  for (let index = 0; index < 2; index += 1) {
    exactKeys(signatureRecord.scenes[index], ["axis_sha256", "normalized", "path", "sha256"], `recorded scene ${index + 1} signature`);
    exactKeys(signatureRecord.scenes[index].axis_sha256, ["actions", "actors", "crop", "terrain"], `recorded scene ${index + 1} signature axes`);
    required(scenes[index].sha256 === Object.values(CONSTANTS.signatures)[index], `scene ${index + 1} signature differs`);
    required(signatureRecord.scenes[index].path === `fixtures/r2/scenes/scene_${index === 0 ? "one" : "two"}.json` && signatureRecord.scenes[index].sha256 === scenes[index].sha256 && JSON.stringify(signatureRecord.scenes[index].axis_sha256) === JSON.stringify(scenes[index].axis_sha256) && JSON.stringify(signatureRecord.scenes[index].normalized) === JSON.stringify(scenes[index].normalized), `recorded scene ${index + 1} signature arithmetic differs`);
  }
  for (const axis of ["crop", "terrain", "actors", "actions"]) required(scenes[0].axis_sha256[axis] !== scenes[1].axis_sha256[axis], `scene signatures do not differ on ${axis}`);
  return scenes.map((row) => ({ sha256: row.sha256, axis_sha256: row.axis_sha256 }));
};

const validateCompileBenchmark = (repo, output) => {
  const root = join(output, "r2/compile-benchmark");
  const fixturePath = join(repo, "fixtures/r2/maximum-observed-scene.json");
  const devices = [repo, output, root, fixturePath].map((path) => statSync(path).dev);
  required(devices.every((device) => device === devices[0]), "compile benchmark output and maximum fixture are not on the checkout filesystem");
  const expectedNames = ["samples.tsv", "summary.json", ...Array.from({ length: 10 }, (_, i) => `warmup-${String(i).padStart(3, "0")}.json`), ...Array.from({ length: 100 }, (_, i) => `sample-${String(i).padStart(3, "0")}.json`)].sort();
  const actualNames = readdirSync(root).sort();
  required(JSON.stringify(actualNames) === JSON.stringify(expectedNames), "compile benchmark retained file set differs");
  const outputs = expectedNames.filter((name) => name.endsWith(".json") && name !== "summary.json").map((name) => {
    const bytes = readRegular(join(root, name));
    const plan = JSON.parse(bytes.toString("utf8"));
    required(plan.schema === "nomos.observed_scene_plan@1" && plan.source_sha256 === CONSTANTS.maximum_fixture_sha256, `compile benchmark output ${name} is not the maximum-scene plan`);
    return { name, bytes: bytes.length, sha256: sha256(bytes) };
  });
  required(new Set(outputs.map((row) => `${row.bytes}:${row.sha256}`)).size === 1, "compile benchmark outputs are not byte-identical");
  const tsv = readText(join(root, "samples.tsv"));
  required(tsv.endsWith("\n") && !tsv.includes("\r"), "compile samples have noncanonical lines");
  const lines = tsv.slice(0, -1).split("\n");
  required(lines.shift() === "ordinal\telapsed_ns\tbytes\tsha256\tpath" && lines.length === 100, "compile samples header/count differs");
  const durations = lines.map((line, index) => {
    const fields = line.split("\t");
    required(fields.length === 5 && fields[0] === String(index) && /^\d+$/.test(fields[1]) && /^\d+$/.test(fields[2]) && HEX.test(fields[3]), `compile sample ${index} is invalid`);
    const expectedPath = realpathSync(join(root, `sample-${String(index).padStart(3, "0")}.json`));
    required(realpathSync(fields[4]) === expectedPath, `compile sample ${index} path differs`);
    const row = outputs.find((one) => one.name === `sample-${String(index).padStart(3, "0")}.json`);
    required(Number(fields[2]) === row.bytes && fields[3] === row.sha256 && BigInt(fields[1]) > 0n, `compile sample ${index} output binding differs`);
    return BigInt(fields[1]);
  });
  const sorted = [...durations].sort((a, b) => a < b ? -1 : a > b ? 1 : 0);
  const medianNumerator = sorted[49] + sorted[50];
  const p95 = sorted[94];
  const summary = readJson(join(root, "summary.json"));
  exactKeys(summary, [
    "architecture", "binary", "binary_sha256", "cpu_count", "fixture", "fixture_sha256",
    "hostname", "median_ceiling_ns", "median_denominator", "median_numerator_ns",
    "median_pass", "node", "output_digest", "p95_ceiling_ns", "p95_ns", "p95_pass",
    "platform", "release", "recorded_samples", "warmups",
  ], "compile summary");
  required(summary.recorded_samples === 100 && summary.warmups === 10 && summary.median_denominator === 2 && summary.median_numerator_ns === medianNumerator.toString() && summary.p95_ns === p95.toString(), "compile summary arithmetic differs");
  required(summary.median_ceiling_ns === 50_000_000 && summary.p95_ceiling_ns === 100_000_000 && summary.median_pass === true && summary.p95_pass === true, "compile summary ceilings differ");
  required(medianNumerator <= 100_000_000n && p95 <= 100_000_000n, "compile latency ceiling exceeded");
  required(summary.fixture_sha256 === CONSTANTS.maximum_fixture_sha256 && summary.output_digest === `${outputs[0].bytes}:${outputs[0].sha256}`, "compile summary fixture/output differs");
  const expectedBinary = realpathSync(join(repo, "target/r2-complete-release/release/nomos-observed-scene"));
  required(realpathSync(summary.binary) === expectedBinary && HEX.test(summary.binary_sha256) && sha256(readRegular(summary.binary)) === summary.binary_sha256, "compile benchmark binary binding differs");
  required(realpathSync(summary.fixture) === realpathSync(fixturePath), "compile benchmark fixture path differs");
  required(typeof summary.architecture === "string" && summary.architecture.length > 0 && Number.isInteger(summary.cpu_count) && summary.cpu_count > 0 && typeof summary.hostname === "string" && summary.hostname.length > 0 && typeof summary.node === "string" && summary.node.length > 0 && typeof summary.platform === "string" && summary.platform.length > 0 && typeof summary.release === "string" && summary.release.length > 0, "compile summary environment fields differ");
  return { warmups: 10, samples: 100, median_numerator_ns: medianNumerator.toString(), median_denominator: 2, p95_ns: p95.toString(), output_sha256: outputs[0].sha256 };
};

const pngDimensions = (bytes, label) => {
  required(bytes.length >= 24 && bytes.subarray(0, 8).toString("hex") === "89504e470d0a1a0a" && bytes.subarray(12, 16).toString("ascii") === "IHDR", `${label} is not a PNG`);
  return { width: bytes.readUInt32BE(16), height: bytes.readUInt32BE(20) };
};

const summarize = (values) => {
  const sorted = values.map(BigInt).sort((a, b) => a < b ? -1 : a > b ? 1 : 0);
  return { median_numerator_ns: (sorted[sorted.length / 2 - 1] + sorted[sorted.length / 2]).toString(), median_denominator: 2, p95_ns: sorted[Math.ceil(0.95 * sorted.length) - 1].toString() };
};

const validateBrowser = (repo, output) => {
  const root = join(output, "r2/browser-smoke");
  required(JSON.stringify(readdirSync(root).sort()) === JSON.stringify(["contact-sheet.png", "receipt.json", "scene_1.png", "scene_2.png"]), "R2 browser retained file set differs");
  const receipt = readJson(join(root, "receipt.json"), "R2 browser receipt");
  exactKeys(receipt, ["browser", "chrome_flags", "closures", "combined", "contact_sheet", "external_requests", "launches", "outcome", "per_scene", "requests", "samples_per_scene", "screenshots"], "R2 browser receipt");
  exactKeys(receipt.combined, ["median_denominator", "median_numerator_ns", "p95_ns"], "R2 browser combined timing");
  required(receipt.outcome === "pass" && typeof receipt.browser === "string" && receipt.browser.length > 0 && Array.isArray(receipt.chrome_flags) && receipt.samples_per_scene === 10 && Array.isArray(receipt.per_scene) && receipt.per_scene.length === 2 && Array.isArray(receipt.requests) && Array.isArray(receipt.launches) && receipt.launches.length === 20 && Array.isArray(receipt.closures) && receipt.closures.length === 22, "R2 browser receipt shape/result differs");
  const all = [];
  const expectedScenes = [];
  receipt.per_scene.forEach((scene, index) => {
    exactKeys(scene, ["plan", "samples_ns", "median_denominator", "median_numerator_ns", "p95_ns"], `browser scene ${index + 1}`);
    exactKeys(scene.plan, ["bytes", "expected_counts", "path", "sha256"], `browser scene ${index + 1} plan`);
    exactKeys(scene.plan.expected_counts, ["actions", "actors", "controlled_markers", "hostile_outlines", "protection_rings", "terrain_cells", "terrain_layers"], `browser scene ${index + 1} counts`);
    const name = index === 0 ? "one" : "two";
    const planBytes = readRegular(join(repo, `fixtures/r2/plans/scene_${name}.json`));
    const plan = JSON.parse(planBytes.toString("utf8"));
    const counts = {
      actions: plan.actions.length,
      actors: plan.actors.length,
      controlled_markers: plan.actors.filter((row) => row.controlled_marker === "present").length,
      hostile_outlines: plan.actors.filter((row) => row.hostile_outline === "present").length,
      protection_rings: plan.actors.filter((row) => row.protection_ring === "present").length,
      terrain_cells: plan.terrain_layers.reduce((sum, row) => sum + row.cells.length, 0),
      terrain_layers: plan.terrain_layers.length,
    };
    required(scene.plan.sha256 === Object.values(CONSTANTS.plans)[index] && scene.plan.path === `plans/scene_${name}.json` && scene.plan.bytes === planBytes.length && JSON.stringify(scene.plan.expected_counts) === JSON.stringify(counts), `browser scene ${index + 1} plan/counts differ`);
    expectedScenes.push({ counts, digest: scene.plan.sha256 });
    required(Array.isArray(scene.samples_ns) && scene.samples_ns.length === 10 && scene.samples_ns.every((one) => /^\d+$/.test(one) && BigInt(one) > 0n), `browser scene ${index + 1} raw samples differ`);
    const recomputed = summarize(scene.samples_ns);
    required(scene.median_numerator_ns === recomputed.median_numerator_ns && scene.median_denominator === 2 && scene.p95_ns === recomputed.p95_ns && BigInt(recomputed.p95_ns) <= 5_000_000_000n, `browser scene ${index + 1} timing summary/ceiling differs`);
    all.push(...scene.samples_ns);
  });
  const launchDurations = [[], []];
  const profiles = new Set();
  const requestPaths = [];
  let localOrigin = null;
  receipt.launches.forEach((launch, launchOrdinal) => {
    exactKeys(launch, ["browser_product", "cache_disabled", "chrome_flags", "closure", "console_errors", "elapsed_ns", "exceptions", "frame", "launch_ordinal", "network_negative_control", "profile", "requests", "sample_ordinal", "scene_ordinal", "screenshot", "webgl2"], `browser launch ${launchOrdinal}`);
    const sceneOrdinal = Math.floor(launchOrdinal / 10);
    const sampleOrdinal = launchOrdinal % 10;
    required(launch.launch_ordinal === launchOrdinal && launch.scene_ordinal === sceneOrdinal && launch.sample_ordinal === sampleOrdinal, `browser launch ${launchOrdinal} ordinals differ`);
    required(launch.browser_product === receipt.browser && launch.cache_disabled === true && launch.network_negative_control === "blocked" && launch.webgl2 === true && Array.isArray(launch.console_errors) && launch.console_errors.length === 0 && Array.isArray(launch.exceptions) && launch.exceptions.length === 0, `browser launch ${launchOrdinal} browser facts differ`);
    required(/^\d+$/.test(launch.elapsed_ns) && BigInt(launch.elapsed_ns) > 0n, `browser launch ${launchOrdinal} duration differs`);
    launchDurations[sceneOrdinal].push(launch.elapsed_ns);
    const profileRelative = relative(join(output, "host/tmp"), resolve(launch.profile));
    required(typeof launch.profile === "string" && inside(join(output, "host/tmp"), resolve(launch.profile)) && profileRelative.startsWith("nomos-observed-chrome-") && !profileRelative.includes(sep) && !profiles.has(launch.profile) && !existsSync(launch.profile), `browser launch ${launchOrdinal} did not use one fresh closed profile`);
    profiles.add(launch.profile);
    const expectedFlags = [
      "--headless=new", "--remote-debugging-port=0",
      "--host-resolver-rules=MAP * ~NOTFOUND, EXCLUDE localhost", "--no-first-run",
      "--no-default-browser-check", "--no-sandbox", "--disable-dev-shm-usage",
      "--disable-extensions", "--disable-sync", "--disable-background-networking",
      "--disable-component-update", "--disable-background-timer-throttling",
      "--disable-renderer-backgrounding", "--disable-backgrounding-occluded-windows",
      "--window-size=1280,720", "--force-device-scale-factor=1", "--hide-scrollbars",
      "--use-gl=angle", "--use-angle=swiftshader", "--enable-unsafe-swiftshader",
      `--user-data-dir=${launch.profile}`, "about:blank",
    ];
    required(JSON.stringify(launch.chrome_flags) === JSON.stringify(expectedFlags), `browser launch ${launchOrdinal} Chrome flags/profile differ`);
    exactKeys(launch.frame, ["consequence_counts", "plan_sha256", "viewport"], `browser launch ${launchOrdinal} frame`);
    exactKeys(launch.frame.consequence_counts, ["actions", "actors", "controlled_markers", "hostile_outlines", "protection_rings", "terrain_cells", "terrain_layers"], `browser launch ${launchOrdinal} counts`);
    exactKeys(launch.frame.viewport, ["height", "width"], `browser launch ${launchOrdinal} viewport`);
    required(launch.frame.plan_sha256 === expectedScenes[sceneOrdinal].digest && JSON.stringify(launch.frame.consequence_counts) === JSON.stringify(expectedScenes[sceneOrdinal].counts) && launch.frame.viewport.width === 1280 && launch.frame.viewport.height === 720, `browser launch ${launchOrdinal} frame differs`);
    required(launch.screenshot === (sampleOrdinal === 0 ? `scene_${sceneOrdinal + 1}.png` : null), `browser launch ${launchOrdinal} screenshot differs`);
    exactKeys(launch.closure, ["after_result_ms", "duration_ms", "exit_code", "signal"], `browser launch ${launchOrdinal} closure`);
    required(typeof launch.closure.after_result_ms === "number" && Number.isFinite(launch.closure.after_result_ms) && launch.closure.after_result_ms >= 0 && launch.closure.after_result_ms <= 2_000 && typeof launch.closure.duration_ms === "number" && Number.isFinite(launch.closure.duration_ms) && launch.closure.duration_ms >= 0 && launch.closure.duration_ms <= 2_000 && (launch.closure.exit_code === null || Number.isInteger(launch.closure.exit_code)) && ["SIGTERM", "SIGKILL"].includes(launch.closure.signal), `browser launch ${launchOrdinal} closure exceeded 2 seconds or has invalid identity`);
    const aggregateClosure = receipt.closures[launchOrdinal];
    exactKeys(aggregateClosure, ["after_result_ms", "duration_ms", "exit_code", "kind", "launch_ordinal", "signal"], `browser aggregate closure ${launchOrdinal}`);
    required(JSON.stringify(aggregateClosure) === JSON.stringify({ ...launch.closure, kind: "browser_launch", launch_ordinal: launchOrdinal }), `browser launch ${launchOrdinal} closure aggregate differs`);
    required(Array.isArray(launch.requests) && launch.requests.length > 0, `browser launch ${launchOrdinal} has no raw requests`);
    for (const raw of launch.requests) {
      let url;
      try { url = new URL(raw); } catch { fail(`browser launch ${launchOrdinal} request is not a URL`); }
      required(url.protocol === "http:" && url.hostname === "localhost" && /^\d+$/.test(url.port), `browser launch ${launchOrdinal} made an external request`);
      localOrigin ??= url.origin;
      required(url.origin === localOrigin, `browser launch ${launchOrdinal} changed loopback origin`);
      requestPaths.push(url.pathname);
    }
    required(launch.requests.includes(`${localOrigin}/?scene=${sceneOrdinal}`), `browser launch ${launchOrdinal} exact navigation request is absent`);
  });
  required(JSON.stringify(receipt.chrome_flags) === JSON.stringify(receipt.launches[0].chrome_flags), "browser top-level Chrome flags differ from launch zero");
  for (let index = 0; index < 2; index += 1) required(JSON.stringify(receipt.per_scene[index].samples_ns) === JSON.stringify(launchDurations[index]), `browser scene ${index + 1} aggregate samples differ from launches`);
  const serverRequests = receipt.requests.map((row) => {
    exactKeys(row, ["path", "status"], "browser server request");
    required(typeof row.path === "string" && row.path.startsWith("/") && (row.status === 200 || row.status === 404), "browser server request differs");
    return row.path;
  });
  required(JSON.stringify([...serverRequests].sort()) === JSON.stringify([...requestPaths].sort()), "browser server requests differ from raw launch requests");
  const combined = summarize(all);
  required(receipt.combined.median_numerator_ns === combined.median_numerator_ns && receipt.combined.median_denominator === 2 && receipt.combined.p95_ns === combined.p95_ns && BigInt(combined.p95_ns) <= 5_000_000_000n, "combined browser timing summary/ceiling differs");
  required(Array.isArray(receipt.external_requests) && receipt.external_requests.length === 0, "R2 browser made an external request");
  required(receipt.closures.every((row) => typeof row.after_result_ms === "number" && Number.isFinite(row.after_result_ms) && row.after_result_ms >= 0 && row.after_result_ms <= 2_000 && typeof row.duration_ms === "number" && Number.isFinite(row.duration_ms) && row.duration_ms >= 0 && row.duration_ms <= 2_000), "R2 browser closure exceeded 2 seconds");
  const serverClosure = receipt.closures[20];
  const sheetClosure = receipt.closures[21];
  exactKeys(serverClosure, ["after_result_ms", "duration_ms", "kind", "sockets_destroyed"], "browser server closure");
  exactKeys(sheetClosure, ["after_result_ms", "duration_ms", "exit_code", "kind", "signal"], "browser contact-sheet closure");
  required(serverClosure.kind === "server" && Number.isInteger(serverClosure.sockets_destroyed) && serverClosure.sockets_destroyed >= 0 && sheetClosure.kind === "contact_sheet_browser", "R2 browser non-launch closure identities differ");
  required(Array.isArray(receipt.screenshots) && receipt.screenshots.length === 2 && receipt.contact_sheet, "R2 browser image bindings are incomplete");
  for (const [index, row] of receipt.screenshots.entries()) {
    exactKeys(row, ["bytes", "path", "sha256", "viewport"], `browser screenshot ${index + 1}`);
    exactKeys(row.viewport, ["width", "height"], `browser screenshot ${index + 1} viewport`);
    const bytes = readRegular(join(root, row.path), `browser screenshot ${index + 1}`);
    const dimensions = pngDimensions(bytes, row.path);
    required(row.path === `scene_${index + 1}.png` && row.bytes === bytes.length && row.sha256 === sha256(bytes) && dimensions.width === 1280 && dimensions.height === 720 && row.viewport.width === 1280 && row.viewport.height === 720, `browser screenshot ${index + 1} binding differs`);
  }
  const sheet = readRegular(join(root, receipt.contact_sheet.path), "browser contact sheet");
  exactKeys(receipt.contact_sheet, ["bytes", "path", "sha256", "viewport"], "browser contact sheet receipt");
  exactKeys(receipt.contact_sheet.viewport, ["width", "height"], "browser contact sheet viewport");
  const dimensions = pngDimensions(sheet, "browser contact sheet");
  required(receipt.contact_sheet.path === "contact-sheet.png" && receipt.contact_sheet.bytes === sheet.length && receipt.contact_sheet.sha256 === sha256(sheet) && dimensions.width === 2560 && dimensions.height === 720 && receipt.contact_sheet.viewport.width === 2560 && receipt.contact_sheet.viewport.height === 720, "browser contact-sheet binding differs");
  return { samples_per_scene: 10, combined_p95_ns: combined.p95_ns, closures: 22, screenshots: receipt.screenshots.map(({ path, sha256 }) => ({ path, sha256 })), contact_sheet_sha256: receipt.contact_sheet.sha256 };
};

const validateR1 = (repo, output, ledger, tap, liveChecks) => {
  const root = join(output, "r1");
  const facts = readJson(join(root, "facts.json"), "R1 facts");
  exactKeys(facts, ["outcome", "areas", "commands", "moves", "cost", "chain_head", "unexpected_viewer_test_skips", "external_requests", "wasm"], "R1 facts");
  exactKeys(facts.wasm, ["bytes", "sha256"], "R1 wasm facts");
  required(facts.outcome === "pass" && facts.areas === 6 && facts.commands === 77 && facts.moves === 65 && facts.cost === 95 && facts.chain_head === CONSTANTS.r1_chain_head && facts.unexpected_viewer_test_skips === 0 && facts.external_requests === 0, "accepted R1 facts differ");
  required(tap.tests === 104 && tap.pass === 104 && tap.skipped === 0, "R1 viewer tests are not 104/104 with zero skips");
  const wasm = readRegular(join(root, "wasm/nomos_play.wasm"), "R1 wasm");
  required(facts.wasm.bytes === wasm.length && facts.wasm.sha256 === sha256(wasm) && wasm.subarray(0, 4).toString("hex") === "0061736d", "R1 wasm binding differs");
  if (liveChecks) required(wasm.length === CONSTANTS.r1_wasm_bytes && sha256(wasm) === CONSTANTS.r1_wasm_sha256, "R1 wasm differs from the accepted R1 artifact");
  const firstWasm = readRegular(join(root, "wasm/first-build.wasm"), "first R1 wasm build");
  required(firstWasm.equals(wasm), "the two R1 wasm builds are not byte-identical");
  const wasmLogPath = ledger.find((row) => row.id === "r1-wasm-build").stdout;
  const wasmLog = readText(join(output, wasmLogPath), "R1 wasm build stdout");
  for (const build of [1, 2]) {
    const matches = [...wasmLog.matchAll(new RegExp(`^build_${build}_sha256 ([0-9a-f]{64})$`, "gm"))];
    required(matches.length === 1 && matches[0][1] === facts.wasm.sha256, `R1 wasm build ${build} digest log differs`);
  }
  const wasmMarkers = [...wasmLog.matchAll(/^NOMOS_PLAY_WASM (\S+) bytes=(\d+) sha256=([0-9a-f]{64})$/gm)];
  required(wasmMarkers.length === 2 && wasmMarkers.every((match) => realpathSync(match[1]) === realpathSync(join(repo, "target/wasm32-unknown-unknown/wasm/nomos_play.wasm")) && Number(match[2]) === wasm.length && match[3] === facts.wasm.sha256), "R1 wasm build markers differ from the two artifacts");
  const build = readJson(join(root, "viewer-build.json"), "R1 viewer build receipt");
  exactKeys(build, ["receipt", "generated_by", "node", "files", "total_bytes", "runtime", "semantics", "scanned", "from", "outcome"], "R1 viewer build receipt");
  required(build.receipt === "nomos-viewer-build/1" && build.generated_by === "apps/nomos-viewer/build.mjs" && typeof build.node === "string" && build.node.length > 0 && build.outcome === "pass" && build.from === "target/executable-gaol" && Array.isArray(build.files) && Array.isArray(build.semantics), "R1 viewer build identity/result differs");
  exactKeys(build.runtime, ["path", "bytes", "sha256", "target", "profile", "built_by"], "R1 viewer runtime receipt");
  build.files.forEach((row) => exactKeys(row, ["path", "bytes", "sha256"], "R1 viewer file receipt row"));
  const dist = treeInventory(join(root, "viewer-dist"));
  const buildFiles = build.files.map(({ path, bytes, sha256: digest }) => ({ path, bytes, sha256: digest }))
    .sort((a, b) => Buffer.from(a.path).compare(Buffer.from(b.path)));
  const distBytes = dist.reduce((sum, row) => sum + row.bytes, 0);
  required(JSON.stringify(buildFiles) === JSON.stringify(dist) && build.total_bytes === distBytes && build.scanned === dist.length, "R1 viewer build receipt/tree differs");
  if (liveChecks) required(dist.length === CONSTANTS.r1_viewer_files && distBytes === CONSTANTS.r1_viewer_bytes, "R1 viewer distribution differs from the accepted count/bytes");
  required(readRegular(join(root, "viewer-dist/nomos_play.wasm"), "R1 viewer distribution wasm").equals(wasm), "R1 viewer distribution wasm differs from the two-build artifact");
  const expectedRuntime = { path: "nomos_play.wasm", bytes: wasm.length, sha256: sha256(wasm), target: "wasm32-unknown-unknown", profile: "wasm", built_by: "crates/nomos-play/build-wasm.sh" };
  required(JSON.stringify(build.runtime) === JSON.stringify(expectedRuntime), "R1 viewer runtime receipt differs");
  const semantics = dist.filter((row) => row.path.endsWith(".simulation.json"));
  const receiptSemantics = build.semantics.map((row) => {
    exactKeys(row, ["path", "bytes", "sha256"], "R1 semantics receipt row");
    return row;
  }).sort((a, b) => Buffer.from(a.path).compare(Buffer.from(b.path)));
  required(semantics.length === 6 && JSON.stringify(receiptSemantics) === JSON.stringify(semantics), "R1 viewer semantics receipt does not bind six dist files");
  required(readText(join(root, "viewer-dist.sha256")) === renderManifest(dist), "R1 first distribution inventory differs");
  const cleanBuild = readJson(join(root, "clean-viewer/receipt.json"), "clean R1 viewer build receipt");
  exactKeys(cleanBuild, ["receipt", "generated_by", "node", "files", "total_bytes", "runtime", "semantics", "scanned", "from", "outcome"], "clean R1 viewer build receipt");
  required(Array.isArray(cleanBuild.files) && Array.isArray(cleanBuild.semantics), "clean R1 viewer receipt rows are absent");
  exactKeys(cleanBuild.runtime, ["path", "bytes", "sha256", "target", "profile", "built_by"], "clean R1 viewer runtime receipt");
  cleanBuild.files.forEach((row) => exactKeys(row, ["path", "bytes", "sha256"], "clean R1 viewer file receipt row"));
  const cleanDist = treeInventory(join(root, "clean-viewer/dist"));
  const cleanFiles = cleanBuild.files.map(({ path, bytes, sha256: digest }) => ({ path, bytes, sha256: digest }))
    .sort((a, b) => Buffer.from(a.path).compare(Buffer.from(b.path)));
  const cleanSemantics = cleanBuild.semantics.map((row) => {
    exactKeys(row, ["path", "bytes", "sha256"], "clean R1 semantics receipt row");
    return row;
  }).sort((a, b) => Buffer.from(a.path).compare(Buffer.from(b.path)));
  required(cleanBuild.receipt === "nomos-viewer-build/1" && cleanBuild.generated_by === "apps/nomos-viewer/build.mjs" && typeof cleanBuild.node === "string" && cleanBuild.node.length > 0 && cleanBuild.outcome === "pass" && cleanBuild.from === "target/executable-gaol" && JSON.stringify(cleanFiles) === JSON.stringify(cleanDist) && cleanBuild.total_bytes === cleanDist.reduce((sum, row) => sum + row.bytes, 0) && cleanBuild.scanned === cleanDist.length && JSON.stringify(cleanBuild.runtime) === JSON.stringify(expectedRuntime) && JSON.stringify(cleanSemantics) === JSON.stringify(semantics), "clean R1 viewer receipt/tree differs");
  required(JSON.stringify(cleanDist) === JSON.stringify(dist), "clean R1 viewer distribution differs from the first build");
  required(readText(join(root, "clean-viewer.sha256")) === renderManifest(cleanDist), "clean R1 distribution inventory differs");
  const smoke = readJson(join(root, "viewer-smoke/receipt.json"), "R1 smoke receipt");
  required(smoke.outcome === "pass" && smoke.commit === git(repo, ["rev-parse", "HEAD"]) && smoke.result.areas_cleared === 6 && smoke.result.moves === 65 && smoke.result.cost === 95 && smoke.session.commands === 77 && smoke.session.receipts === 77 && smoke.session.chain_head === CONSTANTS.r1_chain_head, "R1 smoke facts differ");
  required(Array.isArray(smoke.external_requests) && smoke.external_requests.length === 0 && smoke.native_replay?.ok === true && smoke.shutdown?.outcome === "pass", "R1 smoke isolation/replay/closure differs");
  const replay = readText(join(root, "native-replay.stdout"), "R1 native replay");
  required(replay.includes("NOMOS_PLAY_REPLAY PASS") && replay.includes("areas=6") && replay.includes("commands=77") && replay.includes(CONSTANTS.r1_chain_head), "R1 native replay receipt differs");
  const mirror = join(root, "viewer-mirror");
  const mirrorApp = join(mirror, "apps/nomos-viewer");
  required(existsSync(mirrorApp) && lstatSync(mirrorApp).isDirectory(), "R1 output-local viewer mirror is absent");
  const trackedPaths = git(repo, ["ls-files", "apps/nomos-viewer"]).split("\n");
  required(trackedPaths.length === 33, `accepted R1 viewer tracked inventory is ${trackedPaths.length}, expected 33`);
  const mirrorSourceRows = regularTree(mirrorApp).filter((row) => !row.path.startsWith("dist/"));
  const expectedSourceRows = trackedPaths.map((path) => {
    const rel = path.slice("apps/nomos-viewer/".length);
    const bytes = readRegular(join(repo, path));
    return { path: rel, bytes: bytes.length, sha256: sha256(bytes) };
  }).sort((a, b) => Buffer.from(a.path).compare(Buffer.from(b.path)));
  required(JSON.stringify(mirrorSourceRows) === JSON.stringify(expectedSourceRows), "R1 mirror source inventory/bytes differ from all 33 tracked files");
  const recordedSourceRows = trackedPaths.map((path) => {
    const bytes = readRegular(join(repo, path));
    return { path, bytes: bytes.length, sha256: sha256(bytes) };
  }).sort((a, b) => Buffer.from(a.path).compare(Buffer.from(b.path)));
  required(readText(join(root, "viewer-source.sha256"), "R1 source inventory") === renderManifest(recordedSourceRows), "R1 source inventory format/digests differ");
  required(readText(join(root, "viewer-mirror.sha256"), "R1 mirror inventory") === renderManifest(recordedSourceRows), "R1 mirror inventory format/digests differ");
  required(JSON.stringify(treeInventory(join(mirrorApp, "dist"))) === JSON.stringify(dist), "R1 mirror dist differs from copied first dist");
  const mirrorNative = readRegular(join(mirror, "target/debug/nomos-play"), "R1 mirror native runtime");
  required(mirrorNative.length > 0 && mirrorNative.equals(readRegular(join(repo, "target/debug/nomos-play"), "candidate R1 native runtime")), "R1 mirror native runtime differs from the candidate build");
  const mirrorGaol = treeInventory(join(mirror, "target/executable-gaol"));
  required(mirrorGaol.length > 0 && JSON.stringify(mirrorGaol) === JSON.stringify(treeInventory(join(repo, "target/executable-gaol"))), "R1 mirror six-area artifacts differ from the candidate build");
  required(readRegular(join(mirror, "target/wasm32-unknown-unknown/wasm/nomos_play.wasm"), "R1 mirror wasm").equals(firstWasm), "R1 mirror wasm differs from first build");
  required(readRegular(join(repo, "target/wasm32-unknown-unknown/wasm/nomos_play.wasm"), "candidate R1 wasm").equals(firstWasm), "candidate R1 wasm differs from the two-build evidence");
  return { ...facts, viewer_files: dist.length, viewer_bytes: build.total_bytes };
};

export const validateEvidence = ({ repo: repoArgument, output: outputArgument, candidate, liveChecks = true }) => {
  const { repo, output } = validateRoots(repoArgument, outputArgument);
  requireIgnored(repo, output);
  validateCandidate(repo, output, candidate);
  const source = validateSourceBindings(repo, output, candidate);
  const ledger = parseLedger(output);
  const tests = validateComponentLogs(output, ledger);
  const networkIsolation = validateIsolation(output, liveChecks);
  const filesystemIsolation = validateFilesystemIsolation(repo, output, liveChecks);
  const tools = validateTools(output, liveChecks);
  const closure = validateClosure(repo, output, liveChecks);
  const budgets = validateMeasurements(output);
  const r1 = validateR1(repo, output, ledger, tests.r1_viewer_tests, liveChecks);
  const signatures = validateScenes(repo, output);
  const proofBuild = validateBuildReceipt(repo, join(output, "r2/viewer-proof"), CONSTANTS.plans);
  const buildA = validateBuildReceipt(repo, join(output, "r2/viewer-a"), CONSTANTS.plans);
  const buildB = validateBuildReceipt(repo, join(output, "r2/viewer-b"), CONSTANTS.plans);
  required(JSON.stringify(buildA.inventory) === JSON.stringify(buildB.inventory), "two clean R2 distributions are not byte-identical");
  required(JSON.stringify(proofBuild.inventory) === JSON.stringify(buildA.inventory), "R2 smoke distribution differs from the clean builds");
  const compile = validateCompileBenchmark(repo, output);
  const browser = validateBrowser(repo, output);
  return {
    candidate,
    authority: source,
    isolation: { network: networkIsolation, filesystem: filesystemIsolation },
    commands: { count: ledger.length, sha256: sha256(readRegular(join(output, "commands.tsv"))) },
    tests,
    r1,
    r2: { signatures, distribution_bytes: buildA.total, distribution_files: buildA.inventory.length, compile, browser },
    budgets,
    closure: { process: closure.process.outcome, write_boundary: closure.boundary.outcome },
    environment_sha256: sha256(readRegular(join(output, "metadata/environment.txt"))),
    tools,
  };
};

const manifestRows = (output) => regularTree(output).filter((row) => row.path !== "EVIDENCE.sha256" && row.path !== "receipt.json");

const renderManifest = (rows) => `${rows.map((row) => `${row.sha256}  ${row.path}`).join("\n")}\n`;

const parseManifest = (output) => {
  const text = readText(join(output, "EVIDENCE.sha256"), "EVIDENCE.sha256");
  required(text.endsWith("\n") && !text.includes("\r"), "EVIDENCE.sha256 has noncanonical lines");
  const rows = text.slice(0, -1).split("\n").map((line) => {
    const match = /^([0-9a-f]{64})  (.+)$/.exec(line);
    required(match && safeRelative(match[2]) && !["EVIDENCE.sha256", "receipt.json"].includes(match[2]), `invalid evidence manifest row ${line}`);
    return { sha256: match[1], path: match[2] };
  });
  required(new Set(rows.map((row) => row.path)).size === rows.length, "evidence manifest repeats a path");
  const sorted = [...rows].sort((a, b) => Buffer.from(a.path).compare(Buffer.from(b.path)));
  required(JSON.stringify(rows) === JSON.stringify(sorted), "evidence manifest is not byte-path sorted");
  return { text, rows };
};

const validateManifest = (output) => {
  const manifest = parseManifest(output);
  const actual = manifestRows(output);
  required(manifest.rows.length === actual.length, "evidence manifest has a missing or extra path");
  for (let index = 0; index < actual.length; index += 1) required(manifest.rows[index].path === actual[index].path && manifest.rows[index].sha256 === actual[index].sha256, `evidence digest/path drift at ${actual[index]?.path ?? index}`);
  return { files: actual.length, sha256: sha256(Buffer.from(manifest.text)) };
};

const receiptKeys = ["receipt", "outcome", "candidate", "summary", "evidence"];

export const assembleReceipt = ({ repo: repoArgument, output: outputArgument, commit, tree, issue, issueBodySha256, liveChecks = true }) => {
  required(issue === CONSTANTS.issue && issueBodySha256 === CONSTANTS.issue_body_sha256, "assemble authority arguments differ from issue #199");
  const { output } = validateRoots(repoArgument, outputArgument);
  required(!existsSync(join(output, "EVIDENCE.sha256")) && !existsSync(join(output, "receipt.json")), "receipt or evidence manifest already exists");
  const summary = validateEvidence({ repo: repoArgument, output, candidate: { commit, tree }, liveChecks });
  const rows = manifestRows(output);
  writeFileSync(join(output, "EVIDENCE.sha256"), renderManifest(rows), { flag: "wx" });
  const evidence = validateManifest(output);
  const receipt = { receipt: "nomos-r2-complete-proof/1", outcome: "pass", candidate: { commit, tree, issue, issue_body_sha256: issueBodySha256 }, summary, evidence };
  writeFileSync(join(output, "receipt.json"), `${JSON.stringify(receipt, null, 2)}\n`, { flag: "wx" });
  return receipt;
};

export const verifyReceipt = ({ repo: repoArgument, output: outputArgument, liveChecks = true }) => {
  const { output } = validateRoots(repoArgument, outputArgument);
  const receipt = readJson(join(output, "receipt.json"), "receipt.json");
  exactKeys(receipt, receiptKeys, "receipt.json");
  exactKeys(receipt.candidate, ["commit", "tree", "issue", "issue_body_sha256"], "receipt candidate");
  exactKeys(receipt.evidence, ["files", "sha256"], "receipt evidence");
  required(receipt.receipt === "nomos-r2-complete-proof/1" && receipt.outcome === "pass" && receipt.candidate.issue === CONSTANTS.issue && receipt.candidate.issue_body_sha256 === CONSTANTS.issue_body_sha256, "receipt identity/authority differs");
  const evidence = validateManifest(output);
  required(receipt.evidence.files === evidence.files && receipt.evidence.sha256 === evidence.sha256, "receipt evidence-manifest binding differs");
  const summary = validateEvidence({ repo: repoArgument, output, candidate: { commit: receipt.candidate.commit, tree: receipt.candidate.tree }, liveChecks });
  required(JSON.stringify(receipt.summary) === JSON.stringify(summary), "receipt summary differs from recomputed evidence");
  return receipt;
};

const parseCli = (argv) => {
  if (argv[0] === "assemble") {
    const expected = ["--repo", "--output", "--commit", "--tree", "--issue", "--issue-body-sha256"];
    required(argv.length === 13 && expected.every((flag, index) => argv[index * 2 + 1] === flag), "usage: assemble --repo <root> --output <evidence> --commit <40hex> --tree <40hex> --issue 199 --issue-body-sha256 <64hex>");
    return { mode: "assemble", repo: argv[2], output: argv[4], commit: argv[6], tree: argv[8], issue: Number(argv[10]), issueBodySha256: argv[12] };
  }
  if (argv[0] === "verify") {
    required(argv.length === 5 && argv[1] === "--repo" && argv[3] === "--output", "usage: verify --repo <root> --output <evidence>");
    return { mode: "verify", repo: argv[2], output: argv[4] };
  }
  fail("usage: assemble ... | verify --repo <root> --output <evidence>");
};

if (process.argv[1] && resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  try {
    const options = parseCli(process.argv.slice(2));
    if (options.mode === "assemble") assembleReceipt(options);
    else verifyReceipt(options);
    process.stdout.write(`R2_COMPLETE_PROOF_RECEIPT ${options.mode.toUpperCase()} PASS\n`);
  } catch (error) {
    process.stderr.write(`${error.message}\n`);
    process.exitCode = 1;
  }
}
