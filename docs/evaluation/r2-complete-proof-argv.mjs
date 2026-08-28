import { createHash } from "node:crypto";
import { lstatSync, readFileSync, realpathSync } from "node:fs";
import { join, resolve } from "node:path";

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

const fail = (message) => { throw new Error(`r2 complete proof argv: ${message}`); };
const required = (condition, message) => { if (!condition) fail(message); };
const sha256 = (bytes) => createHash("sha256").update(bytes).digest("hex");

const expectedCommandArgv = ({ repo, output }) => {
  const checkout = realpathSync(resolve(repo));
  const evidence = realpathSync(resolve(output));
  const release = join(checkout, "target/r2-complete-release");
  return [
    ["cargo", "fmt", "--all", "--", "--check"],
    ["cargo", "clippy", "--workspace", "--all-targets", "--locked", "--offline", "--", "-D", "warnings"],
    ["cargo", "test", "--workspace", "--locked", "--offline"],
    ["cargo", "xtask", "boundary"],
    ["experiments/executable-gaol/gaol", "verify"],
    ["r1_wasm_build"],
    ["cargo", "build", "--locked", "--offline", "-p", "nomos-play"],
    ["r1_mirror"],
    ["r1_viewer_build"],
    ["r1_viewer_tests"],
    ["env", `NOMOS_PLAY_BIN=${checkout}/target/debug/nomos-play`, `NOMOS_PLAY_AREAS=${checkout}/target/executable-gaol/areas`, "node", "apps/nomos-viewer/smoke/smoke.mjs", "--dist", `${evidence}/r1/viewer-dist`, "--out", `${evidence}/r1/viewer-smoke`, "--require-chrome"],
    ["target/debug/nomos-play", "replay", "target/executable-gaol/areas", "--session", `${evidence}/r1/viewer-smoke/session.json`],
    ["r1_facts"],
    ["docs/evaluation/r2-schema-ownership.sh"],
    ["env", `R2_SCHEMA_PLANTS_PARENT=${evidence}/host/tmp`, "docs/evaluation/r2-schema-ownership-plants.sh"],
    ["docs/evaluation/r2-source-provenance.sh"],
    ["docs/evaluation/r2-source-provenance.test.sh"],
    ["docs/evaluation/r2-adopter-neutrality.sh"],
    ["docs/evaluation/r2-adopter-neutrality.test.sh"],
    ["node", "docs/evaluation/r2-maximum.test.mjs"],
    ["r2_compiler_tests"],
    ["compile_scene_ten", "fixtures/r2/scenes/scene_one.json", `${evidence}/r2/scene-a`, "fixtures/r2/plans/scene_one.json"],
    ["compile_scene_ten", "fixtures/r2/scenes/scene_two.json", `${evidence}/r2/scene-b`, "fixtures/r2/plans/scene_two.json"],
    ["r2_signatures"],
    ["r2_viewer_tests"],
    ["node", "apps/nomos-observed-viewer/build.mjs", "--plan", "fixtures/r2/plans/scene_one.json", "--plan", "fixtures/r2/plans/scene_two.json", "--out", `${evidence}/r2/viewer-proof/dist`, "--receipt", `${evidence}/r2/viewer-proof/receipt.json`],
    ["node", "apps/nomos-observed-viewer/smoke/smoke.mjs", "--dist", `${evidence}/r2/viewer-proof/dist`, "--out", `${evidence}/r2/browser-smoke`, "--samples", "10"],
    ["/usr/bin/time", "-v", "-o", `${evidence}/measurements/clean-release-time.txt`, "env", `CARGO_TARGET_DIR=${release}`, "cargo", "build", "--workspace", "--release", "--locked", "--offline"],
    ["clean_r1_viewer_build"],
    ["build_r2_viewer", `${evidence}/r2/viewer-a`],
    ["build_r2_viewer", `${evidence}/r2/viewer-b`],
    ["compare_r2_viewers"],
    ["node", "docs/evaluation/measure-r2-compile.mjs", "--binary", `${release}/release/nomos-observed-scene`, "--fixture", "fixtures/r2/maximum-observed-scene.json", "--output", `${evidence}/r2/compile-benchmark`],
  ];
};

const readRegular = (path) => {
  let info;
  try { info = lstatSync(path); } catch (error) { fail(`${path} is unreadable: ${error.message}`); }
  required(info.isFile() && !info.isSymbolicLink(), `${path} is not one regular file`);
  let canonical;
  try { canonical = realpathSync(path); } catch (error) { fail(`${path} is not canonical: ${error.message}`); }
  required(canonical === path, `${path} traverses a symlink`);
  return readFileSync(path);
};

const exactKeys = (value, expected, label) => {
  required(value && typeof value === "object" && !Array.isArray(value), `${label} is not an object`);
  required(JSON.stringify(Object.keys(value).sort()) === JSON.stringify([...expected].sort()), `${label} fields differ`);
};

export { expectedCommandArgv };

export const validateCommandArgv = ({ repo, output, commandRows }) => {
  required(Array.isArray(commandRows), "commands.tsv rows are not an array");
  const path = join(realpathSync(resolve(output)), "commands.argv.ndjson");
  const bytes = readRegular(path);
  const text = bytes.toString("utf8");
  required(text.endsWith("\n") && !text.includes("\r"), "commands.argv.ndjson has noncanonical lines");
  const lines = text.slice(0, -1).split("\n");
  const expected = expectedCommandArgv({ repo, output });
  required(expected.length === COMMAND_IDS.length && commandRows.length === expected.length,
    `argv command count is ${lines.length}, expected ${expected.length}`);
  required(lines.length === expected.length, `argv row count is ${lines.length}, expected ${expected.length}`);
  const ordinals = new Set();
  const ids = new Set();
  for (const [index, line] of lines.entries()) {
    let record;
    try { record = JSON.parse(line); } catch (error) { fail(`argv row ${index + 1} is invalid JSON: ${error.message}`); }
    exactKeys(record, ["ordinal", "command_id", "argv"], `argv row ${index + 1}`);
    required(JSON.stringify(record) === line, `argv row ${index + 1} is not canonical JSON`);
    const ordinal = index + 1;
    const id = COMMAND_IDS[index];
    required(Number.isSafeInteger(record.ordinal) && typeof record.command_id === "string",
      `argv row ${ordinal} identity is invalid`);
    required(!ordinals.has(record.ordinal) && !ids.has(record.command_id), `argv row ${ordinal} is a duplicate`);
    ordinals.add(record.ordinal);
    ids.add(record.command_id);
    required(record.ordinal === ordinal, `argv row ${ordinal} ordinal differs`);
    required(record.command_id === id, `argv row ${ordinal} command id differs`);
    required(Array.isArray(record.argv) && record.argv.length > 0 &&
      record.argv.every((argument) => typeof argument === "string" && !/[\0\r\n]/.test(argument)),
    `argv row ${ordinal} vector is invalid`);
    const tsv = commandRows[index];
    required(tsv && tsv.ordinal === ordinal && tsv.id === id &&
      record.ordinal === tsv.ordinal && record.command_id === tsv.id,
    `argv row ${ordinal} does not bind commands.tsv`);
    required(JSON.stringify(record.argv) === JSON.stringify(expected[index]),
      `argv row ${ordinal} differs from expected command argv`);
  }
  required(ordinals.size === expected.length && ids.size === expected.length, "argv ledger has duplicate or extra rows");
  return Object.freeze({ path, count: lines.length, sha256: sha256(bytes) });
};
