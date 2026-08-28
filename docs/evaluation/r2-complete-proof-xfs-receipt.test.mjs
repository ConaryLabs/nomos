import assert from "node:assert/strict";
import { mkdtempSync, mkdirSync, readFileSync, rmSync, symlinkSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import test from "node:test";

import {
  IMAGE_BYTES,
  SCHEMA,
  TOP_LEVEL_FIELDS,
  XfsWrapperError,
  assembleReceipt,
  assertEmptyDirectory,
  assertNonOverlappingPaths,
  canonicalInventory,
  canonicalInventoryText,
  canonicalPath,
  compareInventories,
  copyTreeNoDeref,
  digestRegular,
  inventoryDigest,
  parseDuOutput,
  validateDuAgainstStatfs,
  validateHostMonitor,
  validateReservationDelta,
  validateStatfsSnapshot,
} from "./r2-complete-proof-xfs-receipt.mjs";

const temporary = () => mkdtempSync(join(tmpdir(), "nomos-r2-xfs-test-"));
const expectFailure = (fn, pattern = XfsWrapperError) => assert.throws(fn, pattern);

test("canonical paths and empty work validation reject overlap and symlinks", () => {
  const root = temporary();
  try {
    const source = join(root, "source");
    const work = join(root, "work");
    mkdirSync(source);
    mkdirSync(work);
    assert.deepEqual(assertNonOverlappingPaths(source, work), { source, work });
    assert.equal(assertEmptyDirectory(work), work);
    expectFailure(() => assertNonOverlappingPaths(source, source));
    const alias = join(root, "alias");
    symlinkSync(source, alias);
    expectFailure(() => canonicalPath(alias, "alias"), /symlink/);
    writeFileSync(join(source, "not-empty"), "x\n");
    expectFailure(() => assertEmptyDirectory(source), /not empty|work/);
  } finally {
    rmSync(root, { recursive: true, force: true });
  }
});

test("inventory, no-dereference copy, and digest are byte stable", () => {
  const root = temporary();
  try {
    const source = join(root, "source");
    const parent = join(root, "export");
    mkdirSync(join(source, "nested"), { recursive: true });
    mkdirSync(parent);
    writeFileSync(join(source, "a.txt"), "alpha\n");
    writeFileSync(join(source, "nested", "b.txt"), "beta\n");
    const before = canonicalInventory(source);
    const destination = join(parent, "copy");
    const after = copyTreeNoDeref(source, destination);
    assert.equal(compareInventories(before, after).equal, true);
    assert.equal(inventoryDigest(before.rows), before.sha256);
    assert.equal(canonicalInventoryText(before.rows), before.text);
    const linked = join(source, "bad");
    symlinkSync("a.txt", linked);
    expectFailure(() => canonicalInventory(source), /symlink/);
  } finally {
    rmSync(root, { recursive: true, force: true });
  }
});

test("du parser and accounting checks require one exact canonical row", () => {
  const root = temporary();
  try {
    const checkout = join(root, "checkout");
    mkdirSync(checkout);
    const parsed = parseDuOutput(`12\t${checkout}\n`, checkout, "", 0);
    assert.equal(parsed.mib, 12n);
    validateDuAgainstStatfs(parsed, { usedBytes: 12n * 1_048_576n });
    expectFailure(() => parseDuOutput(`012\t${checkout}\n`, checkout), /canonical row/);
    expectFailure(() => parseDuOutput(`12\t${checkout}\nextra\n`, checkout), /canonical row/);
    expectFailure(() => validateDuAgainstStatfs({ mib: 13n }, { usedBytes: 12n * 1_048_576n }), /exceeds/);
  } finally {
    rmSync(root, { recursive: true, force: true });
  }
});

test("statfs and reservation arithmetic preserve exact byte units", () => {
  const snapshot = validateStatfsSnapshot({
    f_type: "1481003842",
    f_bsize: "4096",
    f_blocks: "2000000",
    f_bfree: "1990000",
    f_bavail: "1985000",
  }, { fragmentSize: "4096" });
  assert.equal(snapshot.capacity_bytes, "8192000000");
  assert.equal(snapshot.allocated_bytes, "40960000");
  assert.equal(snapshot.allocated_mib, "40");
  expectFailure(() => validateStatfsSnapshot({ ...snapshot, f_bsize: "512" }, { fragmentSize: "4096" }), /differs/);
  assert.equal(validateReservationDelta({ before: "100", after: "40", allocated: "60" }), true);
  expectFailure(() => validateReservationDelta({ before: "59", after: "0", allocated: "60" }), /smaller/);
  expectFailure(() => validateReservationDelta({ before: "100", after: "41", allocated: "60" }), /reduce/);
  assert.equal(IMAGE_BYTES, 8_589_934_592n);
});

test("host monitor rejects surviving image or proof-owned loop", () => {
  const root = temporary();
  try {
    const image = join(root, "filesystem.xfs");
    writeFileSync(image, "image");
    const clean = validateHostMonitor({
      beforeMount: "",
      afterMount: "",
      beforeLoops: { loopdevices: [] },
      afterLoops: { loopdevices: [] },
      image,
      mountpoint: join(root, "fs"),
      mountNamespaceBefore: "mnt:[1]",
      mountNamespaceAfter: "mnt:[1]",
      proofLoopDevice: "/dev/loop9",
    });
    assert.equal(clean.clean, true);
    expectFailure(() => validateHostMonitor({
      beforeMount: "",
      afterMount: "",
      beforeLoops: { loopdevices: [] },
      afterLoops: { loopdevices: [{ name: "/dev/loop9", backing_file: image }] },
      image,
      mountpoint: join(root, "fs"),
      mountNamespaceBefore: "mnt:[1]",
      mountNamespaceAfter: "mnt:[1]",
      proofLoopDevice: "/dev/loop9",
    }), /backing image|proof loop|new or changed loop/);
    expectFailure(() => validateHostMonitor({
      beforeMount: "",
      afterMount: "",
      beforeLoops: { loopdevices: [] },
      afterLoops: { loopdevices: [{ name: "/dev/loop10", backing_file: "/dev/other" }] },
      image,
      mountpoint: join(root, "fs"),
      mountNamespaceBefore: "mnt:[1]",
      mountNamespaceAfter: "mnt:[1]",
      proofLoopDevice: "/dev/loop9",
    }), /new or changed loop/);
  } finally {
    rmSync(root, { recursive: true, force: true });
  }
});

test("receipt has exact top-level schema and success requires inner plus teardown", () => {
  const root = temporary();
  try {
    const work = join(root, "work");
    const source = join(root, "source");
    const fs = join(work, "fs");
    const checkout = join(fs, "checkout");
    const output = join(checkout, "target", "r2-complete-proof");
    const exportDestination = join(work, "export", "target", "r2-complete-proof");
    const image = join(work, "filesystem.xfs");
    const stdout = join(work, "proof.stdout");
    const stderr = join(work, "proof.stderr");
    const evidence = join(output, "EVIDENCE.sha256");
    mkdirSync(source);
    mkdirSync(join(checkout, "docs", "evaluation"), { recursive: true });
    mkdirSync(output, { recursive: true });
    mkdirSync(exportDestination, { recursive: true });
    mkdirSync(fs, { recursive: true });
    writeFileSync(join(checkout, "docs", "evaluation", "r2-complete-proof.sh"), "#!/usr/bin/env bash\n");
    writeFileSync(image, "image\n");
    writeFileSync(stdout, "proof output\n");
    writeFileSync(stderr, "");
    writeFileSync(join(output, "result.txt"), "result\n");
    writeFileSync(join(exportDestination, "result.txt"), "result\n");
    const resultDigest = digestRegular(join(output, "result.txt"), "result").sha256;
    writeFileSync(evidence, `${resultDigest}  result.txt\n`);
    writeFileSync(join(exportDestination, "EVIDENCE.sha256"), `${resultDigest}  result.txt\n`);
    for (const path of [
      join(work, "image-fallocate.stdout"), join(work, "image-fallocate.stderr"),
      join(work, "image-sync.stdout"), join(work, "image-sync.stderr"),
    ]) writeFileSync(path, "");
    writeFileSync(join(work, "image.filefrag"), "Filesystem type is: 0x58465342\nFile size of test is 8589934592 (2097152 blocks of 4096 bytes)\n ext: logical_offset: physical_offset: length: expected: flags:\n   0:        0..       0:        1..       1: last,eof\n");
    writeFileSync(join(work, "xfs-info.txt"), "log = internal\nrealtime = none\n");
    writeFileSync(join(work, "image.stat"), `logical_bytes=${IMAGE_BYTES}\nst_blocks=16777216\nallocated_bytes=${IMAGE_BYTES}\nblock_size=512\n`);
    const statfs = { f_type: "1481003842", f_bsize: "4096", f_blocks: "2000000", f_bfree: "1990000", f_bavail: "1985000" };
    for (const name of ["statfs-mounted.json", "statfs-checkout.json", "statfs-close.json"]) writeFileSync(join(work, name), `${JSON.stringify(statfs)}\n`);
    const hostStatfs = "f_frsize=4096 f_blocks=2000000 f_bfree=1990000 f_bavail=1985000 f_type=xfs f_fsid=1234\n";
    const hostFindmnt = `${work} /dev/test xfs rw,nodev,nosuid private\n`;
    for (const prefix of [join(work, "host-filesystem-before"), join(work, "host-filesystem-after")]) {
      writeFileSync(`${prefix}.statfs`, hostStatfs);
      writeFileSync(`${prefix}.findmnt`, hostFindmnt);
      writeFileSync(`${prefix}.quota`, "xfs quota state\n");
      writeFileSync(`${prefix}.quota.status`, "0\n");
      writeFileSync(`${prefix}.statfs.stderr`, "");
      writeFileSync(`${prefix}.findmnt.stderr`, "");
      writeFileSync(`${prefix}.quota.stderr`, "");
    }
    const sourceInventory = canonicalInventory(output);
    const exportInventory = canonicalInventory(exportDestination);
    const inventoryPath = join(work, "export.json");
    const inventoryDigestPath = join(work, "export.sha256");
    writeFileSync(inventoryPath, `${JSON.stringify({ source: output, destination: exportDestination, source_inventory_sha256: sourceInventory.sha256, export_inventory_sha256: exportInventory.sha256, rows: sourceInventory.rows.length, equal: true })}\n`);
    writeFileSync(inventoryDigestPath, `source\t${sourceInventory.sha256}\nexport\t${exportInventory.sha256}\n`);
    const toolNames = "bash git realpath find stat date mkdir readlink id node jq sudo unshare findmnt losetup mount umount blockdev mkfs.xfs xfs_info xfs_quota filefrag fuser fallocate sync setpriv bwrap tar ionice du blkid chown cp rm env sha256sum rustup cargo rustc".split(" ");
    const toolPaths = Object.fromEntries(toolNames.map((name) => [name, "/usr/bin/node"]));
    Object.assign(toolPaths, {
      bash: "/usr/bin/bash", fallocate: "/usr/bin/fallocate", sync: "/usr/lib/cargo/bin/coreutils/sync", losetup: "/usr/sbin/losetup",
      "mkfs.xfs": "/usr/sbin/mkfs.xfs", mount: "/usr/bin/mount", node: "/usr/bin/node", umount: "/usr/bin/umount",
    });
    const toolRegister = Object.fromEntries(Object.entries(toolPaths).map(([name, path]) => [name, {
      path, version_argv: "--version", version_status: 0, sha256: digestRegular(path, `${name} test tool`).sha256, version: `${name} test-version`,
    }]));
    const operationFiles = Object.fromEntries(["fallocate", "image_sync", "loop_attach", "mkfs_xfs", "mount", "proof", "export", "sync_before_umount", "umount", "loop_detach"].map((name) => [name, [join(work, `${name}.stdout`), join(work, `${name}.stderr`)]]));
    for (const [stdoutPath, stderrPath] of Object.values(operationFiles)) { writeFileSync(stdoutPath, ""); writeFileSync(stderrPath, ""); }
    const operation = (name, argv, cwd) => ({ argv, cwd, status: 0, stdout_path: operationFiles[name][0], stderr_path: operationFiles[name][1] });
    const operations = {
      fallocate: operation("fallocate", ["/usr/bin/fallocate", "--posix", "--length", IMAGE_BYTES.toString(), image], work),
      image_sync: operation("image_sync", ["/usr/bin/sync", "-f", image], work),
      loop_attach: operation("loop_attach", ["/usr/sbin/losetup", "--find", "--show", image], work),
      mkfs_xfs: operation("mkfs_xfs", ["/usr/sbin/mkfs.xfs", "-f", "-l", "internal", "/dev/loop9"], work),
      mount: operation("mount", ["/usr/bin/mount", "-t", "xfs", "-o", "rw,nodev,nosuid", "/dev/loop9", fs], work),
      proof: operation("proof", ["/usr/bin/bash", join(checkout, "docs", "evaluation", "r2-complete-proof.sh"), "--output", output], checkout),
      export: operation("export", ["/usr/bin/node", new URL("./r2-complete-proof-xfs-receipt.mjs", import.meta.url).pathname, "copy", "--source", output, "--destination", exportDestination, "--output", inventoryPath], work),
      sync_before_umount: operation("sync_before_umount", ["/usr/bin/sync", "-f", fs], work),
      umount: operation("umount", ["/usr/bin/umount", fs], work),
      loop_detach: operation("loop_detach", ["/usr/sbin/losetup", "--detach", "/dev/loop9"], work),
    };
    const facts = {
      setup_failed: false,
      inner_pass: true,
      candidate: { source, commit: "a".repeat(40), tree: "b".repeat(40), clean: true, source_status: 0 },
      image: { path: image, stat_path: join(work, "image.stat"), filefrag_path: join(work, "image.filefrag"), fallocate_stdout: join(work, "image-fallocate.stdout"), fallocate_stderr: join(work, "image-fallocate.stderr"), sync_stdout: join(work, "image-sync.stdout"), sync_stderr: join(work, "image-sync.stderr"), status: 0, sync_status: 0, logical_bytes: IMAGE_BYTES.toString(), allocated_bytes: IMAGE_BYTES.toString(), expected_bytes: IMAGE_BYTES.toString() },
      loop_device: { path: "/dev/loop9", major_minor: "7:9", size_bytes: IMAGE_BYTES.toString(), attached: false },
      filesystem: { type: "xfs", uuid: "11111111-2222-3333-4444-555555555555", fragment_size: "4096", capacity_limit_bytes: IMAGE_BYTES.toString(), capacity_ok: true, mounted_statfs_path: join(work, "statfs-mounted.json"), checkout_statfs_path: join(work, "statfs-checkout.json"), close_statfs_path: join(work, "statfs-close.json"), host_filesystem_before_path: join(work, "host-filesystem-before"), host_filesystem_after_path: join(work, "host-filesystem-after"), xfs_info_path: join(work, "xfs-info.txt") },
      mount: { path: fs, source: "/dev/loop9", options: "rw,nodev,nosuid,relatime", propagation: "private", status: 0, mounted: true, unmounted: true, mount_absent: true },
      invocation: {
        argv: ["/usr/bin/bash", join(checkout, "docs", "evaluation", "r2-complete-proof.sh"), "--output", output],
        cwd: checkout, uid: 1000, gid: 1000, user: "test", status: 0, inner_pass: true, start_ns: "10", end_ns: "20", stdout_path: stdout, stderr_path: stderr,
      },
      export: { source: output, destination: exportDestination, status: 0, equal: true, inventory_path: inventoryPath, inventory_digest_path: inventoryDigestPath, source_inventory_sha256: sourceInventory.sha256, export_inventory_sha256: exportInventory.sha256, inner_evidence_manifest_path: join(exportDestination, "EVIDENCE.sha256") },
      teardown: {
        unmounted: true, loop_detached: true, no_holder: true, mount_absent: true, image_unattached: true, fuser_status: 1, umount_status: 0, detach_status: 0, supervisor_status: 0,
        host_monitor: { clean: true, mountpoint: fs, proof_loop_device: "/dev/loop9", new_loop_devices: [], mount_namespace: "mnt:[1]" },
      },
      host_monitor: { clean: true, mountpoint: fs, proof_loop_device: "/dev/loop9", new_loop_devices: [], mount_namespace: "mnt:[1]" },
      operations,
      tool_register: toolRegister,
    };
    const receiptOptions = { statReader: () => ({ size: IMAGE_BYTES, blocks: 16_777_216n }) };
    const receipt = assembleReceipt(facts, receiptOptions);
    assert.deepEqual(Object.keys(receipt).sort(), [...TOP_LEVEL_FIELDS].sort());
    assert.equal(receipt.receipt.schema, SCHEMA);
    assert.equal(receipt.outcome, "pass");
    assert.equal(receipt.export.destination, facts.export.destination);
    assert.equal(receipt.export.evidence_manifest.sha256.length, 64);
    assert.equal(receipt.invocation.stderr.bytes, "0");
    assert.equal(receipt.invocation.operations.proof.stdout.sha256, digestRegular(operationFiles.proof[0], "proof operation stdout").sha256);
    assert.equal(receipt.invocation.tool_register.node.sha256, toolRegister.node.sha256);
    assert.equal(assembleReceipt({ ...facts, inner_pass: false, invocation: { ...facts.invocation, inner_pass: false } }).outcome, "red");
    assert.equal(assembleReceipt({ ...facts, export: { ...facts.export, status: 1, equal: false } }).outcome, "red");
    assert.throws(() => assembleReceipt({ ...facts, image: { ...facts.image, expected_bytes: "1" } }, receiptOptions), /image size/);
    assert.throws(() => assembleReceipt({ ...facts, filesystem: { ...facts.filesystem, type: "ext4" } }, receiptOptions), /filesystem identity/);
    assert.throws(() => assembleReceipt({ ...facts, invocation: { ...facts.invocation, status: 1 } }, receiptOptions), /proof invocation/);
    assert.equal(assembleReceipt({ ...facts, export: { ...facts.export, export_inventory_sha256: "d".repeat(64) } }).outcome, "red");
    assert.equal(assembleReceipt({ ...facts, teardown: { ...facts.teardown, host_monitor: { ...facts.teardown.host_monitor, clean: false } }, host_monitor: { ...facts.host_monitor, clean: false } }).outcome, "red");
    assert.throws(() => assembleReceipt({ ...facts, tool_register: { ...facts.tool_register, extra: { path: "/usr/bin/true", version: "x" } } }, receiptOptions), /tool register fields/);
    assert.throws(() => assembleReceipt({ ...facts, tool_register: { ...facts.tool_register, node: { ...facts.tool_register.node, sha256: "b".repeat(64) } } }, receiptOptions), /digest differs/);
    assert.throws(() => assembleReceipt({ ...facts, filesystem: { ...facts.filesystem, xfs_info_path: null } }, receiptOptions), /XFS info/);
    assert.throws(() => assembleReceipt({ ...facts, export: { ...facts.export, inner_evidence_manifest_path: output } }, receiptOptions), /inner evidence manifest/);
    assert.throws(() => assembleReceipt({ ...facts, operations: { ...facts.operations, proof: { ...facts.operations.proof, argv: ["/usr/bin/false"] } } }, receiptOptions), /operation proof argv/);
    writeFileSync(join(exportDestination, "EVIDENCE.sha256"), `${"0".repeat(64)}  result.txt\n`);
    assert.throws(() => assembleReceipt(facts, receiptOptions), /inner evidence manifest|export inventory digest/);
    writeFileSync(join(exportDestination, "EVIDENCE.sha256"), `${digestRegular(join(exportDestination, "result.txt"), "export result").sha256}  result.txt\n`);
    writeFileSync(join(work, "host-filesystem-before.statfs"), "f_frsize=4096 f_blocks=2000000 f_bfree=1990000 f_bavail=1990001 f_type=xfs f_fsid=1234\n");
    assert.throws(() => assembleReceipt(facts, receiptOptions), /host_filesystem_before_path statfs counters/);
  } finally {
    rmSync(root, { recursive: true, force: true });
  }
});
