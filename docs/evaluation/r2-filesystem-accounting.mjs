import {
  readFileSync,
  realpathSync,
  statSync,
  statfsSync,
} from "node:fs";
import { normalize } from "node:path";

/*
 * The proof wrapper supplies the XFS fragment size and UUID. Node's statfs
 * binding exposes f_bsize but not f_frsize or fsid, so accepting those two
 * values from the wrapper is deliberate. The sampler never shells out and
 * never walks the checkout.
 */

export const XFS_MAGIC = 1_481_003_842n;
export const XFS_MAGIC_HEX = "0x58465342";
export const MEBIBYTE = 1_048_576n;
export const SAMPLE_PERIOD_NS = 50_000_000n;
export const MAX_SAMPLE_GAP_NS = 100_000_000n;
export const MAX_CAPACITY_BYTES = 8_589_934_592n;
export const RAW_COLUMNS = Object.freeze([
  "ordinal",
  "sample_start_ns",
  "elapsed_ns",
  "deadline_ns",
  "mount_id",
  "mountpoint",
  "mount_root",
  "mount_options",
  "source",
  "major_minor",
  "uuid",
  "filesystem_type",
  "device",
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
  "kind",
]);
export const RAW_HEADER = RAW_COLUMNS.join("\t");
export const PUBLIC_HEADER = "ordinal\tsample_start_ns\telapsed_ns\tmebibytes\tkind";

const UUID_PATTERN = /^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$/i;
const DECIMAL_PATTERN = /^(0|[1-9][0-9]*)$/;
const KINDS = new Set(["scheduled", "terminal"]);

export class FilesystemAccountingError extends Error {
  constructor(message) {
    super(message);
    this.name = "FilesystemAccountingError";
  }
}

export class SampleGapError extends FilesystemAccountingError {
  constructor(gapNs, limitNs = MAX_SAMPLE_GAP_NS) {
    super(`R2 disk sampler: retained sample-start gap exceeds ${limitNs} ns`);
    this.name = "SampleGapError";
    this.gapNs = gapNs;
    this.limitNs = limitNs;
  }
}

const fail = (message) => {
  throw new FilesystemAccountingError(message);
};

export const canonicalUnsignedDecimal = (value, label = "decimal") => {
  if (typeof value === "bigint") return value >= 0n ? value.toString() : fail(`${label} must be unsigned`);
  if (typeof value !== "string" || !DECIMAL_PATTERN.test(value)) {
    fail(`${label} is not a canonical unsigned decimal`);
  }
  return value;
};

const asBigInt = (value, label) => {
  if (typeof value === "bigint") return value >= 0n ? value : fail(`${label} must be unsigned`);
  if (typeof value === "string" && DECIMAL_PATTERN.test(value)) return BigInt(value);
  fail(`${label} must be a BigInt or canonical decimal string`);
};

const field = (object, names, label) => {
  for (const name of names) {
    if (object && object[name] !== undefined) return object[name];
  }
  fail(`missing ${label}`);
};

const mountPath = (value, label = "mountpoint") => {
  if (typeof value !== "string" || value.length === 0 || !value.startsWith("/")) {
    fail(`${label} must be an absolute path`);
  }
  return normalize(value);
};

const safeToken = (value, label) => {
  if (typeof value !== "string" || value.length === 0 || /[\t\r\n]/.test(value)) {
    fail(`${label} is not a safe text field`);
  }
  return value;
};

export const normalizeUuid = (value) => {
  const uuid = safeToken(value, "UUID").toLowerCase();
  if (!UUID_PATTERN.test(uuid)) fail("UUID is not canonical XFS form");
  return uuid;
};

export const decodeMountInfoToken = (token) => {
  if (typeof token !== "string") fail("mountinfo token is not text");
  let decoded = "";
  for (let index = 0; index < token.length; index += 1) {
    if (token[index] !== "\\") {
      decoded += token[index];
      continue;
    }
    const octal = token.slice(index + 1, index + 4);
    if (!/^[0-7]{3}$/.test(octal)) fail("malformed mountinfo escape");
    decoded += String.fromCharCode(Number.parseInt(octal, 8));
    index += 3;
  }
  return decoded;
};

export const encodeMountInfoToken = (value, label = "mountinfo token") => {
  if (typeof value !== "string" || value.length === 0 || value.includes("\r")) {
    fail(`${label} is not a safe text field`);
  }
  let encoded = "";
  for (const character of value) {
    if (character === "\\") encoded += "\\134";
    else if (character === " ") encoded += "\\040";
    else if (character === "\t") encoded += "\\011";
    else if (character === "\n") encoded += "\\012";
    else encoded += character;
  }
  return encoded;
};

const parseMountId = (value, label) => canonicalUnsignedDecimal(value, label);

