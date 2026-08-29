import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import { execFileSync } from "node:child_process";
import {
  cpSync,
  mkdirSync,
  mkdtempSync,
  readFileSync,
  readdirSync,
  realpathSync,
  rmSync,
  statSync,
  writeFileSync,
} from "node:fs";
import { tmpdir } from "node:os";
import { dirname, join, relative, resolve } from "node:path";
import test, { before } from "node:test";
import { fileURLToPath } from "node:url";

import { expectedCommandArgv } from "./r2-complete-proof-argv.mjs";
import { assembleReceipt, verifyReceipt } from "./r2-complete-proof-receipt.mjs";
import {
  RAW_HEADER,
  XFS_MAGIC,
  accountingFromStatfs,
  filesystemIdentityDocument,
  publicTsvFromRaw,
  rawRow,
} from "./r2-filesystem-accounting.mjs";
import {
  DU_ARGV,
  EVIDENCE_SCHEMA,
  RESERVATION_LENGTH_BYTES,
  filesystemSnapshotDocument,
  summarizeFilesystemEvidence,
} from "./r2-filesystem-evidence.mjs";

const here = dirname(fileURLToPath(import.meta.url));
const sourceRepo = resolve(here, "../..");
const temporary = mkdtempSync(join(tmpdir(), "nomos-r2-receipt-test-"));
const repo = join(temporary, "repo");
const template = join(repo, "target/template");
const fixtureOutput = join(repo, "target/receipt-test-output");
let fixtureSnapshot;
const fixtureSourcePaths = [
  ".gitignore",
  "R2.md",
  "RUNTIME.md",
  "apps/nomos-viewer",
  "apps/nomos-observed-viewer",
  "fixtures/r2",
  "docs/decisions/0025-r2-filesystem-accounting.md",
  "docs/evaluation/r2-second-scene-packet/MANIFEST.sha256",
  "docs/evaluation/runs/r2/2026-08-27-issue-197-second-author/SCENE_SIGNATURES.json",
  "docs/evaluation/runs/r2/2026-08-27-issue-197-second-author/evidence/contact-sheet.png",
];
const sha256 = (bytes) => createHash("sha256").update(bytes).digest("hex");
const issueBody = "0a701b4238fd6b7f23ba0ae40022bc7c23ca450ad1a8f0febc05ab440f6b3c88";
const filesystemUuid = "11111111-2222-3333-4444-555555555555";
const filesystemDevice = "/dev/loop7";
const filesystemMajorMinor = "7:7";
const filesystemOrigin = 10_000_000_000_000_000n;
const filesystemPeriod = 50_000_000n;
const plans = {
  scene_one: "717b91f3f35d815bfa9f9cc777b38f8a091f7a6339d786c57360e94ffe4c7699",
  scene_two: "1fd08cfb33d07f93a568e4bb337ebfbe8909a22a973d0f137139c92f0481e905",
};
const signatures = {
  scene_one: "ef11771f3f8c210fdd8c9366e780ab720349a49dad88ae8dca969fcbe16c30d2",
  scene_two: "9afb46dc4d7ddb5b79cdcabd63b67d162b3c230aecfde9383e038128572f0f3d",
};
const chain = "43a1b2164f18bc54738d0402013419659576e2d866c3fca630321a2ca641f143";

const commandIds = [
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
];
const commandDisplays = [
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
  "node --test apps/nomos-observed-viewer/test/*.test.mjs docs/evaluation/r2-scene-signature.test.mjs docs/evaluation/r2-complete-proof-process.test.mjs docs/evaluation/r2-complete-proof-receipt.test.mjs docs/evaluation/r2-complete-proof-xfs-evidence.test.mjs docs/evaluation/r2-complete-proof-xfs-receipt.test.mjs docs/evaluation/r2-filesystem-accounting.test.mjs docs/evaluation/r2-filesystem-evidence.test.mjs; docs/evaluation/r2-complete-proof.test.sh",
  "node apps/nomos-observed-viewer/build.mjs --plan scene_one --plan scene_two --out <output>/r2/viewer-proof/dist --receipt <output>/r2/viewer-proof/receipt.json",
  "node apps/nomos-observed-viewer/smoke/smoke.mjs --dist <output>/r2/viewer-proof/dist --out <output>/r2/browser-smoke --samples 10",
  "LC_ALL=C /usr/bin/time -v cargo build --workspace --release --locked --offline (fresh target)",
  "clean R1 viewer build and byte comparison with proof distribution",
  "clean R2 viewer build A",
  "clean R2 viewer build B",
  "compare full regular-file inventories for all clean R2 distributions",
  "node docs/evaluation/measure-r2-compile.mjs --binary <fresh-release>/nomos-observed-scene --fixture maximum --output <output>/r2/compile-benchmark",
];
const toolLabels = [
  "git", "realpath", "readlink", "find", "grep", "awk", "sed", "sort", "cmp", "cut",
  "sha256sum", "stat", "date", "du", "jq", "gnu-time", "fallocate", "sync", "unlink", "ar", "basename", "bash", "bwrap",
  "cargo", "cc", "chmod", "cp", "diff", "dirname", "env", "getconf", "head", "id",
  "install", "ionice", "ip", "ld", "ln", "mkdir", "mktemp", "mv", "node", "paste", "ps", "rm", "rustc",
  "rustup", "seq", "setpriv", "setsid", "sh", "sleep", "strings", "sudo", "tar", "taskset", "timeout", "touch",
  "tr", "uname", "unshare", "wc", "cargo-toolchain", "rustc-toolchain", "rust-lld", "chrome",
];

const json = (path, value) => writeFileSync(path, `${JSON.stringify(value, null, 2)}\n`);
const mkdir = (path) => mkdirSync(path, { recursive: true });
const inventory = (root) => {
  const rows = [];
  const walk = (directory) => {
    for (const entry of readdirSync(directory, { withFileTypes: true })) {
      const path = join(directory, entry.name);
      if (entry.isDirectory()) walk(path);
      else {
        const bytes = readFileSync(path);
        rows.push({ path: relative(root, path).split("\\").join("/"), bytes: bytes.length, sha256: sha256(bytes) });
      }
    }
  };
  walk(root);
  return rows.sort((a, b) => Buffer.from(a.path).compare(Buffer.from(b.path)));
};
const manifest = (rows) => `${rows.map((row) => `${row.sha256}  ${row.path}`).join("\n")}\n`;

const png = (width, height) => {
  const bytes = Buffer.alloc(24);
  Buffer.from("89504e470d0a1a0a", "hex").copy(bytes);
  bytes.write("IHDR", 12, "ascii");
  bytes.writeUInt32BE(width, 16);
  bytes.writeUInt32BE(height, 20);
  return bytes;
};

const tap = (tests) => `TAP version 13\n1..${tests}\n# tests ${tests}\n# suites 0\n# pass ${tests}\n# fail 0\n# cancelled 0\n# skipped 0\n# todo 0\n# duration_ms 1\n`;

