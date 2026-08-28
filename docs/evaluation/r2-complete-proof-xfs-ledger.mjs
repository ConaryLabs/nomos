#!/usr/bin/env node

import { createHash } from "node:crypto";
import { closeSync, fstatSync, lstatSync, openSync, readSync, realpathSync, statSync } from "node:fs";
import { isAbsolute, join, resolve } from "node:path";

const SAFE_TEXT = /^[^\0\r\n\t]+$/;
const DECIMAL = /^(0|[1-9][0-9]*)$/;
const RECORD_FIELDS = Object.freeze([
  "id", "started_ns", "ended_ns", "status", "uid", "gid", "cwd", "argv",
  "stdout_path", "stderr_path",
]);
const EXECUTION_RECORD_FIELDS = Object.freeze([
  "id", "started_ns", "ended_ns", "status", "uid", "gid",
  "actual_argv", "actual_cwd", "actual_stdout_path", "actual_stderr_path",
  "bound_argv", "bound_cwd", "bound_stdout_path", "bound_stderr_path",
  "canonical_work_path", "work_identity", "work_identity_before", "work_identity_after",
]);
const OUTER_PREFLIGHT_FIELDS = Object.freeze([
  "cap_ambient", "cap_bounding", "cap_effective", "cap_inheritable", "cap_permitted",
  "host_network_namespace", "host_pid_namespace", "network_namespace", "no_new_privs",
  "pid_namespace",
]);
const COMMAND_LEDGER_ROWS = Object.freeze([
  ["host-filesystem-before.quota", "host-filesystem-before.quota", "host-filesystem-before.quota.stderr"],
  ["image-fallocate", "image-fallocate.stdout", "image-fallocate.stderr"],
  ["image-sync", "image-sync.stdout", "image-sync.stderr"],
  ["image.filefrag", "image.filefrag", "image.filefrag.stderr"],
  ["host-filesystem-after.quota", "host-filesystem-after.quota", "host-filesystem-after.quota.stderr"],
  ["loop-attach", "loop-attach.stdout", "loop-attach.stderr"],
  ["loop-size", "loop-size.stdout", "loop-size.stderr"],
  ["mkfs-xfs", "mkfs-xfs.stdout", "mkfs-xfs.stderr"],
  ["xfs-info.txt", "xfs-info.txt", "xfs-info.stderr"],
  ["blkid", "blkid.stdout", "blkid.stderr"],
  ["xfs-uuid", "xfs-uuid.stdout", "xfs-uuid.stderr"],
  ["mount", "mount.stdout", "mount.stderr"],
  ["clone", "clone.stdout", "clone.stderr"],
  ["inner-proof", "proof.stdout", "proof.stderr"],
  ["export", "export.stdout", "export.stderr"],
  ["fuser", "fuser.stdout", "fuser.stderr"],
  ["sync-before-umount", "sync-before-umount.stdout", "sync-before-umount.stderr"],
  ["umount", "umount.stdout", "umount.stderr"],
  ["loop-detach", "loop-detach.stdout", "loop-detach.stderr"],
  ["loop-fuser", "loop-fuser.stdout", "loop-fuser.stderr"],
  ["loop-associated", "loop-associated.stdout", "loop-associated.stderr"],
]);

const fail = (message) => { throw new Error(message); };
const required = (condition, message) => { if (!condition) fail(message); };

const safeText = (value, label) => {
  required(typeof value === "string" && SAFE_TEXT.test(value), `${label} is not safe text`);
  return value;
};

const decimal = (value, label) => {
  required(typeof value === "string" && DECIMAL.test(value), `${label} is not canonical unsigned decimal`);
  return BigInt(value);
};

const canonicalRegular = (value, label) => {
  required(typeof value === "string" && isAbsolute(value) && resolve(value) === value && SAFE_TEXT.test(value),
    `${label} is not one canonical absolute path`);
  let actual;
  try { actual = realpathSync.native(value); }
  catch (error) { fail(`${label} is not an existing path: ${error.message}`); }
  required(actual === value, `${label} traverses a symlink`);
  const info = lstatSync(value, { bigint: true });
  required(info.isFile() && !info.isSymbolicLink(), `${label} is not one regular file`);
  return info;
};

const readRegularEvidence = (path, label) => {
  const info = canonicalRegular(path, label);
  const hash = createHash("sha256");
  const descriptor = openSync(path, "r");
  const buffer = Buffer.allocUnsafe(1024 * 1024);
  const chunks = [];
  let bytes = 0n;
  try {
    const opened = fstatSync(descriptor, { bigint: true });
    required(opened.isFile() && opened.dev === info.dev && opened.ino === info.ino,
      `${label} identity changed while opening`);
    for (;;) {
      const count = readSync(descriptor, buffer, 0, buffer.length, null);
      if (count === 0) break;
      const chunk = Buffer.from(buffer.subarray(0, count));
      chunks.push(chunk);
      hash.update(chunk);
      bytes += BigInt(count);
    }
    const closed = fstatSync(descriptor, { bigint: true });
    required(closed.isFile() && closed.dev === opened.dev && closed.ino === opened.ino &&
      closed.size === opened.size && bytes === opened.size,
    `${label} changed while reading`);
  } finally {
    closeSync(descriptor);
  }
  return Object.freeze({ path, bytes: bytes.toString(), sha256: hash.digest("hex"), content: Buffer.concat(chunks) });
};

