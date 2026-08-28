import {
  closeSync,
  fsyncSync,
  lstatSync,
  linkSync,
  mkdirSync,
  openSync,
  readFileSync,
  realpathSync,
  unlinkSync,
  writeSync,
} from "node:fs";
import { dirname, resolve } from "node:path";
import { fileURLToPath } from "node:url";

import {
  MAX_CAPACITY_BYTES,
  MAX_SAMPLE_GAP_NS,
  RAW_HEADER,
  SAMPLE_PERIOD_NS,
  SampleGapError,
  FilesystemAccountingError,
  canonicalUnsignedDecimal,
  captureFilesystemIdentity,
  filesystemIdentityDocument,
  parseRawRow,
  publicTsvFromRaw,
  rawRow,
  sampleFilesystem,
  scheduleDeadline,
  sleepUntil,
  validateRawLedger,
} from "./r2-filesystem-accounting.mjs";

const fail = (message) => {
  throw new FilesystemAccountingError(message);
};

const writeExclusive = (path, content) => {
  mkdirSync(dirname(path), { recursive: true });
  const fd = openSync(path, "wx", 0o644);
  try {
    writeSync(fd, content, undefined, "utf8");
    fsyncSync(fd);
  } finally {
    closeSync(fd);
  }
};

export const writeNewFile = writeExclusive;

const pathIsAbsent = (path) => {
  try {
    lstatSync(path);
    return false;
  } catch (error) {
    if (error.code === "ENOENT") return true;
    throw error;
  }
};

export const assertFreshMarker = (path) => {
  if (!pathIsAbsent(path)) fail(`control marker path is not fresh: ${path}`);
};

export const publishMarker = (path, value, { beforeLink } = {}) => {
  const content = `${canonicalUnsignedDecimal(value, "marker")}\n`;
  assertFreshMarker(path);
  mkdirSync(dirname(path), { recursive: true });
  const temporary = `${path}.tmp-${process.pid}-${process.hrtime.bigint()}`;
  try {
    writeExclusive(temporary, content);
    if (beforeLink !== undefined) {
      if (typeof beforeLink !== "function") fail("marker before-link hook is not callable");
      beforeLink();
    }
    try {
      // A hard link is an atomic create-if-absent publication. Unlike rename,
      // it cannot replace a marker created by a competing publisher.
      linkSync(temporary, path);
    } catch (error) {
      if (error.code === "EEXIST") fail(`control marker path is not fresh: ${path}`);
      throw error;
    }
  } catch (error) {
    try { unlinkSync(temporary); } catch (cleanupError) { if (cleanupError.code !== "ENOENT") throw cleanupError; }
    throw error;
  }
  unlinkSync(temporary);
};

export const readMarker = (path) => {
  if (pathIsAbsent(path)) return null;
  const text = readFileSync(path, "utf8");
  if (!/^(0|[1-9][0-9]*)\n$/.test(text)) fail(`malformed control marker: ${path}`);
  return BigInt(text.slice(0, -1));
};

export class PreallocatedRawLedger {
  constructor(maxRows = 100_000) {
    if (!Number.isSafeInteger(maxRows) || maxRows < 2) fail("raw ledger capacity is invalid");
    this.capacity = maxRows;
    this.slots = new Array(maxRows);
    this.count = 0;
  }

  append(line) {
    const value = typeof line === "string" ? line : rawRow(line);
    parseRawRow(value);
    if (this.count >= this.capacity) fail("raw ledger capacity exhausted");
    this.slots[this.count] = value;
    this.count += 1;
    return this.count - 1;
  }

  lines() {
    return this.slots.slice(0, this.count);
  }

  text() {
    return `${RAW_HEADER}\n${this.lines().join("\n")}\n`;
  }

  flush(rawPath, publicPath) {
    writeExclusive(rawPath, this.text());
    if (publicPath !== undefined) writeExclusive(publicPath, publicTsvFromRaw(this.lines()));
  }
}

const parseCli = (argv) => {
  const values = {};
  for (let index = 0; index < argv.length; index += 1) {
    const argument = argv[index];
    if (!argument.startsWith("--") || index + 1 >= argv.length) fail("sampler arguments must be --name value pairs");
    values[argument.slice(2).replaceAll("-", "_")] = argv[index + 1];
    index += 1;
  }
  return values;
};

const requiredCli = (options, name) => {
  if (options[name] === undefined) fail(`sampler requires --${name.replaceAll("_", "-")}`);
  return options[name];
};

const canonicalPath = (value, label) => {
  if (typeof value !== "string" || !value.startsWith("/")) fail(`${label} must be an absolute path`);
  const lexical = resolve(value);
  if (lexical !== value) fail(`${label} is not canonical: ${value}`);
  const actual = realpathSync(value);
  if (actual !== value) fail(`${label} resolves to a different path`);
  return actual;
};

const waitForMarker = async (path, timeoutNs = 300_000_000_000n) => {
  const deadline = process.hrtime.bigint() + timeoutNs;
  while (process.hrtime.bigint() < deadline) {
    const marker = readMarker(path);
    if (marker !== null) return marker;
    await new Promise((resolvePromise) => setTimeout(resolvePromise, 5));
  }
  fail(`control marker did not arrive: ${path}`);
};

