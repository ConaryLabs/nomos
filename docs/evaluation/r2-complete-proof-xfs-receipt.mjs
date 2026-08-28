#!/usr/bin/env node
/*
 * Pure and low-level support for r2-complete-proof-xfs.sh.
 *
 * This file deliberately does not provision a filesystem or invoke a proof.
 * The shell wrapper owns privilege and lifecycle.  Keeping path, inventory,
 * counter, and receipt decisions here makes them directly testable without a
 * privileged 8 GiB run.
 */

import { createHash } from "node:crypto";
import { closeSync, existsSync, fchmodSync, fstatSync, lstatSync, mkdirSync, statSync,
  openSync, readdirSync, readSync, realpathSync, writeFileSync,
  writeSync } from "node:fs";
import { basename, dirname, isAbsolute, join, relative, resolve, sep } from "node:path";
import { fileURLToPath } from "node:url";

import {
  XFS_MAGIC,
  accountingFromStatfs,
  ceilMebibytes,
} from "./r2-filesystem-accounting.mjs";
import {
  validateFilefragEvidence,
  validateEvidenceManifest,
  validateHostFilesystemEvidence,
  validateOperationsAndTools,
  validateXfsInfoEvidence,
} from "./r2-complete-proof-xfs-evidence.mjs";
import {
  validateBoundHostMonitor as validateBoundHostMonitorCore,
  validateCommandLedger,
  validateHostMonitor as validateHostMonitorCore,
  validateOuterPreflight,
} from "./r2-complete-proof-xfs-ledger.mjs";

export const SCHEMA = "nomos-r2-xfs-proof/1";
export const IMAGE_BYTES = 8_589_934_592n;
export const FINALIZATION_RESERVATION_BYTES = 16_777_216n;
export const MEBIBYTE = 1_048_576n;
export const TOP_LEVEL_FIELDS = Object.freeze([
  "receipt",
  "outcome",
  "candidate",
  "image",
  "loop_device",
  "filesystem",
  "mount",
  "invocation",
  "export",
  "teardown",
]);

/* The input facts have a closed pass-path shape.  Red/setup-fail facts are
 * intentionally allowed to be partial so that they can preserve diagnostics. */
const FACT_FIELDS = Object.freeze([
  "setup_failed", "inner_pass", "candidate", "image", "loop_device", "filesystem", "mount",
  "invocation", "export", "teardown", "host_monitor", "operations", "tool_register",
]);
const SECTION_FIELDS = Object.freeze({
  candidate: ["source", "commit", "tree", "clean", "source_status"],
  image: ["path", "stat_path", "filefrag_path", "fallocate_stdout", "fallocate_stderr", "sync_stdout", "sync_stderr", "status", "sync_status", "logical_bytes", "allocated_bytes", "expected_bytes"],
  loop_device: ["path", "major_minor", "size_bytes", "attached"],
  filesystem: ["type", "uuid", "fragment_size", "capacity_limit_bytes", "capacity_ok", "mounted_statfs_path", "checkout_statfs_path", "close_statfs_path", "host_filesystem_before_path", "host_filesystem_after_path", "xfs_info_path"],
  mount: ["path", "source", "options", "propagation", "status", "mounted", "unmounted", "mount_absent"],
  invocation: ["argv", "cwd", "uid", "gid", "user", "status", "inner_pass", "start_ns", "end_ns", "stdout_path", "stderr_path", "command_ledger_path", "execution_ledger_path", "outer_preflight_path"],
  export: ["source", "destination", "status", "equal", "source_inventory_sha256", "export_inventory_sha256", "inventory_path", "inventory_digest_path", "inner_evidence_manifest_path"],
  teardown: ["unmounted", "loop_detached", "no_holder", "mount_absent", "image_unattached", "fuser_status", "umount_status", "detach_status", "supervisor_status", "host_monitor"],
});

const UUID = /^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$/i;
const DECIMAL = /^(0|[1-9][0-9]*)$/;
const SAFE_TEXT = /^[^\0\r\n\t]+$/;
const PROCESS_FD_PATH = /^\/proc\/self\/fd\/([1-9][0-9]*)(?:\/.*)?$/;

export class XfsWrapperError extends Error {
  constructor(message) {
    super(message);
    this.name = "XfsWrapperError";
  }
}

const fail = (message) => { throw new XfsWrapperError(message); };
const requireValue = (condition, message) => { if (!condition) fail(message); };

const decimal = (value, label) => {
  if (typeof value === "bigint") {
    if (value < 0n) fail(`${label} must be unsigned`);
    return value;
  }
  if (typeof value !== "string" || !DECIMAL.test(value)) fail(`${label} is not canonical unsigned decimal`);
  return BigInt(value);
};

const bool = (value, label) => {
  if (typeof value !== "boolean") fail(`${label} must be boolean`);
  return value;
};

export const safeText = (value, label) => {
  if (typeof value !== "string" || !SAFE_TEXT.test(value)) fail(`${label} is not safe text`);
  return value;
};

const isSymlink = (path) => {
  try { return lstatSync(path).isSymbolicLink(); }
  catch (error) {
    if (error.code === "ENOENT") return false;
    throw error;
  }
};

export const readableRealpath = (path, label) => {
  try { return realpathSync.native(path); }
  catch (error) { fail(`${label} is not an existing path: ${error.message}`); }
};

/*
 * A canonical path is intentionally stricter than path.resolve(): callers
 * must provide the exact absolute spelling returned by realpath(3).  Thus a
 * symlink in any existing component, a trailing slash, '.', and '..' are all
 * rejected before a privileged process is started.
 */
export const canonicalPath = (value, label = "path", { mustExist = true } = {}) => {
  if (typeof value !== "string" || value.length === 0 || !isAbsolute(value) || !SAFE_TEXT.test(value)) {
    fail(`${label} must be one absolute safe path`);
  }
  const lexical = resolve(value);
  if (lexical !== value) fail(`${label} is not canonical: ${value}`);
  if (!mustExist) {
    const parent = dirname(value);
    const canonicalParent = readableRealpath(parent, `${label} parent`);
    if (canonicalParent !== parent) fail(`${label} parent is not canonical`);
    if (isSymlink(value)) fail(`${label} is a symlink`);
    return value;
  }
  const actual = readableRealpath(value, label);
  if (actual !== value) fail(`${label} traverses a symlink or is not canonical`);
  return actual;
};

/* Check every existing component, independently of the lexical check above. */
export const assertNoSymlinkComponents = (path, stop = "/") => {
  const absolute = resolve(path);
  const boundary = resolve(stop);
  requireValue(boundary === "/" ? absolute.startsWith("/") : absolute === boundary || absolute.startsWith(`${boundary}${sep}`), `${path} is outside ${stop}`);
  let cursor = absolute;
  while (true) {
    let info;
    try { info = lstatSync(cursor); }
    catch (error) {
      if (error.code === "ENOENT") fail(`path component is absent: ${cursor}`);
      throw error;
    }
    if (info.isSymbolicLink()) fail(`symlinked path component: ${cursor}`);
    if (cursor === boundary) break;
    cursor = dirname(cursor);
  }
  return true;
};