const digestRegular = (path, label) => {
  const { content: _content, ...digest } = readRegularEvidence(path, label);
  return Object.freeze(digest);
};

const readCommandLedger = (path) => {
  const evidence = readRegularEvidence(path, "wrapper command ledger");
  const { content, ...digest } = evidence;
  const text = content.toString("utf8");
  required(text.length > 0 && text.endsWith("\n"), "wrapper command ledger is empty or not newline terminated");
  const records = text.slice(0, -1).split("\n").map((line, index) => {
    let record;
    try { record = JSON.parse(line); }
    catch (error) { fail(`wrapper command ledger row ${index + 1} is invalid JSON: ${error.message}`); }
    required(JSON.stringify(record) === line,
      `wrapper command ledger row ${index + 1} is not canonical JSON`);
    required(record && typeof record === "object" && !Array.isArray(record),
      `wrapper command ledger row ${index + 1} is not an object`);
    required(JSON.stringify(Object.keys(record).sort()) === JSON.stringify([...RECORD_FIELDS].sort()),
      `wrapper command ledger row ${index + 1} fields differ`);
    safeText(record.id, `wrapper command ledger row ${index + 1} id`);
    const started = decimal(record.started_ns, `wrapper command ledger row ${index + 1} start`);
    const ended = decimal(record.ended_ns, `wrapper command ledger row ${index + 1} end`);
    required(ended >= started, `wrapper command ledger row ${index + 1} timestamps are reversed`);
    required(Number.isInteger(record.status), `wrapper command ledger row ${index + 1} status is not an integer`);
    required(Array.isArray(record.argv) && record.argv.length > 0,
      `wrapper command ledger row ${index + 1} argv is empty`);
    record.argv.forEach((argument, argumentIndex) =>
      safeText(argument, `wrapper command ledger row ${index + 1} argv ${argumentIndex}`));
    required(typeof record.cwd === "string" && isAbsolute(record.cwd) && resolve(record.cwd) === record.cwd && SAFE_TEXT.test(record.cwd),
      `wrapper command ledger row ${index + 1} cwd is not a canonical absolute path`);
    canonicalRegular(record.stdout_path, `wrapper command ledger row ${index + 1} stdout`);
    canonicalRegular(record.stderr_path, `wrapper command ledger row ${index + 1} stderr`);
    required(Number.isInteger(record.uid) && record.uid >= 0,
      `wrapper command ledger row ${index + 1} uid is invalid`);
    required(Number.isInteger(record.gid) && record.gid >= 0,
      `wrapper command ledger row ${index + 1} gid is invalid`);
    return Object.freeze(record);
  });
  return Object.freeze({ ...digest, records: Object.freeze(records) });
};

const readExecutionLedger = (path) => {
  const evidence = readRegularEvidence(path, "wrapper execution ledger");
  const { content, ...digest } = evidence;
  const text = content.toString("utf8");
  required(text.length > 0 && text.endsWith("\n"),
    "wrapper execution ledger is empty or not newline terminated");
  const records = text.slice(0, -1).split("\n").map((line, index) => {
    let record;
    try { record = JSON.parse(line); }
    catch (error) { fail(`wrapper execution ledger row ${index + 1} is invalid JSON: ${error.message}`); }
    required(JSON.stringify(record) === line,
      `wrapper execution ledger row ${index + 1} is not canonical JSON`);
    required(record && typeof record === "object" && !Array.isArray(record),
      `wrapper execution ledger row ${index + 1} is not an object`);
    required(JSON.stringify(Object.keys(record).sort()) === JSON.stringify([...EXECUTION_RECORD_FIELDS].sort()),
      `wrapper execution ledger row ${index + 1} fields differ`);
    safeText(record.id, `wrapper execution ledger row ${index + 1} id`);
    const started = decimal(record.started_ns, `wrapper execution ledger row ${index + 1} start`);
    const ended = decimal(record.ended_ns, `wrapper execution ledger row ${index + 1} end`);
    required(ended >= started, `wrapper execution ledger row ${index + 1} timestamps are reversed`);
    required(Number.isInteger(record.status),
      `wrapper execution ledger row ${index + 1} status is not an integer`);
    required(Number.isInteger(record.uid) && record.uid >= 0,
      `wrapper execution ledger row ${index + 1} uid is invalid`);
    required(Number.isInteger(record.gid) && record.gid >= 0,
      `wrapper execution ledger row ${index + 1} gid is invalid`);
    for (const name of ["actual_argv", "bound_argv"]) {
      required(Array.isArray(record[name]) && record[name].length > 0,
        `wrapper execution ledger row ${index + 1} ${name} is empty`);
      record[name].forEach((argument, argumentIndex) =>
        safeText(argument, `wrapper execution ledger row ${index + 1} ${name} ${argumentIndex}`));
    }
    for (const name of [
      "actual_cwd", "actual_stdout_path", "actual_stderr_path", "bound_cwd",
      "bound_stdout_path", "bound_stderr_path", "canonical_work_path",
      "work_identity", "work_identity_before", "work_identity_after",
    ]) safeText(record[name], `wrapper execution ledger row ${index + 1} ${name}`);
    for (const name of ["work_identity", "work_identity_before", "work_identity_after"]) {
      required(/^[0-9]+:[0-9]+$/.test(record[name]),
        `wrapper execution ledger row ${index + 1} ${name} is malformed`);
    }
    return Object.freeze(record);
  });
  return Object.freeze({ ...digest, records: Object.freeze(records) });
};