const makeFilesystemIdentity = (checkout, output, statfs = {}) => {
  const accounting = accountingFromStatfs({
    type: XFS_MAGIC,
    bsize: 4096n,
    blocks: 1_000_000n,
    bfree: 875_000n,
    bavail: 874_000n,
    ...statfs,
  }, { fragmentSize: 4096n });
  return {
    schema: 1,
    mountId: 32n,
    mountpoint: checkout,
    mountRoot: "/checkout",
    mountOptions: "ro,nodev,nosuid,relatime",
    source: filesystemDevice,
    majorMinor: filesystemMajorMinor,
    mountFilesystemType: "xfs",
    uuid: filesystemUuid,
    sourceDevice: filesystemDevice,
    sourceMajorMinor: filesystemMajorMinor,
    filesystemType: XFS_MAGIC,
    filesystemMagic: "0x58465342",
    rootDevice: 7007n,
    rootInode: 123n,
    fragmentSize: 4096n,
    blockSize: 4096n,
    capacityBlocks: accounting.blocks,
    capacityBytes: accounting.capacityBytes,
    dedicatedFixedCapacity: true,
    targetPath: join(checkout, "target"),
    outputPath: output,
    nestedMounts: [
      { mountId: 33n, mountpoint: join(checkout, "target"), root: "/checkout/target", mountOptions: "rw,nodev,nosuid,relatime", source: filesystemDevice, majorMinor: filesystemMajorMinor, filesystemType: "xfs", uuid: filesystemUuid },
      { mountId: 34n, mountpoint: output, root: `/checkout/${relative(checkout, output).split("\\").join("/")}`, mountOptions: "rw,nodev,nosuid,relatime", source: filesystemDevice, majorMinor: filesystemMajorMinor, filesystemType: "xfs", uuid: filesystemUuid },
    ],
    accounting,
  };
};

const makeFilesystemEvidence = () => {
  const checkout = repo;
  const output = template;
  const target = join(checkout, "target");
  const identity = makeFilesystemIdentity(checkout, output);
  const after = makeFilesystemIdentity(checkout, output, { bfree: 879_096n, bavail: 878_096n });
  const rows = [
    rawRow({ ordinal: 0n, sampleStartNs: filesystemOrigin, elapsedNs: 0n, deadlineNs: filesystemOrigin, identity, kind: "scheduled" }),
    rawRow({ ordinal: 1n, sampleStartNs: filesystemOrigin + filesystemPeriod, elapsedNs: filesystemPeriod, deadlineNs: filesystemOrigin + filesystemPeriod, identity, kind: "terminal" }),
  ];
  const du = (phase) => ({
    schema: EVIDENCE_SCHEMA,
    mode: "du-check",
    phase,
    invocation: {
      argv: [...DU_ARGV, checkout], cwd: checkout, status: 0,
      stdout: `489\t${checkout}\n`, stderr: "", started_ns: "10", ended_ns: "20",
    },
    du_mib: "489",
    snapshot: filesystemSnapshotDocument(identity),
  });
  const evidence = {
    identity: JSON.parse(filesystemIdentityDocument(identity, {
      samplerOriginNs: filesystemOrigin,
      nominalIntervalNs: filesystemPeriod,
    })),
    raw: `${RAW_HEADER}\n${rows.join("\n")}\n`,
    public: publicTsvFromRaw(rows),
    setupDu: du("setup"),
    shutdownDu: du("shutdown"),
    finalization: {
      schema: EVIDENCE_SCHEMA,
      mode: "release-check",
      reservation: {
        path: `${output}/host/finalization.reserve`,
        length_bytes: RESERVATION_LENGTH_BYTES.toString(),
        allocated_bytes: RESERVATION_LENGTH_BYTES.toString(),
      },
      a_before_bytes: identity.accounting.usedBytes.toString(),
      a_after_bytes: after.accounting.usedBytes.toString(),
      snapshot: filesystemSnapshotDocument(after),
    },
    stop: `${filesystemOrigin + 25_000_000n}\n`,
  };
  const summary = summarizeFilesystemEvidence(evidence, {
    checkout, target, output, device: filesystemDevice,
    majorMinor: filesystemMajorMinor, uuid: filesystemUuid,
    fragmentSize: 4096n, nominalIntervalNs: filesystemPeriod,
  });
  return { ...evidence, summary };
};

const buildR2 = (name) => {
  const directory = join(template, `r2/${name}`);
  execFileSync(process.execPath, [
    "apps/nomos-observed-viewer/build.mjs",
    "--plan", "fixtures/r2/plans/scene_one.json",
    "--plan", "fixtures/r2/plans/scene_two.json",
    "--out", join(directory, "dist"),
    "--receipt", join(directory, "receipt.json"),
  ], { cwd: repo, stdio: "ignore" });
};

const refreshR2Receipt = (output, name) => {
  const directory = join(output, `r2/${name}`);
  const path = join(directory, "receipt.json");
  const receipt = JSON.parse(readFileSync(path));
  receipt.files = inventory(join(directory, "dist"));
  receipt.total_bytes = receipt.files.reduce((sum, row) => sum + row.bytes, 0);
  json(path, receipt);
};

