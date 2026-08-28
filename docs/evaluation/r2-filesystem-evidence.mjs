import {
  readFileSync,
  realpathSync,
  statSync,
  statfsSync,
} from "node:fs";
import { spawnSync } from "node:child_process";
import { dirname, resolve } from "node:path";
import { fileURLToPath } from "node:url";

import {
  MAX_CAPACITY_BYTES,
  MAX_SAMPLE_GAP_NS,
  MEBIBYTE,
  RAW_HEADER,
  SAMPLE_PERIOD_NS,
  XFS_MAGIC,
  XFS_MAGIC_HEX,
  FilesystemAccountingError,
  accountingFromStatfs,
  canonicalUnsignedDecimal,
  captureFilesystemIdentity,
  ceilMebibytes,
  decodeMountInfoToken,
  parseRawRow,
  validateRawLedger,
} from "./r2-filesystem-accounting.mjs";

/*
 * This module is a receipt boundary.  The du mode owns exactly one captured
 * ionice/du invocation and takes the immediate direct statfs snapshot before
 * emitting anything.  The other modes never walk a tree; release mode only
 * reads scalar reservation arguments before its direct statfs snapshot.
 */

export const EVIDENCE_SCHEMA = "nomos-r2-filesystem-evidence/1";
export const SUMMARY_SCHEMA = "nomos-r2-checkout-disk-summary/1";
export const RESERVATION_LENGTH_BYTES = 16_777_216n;
export const DU_ARGV = Object.freeze(["ionice", "-c", "3", "du", "-sm", "--"]);
const SUMMARY_KEYS = Object.freeze([
  "outcome",
  "schema",
  "method",
  "counter",
  "sampler_origin_ns",
  "stop_requested_ns",
  "nominal_interval_ns",
  "samples",
  "initial_mib",
  "final_mib",
  "maximum_mib",
  "maximum_allocated_bytes",
  "capacity_bytes",
  "maximum_gap_ns",
  "setup_du_mib",
  "shutdown_du_mib",
  "du_arguments",
  "reservation_length_bytes",
  "reservation_allocated_bytes",
  "a_before_bytes",
  "a_after_bytes",
]);

const DECIMAL_PATTERN = /^(0|[1-9][0-9]*)$/;
const DU_PHASES = new Set(["setup", "shutdown"]);
const SNAPSHOT_KEYS = Object.freeze([
  "schema",
  "filesystem_type",
  "filesystem_magic",
  "counter",
  "mount_id",
  "mountpoint",
  "mount_root",
  "mount_options",
  "source",
  "major_minor",
  "uuid",
  "source_device",
  "source_major_minor",
  "root_device",
  "root_inode",
  "fragment_size",
  "block_size",
  "blocks",
  "free_blocks",
  "available_blocks",
  "used_blocks",
  "capacity_bytes",
  "used_bytes",
  "mebibytes",
]);

const fail = (message) => {
  throw new FilesystemAccountingError(message);
};

const asBigInt = (value, label) => {
  if (typeof value === "bigint") return value >= 0n ? value : fail(`${label} must be unsigned`);
  if (typeof value === "string" && DECIMAL_PATTERN.test(value)) return BigInt(value);
  fail(`${label} must be a canonical unsigned decimal string or BigInt`);
};

const decimal = (value, label) => asBigInt(value, label).toString();

const safeText = (value, label) => {
  if (typeof value !== "string" || value.length === 0 || /[\t\r\n]/.test(value)) {
    fail(`${label} is not safe text`);
  }
  return value;
};

const exactKeys = (value, keys, label) => {
  if (value === null || typeof value !== "object" || Array.isArray(value)) fail(`${label} is not an object`);
  const actual = Object.keys(value).sort();
  const expected = [...keys].sort();
  if (actual.length !== expected.length || actual.some((key, index) => key !== expected[index])) {
    fail(`${label} has unexpected keys`);
  }
  return value;
};

const optionalKeys = (value, required, optional, label) => {
  if (value === null || typeof value !== "object" || Array.isArray(value)) fail(`${label} is not an object`);
  const allowed = new Set([...required, ...optional]);
  for (const key of Object.keys(value)) if (!allowed.has(key)) fail(`${label} has unexpected key ${key}`);
  for (const key of required) if (!Object.hasOwn(value, key)) fail(`${label} is missing ${key}`);
  return value;
};

const absolutePath = (value, label) => {
  if (typeof value !== "string" || !value.startsWith("/")) fail(`${label} must be absolute`);
  const lexical = resolve(value);
  if (lexical !== value) fail(`${label} is not canonical: ${value}`);
  return value;
};

const existingCanonicalPath = (value, label) => {
  const lexical = absolutePath(value, label);
  let actual;
  try {
    actual = realpathSync(lexical);
  } catch (error) {
    fail(`${label} cannot be resolved: ${error.message}`);
  }
  if (actual !== lexical) fail(`${label} resolves to a different path`);
  return lexical;
};