/* Inventory/export may run beneath the wrapper's retained work descriptor.
 * The procfs descriptor link itself is trusted as the opened root; every
 * component below it remains no-dereference, and published paths use the
 * resolved canonical spelling. */
const inventoryRootBinding = (value, label) => {
  if (typeof value !== "string" || value.length === 0 || !isAbsolute(value) ||
      !SAFE_TEXT.test(value) || resolve(value) !== value) fail(`${label} must be one absolute safe path`);
  const match = PROCESS_FD_PATH.exec(value);
  if (!match) {
    const canonical = canonicalPath(value, label);
    return Object.freeze({ access: canonical, display: canonical });
  }
  const descriptorRoot = `/proc/self/fd/${match[1]}`;
  let descriptorInfo;
  try { descriptorInfo = statSync(descriptorRoot); }
  catch (error) { fail(`${label} descriptor root is unavailable: ${error.message}`); }
  if (!descriptorInfo.isDirectory()) fail(`${label} descriptor root is not a directory`);
  let cursor = value;
  while (cursor !== descriptorRoot) {
    let info;
    try { info = lstatSync(cursor); }
    catch (error) { fail(`${label} component is unavailable: ${error.message}`); }
    if (info.isSymbolicLink()) fail(`${label} contains a symlink below its descriptor root`);
    const parent = dirname(cursor);
    if (parent !== descriptorRoot && !parent.startsWith(`${descriptorRoot}/`)) {
      fail(`${label} escapes its descriptor root`);
    }
    cursor = parent;
  }
  return Object.freeze({ access: value, display: readableRealpath(value, label) });
};

export const assertNonOverlappingPaths = (source, work) => {
  const left = canonicalPath(source, "source");
  const right = canonicalPath(work, "work");
  if (left === right || left.startsWith(`${right}${sep}`) || right.startsWith(`${left}${sep}`)) {
    fail("source and work paths overlap");
  }
  return Object.freeze({ source: left, work: right });
};

export const assertEmptyDirectory = (path, label = "work") => {
  const canonical = canonicalPath(path, label);
  const info = lstatSync(canonical);
  if (!info.isDirectory() || info.isSymbolicLink()) fail(`${label} is not a real directory`);
  if (readdirSync(canonical).length !== 0) fail(`${label} is not empty`);
  assertNoSymlinkComponents(canonical, "/");
  return canonical;
};

const relativePath = (root, path) => {
  const value = relative(root, path).split(sep).join("/");
  if (value.length === 0 || value.startsWith("/") || value.includes("\\") || value.includes("\0") ||
      value.split("/").some((part) => part.length === 0 || part === "." || part === "..")) {
    fail(`unsafe inventory path: ${value}`);
  }
  return value;
};

const modeBits = (info) => info.mode & 0o7777;

const hashRegularFile = (path, before) => {
  const flags = 0 | 0x20000 /* O_NOFOLLOW on Linux */;
  let fd;
  try { fd = openSync(path, flags | 0 /* O_RDONLY */); }
  catch (error) { fail(`cannot safely open ${path}: ${error.message}`); }
  const hash = createHash("sha256");
  let bytes = 0n;
  const buffer = Buffer.allocUnsafe(1024 * 1024);
  try {
    const opened = fstatSync(fd, { bigint: true });
    if (!opened.isFile() || opened.dev !== BigInt(before.dev) || opened.ino !== BigInt(before.ino) || opened.size !== BigInt(before.size)) {
      fail(`inventory file changed before hashing: ${path}`);
    }
    while (true) {
      const count = readSync(fd, buffer, 0, buffer.length, null);
      if (count === 0) break;
      hash.update(buffer.subarray(0, count));
      bytes += BigInt(count);
    }
    const after = fstatSync(fd, { bigint: true });
    if (!after.isFile() || after.dev !== BigInt(before.dev) || after.ino !== BigInt(before.ino) || after.size !== BigInt(before.size) || bytes !== BigInt(before.size)) {
      fail(`inventory file changed while hashing: ${path}`);
    }
  } finally {
    closeSync(fd);
  }
  return Object.freeze({ bytes, sha256: hash.digest("hex") });
};

const walkInventory = (root, directory, rows) => {
  const entries = readdirSync(directory, { withFileTypes: true }).sort((left, right) => Buffer.from(left.name).compare(Buffer.from(right.name)));
  for (const entry of entries) {
    const path = join(directory, entry.name);
    const info = lstatSync(path);
    const rel = relativePath(root, path);
    if (info.isSymbolicLink()) fail(`inventory contains symlink: ${path}`);
    if (info.isDirectory()) {
      rows.push({ path: rel, type: "directory", mode: modeBits(info) });
      walkInventory(root, path, rows);
      continue;
    }
    if (!info.isFile()) fail(`inventory contains non-regular entry: ${path}`);
    const digest = hashRegularFile(path, info);
    rows.push({ path: rel, type: "file", mode: modeBits(info), bytes: digest.bytes.toString(), sha256: digest.sha256 });
  }
};

export const canonicalInventoryText = (rows) => {
  if (!Array.isArray(rows)) fail("inventory rows must be an array");
  return `${rows.map((row) => JSON.stringify(row)).join("\n")}${rows.length === 0 ? "" : "\n"}`;
};

export const inventoryDigest = (rows) => createHash("sha256").update(canonicalInventoryText(rows), "utf8").digest("hex");

export const canonicalInventory = (root, label = "inventory root") => {
  const binding = inventoryRootBinding(root, label);
  const info = lstatSync(binding.access);
  if (!info.isDirectory() || info.isSymbolicLink()) fail(`${label} is not a real directory`);
  const rows = [];
  walkInventory(binding.access, binding.access, rows);
  rows.sort((left, right) => Buffer.from(left.path).compare(Buffer.from(right.path)));
  return Object.freeze({
    root: binding.display,
    rows: Object.freeze(rows),
    text: canonicalInventoryText(rows),
    sha256: inventoryDigest(rows),
  });
};

export const compareInventories = (left, right) => {
  const leftRows = Array.isArray(left) ? left : left?.rows;
  const rightRows = Array.isArray(right) ? right : right?.rows;
  if (!Array.isArray(leftRows) || !Array.isArray(rightRows)) fail("inventory comparison requires two row sets");
  const leftText = canonicalInventoryText(leftRows);
  const rightText = canonicalInventoryText(rightRows);
  if (leftText !== rightText) fail("canonical inventories differ");
  return Object.freeze({ equal: true, sha256: inventoryDigest(leftRows), rows: leftRows.length });
};