const makeTemplate = () => {
  mkdir(join(template, "metadata"));
  mkdir(join(template, "logs"));
  mkdir(join(template, "measurements"));
  mkdir(join(template, "host"));
  const commit = execFileSync("git", ["-C", repo, "rev-parse", "HEAD"], { encoding: "utf8" }).trim();
  const tree = execFileSync("git", ["-C", repo, "rev-parse", "HEAD^{tree}"], { encoding: "utf8" }).trim();
  json(join(template, "metadata/source-tree.json"), {
    outcome: "pass", commit, tree, issue: 199, issue_body_sha256: issueBody,
    r2_contract_sha256: "625f4bb1ea7c7400a6717c14b51cc6da51b32421e49bba98cf3d7ed9ff4a1254",
    r2_revision_3_authority_sha256: "a6a50bca56c4a990b44968ffefc31103a88e48b52904728693a166ba0d66d3ae",
    runtime_contract_sha256: "dd6f4b2ce48557f48df61d50cdc25b4ebaf0904331f4fd78d804e3af536db593",
    catalog_sha256: "6259520fbf318ae0393ea4ae69649864acb154db4034d081435416be2ffa9323",
    packet_manifest_sha256: "d5708087cf7967a420667c56a7b02ed052b7058ed8545af06e6771170003c948",
    committed_contact_sheet_sha256: "b76edbd9dd03fce5a99c074200ee7311bf87d5d2e5829c800170c129d00bf576",
    plan_sha256: plans, scene_signature_sha256: signatures,
  });
  for (const name of ["clean-start", "clean-end"]) json(join(template, `metadata/${name}.json`), { outcome: "pass", commit, tree, porcelain: "" });
  json(join(template, "metadata/isolation.json"), { outcome: "pass", namespace: "fresh", pid_namespace: "fresh", external_negative_control: "blocked", loopback_only: true });
  json(join(template, "metadata/ip-address.json"), [{ ifname: "lo", link_type: "loopback", flags: ["LOOPBACK", "UP"], addr_info: [{ family: "inet", local: "127.0.0.1", prefixlen: 8, scope: "host" }, { family: "inet6", local: "::1", prefixlen: 128, scope: "host" }] }]);
  json(join(template, "metadata/ip-route-v4.json"), [
    { type: "local", dst: "127.0.0.0/8", dev: "lo", table: "local", protocol: "kernel", scope: "host", prefsrc: "127.0.0.1" },
    { type: "local", dst: "127.0.0.1", dev: "lo", table: "local", protocol: "kernel", scope: "host", prefsrc: "127.0.0.1" },
    { type: "broadcast", dst: "127.255.255.255", dev: "lo", table: "local", protocol: "kernel", scope: "link", prefsrc: "127.0.0.1" },
  ]);
  json(join(template, "metadata/ip-route-v6.json"), [{ type: "local", dst: "::1", dev: "lo", table: "local", protocol: "kernel" }]);
  json(join(template, "metadata/network-control.json"), {
    outcome: "pass", destination: "1.1.1.1:53",
    outer_positive: { outcome: "connected", exit_code: 0, stdout: "connected\n", stderr: "" },
    inner_negative: { outcome: "blocked", exit_code: 1, stdout: "", stderr: "network unreachable" },
  });
  writeFileSync(join(template, "metadata/network-outer-positive.stdout"), "connected\n");
  writeFileSync(join(template, "metadata/network-outer-positive.stderr"), "");
  writeFileSync(join(template, "metadata/network-inner-negative.stdout"), "");
  writeFileSync(join(template, "metadata/network-inner-negative.stderr"), "network unreachable");
  writeFileSync(join(template, "metadata/xfs-shell-validation.stdout"), "r2-complete-proof-xfs shell validation tests: PASS\n");
  writeFileSync(join(template, "metadata/xfs-shell-validation.stderr"), "");
  writeFileSync(join(template, "metadata/xfs-shell-validation.status"), "0\n");
  json(join(template, "metadata/xfs-shell-validation.argv.json"), {
    argv: ["/usr/bin/bash", join(repo, "docs/evaluation/r2-complete-proof-xfs.test.sh")],
    cwd: repo,
    proof_token: "1".repeat(64),
  });
  json(join(template, "metadata/xfs-shell-validation.candidate.json"), {
    outcome: "pass", commit, tree, porcelain: "",
  });
  json(join(template, "metadata/filesystem-isolation.json"), {
    outcome: "pass", mechanism: "bubblewrap", repository_mount: "read-only",
    writable_roots: ["target/template", "target"],
    negative_control: { path: "README.md", operation: "append", exit_code: 1, stdout: "", stderr: "Read-only file system" },
  });
  writeFileSync(join(template, "metadata/read-only-negative-control.stdout"), "");
  writeFileSync(join(template, "metadata/read-only-negative-control.stderr"), "Read-only file system");
  writeFileSync(join(template, "metadata/mountinfo.txt"), [
    "1 0 0:1 / / ro - none none ro",
    `2 1 0:2 / ${join(repo, "target")} rw - none none rw`,
    `3 2 0:3 / ${template} rw - none none rw`,
    "",
  ].join("\n"));
  json(join(template, "metadata/process-closure.json"), { outcome: "pass", checked_while_sampler: true, checked_after_sampler: true, leaked_processes: [], namespace_children_before_sampler_stop: [], namespace_children: [] });
  writeFileSync(join(template, "metadata/namespace-children-before-sampler-stop.txt"), "");
  writeFileSync(join(template, "metadata/namespace-children.txt"), "");
  json(join(template, "metadata/write-boundary.json"), { outcome: "pass", output_relative: "target/template", allowed_roots: ["target/template", "target"], outside_writes: [], inputs_unchanged: true });
  writeFileSync(join(template, "metadata/environment.txt"), "LC_ALL=C\nCARGO_NET_OFFLINE=true\n");
  const toolPath = realpathSync(process.execPath);
  const toolDigest = sha256(readFileSync(toolPath));
  writeFileSync(join(template, "metadata/tools.txt"), `tool\tpath\tsha256\n${toolLabels.map((label) => `${label}\t${toolPath}\t${toolDigest}`).join("\n")}\n`);
  writeFileSync(join(template, "metadata/tool-versions.txt"), `${["git", "bash", "rustc", "cargo", "rustup", "node", "jq", "bubblewrap", "cc", "ld", "chrome"].map((key) => `${key}=test`).join("\n")}\n`);

  const wasm = Buffer.from("0061736d01000000", "hex");
  const wasmDigest = sha256(wasm);
  const ledger = ["ordinal\tcommand_id\tstarted_ns\tended_ns\texit_code\tstdout_path\tstderr_path\tcommand"];
  for (const [index, id] of commandIds.entries()) {
    const prefix = String(index + 1).padStart(2, "0");
    const stdout = `logs/${prefix}-${id}.stdout`;
    const stderr = `logs/${prefix}-${id}.stderr`;
    let content = "PASS\n";
    if (id === "workspace-boundary") content = "boundary: clean\n";
    if (id === "r1-gaol-verify") content = tap(1);
    if (id === "r1-wasm-build") content = `NOMOS_PLAY_WASM ${join(repo, "target/wasm32-unknown-unknown/wasm/nomos_play.wasm")} bytes=${wasm.length} sha256=${wasmDigest}\nNOMOS_PLAY_WASM ${join(repo, "target/wasm32-unknown-unknown/wasm/nomos_play.wasm")} bytes=${wasm.length} sha256=${wasmDigest}\nbuild_1_sha256 ${wasmDigest}\nbuild_2_sha256 ${wasmDigest}\n`;
    if (id === "r1-viewer-build" || id === "clean-r1-viewer-build") content = "NOMOS_VIEWER_BUILD PASS\n";
    if (id === "r1-viewer-tests") content = tap(104);
    if (id === "r1-browser-smoke") content = "NOMOS_VIEWER_SMOKE PASS areas=6 moves=65 cost=95 requests=1 external=0\n";
    if (id === "r1-native-replay") content = `NOMOS_PLAY_REPLAY PASS areas=6 commands=77 receipts=77 chain=${chain}\n`;
    if (id === "r2-viewer-tests") content = `${tap(3)}# includes docs/evaluation/r2-complete-proof-process.test.mjs\nR2_COMPLETE_PROOF_PLANTS PASS\n`;
    if (id === "r2-schema-ownership") content = "R2_SCHEMA_OWNERSHIP PASS\n";
    if (id === "r2-schema-plants") content = "expected refusal: missing\nexpected refusal: duplicate\nexpected refusal: third\n";
    if (id === "r2-source-provenance") content = "R2_SOURCE_PROVENANCE PASS\n";
    if (id === "r2-source-provenance-plants") content = "R2_SOURCE_PROVENANCE_PLANTS PASS\n";
    if (id === "r2-adopter-neutrality") content = "R2_ADOPTER_NEUTRALITY PASS\n";
    if (id === "r2-adopter-neutrality-plants") content = "R2_ADOPTER_NEUTRALITY_PLANTS PASS\n";
    if (id === "r2-maximum-fixture") content = "r2 maximum: 98421 bytes fe332f711437dab15e4d1315cc3ca57dba6521350ff673941e77feb414585909\n";
    if (id === "r2-compiler-tests") content = "R2_SECOND_SCENE_PACKET_PLANTS PASS\n";
    if (["r2-viewer-build", "clean-r2-viewer-build-a", "clean-r2-viewer-build-b"].includes(id)) content = "NOMOS_OBSERVED_BUILD PASS\n";
    if (id === "r2-browser-smoke") content = "NOMOS_OBSERVED_SMOKE PASS scenes=2 samples=20 external=0\n";
    if (id === "maximum-compile-benchmark") content = "r2 compile latency: median 20000000/2 ns; p95 10000000 ns; PASS\n";
    writeFileSync(join(template, stdout), content);
    writeFileSync(join(template, stderr), "");
    ledger.push(`${index + 1}\t${id}\t${1000 + index}\t${1001 + index}\t0\t${stdout}\t${stderr}\t${commandDisplays[index]}`);
  }
  writeFileSync(join(template, "commands.tsv"), `${ledger.join("\n")}\n`);
  const argvRows = expectedCommandArgv({ repo, output: template }).map((argv, index) =>
    JSON.stringify({ ordinal: index + 1, command_id: commandIds[index], argv }));
  writeFileSync(join(template, "commands.argv.ndjson"), `${argvRows.join("\n")}\n`);

  writeFileSync(join(template, "measurements/clean-release-time.txt"), "\tElapsed (wall clock) time (h:mm:ss or m:ss): 0:12.34\n\tExit status: 0\n");
  const filesystemEvidence = makeFilesystemEvidence();
  const filesystemDirectory = join(template, "measurements/filesystem");
  mkdir(filesystemDirectory);
  writeFileSync(join(filesystemDirectory, "identity.json"), `${JSON.stringify(filesystemEvidence.identity, null, 2)}\n`);
  writeFileSync(join(filesystemDirectory, "raw.tsv"), filesystemEvidence.raw);
  writeFileSync(join(filesystemDirectory, "public.tsv"), filesystemEvidence.public);
  json(join(filesystemDirectory, "du-setup.json"), filesystemEvidence.setupDu);
  json(join(filesystemDirectory, "du-shutdown.json"), filesystemEvidence.shutdownDu);
  json(join(filesystemDirectory, "release.json"), filesystemEvidence.finalization);
  writeFileSync(join(filesystemDirectory, "stop"), filesystemEvidence.stop);
  json(join(filesystemDirectory, "summary.json"), filesystemEvidence.summary);

  mkdir(join(template, "r1/wasm"));
  writeFileSync(join(template, "r1/wasm/nomos_play.wasm"), wasm);
  writeFileSync(join(template, "r1/wasm/first-build.wasm"), wasm);
  mkdir(join(template, "r1/viewer-dist"));
  writeFileSync(join(template, "r1/viewer-dist/index.html"), "<!doctype html>\n");
  writeFileSync(join(template, "r1/viewer-dist/nomos_play.wasm"), wasm);
  for (const area of ["cistern-walk", "drowned-stair", "ember-vault", "gloam-bastion", "north-gaol", "ossuary-reach"]) {
    mkdir(join(template, "r1/viewer-dist/areas"));
    writeFileSync(join(template, `r1/viewer-dist/areas/${area}.simulation.json`), "{}\n");
  }
  const r1Files = inventory(join(template, "r1/viewer-dist"));
  const r1Bytes = r1Files.reduce((sum, row) => sum + row.bytes, 0);
  const r1Runtime = { path: "nomos_play.wasm", bytes: wasm.length, sha256: wasmDigest, target: "wasm32-unknown-unknown", profile: "wasm", built_by: "crates/nomos-play/build-wasm.sh" };
  const r1Semantics = r1Files.filter((row) => row.path.endsWith(".simulation.json"));
  const r1Receipt = { receipt: "nomos-viewer-build/1", generated_by: "apps/nomos-viewer/build.mjs", node: process.version, files: r1Files, total_bytes: r1Bytes, runtime: r1Runtime, semantics: r1Semantics, scanned: r1Files.length, from: "target/executable-gaol", outcome: "pass" };
  json(join(template, "r1/viewer-build.json"), r1Receipt);
  writeFileSync(join(template, "r1/viewer-dist.sha256"), manifest(r1Files));
  mkdir(join(template, "r1/clean-viewer/dist"));
  cpSync(join(template, "r1/viewer-dist"), join(template, "r1/clean-viewer/dist"), { recursive: true });
  json(join(template, "r1/clean-viewer/receipt.json"), r1Receipt);
  writeFileSync(join(template, "r1/clean-viewer.sha256"), manifest(r1Files));
  mkdir(join(template, "r1/viewer-smoke"));
  json(join(template, "r1/viewer-smoke/receipt.json"), {
    outcome: "pass", commit, result: { areas_cleared: 6, moves: 65, cost: 95 },
    session: { commands: 77, receipts: 77, chain_head: chain }, external_requests: [],
    native_replay: { ok: true }, shutdown: { outcome: "pass" },
  });
  writeFileSync(join(template, "r1/native-replay.stdout"), `NOMOS_PLAY_REPLAY PASS areas=6 commands=77 chain_head=${chain}\n`);
  json(join(template, "r1/facts.json"), { outcome: "pass", areas: 6, commands: 77, moves: 65, cost: 95, chain_head: chain, unexpected_viewer_test_skips: 0, external_requests: 0, wasm: { bytes: wasm.length, sha256: sha256(wasm) } });
  const tracked = execFileSync("git", ["-C", repo, "ls-files", "apps/nomos-viewer"], { encoding: "utf8" }).trimEnd().split("\n");
  const sourceRows = tracked.map((path) => {
    const bytes = readFileSync(join(repo, path));
    return { path, bytes: bytes.length, sha256: sha256(bytes) };
  }).sort((left, right) => Buffer.from(left.path).compare(Buffer.from(right.path)));
  writeFileSync(join(template, "r1/viewer-source.sha256"), manifest(sourceRows));
  writeFileSync(join(template, "r1/viewer-mirror.sha256"), manifest(sourceRows));
  for (const path of tracked) {
    const destination = join(template, "r1/viewer-mirror", path);
    mkdir(dirname(destination));
    cpSync(join(repo, path), destination);
  }
  mkdir(join(template, "r1/viewer-mirror/apps/nomos-viewer/dist"));
  cpSync(join(template, "r1/viewer-dist"), join(template, "r1/viewer-mirror/apps/nomos-viewer/dist"), { recursive: true });
  mkdir(join(repo, "target/debug"));
  writeFileSync(join(repo, "target/debug/nomos-play"), "binary\n");
  mkdir(join(template, "r1/viewer-mirror/target/debug"));
  cpSync(join(repo, "target/debug/nomos-play"), join(template, "r1/viewer-mirror/target/debug/nomos-play"));
  mkdir(join(repo, "target/executable-gaol"));
  writeFileSync(join(repo, "target/executable-gaol/areas.json"), "{}\n");
  mkdir(join(template, "r1/viewer-mirror/target/executable-gaol"));
  cpSync(join(repo, "target/executable-gaol"), join(template, "r1/viewer-mirror/target/executable-gaol"), { recursive: true });
  mkdir(join(repo, "target/wasm32-unknown-unknown/wasm"));
  writeFileSync(join(repo, "target/wasm32-unknown-unknown/wasm/nomos_play.wasm"), wasm);
  mkdir(join(template, "r1/viewer-mirror/target/wasm32-unknown-unknown/wasm"));
  writeFileSync(join(template, "r1/viewer-mirror/target/wasm32-unknown-unknown/wasm/nomos_play.wasm"), wasm);

  for (const [directory, plan] of [["scene-a", "scene_one"], ["scene-b", "scene_two"]]) {
    mkdir(join(template, `r2/${directory}`));
    for (let index = 0; index < 10; index += 1) cpSync(join(repo, `fixtures/r2/plans/${plan}.json`), join(template, `r2/${directory}/plan-${String(index).padStart(2, "0")}.json`));
  }
  mkdir(join(template, "r2"));
  cpSync(join(repo, "docs/evaluation/runs/r2/2026-08-27-issue-197-second-author/SCENE_SIGNATURES.json"), join(template, "r2/signatures.json"));
  buildR2("viewer-proof");
  buildR2("viewer-a");
  buildR2("viewer-b");

  const binary = join(repo, "target/r2-complete-release/release/nomos-observed-scene");
  mkdir(dirname(binary));
  writeFileSync(binary, "test compiler\n");
  const benchmark = join(template, "r2/compile-benchmark");
  mkdir(benchmark);
  const plan = `${JSON.stringify({ schema: "nomos.observed_scene_plan@1", source_sha256: "fe332f711437dab15e4d1315cc3ca57dba6521350ff673941e77feb414585909" })}\n`;
  for (let index = 0; index < 10; index += 1) writeFileSync(join(benchmark, `warmup-${String(index).padStart(3, "0")}.json`), plan);
  const sampleRows = ["ordinal\telapsed_ns\tbytes\tsha256\tpath"];
  for (let index = 0; index < 100; index += 1) {
    const path = join(benchmark, `sample-${String(index).padStart(3, "0")}.json`);
    writeFileSync(path, plan);
    sampleRows.push(`${index}\t10000000\t${Buffer.byteLength(plan)}\t${sha256(plan)}\t${path}`);
  }
  writeFileSync(join(benchmark, "samples.tsv"), `${sampleRows.join("\n")}\n`);
  json(join(benchmark, "summary.json"), {
    architecture: "x64", binary, binary_sha256: sha256(readFileSync(binary)), cpu_count: 1,
    fixture: join(repo, "fixtures/r2/maximum-observed-scene.json"),
    fixture_sha256: "fe332f711437dab15e4d1315cc3ca57dba6521350ff673941e77feb414585909",
    hostname: "synthetic",
    recorded_samples: 100, warmups: 10, median_denominator: 2, median_numerator_ns: "20000000",
    p95_ns: "10000000", median_ceiling_ns: 50000000, p95_ceiling_ns: 100000000,
    median_pass: true, node: process.version,
    output_digest: `${Buffer.byteLength(plan)}:${sha256(plan)}`,
    p95_pass: true, platform: process.platform, release: "synthetic",
  });

  const browser = join(template, "r2/browser-smoke");
  mkdir(browser);
  const shot1 = png(1280, 720);
  const shot2 = png(1280, 720);
  const sheet = png(2560, 720);
  writeFileSync(join(browser, "scene_1.png"), shot1);
  writeFileSync(join(browser, "scene_2.png"), shot2);
  writeFileSync(join(browser, "contact-sheet.png"), sheet);
  const samples = Array(10).fill("100000000");
  const scene = (index) => {
    const name = index === 0 ? "one" : "two";
    const planBytes = readFileSync(join(repo, `fixtures/r2/plans/scene_${name}.json`));
    const plan = JSON.parse(planBytes);
    return {
      plan: {
        path: `plans/scene_${name}.json`, sha256: Object.values(plans)[index], bytes: planBytes.length,
        expected_counts: {
          actions: plan.actions.length, actors: plan.actors.length,
          controlled_markers: plan.actors.filter((row) => row.controlled_marker === "present").length,
          hostile_outlines: plan.actors.filter((row) => row.hostile_outline === "present").length,
          protection_rings: plan.actors.filter((row) => row.protection_ring === "present").length,
          terrain_cells: plan.terrain_layers.reduce((sum, row) => sum + row.cells.length, 0),
          terrain_layers: plan.terrain_layers.length,
        },
      },
      samples_ns: samples, median_denominator: 2, median_numerator_ns: "200000000", p95_ns: "100000000",
    };
  };
  const scenes = [scene(0), scene(1)];
  const closure = () => ({ duration_ms: 1, exit_code: null, signal: "SIGTERM", after_result_ms: 1 });
  const chromeFlags = (profile) => [
    "--headless=new", "--remote-debugging-port=0",
    "--host-resolver-rules=MAP * ~NOTFOUND, EXCLUDE localhost", "--no-first-run",
    "--no-default-browser-check", "--no-sandbox", "--disable-dev-shm-usage",
    "--disable-extensions", "--disable-sync", "--disable-background-networking",
    "--disable-component-update", "--disable-background-timer-throttling",
    "--disable-renderer-backgrounding", "--disable-backgrounding-occluded-windows",
    "--window-size=1280,720", "--force-device-scale-factor=1", "--hide-scrollbars",
    "--use-gl=angle", "--use-angle=swiftshader", "--enable-unsafe-swiftshader",
    `--user-data-dir=${profile}`, "about:blank",
  ];
  const launches = Array.from({ length: 20 }, (_, launchOrdinal) => {
    const sceneOrdinal = Math.floor(launchOrdinal / 10);
    const sampleOrdinal = launchOrdinal % 10;
    const profile = join(template, "host/tmp", `nomos-observed-chrome-${String(launchOrdinal).padStart(2, "0")}`);
    return {
      browser_product: "HeadlessChrome/test", cache_disabled: true, chrome_flags: chromeFlags(profile),
      closure: closure(), console_errors: [], elapsed_ns: "100000000", exceptions: [],
      frame: { consequence_counts: scenes[sceneOrdinal].plan.expected_counts, plan_sha256: scenes[sceneOrdinal].plan.sha256, viewport: { width: 1280, height: 720 } },
      launch_ordinal: launchOrdinal, network_negative_control: "blocked", profile,
      requests: [`http://localhost:4321/?scene=${sceneOrdinal}`], sample_ordinal: sampleOrdinal,
      scene_ordinal: sceneOrdinal, screenshot: sampleOrdinal === 0 ? `scene_${sceneOrdinal + 1}.png` : null,
      webgl2: true,
    };
  });
  json(join(browser, "receipt.json"), {
    outcome: "pass", browser: "HeadlessChrome/test", chrome_flags: launches[0].chrome_flags, samples_per_scene: 10, per_scene: scenes,
    combined: { median_denominator: 2, median_numerator_ns: "200000000", p95_ns: "100000000" },
    external_requests: [], launches,
    closures: [...launches.map((launch) => ({ ...launch.closure, kind: "browser_launch", launch_ordinal: launch.launch_ordinal })), { duration_ms: 1, sockets_destroyed: 0, after_result_ms: 1, kind: "server" }, { ...closure(), kind: "contact_sheet_browser" }],
    requests: Array.from({ length: 20 }, () => ({ path: "/", status: 200 })),
    screenshots: [
      { path: "scene_1.png", bytes: shot1.length, sha256: sha256(shot1), viewport: { width: 1280, height: 720 } },
      { path: "scene_2.png", bytes: shot2.length, sha256: sha256(shot2), viewport: { width: 1280, height: 720 } },
    ],
    contact_sheet: { path: "contact-sheet.png", bytes: sheet.length, sha256: sha256(sheet), viewport: { width: 2560, height: 720 } },
  });
};

