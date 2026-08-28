import assert from "node:assert/strict";
import { mkdtempSync, readdirSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import test from "node:test";
import { tmpdir } from "node:os";
import { join } from "node:path";

import {
  MAX_SAMPLE_GAP_NS,
  RAW_COLUMNS,
  RAW_HEADER,
  SAMPLE_PERIOD_NS,
  XFS_MAGIC,
  SampleGapError,
  accountingFromStatfs,
  assertStableFilesystemIdentity,
  captureFilesystemIdentity,
  ceilMebibytes,
  parseMountInfo,
  parseRawRow,
  publicTsvFromRaw,
  rawRow,
  runAbsoluteSchedule,
  validateExactR2MountTopology,
  validateAbsoluteSchedule,
  validateMountTopology,
  validateRawLedger,
} from "./r2-filesystem-accounting.mjs";
import {
  PreallocatedRawLedger,
  assertFreshMarker,
  publishMarker,
  readMarker,
} from "./r2-filesystem-sampler.mjs";

const UUID = "11111111-2222-3333-4444-555555555555";
const MOUNTINFO = [
  "30 1 8:1 / / rw,relatime - ext4 /dev/root rw",
  "31 30 252:7 / /mnt/proof\\040volume rw,relatime - xfs /dev/mapper/proof\\040volume rw",
  "32 31 252:7 /checkout /mnt/proof\\040volume/checkout rw,relatime - xfs /dev/mapper/proof\\040volume rw",
].join("\n") + "\n";

const STRICT_MOUNTINFO = [
  "30 1 8:1 / / rw,relatime - ext4 /dev/root rw",
  "31 30 7:7 / /mnt/proof\\040volume rw,relatime - xfs /dev/loop7 rw",
  "32 31 7:7 /checkout /mnt/proof\\040volume/checkout ro,nodev,nosuid,relatime - xfs /dev/loop7 ro",
  "33 32 7:7 /checkout/target /mnt/proof\\040volume/checkout/target rw,nodev,nosuid,relatime - xfs /dev/loop7 rw",
  "34 32 7:7 /checkout/output /mnt/proof\\040volume/checkout/output rw,nodev,nosuid,relatime - xfs /dev/loop7 rw",
].join("\n") + "\n";

const statfsFixture = (overrides = {}) => ({
  type: XFS_MAGIC,
  bsize: 4096n,
  blocks: 1_000_000n,
  bfree: 875_000n,
  bavail: 874_000n,
  ...overrides,
});

const identityFixture = () => {
  const accounting = accountingFromStatfs(statfsFixture(), { fragmentSize: 4096n });
  return {
    schema: 1,
    mountId: "32",
    mountpoint: "/mnt/proof volume/checkout",
    mountRoot: "/checkout",
    mountOptions: "ro,nodev,nosuid,relatime",
    source: "/dev/mapper/proof volume",
    majorMinor: "252:7",
    mountFilesystemType: "xfs",
    uuid: UUID,
    sourceDevice: "/dev/mapper/proof volume",
    sourceMajorMinor: "252:7",
    filesystemType: XFS_MAGIC,
    filesystemMagic: "0x58465342",
    rootDevice: 252007n,
    rootInode: 123n,
    fragmentSize: 4096n,
    blockSize: 4096n,
    capacityBlocks: accounting.blocks,
    capacityBytes: accounting.capacityBytes,
    dedicatedFixedCapacity: true,
    targetPath: "/mnt/proof volume/checkout/target",
    outputPath: "/mnt/proof volume/checkout/output",
    nestedMounts: [],
    accounting,
  };
};

test("XFS accounting uses exact BigInt counters and fragment-size equality", () => {
  const accounting = accountingFromStatfs(statfsFixture(), { fragmentSize: 4096n });
  assert.equal(accounting.filesystemType, XFS_MAGIC);
  assert.equal(accounting.capacityBytes, 4_096_000_000n);
  assert.equal(accounting.usedBlocks, 125_000n);
  assert.equal(accounting.usedBytes, 512_000_000n);
  assert.equal(accounting.mebibytes, 489n);
  assert.equal(ceilMebibytes(1_048_577n), 2n);
  assert.throws(
    () => accountingFromStatfs(statfsFixture(), { fragmentSize: 512n }),
    /fragment size 512 differs from statfs block size 4096/,
  );
  assert.throws(
    () => accountingFromStatfs(statfsFixture({ bavail: 900_000n }), { fragmentSize: 4096n }),
    /out of range/,
  );
  assert.throws(
    () => accountingFromStatfs({ ...statfsFixture(), type: 0xEF53n }, { fragmentSize: 4096n }),
    /not XFS/,
  );
});

test("mountinfo decodes escaped paths and validates exact covering/nested mounts", () => {
  const mounts = parseMountInfo(MOUNTINFO);
  assert.equal(mounts[1].mountpoint, "/mnt/proof volume");
  assert.equal(mounts[1].source, "/dev/mapper/proof volume");
  assert.equal(mounts[2].root, "/checkout");
  const topology = validateMountTopology({
    mounts,
    checkout: "/mnt/proof volume/checkout",
    expected: {
      mountId: "32",
      mountpoint: "/mnt/proof volume/checkout",
      source: "/dev/mapper/proof volume",
      majorMinor: "252:7",
      filesystemType: "xfs",
      uuid: UUID,
    },
    uuidByMountpoint: new Map([["/mnt/proof volume/checkout", UUID]]),
  });
  assert.equal(topology.coveringMount.mountId, "32");
  assert.deepEqual(topology.nestedMounts, []);
  assert.throws(
    () => validateMountTopology({
      mounts,
      checkout: "/mnt/proof volume",
      expected: {
        mountId: "31",
        mountpoint: "/mnt/proof volume",
        source: "/dev/mapper/proof volume",
        majorMinor: "252:7",
        filesystemType: "xfs",
        uuid: UUID,
      },
      uuidByMountpoint: new Map([["/mnt/proof volume", UUID]]),
    }),
    /unexpected nested mount/,
  );
});

test("strict R2 topology admits only the canonical checkout, target, and output binds", () => {
  const common = {
    checkout: "/mnt/proof volume/checkout",
    target: "/mnt/proof volume/checkout/target",
    output: "/mnt/proof volume/checkout/output",
    device: "/dev/loop7",
    majorMinor: "7:7",
    uuid: UUID,
    uuidByMountpoint: new Map([
      ["/mnt/proof volume/checkout", UUID],
      ["/mnt/proof volume/checkout/target", UUID],
      ["/mnt/proof volume/checkout/output", UUID],
    ]),
  };
  const admitted = validateExactR2MountTopology({ mounts: parseMountInfo(STRICT_MOUNTINFO), ...common });
  assert.deepEqual(admitted.nestedMounts.map((mount) => mount.mountpoint), [
    "/mnt/proof volume/checkout/output",
    "/mnt/proof volume/checkout/target",
  ]);
  const baseline = captureFilesystemIdentity("/ignored", {
    fragmentSize: 4096n,
    uuid: UUID,
    realpathReader: () => common.checkout,
    statReader: () => ({ dev: 7007n, ino: 123n, isDirectory: () => true }),
    statfsReader: () => statfsFixture({ blocks: 1_000_000n }),
    mountInfoReader: () => STRICT_MOUNTINFO,
    target: common.target,
    output: common.output,
    device: common.device,
    majorMinor: common.majorMinor,
    uuidByMountpoint: common.uuidByMountpoint,
    capacityLimitBytes: null,
  });
  assert.equal(baseline.mountRoot, "/checkout");
  assert.equal(baseline.mountOptions, "ro,nodev,nosuid,relatime");
  assert.equal(baseline.nestedMounts.length, 2);
  assertStableFilesystemIdentity(baseline, baseline);
  assert.throws(
    () => assertStableFilesystemIdentity(baseline, {
      ...baseline,
      nestedMounts: [{ ...baseline.nestedMounts[0], mountId: "99" }, baseline.nestedMounts[1]],
    }),
    /nestedMounts/,
  );
  const cases = [
    ["absent nested mount", STRICT_MOUNTINFO.replace(/34 32[^\n]*\n/, ""), /exactly two nested mounts/],
    ["extra nested mount", `${STRICT_MOUNTINFO}35 32 7:7 /checkout/cache /mnt/proof\\040volume/checkout/cache rw,nodev,nosuid,relatime - xfs /dev/loop7 rw\n`, /exactly two nested mounts/],
    ["wrong nested root", STRICT_MOUNTINFO.replace("/checkout/target /mnt/proof", "/wrong /mnt/proof"), /nested mount root changed/],
    ["wrong source device", STRICT_MOUNTINFO.replaceAll("/dev/loop7", "/dev/other"), /source device changed/],
    ["wrong root options", STRICT_MOUNTINFO.replace("ro,nodev,nosuid,relatime", "ro,nodev,relatime"), /missing mount option nosuid/],
    ["wrong nested options", STRICT_MOUNTINFO.replace("/mnt/proof\\040volume/checkout/target rw,nodev,nosuid,relatime", "/mnt/proof\\040volume/checkout/target rw,nodev,relatime"), /missing mount option nosuid/],
  ];
  for (const [label, text, pattern] of cases) {
    assert.throws(
      () => validateExactR2MountTopology({ mounts: parseMountInfo(text), ...common }),
      pattern,
      label,
    );
  }
});

test("capture records root identity and rejects identity drift", () => {
  const snapshot = statfsFixture();
  const rootStat = { dev: 252007n, ino: 123n, isDirectory: () => true };
  const baseline = captureFilesystemIdentity("/ignored", {
    fragmentSize: 4096n,
    uuid: UUID,
    realpathReader: () => "/mnt/proof volume/checkout",
    statReader: () => rootStat,
    statfsReader: () => snapshot,
    mountInfoReader: () => MOUNTINFO,
    capacityLimitBytes: null,
    uuidByMountpoint: new Map([["/mnt/proof volume/checkout", UUID]]),
  });
  assert.equal(baseline.rootDevice, 252007n);
  assert.equal(baseline.rootInode, 123n);
  assert.equal(baseline.fragmentSize, 4096n);
  assertStableFilesystemIdentity(baseline, baseline);
  assert.throws(
    () => assertStableFilesystemIdentity(baseline, { ...baseline, capacityBlocks: baseline.capacityBlocks + 1n }),
    /capacityBlocks/,
  );
});

test("raw TSV binds mount identity, fragment size, observed block size, and derived counters", () => {
  const identity = identityFixture();
  const line = rawRow({
    ordinal: 0n,
    sampleStartNs: 10_000n,
    elapsedNs: 0n,
    deadlineNs: 10_000n,
    identity,
    kind: "scheduled",
  });
  assert.equal(line.split("\t").length, RAW_COLUMNS.length);
  assert.equal(line.split("\t")[5], "/mnt/proof\\040volume/checkout");
  assert.equal(line.split("\t")[8], "/dev/mapper/proof\\040volume");
  const parsed = parseRawRow(line);
  assert.equal(parsed.fragmentSize, 4096n);
  assert.equal(parsed.blockSize, 4096n);
  assert.equal(parsed.usedBlocks, 125_000n);
  assert.equal(parsed.mebibytes, 489n);
  const ledger = new PreallocatedRawLedger(4);
  ledger.append(line);
  ledger.append(rawRow({
    ordinal: 1n,
    sampleStartNs: 20_000n,
    elapsedNs: 10_000n,
    deadlineNs: 20_000n,
    identity,
    kind: "terminal",
  }));
  const text = ledger.text();
  assert.ok(text.startsWith(`${RAW_HEADER}\n`));
  const rows = validateRawLedger(text, identity, {
    originNs: 10_000n,
    periodNs: 10_000n,
    maxGapNs: 20_000n,
  });
  assert.equal(rows.length, 2);
  assert.match(publicTsvFromRaw(rows), /ordinal\tsample_start_ns\telapsed_ns\tmebibytes\tkind/);
  const forgedDeadline = line.replace("\t10000\t32", "\t10001\t32");
  assert.throws(
    () => validateRawLedger(`${RAW_HEADER}\n${forgedDeadline}\n${ledger.lines()[1]}\n`, identity, {
      originNs: 10_000n,
      periodNs: 10_000n,
      maxGapNs: 20_000n,
    }),
    /scheduled deadline is inconsistent/,
  );
  assert.throws(() => new PreallocatedRawLedger(1), /capacity is invalid/);
  assert.throws(() => {
    const exhausted = new PreallocatedRawLedger(2);
    exhausted.append(line);
    exhausted.append(line.replace("\t0\t10000\t", "\t1\t20000\t"));
    exhausted.append(line);
  }, /capacity exhausted/);
});

test("raw ledger rejects non-canonical line termination and row whitespace", () => {
  const identity = identityFixture();
  const ledger = new PreallocatedRawLedger(4);
  ledger.append(rawRow({
    ordinal: 0n,
    sampleStartNs: 10_000n,
    elapsedNs: 0n,
    deadlineNs: 10_000n,
    identity,
    kind: "scheduled",
  }));
  ledger.append(rawRow({
    ordinal: 1n,
    sampleStartNs: 20_000n,
    elapsedNs: 10_000n,
    deadlineNs: 20_000n,
    identity,
    kind: "terminal",
  }));
  const valid = ledger.text();
  const cases = [
    ["missing terminal LF", valid.slice(0, -1), /one canonical newline-terminated stream/],
    ["extra terminal LF", `${valid}\n`, /one canonical newline-terminated stream/],
    ["blank interior row", valid.replace("\n1\t", "\n\n1\t"), /contains a blank row/],
    ["trailing row whitespace", valid.replace("\n1\t", " \n1\t"), /trailing whitespace/],
  ];
  for (const [label, malformed, pattern] of cases) {
    assert.throws(
      () => validateRawLedger(malformed, identity, {
        originNs: 10_000n,
        periodNs: 10_000n,
        maxGapNs: 20_000n,
      }),
      pattern,
      label,
    );
  }
  const trailingFieldWhitespace = valid.replace("\tterminal\n", "\tterminal \n");
  assert.throws(
    () => validateRawLedger(trailingFieldWhitespace, identity, {
      originNs: 10_000n,
      periodNs: 10_000n,
      maxGapNs: 20_000n,
    }),
    /trailing whitespace/,
  );
});

test("absolute scheduler retains its origin when one sample is delayed", async () => {
  let now = 1_000_000_000n;
  let sleeps = 0;
  const result = await runAbsoluteSchedule({
    clock: () => now,
    sleep: async (deadline) => {
      if (now < deadline) now = deadline;
      if (sleeps === 1) now += 10_000_000n;
      sleeps += 1;
    },
    snapshot: ({ sampleStartNs }) => sampleStartNs,
    originNs: now,
    periodNs: SAMPLE_PERIOD_NS,
    maxGapNs: MAX_SAMPLE_GAP_NS,
    maxSamples: 3,
  });
  assert.deepEqual(result.rows.map((row) => row.deadlineNs), [
    1_000_000_000n,
    1_050_000_000n,
    1_100_000_000n,
  ]);
  assert.deepEqual(result.rows.map((row) => row.sampleStartNs), [
    1_000_000_000n,
    1_060_000_000n,
    1_100_000_000n,
  ]);
  assert.doesNotThrow(() => validateAbsoluteSchedule(result.rows, {
    originNs: 1_000_000_000n,
    periodNs: SAMPLE_PERIOD_NS,
    maxGapNs: MAX_SAMPLE_GAP_NS,
  }));
});

test("absolute scheduler makes a gap over 100 ms a red result", async () => {
  let now = 0n;
  let sleeps = 0;
  await assert.rejects(
    () => runAbsoluteSchedule({
      clock: () => now,
      sleep: async (deadline) => {
        now = deadline;
        if (sleeps === 1) now += SAMPLE_PERIOD_NS + 1n;
        sleeps += 1;
      },
      snapshot: () => null,
      originNs: 0n,
      maxSamples: 3,
    }),
    (error) => error instanceof SampleGapError && error.gapNs === MAX_SAMPLE_GAP_NS + 1n,
  );
});

test("control markers are fresh, atomic, and exactly one canonical line", () => {
  const directory = mkdtempSync(join(tmpdir(), "nomos-r2-marker-"));
  const marker = join(directory, "ready");
  try {
    assertFreshMarker(marker);
    publishMarker(marker, 42n);
    assert.equal(readFileSync(marker, "utf8"), "42\n");
    assert.equal(readMarker(marker), 42n);
    assert.throws(() => publishMarker(marker, 43n), /not fresh/);
    writeFileSync(marker, "42");
    assert.throws(() => readMarker(marker), /malformed control marker/);
    assert.throws(() => assertFreshMarker(marker), /not fresh/);
  } finally {
    rmSync(directory, { recursive: true, force: true });
  }
});

test("control marker publication refuses a competing publisher after freshness check", () => {
  const directory = mkdtempSync(join(tmpdir(), "nomos-r2-marker-race-"));
  const marker = join(directory, "ready");
  try {
    assert.throws(
      () => publishMarker(marker, 42n, {
        beforeLink: () => writeFileSync(marker, "99\n", { flag: "wx" }),
      }),
      /not fresh/,
    );
    assert.equal(readFileSync(marker, "utf8"), "99\n");
    assert.deepEqual(readdirSync(directory), ["ready"]);
  } finally {
    rmSync(directory, { recursive: true, force: true });
  }
});