export const parseMountInfo = (text) => {
  if (typeof text !== "string") fail("mountinfo is not text");
  return text.split("\n").filter((line) => line.length > 0).map((line) => {
    const separator = line.indexOf(" - ");
    if (separator < 0) fail("mountinfo line has no separator");
    const left = line.slice(0, separator).split(" ");
    const right = line.slice(separator + 3).split(" ");
    if (left.length < 6 || right.length < 3) fail("mountinfo line has insufficient fields");
    const majorMinor = left[2];
    if (!/^\d+:\d+$/.test(majorMinor)) fail("mountinfo major:minor is malformed");
    return Object.freeze({
      mountId: parseMountId(left[0], "mount ID"),
      parentId: parseMountId(left[1], "mount parent ID"),
      majorMinor,
      root: decodeMountInfoToken(left[3]),
      mountpoint: mountPath(decodeMountInfoToken(left[4])),
      mountOptions: left[5],
      optionalFields: Object.freeze(left.slice(6)),
      filesystemType: safeToken(right[0], "filesystem type"),
      source: decodeMountInfoToken(right[1]),
      superOptions: right.slice(2).join(" "),
    });
  });
};

const coveredBy = (candidate, parent) => {
  const child = mountPath(candidate, "candidate mountpoint");
  const root = mountPath(parent, "covering mountpoint");
  return root === "/" ? child.startsWith("/") : child === root || child.startsWith(`${root}/`);
};

export const coveringMount = (mounts, target) => {
  const absoluteTarget = mountPath(target, "checkout");
  const candidates = mounts.filter((mount) => coveredBy(absoluteTarget, mount.mountpoint));
  candidates.sort((left, right) => right.mountpoint.length - left.mountpoint.length);
  if (candidates.length === 0) fail(`no mount covers ${absoluteTarget}`);
  return candidates[0];
};

export const nestedMounts = (mounts, target) => {
  const absoluteTarget = mountPath(target, "checkout");
  return mounts
    .filter((mount) => mount.mountpoint !== absoluteTarget && coveredBy(mount.mountpoint, absoluteTarget))
    .sort((left, right) => left.mountpoint.localeCompare(right.mountpoint));
};

const mountUuid = (mount, uuidByMountpoint) => {
  if (mount.uuid !== undefined && mount.uuid !== null) return normalizeUuid(mount.uuid);
  if (uuidByMountpoint instanceof Map && uuidByMountpoint.has(mount.mountpoint)) {
    return normalizeUuid(uuidByMountpoint.get(mount.mountpoint));
  }
  return null;
};

const compareMountField = (actual, expected, label) => {
  if (expected !== undefined && expected !== null && actual !== expected) {
    fail(`${label} changed: expected ${expected}, got ${actual}`);
  }
};

const mountIdentity = (mount, uuidByMountpoint) => ({
  mountId: mount.mountId,
  mountpoint: mount.mountpoint,
  root: mount.root,
  mountOptions: mount.mountOptions,
  source: mount.source,
  majorMinor: mount.majorMinor,
  filesystemType: mount.filesystemType,
  uuid: mountUuid(mount, uuidByMountpoint),
});

/*
 * `expected` identifies the exact mount covering checkout. Every mount below
 * checkout must be named by an exact allowedNested rule. This deliberately
 * rejects a new or unrelated nested mount instead of treating statfs as if it
 * were path scoped.
 */
export const validateMountTopology = ({ mounts, checkout, expected, allowedNested = [], uuidByMountpoint }) => {
  const parsed = typeof mounts === "string" ? parseMountInfo(mounts) : mounts;
  if (!Array.isArray(parsed)) fail("mounts must be an array or mountinfo text");
  const absoluteCheckout = mountPath(checkout, "checkout");
  const covering = coveringMount(parsed, absoluteCheckout);
  const actualIdentity = mountIdentity(covering, uuidByMountpoint);
  for (const key of ["mountId", "mountpoint", "root", "mountOptions", "source", "majorMinor", "filesystemType", "uuid"]) {
    compareMountField(actualIdentity[key], expected?.[key], `covering mount ${key}`);
  }

  const descendants = nestedMounts(parsed, absoluteCheckout);
  const rules = new Map();
  for (const rule of allowedNested) {
    const rulePath = mountPath(rule.mountpoint, "allowed nested mountpoint");
    if (rules.has(rulePath)) fail(`duplicate allowed nested mount ${rulePath}`);
    rules.set(rulePath, { ...rule, mountpoint: rulePath });
  }
  const seen = new Set();
  for (const descendant of descendants) {
    const rule = rules.get(descendant.mountpoint);
    if (!rule) fail(`unexpected nested mount ${descendant.mountpoint}`);
    seen.add(descendant.mountpoint);
    const actual = mountIdentity(descendant, uuidByMountpoint);
    for (const key of ["mountpoint", "root", "mountOptions", "source", "majorMinor", "filesystemType", "uuid"]) {
      compareMountField(actual[key], rule[key] ?? expected?.[key], `nested mount ${descendant.mountpoint} ${key}`);
    }
  }
  for (const rulePath of rules.keys()) {
    if (!seen.has(rulePath)) fail(`allowed nested mount is absent: ${rulePath}`);
  }
  return Object.freeze({ coveringMount: covering, nestedMounts: Object.freeze(descendants) });
};