const restoreFixture = () => {
  const current = new Set(inventory(fixtureOutput).map((row) => row.path));
  for (const path of current) {
    if (!fixtureSnapshot.has(path)) rmSync(join(fixtureOutput, path));
  }
  for (const [path, bytes] of fixtureSnapshot) {
    const destination = join(fixtureOutput, path);
    if (!current.has(path) || !readFileSync(destination).equals(bytes)) {
      mkdirSync(dirname(destination), { recursive: true });
      writeFileSync(destination, bytes);
    }
  }
};
const fixtureForCase = () => {
  if (fixtureSnapshot) {
    restoreFixture();
    return fixtureOutput;
  }
  cpSync(template, fixtureOutput, { recursive: true });
  const output = fixtureOutput;
  const boundary = JSON.parse(readFileSync(join(output, "metadata/write-boundary.json")));
  boundary.output_relative = relative(repo, output);
  boundary.allowed_roots = [relative(repo, output), "target"];
  json(join(output, "metadata/write-boundary.json"), boundary);
  const filesystem = JSON.parse(readFileSync(join(output, "metadata/filesystem-isolation.json")));
  filesystem.writable_roots = [relative(repo, output), "target"];
  json(join(output, "metadata/filesystem-isolation.json"), filesystem);
  const mountinfo = join(output, "metadata/mountinfo.txt");
  writeFileSync(mountinfo, readFileSync(mountinfo, "utf8").replaceAll(template, output));
  const filesystemDirectory = join(output, "measurements/filesystem");
  const templateMountRoot = `/checkout/${relative(repo, template).split("\\").join("/")}`;
  const outputMountRoot = `/checkout/${relative(repo, output).split("\\").join("/")}`;
  for (const name of ["identity.json", "du-setup.json", "du-shutdown.json", "release.json", "summary.json"]) {
    const path = join(filesystemDirectory, name);
    writeFileSync(path, readFileSync(path, "utf8").replaceAll(template, output).replaceAll(templateMountRoot, outputMountRoot));
  }
  const samples = join(output, "r2/compile-benchmark/samples.tsv");
  writeFileSync(samples, readFileSync(samples, "utf8").replaceAll(template, output));
  const argv = join(output, "commands.argv.ndjson");
  writeFileSync(argv, readFileSync(argv, "utf8").replaceAll(template, output));
  const browser = join(output, "r2/browser-smoke/receipt.json");
  writeFileSync(browser, readFileSync(browser, "utf8").replaceAll(template, output));
  fixtureSnapshot = new Map(inventory(output).map((row) => [row.path, readFileSync(join(output, row.path))]));
  rmSync(template, { recursive: true, force: true });
  return output;
};