const readText = (path, label) => readFileSync(existingCanonicalPath(path, label), "utf8");

const readJson = (path, label) => {
  const text = readText(path, label);
  try {
    return JSON.parse(text);
  } catch (error) {
    fail(`${label} is not valid JSON: ${error.message}`);
  }
};

const same = (actual, expected, label) => {
  if (expected !== undefined && actual !== expected) fail(`${label} changed: expected ${expected}, got ${actual}`);
};

const canonicalJson = (value) => JSON.stringify(value) + "\n";

const jsonNumberKeys = Object.freeze([
  "filesystem_type",
  "mount_id",
  "root_device",
  "root_inode",
  "fragment_size",
  "block_size",
  "blocks",
  "free_blocks",
  "available_blocks",
  "used_blocks",
  "capacity_bytes",
  "used_bytes",
  "mebibytes",
]);

const identityDocumentKeys = Object.freeze([
  "schema",
  "filesystem_type",
  "filesystem_magic",
  "counter",
  "mount_id",
  "mountpoint",
  "mount_root",
  "mount_options",
  "source",
  "major_minor",
  "uuid",
  "source_device",
  "source_major_minor",
  "root_device",
  "root_inode",
  "fragment_size",
  "block_size",
  "capacity_blocks",
  "capacity_bytes",
  "target",
  "output",
  "nested_mounts",
  "dedicated_fixed_capacity",
  "sampler_origin_ns",
  "nominal_interval_ns",
]);

const identityRequiredKeys = identityDocumentKeys.filter((key) => !["target", "output", "sampler_origin_ns", "nominal_interval_ns"].includes(key));
const UUID_PATTERN = /^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$/i;

const decodePath = (value, label) => {
  if (typeof value !== "string") fail(`${label} is not text`);
  const decoded = decodeMountInfoToken(value);
  return absolutePath(decoded, label);
};

const parseIdentityDocument = (document) => {
  optionalKeys(document, identityRequiredKeys, ["target", "output", "sampler_origin_ns", "nominal_interval_ns"], "identity");
  if (document.schema !== 1 || document.filesystem_magic !== XFS_MAGIC_HEX || document.counter !== "f_blocks-f_bfree") {
    fail("identity contract fields are invalid");
  }
  if (document.dedicated_fixed_capacity !== true) fail("identity is not dedicated fixed capacity");
  const nestedMounts = document.nested_mounts.map((mount) => {
    exactKeys(mount, ["mount_id", "mountpoint", "root", "mount_options", "source", "major_minor", "filesystem_type", "uuid"], "nested identity");
    return {
      mountId: asBigInt(mount.mount_id, "nested mount_id"),
      mountpoint: decodePath(mount.mountpoint, "nested mountpoint"),
      root: decodePath(mount.root, "nested mount root"),
      mountOptions: safeText(mount.mount_options, "nested mount options"),
      source: decodeMountInfoToken(safeText(mount.source, "nested source")),
      majorMinor: safeText(mount.major_minor, "nested major:minor"),
      filesystemType: safeText(mount.filesystem_type, "nested filesystem type"),
      uuid: safeText(mount.uuid, "nested UUID").toLowerCase(),
    };
  });
  const identity = {
    schema: 1,
    mountId: asBigInt(document.mount_id, "mount_id"),
    mountpoint: decodePath(document.mountpoint, "mountpoint"),
    mountRoot: decodePath(document.mount_root, "mount_root"),
    mountOptions: safeText(document.mount_options, "mount options"),
    source: decodeMountInfoToken(safeText(document.source, "source")),
    majorMinor: safeText(document.major_minor, "major:minor"),
    mountFilesystemType: "xfs",
    uuid: safeText(document.uuid, "UUID").toLowerCase(),
    filesystemType: asBigInt(document.filesystem_type, "filesystem_type"),
    filesystemMagic: document.filesystem_magic,
    rootDevice: asBigInt(document.root_device, "root_device"),
    rootInode: asBigInt(document.root_inode, "root_inode"),
    fragmentSize: asBigInt(document.fragment_size, "fragment_size"),
    blockSize: asBigInt(document.block_size, "block_size"),
    capacityBlocks: asBigInt(document.capacity_blocks, "capacity_blocks"),
    capacityBytes: asBigInt(document.capacity_bytes, "capacity_bytes"),
    targetPath: document.target === undefined ? undefined : decodePath(document.target, "target"),
    outputPath: document.output === undefined ? undefined : decodePath(document.output, "output"),
    sourceDevice: safeText(document.source_device, "source device"),
    sourceMajorMinor: safeText(document.source_major_minor, "source major:minor"),
    nestedMounts,
    samplerOriginNs: document.sampler_origin_ns === undefined ? undefined : asBigInt(document.sampler_origin_ns, "sampler origin"),
    nominalIntervalNs: document.nominal_interval_ns === undefined ? undefined : asBigInt(document.nominal_interval_ns, "nominal interval"),
  };
  if (identity.filesystemType !== XFS_MAGIC) fail("identity filesystem is not XFS");
  if (identity.blockSize !== identity.fragmentSize) fail("identity fragment and block sizes differ");
  if (identity.capacityBytes !== identity.capacityBlocks * identity.fragmentSize) fail("identity capacity formula is inconsistent");
  if (!/^\d+:\d+$/.test(identity.majorMinor) || !/^\d+:\d+$/.test(identity.sourceMajorMinor)) fail("identity major:minor is malformed");
  if (!UUID_PATTERN.test(identity.uuid)) fail("identity UUID is malformed");
  if (identity.targetPath === undefined || identity.outputPath === undefined) fail("identity lacks target or output path");
  if (identity.targetPath === identity.outputPath || !identity.targetPath.startsWith(`${identity.mountpoint}/`) || !identity.outputPath.startsWith(`${identity.mountpoint}/`)) fail("identity target/output paths are not checkout descendants");
  return Object.freeze(identity);
};