const descriptorRoot = (path, label) => {
  const match = /^\/proc\/self\/fd\/([1-9][0-9]*)(?:\/|$)/.exec(path);
  required(match !== null, `${label} is not descriptor-derived`);
  return `/proc/self/fd/${match[1]}`;
};

const mapExecutionValue = (value, root, work, label) => {
  safeText(value, label);
  const mapPath = (path) => {
    if (path === root) return work;
    if (path.startsWith(`${root}/`)) return `${work}${path.slice(root.length)}`;
    return null;
  };
  const direct = mapPath(value);
  if (direct !== null) return direct;
  const separator = value.indexOf("=");
  if (separator >= 1) {
    const key = value.slice(0, separator + 1);
    const mapped = mapPath(value.slice(separator + 1));
    if (mapped !== null) return `${key}${mapped}`;
  }
  required(!value.includes("/proc/self/fd/"), `${label} references an unexpected descriptor`);
  return value;
};

const workPathValue = (value, work) => {
  if (value === work || value.startsWith(`${work}/`)) return true;
  const separator = value.indexOf("=");
  if (separator < 1) return false;
  const path = value.slice(separator + 1);
  return path === work || path.startsWith(`${work}/`);
};

const descriptorWorkValue = (value, root) => {
  if (value === root || value.startsWith(`${root}/`)) return true;
  const separator = value.indexOf("=");
  if (separator < 1) return false;
  const path = value.slice(separator + 1);
  return path === root || path.startsWith(`${root}/`);
};

const validateExecutionLedger = (path, semantic, work) => {
  const execution = readExecutionLedger(path);
  required(execution.path === join(work, "wrapper-execution.ndjson"),
    "wrapper execution ledger path differs");
  required(execution.records.length === semantic.records.length,
    "wrapper execution ledger row count differs");
  const root = descriptorRoot(execution.records[0]?.actual_stdout_path ?? "",
    "wrapper execution ledger descriptor root");
  const workInfo = statSync(work, { bigint: true });
  required(workInfo.isDirectory(), "wrapper execution ledger work path is not a directory");
  const actualIdentity = `${workInfo.dev}:${workInfo.ino}`;
  const dangerousLastArguments = Object.freeze({
    "loop-attach": `${root}/filesystem.xfs`,
    mount: `${root}/fs`,
    fuser: `${root}/fs`,
    "sync-before-umount": `${root}/fs`,
    umount: `${root}/fs`,
    "loop-associated": `${root}/filesystem.xfs`,
  });
  for (let index = 0; index < semantic.records.length; index += 1) {
    const bound = semantic.records[index];
    const record = execution.records[index];
    for (const name of ["id", "started_ns", "ended_ns", "status", "uid", "gid"]) {
      required(record[name] === bound[name],
        `wrapper execution ledger ${bound.id} ${name} differs from semantic ledger`);
    }
    required(record.canonical_work_path === work,
      `wrapper execution ledger ${bound.id} canonical work path differs`);
    required(record.work_identity === actualIdentity && record.work_identity_before === actualIdentity &&
      record.work_identity_after === actualIdentity,
    `wrapper execution ledger ${bound.id} work identity differs`);
    required(JSON.stringify(record.bound_argv) === JSON.stringify(bound.argv) &&
      record.bound_cwd === bound.cwd && record.bound_stdout_path === bound.stdout_path &&
      record.bound_stderr_path === bound.stderr_path,
    `wrapper execution ledger ${bound.id} bound projection differs`);
    required(descriptorRoot(record.actual_cwd, `wrapper execution ledger ${bound.id} actual cwd`) === root &&
      descriptorRoot(record.actual_stdout_path, `wrapper execution ledger ${bound.id} actual stdout`) === root &&
      descriptorRoot(record.actual_stderr_path, `wrapper execution ledger ${bound.id} actual stderr`) === root,
    `wrapper execution ledger ${bound.id} uses inconsistent descriptor roots`);
    const mappedArgv = record.actual_argv.map((argument, argumentIndex) =>
      mapExecutionValue(argument, root, work,
        `wrapper execution ledger ${bound.id} actual argv ${argumentIndex}`));
    for (let argumentIndex = 0; argumentIndex < bound.argv.length; argumentIndex += 1) {
      if (workPathValue(bound.argv[argumentIndex], work)) {
        required(descriptorWorkValue(record.actual_argv[argumentIndex], root),
          `wrapper execution ledger ${bound.id} work argv ${argumentIndex} is not descriptor-derived`);
      }
    }
    required(JSON.stringify(mappedArgv) === JSON.stringify(bound.argv) &&
      mapExecutionValue(record.actual_cwd, root, work,
        `wrapper execution ledger ${bound.id} actual cwd`) === bound.cwd &&
      mapExecutionValue(record.actual_stdout_path, root, work,
        `wrapper execution ledger ${bound.id} actual stdout`) === bound.stdout_path &&
      mapExecutionValue(record.actual_stderr_path, root, work,
        `wrapper execution ledger ${bound.id} actual stderr`) === bound.stderr_path,
    `wrapper execution ledger ${bound.id} actual projection differs`);
    if (Object.prototype.hasOwnProperty.call(dangerousLastArguments, bound.id)) {
      required(record.actual_argv.at(-1) === dangerousLastArguments[bound.id],
        `wrapper execution ledger ${bound.id} privileged path is not descriptor-derived`);
    }
  }
  return Object.freeze({
    path: execution.path, bytes: execution.bytes, sha256: execution.sha256,
    record_count: execution.records.length, descriptor_root: root,
    work_identity: actualIdentity, records: execution.records,
  });
};