const makeFixtureRepository = () => {
  const archive = join(temporary, "source.tar");
  mkdirSync(repo, { recursive: true });
  execFileSync("git", ["-C", sourceRepo, "archive", "--format=tar", `--output=${archive}`, "HEAD", "--", ...fixtureSourcePaths]);
  execFileSync("tar", ["-xf", archive, "-C", repo]);
  rmSync(archive);
  execFileSync("git", ["-C", repo, "init", "--quiet", "--object-format=sha1"]);
  execFileSync("git", ["-C", repo, "config", "core.autocrlf", "false"]);
  execFileSync("git", ["-C", repo, "add", "--all"]);
  const tree = execFileSync("git", ["-C", repo, "write-tree"], { encoding: "utf8" }).trim();
  const env = {
    ...process.env,
    GIT_AUTHOR_NAME: "Nomos synthetic verifier fixture",
    GIT_AUTHOR_EMAIL: "nomos-fixture@example.invalid",
    GIT_AUTHOR_DATE: "2000-01-01T00:00:00Z",
    GIT_COMMITTER_NAME: "Nomos synthetic verifier fixture",
    GIT_COMMITTER_EMAIL: "nomos-fixture@example.invalid",
    GIT_COMMITTER_DATE: "2000-01-01T00:00:00Z",
  };
  const commit = execFileSync("git", ["-C", repo, "commit-tree", tree, "-m", "synthetic verifier fixture"], { encoding: "utf8", env }).trim();
  execFileSync("git", ["-C", repo, "checkout", "--quiet", "--detach", commit]);
};

