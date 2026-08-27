import { existsSync, readFileSync, readdirSync, readlinkSync } from "node:fs";
import { join } from "node:path";

const NETNS = /^net:\[\d+\]$/;
const fail = (message) => { throw new Error(`r2 process namespace audit: ${message}`); };
const required = (condition, message) => { if (!condition) fail(message); };

const statusInteger = (text, label, pid) => {
  const match = new RegExp(`^${label}:\\s+(\\d+)`, "m").exec(text);
  required(match, `process ${pid} has no ${label} status field`);
  return Number(match[1]);
};

export const verifyProcessNamespaceSnapshot = ({
  currentPid,
  currentNamespace,
  hostNamespace,
  rows,
}) => {
  required(Number.isInteger(currentPid) && currentPid > 0, "current PID is invalid");
  required(NETNS.test(currentNamespace), "current network namespace identity is invalid");
  required(NETNS.test(hostNamespace), "host network namespace identity is invalid");
  required(currentNamespace !== hostNamespace, "current network namespace is not fresh");
  required(Array.isArray(rows), "process snapshot is not an array");

  const byPid = new Map();
  for (const row of rows) {
    required(row && Number.isInteger(row.pid) && row.pid > 0
      && Number.isInteger(row.ppid) && row.ppid >= 0
      && Number.isInteger(row.uid) && row.uid >= 0
      && (row.namespace === null || NETNS.test(row.namespace)), "process snapshot row is invalid");
    required(!byPid.has(row.pid), `process snapshot repeats PID ${row.pid}`);
    byPid.set(row.pid, row);
  }
  const current = byPid.get(currentPid);
  required(current && current.namespace === currentNamespace, "current process namespace differs from the snapshot");

  const ancestors = new Set();
  let cursor = currentPid;
  while (cursor > 0) {
    required(!ancestors.has(cursor), "process ancestor chain contains a cycle");
    ancestors.add(cursor);
    const row = byPid.get(cursor);
    required(row, `process ancestor ${cursor} is absent from the snapshot`);
    cursor = row.ppid;
  }

  const leaks = [];
  const unreadable = [];
  let compared = 0;
  for (const row of rows) {
    if (ancestors.has(row.pid)) continue;
    if (row.namespace === null) {
      if (row.uid === current.uid) unreadable.push(row.pid);
      continue;
    }
    compared += 1;
    if (row.namespace === currentNamespace) leaks.push(row.pid);
  }
  required(unreadable.length === 0, `could not compare current-uid process namespaces: ${unreadable.join(",")}`);
  required(leaks.length === 0, `live same-network-namespace processes: ${leaks.join(",")}`);
  return {
    ancestors: [...ancestors],
    compared,
    current_namespace: currentNamespace,
    host_namespace: hostNamespace,
  };
};

const processRow = (procRoot, pid) => {
  const root = join(procRoot, String(pid));
  let status;
  try { status = readFileSync(join(root, "status"), "utf8"); }
  catch (error) {
    if (!existsSync(root)) return null;
    throw error;
  }
  let namespace = null;
  try { namespace = readlinkSync(join(root, "ns/net")); }
  catch (error) {
    if (!existsSync(root)) return null;
    if (!["EACCES", "ENOENT", "EPERM", "ESRCH"].includes(error.code)) throw error;
  }
  return {
    namespace,
    pid,
    ppid: statusInteger(status, "PPid", pid),
    uid: statusInteger(status, "Uid", pid),
  };
};

export const auditLiveProcessNamespace = ({
  procRoot = "/proc",
  currentPid = process.pid,
  hostNamespace = process.env.NOMOS_R2_HOST_NETNS,
} = {}) => {
  const currentNamespace = readlinkSync(join(procRoot, String(currentPid), "ns/net"));
  const pids = readdirSync(procRoot).filter((name) => /^[1-9]\d*$/.test(name)).map(Number);
  const rows = pids.map((pid) => processRow(procRoot, pid)).filter(Boolean);
  return verifyProcessNamespaceSnapshot({ currentPid, currentNamespace, hostNamespace, rows });
};