export const validateOuterPreflight = (path, { work } = {}) => {
  const evidence = readRegularEvidence(path, "outer preflight evidence");
  const { content, ...digest } = evidence;
  required(digest.path === join(work, "outer-preflight.json"), "outer preflight path differs");
  const text = content.toString("utf8");
  required(text.length > 0 && text.endsWith("\n") && text.indexOf("\n") === text.length - 1,
    "outer preflight evidence is not one newline-terminated record");
  let value;
  try { value = JSON.parse(text); }
  catch (error) { fail(`outer preflight evidence is invalid JSON: ${error.message}`); }
  required(value && typeof value === "object" && !Array.isArray(value),
    "outer preflight evidence is not an object");
  required(JSON.stringify(value) === text.slice(0, -1), "outer preflight evidence is not canonical JSON");
  required(JSON.stringify(Object.keys(value).sort()) === JSON.stringify([...OUTER_PREFLIGHT_FIELDS].sort()),
    "outer preflight evidence fields differ");
  for (const name of [
    "cap_ambient", "cap_bounding", "cap_effective", "cap_inheritable", "cap_permitted",
  ]) required(value[name] === "0000000000000000", `outer preflight ${name} is not zero`);
  required(value.no_new_privs === 1, "outer preflight no_new_privs is not one");
  required(/^net:\[[0-9]+\]$/.test(value.host_network_namespace) &&
    /^net:\[[0-9]+\]$/.test(value.network_namespace) &&
    value.host_network_namespace !== value.network_namespace,
  "outer preflight network namespace is not fresh");
  required(/^pid:\[[0-9]+\]$/.test(value.host_pid_namespace) &&
    /^pid:\[[0-9]+\]$/.test(value.pid_namespace) &&
    value.host_pid_namespace !== value.pid_namespace,
  "outer preflight PID namespace is not fresh");
  const hostEvidence = {};
  for (const [name, kind, expected] of [
    ["network_before", "net", value.host_network_namespace],
    ["network_after", "net", value.host_network_namespace],
    ["pid_before", "pid", value.host_pid_namespace],
    ["pid_after", "pid", value.host_pid_namespace],
  ]) {
    const phase = name.endsWith("before") ? "before" : "after";
    const evidencePath = join(work, `host-${phase}-${kind}-ns`);
    const hostRead = readRegularEvidence(evidencePath, `outer preflight host ${name}`);
    const { content: hostContent, ...evidence } = hostRead;
    const evidenceText = hostContent.toString("utf8");
    required(evidenceText === `${expected}\n`,
      `outer preflight host ${name} namespace differs`);
    hostEvidence[name] = evidence;
  }
  return Object.freeze({
    ...digest, facts: Object.freeze(value), host_evidence: Object.freeze(hostEvidence),
  });
};

const operationArgv = (operations, name) => {
  const operation = operations?.[name];
  required(operation && typeof operation === "object" && !Array.isArray(operation),
    `operation ${name} is missing`);
  required(Array.isArray(operation.argv) && operation.argv.length > 0,
    `operation ${name} argv is missing`);
  return operation.argv;
};

/*
 * The shell ledger is intentionally a closed, fixed sequence.  Keep the
 * command spelling here, beside the row-order contract, instead of accepting
 * an arbitrary argv projection from the facts document.  The few values which
 * vary per run are derived from already-validated operation/fact paths.
 */
