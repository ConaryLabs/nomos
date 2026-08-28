import assert from "node:assert/strict";
import test from "node:test";

import {
  RAW_HEADER,
  XFS_MAGIC,
  accountingFromStatfs,
  filesystemIdentityDocument,
  publicTsvFromRaw,
  rawRow,
} from "./r2-filesystem-accounting.mjs";
import {
  EVIDENCE_SCHEMA,
  RESERVATION_LENGTH_BYTES,
  SUMMARY_SCHEMA,
  DU_ARGV,
  filesystemSnapshotDocument,
  summarizeFilesystemEvidence,
  validateDuCheck,
  validateFinalCheck,
  validateFilesystemEvidence,
  validateReleaseCheck,
} from "./r2-filesystem-evidence.mjs";

const UUID = "11111111-2222-3333-4444-555555555555";
const CHECKOUT = "/mnt/proof/checkout";
const TARGET = `${CHECKOUT}/target`;
const OUTPUT = `${CHECKOUT}/output`;
const DEVICE = "/dev/loop7";
const MAJOR_MINOR = "7:7";
const ORIGIN = 1_000n;
const PERIOD = 10_000n;

const statfsFixture = (overrides = {}) => ({
  type: XFS_MAGIC,
  bsize: 4096n,
  blocks: 1_000_000n,
  bfree: 875_000n,
  bavail: 874_000n,
  ...overrides,
});

const identityFixture = (overrides = {}) => {
  const accounting = accountingFromStatfs(statfsFixture(overrides.statfs), { fragmentSize: 4096n });
  return {
    schema: 1,
    mountId: 32n,
    mountpoint: CHECKOUT,
    mountRoot: "/checkout",
    mountOptions: "ro,nodev,nosuid,relatime",
    source: DEVICE,
    majorMinor: MAJOR_MINOR,
    mountFilesystemType: "xfs",
    uuid: UUID,
    sourceDevice: DEVICE,
    sourceMajorMinor: MAJOR_MINOR,
    filesystemType: XFS_MAGIC,
    filesystemMagic: "0x58465342",
    rootDevice: 7007n,
    rootInode: 123n,
    fragmentSize: 4096n,
    blockSize: 4096n,
    capacityBlocks: accounting.blocks,
    capacityBytes: accounting.capacityBytes,
    dedicatedFixedCapacity: true,
    targetPath: TARGET,
    outputPath: OUTPUT,
    nestedMounts: [
      { mountId: 33n, mountpoint: TARGET, root: "/checkout/target", mountOptions: "rw,nodev,nosuid,relatime", source: DEVICE, majorMinor: MAJOR_MINOR, filesystemType: "xfs", uuid: UUID },
      { mountId: 34n, mountpoint: OUTPUT, root: "/checkout/output", mountOptions: "rw,nodev,nosuid,relatime", source: DEVICE, majorMinor: MAJOR_MINOR, filesystemType: "xfs", uuid: UUID },
    ],
    accounting,
  };
};

const identity = identityFixture();
const identityAfter = identityFixture({ statfs: { bfree: 879_096n, bavail: 878_096n } });
const topology = {
  checkout: CHECKOUT,
  target: TARGET,
  output: OUTPUT,
  device: DEVICE,
  majorMinor: MAJOR_MINOR,
  uuid: UUID,
  fragmentSize: 4096n,
};

const rowsFixture = (lastStart = 21_000n) => [
  rawRow({ ordinal: 0n, sampleStartNs: ORIGIN, elapsedNs: 0n, deadlineNs: ORIGIN, identity, kind: "scheduled" }),
  rawRow({ ordinal: 1n, sampleStartNs: ORIGIN + PERIOD, elapsedNs: PERIOD, deadlineNs: ORIGIN + PERIOD, identity, kind: "scheduled" }),
  rawRow({ ordinal: 2n, sampleStartNs: lastStart, elapsedNs: lastStart - ORIGIN, deadlineNs: lastStart, identity, kind: "terminal" }),
];