const optionSet = (mount) => new Set(mount.mountOptions.split(",").filter((option) => option.length > 0));

const requireMountOptions = (mount, required, label) => {
  const options = optionSet(mount);
  for (const option of required) {
    if (!options.has(option)) fail(`${label} is missing mount option ${option}`);
  }
};

const relativeMountRoot = (checkout, mountpoint) => {
  const root = mountPath(checkout, "checkout");
  const point = mountPath(mountpoint, "nested mountpoint");
  if (!coveredBy(point, root) || point === root) fail(`nested mountpoint is outside checkout: ${point}`);
  const relative = point.slice(root.length + 1);
  if (relative.length === 0 || relative.includes("/../") || relative === ".." || relative.startsWith("../")) {
    fail(`nested mountpoint has unsafe relative path: ${point}`);
  }
  return normalize(`/checkout/${relative}`);
};

/*
 * Revision 3's bwrap topology is intentionally stricter than the generic
 * helper above: the checkout bind is the exact canonical path, and exactly
 * target/ and output/ are the writable descendants.
 */
export const validateExactR2MountTopology = ({
  mounts,
  checkout,
  target,
  output,
  device,
  majorMinor,
  uuid,
  uuidByMountpoint,
}) => {
  const parsed = typeof mounts === "string" ? parseMountInfo(mounts) : mounts;
  if (!Array.isArray(parsed)) fail("mounts must be an array or mountinfo text");
  const absoluteCheckout = mountPath(checkout, "checkout");
  const absoluteTarget = mountPath(target, "target");
  const absoluteOutput = mountPath(output, "output");
  if (absoluteTarget === absoluteCheckout || absoluteOutput === absoluteCheckout || absoluteTarget === absoluteOutput) {
    fail("target and output must be distinct descendants of checkout");
  }
  if (!coveredBy(absoluteTarget, absoluteCheckout) || !coveredBy(absoluteOutput, absoluteCheckout)) {
    fail("target and output must be descendants of checkout");
  }
  const expectedDevice = safeToken(device, "source device");
  const expectedMajorMinor = safeToken(majorMinor, "major:minor");
  if (!/^\d+:\d+$/.test(expectedMajorMinor)) fail("major:minor is malformed");
  const expectedUuid = normalizeUuid(uuid);
  const covering = coveringMount(parsed, absoluteCheckout);
  if (covering.mountpoint !== absoluteCheckout) fail("checkout covering mount is not the exact canonical path");
  if (covering.root !== "/checkout") fail(`checkout mount root is not /checkout: ${covering.root}`);
  if (covering.source !== expectedDevice) fail(`checkout source device changed: expected ${expectedDevice}, got ${covering.source}`);
  if (covering.majorMinor !== expectedMajorMinor) fail("checkout major:minor changed");
  if (covering.filesystemType !== "xfs") fail("checkout covering mount is not xfs");
  requireMountOptions(covering, ["ro", "nodev", "nosuid"], "checkout covering mount");
  const coveringUuid = mountUuid(covering, uuidByMountpoint);
  if (coveringUuid !== null && coveringUuid !== expectedUuid) fail("checkout UUID changed");

  const descendants = nestedMounts(parsed, absoluteCheckout);
  if (descendants.length !== 2) fail(`expected exactly two nested mounts, got ${descendants.length}`);
  const expectedPoints = new Map([
    [absoluteTarget, relativeMountRoot(absoluteCheckout, absoluteTarget)],
    [absoluteOutput, relativeMountRoot(absoluteCheckout, absoluteOutput)],
  ]);
  const seen = new Set();
  for (const mount of descendants) {
    const expectedRoot = expectedPoints.get(mount.mountpoint);
    if (expectedRoot === undefined) fail(`unexpected nested mount ${mount.mountpoint}`);
    if (seen.has(mount.mountpoint)) fail(`duplicate nested mount ${mount.mountpoint}`);
    seen.add(mount.mountpoint);
    if (mount.root !== expectedRoot) fail(`nested mount root changed at ${mount.mountpoint}`);
    if (mount.source !== expectedDevice) fail(`nested source device changed at ${mount.mountpoint}`);
    if (mount.majorMinor !== expectedMajorMinor) fail(`nested major:minor changed at ${mount.mountpoint}`);
    if (mount.filesystemType !== "xfs") fail(`nested mount is not xfs at ${mount.mountpoint}`);
    requireMountOptions(mount, ["rw", "nodev", "nosuid"], `nested mount ${mount.mountpoint}`);
    const nestedUuid = mountUuid(mount, uuidByMountpoint);
    if (nestedUuid !== null && nestedUuid !== expectedUuid) fail(`nested UUID changed at ${mount.mountpoint}`);
  }
  for (const point of expectedPoints.keys()) {
    if (!seen.has(point)) fail(`required nested mount is absent: ${point}`);
  }
  return Object.freeze({ coveringMount: covering, nestedMounts: Object.freeze(descendants) });
};

export const ceilMebibytes = (bytes) => {
  const value = asBigInt(bytes, "bytes");
  return (value + MEBIBYTE - 1n) / MEBIBYTE;
};