const copyFileNoDeref = (source, destination, sourceInfo) => {
  const inputFlags = 0 | 0x20000;
  let input;
  let output;
  try {
    input = openSync(source, inputFlags | 0 /* O_RDONLY */);
    output = openSync(destination, 0x1 | 0x40 | 0x20000, 0o600); /* WRONLY|CREAT|NOFOLLOW */
    const opened = fstatSync(input, { bigint: true });
    if (!opened.isFile() || opened.dev !== BigInt(sourceInfo.dev) || opened.ino !== BigInt(sourceInfo.ino) || opened.size !== BigInt(sourceInfo.size)) {
      fail(`source changed before export: ${source}`);
    }
    const buffer = Buffer.allocUnsafe(1024 * 1024);
    let copied = 0n;
    while (true) {
      const count = readSync(input, buffer, 0, buffer.length, null);
      if (count === 0) break;
      let offset = 0;
      while (offset < count) offset += writeSync(output, buffer, offset, count - offset);
      copied += BigInt(count);
    }
    const after = fstatSync(input, { bigint: true });
    if (copied !== BigInt(sourceInfo.size) || after.dev !== BigInt(sourceInfo.dev) || after.ino !== BigInt(sourceInfo.ino) || after.size !== BigInt(sourceInfo.size)) {
      fail(`source changed while exporting: ${source}`);
    }
    fchmodSync(output, sourceInfo.mode & 0o7777);
  } finally {
    if (input !== undefined) closeSync(input);
    if (output !== undefined) closeSync(output);
  }
};

const copyTreeRecursive = (source, destination) => {
  const sourceInfo = lstatSync(source);
  if (sourceInfo.isSymbolicLink()) fail(`export source contains symlink: ${source}`);
  if (sourceInfo.isDirectory()) {
    mkdirSync(destination, sourceInfo.mode & 0o7777);
    for (const entry of readdirSync(source, { withFileTypes: true }).sort((left, right) => Buffer.from(left.name).compare(Buffer.from(right.name)))) {
      copyTreeRecursive(join(source, entry.name), join(destination, entry.name));
    }
    return;
  }
  if (!sourceInfo.isFile()) fail(`export source contains non-regular entry: ${source}`);
  copyFileNoDeref(source, destination, sourceInfo);
};

export const copyTreeNoDeref = (source, destination) => {
  const sourceRoot = inventoryRootBinding(source, "export source");
  const destinationParent = dirname(destination);
  const destinationName = basename(destination);
  if (join(destinationParent, destinationName) !== destination) fail("export destination is not canonical");
  const destinationRoot = inventoryRootBinding(destinationParent, "export destination parent");
  const destinationDisplay = join(destinationRoot.display, destinationName);
  if (existsSync(destination) || isSymlink(destination)) fail("export destination already exists");
  const sourceInfo = lstatSync(sourceRoot.access);
  if (!sourceInfo.isDirectory() || sourceInfo.isSymbolicLink()) fail("export source is not a real directory");
  copyTreeRecursive(sourceRoot.access, destination);
  const inventory = canonicalInventory(destination, "export destination");
  if (inventory.root !== destinationDisplay) fail("export destination identity changed during copy");
  return inventory;
};

export const parseDuOutput = (stdout, expectedPath, stderr = "", status = 0) => {
  const canonicalExpected = canonicalPath(expectedPath, "du checkout");
  requireValue(status === 0, `du exited ${status}`);
  requireValue(stderr === "", "du wrote stderr");
  requireValue(typeof stdout === "string" && /^(0|[1-9][0-9]*)\t[^\t\r\n]+\n$/.test(stdout), "du output is not one canonical row");
  const separator = stdout.indexOf("\t");
  const mibText = stdout.slice(0, separator);
  const pathText = stdout.slice(separator + 1, -1);
  requireValue(pathText === canonicalExpected, "du path is not the canonical checkout");
  const mib = decimal(mibText, "du_mib");
  return Object.freeze({
    command: ["/usr/bin/ionice", "-c", "3", "/usr/bin/du", "-sm", "--", canonicalExpected],
    path: canonicalExpected,
    mib,
    stdout,
    stderr,
    status,
  });
};

export const validateDuAgainstStatfs = (du, accounting) => {
  const mib = decimal(du?.mib, "du_mib");
  const maximum = ceilMebibytes(accounting?.usedBytes ?? accounting?.used_bytes ?? 0n);
  if (mib > maximum) fail(`du exceeds its following statfs allocation: ${mib} > ${maximum}`);
  return true;
};

export const validateStatfsSnapshot = (snapshot, { fragmentSize, capacityLimitBytes = IMAGE_BYTES } = {}) => {
  const accounting = accountingFromStatfs(snapshot, { fragmentSize });
  const limit = capacityLimitBytes === null ? null : decimal(capacityLimitBytes, "capacity limit");
  if (limit !== null && accounting.capacityBytes > limit) fail(`statfs capacity exceeds limit: ${accounting.capacityBytes}`);
  return Object.freeze({
    f_type: accounting.filesystemType.toString(),
    f_bsize: accounting.blockSize.toString(),
    f_frsize: accounting.fragmentSize.toString(),
    f_blocks: accounting.blocks.toString(),
    f_bfree: accounting.freeBlocks.toString(),
    f_bavail: accounting.availableBlocks.toString(),
    capacity_bytes: accounting.capacityBytes.toString(),
    allocated_bytes: accounting.usedBytes.toString(),
    allocated_mib: accounting.mebibytes.toString(),
  });
};

export const validateReservationDelta = ({ before, after, allocated }) => {
  const aBefore = decimal(before, "A_before");
  const aAfter = decimal(after, "A_after");
  const reservation = decimal(allocated, "R_allocated");
  if (aBefore < reservation) fail("A_before is smaller than the reservation allocation");
  if (aAfter > aBefore - reservation) fail("reservation release did not reduce allocation by R_allocated");
  return true;
};

export const validateHostMonitor = (options = {}) => {
  try { return validateHostMonitorCore(options); }
  catch (error) { fail(error instanceof Error ? error.message : String(error)); }
};

const validateBoundHostMonitor = (monitor, options = {}) => {
  try { return validateBoundHostMonitorCore(monitor, options); }
  catch (error) { fail(error instanceof Error ? error.message : String(error)); }
};

export const bindHostMonitor = (facts, monitor) => {
  const value = objectValue(facts, "facts");
  const teardown = objectValue(requiredField(value, "teardown"), "teardown");
  const checkedMonitor = objectValue(monitor, "host monitor");
  return { ...value, host_monitor: checkedMonitor, teardown: { ...teardown, host_monitor: checkedMonitor } };
};