const removeMarker = (path) => {
  try {
    unlinkSync(path);
  } catch (error) {
    if (error.code !== "ENOENT") throw error;
  }
};

export const runPersistentSampler = async (options) => {
  const root = canonicalPath(requiredCli(options, "root"), "checkout root");
  const target = canonicalPath(requiredCli(options, "target"), "target");
  const output = canonicalPath(requiredCli(options, "output"), "output");
  const rawPath = requiredCli(options, "raw");
  const publicPath = requiredCli(options, "public");
  const identityPath = requiredCli(options, "identity");
  const stateDir = requiredCli(options, "state_dir");
  const stopPath = requiredCli(options, "stop");
  const device = requiredCli(options, "device");
  const majorMinor = requiredCli(options, "major_minor");
  const fragmentSize = requiredCli(options, "fragment_size");
  const uuid = requiredCli(options, "uuid");
  const periodNs = options.period_ns === undefined ? SAMPLE_PERIOD_NS : BigInt(canonicalUnsignedDecimal(options.period_ns, "nominal period"));
  const maxGapNs = options.max_gap_ns === undefined ? MAX_SAMPLE_GAP_NS : BigInt(canonicalUnsignedDecimal(options.max_gap_ns, "maximum sample gap"));
  const maxRows = options.max_rows === undefined ? 100_000 : Number(options.max_rows);
  if (periodNs === 0n) fail("nominal period is zero");
  if (maxGapNs === 0n) fail("maximum sample gap is zero");
  const drainPath = `${stateDir}/drain-request`;
  const readyPath = `${stateDir}/ready`;
  const drainReadyPath = `${stateDir}/drain-ready`;
  for (const markerPath of [stopPath, drainPath, readyPath, drainReadyPath]) assertFreshMarker(markerPath);
  mkdirSync(stateDir, { recursive: true });
  const baseline = captureFilesystemIdentity(root, {
    fragmentSize,
    uuid,
    target,
    output,
    device,
    majorMinor,
    capacityLimitBytes: MAX_CAPACITY_BYTES,
  });
  const ledger = new PreallocatedRawLedger(maxRows);
  const originNs = process.hrtime.bigint();
  let previous = null;
  let ordinal = 0n;

  const appendSample = (kind) => {
    const sampleStartNs = process.hrtime.bigint();
    if (previous !== null) {
      if (sampleStartNs <= previous) fail("sample clock regressed");
      const gap = sampleStartNs - previous;
      if (gap > maxGapNs) throw new SampleGapError(gap, maxGapNs);
    }
    const current = sampleFilesystem(root, baseline);
    const deadlineNs = kind === "scheduled" ? scheduleDeadline(originNs, ordinal, periodNs) : sampleStartNs;
    ledger.append(rawRow({
      ordinal,
      sampleStartNs,
      elapsedNs: sampleStartNs - originNs,
      deadlineNs,
      identity: current,
      accounting: current.accounting,
      kind,
    }));
    previous = sampleStartNs;
    ordinal += 1n;
  };

  try {
    appendSample("scheduled");
    publishMarker(readyPath, originNs);
    let draining = false;
    while (!draining) {
      if (readMarker(stopPath) !== null) break;
      const request = readMarker(drainPath);
      if (request !== null) {
        await sleepUntil(scheduleDeadline(originNs, ordinal, periodNs));
        appendSample("scheduled");
        publishMarker(drainReadyPath, request);
        draining = true;
        break;
      }
      await sleepUntil(scheduleDeadline(originNs, ordinal, periodNs));
      appendSample("scheduled");
    }
    if (draining) await waitForMarker(stopPath);
    appendSample("terminal");
    const rawText = ledger.text();
    validateRawLedger(rawText, baseline, { originNs, periodNs, maxGapNs });
    ledger.flush(rawPath, publicPath);
    writeNewFile(identityPath, filesystemIdentityDocument(baseline, {
      samplerOriginNs: originNs,
      nominalIntervalNs: periodNs,
    }));
    removeMarker(readyPath);
    removeMarker(drainPath);
    removeMarker(drainReadyPath);
  } catch (error) {
    try {
      if (ledger.count > 0 && pathIsAbsent(rawPath)) writeNewFile(rawPath, ledger.text());
      if (pathIsAbsent(identityPath)) writeNewFile(identityPath, filesystemIdentityDocument(baseline, {
        samplerOriginNs: originNs,
        nominalIntervalNs: periodNs,
      }));
    } catch {
      // Preserve the original sampler failure; the parent records missing red evidence.
    }
    throw error;
  }
};

export const runSamplerCli = async (argv = process.argv.slice(2)) => {
  if (argv[0] !== "sample") {
    process.stderr.write("usage: r2-filesystem-sampler.mjs sample --root PATH --target PATH --output PATH --raw PATH --public PATH --identity PATH --state-dir PATH --stop PATH --device DEVICE --major-minor M:N --fragment-size N --uuid UUID\n");
    return 2;
  }
  try {
    await runPersistentSampler(parseCli(argv.slice(1)));
    return 0;
  } catch (error) {
    process.stderr.write(`${error instanceof Error ? error.message : String(error)}\n`);
    return 1;
  }
};

if (process.argv[1] && resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  runSamplerCli().then((status) => { process.exitCode = status; });
}