const statfsValues = (snapshot) => ({
  filesystemType: asBigInt(field(snapshot, ["type", "f_type"], "f_type"), "f_type"),
  blockSize: asBigInt(field(snapshot, ["bsize", "f_bsize"], "f_bsize"), "f_bsize"),
  blocks: asBigInt(field(snapshot, ["blocks", "f_blocks"], "f_blocks"), "f_blocks"),
  freeBlocks: asBigInt(field(snapshot, ["bfree", "f_bfree"], "f_bfree"), "f_bfree"),
  availableBlocks: asBigInt(field(snapshot, ["bavail", "f_bavail"], "f_bavail"), "f_bavail"),
});

export const accountingFromStatfs = (snapshot, { fragmentSize } = {}) => {
  const values = statfsValues(snapshot);
  const fragment = asBigInt(fragmentSize, "fragment_size");
  if (values.filesystemType !== XFS_MAGIC) fail(`filesystem is not XFS: ${values.filesystemType}`);
  if (values.blockSize === 0n || fragment === 0n) fail("filesystem block size is zero");
  if (values.blockSize !== fragment) {
    fail(`fragment size ${fragment} differs from statfs block size ${values.blockSize}`);
  }
  if (values.freeBlocks > values.blocks || values.availableBlocks > values.freeBlocks) {
    fail("statfs free-block counters are out of range");
  }
  const usedBlocks = values.blocks - values.freeBlocks;
  const capacityBytes = values.blocks * fragment;
  const usedBytes = usedBlocks * fragment;
  return Object.freeze({
    filesystemType: values.filesystemType,
    blockSize: values.blockSize,
    fragmentSize: fragment,
    blocks: values.blocks,
    freeBlocks: values.freeBlocks,
    availableBlocks: values.availableBlocks,
    usedBlocks,
    capacityBytes,
    usedBytes,
    mebibytes: ceilMebibytes(usedBytes),
  });
};

const stableIdentityKeys = Object.freeze([
  "mountId",
  "mountpoint",
  "mountRoot",
  "mountOptions",
  "source",
  "majorMinor",
  "sourceDevice",
  "sourceMajorMinor",
  "uuid",
  "filesystemType",
  "fragmentSize",
  "blockSize",
  "capacityBlocks",
  "rootDevice",
  "rootInode",
  "targetPath",
  "outputPath",
  "nestedMounts",
]);

export const assertStableFilesystemIdentity = (baseline, current) => {
  for (const key of stableIdentityKeys) {
    const baselineValue = key === "nestedMounts" ? JSON.stringify(baseline?.[key]) : String(baseline?.[key]);
    const currentValue = key === "nestedMounts" ? JSON.stringify(current?.[key]) : String(current?.[key]);
    if (baselineValue !== currentValue) {
      fail(`filesystem identity changed: ${key}`);
    }
  }
  return true;
};

const identityMountExpectation = (identity) => ({
  mountId: identity.mountId,
  mountpoint: identity.mountpoint,
  root: identity.mountRoot,
  mountOptions: identity.mountOptions,
  source: identity.source,
  majorMinor: identity.majorMinor,
  filesystemType: identity.mountFilesystemType,
  uuid: identity.uuid,
});

const readMountText = (reader) => (typeof reader === "function" ? reader() : reader);