const expectedArgv = ({ work, checkout, source, imagePath, loopPath, mountPath,
  proofScript, output, receiptHelper, inventoryPath, uuid, fragmentSize,
  majorMinor, hostQuotaTarget, operations }) => ({
  "host-filesystem-before.quota": ["/usr/sbin/xfs_quota", "-x", "-c", "state -v", hostQuotaTarget],
  "image-fallocate": operationArgv(operations, "fallocate"),
  "image-sync": operationArgv(operations, "image_sync"),
  "image.filefrag": ["/usr/sbin/filefrag", "-v", imagePath],
  "host-filesystem-after.quota": ["/usr/sbin/xfs_quota", "-x", "-c", "state -v", hostQuotaTarget],
  "loop-attach": operationArgv(operations, "loop_attach"),
  "loop-size": ["/usr/sbin/blockdev", "--getsize64", loopPath],
  "mkfs-xfs": operationArgv(operations, "mkfs_xfs"),
  "xfs-info.txt": ["/usr/sbin/xfs_info", loopPath],
  blkid: ["/usr/sbin/blkid", "-p", "-s", "TYPE", "-o", "value", loopPath],
  "xfs-uuid": ["/usr/sbin/blkid", "-p", "-s", "UUID", "-o", "value", loopPath],
  mount: operationArgv(operations, "mount"),
  clone: ["/usr/bin/git", "-c", "protocol.file.allow=always", "clone", "--no-local", "--no-hardlinks", "--no-checkout", "--config", "core.hooksPath=/dev/null", source, checkout],
  "inner-proof": ["/usr/bin/env", "NOMOS_R2_XFS_WRAPPER=1", `NOMOS_R2_XFS_UUID=${uuid}`,
    `NOMOS_R2_XFS_FRAGMENT_SIZE=${fragmentSize}`, `NOMOS_R2_XFS_DEVICE=${loopPath}`,
    `NOMOS_R2_XFS_MAJOR_MINOR=${majorMinor}`,
    `NOMOS_R2_OUTER_PREFLIGHT_LOG=${work}/outer-preflight.json`,
    `NOMOS_R2_OUTER_POSITIVE_STDOUT=${work}/network-outer-positive.stdout`,
    `NOMOS_R2_OUTER_POSITIVE_STDERR=${work}/network-outer-positive.stderr`,
    "/usr/bin/bash", proofScript, "--output", output],
  export: operationArgv(operations, "export"),
  fuser: ["/usr/bin/fuser", "-m", mountPath],
  "sync-before-umount": operationArgv(operations, "sync_before_umount"),
  umount: operationArgv(operations, "umount"),
  "loop-detach": operationArgv(operations, "loop_detach"),
  "loop-fuser": ["/usr/bin/fuser", loopPath],
  "loop-associated": ["/usr/sbin/losetup", "--associated", imagePath],
});