const duFixture = (phase, mib = "489", snapshot = identity) => ({
  schema: EVIDENCE_SCHEMA,
  mode: "du-check",
  phase,
  invocation: {
    argv: [...DU_ARGV, CHECKOUT],
    cwd: CHECKOUT,
    status: 0,
    stdout: `${mib}\t${CHECKOUT}\n`,
    stderr: "",
    started_ns: "10",
    ended_ns: "20",
  },
  du_mib: mib,
  snapshot: filesystemSnapshotDocument(snapshot),
});

const releaseFixture = (snapshot = identityAfter, changes = {}) => ({
  schema: EVIDENCE_SCHEMA,
  mode: "release-check",
  reservation: {
    path: `${OUTPUT}/host/finalization.reserve`,
    length_bytes: RESERVATION_LENGTH_BYTES.toString(),
    allocated_bytes: RESERVATION_LENGTH_BYTES.toString(),
  },
  a_before_bytes: identity.accounting.usedBytes.toString(),
  a_after_bytes: snapshot.accounting.usedBytes.toString(),
  snapshot: filesystemSnapshotDocument(snapshot),
  ...changes,
});

const evidenceFixture = (changes = {}) => {
  const rawRows = rowsFixture();
  const evidence = {
    identity: JSON.parse(filesystemIdentityDocument(identity, {
      samplerOriginNs: ORIGIN,
      nominalIntervalNs: PERIOD,
    })),
    raw: `${RAW_HEADER}\n${rawRows.join("\n")}\n`,
    public: publicTsvFromRaw(rawRows),
    setupDu: duFixture("setup"),
    shutdownDu: duFixture("shutdown"),
    finalization: releaseFixture(),
    stop: "20500\n",
  };
  return { ...evidence, ...changes };
};

test("du-check retains the exact invocation and bounds recursive du by immediate statfs", () => {
  const invocation = {
    phase: "setup",
    argv: [...DU_ARGV, CHECKOUT],
    cwd: CHECKOUT,
    status: 0,
    stdout: `489\t${CHECKOUT}\n`,
    stderr: "",
    started_ns: "10",
    ended_ns: "20",
  };
  const checked = validateDuCheck(invocation, { checkout: CHECKOUT, snapshot: identity, expectations: topology });
  assert.equal(checked.du_mib, "489");
  assert.equal(checked.snapshot.accounting.usedBytes, 512_000_000n);
  for (const forged of [
    { ...invocation, stdout: `0489\t${CHECKOUT}\n` },
    { ...invocation, stderr: "warning\n" },
    { ...invocation, argv: ["du", "-sm", "--", CHECKOUT] },
    { ...invocation, stdout: "489\t/other\n" },
  ]) assert.throws(() => validateDuCheck(forged, { checkout: CHECKOUT, snapshot: identity, expectations: topology }));
  assert.throws(() => validateDuCheck({ ...invocation, stdout: "513\t/mnt/proof/checkout\n" }, { checkout: CHECKOUT, snapshot: identity, expectations: topology }), /exceeds/);
});

test("release-check enforces the exact reservation and both allocation inequalities", () => {
  const checked = validateReleaseCheck({
    reservation: {
      reservation_path: `${OUTPUT}/host/finalization.reserve`,
      reservation_length_bytes: RESERVATION_LENGTH_BYTES.toString(),
      reservation_allocated_bytes: RESERVATION_LENGTH_BYTES.toString(),
      a_before_bytes: identity.accounting.usedBytes.toString(),
    },
    aAfterBytes: identityAfter.accounting.usedBytes,
    snapshot: identityAfter,
    expectations: topology,
  });
  assert.equal(checked.aAfter, identityAfter.accounting.usedBytes);
  assert.throws(() => validateReleaseCheck({
    reservation: {
      reservation_path: `${OUTPUT}/host/finalization.reserve`,
      reservation_length_bytes: "16777215",
      reservation_allocated_bytes: RESERVATION_LENGTH_BYTES.toString(),
      a_before_bytes: identity.accounting.usedBytes.toString(),
    },
    aAfterBytes: identityAfter.accounting.usedBytes,
    snapshot: identityAfter,
    expectations: topology,
  }), /exactly 16777216/);
  assert.throws(() => validateReleaseCheck({
    reservation: {
      reservation_path: `${OUTPUT}/host/finalization.reserve`,
      reservation_length_bytes: RESERVATION_LENGTH_BYTES.toString(),
      reservation_allocated_bytes: RESERVATION_LENGTH_BYTES.toString(),
      a_before_bytes: identity.accounting.usedBytes.toString(),
    },
    aAfterBytes: identity.accounting.usedBytes,
    snapshot: identity,
    expectations: topology,
  }), /A_after/);
});