export const captureFilesystemIdentity = (root, {
  fragmentSize,
  uuid,
  statReader = statSync,
  statfsReader = statfsSync,
  realpathReader = realpathSync,
  mountInfoReader = () => readFileSync("/proc/self/mountinfo", "utf8"),
  allowedNested = [],
  uuidByMountpoint,
  target,
  output,
  device,
  majorMinor,
  expected,
  expectedCapacityBytes,
  capacityLimitBytes = MAX_CAPACITY_BYTES,
} = {}) => {
  const canonicalRoot = mountPath(realpathReader(root), "checkout");
  const rootStat = statReader(canonicalRoot, { bigint: true });
  if (typeof rootStat.isDirectory === "function" && !rootStat.isDirectory()) fail("checkout root is not a directory");
  const statfs = statfsReader(canonicalRoot, { bigint: true });
  const accounting = accountingFromStatfs(statfs, { fragmentSize });
  const capacityLimit = capacityLimitBytes === null ? null : asBigInt(capacityLimitBytes, "capacity limit");
  if (capacityLimit !== null && accounting.capacityBytes > capacityLimit) {
    fail(`filesystem capacity exceeds limit: ${accounting.capacityBytes}`);
  }
  const filesystemUuid = normalizeUuid(uuid);
  const mounts = parseMountInfo(readMountText(mountInfoReader));
  const covering = coveringMount(mounts, canonicalRoot);
  const mountRecords = mounts.map((mount) => Object.freeze({
    ...mount,
    uuid: mountUuid(mount, uuidByMountpoint),
  }));
  const rootMountIndex = mountRecords.findIndex((mount) => mount.mountId === covering.mountId);
  if (rootMountIndex < 0) fail("covering mount disappeared while reading mountinfo");
  mountRecords[rootMountIndex] = Object.freeze({ ...mountRecords[rootMountIndex], uuid: filesystemUuid });
  const mountExpectation = expected
    ? { ...expected, uuid: expected.uuid ?? filesystemUuid }
    : {
      mountId: covering.mountId,
      mountpoint: covering.mountpoint,
      source: covering.source,
      majorMinor: covering.majorMinor,
      filesystemType: covering.filesystemType,
      uuid: filesystemUuid,
    };
  const strictArguments = [target, output, device, majorMinor];
  const strictRequested = strictArguments.some((argument) => argument !== undefined);
  if (strictRequested && strictArguments.some((argument) => argument === undefined)) {
    fail("exact R2 mount topology requires target, output, device, and major:minor");
  }
  const topology = strictRequested
    ? validateExactR2MountTopology({
      mounts: mountRecords,
      checkout: canonicalRoot,
      target,
      output,
      device,
      majorMinor,
      uuid: filesystemUuid,
      uuidByMountpoint,
    })
    : validateMountTopology({
      mounts: mountRecords,
      checkout: canonicalRoot,
      expected: mountExpectation,
      allowedNested,
      uuidByMountpoint,
    });
  const nested = topology.nestedMounts.map((mount) => Object.freeze({
    mountId: mount.mountId,
    mountpoint: mount.mountpoint,
    root: mount.root,
    mountOptions: mount.mountOptions,
    source: mount.source,
    majorMinor: mount.majorMinor,
    filesystemType: mount.filesystemType,
    uuid: mountUuid(mount, uuidByMountpoint) ?? filesystemUuid,
  }));
  const identity = Object.freeze({
    schema: 1,
    mountId: covering.mountId,
    mountpoint: covering.mountpoint,
    mountRoot: covering.root,
    mountOptions: covering.mountOptions,
    source: covering.source,
    majorMinor: covering.majorMinor,
    mountFilesystemType: covering.filesystemType,
    uuid: filesystemUuid,
    filesystemType: accounting.filesystemType,
    filesystemMagic: XFS_MAGIC_HEX,
    rootDevice: asBigInt(field(rootStat, ["dev"], "root device"), "root device"),
    rootInode: asBigInt(field(rootStat, ["ino"], "root inode"), "root inode"),
    fragmentSize: accounting.fragmentSize,
    blockSize: accounting.blockSize,
    capacityBlocks: accounting.blocks,
    capacityBytes: accounting.capacityBytes,
    dedicatedFixedCapacity: true,
    targetPath: target === undefined ? undefined : mountPath(target, "target"),
    outputPath: output === undefined ? undefined : mountPath(output, "output"),
    sourceDevice: device === undefined ? covering.source : safeToken(device, "source device"),
    sourceMajorMinor: majorMinor === undefined ? covering.majorMinor : safeToken(majorMinor, "major:minor"),
    nestedMounts: Object.freeze(nested),
    accounting,
  });
  if (expectedCapacityBytes !== undefined && identity.capacityBytes !== asBigInt(expectedCapacityBytes, "expected capacity")) {
    fail("filesystem capacity changed from expected capacity");
  }
  return identity;
};

export const sampleFilesystem = (root, baseline, options = {}) => {
  const current = captureFilesystemIdentity(root, {
    ...options,
    fragmentSize: baseline.fragmentSize,
    uuid: baseline.uuid,
    target: baseline.targetPath,
    output: baseline.outputPath,
    device: baseline.sourceDevice,
    majorMinor: baseline.sourceMajorMinor,
    expected: identityMountExpectation(baseline),
    expectedCapacityBytes: baseline.capacityBytes,
    capacityLimitBytes: options.capacityLimitBytes ?? baseline.capacityBytes,
  });
  assertStableFilesystemIdentity(baseline, current);
  return current;
};