const identityStableSnapshot = (identity) => ({
  schema: 1,
  filesystem_type: decimal(identity.filesystemType, "filesystem_type"),
  filesystem_magic: XFS_MAGIC_HEX,
  counter: "f_blocks-f_bfree",
  mount_id: decimal(identity.mountId, "mount_id"),
  mountpoint: identity.mountpoint,
  mount_root: identity.mountRoot,
  mount_options: safeText(identity.mountOptions, "mount options"),
  source: identity.source,
  major_minor: safeText(identity.majorMinor, "major:minor"),
  uuid: safeText(identity.uuid, "UUID").toLowerCase(),
  source_device: safeText(identity.sourceDevice, "source device"),
  source_major_minor: safeText(identity.sourceMajorMinor, "source major:minor"),
  root_device: decimal(identity.rootDevice, "root_device"),
  root_inode: decimal(identity.rootInode, "root_inode"),
});

export const filesystemSnapshotDocument = (identity) => {
  if (!identity || !identity.accounting) fail("filesystem snapshot has no accounting");
  const accounting = identity.accounting;
  return Object.freeze({
    ...identityStableSnapshot(identity),
    fragment_size: decimal(accounting.fragmentSize, "fragment_size"),
    block_size: decimal(accounting.blockSize, "block_size"),
    blocks: decimal(accounting.blocks, "blocks"),
    free_blocks: decimal(accounting.freeBlocks, "free_blocks"),
    available_blocks: decimal(accounting.availableBlocks, "available_blocks"),
    used_blocks: decimal(accounting.usedBlocks, "used_blocks"),
    capacity_bytes: decimal(accounting.capacityBytes, "capacity_bytes"),
    used_bytes: decimal(accounting.usedBytes, "used_bytes"),
    mebibytes: decimal(accounting.mebibytes, "mebibytes"),
  });
};

const parseSnapshotDocument = (document, baseline) => {
  exactKeys(document, SNAPSHOT_KEYS, "filesystem snapshot");
  if (!baseline || !baseline.accounting && baseline.capacityBlocks === undefined) fail("filesystem snapshot has no baseline identity");
  if (document.schema !== 1 || document.filesystem_magic !== XFS_MAGIC_HEX || document.counter !== "f_blocks-f_bfree") {
    fail("filesystem snapshot contract fields are invalid");
  }
  const stable = identityStableSnapshot(baseline);
  for (const key of Object.keys(stable)) same(document[key], stable[key], `snapshot ${key}`);
  const accounting = accountingFromStatfs({
    type: asBigInt(document.filesystem_type, "filesystem_type"),
    bsize: asBigInt(document.block_size, "block_size"),
    blocks: asBigInt(document.blocks, "blocks"),
    bfree: asBigInt(document.free_blocks, "free_blocks"),
    bavail: asBigInt(document.available_blocks, "available_blocks"),
  }, { fragmentSize: asBigInt(document.fragment_size, "fragment_size") });
  for (const [label, actual, expected] of [
    ["filesystem_type", accounting.filesystemType, baseline.filesystemType],
    ["fragment_size", accounting.fragmentSize, baseline.fragmentSize],
    ["block_size", accounting.blockSize, baseline.blockSize],
    ["blocks", accounting.blocks, baseline.capacityBlocks],
    ["capacity_bytes", accounting.capacityBytes, baseline.capacityBytes],
  ]) same(actual.toString(), expected.toString(), `snapshot ${label}`);
  for (const [key, value] of [
    ["fragment_size", accounting.fragmentSize],
    ["block_size", accounting.blockSize],
    ["blocks", accounting.blocks],
    ["free_blocks", accounting.freeBlocks],
    ["available_blocks", accounting.availableBlocks],
    ["used_blocks", accounting.usedBlocks],
    ["capacity_bytes", accounting.capacityBytes],
    ["used_bytes", accounting.usedBytes],
    ["mebibytes", accounting.mebibytes],
  ]) same(document[key], value.toString(), `snapshot ${key}`);
  return Object.freeze({ ...baseline, accounting });
};