export const readRegularEvidence = (path, label) => {
  const canonical = canonicalPath(path, label);
  const before = lstatSync(canonical, { bigint: true });
  if (!before.isFile() || before.isSymbolicLink()) fail(`${label} is not one regular file`);
  const flags = 0 | 0x20000 /* O_NOFOLLOW on Linux */;
  let descriptor;
  try { descriptor = openSync(canonical, flags); }
  catch (error) { fail(`cannot safely open ${label}: ${error.message}`); }
  const hash = createHash("sha256");
  const chunks = [];
  const buffer = Buffer.allocUnsafe(1024 * 1024);
  let bytes = 0n;
  try {
    const opened = fstatSync(descriptor, { bigint: true });
    if (!opened.isFile() || opened.dev !== before.dev || opened.ino !== before.ino || opened.size !== before.size) {
      fail(`${label} changed while opening`);
    }
    for (;;) {
      const count = readSync(descriptor, buffer, 0, buffer.length, null);
      if (count === 0) break;
      const chunk = Buffer.from(buffer.subarray(0, count));
      chunks.push(chunk);
      hash.update(chunk);
      bytes += BigInt(count);
    }
    const after = fstatSync(descriptor, { bigint: true });
    if (!after.isFile() || after.dev !== opened.dev || after.ino !== opened.ino ||
        after.size !== opened.size || bytes !== opened.size) fail(`${label} changed while reading`);
  } finally {
    closeSync(descriptor);
  }
  return Object.freeze({ path: canonical, bytes: bytes.toString(), sha256: hash.digest("hex"), content: Buffer.concat(chunks) });
};

export const digestRegular = (path, label) => {
  const { content: _content, ...digest } = readRegularEvidence(path, label);
  return Object.freeze(digest);
};

const readToolRegister = (path) => {
  const evidence = readRegularEvidence(path, "wrapper tool register");
  const { content, ...digest } = evidence;
  const text = content.toString("utf8");
  const lines = text.split("\n");
  requireValue(lines[0] === "name\tpath\tversion_argv\tversion_status\tsha256\tversion", "wrapper tool register header differs");
  requireValue(text.endsWith("\n"), "wrapper tool register is not newline terminated");
  const rows = lines.slice(1, -1).map((line, index) => {
    const fields = line.split("\t");
    requireValue(fields.length === 6, `wrapper tool register row ${index + 1} has the wrong field count`);
    const [name, toolPath, versionArgv, status, sha256, version] = fields;
    safeText(name, `wrapper tool register row ${index + 1} name`);
    canonicalPath(toolPath, `wrapper tool register row ${index + 1} path`);
    safeText(versionArgv, `wrapper tool register row ${index + 1} version argv`);
    decimal(status, `wrapper tool register row ${index + 1} status`);
    requireValue(/^[0-9a-f]{64}$/.test(sha256), `wrapper tool register row ${index + 1} digest is malformed`);
    safeText(version, `wrapper tool register row ${index + 1} version`);
    return { name, path: toolPath, version_argv: versionArgv, version_status: Number(status), sha256, version };
  });
  requireValue(rows.length > 0, "wrapper tool register has no tools");
  return Object.freeze({ ...digest, rows: Object.freeze(rows) });
};

const exactTopLevel = (value) => {
  requireValue(value && typeof value === "object" && !Array.isArray(value), "receipt is not an object");
  const actual = Object.keys(value).sort();
  const expected = [...TOP_LEVEL_FIELDS].sort();
  requireValue(JSON.stringify(actual) === JSON.stringify(expected), `receipt top-level fields differ: ${actual.join(",")}`);
};

const section = (value) => value && typeof value === "object" && !Array.isArray(value) ? value : {};

const canonicalCandidate = (value) => {
  const candidate = section(value);
  return {
    source: candidate.source ?? null,
    commit: candidate.commit ?? null,
    tree: candidate.tree ?? null,
    clean: candidate.clean === true,
    source_status: candidate.source_status ?? null,
  };
};

const innerStatus = (facts) => {
  const invocation = section(facts.invocation);
  if (facts.inner_pass === true || invocation.inner_pass === true) return true;
  return false;
};

const teardownPass = (facts) => {
  const teardown = section(facts.teardown);
  const monitor = section(facts.host_monitor ?? teardown.host_monitor);
  return teardown.unmounted === true && teardown.loop_detached === true && teardown.no_holder === true &&
    teardown.mount_absent === true && teardown.image_unattached === true && monitor.clean === true;
};

export const objectValue = (value, label) => {
  if (!value || typeof value !== "object" || Array.isArray(value)) fail(`${label} must be an object`);
  return value;
};

export const requiredField = (object, name, label = name) => {
  if (!Object.prototype.hasOwnProperty.call(object, name)) fail(`${label} is missing`);
  return object[name];
};

export const statusCode = (value, label) => {
  if (typeof value !== "number" || !Number.isSafeInteger(value) || value < 0) fail(`${label} is not a status code`);
  return value;
};

export const canonicalHex = (value, length, label) => {
  if (typeof value !== "string" || !new RegExp(`^[0-9a-f]{${length}}$`).test(value)) fail(`${label} is not lowercase hexadecimal`);
  return value;
};

export const canonicalRegular = (value, label) => {
  const canonical = canonicalPath(value, label);
  const info = lstatSync(canonical);
  if (!info.isFile() || info.isSymbolicLink()) fail(`${label} is not one regular file`);
  return canonical;
};

const canonicalDirectory = (value, label) => {
  const canonical = canonicalPath(value, label);
  const info = lstatSync(canonical);
  if (!info.isDirectory() || info.isSymbolicLink()) fail(`${label} is not one real directory`);
  return canonical;
};

/* A path inside the XFS checkout is intentionally unavailable after ordinary
 * unmount.  Validate its exact canonical spelling and relationships without
 * trying to resolve the now-hidden components. */
export const canonicalStringPath = (value, label) => {
  safeText(value, label);
  if (!isAbsolute(value) || resolve(value) !== value) fail(`${label} is not one canonical absolute path`);
  return value;
};

export const executablePath = (value, label) => {
  safeText(value, label);
  if (!isAbsolute(value) || resolve(value) !== value) fail(`${label} is not one absolute executable path`);
  let info;
  try { info = lstatSync(value); } catch (error) { fail(`${label} is not an existing executable: ${error.message}`); }
  if (info.isSymbolicLink()) {
    const target = readableRealpath(value, label);
    info = lstatSync(target);
  }
  if (!info.isFile()) fail(`${label} is not one executable file`);
  return value;
};

const jsonEvidence = (value, label) => {
  const evidence = readRegularEvidence(value, label);
  try { return { path: evidence.path, value: JSON.parse(evidence.content.toString("utf8")), digest: Object.freeze({ path: evidence.path, bytes: evidence.bytes, sha256: evidence.sha256 }) }; }
  catch (error) { fail(`${label} is invalid JSON: ${error.message}`); }
};

