import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import { mkdirSync, mkdtempSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import test from "node:test";

import {
  parseHostFindmnt,
  parseHostStatfs,
  validateFilefragEvidence,
  validateHostFilesystemEvidence,
  validateXfsInfoEvidence,
} from "./r2-complete-proof-xfs-evidence.mjs";

const temporary = () => mkdtempSync(join(tmpdir(), "nomos-r2-xfs-evidence-test-"));

test("host evidence parser binds canonical statfs, mount, quota, and empty stderr", () => {
  const root = temporary();
  try {
    const prefix = join(root, "host-before");
    const statfsText = "f_frsize=4096 f_blocks=100 f_bfree=90 f_bavail=80 f_type=0x58465342 f_fsid=123\n";
    writeFileSync(`${prefix}.statfs`, statfsText);
    writeFileSync(`${prefix}.findmnt`, `${root} /dev/loop0 xfs rw,nodev,nosuid,noquota private\n`);
    writeFileSync(`${prefix}.quota`, "");
    writeFileSync(`${prefix}.statfs.status`, "0\n");
    writeFileSync(`${prefix}.findmnt.status`, "0\n");
    writeFileSync(`${prefix}.quota.status`, "0\n");
    for (const name of ["statfs", "findmnt", "quota"]) writeFileSync(`${prefix}.${name}.stderr`, "");
    const parsedStatfs = parseHostStatfs(prefix, "host");
    assert.equal(parsedStatfs.facts.free_blocks, "90");
    assert.equal(parsedStatfs.statfs.sha256, createHash("sha256").update(statfsText).digest("hex"));
    assert.equal(parseHostFindmnt(prefix, "host").fields[4], "private");
    const evidence = validateHostFilesystemEvidence(prefix, "host");
    assert.equal(evidence.quota.text, "");
    assert.equal(evidence.statfs_status.text, "0\n");
    assert.equal(evidence.findmnt_status.text, "0\n");
    assert.equal(evidence.statfs_status.sha256.length, 64);
    writeFileSync(`${prefix}.findmnt`, `${JSON.stringify({ filesystems: [{ target: root, source: "/dev/loop0", fstype: "xfs", options: "rw,nodev,nosuid,noquota", propagation: "private" }] })}\n`);
    assert.equal(parseHostFindmnt(prefix, "host").fields[0], root);
    writeFileSync(`${prefix}.statfs`, "f_frsize=4096 f_blocks=100 f_bfree=90 f_bavail=91 f_type=0x58465342 f_fsid=123\n");
    assert.throws(() => parseHostStatfs(prefix, "host"), /canonical|inconsistent/);
  } finally {
    rmSync(root, { recursive: true, force: true });
  }
});

test("XFS info and filefrag evidence require the decisive lines", () => {
  const root = temporary();
  try {
    const info = join(root, "xfs-info");
    const frag = join(root, "filefrag");
    writeFileSync(info, "meta-data=/dev/loop0\nlog =internal\nrealtime =none\n");
    writeFileSync(frag, "Filesystem type is: 0x58465342\nFile size of x is 8 (1 block)\n ext: logical_offset: physical_offset: length: expected: flags:\n   0:        0..       0:        1..       1: last,eof\n");
    assert.equal(validateXfsInfoEvidence(info).path, info);
    assert.equal(validateFilefragEvidence(frag).path, frag);
    writeFileSync(info, "log =external\nrealtime =none\n");
    assert.throws(() => validateXfsInfoEvidence(info), /internal log/);
    writeFileSync(frag, "Filesystem type is: 0x58465342\nFile size of x is 8\n");
    assert.throws(() => validateFilefragEvidence(frag), /incomplete/);
  } finally {
    rmSync(root, { recursive: true, force: true });
  }
});

test("host evidence rejects command stderr and quota failure", () => {
  const root = temporary();
  try {
    const prefix = join(root, "host");
    writeFileSync(`${prefix}.statfs`, "f_frsize=4096 f_blocks=100 f_bfree=90 f_bavail=80 f_type=xfs f_fsid=123\n");
    writeFileSync(`${prefix}.findmnt`, `${root} /dev/loop0 xfs rw,nodev,nosuid,noquota private\n`);
    writeFileSync(`${prefix}.quota`, "");
    writeFileSync(`${prefix}.statfs.status`, "0\n");
    writeFileSync(`${prefix}.findmnt.status`, "0\n");
    writeFileSync(`${prefix}.quota.status`, "1\n");
    for (const name of ["statfs", "findmnt", "quota"]) writeFileSync(`${prefix}.${name}.stderr`, "");
    assert.throws(() => validateHostFilesystemEvidence(prefix, "host"), /quota/);
    writeFileSync(`${prefix}.quota.status`, "0\n");
    writeFileSync(`${prefix}.statfs.status`, "1\n");
    assert.throws(() => validateHostFilesystemEvidence(prefix, "host"), /statfs/);
    writeFileSync(`${prefix}.statfs.status`, "0\n");
    writeFileSync(`${prefix}.findmnt.stderr`, "failure\n");
    assert.throws(() => validateHostFilesystemEvidence(prefix, "host"), /stderr/);
  } finally {
    rmSync(root, { recursive: true, force: true });
  }
});