const candidate = () => ({
  commit: execFileSync("git", ["-C", repo, "rev-parse", "HEAD"], { encoding: "utf8" }).trim(),
  tree: execFileSync("git", ["-C", repo, "rev-parse", "HEAD^{tree}"], { encoding: "utf8" }).trim(),
});

const assemble = (output) => assembleReceipt({ repo, output, ...candidate(), issue: 199, issueBodySha256: issueBody, liveChecks: false });

before(() => {
  // Exercise the real standalone-root checks without duplicating irrelevant
  // candidate history or source that the synthetic verifier never reads.
  makeFixtureRepository();
  makeTemplate();
});

test("synthetic complete evidence assembles and verifies without trusting summaries", () => {
  const output = fixtureForCase();
  const receipt = assemble(output);
  assert.equal(receipt.outcome, "pass");
  assert.equal(receipt.summary.commands.argv_count, 33);
  assert.equal(receipt.summary.r2.compile.samples, 100);
  assert.equal(receipt.summary.r2.browser.closures, 22);
  assert.equal(receipt.summary.budgets.checkout_peak_mib, 489);
  assert.equal(receipt.summary.budgets.disk_samples, 2);
  assert.equal(receipt.summary.budgets.disk_maximum_gap_ns, "50000000");
  assert.equal(receipt.summary.budgets.disk_stop_requested_ns, (filesystemOrigin + 25_000_000n).toString());
  assert.equal(receipt.summary.budgets.filesystem_capacity_bytes, "4096000000");
  assert.equal(receipt.summary.budgets.filesystem_counter, "f_blocks-f_bfree");
  assert.equal(receipt.summary.budgets.setup_du_mib, "489");
  assert.equal(receipt.summary.budgets.shutdown_du_mib, "489");
  assert.equal(receipt.summary.budgets.reservation_allocated_bytes, "16777216");
  assert.equal(receipt.summary.budgets.post_release_allocated_bytes, "495222784");
  assert.equal(verifyReceipt({ repo, output, liveChecks: false }).outcome, "pass");

  writeFileSync(join(output, "unexpected.txt"), "drift\n");
  assert.throws(() => verifyReceipt({ repo, output, liveChecks: false }), /missing or extra path/);
  rmSync(join(output, "unexpected.txt"));
  const commands = join(output, "commands.tsv");
  writeFileSync(commands, readFileSync(commands, "utf8").replace(commandDisplays[0], "forged workspace-fmt"));
  assert.throws(() => verifyReceipt({ repo, output, liveChecks: false }), /digest\/path drift/);

  const argvOutput = fixtureForCase();
  const argv = join(argvOutput, "commands.argv.ndjson");
  const argvLines = readFileSync(argv, "utf8").trimEnd().split("\n");
  const forgedArgv = JSON.parse(argvLines[0]);
  forgedArgv.argv.push("forged");
  argvLines[0] = JSON.stringify(forgedArgv);
  writeFileSync(argv, `${argvLines.join("\n")}\n`);
  assert.throws(() => assemble(argvOutput), /expected command argv/);

  const extraArgv = fixtureForCase();
  const extraArgvPath = join(extraArgv, "commands.argv.ndjson");
  const extraArgvText = readFileSync(extraArgvPath, "utf8");
  writeFileSync(extraArgvPath, `${extraArgvText}${extraArgvText.split("\n")[0]}\n`);
  assert.throws(() => assemble(extraArgv), /argv row count/);

  const duplicateArgv = fixtureForCase();
  const duplicateArgvPath = join(duplicateArgv, "commands.argv.ndjson");
  const duplicateArgvLines = readFileSync(duplicateArgvPath, "utf8").trimEnd().split("\n");
  duplicateArgvLines[1] = duplicateArgvLines[0];
  writeFileSync(duplicateArgvPath, `${duplicateArgvLines.join("\n")}\n`);
  assert.throws(() => assemble(duplicateArgv), /argv row 2 is a duplicate/);

  const missing = fixtureForCase();
  assemble(missing);
  rmSync(join(missing, "logs/01-workspace-fmt.stdout"));
  assert.throws(() => verifyReceipt({ repo, output: missing, liveChecks: false }), /missing or extra path|digest\/path drift/);

  const falsified = fixtureForCase();
  assemble(falsified);
  const receiptPath = join(falsified, "receipt.json");
  const forged = JSON.parse(readFileSync(receiptPath));
  forged.summary.budgets.checkout_peak_mib = 1;
  json(receiptPath, forged);
  assert.throws(() => verifyReceipt({ repo, output: falsified, liveChecks: false }), /receipt summary differs/);
});