const requirePath = (actual, expected, label) => {
  if (actual !== expected) fail(`${label} differs from its canonical expected path`);
};

const requireOption = (options, option, label) => {
  if (typeof options !== "string" || !options.split(",").includes(option)) fail(`${label} is missing ${option}`);
};

const imageStatFacts = (path) => {
  const evidence = readRegularEvidence(path, "image stat evidence");
  const text = evidence.content.toString("utf8");
  if (!text.endsWith("\n") || text.includes("\r")) fail("image stat evidence is not canonical");
  const rows = text.slice(0, -1).split("\n");
  const facts = {};
  for (const row of rows) {
    const separator = row.indexOf("=");
    if (separator < 1 || Object.prototype.hasOwnProperty.call(facts, row.slice(0, separator))) fail("image stat evidence has duplicate or malformed fields");
    facts[row.slice(0, separator)] = row.slice(separator + 1);
  }
  const expected = ["logical_bytes", "st_blocks", "allocated_bytes", "block_size"];
  if (JSON.stringify(Object.keys(facts)) !== JSON.stringify(expected)) fail("image stat evidence fields differ");
  return { facts, digest: Object.freeze({ path: evidence.path, bytes: evidence.bytes, sha256: evidence.sha256 }) };
};

export const exactFields = (value, fields, label) => {
  const actual = Object.keys(objectValue(value, label)).sort();
  const expected = [...fields].sort();
  if (JSON.stringify(actual) !== JSON.stringify(expected)) fail(`${label} fields differ`);
};

/*
 * A pass receipt is an attestation, not a projection of arbitrary facts.  The
 * shell supervisor supplies the facts, but this boundary re-reads every file
 * whose path is placed in a pass receipt and checks the relationships that
 * make the paths, counters, identities, and lifecycle one proof.
 */