const identityDocument = (identity) => ({
  schema: 1,
  filesystem_type: canonicalUnsignedDecimal(identity.filesystemType, "filesystem_type"),
  filesystem_magic: XFS_MAGIC_HEX,
  counter: "f_blocks-f_bfree",
  mount_id: canonicalUnsignedDecimal(identity.mountId, "mount_id"),
  mountpoint: encodeMountInfoToken(identity.mountpoint, "mountpoint"),
  mount_root: encodeMountInfoToken(identity.mountRoot, "mount_root"),
  mount_options: safeToken(identity.mountOptions, "mount options"),
  source: encodeMountInfoToken(identity.source, "source"),
  major_minor: safeToken(identity.majorMinor, "major:minor"),
  uuid: normalizeUuid(identity.uuid),
  source_device: safeToken(identity.sourceDevice, "source device"),
  source_major_minor: safeToken(identity.sourceMajorMinor, "source major:minor"),
  root_device: canonicalUnsignedDecimal(identity.rootDevice, "root_device"),
  root_inode: canonicalUnsignedDecimal(identity.rootInode, "root_inode"),
  fragment_size: canonicalUnsignedDecimal(identity.fragmentSize, "fragment_size"),
  block_size: canonicalUnsignedDecimal(identity.blockSize, "block_size"),
  capacity_blocks: canonicalUnsignedDecimal(identity.capacityBlocks, "capacity_blocks"),
  capacity_bytes: canonicalUnsignedDecimal(identity.capacityBytes, "capacity_bytes"),
  target: identity.targetPath === undefined ? undefined : encodeMountInfoToken(identity.targetPath, "target"),
  output: identity.outputPath === undefined ? undefined : encodeMountInfoToken(identity.outputPath, "output"),
  nested_mounts: (identity.nestedMounts ?? []).map((mount) => ({
    mount_id: canonicalUnsignedDecimal(mount.mountId, "nested mount_id"),
    mountpoint: encodeMountInfoToken(mount.mountpoint, "nested mountpoint"),
    root: encodeMountInfoToken(mount.root, "nested mount root"),
    mount_options: safeToken(mount.mountOptions, "nested mount options"),
    source: encodeMountInfoToken(mount.source, "nested source"),
    major_minor: safeToken(mount.majorMinor, "nested major:minor"),
    filesystem_type: safeToken(mount.filesystemType, "nested filesystem type"),
    uuid: normalizeUuid(mount.uuid),
  })),
  dedicated_fixed_capacity: true,
});

export const filesystemIdentityDocument = (identity, { samplerOriginNs, nominalIntervalNs } = {}) => {
  const document = identityDocument(identity);
  if (samplerOriginNs !== undefined) document.sampler_origin_ns = canonicalUnsignedDecimal(samplerOriginNs, "sampler origin");
  if (nominalIntervalNs !== undefined) document.nominal_interval_ns = canonicalUnsignedDecimal(nominalIntervalNs, "nominal interval");
  return JSON.stringify(document, null, 2) + "\n";
};

const rowText = (value, label) => typeof value === "string" ? safeToken(value, label) : canonicalUnsignedDecimal(value, label);

export const rawRow = ({ ordinal, sampleStartNs, elapsedNs, deadlineNs, identity, accounting = identity.accounting, kind }) => {
  if (!KINDS.has(kind)) fail(`invalid sample kind: ${kind}`);
  const values = [
    rowText(ordinal, "ordinal"),
    rowText(sampleStartNs, "sample_start_ns"),
    rowText(elapsedNs, "elapsed_ns"),
    rowText(deadlineNs, "deadline_ns"),
    rowText(identity.mountId, "mount_id"),
    encodeMountInfoToken(identity.mountpoint, "mountpoint"),
    encodeMountInfoToken(identity.mountRoot, "mount_root"),
    safeToken(identity.mountOptions, "mount options"),
    encodeMountInfoToken(identity.source, "source"),
    safeToken(identity.majorMinor, "major:minor"),
    normalizeUuid(identity.uuid),
    rowText(accounting.filesystemType, "filesystem_type"),
    rowText(identity.rootDevice, "device"),
    rowText(identity.rootInode, "root_inode"),
    rowText(accounting.fragmentSize, "fragment_size"),
    rowText(accounting.blockSize, "block_size"),
    rowText(accounting.blocks, "blocks"),
    rowText(accounting.freeBlocks, "free_blocks"),
    rowText(accounting.availableBlocks, "available_blocks"),
    rowText(accounting.usedBlocks, "used_blocks"),
    rowText(accounting.capacityBytes, "capacity_bytes"),
    rowText(accounting.usedBytes, "used_bytes"),
    rowText(accounting.mebibytes, "mebibytes"),
    kind,
  ];
  if (values.length !== RAW_COLUMNS.length) fail("raw row has wrong field count");
  return values.join("\t");
};

const parseTextDecimal = (value, label) => BigInt(canonicalUnsignedDecimal(value, label));

export const parseRawRow = (line) => {
  if (typeof line !== "string" || line.includes("\n") || line.includes("\r")) fail("raw row is not one line");
  const values = line.split("\t");
  if (values.length !== RAW_COLUMNS.length) fail("raw row has wrong field count");
  const result = {
    ordinal: parseTextDecimal(values[0], "ordinal"),
    sampleStartNs: parseTextDecimal(values[1], "sample_start_ns"),
    elapsedNs: parseTextDecimal(values[2], "elapsed_ns"),
    deadlineNs: parseTextDecimal(values[3], "deadline_ns"),
    mountId: parseTextDecimal(values[4], "mount_id"),
    mountpoint: decodeMountInfoToken(values[5]),
    mountRoot: decodeMountInfoToken(values[6]),
    mountOptions: values[7],
    source: decodeMountInfoToken(values[8]),
    majorMinor: values[9],
    uuid: normalizeUuid(values[10]),
    filesystemType: parseTextDecimal(values[11], "filesystem_type"),
    rootDevice: parseTextDecimal(values[12], "device"),
    rootInode: parseTextDecimal(values[13], "root_inode"),
    fragmentSize: parseTextDecimal(values[14], "fragment_size"),
    blockSize: parseTextDecimal(values[15], "block_size"),
    blocks: parseTextDecimal(values[16], "blocks"),
    freeBlocks: parseTextDecimal(values[17], "free_blocks"),
    availableBlocks: parseTextDecimal(values[18], "available_blocks"),
    usedBlocks: parseTextDecimal(values[19], "used_blocks"),
    capacityBytes: parseTextDecimal(values[20], "capacity_bytes"),
    usedBytes: parseTextDecimal(values[21], "used_bytes"),
    mebibytes: parseTextDecimal(values[22], "mebibytes"),
    kind: values[23],
  };
  result.mountRoot = mountPath(result.mountRoot, "mount_root");
  result.mountOptions = safeToken(result.mountOptions, "mount options");
  if (!/^\d+:\d+$/.test(result.majorMinor)) fail("raw major:minor is malformed");
  if (!KINDS.has(result.kind)) fail(`invalid raw sample kind: ${result.kind}`);
  return Object.freeze(result);
};