export const validateCommandLedger = (path, {
  work, checkout, source, imagePath, loopPath, mountPath, proofScript, output,
  receiptHelper, inventoryPath, uuid, fragmentSize, majorMinor,
  hostQuotaTarget, callerUid, callerGid, invocationStartNs, invocationEndNs, operations,
  executionPath,
} = {}) => {
  const ledger = readCommandLedger(path);
  required(ledger.path === join(work, "wrapper-commands.ndjson"), "wrapper command ledger path differs");
  const expectedIds = COMMAND_LEDGER_ROWS.map(([id]) => id);
  required(JSON.stringify(ledger.records.map(({ id }) => id)) === JSON.stringify(expectedIds),
    "wrapper command ledger row order differs");
  required(typeof hostQuotaTarget === "string", "host filesystem quota target is missing");
  const commands = expectedArgv({ work, checkout, source, imagePath, loopPath, mountPath,
    proofScript, output, receiptHelper, inventoryPath, uuid, fragmentSize, majorMinor,
    hostQuotaTarget, operations });
  let previousEnd = 0n;
  const callerRows = new Set([
    "image-fallocate", "image-sync", "image.filefrag", "clone", "inner-proof", "export",
  ]);
  const nonzeroRows = new Set(["fuser", "loop-fuser"]);
  for (let index = 0; index < ledger.records.length; index += 1) {
    const record = ledger.records[index];
    const [id, stdoutName, stderrName] = COMMAND_LEDGER_ROWS[index];
    const started = decimal(record.started_ns, `wrapper command ledger ${id} start`);
    const ended = decimal(record.ended_ns, `wrapper command ledger ${id} end`);
    required(started >= previousEnd, `wrapper command ledger ${id} overlaps or precedes its prior row`);
    previousEnd = ended;
    required(record.stdout_path === join(work, stdoutName) && record.stderr_path === join(work, stderrName),
      `wrapper command ledger ${id} stream paths differ`);
    required(record.cwd === (id === "inner-proof" ? checkout : work),
      `wrapper command ledger ${id} cwd differs`);
    const expectedUid = callerRows.has(id) ? callerUid : 0;
    const expectedGid = callerRows.has(id) ? callerGid : 0;
    required(record.uid === expectedUid && record.gid === expectedGid,
      `wrapper command ledger ${id} identity differs`);
    required(record.status === (nonzeroRows.has(id) ? 1 : 0),
      `wrapper command ledger ${id} status differs`);
    required(Object.prototype.hasOwnProperty.call(commands, id),
      `wrapper command ledger ${id} command specification is missing`);
    required(JSON.stringify(record.argv) === JSON.stringify(commands[id]),
      `wrapper command ledger ${id} argv differs from the exact wrapper command`);
  }
  const operationRows = {
    fallocate: "image-fallocate", image_sync: "image-sync", loop_attach: "loop-attach",
    mkfs_xfs: "mkfs-xfs", mount: "mount", proof: "inner-proof", export: "export",
    sync_before_umount: "sync-before-umount", umount: "umount", loop_detach: "loop-detach",
  };
  for (const [operationName, rowId] of Object.entries(operationRows)) {
    const operation = operations?.[operationName];
    required(operation && typeof operation === "object" && !Array.isArray(operation),
      `operation ${operationName} is missing`);
    const record = ledger.records.find(({ id }) => id === rowId);
    required(record.cwd === operation.cwd && record.status === operation.status &&
      record.stdout_path === operation.stdout_path && record.stderr_path === operation.stderr_path,
    `wrapper command ledger ${rowId} differs from operation facts`);
    const expectedOperationArgv = operation.argv;
    required(JSON.stringify(expectedOperationArgv) === JSON.stringify(
      operationName === "proof" ? record.argv.slice(-expectedOperationArgv.length) : record.argv),
    `wrapper command ledger ${rowId} argv differs from operation facts`);
  }
  required(invocationStartNs !== undefined && invocationEndNs !== undefined,
    "wrapper command ledger invocation timestamps are missing");
  const proofRecord = ledger.records.find(({ id }) => id === "inner-proof");
  required(proofRecord.started_ns === String(invocationStartNs) && proofRecord.ended_ns === String(invocationEndNs),
    "wrapper command ledger inner-proof timestamps differ from invocation");
  const execution = validateExecutionLedger(executionPath, ledger, work);
  return Object.freeze({
    path: ledger.path, bytes: ledger.bytes, sha256: ledger.sha256,
    record_count: ledger.records.length, records: ledger.records, execution,
  });
};

const MONITOR_FIELDS = Object.freeze([
  "clean", "mountpoint", "proof_loop_device", "new_loop_devices", "mount_namespace", "evidence",
]);
const MONITOR_EVIDENCE_FIELDS = Object.freeze([
  "before_mount", "after_mount", "before_mount_stderr", "after_mount_stderr",
  "before_mount_status", "after_mount_status", "before_loops", "after_loops",
  "before_loops_status", "after_loops_status",
  "before_loops_stderr", "after_loops_stderr", "mount_namespace_before", "mount_namespace_after",
]);
const MONITOR_STATUS_FIELDS = new Set([
  "before_mount_status", "after_mount_status", "before_loops_status", "after_loops_status",
]);

const exactFields = (value, fields, label) => {
  required(value && typeof value === "object" && !Array.isArray(value), `${label} is not an object`);
  required(JSON.stringify(Object.keys(value).sort()) === JSON.stringify([...fields].sort()), `${label} fields differ`);
};

const canonicalMonitorPath = (value, label) => {
  required(typeof value === "string" && isAbsolute(value) && resolve(value) === value && SAFE_TEXT.test(value),
    `${label} is not one canonical absolute path`);
  return value;
};

const digestMonitorFile = (path, label) => digestRegular(canonicalMonitorPath(path, label), label);

const readMonitorText = (path, label, { allowEmpty = true } = {}) => {
  const evidence = readRegularEvidence(canonicalMonitorPath(path, label), label);
  const { content, ...digest } = evidence;
  const text = content.toString("utf8");
  required(!text.includes("\0") && !text.includes("\r") &&
    (allowEmpty ? text === "" || text.endsWith("\n") : text.endsWith("\n")),
  `${label} is not canonical text`);
  return Object.freeze({ digest, text });
};

const readMonitorStatus = (path, label) => {
  const value = readMonitorText(path, label, { allowEmpty: false });
  required(value.text === "0\n" || value.text === "1\n", `${label} is not one canonical status`);
  return Object.freeze({ ...value, status: Number(value.text.slice(0, -1)) });
};

const readMonitorJson = (path, label) => {
  const value = readMonitorText(path, label, { allowEmpty: false });
  let parsed;
  try { parsed = JSON.parse(value.text); }
  catch (error) { fail(`${label} is invalid JSON: ${error.message}`); }
  return Object.freeze({ ...value, value: parsed });
};

const readMonitorNamespace = (path, label) => {
  const value = readMonitorText(path, label, { allowEmpty: false });
  required(/^mnt:\[[0-9]+\]\n$/.test(value.text), `${label} is not one mount namespace identity`);
  return Object.freeze({ ...value, value: value.text.slice(0, -1) });
};