const snapshotInput = (snapshot, baseline) => {
  if (snapshot && snapshot.accounting) {
    const current = snapshot;
    if (baseline) {
      for (const key of ["mountId", "mountpoint", "mountRoot", "mountOptions", "source", "majorMinor", "uuid", "rootDevice", "rootInode", "fragmentSize", "blockSize", "capacityBlocks", "capacityBytes"]) {
        same(String(current[key]), String(baseline[key]), `snapshot ${key}`);
      }
    }
    return current;
  }
  return parseSnapshotDocument(snapshot, baseline);
};

const captureStrictSnapshot = ({
  checkout,
  target,
  output,
  device,
  majorMinor,
  fragmentSize,
  uuid,
  readers = {},
}) => {
  const root = existingCanonicalPath(checkout, "checkout");
  const targetPath = existingCanonicalPath(target, "target");
  const outputPath = existingCanonicalPath(output, "output");
  return captureFilesystemIdentity(root, {
    fragmentSize,
    uuid,
    target: targetPath,
    output: outputPath,
    device,
    majorMinor,
    capacityLimitBytes: MAX_CAPACITY_BYTES,
    ...readers,
  });
};

const topologyExpectations = (identity, expectations = {}) => {
  const expected = {
    checkout: expectations.checkout,
    target: expectations.target,
    output: expectations.output,
    device: expectations.device,
    majorMinor: expectations.majorMinor,
    uuid: expectations.uuid,
    fragmentSize: expectations.fragmentSize,
  };
  const checks = [
    ["checkout", identity.mountpoint, expected.checkout],
    ["target", identity.targetPath, expected.target],
    ["output", identity.outputPath, expected.output],
    ["device", identity.sourceDevice, expected.device],
    ["majorMinor", identity.sourceMajorMinor, expected.majorMinor],
    ["uuid", identity.uuid, expected.uuid],
    ["fragmentSize", identity.fragmentSize, expected.fragmentSize],
  ];
  for (const [label, actual, value] of checks) {
    if (value !== undefined) same(String(actual), String(value), `identity ${label}`);
  }
  if (identity.mountRoot !== "/checkout") fail("identity checkout mount root is not /checkout");
  if (identity.mountFilesystemType !== "xfs") fail("identity checkout mount is not xfs");
  if (identity.mountOptions.split(",").includes("rw")) fail("identity checkout mount is not read-only");
  for (const option of ["ro", "nodev", "nosuid"]) if (!identity.mountOptions.split(",").includes(option)) fail(`identity checkout mount lacks ${option}`);
  if (identity.nestedMounts.length !== 2) fail("identity does not retain exactly two nested mounts");
  const nestedByPoint = new Map(identity.nestedMounts.map((mount) => [mount.mountpoint, mount]));
  for (const point of [identity.targetPath, identity.outputPath]) {
    const mount = nestedByPoint.get(point);
    if (mount === undefined) fail(`identity lacks nested mount ${point}`);
    const relative = point.slice(identity.mountpoint.length + 1);
    if (mount.root !== `/checkout/${relative}`) fail(`identity nested root changed at ${point}`);
    if (mount.source !== identity.source || mount.majorMinor !== identity.majorMinor || mount.filesystemType !== "xfs" || mount.uuid !== identity.uuid) fail(`identity nested filesystem changed at ${point}`);
    if (mount.mountOptions.split(",").includes("ro")) fail(`identity nested mount is not writable at ${point}`);
    for (const option of ["rw", "nodev", "nosuid"]) if (!mount.mountOptions.split(",").includes(option)) fail(`identity nested mount lacks ${option} at ${point}`);
  }
  return true;
};

const expectedDuArgv = (checkout) => [...DU_ARGV, checkout];

export const validateDuInvocation = (invocation, checkout, snapshot) => {
  exactKeys(invocation, ["phase", "argv", "cwd", "status", "stdout", "stderr", "started_ns", "ended_ns"], "du invocation");
  if (!DU_PHASES.has(invocation.phase)) fail("du phase is invalid");
  if (!Array.isArray(invocation.argv) || invocation.argv.length !== 7 || invocation.argv.some((item) => typeof item !== "string")) fail("du argv is invalid");
  if (JSON.stringify(invocation.argv) !== JSON.stringify(expectedDuArgv(checkout))) fail("du argv is not the exact R2 command");
  if (invocation.cwd !== checkout) fail("du cwd is not canonical checkout");
  if (invocation.status !== 0) fail("du status is not zero");
  if (invocation.stderr !== "") fail("du stderr is not empty");
  const started = asBigInt(invocation.started_ns, "du start timestamp");
  const ended = asBigInt(invocation.ended_ns, "du end timestamp");
  if (ended < started) fail("du timestamps are reversed");
  if (typeof invocation.stdout !== "string" || !invocation.stdout.endsWith("\n")) fail("du stdout lacks exact final newline");
  const stdout = invocation.stdout.slice(0, -1).split("\t");
  if (stdout.length !== 2 || !DECIMAL_PATTERN.test(stdout[0]) || stdout[1] !== checkout) fail("du stdout is not canonical");
  const duMib = BigInt(stdout[0]);
  const current = snapshotInput(snapshot);
  if (duMib > current.accounting.mebibytes) fail("du MiB exceeds statfs allocated MiB");
  return Object.freeze({
    ...invocation,
    started_ns: started.toString(),
    ended_ns: ended.toString(),
    du_mib: duMib.toString(),
    snapshot: current,
  });
};