test("filesystem cadence accepts the exact 100000000 ns boundary", () => {
  const output = fixtureForCase();
  const rawPath = join(output, "measurements/filesystem/raw.tsv");
  const lines = readFileSync(rawPath, "utf8").trimEnd().split("\n");
  const terminal = lines[2].split("\t");
  terminal[1] = (filesystemOrigin + 100_000_000n).toString();
  terminal[2] = "100000000";
  terminal[3] = terminal[1];
  lines[2] = terminal.join("\t");
  writeFileSync(rawPath, `${lines.join("\n")}\n`);
  const publicPath = join(output, "measurements/filesystem/public.tsv");
  const publicLines = readFileSync(publicPath, "utf8").trimEnd().split("\n");
  const publicTerminal = publicLines[2].split("\t");
  publicTerminal[1] = terminal[1];
  publicTerminal[2] = terminal[2];
  publicLines[2] = publicTerminal.join("\t");
  writeFileSync(publicPath, `${publicLines.join("\n")}\n`);
  const summaryPath = join(output, "measurements/filesystem/summary.json");
  const summary = JSON.parse(readFileSync(summaryPath));
  summary.maximum_gap_ns = "100000000";
  json(summaryPath, summary);
  assert.equal(assemble(output).summary.budgets.disk_maximum_gap_ns, "100000000");
});

test("filesystem raw counters cannot be forged", () => {
  const output = fixtureForCase();
  const rawPath = join(output, "measurements/filesystem/raw.tsv");
  const lines = readFileSync(rawPath, "utf8").trimEnd().split("\n");
  const row = lines[1].split("\t");
  row[21] = "512000001";
  lines[1] = row.join("\t");
  writeFileSync(rawPath, `${lines.join("\n")}\n`);
  assert.throws(() => assemble(output), /raw counter mismatch/);
});