const monitorEvidence = (paths) => {
  required(paths && typeof paths === "object" && !Array.isArray(paths), "host monitor evidence paths are missing");
  exactFields(paths, MONITOR_EVIDENCE_FIELDS, "host monitor evidence paths");
  const beforeMount = readMonitorText(paths.before_mount, "host monitor before mount");
  const afterMount = readMonitorText(paths.after_mount, "host monitor after mount");
  const beforeMountStderr = readMonitorText(paths.before_mount_stderr, "host monitor before mount stderr");
  const afterMountStderr = readMonitorText(paths.after_mount_stderr, "host monitor after mount stderr");
  const beforeMountStatus = readMonitorStatus(paths.before_mount_status, "host monitor before mount status");
  const afterMountStatus = readMonitorStatus(paths.after_mount_status, "host monitor after mount status");
  const beforeLoops = readMonitorJson(paths.before_loops, "host monitor before loops");
  const afterLoops = readMonitorJson(paths.after_loops, "host monitor after loops");
  const beforeLoopsStatus = readMonitorStatus(paths.before_loops_status, "host monitor before loops status");
  const afterLoopsStatus = readMonitorStatus(paths.after_loops_status, "host monitor after loops status");
  const beforeLoopsStderr = readMonitorText(paths.before_loops_stderr, "host monitor before loops stderr");
  const afterLoopsStderr = readMonitorText(paths.after_loops_stderr, "host monitor after loops stderr");
  const mountNamespaceBefore = readMonitorNamespace(paths.mount_namespace_before, "host monitor before mount namespace");
  const mountNamespaceAfter = readMonitorNamespace(paths.mount_namespace_after, "host monitor after mount namespace");
  return Object.freeze({
    values: Object.freeze({
      beforeMount: beforeMount.text, afterMount: afterMount.text,
      beforeMountStderr: beforeMountStderr.text, afterMountStderr: afterMountStderr.text,
      beforeMountStatus: beforeMountStatus.status, afterMountStatus: afterMountStatus.status,
      beforeLoops: beforeLoops.value, afterLoops: afterLoops.value,
      beforeLoopsStatus: beforeLoopsStatus.status, afterLoopsStatus: afterLoopsStatus.status,
      beforeLoopsStderr: beforeLoopsStderr.text, afterLoopsStderr: afterLoopsStderr.text,
      mountNamespaceBefore: mountNamespaceBefore.value, mountNamespaceAfter: mountNamespaceAfter.value,
    }),
    evidence: Object.freeze({
      before_mount: beforeMount.digest,
      after_mount: afterMount.digest,
      before_mount_stderr: beforeMountStderr.digest,
      after_mount_stderr: afterMountStderr.digest,
      before_mount_status: Object.freeze({ ...beforeMountStatus.digest, status: beforeMountStatus.status }),
      after_mount_status: Object.freeze({ ...afterMountStatus.digest, status: afterMountStatus.status }),
      before_loops: beforeLoops.digest,
      after_loops: afterLoops.digest,
      before_loops_status: Object.freeze({ ...beforeLoopsStatus.digest, status: beforeLoopsStatus.status }),
      after_loops_status: Object.freeze({ ...afterLoopsStatus.digest, status: afterLoopsStatus.status }),
      before_loops_stderr: beforeLoopsStderr.digest,
      after_loops_stderr: afterLoopsStderr.digest,
      mount_namespace_before: mountNamespaceBefore.digest,
      mount_namespace_after: mountNamespaceAfter.digest,
    }),
  });
};

const loopName = (row) => row?.name ?? row?.NAME;
const loopBacking = (row) => row?.backing_file ?? row?.["back-file"] ?? row?.BACK_FILE ?? row?.["BACK-FILE"] ?? null;

const loopRows = (value, label) => {
  required(value && typeof value === "object" && !Array.isArray(value) && Array.isArray(value.loopdevices),
    `${label} has no loopdevices array`);
  return value.loopdevices;
};

const imageBacksLoop = (row, image) => {
  const backing = loopBacking(row);
  if (typeof backing !== "string") return false;
  try { return realpathSync.native(backing) === image; }
  catch { return resolve(backing) === image; }
};