export const validateDuCheck = (invocation, { checkout, snapshot, expectations = {} }) => {
  const root = absolutePath(checkout, "checkout");
  const current = snapshotInput(snapshot);
  topologyExpectations(current, { ...expectations, checkout: expectations.checkout ?? root });
  return validateDuInvocation(invocation, root, current);
};

const duOutputDocument = (validated) => ({
  schema: EVIDENCE_SCHEMA,
  mode: "du-check",
  phase: validated.phase,
  invocation: {
    argv: validated.argv,
    cwd: validated.cwd,
    status: validated.status,
    stdout: validated.stdout,
    stderr: validated.stderr,
    started_ns: validated.started_ns,
    ended_ns: validated.ended_ns,
  },
  du_mib: validated.du_mib,
  snapshot: filesystemSnapshotDocument(validated.snapshot),
});

const parseDuOutput = (document, baseline, phase) => {
  exactKeys(document, ["schema", "mode", "phase", "invocation", "du_mib", "snapshot"], "du evidence");
  if (document.schema !== EVIDENCE_SCHEMA || document.mode !== "du-check" || document.phase !== phase) fail("du evidence mode or phase is invalid");
  const snapshot = parseSnapshotDocument(document.snapshot, baseline);
  const validated = validateDuInvocation({ ...document.invocation, phase: document.phase }, baseline.mountpoint, snapshot);
  same(document.du_mib, validated.du_mib, "du_mib");
  return Object.freeze({ ...document, du_mib: validated.du_mib, snapshot });
};

const reservationInput = (value) => {
  exactKeys(value, ["reservation_path", "reservation_length_bytes", "reservation_allocated_bytes", "a_before_bytes"], "reservation input");
  const path = absolutePath(value.reservation_path, "reservation path");
  if (!path.endsWith("/host/finalization.reserve")) fail("reservation path is not finalization.reserve under output/host");
  const length = asBigInt(value.reservation_length_bytes, "reservation length");
  const allocated = asBigInt(value.reservation_allocated_bytes, "reservation allocated bytes");
  const before = asBigInt(value.a_before_bytes, "A_before");
  if (length !== RESERVATION_LENGTH_BYTES) fail("reservation length is not exactly 16777216 bytes");
  if (allocated < length) fail("reservation allocated bytes are below the exact reservation length");
  if (before < allocated) fail("A_before is below reservation allocation");
  return Object.freeze({ path, length, allocated, before });
};

export const validateReleaseCheck = ({ reservation, aAfterBytes, snapshot, expectations = {} }) => {
  const record = reservationInput(reservation);
  if (expectations.output !== undefined && record.path !== `${absolutePath(expectations.output, "output")}/host/finalization.reserve`) {
    fail("reservation path is not beneath the canonical output");
  }
  const current = snapshotInput(snapshot);
  topologyExpectations(current, expectations);
  const after = asBigInt(aAfterBytes, "A_after");
  if (current.accounting.usedBytes !== after) fail("A_after does not equal captured statfs allocation");
  if (after > record.before - record.allocated) fail("A_after is not at least reservation allocation below A_before");
  return Object.freeze({
    reservation: record,
    aAfter: after,
    snapshot: current,
  });
};

const releaseOutputDocument = (validated) => ({
  schema: EVIDENCE_SCHEMA,
  mode: "release-check",
  reservation: {
    path: validated.reservation.path,
    length_bytes: validated.reservation.length.toString(),
    allocated_bytes: validated.reservation.allocated.toString(),
  },
  a_before_bytes: validated.reservation.before.toString(),
  a_after_bytes: validated.aAfter.toString(),
  snapshot: filesystemSnapshotDocument(validated.snapshot),
});

const parseReleaseOutput = (document, baseline) => {
  exactKeys(document, ["schema", "mode", "reservation", "a_before_bytes", "a_after_bytes", "snapshot"], "release evidence");
  if (document.schema !== EVIDENCE_SCHEMA || document.mode !== "release-check") fail("release evidence mode is invalid");
  exactKeys(document.reservation, ["path", "length_bytes", "allocated_bytes"], "release reservation");
  const snapshot = parseSnapshotDocument(document.snapshot, baseline);
  const validated = validateReleaseCheck({
    reservation: {
      reservation_path: document.reservation.path,
      reservation_length_bytes: document.reservation.length_bytes,
      reservation_allocated_bytes: document.reservation.allocated_bytes,
      a_before_bytes: document.a_before_bytes,
    },
    aAfterBytes: document.a_after_bytes,
    snapshot,
  });
  return Object.freeze({ ...document, snapshot: validated.snapshot });
};