export const validateRawRow = (row, baseline) => {
  const parsed = typeof row === "string" ? parseRawRow(row) : row;
  for (const [key, expected] of [
    ["mountId", baseline.mountId],
    ["mountpoint", baseline.mountpoint],
    ["mountRoot", baseline.mountRoot],
    ["mountOptions", baseline.mountOptions],
    ["source", baseline.source],
    ["majorMinor", baseline.majorMinor],
    ["uuid", baseline.uuid],
    ["filesystemType", baseline.filesystemType],
    ["rootDevice", baseline.rootDevice],
    ["rootInode", baseline.rootInode],
    ["fragmentSize", baseline.fragmentSize],
    ["blockSize", baseline.blockSize],
    ["blocks", baseline.capacityBlocks],
  ]) {
    if (String(parsed[key]) !== String(expected)) fail(`raw identity mismatch: ${key}`);
  }
  const accounting = accountingFromStatfs({
    type: parsed.filesystemType,
    bsize: parsed.blockSize,
    blocks: parsed.blocks,
    bfree: parsed.freeBlocks,
    bavail: parsed.availableBlocks,
  }, { fragmentSize: parsed.fragmentSize });
  for (const [key, expected] of [
    ["filesystemType", accounting.filesystemType],
    ["fragmentSize", accounting.fragmentSize],
    ["blockSize", accounting.blockSize],
    ["blocks", accounting.blocks],
    ["freeBlocks", accounting.freeBlocks],
    ["availableBlocks", accounting.availableBlocks],
    ["usedBlocks", accounting.usedBlocks],
    ["capacityBytes", accounting.capacityBytes],
    ["usedBytes", accounting.usedBytes],
    ["mebibytes", accounting.mebibytes],
  ]) {
    if (parsed[key] !== expected) fail(`raw counter mismatch: ${key}`);
  }
  return parsed;
};

export const validateRetainedGaps = (rows, { maxGapNs = MAX_SAMPLE_GAP_NS } = {}) => {
  const limit = asBigInt(maxGapNs, "maximum sample gap");
  let previous = null;
  for (const row of rows) {
    const start = typeof row === "bigint" ? row : asBigInt(row.sampleStartNs, "sample_start_ns");
    if (previous !== null) {
      if (start <= previous) fail("sample-start timestamps are not strictly increasing");
      const gap = start - previous;
      if (gap > limit) throw new SampleGapError(gap, limit);
    }
    previous = start;
  }
  return true;
};

export const validateRawLedger = (text, baseline, {
  originNs,
  periodNs = SAMPLE_PERIOD_NS,
  maxGapNs = MAX_SAMPLE_GAP_NS,
} = {}) => {
  if (
    typeof text !== "string"
    || !text.startsWith(`${RAW_HEADER}\n`)
    || !text.endsWith("\n")
    || text.endsWith("\n\n")
    || text.includes("\r")
  ) {
    fail("raw ledger is not one canonical newline-terminated stream");
  }
  const origin = asBigInt(originNs, "sampler origin");
  const period = asBigInt(periodNs, "nominal interval");
  if (period === 0n) fail("nominal period is zero");
  const lines = text.slice(0, -1).split("\n").slice(1);
  if (lines.some((line) => line.length === 0)) fail("raw ledger contains a blank row");
  if (lines.some((line) => line !== line.trimEnd())) fail("raw ledger row has trailing whitespace");
  if (lines.length < 2) fail("raw ledger needs at least two rows");
  const rows = lines.map((line) => validateRawRow(line, baseline));
  for (let index = 0; index < rows.length; index += 1) {
    if (rows[index].ordinal !== BigInt(index)) fail("raw ordinals are not contiguous");
    if (rows[index].elapsedNs !== rows[index].sampleStartNs - origin) {
      fail("raw elapsed timestamp is inconsistent");
    }
    if (rows[index].kind === "scheduled") {
      const expectedDeadline = scheduleDeadline(origin, rows[index].ordinal, period);
      if (rows[index].deadlineNs !== expectedDeadline) fail("scheduled deadline is inconsistent");
      if (rows[index].sampleStartNs < expectedDeadline) fail("scheduled sample started before deadline");
    } else if (rows[index].deadlineNs !== rows[index].sampleStartNs) {
      fail("terminal deadline is inconsistent");
    }
  }
  const terminal = rows.filter((row) => row.kind === "terminal");
  if (terminal.length !== 1 || terminal[0] !== rows[rows.length - 1]) fail("raw ledger terminal row is not unique and last");
  if (rows.slice(0, -1).some((row) => row.kind !== "scheduled")) fail("non-terminal row is not scheduled");
  validateRetainedGaps(rows, { maxGapNs });
  return Object.freeze(rows);
};