export const validateHostMonitor = ({
  beforeMount = "", afterMount = "", beforeMountStderr = "", afterMountStderr = "",
  beforeMountStatus = 1, afterMountStatus = 1, beforeLoops, afterLoops,
  beforeLoopsStatus = 0, afterLoopsStatus = 0,
  beforeLoopsStderr = "", afterLoopsStderr = "", image, mountpoint,
  mountNamespaceBefore, mountNamespaceAfter, proofLoopDevice, evidencePaths,
} = {}) => {
  const raw = evidencePaths ? monitorEvidence(evidencePaths) : Object.freeze({
    values: Object.freeze({ beforeMount, afterMount, beforeMountStderr, afterMountStderr,
      beforeMountStatus, afterMountStatus, beforeLoops, afterLoops, beforeLoopsStderr,
      beforeLoopsStatus, afterLoopsStatus, afterLoopsStderr, mountNamespaceBefore, mountNamespaceAfter }),
    evidence: null,
  });
  const values = raw.values;
  required(values.beforeMountStatus === 1 && values.afterMountStatus === 1,
    "host mount probes did not report canonical absence");
  required(values.beforeLoopsStatus === 0 && values.afterLoopsStatus === 0,
    "host loop probes did not report success");
  required(values.beforeMountStderr === "" && values.afterMountStderr === "" &&
    values.beforeLoopsStderr === "" && values.afterLoopsStderr === "",
  "host monitor probes wrote stderr");
  required(typeof values.beforeMount === "string" && values.beforeMount === "",
    "mountpoint was already mounted before provisioning");
  required(typeof values.afterMount === "string" && values.afterMount === "",
    "proof mount remains in host namespace");
  required(typeof mountpoint === "string" && isAbsolute(mountpoint) && resolve(mountpoint) === mountpoint && SAFE_TEXT.test(mountpoint),
    "host monitor mountpoint is invalid");
  const before = loopRows(values.beforeLoops, "host monitor before loops");
  const after = loopRows(values.afterLoops, "host monitor after loops");
  const canonicalImage = canonicalRegular(image, "backing image") && image;
  required(typeof proofLoopDevice === "string" && /^\/dev\/loop[0-9]+$/.test(proofLoopDevice),
    "host monitor proof loop device is missing");
  for (const row of [...before, ...after]) required(/^\/dev\/loop[0-9]+$/.test(loopName(row) ?? ""),
    "host monitor loop identity is malformed");
  /* Only rows owned by this proof are a teardown failure.  Other processes may
   * attach or detach unrelated loops during a long host run. */
  /* A loop number is reusable immediately after detach.  Ownership therefore
   * comes from the canonical backing image, not from the proof's former loop
   * name.  Retain the name in the diagnostic list, but do not red on an
   * unrelated process reusing that number. */
  const proofOwned = [...new Set(after.filter((row) => imageBacksLoop(row, canonicalImage))
    .map((row) => loopName(row)))];
  required(proofOwned.length === 0, `host monitor observed surviving proof loop devices: ${proofOwned.join(",")}`);
  required(after.every((row) => !imageBacksLoop(row, canonicalImage)),
    "proof backing image remains attached in host namespace");
  required(typeof values.mountNamespaceBefore === "string" && typeof values.mountNamespaceAfter === "string" &&
    values.mountNamespaceBefore === values.mountNamespaceAfter,
  "host monitor changed mount namespace");
  const result = { clean: true, mountpoint, proof_loop_device: proofLoopDevice,
    new_loop_devices: proofOwned, mount_namespace: values.mountNamespaceAfter };
  if (raw.evidence !== null) result.evidence = raw.evidence;
  return Object.freeze(result);
};

const stableJson = (value) => {
  if (Array.isArray(value)) return `[${value.map(stableJson).join(",")}]`;
  if (value && typeof value === "object") return `{${Object.keys(value).sort().map((key) => `${JSON.stringify(key)}:${stableJson(value[key])}`).join(",")}}`;
  return JSON.stringify(value);
};

const validateMonitorDigest = (record, label, { status = false } = {}) => {
  exactFields(record, status ? ["path", "bytes", "sha256", "status"] : ["path", "bytes", "sha256"], label);
  const evidence = readRegularEvidence(canonicalMonitorPath(record.path, label), label);
  const { content, ...actual } = evidence;
  required(actual.bytes === record.bytes && actual.sha256 === record.sha256, `${label} digest differs`);
  if (status) {
    const text = content.toString("utf8");
    required(text === `${record.status}\n` && (record.status === 0 || record.status === 1), `${label} status differs`);
  }
};

export const validateBoundHostMonitor = (monitor, {
  image, mountpoint, proofLoopDevice, expectedEvidencePaths,
} = {}) => {
  exactFields(monitor, MONITOR_FIELDS, "host monitor");
  exactFields(monitor.evidence, MONITOR_EVIDENCE_FIELDS, "host monitor evidence");
  for (const name of MONITOR_EVIDENCE_FIELDS) {
    validateMonitorDigest(monitor.evidence[name], `host monitor evidence ${name}`, { status: MONITOR_STATUS_FIELDS.has(name) });
    if (expectedEvidencePaths !== undefined) {
      required(monitor.evidence[name].path === expectedEvidencePaths[name],
        `host monitor evidence ${name} path differs`);
    }
  }
  const evidencePaths = Object.fromEntries(MONITOR_EVIDENCE_FIELDS.map((name) => [name, monitor.evidence[name].path]));
  const normalized = validateHostMonitor({ evidencePaths, image, mountpoint, proofLoopDevice });
  required(stableJson(monitor) === stableJson(normalized), "host monitor summary or evidence is not normalized");
  return normalized;
};