const parsePublicTsv = (text, rawRows) => {
  if (typeof text !== "string" || !text.startsWith("ordinal\tsample_start_ns\telapsed_ns\tmebibytes\tkind\n") || !text.endsWith("\n")) fail("public ledger header is invalid");
  const lines = text.slice(text.indexOf("\n") + 1, -1).split("\n");
  if (lines.length !== rawRows.length || lines.some((line) => line.length === 0)) fail("public ledger row count is invalid");
  const rows = lines.map((line, index) => {
    const fields = line.split("\t");
    if (fields.length !== 5) fail("public ledger row has wrong field count");
    const row = {
      ordinal: asBigInt(fields[0], "public ordinal"),
      sampleStartNs: asBigInt(fields[1], "public sample_start_ns"),
      elapsedNs: asBigInt(fields[2], "public elapsed_ns"),
      mebibytes: asBigInt(fields[3], "public mebibytes"),
      kind: fields[4],
    };
    const raw = rawRows[index];
    for (const [key, expected] of [["ordinal", raw.ordinal], ["sampleStartNs", raw.sampleStartNs], ["elapsedNs", raw.elapsedNs], ["mebibytes", raw.mebibytes], ["kind", raw.kind]]) {
      if (row[key] !== expected) fail(`public ledger mismatch: ${key}`);
    }
    return Object.freeze(row);
  });
  return Object.freeze(rows);
};

const markerValue = (text) => {
  if (typeof text !== "string" || !/^(0|[1-9][0-9]*)\n$/.test(text)) fail("stop marker is not one canonical decimal line");
  return BigInt(text.slice(0, -1));
};

const expectationForIdentity = (identity, expectations) => {
  topologyExpectations(identity, expectations);
  if (expectations.nominalIntervalNs !== undefined) same(identity.nominalIntervalNs?.toString(), decimal(expectations.nominalIntervalNs, "nominal interval"), "identity nominal interval");
  return identity;
};

const parseEvidenceIdentity = (identity) => {
  if (typeof identity === "string") {
    try { return parseIdentityDocument(JSON.parse(identity)); } catch (error) { if (error instanceof FilesystemAccountingError) throw error; fail(`identity JSON is invalid: ${error.message}`); }
  }
  return parseIdentityDocument(identity);
};

const parseRawEvidence = (raw, identity) => {
  if (typeof raw !== "string" || !raw.startsWith(`${RAW_HEADER}\n`)) fail("raw ledger is missing the exact header");
  if (!raw.endsWith("\n") || raw.endsWith("\n\n") || raw.includes("\r")) fail("raw ledger is not one canonical newline-terminated stream");
  if (identity.samplerOriginNs === undefined || identity.nominalIntervalNs === undefined) fail("identity lacks sampler origin or nominal interval");
  return validateRawLedger(raw, identity, {
    originNs: identity.samplerOriginNs,
    periodNs: identity.nominalIntervalNs,
    maxGapNs: MAX_SAMPLE_GAP_NS,
  });
};

const parseEvidence = (evidence, expectations = {}) => {
  optionalKeys(evidence, ["identity", "raw", "public", "setupDu", "shutdownDu", "finalization", "stop"], ["summary"], "filesystem evidence");
  const identity = expectationForIdentity(parseEvidenceIdentity(evidence.identity), expectations);
  const rawRows = parseRawEvidence(evidence.raw, identity);
  const publicRows = parsePublicTsv(evidence.public, rawRows);
  const setupDu = parseDuOutput(evidence.setupDu, identity, "setup");
  const shutdownDu = parseDuOutput(evidence.shutdownDu, identity, "shutdown");
  const finalization = parseReleaseOutput(evidence.finalization, identity);
  const stopNs = markerValue(evidence.stop);
  const terminal = rawRows[rawRows.length - 1];
  if (terminal.sampleStartNs < stopNs) fail("terminal sample precedes monotonic stop request");
  const shutdownUsed = asBigInt(shutdownDu.snapshot.used_bytes ?? shutdownDu.snapshot.accounting.usedBytes, "shutdown used bytes");
  if (finalization.reservation === undefined) fail("finalization reservation is absent");
  if (asBigInt(finalization.a_before_bytes, "A_before") !== shutdownUsed) fail("A_before does not equal immediate shutdown statfs snapshot");
  const snapshots = [
    ...rawRows.map((row) => row.usedBytes),
    asBigInt(setupDu.snapshot.used_bytes ?? setupDu.snapshot.accounting.usedBytes, "setup used bytes"),
    shutdownUsed,
    asBigInt(finalization.a_before_bytes, "A_before"),
    asBigInt(finalization.a_after_bytes, "A_after"),
  ];
  const maximumAllocatedBytes = snapshots.reduce((maximum, value) => value > maximum ? value : maximum, 0n);
  if (identity.capacityBytes > MAX_CAPACITY_BYTES || maximumAllocatedBytes > identity.capacityBytes) fail("filesystem usage exceeds fixed capacity");
  const maximumGapNs = rawRows.slice(1).reduce((maximum, row, index) => {
    const gap = row.sampleStartNs - rawRows[index].sampleStartNs;
    return gap > maximum ? gap : maximum;
  }, 0n);
  return Object.freeze({ identity, rawRows, publicRows, setupDu, shutdownDu, finalization, stopNs, maximumAllocatedBytes, maximumGapNs });
};

