/*
 * Evidence binding for the XFS wrapper receipt.  This boundary is separate
 * from receipt assembly so host-side command output, tool identity, and
 * operation logs cannot be silently forwarded as unexamined paths.
 */
import { readFileSync } from "node:fs";
import { dirname, isAbsolute, join } from "node:path";
import { fileURLToPath } from "node:url";

import {
  IMAGE_BYTES,
  canonicalHex,
  canonicalPath,
  canonicalRegular,
  canonicalStringPath,
  digestRegular,
  executablePath,
  exactFields,
  objectValue,
  readableRealpath,
  requiredField,
  safeText,
  statusCode,
  XfsWrapperError,
} from "./r2-complete-proof-xfs-receipt.mjs";

export const TOOL_NAMES = Object.freeze(
  "bash git realpath find stat date mkdir readlink id node jq sudo unshare ip findmnt losetup mount umount blockdev mkfs.xfs xfs_info xfs_quota filefrag fuser fallocate sync setpriv bwrap tar ionice du blkid chown cp rm env sha256sum rustup cargo rustc".split(" "),
);

const fail = (message) => { throw new XfsWrapperError(message); };

const evidenceText = (path, label, { nonempty = true } = {}) => {
  const canonical = canonicalRegular(path, label);
  const text = readFileSync(canonical, "utf8");
  if (text.includes("\0") || text.includes("\r") || (nonempty && text.length === 0) ||
      (text.length > 0 && !text.endsWith("\n"))) fail(`${label} is not canonical text`);
  return Object.freeze({ ...digestRegular(canonical, label), text });
};

export const parseHostStatfs = (prefix, label) => {
  const evidence = evidenceText(`${prefix}.statfs`, `${label} statfs`);
  const match = /^f_frsize=(\d+) f_blocks=(\d+) f_bfree=(\d+) f_bavail=(\d+) f_type=([^\t\r\n ]+) f_fsid=([^\t\r\n ]+)\n$/.exec(evidence.text);
  if (!match) fail(`${label} statfs evidence is not canonical`);
  const [, fragmentSize, blocks, freeBlocks, availableBlocks, filesystemType, filesystemId] = match;
  for (const [value, name] of [[fragmentSize, "fragment size"], [blocks, "blocks"], [freeBlocks, "free blocks"], [availableBlocks, "available blocks"]]) {
    if (!/^(0|[1-9][0-9]*)$/.test(value)) fail(`${label} ${name} is not canonical`);
  }
  if (BigInt(fragmentSize) === 0n || BigInt(availableBlocks) > BigInt(freeBlocks) || BigInt(freeBlocks) > BigInt(blocks)) fail(`${label} statfs counters are inconsistent`);
  safeText(filesystemType, `${label} filesystem type`);
  safeText(filesystemId, `${label} filesystem id`);
  return Object.freeze({
    facts: Object.freeze({ fragment_size: fragmentSize, blocks, free_blocks: freeBlocks, available_blocks: availableBlocks, filesystem_type: filesystemType, filesystem_id: filesystemId }),
    statfs: evidence,
  });
};

export const parseHostFindmnt = (prefix, label) => {
  const evidence = evidenceText(`${prefix}.findmnt`, `${label} mount`);
  const fields = evidence.text.slice(0, -1).split(/\s+/);
  if (fields.length !== 5 || !isAbsolute(fields[0]) || !fields.every((field) => SAFE_TEXT.test(field))) fail(`${label} mount evidence is not canonical`);
  return Object.freeze({ fields: Object.freeze(fields), mount: evidence });
};

const SAFE_TEXT = /^[^\0\r\n\t]+$/;

export const validateHostFilesystemEvidence = (prefix, label) => {
  const canonicalPrefix = canonicalPath(prefix, label, { mustExist: false });
  const statfs = parseHostStatfs(canonicalPrefix, label);
  const mount = parseHostFindmnt(canonicalPrefix, label);
  const quota = evidenceText(`${canonicalPrefix}.quota`, `${label} quota`, { nonempty: false });
  const quotaStatus = evidenceText(`${canonicalPrefix}.quota.status`, `${label} quota status`);
  if (quotaStatus.text !== "0\n") fail(`${label} quota command did not pass`);
  const stderr = {};
  for (const name of ["statfs", "findmnt", "quota"]) {
    const value = evidenceText(`${canonicalPrefix}.${name}.stderr`, `${label} ${name} stderr`, { nonempty: false });
    if (value.text !== "") fail(`${label} ${name} wrote stderr`);
    stderr[name] = value;
  }
  return Object.freeze({ prefix: canonicalPrefix, statfs, mount, quota, quota_status: quotaStatus, stderr: Object.freeze(stderr) });
};

export const validateXfsInfoEvidence = (path) => {
  const evidence = evidenceText(path, "XFS info");
  if (!/^\s*log\s*=\s*internal(?:\s|$)/m.test(evidence.text) || !/^\s*realtime\s*=\s*none(?:\s|$)/m.test(evidence.text)) fail("XFS info does not prove internal log and no realtime device");
  return evidence;
};

export const validateFilefragEvidence = (path) => {
  const evidence = evidenceText(path, "filefrag");
  if (!/^Filesystem type is:/m.test(evidence.text) || !/^File size of /m.test(evidence.text) || !/^\s*ext:\s*logical_offset:/m.test(evidence.text) || !/^\s*\d+:\s*\S+/m.test(evidence.text)) fail("filefrag evidence is incomplete");
  return evidence;
};