export const validatePassFacts = (facts, { statReader = (path) => statSync(path, { bigint: true }) } = {}) => {
  const value = objectValue(facts, "wrapper facts");
  exactFields(value, FACT_FIELDS, "wrapper facts");
  if (value.setup_failed !== false || value.inner_pass !== true) fail("pass receipt facts do not prove setup and inner success");

  for (const [name, fields] of Object.entries(SECTION_FIELDS)) exactFields(value[name], fields, name);

  const candidate = objectValue(requiredField(value, "candidate"), "candidate");
  const candidateSource = canonicalDirectory(requiredField(candidate, "source", "candidate source"), "candidate source");
  canonicalHex(requiredField(candidate, "commit", "candidate commit"), 40, "candidate commit");
  canonicalHex(requiredField(candidate, "tree", "candidate tree"), 40, "candidate tree");
  if (candidate.clean !== true || statusCode(requiredField(candidate, "source_status"), "candidate source status") !== 0) fail("candidate is not clean");

  const image = objectValue(requiredField(value, "image"), "image");
  const imagePath = canonicalRegular(requiredField(image, "path", "image path"), "image path");
  const work = canonicalDirectory(dirname(imagePath), "work directory");
  requirePath(imagePath, join(work, "filesystem.xfs"), "image path");
  if (statusCode(requiredField(image, "status", "fallocate status"), "fallocate status") !== 0 ||
      statusCode(requiredField(image, "sync_status", "image sync status"), "image sync status") !== 0) fail("image provisioning did not pass");
  if (decimal(requiredField(image, "expected_bytes"), "image expected bytes") !== IMAGE_BYTES ||
      decimal(requiredField(image, "logical_bytes"), "image logical bytes") !== IMAGE_BYTES ||
      decimal(requiredField(image, "allocated_bytes"), "image allocated bytes") < IMAGE_BYTES) fail("image size or allocation is not exact");
  const imageActual = statReader(imagePath);
  const imageActualAllocated = imageActual.blocks * 512n;
  if (imageActual.size !== IMAGE_BYTES || imageActualAllocated < IMAGE_BYTES) fail("image file size or allocation differs from the receipt facts");
  for (const field of ["fallocate_stdout", "fallocate_stderr", "sync_stdout", "sync_stderr"]) {
    canonicalRegular(requiredField(image, field, `image ${field}`), `image ${field}`);
  }
  const filefragEvidence = validateFilefragEvidence(requiredField(image, "filefrag_path", "filefrag path"));
  const imageStatPath = requiredField(image, "stat_path", "image stat path");
  const imageStatRead = imageStatFacts(imageStatPath);
  const imageStatEvidence = imageStatRead.digest;
  const imageStat = imageStatRead.facts;
  if (decimal(imageStat.logical_bytes, "image stat logical bytes") !== IMAGE_BYTES ||
      decimal(imageStat.allocated_bytes, "image stat allocated bytes") < IMAGE_BYTES ||
      decimal(imageStat.allocated_bytes, "image stat allocated bytes") !==
        decimal(imageStat.st_blocks, "image stat blocks") * decimal(imageStat.block_size, "image stat block size")) fail("image stat allocation is inconsistent");
  if (decimal(imageStat.logical_bytes, "image stat logical bytes") !== imageActual.size ||
      decimal(imageStat.allocated_bytes, "image stat allocated bytes") !== imageActualAllocated) fail("image stat evidence differs from the image file");

  const loop = objectValue(requiredField(value, "loop_device"), "loop device");
  const loopPath = requiredField(loop, "path", "loop device path");
  if (typeof loopPath !== "string" || !/^\/dev\/loop[0-9]+$/.test(loopPath)) fail("loop device path is invalid");
  if (!/^\d+:\d+$/.test(requiredField(loop, "major_minor", "loop major:minor"))) fail("loop major:minor is invalid");
  if (decimal(requiredField(loop, "size_bytes", "loop size")) !== IMAGE_BYTES || loop.attached !== false) fail("loop device size or final attachment state is invalid");

  const filesystem = objectValue(requiredField(value, "filesystem"), "filesystem");
  if (requiredField(filesystem, "type") !== "xfs" || !UUID.test(requiredField(filesystem, "uuid"))) fail("filesystem identity is not XFS");
  const fragmentSize = decimal(requiredField(filesystem, "fragment_size"), "filesystem fragment size");
  if (fragmentSize === 0n || decimal(requiredField(filesystem, "capacity_limit_bytes")) !== IMAGE_BYTES || filesystem.capacity_ok !== true) fail("filesystem capacity binding is invalid");
  const statfsSnapshots = [];
  const statfsEvidence = {};
  for (const field of ["mounted_statfs_path", "checkout_statfs_path", "close_statfs_path"]) {
    const parsed = jsonEvidence(requiredField(filesystem, field, `filesystem ${field}`), `filesystem ${field}`);
    statfsSnapshots.push(validateStatfsSnapshot(parsed.value, { fragmentSize, capacityLimitBytes: IMAGE_BYTES }));
    statfsEvidence[field] = parsed.digest;
  }
  for (const field of ["f_type", "f_bsize", "f_blocks", "f_frsize"]) {
    if (new Set(statfsSnapshots.map((snapshot) => snapshot[field])).size !== 1) fail(`filesystem ${field} drifted across checkpoints`);
  }
  const xfsInfoEvidence = validateXfsInfoEvidence(requiredField(filesystem, "xfs_info_path", "XFS info path"));
  const hostFilesystem = {};
  for (const [field, suffix] of [["host_filesystem_before_path", "host-filesystem-before"], ["host_filesystem_after_path", "host-filesystem-after"]]) {
    const prefix = canonicalPath(requiredField(filesystem, field, `filesystem ${field}`), `filesystem ${field}`, { mustExist: false });
    requirePath(prefix, join(work, suffix), `filesystem ${field}`);
    hostFilesystem[field] = validateHostFilesystemEvidence(prefix, field);
  }
  const hostBefore = hostFilesystem.host_filesystem_before_path;
  const hostAfter = hostFilesystem.host_filesystem_after_path;
  for (const field of ["fragment_size", "filesystem_type", "filesystem_id"]) {
    if (hostBefore.statfs.facts[field] !== hostAfter.statfs.facts[field]) fail(`host filesystem ${field} drifted`);
  }
  if (JSON.stringify(hostBefore.mount.fields) !== JSON.stringify(hostAfter.mount.fields)) fail("host filesystem mount identity or options drifted");

  const mount = objectValue(requiredField(value, "mount"), "mount");
  const mountPath = canonicalDirectory(requiredField(mount, "path", "mount path"), "mount path");
  requirePath(mountPath, join(work, "fs"), "mount path");
  requirePath(requiredField(mount, "source", "mount source"), loopPath, "mount source");
  requireOption(requiredField(mount, "options", "mount options"), "rw", "mount options");
  requireOption(mount.options, "nodev", "mount options");
  requireOption(mount.options, "nosuid", "mount options");
  if (requiredField(mount, "propagation") !== "private" || statusCode(requiredField(mount, "status"), "mount status") !== 0 ||
      mount.mounted !== true || mount.unmounted !== true || mount.mount_absent !== true) fail("mount lifecycle or options are invalid");

  const exportSection = objectValue(requiredField(value, "export"), "export");
  const exportSource = canonicalStringPath(requiredField(exportSection, "source", "export source"), "export source");
  const exportDestination = canonicalDirectory(requiredField(exportSection, "destination", "export destination"), "export destination");
  const invocation = objectValue(requiredField(value, "invocation"), "invocation");
  const checkout = canonicalStringPath(requiredField(invocation, "cwd", "proof cwd"), "proof cwd");
  const output = join(checkout, "target", "r2-complete-proof");
  requirePath(exportSource, output, "export source");
  requirePath(exportDestination, join(work, "export", "target", "r2-complete-proof"), "export destination");
  assertNonOverlappingPaths(candidateSource, work);
  if (statusCode(requiredField(exportSection, "status"), "export status") !== 0 || exportSection.equal !== true) fail("export did not pass");
  const destinationInventory = canonicalInventory(exportDestination, "export destination");
  if (exportSection.source_inventory_sha256 !== destinationInventory.sha256 || exportSection.export_inventory_sha256 !== destinationInventory.sha256 ||
      !/^[0-9a-f]{64}$/.test(exportSection.source_inventory_sha256) || !/^[0-9a-f]{64}$/.test(exportSection.export_inventory_sha256)) fail("export inventory digest binding differs");
  const inventoryDocument = jsonEvidence(requiredField(exportSection, "inventory_path", "export inventory path"), "export inventory").value;
  if (inventoryDocument.source !== exportSource || inventoryDocument.destination !== exportDestination || inventoryDocument.source_inventory_sha256 !== destinationInventory.sha256 ||
      inventoryDocument.export_inventory_sha256 !== destinationInventory.sha256 || inventoryDocument.equal !== true || inventoryDocument.rows !== destinationInventory.rows.length) fail("export inventory document differs");
  const digestText = readRegularEvidence(requiredField(exportSection, "inventory_digest_path", "export inventory digest path"), "export inventory digest").content.toString("utf8");
  if (digestText !== `source\t${destinationInventory.sha256}\nexport\t${destinationInventory.sha256}\n`) fail("export inventory digest evidence differs");
  const manifestPath = requiredField(exportSection, "inner_evidence_manifest_path", "inner evidence manifest path");
  const manifest = canonicalRegular(manifestPath, "inner evidence manifest");
  requirePath(manifest, join(exportDestination, "EVIDENCE.sha256"), "inner evidence manifest");
  const manifestEvidence = validateEvidenceManifest(manifest, destinationInventory);

  const argv = requiredField(invocation, "argv", "proof argv");
  const proofScript = join(checkout, "docs", "evaluation", "r2-complete-proof.sh");
  canonicalStringPath(proofScript, "proof script");
  if (!Array.isArray(argv) || argv.length !== 4 || JSON.stringify(argv) !== JSON.stringify(["/usr/bin/bash", proofScript, "--output", output])) fail("proof argv is not exact");
  if (typeof invocation.uid !== "number" || !Number.isSafeInteger(invocation.uid) || invocation.uid < 0 ||
      typeof invocation.gid !== "number" || !Number.isSafeInteger(invocation.gid) || invocation.gid < 0 ||
      statusCode(requiredField(invocation, "status", "proof status"), "proof status") !== 0 || invocation.inner_pass !== true) fail("proof invocation identity or status is invalid");
  const startNs = decimal(requiredField(invocation, "start_ns", "proof start timestamp"), "proof start timestamp");
  const endNs = decimal(requiredField(invocation, "end_ns", "proof end timestamp"), "proof end timestamp");
  if (endNs < startNs) fail("proof invocation timestamps are reversed");
  const stdoutPath = canonicalRegular(requiredField(invocation, "stdout_path", "proof stdout path"), "proof stdout");
  const stderrPath = canonicalRegular(requiredField(invocation, "stderr_path", "proof stderr path"), "proof stderr");
  if (stdoutPath !== join(work, "proof.stdout") || stderrPath !== join(work, "proof.stderr")) fail("proof output paths differ");

  const teardown = objectValue(requiredField(value, "teardown"), "teardown");
  for (const field of ["unmounted", "loop_detached", "no_holder", "mount_absent", "image_unattached"]) {
    if (teardown[field] !== true) fail(`teardown ${field} is not true`);
  }
  for (const [field, expected] of [["fuser_status", 1], ["umount_status", 0], ["detach_status", 0], ["supervisor_status", 0]]) {
    if (statusCode(requiredField(teardown, field, `teardown ${field}`), `teardown ${field}`) !== expected) fail(`teardown ${field} is invalid`);
  }
  const nestedMonitor = objectValue(requiredField(teardown, "host_monitor", "teardown host monitor"), "teardown host monitor");
  exactFields(nestedMonitor, ["clean", "mountpoint", "proof_loop_device", "new_loop_devices", "mount_namespace", "evidence"], "teardown host monitor");
  const monitor = objectValue(value.host_monitor ?? nestedMonitor, "host monitor");
  exactFields(monitor, ["clean", "mountpoint", "proof_loop_device", "new_loop_devices", "mount_namespace", "evidence"], "host monitor");
  if (JSON.stringify(nestedMonitor) !== JSON.stringify(monitor)) fail("host monitor copies differ");
  const monitorEvidencePaths = {
    before_mount: join(work, "host-before-mount.txt"),
    after_mount: join(work, "host-after-mount.txt"),
    before_mount_stderr: join(work, "host-before-mount.stderr"),
    after_mount_stderr: join(work, "host-after-mount.stderr"),
    before_mount_status: join(work, "host-before-mount.status"),
    after_mount_status: join(work, "host-after-mount.status"),
    before_loops: join(work, "host-before-loops.json"),
    after_loops: join(work, "host-after-loops.json"),
    before_loops_status: join(work, "host-before-loops.status"),
    after_loops_status: join(work, "host-after-loops.status"),
    before_loops_stderr: join(work, "host-before-loops.stderr"),
    after_loops_stderr: join(work, "host-after-loops.stderr"),
    mount_namespace_before: join(work, "host-before-mnt-ns"),
    mount_namespace_after: join(work, "host-after-mnt-ns"),
  };
  validateBoundHostMonitor(monitor, { image: imagePath, mountpoint: mountPath, proofLoopDevice: loopPath,
    expectedEvidencePaths: monitorEvidencePaths });
  const commandEvidence = validateOperationsAndTools(value, {
    imagePath, work, mountPath, checkout, output, loopPath, exportDestination,
    inventoryPath: exportSection.inventory_path,
  });
  const commandLedger = validateCommandLedger(requiredField(invocation, "command_ledger_path", "wrapper command ledger path"), {
    work, checkout, source: candidateSource, imagePath, loopPath, mountPath, proofScript, output,
    receiptHelper: value.operations.export.argv[1], inventoryPath: exportSection.inventory_path,
    uuid: filesystem.uuid, fragmentSize: filesystem.fragment_size, majorMinor: loop.major_minor,
    hostQuotaTarget: hostBefore.mount.fields[0], callerUid: invocation.uid, callerGid: invocation.gid,
    invocationStartNs: startNs.toString(), invocationEndNs: endNs.toString(), operations: value.operations,
    executionPath: requiredField(invocation, "execution_ledger_path", "wrapper execution ledger path"),
  });
  const preflight = validateOuterPreflight(
    requiredField(invocation, "outer_preflight_path", "outer preflight path"), { work });
  return Object.freeze({
    image: Object.freeze({ stat: imageStatEvidence, filefrag: filefragEvidence, xfs_info: xfsInfoEvidence }),
    filesystem: Object.freeze({
      statfs: Object.freeze(statfsEvidence),
      host: Object.freeze(hostFilesystem),
    }),
    manifest: manifestEvidence,
    operations: commandEvidence.operations,
    tool_register: commandEvidence.tool_register,
    command_ledger: commandLedger,
    preflight,
  });
};