const summaryFromChecked = (checked) => {
  const { identity, rawRows, setupDu, shutdownDu, finalization } = checked;
  return Object.freeze({
    outcome: "pass",
    schema: SUMMARY_SCHEMA,
    method: "statfs",
    counter: "f_blocks-f_bfree",
    sampler_origin_ns: decimal(identity.samplerOriginNs, "sampler origin"),
    stop_requested_ns: checked.stopNs.toString(),
    nominal_interval_ns: decimal(identity.nominalIntervalNs, "nominal interval"),
    samples: rawRows.length.toString(),
    initial_mib: rawRows[0].mebibytes.toString(),
    final_mib: rawRows[rawRows.length - 1].mebibytes.toString(),
    maximum_mib: ceilMebibytes(checked.maximumAllocatedBytes).toString(),
    maximum_allocated_bytes: checked.maximumAllocatedBytes.toString(),
    capacity_bytes: identity.capacityBytes.toString(),
    maximum_gap_ns: checked.maximumGapNs.toString(),
    setup_du_mib: setupDu.du_mib,
    shutdown_du_mib: shutdownDu.du_mib,
    du_arguments: ["-sm", "--", identity.mountpoint],
    reservation_length_bytes: finalization.reservation.length_bytes,
    reservation_allocated_bytes: finalization.reservation.allocated_bytes,
    a_before_bytes: finalization.a_before_bytes,
    a_after_bytes: finalization.a_after_bytes,
  });
};

const compareSummary = (provided, expected) => {
  exactKeys(provided, SUMMARY_KEYS, "checkout-disk-summary");
  for (const key of SUMMARY_KEYS) {
    if (JSON.stringify(provided[key]) !== JSON.stringify(expected[key])) fail(`checkout-disk-summary differs at ${key}`);
  }
  return true;
};

export const summarizeFilesystemEvidence = (evidence, expectations = {}) => {
  const checked = parseEvidence(evidence, expectations);
  const summary = summaryFromChecked(checked);
  if (evidence.summary !== undefined) compareSummary(evidence.summary, summary);
  return summary;
};

export const validateFinalCheck = ({ current, closedMaximumBytes, expectations = {} }) => {
  const maximum = asBigInt(closedMaximumBytes, "closed maximum bytes");
  const identity = current && current.accounting ? current : parseSnapshotDocument(current, expectations.identity);
  topologyExpectations(identity, expectations);
  if (identity.accounting.usedBytes > maximum) fail("current statfs usage exceeds closed maximum");
  return Object.freeze({
    schema: EVIDENCE_SCHEMA,
    mode: "final-check",
    outcome: "pass",
    closed_maximum_bytes: maximum.toString(),
    current_used_bytes: identity.accounting.usedBytes.toString(),
    snapshot: filesystemSnapshotDocument(identity),
  });
};

/* Public receipt validation entry point used by the final receipt generator. */
export const validateFilesystemEvidence = (output, topologyExpectationsValue = {}) => {
  let evidence = output;
  if (typeof output === "string") {
    const base = existingCanonicalPath(output, "evidence directory");
    const file = (name) => resolve(base, name);
    evidence = {
      identity: readText(file("identity.json"), "identity evidence"),
      raw: readText(file("raw.tsv"), "raw evidence"),
      public: readText(file("public.tsv"), "public evidence"),
      setupDu: readJson(file("du-setup.json"), "setup du evidence"),
      shutdownDu: readJson(file("du-shutdown.json"), "shutdown du evidence"),
      finalization: readJson(file("release.json"), "release evidence"),
      stop: readText(file("stop"), "stop marker"),
      summary: readJson(file("summary.json"), "checkout-disk-summary"),
    };
  }
  const checked = parseEvidence(evidence, topologyExpectationsValue);
  const summary = summaryFromChecked(checked);
  if (evidence.summary !== undefined) compareSummary(evidence.summary, summary);
  return Object.freeze({ ...checked, summary });
};