test("summary validates identity, raw/public parity, deadlines, stop ordering, capacity, and crosscheck maximum", () => {
  const summary = summarizeFilesystemEvidence(evidenceFixture(), topology);
  assert.equal(summary.schema, SUMMARY_SCHEMA);
  assert.equal(summary.outcome, "pass");
  assert.equal(summary.samples, "3");
  assert.equal(summary.initial_mib, "489");
  assert.equal(summary.final_mib, "489");
  assert.equal(summary.maximum_mib, "489");
  assert.equal(summary.maximum_allocated_bytes, "512000000");
  assert.equal(summary.capacity_bytes, "4096000000");
  assert.equal(summary.maximum_gap_ns, "10000");
  assert.equal(summary.a_before_bytes, "512000000");
  assert.equal(summary.a_after_bytes, identityAfter.accounting.usedBytes.toString());

  const forgedDeadline = rowsFixture().map((line, index) => index === 1 ? line.replace("\t11000\t10000\t11000\t32", "\t11000\t10000\t11001\t32") : line);
  assert.throws(() => summarizeFilesystemEvidence(evidenceFixture({
    raw: `${RAW_HEADER}\n${forgedDeadline.join("\n")}\n`,
  }), topology), /deadline/);

  const forgedGapRows = rowsFixture(120_000_001n);
  assert.throws(() => summarizeFilesystemEvidence(evidenceFixture({
    raw: `${RAW_HEADER}\n${forgedGapRows.join("\n")}\n`,
    public: publicTsvFromRaw(forgedGapRows),
  }), topology), /gap/);
  assert.throws(() => summarizeFilesystemEvidence(evidenceFixture({ stop: "22000\n" }), topology), /terminal sample/);
  assert.throws(() => summarizeFilesystemEvidence(evidenceFixture({
    finalization: releaseFixture(identityAfter, { a_after_bytes: identity.accounting.usedBytes.toString() }),
  }), topology), /A_after/);
  assert.throws(() => summarizeFilesystemEvidence(evidenceFixture({
    public: publicTsvFromRaw(rowsFixture()).replace("489\tscheduled", "490\tscheduled"),
  }), topology), /public ledger mismatch/);
});

test("filesystem receipt validation returns normalized evidence and rejects identity/counter forgeries", () => {
  const checked = validateFilesystemEvidence(evidenceFixture(), topology);
  assert.equal(checked.rawRows.length, 3);
  assert.equal(checked.stopNs, 20_500n);
  assert.equal(checked.maximumAllocatedBytes, 512_000_000n);
  const withSummary = evidenceFixture({ summary: summarizeFilesystemEvidence(evidenceFixture(), topology) });
  assert.equal(validateFilesystemEvidence(withSummary, topology).summary.maximum_mib, "489");
  assert.throws(() => validateFilesystemEvidence({ ...withSummary, summary: { ...withSummary.summary, maximum_mib: "490" } }, topology), /summary differs/);
  const forged = evidenceFixture({
    identity: { ...evidenceFixture().identity, capacity_bytes: "4096000001" },
  });
  assert.throws(() => validateFilesystemEvidence(forged, topology));
  const snapshot = duFixture("setup");
  snapshot.snapshot = { ...snapshot.snapshot, used_bytes: "512000001" };
  assert.throws(() => validateFilesystemEvidence(evidenceFixture({ setupDu: snapshot }), topology), /snapshot used_bytes/);
});

test("final-check is a no-write closed-maximum guard", () => {
  const result = validateFinalCheck({ current: identityAfter, closedMaximumBytes: identity.accounting.usedBytes, expectations: topology });
  assert.equal(result.outcome, "pass");
  assert.equal(result.current_used_bytes, identityAfter.accounting.usedBytes.toString());
  assert.throws(() => validateFinalCheck({ current: identity, closedMaximumBytes: identity.accounting.usedBytes - 1n, expectations: topology }), /closed maximum/);
});