export const publicTsvFromRaw = (rows) => {
  const parsed = rows.map((row) => typeof row === "string" ? parseRawRow(row) : row);
  return `${PUBLIC_HEADER}\n${parsed.map((row) => [
    canonicalUnsignedDecimal(row.ordinal, "ordinal"),
    canonicalUnsignedDecimal(row.sampleStartNs, "sample_start_ns"),
    canonicalUnsignedDecimal(row.elapsedNs, "elapsed_ns"),
    canonicalUnsignedDecimal(row.mebibytes, "mebibytes"),
    row.kind,
  ].join("\t")).join("\n")}\n`;
};

export const scheduleDeadline = (originNs, ordinal, periodNs = SAMPLE_PERIOD_NS) => (
  (() => {
    const origin = asBigInt(originNs, "schedule origin");
    const index = asBigInt(ordinal, "ordinal");
    const period = asBigInt(periodNs, "nominal period");
    if (period === 0n) fail("nominal period is zero");
    return origin + index * period;
  })()
);

export const validateAbsoluteSchedule = (samples, {
  originNs,
  periodNs = SAMPLE_PERIOD_NS,
  maxGapNs = MAX_SAMPLE_GAP_NS,
} = {}) => {
  if (!Array.isArray(samples) || samples.length === 0) fail("schedule has no samples");
  const origin = asBigInt(originNs, "schedule origin");
  const period = asBigInt(periodNs, "nominal period");
  if (period === 0n) fail("nominal period is zero");
  const rows = samples.map((sample, index) => {
    const start = asBigInt(sample.sampleStartNs, "sample_start_ns");
    const deadline = asBigInt(sample.deadlineNs, "deadline_ns");
    if (deadline !== scheduleDeadline(origin, BigInt(index), period)) fail("absolute schedule deadline shifted");
    if (start < deadline) fail("sample started before its absolute deadline");
    return { start, deadline };
  });
  validateRetainedGaps(rows.map(({ start }) => start), { maxGapNs });
  return true;
};

const defaultSleepUntil = async (deadlineNs, clock = process.hrtime.bigint) => {
  while (true) {
    const remaining = deadlineNs - clock();
    if (remaining <= 0n) return;
    const milliseconds = Number((remaining + 999_999n) / 1_000_000n);
    await new Promise((resolvePromise) => setTimeout(resolvePromise, Math.min(milliseconds, 2_147_483_647)));
  }
};

export const sleepUntil = defaultSleepUntil;

/* A small injectable scheduler makes the absolute-deadline invariant testable without waiting. */
export const runAbsoluteSchedule = async ({
  clock = process.hrtime.bigint,
  sleep = defaultSleepUntil,
  snapshot,
  onSample = async () => {},
  originNs,
  periodNs = SAMPLE_PERIOD_NS,
  maxGapNs = MAX_SAMPLE_GAP_NS,
  maxSamples = 1,
} = {}) => {
  if (typeof snapshot !== "function") fail("scheduler snapshot callback is required");
  const count = Number(maxSamples);
  if (!Number.isSafeInteger(count) || count < 1) fail("scheduler maxSamples is invalid");
  const origin = originNs === undefined ? clock() : asBigInt(originNs, "schedule origin");
  const period = asBigInt(periodNs, "nominal period");
  const rows = [];
  let previous = null;
  for (let ordinal = 0; ordinal < count; ordinal += 1) {
    const deadlineNs = scheduleDeadline(origin, BigInt(ordinal), period);
    await sleep(deadlineNs, clock);
    const sampleStartNs = clock();
    if (previous !== null) {
      if (sampleStartNs <= previous) fail("sample clock regressed");
      const gap = sampleStartNs - previous;
      if (gap > asBigInt(maxGapNs, "maximum sample gap")) throw new SampleGapError(gap, asBigInt(maxGapNs, "maximum sample gap"));
    }
    const row = Object.freeze({
      ordinal: BigInt(ordinal),
      deadlineNs,
      sampleStartNs,
      elapsedNs: sampleStartNs - origin,
      snapshot: await snapshot({ ordinal: BigInt(ordinal), deadlineNs, sampleStartNs }),
    });
    await onSample(row);
    rows.push(row);
    previous = sampleStartNs;
  }
  validateAbsoluteSchedule(rows, { originNs: origin, periodNs: period, maxGapNs });
  return Object.freeze({ originNs: origin, rows: Object.freeze(rows) });
};
