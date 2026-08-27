import assert from "node:assert/strict";
import test from "node:test";

import { verifyProcessNamespaceSnapshot } from "./r2-complete-proof-process.mjs";

const currentPid = 100;
const currentNamespace = "net:[222]";
const hostNamespace = "net:[111]";
const rows = [
  { pid: 1, ppid: 0, uid: 0, namespace: null },
  { pid: 50, ppid: 1, uid: 0, namespace: currentNamespace },
  { pid: currentPid, ppid: 50, uid: 1001, namespace: currentNamespace },
  { pid: 200, ppid: 1, uid: 1001, namespace: hostNamespace },
];

const verify = (overrides = {}) => verifyProcessNamespaceSnapshot({
  currentPid, currentNamespace, hostNamespace, rows, ...overrides,
});

test("only non-ancestor processes in the fresh network namespace are leaks", () => {
  assert.deepEqual(verify(), {
    ancestors: [100, 50, 1],
    compared: 1,
    current_namespace: currentNamespace,
    host_namespace: hostNamespace,
  });
});

test("a token-cleared pathless same-namespace row fails closed", () => {
  const pathless = { pid: 201, ppid: 1, uid: 1001, namespace: currentNamespace };
  assert.throws(() => verify({ rows: [...rows, pathless] }), /same-network-namespace processes: 201/);
});

test("an unreadable current-uid namespace fails closed", () => {
  const unreadable = { pid: 202, ppid: 1, uid: 1001, namespace: null };
  assert.throws(() => verify({ rows: [...rows, unreadable] }), /could not compare current-uid process namespaces: 202/);
});

test("unchanged or malformed namespace identities are refused", () => {
  assert.throws(() => verify({ currentNamespace: hostNamespace }), /not fresh/);
  const malformed = rows.map((row) => row.pid === 200 ? { ...row, namespace: "net:not-an-inode" } : row);
  assert.throws(() => verify({ rows: malformed }), /snapshot row is invalid/);
});