export const validateEvidenceManifest = (path, inventory) => {
  const evidence = evidenceText(path, "inner evidence manifest");
  const lines = (evidence.text === "\n" ? [] : evidence.text.slice(0, -1).split("\n")).map((line) => {
    const match = /^([0-9a-f]{64})  (.+)$/.exec(line);
    if (!match || match[2].startsWith("/") || match[2].split("/").some((part) => part === "" || part === "." || part === "..")) fail("inner evidence manifest row is unsafe");
    return { sha256: match[1], path: match[2] };
  });
  const expected = inventory.rows.filter((row) => !["EVIDENCE.sha256", "receipt.json"].includes(row.path)).map((row) => ({ sha256: row.sha256, path: row.path }));
  const sorted = [...lines].sort((left, right) => Buffer.from(left.path).compare(Buffer.from(right.path)));
  if (JSON.stringify(lines) !== JSON.stringify(sorted) || JSON.stringify(lines) !== JSON.stringify(expected)) fail("inner evidence manifest does not bind exported output");
  return Object.freeze({ ...digestRegular(path, "inner evidence manifest"), files: lines.length });
};

const operationRecord = (value, name) => {
  const operation = objectValue(requiredField(value, name, `operation ${name}`), `operation ${name}`);
  exactFields(operation, ["argv", "cwd", "status", "stdout_path", "stderr_path"], `operation ${name}`);
  if (!Array.isArray(operation.argv) || operation.argv.length === 0 || operation.argv.some((argument) => typeof argument !== "string" || !SAFE_TEXT.test(argument))) fail(`operation ${name} argv is invalid`);
  canonicalStringPath(operation.cwd, `operation ${name} cwd`);
  if (statusCode(operation.status, `operation ${name} status`) !== 0) fail(`operation ${name} did not pass`);
  return Object.freeze({ operation, stdout: digestRegular(operation.stdout_path, `operation ${name} stdout`), stderr: digestRegular(operation.stderr_path, `operation ${name} stderr`) });
};

export const validateOperationsAndTools = (value, { imagePath, work, mountPath, checkout, output, loopPath, exportDestination, inventoryPath }) => {
  const operationNames = ["fallocate", "image_sync", "loop_attach", "mkfs_xfs", "mount", "proof", "export", "sync_before_umount", "umount", "loop_detach"];
  exactFields(value.operations, operationNames, "operations");
  const helperPath = fileURLToPath(new URL("./r2-complete-proof-xfs-receipt.mjs", import.meta.url));
  const expected = {
    fallocate: ["/usr/bin/fallocate", "--posix", "--length", IMAGE_BYTES.toString(), imagePath],
    image_sync: ["/usr/bin/sync", "-f", imagePath],
    loop_attach: ["/usr/sbin/losetup", "--find", "--show", imagePath],
    mkfs_xfs: ["/usr/sbin/mkfs.xfs", "-f", "-l", "internal", loopPath],
    mount: ["/usr/bin/mount", "-t", "xfs", "-o", "rw,nodev,nosuid", loopPath, mountPath],
    proof: ["/usr/bin/bash", join(checkout, "docs", "evaluation", "r2-complete-proof.sh"), "--output", output],
    export: ["/usr/bin/node", helperPath, "copy", "--source", output, "--destination", exportDestination, "--output", inventoryPath],
    sync_before_umount: ["/usr/bin/sync", "-f", mountPath],
    umount: ["/usr/bin/umount", mountPath],
    loop_detach: ["/usr/sbin/losetup", "--detach", loopPath],
  };
  const records = {};
  for (const name of operationNames) {
    const record = operationRecord(value.operations, name);
    if (JSON.stringify(record.operation.argv) !== JSON.stringify(expected[name])) fail(`operation ${name} argv differs from the exact wrapper command`);
    if (record.operation.cwd !== (name === "proof" ? checkout : work)) fail(`operation ${name} cwd differs from the wrapper root`);
    records[name] = Object.freeze({ argv: Object.freeze([...record.operation.argv]), cwd: record.operation.cwd, status: record.operation.status, stdout: record.stdout, stderr: record.stderr });
  }
  const tools = objectValue(requiredField(value, "tool_register"), "tool register");
  exactFields(tools, TOOL_NAMES, "tool register");
  const commandToolPaths = Object.fromEntries(operationNames.map((name) => [expected[name][0].split("/").pop(), expected[name][0]]));
  const normalizedTools = {};
  for (const name of TOOL_NAMES) {
    const tool = objectValue(tools[name], `tool register ${name}`);
    exactFields(tool, ["path", "version_argv", "version_status", "sha256", "version"], `tool register ${name}`);
    const toolPath = executablePath(tool.path, `tool register ${name} path`);
    if (commandToolPaths[name] !== undefined && readableRealpath(toolPath, `tool register ${name} path`) !== readableRealpath(commandToolPaths[name], `wrapper ${name} command`)) fail(`tool register ${name} path differs`);
    safeText(tool.version_argv, `tool register ${name} version argv`);
    if (statusCode(tool.version_status, `tool register ${name} version status`) !== 0) fail(`tool register ${name} version command did not pass`);
    canonicalHex(tool.sha256, 64, `tool register ${name} sha256`);
    if (digestRegular(toolPath, `tool register ${name}`).sha256 !== tool.sha256) fail(`tool register ${name} digest differs`);
    safeText(tool.version, `tool register ${name} version`);
    normalizedTools[name] = Object.freeze({ path: toolPath, version_argv: tool.version_argv, version_status: tool.version_status, sha256: tool.sha256, version: tool.version });
  }
  return Object.freeze({ operations: Object.freeze(records), tool_register: Object.freeze(normalizedTools) });
};