export const assembleReceipt = (facts = {}, options = {}) => {
  const value = section(facts);
  const exportSource = section(value.export);
  const exportPass = exportSource.status === 0 && exportSource.equal === true &&
    typeof exportSource.source_inventory_sha256 === "string" &&
    exportSource.source_inventory_sha256 === exportSource.export_inventory_sha256;
  const pass = innerStatus(value) && exportPass && teardownPass(value);
  const passEvidence = pass ? validatePassFacts(value, options) : null;
  const invocationSource = section(value.invocation);
  const invocation = {
    argv: Array.isArray(invocationSource.argv) ? invocationSource.argv : [],
    cwd: invocationSource.cwd ?? null,
    uid: invocationSource.uid ?? null,
    gid: invocationSource.gid ?? null,
    status: invocationSource.status ?? null,
    inner_pass: innerStatus(value),
    start_ns: invocationSource.start_ns ?? null,
    end_ns: invocationSource.end_ns ?? null,
    stdout: invocationSource.stdout_path ? digestRegular(invocationSource.stdout_path, "proof stdout") : null,
    stderr: invocationSource.stderr_path ? digestRegular(invocationSource.stderr_path, "proof stderr") : null,
    operations: passEvidence?.operations ?? value.operations ?? null,
    tool_register: passEvidence?.tool_register ?? value.tool_register ?? null,
    command_ledger: passEvidence?.command_ledger ?? null,
    preflight: passEvidence?.preflight ?? null,
  };
  let exportSection = {
    source: exportSource.source ?? null,
    destination: exportSource.destination ?? null,
    status: exportSource.status ?? null,
    equal: exportSource.equal ?? null,
    inventory_path: exportSource.inventory_path ?? null,
    inventory_digest_path: exportSource.inventory_digest_path ?? null,
    inventory: exportSource.inventory ?? null,
    source_inventory_sha256: exportSource.source_inventory_sha256 ?? null,
    export_inventory_sha256: exportSource.export_inventory_sha256 ?? null,
    inner_evidence_manifest_path: exportSource.inner_evidence_manifest_path ?? null,
    evidence_manifest: null,
  };
  if (passEvidence !== null) exportSection.evidence_manifest = passEvidence.manifest;

  const teardown = {
    ...(section(value.teardown)),
    host_monitor: section(value.host_monitor ?? section(value.teardown).host_monitor),
  };
  const outcome = pass ? "pass" : value.setup_failed === true ? "setup-fail" : "red";
  const receipt = {
    receipt: { schema: SCHEMA, version: 1 },
    outcome,
    candidate: canonicalCandidate(value.candidate),
    image: passEvidence === null ? section(value.image) : { ...section(value.image), evidence: passEvidence.image },
    loop_device: section(value.loop_device),
    filesystem: passEvidence === null ? section(value.filesystem) : { ...section(value.filesystem), evidence: passEvidence.filesystem },
    mount: section(value.mount),
    invocation,
    export: exportSection,
    teardown,
  };
  exactTopLevel(receipt);
  return receipt;
};

const parseArgs = (argv) => {
  const result = {};
  for (let index = 0; index < argv.length; index += 1) {
    const argument = argv[index];
    if (!argument.startsWith("--") || index + 1 >= argv.length) fail("arguments must be --name value pairs");
    const key = argument.slice(2).replaceAll("-", "_");
    if (result[key] !== undefined) fail(`duplicate argument: ${argument}`);
    result[key] = argv[index + 1];
    index += 1;
  }
  return result;
};

const requiredArg = (options, key) => {
  if (options[key] === undefined) fail(`missing --${key.replaceAll("_", "-")}`);
  return options[key];
};