const parseCli = (argv) => {
  const values = {};
  for (let index = 0; index < argv.length; index += 1) {
    const key = argv[index];
    if (!key.startsWith("--") || index + 1 >= argv.length || argv[index + 1].startsWith("--")) fail("evidence arguments must be --name value pairs");
    const normalized = key.slice(2).replaceAll("-", "_");
    if (Object.hasOwn(values, normalized)) fail(`duplicate evidence argument: ${key}`);
    values[normalized] = argv[index + 1];
    index += 1;
  }
  return values;
};

const required = (options, name) => {
  if (options[name] === undefined) fail(`evidence requires --${name.replaceAll("_", "-")}`);
  return options[name];
};

const strictCliOptions = (options) => ({
  checkout: required(options, "checkout"),
  target: required(options, "target"),
  output: required(options, "output"),
  device: required(options, "device"),
  majorMinor: required(options, "major_minor"),
  fragmentSize: required(options, "fragment_size"),
  uuid: required(options, "uuid"),
});

const strictSnapshotFromCli = (options) => captureStrictSnapshot(strictCliOptions(options));

const runDuCheck = (options) => {
  const strict = strictCliOptions(options);
  const phase = required(options, "phase");
  if (!DU_PHASES.has(phase)) fail("du phase is invalid");
  const checkout = existingCanonicalPath(strict.checkout, "checkout");
  const started = process.hrtime.bigint();
  const child = spawnSync("ionice", ["-c", "3", "du", "-sm", "--", checkout], {
    cwd: checkout,
    encoding: "utf8",
    shell: false,
    stdio: ["ignore", "pipe", "pipe"],
  });
  const ended = process.hrtime.bigint();
  if (child.error) fail(`du could not be executed: ${child.error.message}`);
  const invocation = {
    phase,
    argv: ["ionice", "-c", "3", "du", "-sm", "--", checkout],
    cwd: checkout,
    status: child.status,
    stdout: child.stdout,
    stderr: child.stderr,
    started_ns: started.toString(),
    ended_ns: ended.toString(),
  };
  // This is intentionally the next filesystem observation after spawnSync;
  // no evidence file or marker is written before this call returns.
  const snapshot = captureStrictSnapshot(strict);
  return duOutputDocument(validateDuCheck(invocation, { checkout, snapshot, expectations: strict }));
};

const runReleaseCheck = (options) => {
  const strict = strictCliOptions(options);
  const reservation = {
    reservation_path: required(options, "reservation_path"),
    reservation_length_bytes: required(options, "reservation_length_bytes"),
    reservation_allocated_bytes: required(options, "reservation_allocated_bytes"),
    a_before_bytes: required(options, "a_before_bytes"),
  };
  // The caller has already unlinked the reservation and completed sync -f.
  // Capture A_after before this process writes any output or receipt file.
  const snapshot = captureStrictSnapshot(strict);
  const validated = validateReleaseCheck({ reservation, aAfterBytes: snapshot.accounting.usedBytes, snapshot, expectations: strict });
  return releaseOutputDocument(validated);
};

const runSummary = (options) => {
  const evidence = {
    identity: readText(required(options, "identity_json"), "identity evidence"),
    raw: readText(required(options, "raw"), "raw evidence"),
    public: readText(required(options, "public"), "public evidence"),
    setupDu: readJson(required(options, "setup_du_json"), "setup du evidence"),
    shutdownDu: readJson(required(options, "shutdown_du_json"), "shutdown du evidence"),
    finalization: readJson(required(options, "finalization_json"), "finalization evidence"),
    stop: readText(required(options, "stop"), "stop marker"),
  };
  return summarizeFilesystemEvidence(evidence, {
    checkout: options.checkout,
    target: options.target,
    output: options.output,
    device: options.device,
    majorMinor: options.major_minor,
    uuid: options.uuid,
    fragmentSize: options.fragment_size,
    nominalIntervalNs: options.nominal_interval_ns,
  });
};

const runFinalCheck = (options) => {
  const strict = strictCliOptions(options);
  const closedMaximumBytes = required(options, "closed_maximum_bytes");
  const snapshot = strictSnapshotFromCli(options);
  return validateFinalCheck({ current: snapshot, closedMaximumBytes, expectations: strict });
};

export const runEvidenceCli = (argv = process.argv.slice(2)) => {
  const mode = argv[0];
  if (!["du-check", "release-check", "summarize", "final-check"].includes(mode)) {
    process.stderr.write("usage: r2-filesystem-evidence.mjs {du-check|release-check|summarize|final-check} ...\n");
    return 2;
  }
  try {
    const options = parseCli(argv.slice(1));
    const result = mode === "du-check"
      ? runDuCheck(options)
      : mode === "release-check"
        ? runReleaseCheck(options)
        : mode === "summarize"
          ? runSummary(options)
          : runFinalCheck(options);
    process.stdout.write(canonicalJson(result));
    return 0;
  } catch (error) {
    process.stderr.write(`${error instanceof Error ? error.message : String(error)}\n`);
    return 1;
  }
};

if (process.argv[1] && resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  Promise.resolve(runEvidenceCli()).then((status) => { process.exitCode = status; });
}