test("major plants fail closed before a receipt can be assembled", async (t) => {
  const plants = [
    ["command-result", (out) => {
      const path = join(out, "commands.tsv");
      writeFileSync(path, readFileSync(path, "utf8").replace("\t0\tlogs/01-workspace-fmt", "\t1\tlogs/01-workspace-fmt"));
    }, /exited 1/],
    ["route-leak", (out) => json(join(out, "metadata/ip-route-v4.json"), [{ dst: "default", dev: "eth0" }]), /route counts are not exact loopback|IPv4 route/],
    ["address-leak", (out) => {
      const path = join(out, "metadata/ip-address.json");
      const value = JSON.parse(readFileSync(path));
      value[0].addr_info.push({ family: "inet", local: "10.0.0.1", prefixlen: 24, scope: "global" });
      json(path, value);
    }, /address families are not exact loopback/],
    ["pid-namespace", (out) => {
      const path = join(out, "metadata/isolation.json");
      const value = JSON.parse(readFileSync(path));
      value.pid_namespace = "host";
      json(path, value);
    }, /isolation summary is not a pass/],
    ["outer-positive-control", (out) => {
      const path = join(out, "metadata/network-control.json");
      const value = JSON.parse(readFileSync(path));
      value.outer_positive.exit_code = 1;
      json(path, value);
    }, /positive control did not connect/],
    ["missing-outer-positive", (out) => rmSync(join(out, "metadata/network-outer-positive.stdout")), /network-outer-positive.stdout is missing/],
    ["missing-xfs-validation", (out) => rmSync(join(out, "metadata/xfs-shell-validation.stdout")), /XFS shell-validation stdout is missing/],
    ["xfs-validation-status", (out) => writeFileSync(join(out, "metadata/xfs-shell-validation.status"), "1\n"), /XFS shell-validation status or stderr differs/],
    ["xfs-validation-drift", (out) => writeFileSync(join(out, "metadata/xfs-shell-validation.stdout"), "forged pass\n"), /XFS shell-validation PASS marker differs/],
    ["xfs-validation-argv", (out) => {
      const path = join(out, "metadata/xfs-shell-validation.argv.json");
      const value = JSON.parse(readFileSync(path));
      value.argv[1] = "/tmp/forged-test.sh";
      json(path, value);
    }, /XFS shell-validation invocation differs/],
    ["xfs-validation-candidate", (out) => {
      const path = join(out, "metadata/xfs-shell-validation.candidate.json");
      const value = JSON.parse(readFileSync(path));
      value.tree = "0".repeat(40);
      json(path, value);
    }, /XFS shell-validation candidate differs/],
    ["inner-negative-control", (out) => {
      const path = join(out, "metadata/network-control.json");
      const value = JSON.parse(readFileSync(path));
      value.inner_negative = { outcome: "connected", exit_code: 0, stdout: "connected\n", stderr: "" };
      json(path, value);
    }, /negative control/],
    ["network-raw-drift", (out) => writeFileSync(join(out, "metadata/network-inner-negative.stderr"), "different refusal"), /does not match its raw streams/],
    ["filesystem-control", (out) => {
      const path = join(out, "metadata/filesystem-isolation.json");
      const value = JSON.parse(readFileSync(path));
      value.negative_control.exit_code = 0;
      json(path, value);
    }, /filesystem negative control/],
    ["mount-read-write", (out) => {
      const path = join(out, "metadata/mountinfo.txt");
      writeFileSync(path, readFileSync(path, "utf8").replace("/ / ro", "/ / rw"));
    }, /record \/ as ro/],
    ["tool-digest", (out) => {
      const path = join(out, "metadata/tools.txt");
      writeFileSync(path, readFileSync(path, "utf8").replace(/[0-9a-f]{64}/, "0".repeat(64)));
    }, /tool git digest differs/],
    ["command-display", (out) => {
      const path = join(out, "commands.tsv");
      writeFileSync(path, readFileSync(path, "utf8").replace(commandDisplays[0], "cargo fmt --all"));
    }, /command workspace-fmt display differs/],
    ["proof-marker", (out) => writeFileSync(join(out, "logs/04-workspace-boundary.stdout"), "forged pass\n"), /proof marker is absent/],
    ["authority-drift", (out) => {
      const path = join(out, "metadata/source-tree.json");
      const value = JSON.parse(readFileSync(path));
      value.catalog_sha256 = "0".repeat(64);
      json(path, value);
    }, /catalog_sha256 differs/],
    ["r1-skip", (out) => {
      const path = join(out, "logs/10-r1-viewer-tests.stdout");
      writeFileSync(path, tap(104).replace("# pass 104", "# pass 103").replace("# skipped 0", "# skipped 1"));
    }, /unskipped pass/],
    ["build-overflow", (out) => {
      const path = join(out, "measurements/clean-release-time.txt");
      writeFileSync(path, readFileSync(path, "utf8").replace("0:12.34", "1:00.01"));
    }, /exceeded 60 s/],
    ["filesystem-raw-counter", (out) => {
      const path = join(out, "measurements/filesystem/raw.tsv");
      const lines = readFileSync(path, "utf8").trimEnd().split("\n");
      const fields = lines[1].split("\t");
      fields[21] = "512000001";
      lines[1] = fields.join("\t");
      writeFileSync(path, `${lines.join("\n")}\n`);
    }, /raw counter mismatch/],
    ["filesystem-identity", (out) => {
      const path = join(out, "measurements/filesystem/identity.json");
      const value = JSON.parse(readFileSync(path));
      value.uuid = "not-a-uuid";
      json(path, value);
    }, /identity UUID is malformed/],
    ["filesystem-gap", (out) => {
      const path = join(out, "measurements/filesystem/raw.tsv");
      const lines = readFileSync(path, "utf8").trimEnd().split("\n");
      const fields = lines[2].split("\t");
      fields[1] = (filesystemOrigin + 100_000_001n).toString();
      fields[2] = "100000001";
      fields[3] = fields[1];
      lines[2] = fields.join("\t");
      writeFileSync(path, `${lines.join("\n")}\n`);
    }, /gap exceeds 100000000 ns/],
    ["filesystem-du", (out) => {
      const path = join(out, "measurements/filesystem/du-setup.json");
      const value = JSON.parse(readFileSync(path));
      value.invocation.stdout = `490\t${repo}\n`;
      json(path, value);
    }, /du MiB exceeds statfs allocated MiB/],
    ["filesystem-release", (out) => {
      const path = join(out, "measurements/filesystem/release.json");
      const value = JSON.parse(readFileSync(path));
      value.a_before_bytes = "1";
      json(path, value);
    }, /A_before is below reservation allocation/],
    ["filesystem-summary", (out) => {
      const path = join(out, "measurements/filesystem/summary.json");
      const value = JSON.parse(readFileSync(path));
      value.maximum_mib = "1";
      json(path, value);
    }, /checkout-disk-summary differs/],
    ["filesystem-stop", (out) => {
      writeFileSync(join(out, "measurements/filesystem/stop"), `${filesystemOrigin + 100_000_001n}\n`);
    }, /terminal sample precedes monotonic stop request/],
    ["compile-summary", (out) => {
      const path = join(out, "r2/compile-benchmark/summary.json");
      const value = JSON.parse(readFileSync(path));
      value.p95_ns = "1";
      json(path, value);
    }, /compile summary arithmetic differs/],
    ["compile-missing", (out) => rmSync(join(out, "r2/compile-benchmark/sample-099.json")), /retained file set differs/],
    ["compile-path", (out) => {
      const path = join(out, "r2/compile-benchmark/samples.tsv");
      writeFileSync(path, readFileSync(path, "utf8").replace("sample-000.json", "sample-001.json"));
    }, /sample 0 path differs/],
    ["compile-ceiling", (out) => {
      const samplesPath = join(out, "r2/compile-benchmark/samples.tsv");
      writeFileSync(samplesPath, readFileSync(samplesPath, "utf8").replaceAll("\t10000000\t", "\t100000001\t"));
      const summaryPath = join(out, "r2/compile-benchmark/summary.json");
      const value = JSON.parse(readFileSync(summaryPath));
      value.median_numerator_ns = "200000002";
      value.p95_ns = "100000001";
      json(summaryPath, value);
    }, /latency ceiling exceeded/],
    ["dist-drift", (out) => writeFileSync(join(out, "r2/viewer-b/dist/extra"), "extra\n"), /receipt does not match its dist tree/],
    ["dist-prohibited-payload", (out) => {
      for (const name of ["viewer-proof", "viewer-a", "viewer-b"]) {
        const path = join(out, `r2/${name}/dist/index.html`);
        writeFileSync(path, `${readFileSync(path, "utf8")}https://evil.invalid/\n`);
        refreshR2Receipt(out, name);
      }
    }, /forbidden source, origin, or machine path/],
    ["browser-external", (out) => {
      const path = join(out, "r2/browser-smoke/receipt.json");
      const value = JSON.parse(readFileSync(path));
      value.external_requests = ["https://example.invalid/"];
      json(path, value);
    }, /external request/],
    ["browser-samples", (out) => {
      const path = join(out, "r2/browser-smoke/receipt.json");
      const value = JSON.parse(readFileSync(path));
      value.per_scene[0].samples_ns.pop();
      json(path, value);
    }, /raw samples differ/],
    ["browser-ceiling", (out) => {
      const path = join(out, "r2/browser-smoke/receipt.json");
      const value = JSON.parse(readFileSync(path));
      value.per_scene[0].samples_ns = Array(10).fill("6000000000");
      for (const launch of value.launches.slice(0, 10)) launch.elapsed_ns = "6000000000";
      value.per_scene[0].median_numerator_ns = "12000000000";
      value.per_scene[0].p95_ns = "6000000000";
      value.combined.median_numerator_ns = "6100000000";
      value.combined.p95_ns = "6000000000";
      json(path, value);
    }, /timing summary\/ceiling differs/],
    ["browser-webgl", (out) => {
      const path = join(out, "r2/browser-smoke/receipt.json");
      const value = JSON.parse(readFileSync(path));
      value.launches[0].webgl2 = false;
      json(path, value);
    }, /browser facts differ/],
    ["browser-cache", (out) => {
      const path = join(out, "r2/browser-smoke/receipt.json");
      const value = JSON.parse(readFileSync(path));
      value.launches[0].cache_disabled = false;
      json(path, value);
    }, /browser facts differ/],
    ["browser-profile-reuse", (out) => {
      const path = join(out, "r2/browser-smoke/receipt.json");
      const value = JSON.parse(readFileSync(path));
      value.launches[1].profile = value.launches[0].profile;
      value.launches[1].chrome_flags = value.launches[0].chrome_flags;
      json(path, value);
    }, /fresh closed profile/],
    ["browser-navigation", (out) => {
      const path = join(out, "r2/browser-smoke/receipt.json");
      const value = JSON.parse(readFileSync(path));
      for (const launch of value.launches) launch.requests = ["http://localhost:4321/index.html"];
      value.requests = Array.from({ length: 20 }, () => ({ path: "/index.html", status: 200 }));
      json(path, value);
    }, /exact navigation request is absent/],
    ["browser-launch-external", (out) => {
      const path = join(out, "r2/browser-smoke/receipt.json");
      const value = JSON.parse(readFileSync(path));
      value.launches[0].requests = ["https://example.invalid/"];
      json(path, value);
    }, /made an external request/],
    ["browser-closure", (out) => {
      const path = join(out, "r2/browser-smoke/receipt.json");
      const value = JSON.parse(readFileSync(path));
      value.closures[0].after_result_ms = 2001;
      value.launches[0].closure.after_result_ms = 2001;
      json(path, value);
    }, /closure exceeded/],
    ["image-digest", (out) => writeFileSync(join(out, "r2/browser-smoke/scene_1.png"), png(1280, 721)), /screenshot 1 binding differs/],
    ["leaked-child", (out) => json(join(out, "metadata/process-closure.json"), { outcome: "pass", checked_while_sampler: true, checked_after_sampler: true, leaked_processes: [123], namespace_children_before_sampler_stop: [], namespace_children: [] }), /did not close/],
    ["closure-raw-drift", (out) => writeFileSync(join(out, "metadata/namespace-children-before-sampler-stop.txt"), "123\n"), /pre-stop process-closure raw list differs/],
    ["dirty-candidate-record", (out) => {
      const path = join(out, "metadata/clean-end.json");
      const value = JSON.parse(readFileSync(path));
      value.porcelain = " M input";
      json(path, value);
    }, /not clean at candidate/],
    ["write-boundary", (out) => json(join(out, "metadata/write-boundary.json"), { outcome: "pass", output_relative: relative(repo, out), allowed_roots: [relative(repo, out), "target"], outside_writes: ["elsewhere"], inputs_unchanged: true }), /write boundary is not clean/],
  ];
  for (const [name, mutate, pattern] of plants) await t.test(name, () => {
    const output = fixtureForCase();
    mutate(output);
    assert.throws(() => assemble(output), pattern);
  });

  await t.test("wrong-candidate", () => {
    const output = fixtureForCase();
    assert.throws(() => assembleReceipt({ repo, output, commit: "0".repeat(40), tree: candidate().tree, issue: 199, issueBodySha256: issueBody, liveChecks: false }), /HEAD differs/);
  });
  await t.test("outside-output", () => {
    const output = join(temporary, "outside-output");
    try {
      cpSync(fixtureOutput, output, { recursive: true });
      assert.throws(() => assembleReceipt({ repo, output, ...candidate(), issue: 199, issueBodySha256: issueBody, liveChecks: false }), /physically inside/);
    } finally {
      rmSync(output, { recursive: true, force: true });
    }
  });
});