const readJson = (path, label) => {
  const bytes = readRegularEvidence(path, label).content.toString("utf8");
  try { return JSON.parse(bytes); } catch (error) { fail(`${label} is invalid JSON: ${error.message}`); }
};

const readEvidenceText = (path, label) => readRegularEvidence(path, label).content.toString("utf8");

export const runCli = (argv = process.argv.slice(2)) => {
  const command = argv.shift();
  try {
    if (command === "validate-paths") {
      const options = parseArgs(argv);
      const roots = assertNonOverlappingPaths(requiredArg(options, "source"), requiredArg(options, "work"));
      assertEmptyDirectory(roots.work, "work");
      process.stdout.write(`${JSON.stringify(roots)}\n`);
      return 0;
    }
    if (command === "inventory") {
      const options = parseArgs(argv);
      const value = canonicalInventory(requiredArg(options, "root"));
      if (options.output !== undefined) writeFileSync(options.output, `${JSON.stringify(value, null, 2)}\n`, { flag: "wx" });
      process.stdout.write(`${JSON.stringify({ root: value.root, rows: value.rows, sha256: value.sha256 })}\n`);
      return 0;
    }
    if (command === "copy") {
      const options = parseArgs(argv);
      const source = requiredArg(options, "source");
      const destination = requiredArg(options, "destination");
      const sourceInventory = canonicalInventory(source, "export source");
      const destinationInventory = copyTreeNoDeref(source, destination);
      const comparison = compareInventories(sourceInventory, destinationInventory);
      const result = {
        source: sourceInventory.root,
        destination: destinationInventory.root,
        source_inventory_sha256: sourceInventory.sha256,
        export_inventory_sha256: destinationInventory.sha256,
        rows: comparison.rows,
        equal: comparison.equal,
      };
      if (options.output !== undefined) writeFileSync(options.output, `${JSON.stringify(result, null, 2)}\n`, { flag: "wx" });
      process.stdout.write(`${JSON.stringify(result)}\n`);
      return 0;
    }
    if (command === "compare-inventory") {
      const options = parseArgs(argv);
      const result = compareInventories(readJson(requiredArg(options, "left"), "left inventory"), readJson(requiredArg(options, "right"), "right inventory"));
      process.stdout.write(`${JSON.stringify(result)}\n`);
      return 0;
    }
    if (command === "du-check") {
      const options = parseArgs(argv);
      const result = parseDuOutput(
        readEvidenceText(requiredArg(options, "stdout"), "du stdout"), requiredArg(options, "path"),
        readEvidenceText(requiredArg(options, "stderr"), "du stderr"), Number(requiredArg(options, "status")),
      );
      process.stdout.write(`${JSON.stringify({ ...result, mib: result.mib.toString() })}\n`);
      return 0;
    }
    if (command === "du-validate") {
      const options = parseArgs(argv);
      const du = parseDuOutput(
        readEvidenceText(requiredArg(options, "stdout"), "du stdout"), requiredArg(options, "path"),
        readEvidenceText(requiredArg(options, "stderr"), "du stderr"), Number(requiredArg(options, "status")),
      );
      const snapshot = readJson(requiredArg(options, "statfs"), "statfs snapshot");
      const accounting = accountingFromStatfs(snapshot, { fragmentSize: requiredArg(options, "fragment_size") });
      validateDuAgainstStatfs(du, accounting);
      process.stdout.write(`${JSON.stringify({ ...du, mib: du.mib.toString(), allocated_mib: accounting.mebibytes.toString() })}\n`);
      return 0;
    }
    if (command === "receipt") {
      const options = parseArgs(argv);
      let facts = readJson(requiredArg(options, "facts"), "facts");
      if (options.host_monitor !== undefined) {
        const monitor = readJson(options.host_monitor, "host monitor");
        facts = bindHostMonitor(facts, monitor);
      }
      const receipt = assembleReceipt(facts);
      const output = requiredArg(options, "output");
      writeFileSync(output, `${JSON.stringify(receipt, null, 2)}\n`, { flag: "wx" });
      process.stdout.write(`${JSON.stringify({ outcome: receipt.outcome, schema: SCHEMA })}\n`);
      return receipt.outcome === "pass" ? 0 : 1;
    }
    if (command === "host-check") {
      const options = parseArgs(argv);
      const beforeMount = requiredArg(options, "before_mount");
      const afterMount = requiredArg(options, "after_mount");
      const beforeMountStderr = requiredArg(options, "before_mount_stderr");
      const afterMountStderr = requiredArg(options, "after_mount_stderr");
      const beforeMountStatus = requiredArg(options, "before_mount_status");
      const afterMountStatus = requiredArg(options, "after_mount_status");
      const beforeLoops = requiredArg(options, "before_loops");
      const afterLoops = requiredArg(options, "after_loops");
      const beforeLoopsStatus = requiredArg(options, "before_loops_status");
      const afterLoopsStatus = requiredArg(options, "after_loops_status");
      const beforeLoopsStderr = requiredArg(options, "before_loops_stderr");
      const afterLoopsStderr = requiredArg(options, "after_loops_stderr");
      const mountNamespaceBefore = requiredArg(options, "mount_ns_before");
      const mountNamespaceAfter = requiredArg(options, "mount_ns_after");
      const result = validateHostMonitor({
        evidencePaths: {
          before_mount: beforeMount, after_mount: afterMount,
          before_mount_stderr: beforeMountStderr, after_mount_stderr: afterMountStderr,
          before_mount_status: beforeMountStatus, after_mount_status: afterMountStatus,
          before_loops: beforeLoops, after_loops: afterLoops,
          before_loops_status: beforeLoopsStatus, after_loops_status: afterLoopsStatus,
          before_loops_stderr: beforeLoopsStderr, after_loops_stderr: afterLoopsStderr,
          mount_namespace_before: mountNamespaceBefore, mount_namespace_after: mountNamespaceAfter,
        },
        image: requiredArg(options, "image"),
        mountpoint: requiredArg(options, "mountpoint"),
        proofLoopDevice: options.proof_loop_device,
      });
      process.stdout.write(`${JSON.stringify(result)}\n`);
      return 0;
    }
    process.stderr.write("usage: r2-complete-proof-xfs-receipt.mjs validate-paths|inventory|copy|compare-inventory|du-check|du-validate|receipt|host-check ...\n");
    return 2;
  } catch (error) {
    process.stderr.write(`${error instanceof Error ? error.message : String(error)}\n`);
    return 1;
  }
};

let invokedAsMain = false;
try { invokedAsMain = Boolean(process.argv[1]) && realpathSync.native(process.argv[1]) === fileURLToPath(import.meta.url); }
catch { invokedAsMain = false; }
if (invokedAsMain) {
  process.exitCode = runCli();
}
