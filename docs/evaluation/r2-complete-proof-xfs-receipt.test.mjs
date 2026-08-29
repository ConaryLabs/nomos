import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import { closeSync, existsSync, mkdtempSync, mkdirSync, openSync, readFileSync, rmSync, statSync, symlinkSync, writeFileSync } from "node:fs";
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
  bindHostMonitor,
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

test("inventory and export retain descriptor roots while publishing canonical paths", () => {
  const root = temporary();
  let descriptor;
  try {
    const source = join(root, "source");
    const parent = join(root, "export");
    const destination = join(parent, "copy");
    mkdirSync(source);
    mkdirSync(parent);
    writeFileSync(join(source, "value"), "descriptor-bound\n");
    descriptor = openSync(root, "r");
    const descriptorRoot = `/proc/self/fd/${descriptor}`;
    const before = canonicalInventory(join(descriptorRoot, "source"));
    const after = copyTreeNoDeref(join(descriptorRoot, "source"), join(descriptorRoot, "export", "copy"));
    assert.equal(before.root, source);
    assert.equal(after.root, destination);
    assert.equal(compareInventories(before, after).equal, true);
    symlinkSync("value", join(source, "linked"));
    expectFailure(() => canonicalInventory(join(descriptorRoot, "source")), /symlink/);
  } finally {
    if (descriptor !== undefined) closeSync(descriptor);
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
    const bound = bindHostMonitor({ teardown: { host_monitor: { clean: false } } }, clean);
    assert.deepEqual(bound.host_monitor, clean);
    assert.deepEqual(bound.teardown.host_monitor, clean);
    expectFailure(() => validateHostMonitor({
      beforeMount: "", afterMount: "", beforeMountStatus: 2,
      beforeLoops: { loopdevices: [] }, afterLoops: { loopdevices: [] },
      image, mountpoint: join(root, "fs"), mountNamespaceBefore: "mnt:[1]",
      mountNamespaceAfter: "mnt:[1]", proofLoopDevice: "/dev/loop9",
    }), /mount probes/);
    expectFailure(() => validateHostMonitor({
      beforeMount: "", afterMount: "", afterMountStderr: "findmnt failed\n",
      beforeLoops: { loopdevices: [] }, afterLoops: { loopdevices: [] },
      image, mountpoint: join(root, "fs"), mountNamespaceBefore: "mnt:[1]",
      mountNamespaceAfter: "mnt:[1]", proofLoopDevice: "/dev/loop9",
    }), /wrote stderr/);
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
    const unrelated = validateHostMonitor({
      beforeMount: "",
      afterMount: "",
      beforeLoops: { loopdevices: [] },
      afterLoops: { loopdevices: [{ name: "/dev/loop10", backing_file: "/dev/other" }] },
      image,
      mountpoint: join(root, "fs"),
      mountNamespaceBefore: "mnt:[1]",
      mountNamespaceAfter: "mnt:[1]",
      proofLoopDevice: "/dev/loop9",
    });
    assert.equal(unrelated.clean, true);
    const unrelatedReuse = validateHostMonitor({
      beforeMount: "",
      afterMount: "",
      beforeLoops: { loopdevices: [] },
      afterLoops: { loopdevices: [{ name: "/dev/loop9", backing_file: "/dev/other" }] },
      image,
      mountpoint: join(root, "fs"),
      mountNamespaceBefore: "mnt:[1]",
      mountNamespaceAfter: "mnt:[1]",
      proofLoopDevice: "/dev/loop9",
    });
    assert.equal(unrelatedReuse.clean, true);
    expectFailure(() => validateHostMonitor({
      beforeMount: "",
      afterMount: "",
      beforeLoops: { loopdevices: [] },
      afterLoops: { loopdevices: [{ name: "/dev/loop10", backing_file: image }] },
      image,
      mountpoint: join(root, "fs"),
      mountNamespaceBefore: "mnt:[1]",
      mountNamespaceAfter: "mnt:[1]",
      proofLoopDevice: "/dev/loop9",
    }), /backing image|proof loop/);
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
    const evidenceBytes = `${resultDigest}  result.txt\n`;
    writeFileSync(evidence, evidenceBytes);
    writeFileSync(join(exportDestination, "EVIDENCE.sha256"), evidenceBytes);
    for (const path of [
      join(work, "image-fallocate.stdout"), join(work, "image-fallocate.stderr"),
      join(work, "image-sync.stdout"), join(work, "image-sync.stderr"),
    ]) writeFileSync(path, "");
    writeFileSync(join(work, "image.filefrag"), "Filesystem type is: 0x58465342\nFile size of test is 8589934592 (2097152 blocks of 4096 bytes)\n ext: logical_offset: physical_offset: length: expected: flags:\n   0:        0..       0:        1..       1: last,eof\n");
    writeFileSync(join(work, "xfs-info.txt"), "log = internal\nrealtime = none\n");
    const preFormatStatPath = join(work, "image-pre-format.stat");
    const postTeardownStatPath = join(work, "image-post-teardown.stat");
    const preFormatBlocks = IMAGE_BYTES / 512n;
    const postTeardownBlocks = preFormatBlocks + 8n;
    const imageStatText = ({ logical = IMAGE_BYTES, blocks, allocated = blocks * 512n, blockSize = 512n }) =>
      `logical_bytes=${logical}\nst_blocks=${blocks}\nallocated_bytes=${allocated}\nblock_size=${blockSize}\n`;
    writeFileSync(preFormatStatPath, imageStatText({ blocks: preFormatBlocks }));
    writeFileSync(postTeardownStatPath, imageStatText({ blocks: postTeardownBlocks }));
    const statfs = { f_type: "1481003842", f_bsize: "4096", f_blocks: "2000000", f_bfree: "1990000", f_bavail: "1985000" };
    for (const name of ["statfs-mounted.json", "statfs-checkout.json", "statfs-close.json"]) writeFileSync(join(work, name), `${JSON.stringify(statfs)}\n`);
    const hostStatfs = "f_frsize=4096 f_blocks=2000000 f_bfree=1990000 f_bavail=1985000 f_type=xfs f_fsid=1234\n";
    const hostFindmnt = `${work} /dev/test xfs rw,nodev,nosuid private\n`;
    for (const prefix of [join(work, "host-filesystem-before"), join(work, "host-filesystem-after")]) {
      writeFileSync(`${prefix}.statfs`, hostStatfs);
      writeFileSync(`${prefix}.findmnt`, hostFindmnt);
      writeFileSync(`${prefix}.quota`, "xfs quota state\n");
      writeFileSync(`${prefix}.statfs.status`, "0\n");
      writeFileSync(`${prefix}.findmnt.status`, "0\n");
      writeFileSync(`${prefix}.quota.status`, "0\n");
      writeFileSync(`${prefix}.statfs.stderr`, "");
      writeFileSync(`${prefix}.findmnt.stderr`, "");
      writeFileSync(`${prefix}.quota.stderr`, "");
    }
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
    writeFileSync(monitorEvidencePaths.before_mount, "");
    writeFileSync(monitorEvidencePaths.after_mount, "");
    writeFileSync(monitorEvidencePaths.before_mount_stderr, "");
    writeFileSync(monitorEvidencePaths.after_mount_stderr, "");
    writeFileSync(monitorEvidencePaths.before_mount_status, "1\n");
    writeFileSync(monitorEvidencePaths.after_mount_status, "1\n");
    writeFileSync(monitorEvidencePaths.before_loops, "{\"loopdevices\":[]}\n");
    writeFileSync(monitorEvidencePaths.after_loops, "{\"loopdevices\":[]}\n");
    writeFileSync(monitorEvidencePaths.before_loops_status, "0\n");
    writeFileSync(monitorEvidencePaths.after_loops_status, "0\n");
    writeFileSync(monitorEvidencePaths.before_loops_stderr, "");
    writeFileSync(monitorEvidencePaths.after_loops_stderr, "");
    writeFileSync(monitorEvidencePaths.mount_namespace_before, "mnt:[1]\n");
    writeFileSync(monitorEvidencePaths.mount_namespace_after, "mnt:[1]\n");
    writeFileSync(join(work, "host-before-net-ns"), "net:[10]\n");
    writeFileSync(join(work, "host-after-net-ns"), "net:[10]\n");
    writeFileSync(join(work, "host-before-pid-ns"), "pid:[20]\n");
    writeFileSync(join(work, "host-after-pid-ns"), "pid:[20]\n");
    const outerPreflightPath = join(work, "outer-preflight.json");
    const outerPreflight = {
      cap_inheritable: "0000000000000000", cap_permitted: "0000000000000000",
      cap_effective: "0000000000000000", cap_bounding: "0000000000000000",
      cap_ambient: "0000000000000000", no_new_privs: 1,
      host_network_namespace: "net:[10]", host_pid_namespace: "pid:[20]",
      network_namespace: "net:[30]", pid_namespace: "pid:[40]",
    };
    const outerPreflightText = `${JSON.stringify(outerPreflight)}\n`;
    writeFileSync(outerPreflightPath, outerPreflightText);
    const cleanMonitor = validateHostMonitor({ evidencePaths: monitorEvidencePaths, image, mountpoint: fs, proofLoopDevice: "/dev/loop9" });
    const sourceInventory = canonicalInventory(output);
    const exportInventory = canonicalInventory(exportDestination);
    const inventoryPath = join(work, "export.json");
    const inventoryDigestPath = join(work, "export.sha256");
    writeFileSync(inventoryPath, `${JSON.stringify({ source: output, destination: exportDestination, source_inventory_sha256: sourceInventory.sha256, export_inventory_sha256: exportInventory.sha256, rows: sourceInventory.rows.length, equal: true })}\n`);
    writeFileSync(inventoryDigestPath, `source\t${sourceInventory.sha256}\nexport\t${exportInventory.sha256}\n`);
    const toolNames = "bash sh git realpath find stat date mkdir readlink id node jq sudo unshare findmnt losetup mount umount blockdev mkfs.xfs xfs_info xfs_quota filefrag fuser fallocate sync setpriv bwrap tar ionice du blkid chown cp rm env sha256sum awk grep tr chmod mv dirname pwd cut rustup cargo rustc".split(" ");
    const toolPaths = Object.fromEntries(toolNames.map((name) => [name, "/usr/bin/node"]));
    Object.assign(toolPaths, {
      bash: "/usr/bin/bash", fallocate: "/usr/bin/fallocate", sync: "/usr/lib/cargo/bin/coreutils/sync", losetup: "/usr/sbin/losetup",
      "mkfs.xfs": "/usr/sbin/mkfs.xfs", mount: "/usr/bin/mount", node: "/usr/bin/node", umount: "/usr/bin/umount",
    });
    const toolRegister = Object.fromEntries(Object.entries(toolPaths).map(([name, path]) => [name, {
      path, version_argv: "--version", version_status: 0, sha256: digestRegular(path, `${name} test tool`).sha256, version: `${name} test-version`,
    }]));
    toolRegister.sh.version_status = 2;
    const operationPrefixes = {
      fallocate: "image-fallocate", image_sync: "image-sync", loop_attach: "loop-attach",
      mkfs_xfs: "mkfs-xfs", mount: "mount", proof: "proof", export: "export",
      sync_before_umount: "sync-before-umount", umount: "umount", loop_detach: "loop-detach",
    };
    const operationFiles = Object.fromEntries(Object.entries(operationPrefixes)
      .map(([name, prefix]) => [name, [join(work, `${prefix}.stdout`), join(work, `${prefix}.stderr`)]]));
    for (const [stdoutPath, stderrPath] of Object.values(operationFiles)) { writeFileSync(stdoutPath, ""); writeFileSync(stderrPath, ""); }
    const operation = (name, argv, cwd) => ({ argv, cwd, status: 0, stdout_path: operationFiles[name][0], stderr_path: operationFiles[name][1] });
    const operations = {
      fallocate: operation("fallocate", ["/usr/bin/fallocate", "--posix", "--length", IMAGE_BYTES.toString(), image], work),
      image_sync: operation("image_sync", ["/usr/bin/sync", "-f", image], work),
      loop_attach: operation("loop_attach", ["/usr/sbin/losetup", "--find", "--show", image], work),
      mkfs_xfs: operation("mkfs_xfs", ["/usr/sbin/mkfs.xfs", "-f", "-K", "-l", "internal", "/dev/loop9"], work),
      mount: operation("mount", ["/usr/bin/mount", "-t", "xfs", "-o", "rw,nodev,nosuid", "/dev/loop9", fs], work),
      proof: operation("proof", ["/usr/bin/bash", join(checkout, "docs", "evaluation", "r2-complete-proof.sh"), "--output", output], checkout),
      export: operation("export", ["/usr/bin/node", join(checkout, "docs", "evaluation", "r2-complete-proof-xfs-receipt.mjs"), "copy", "--source", output, "--destination", exportDestination, "--output", inventoryPath], work),
      sync_before_umount: operation("sync_before_umount", ["/usr/bin/sync", "-f", fs], work),
      umount: operation("umount", ["/usr/bin/umount", fs], work),
      loop_detach: operation("loop_detach", ["/usr/sbin/losetup", "--detach", "/dev/loop9"], work),
    };
    const ledgerRows = [
      ["host-filesystem-before.quota", "host-filesystem-before.quota", "host-filesystem-before.quota.stderr"],
      ["image-fallocate", "image-fallocate.stdout", "image-fallocate.stderr", "fallocate"],
      ["image-sync", "image-sync.stdout", "image-sync.stderr", "image_sync"],
      ["image.filefrag", "image.filefrag", "image.filefrag.stderr"],
      ["host-filesystem-after.quota", "host-filesystem-after.quota", "host-filesystem-after.quota.stderr"],
      ["loop-attach", "loop-attach.stdout", "loop-attach.stderr", "loop_attach"],
      ["loop-size", "loop-size.stdout", "loop-size.stderr"],
      ["mkfs-xfs", "mkfs-xfs.stdout", "mkfs-xfs.stderr", "mkfs_xfs"],
      ["xfs-info.txt", "xfs-info.txt", "xfs-info.stderr"],
      ["blkid", "blkid.stdout", "blkid.stderr"],
      ["xfs-uuid", "xfs-uuid.stdout", "xfs-uuid.stderr"],
      ["mount", "mount.stdout", "mount.stderr", "mount"],
      ["clone", "clone.stdout", "clone.stderr"],
      ["inner-proof", "proof.stdout", "proof.stderr", "proof"],
      ["export", "export.stdout", "export.stderr", "export"],
      ["fuser", "fuser.stdout", "fuser.stderr"],
      ["sync-before-umount", "sync-before-umount.stdout", "sync-before-umount.stderr", "sync_before_umount"],
      ["umount", "umount.stdout", "umount.stderr", "umount"],
      ["loop-detach", "loop-detach.stdout", "loop-detach.stderr", "loop_detach"],
      ["loop-fuser", "loop-fuser.stdout", "loop-fuser.stderr"],
      ["loop-associated", "loop-associated.stdout", "loop-associated.stderr"],
    ];
    const ledgerPath = join(work, "wrapper-commands.ndjson");
    const callerLedgerRows = new Set([
      "image-fallocate", "image-sync", "image.filefrag", "clone", "inner-proof", "export",
    ]);
    const nonzeroLedgerRows = new Set(["fuser", "loop-fuser"]);
    const proofLedgerArgv = ["/usr/bin/env", "NOMOS_R2_XFS_WRAPPER=1", "NOMOS_R2_XFS_UUID=11111111-2222-3333-4444-555555555555",
      "NOMOS_R2_XFS_FRAGMENT_SIZE=4096", "NOMOS_R2_XFS_DEVICE=/dev/loop9", "NOMOS_R2_XFS_MAJOR_MINOR=7:9",
      `NOMOS_R2_OUTER_PREFLIGHT_LOG=${outerPreflightPath}`,
      `NOMOS_R2_OUTER_POSITIVE_STDOUT=${join(work, "network-outer-positive.stdout")}`,
      `NOMOS_R2_OUTER_POSITIVE_STDERR=${join(work, "network-outer-positive.stderr")}`,
      "/usr/bin/bash", join(checkout, "docs", "evaluation", "r2-complete-proof.sh"), "--output", output];
    writeFileSync(join(work, "network-outer-positive.stdout"), "");
    writeFileSync(join(work, "network-outer-positive.stderr"), "");
    const ledgerArgv = {
      "host-filesystem-before.quota": ["/usr/sbin/xfs_quota", "-x", "-c", "state -v", work],
      "image-fallocate": operations.fallocate.argv,
      "image-sync": operations.image_sync.argv,
      "image.filefrag": ["/usr/sbin/filefrag", "-v", image],
      "host-filesystem-after.quota": ["/usr/sbin/xfs_quota", "-x", "-c", "state -v", work],
      "loop-attach": operations.loop_attach.argv,
      "loop-size": ["/usr/sbin/blockdev", "--getsize64", "/dev/loop9"],
      "mkfs-xfs": operations.mkfs_xfs.argv,
      "xfs-info.txt": ["/usr/sbin/xfs_info", "/dev/loop9"],
      blkid: ["/usr/sbin/blkid", "-p", "-s", "TYPE", "-o", "value", "/dev/loop9"],
      "xfs-uuid": ["/usr/sbin/blkid", "-p", "-s", "UUID", "-o", "value", "/dev/loop9"],
      mount: operations.mount.argv,
      clone: ["/usr/bin/git", "-c", "protocol.file.allow=always", "clone", "--no-local", "--no-hardlinks", "--no-checkout", "--config", "core.hooksPath=/dev/null", source, checkout],
      "inner-proof": proofLedgerArgv,
      export: operations.export.argv,
      fuser: ["/usr/bin/fuser", "-m", fs],
      "sync-before-umount": operations.sync_before_umount.argv,
      umount: operations.umount.argv,
      "loop-detach": operations.loop_detach.argv,
      "loop-fuser": ["/usr/bin/fuser", "/dev/loop9"],
      "loop-associated": ["/usr/sbin/losetup", "--associated", image],
    };
    let ledgerTimestamp = 100n;
    const ledgerText = ledgerRows.map(([id, stdoutName, stderrName, operationName]) => {
      const started = ledgerTimestamp;
      ledgerTimestamp += 10n;
      const ended = ledgerTimestamp;
      ledgerTimestamp += 10n;
      const argv = ledgerArgv[id];
      const callerRow = callerLedgerRows.has(id);
      return JSON.stringify({
        id, started_ns: started.toString(), ended_ns: ended.toString(),
        status: nonzeroLedgerRows.has(id) ? 1 : 0,
        uid: callerRow ? 1000 : 0, gid: callerRow ? 1000 : 0,
        cwd: id === "inner-proof" ? checkout : work, argv,
        stdout_path: join(work, stdoutName), stderr_path: join(work, stderrName),
      });
    }).join("\n") + "\n";
    for (const [, stdoutName, stderrName] of ledgerRows) {
      if (!existsSync(join(work, stdoutName))) writeFileSync(join(work, stdoutName), "");
      if (!existsSync(join(work, stderrName))) writeFileSync(join(work, stderrName), "");
    }
    writeFileSync(ledgerPath, ledgerText);
    const executionLedgerPath = join(work, "wrapper-execution.ndjson");
    const executionRoot = "/proc/self/fd/19";
    const executionSourceRoot = "/proc/self/fd/18";
    const toDescriptorPath = (value) => value === work ? executionRoot :
      value.startsWith(`${work}/`) ? `${executionRoot}${value.slice(work.length)}` : value;
    const toActualArgument = (id, value) => {
      if (id === "clone" && value === source) return executionSourceRoot;
      const direct = toDescriptorPath(value);
      if (direct !== value) return direct;
      const separator = value.indexOf("=");
      if (separator >= 1) {
        const mapped = toDescriptorPath(value.slice(separator + 1));
        if (mapped !== value.slice(separator + 1)) return `${value.slice(0, separator + 1)}${mapped}`;
      }
      return direct;
    };
    const workStat = statSync(work, { bigint: true });
    const workIdentity = `${workStat.dev}:${workStat.ino}`;
    const semanticRows = ledgerText.trimEnd().split("\n").map((row) => JSON.parse(row));
    const executionRows = semanticRows.map((row) => ({
      id: row.id, started_ns: row.started_ns, ended_ns: row.ended_ns,
      status: row.status, uid: row.uid, gid: row.gid,
      actual_argv: row.argv.map((argument) => toActualArgument(row.id, argument)),
      actual_cwd: toDescriptorPath(row.cwd),
      actual_stdout_path: toDescriptorPath(row.stdout_path),
      actual_stderr_path: toDescriptorPath(row.stderr_path),
      bound_argv: row.argv, bound_cwd: row.cwd,
      bound_stdout_path: row.stdout_path, bound_stderr_path: row.stderr_path,
      canonical_work_path: work, work_identity: workIdentity,
      work_identity_before: workIdentity, work_identity_after: workIdentity,
    }));
    const executionLedgerText = executionRows.map((row) => JSON.stringify(row)).join("\n") + "\n";
    writeFileSync(executionLedgerPath, executionLedgerText);
    const facts = {
      setup_failed: false,
      inner_pass: true,
      candidate: { source, commit: "a".repeat(40), tree: "b".repeat(40), clean: true, source_status: 0 },
      image: {
        path: image,
        pre_format_stat_path: preFormatStatPath,
        pre_format_filefrag_path: join(work, "image.filefrag"),
        post_teardown_stat_path: postTeardownStatPath,
        fallocate_stdout: join(work, "image-fallocate.stdout"),
        fallocate_stderr: join(work, "image-fallocate.stderr"),
        sync_stdout: join(work, "image-sync.stdout"),
        sync_stderr: join(work, "image-sync.stderr"),
        status: 0,
        sync_status: 0,
        pre_format_logical_bytes: IMAGE_BYTES.toString(),
        pre_format_st_blocks: preFormatBlocks.toString(),
        pre_format_block_size: "512",
        pre_format_allocated_bytes: IMAGE_BYTES.toString(),
        post_teardown_logical_bytes: IMAGE_BYTES.toString(),
        post_teardown_st_blocks: postTeardownBlocks.toString(),
        post_teardown_block_size: "512",
        post_teardown_allocated_bytes: (postTeardownBlocks * 512n).toString(),
        expected_bytes: IMAGE_BYTES.toString(),
      },
      loop_device: { path: "/dev/loop9", major_minor: "7:9", size_bytes: IMAGE_BYTES.toString(), attached: false },
      filesystem: { type: "xfs", uuid: "11111111-2222-3333-4444-555555555555", fragment_size: "4096", capacity_limit_bytes: IMAGE_BYTES.toString(), capacity_ok: true, mounted_statfs_path: join(work, "statfs-mounted.json"), checkout_statfs_path: join(work, "statfs-checkout.json"), close_statfs_path: join(work, "statfs-close.json"), host_filesystem_before_path: join(work, "host-filesystem-before"), host_filesystem_after_path: join(work, "host-filesystem-after"), xfs_info_path: join(work, "xfs-info.txt") },
      mount: { path: fs, source: "/dev/loop9", options: "rw,nodev,nosuid,relatime", propagation: "private", status: 0, mounted: true, unmounted: true, mount_absent: true },
      invocation: {
        argv: ["/usr/bin/bash", join(checkout, "docs", "evaluation", "r2-complete-proof.sh"), "--output", output],
        cwd: checkout, uid: 1000, gid: 1000, user: "test", status: 0, inner_pass: true,
        start_ns: "360", end_ns: "370", stdout_path: stdout, stderr_path: stderr,
        command_ledger_path: ledgerPath, execution_ledger_path: executionLedgerPath,
        outer_preflight_path: outerPreflightPath,
      },
      export: { source: output, destination: exportDestination, status: 0, equal: true, inventory_path: inventoryPath, inventory_digest_path: inventoryDigestPath, source_inventory_sha256: sourceInventory.sha256, export_inventory_sha256: exportInventory.sha256, inner_evidence_manifest_path: join(exportDestination, "EVIDENCE.sha256") },
      teardown: {
        unmounted: true, loop_detached: true, no_holder: true, mount_absent: true, image_unattached: true, fuser_status: 1, umount_status: 0, detach_status: 0, supervisor_status: 0,
        host_monitor: cleanMonitor,
      },
      host_monitor: cleanMonitor,
      operations,
      tool_register: toolRegister,
    };
    const receiptOptions = { statReader: () => ({ size: IMAGE_BYTES, blocks: postTeardownBlocks }) };
    const receipt = assembleReceipt(facts, receiptOptions);
    assert.deepEqual(Object.keys(receipt).sort(), [...TOP_LEVEL_FIELDS].sort());
    assert.equal(receipt.receipt.schema, SCHEMA);
    assert.equal(receipt.outcome, "pass");
    assert.equal(receipt.image.pre_format_allocated_bytes, IMAGE_BYTES.toString());
    assert.equal(receipt.image.post_teardown_allocated_bytes, (IMAGE_BYTES + 4096n).toString());
    assert.equal(receipt.image.evidence.pre_format_stat.sha256,
      digestRegular(preFormatStatPath, "pre-format stat").sha256);
    assert.equal(receipt.image.evidence.post_teardown_stat.sha256,
      digestRegular(postTeardownStatPath, "post-teardown stat").sha256);
    assert.equal(receipt.export.destination, facts.export.destination);
    assert.equal(receipt.export.evidence_manifest.sha256,
      createHash("sha256").update(evidenceBytes).digest("hex"));
    assert.equal(receipt.invocation.stderr.bytes, "0");
    assert.equal(receipt.invocation.operations.proof.stdout.sha256, digestRegular(operationFiles.proof[0], "proof operation stdout").sha256);
    assert.equal(receipt.invocation.tool_register.node.sha256, toolRegister.node.sha256);
    assert.equal(receipt.invocation.command_ledger.record_count, ledgerRows.length);
    assert.equal(receipt.invocation.command_ledger.sha256, digestRegular(ledgerPath, "ledger").sha256);
    assert.equal(receipt.invocation.command_ledger.execution.record_count, ledgerRows.length);
    assert.equal(receipt.invocation.command_ledger.execution.work_identity, workIdentity);
    assert.equal(receipt.invocation.command_ledger.execution.source_descriptor_root, executionSourceRoot);
    assert.equal(receipt.invocation.preflight.sha256,
      digestRegular(outerPreflightPath, "outer preflight").sha256);
    const validatorHelper = new URL("./r2-complete-proof-xfs-receipt.mjs", import.meta.url).pathname;
    const wrongHelperFacts = {
      ...facts,
      operations: {
        ...facts.operations,
        export: { ...facts.operations.export, argv: facts.operations.export.argv.with(1, validatorHelper) },
      },
    };
    assert.throws(() => assembleReceipt(wrongHelperFacts, receiptOptions),
      /operation export argv differs from the exact wrapper command/);
    const missingMkfsNoDiscardFacts = {
      ...facts,
      operations: {
        ...facts.operations,
        mkfs_xfs: { ...facts.operations.mkfs_xfs, argv: facts.operations.mkfs_xfs.argv.toSpliced(2, 1) },
      },
    };
    assert.throws(() => assembleReceipt(missingMkfsNoDiscardFacts, receiptOptions),
      /operation mkfs_xfs argv differs from the exact wrapper command/);
    const rewriteExecutionLedger = (rows) =>
      writeFileSync(executionLedgerPath, rows.map((row) => JSON.stringify(row)).join("\n") + "\n");
    const mutatedExecutionRows = executionLedgerText.trimEnd().split("\n").map((row) => JSON.parse(row));
    const cloneExecution = mutatedExecutionRows.find(({ id }) => id === "clone");
    cloneExecution.actual_argv[cloneExecution.actual_argv.length - 2] = source;
    rewriteExecutionLedger(mutatedExecutionRows);
    assert.throws(() => assembleReceipt(facts, receiptOptions), /clone source is not descriptor-derived/);
    cloneExecution.actual_argv[cloneExecution.actual_argv.length - 2] = executionRoot;
    rewriteExecutionLedger(mutatedExecutionRows);
    assert.throws(() => assembleReceipt(facts, receiptOptions), /clone source is not a distinct descriptor root/);
    cloneExecution.actual_argv = executionRows.find(({ id }) => id === "clone").actual_argv;
    mutatedExecutionRows.find(({ id }) => id === "loop-attach").actual_argv =
      ["/usr/sbin/losetup", "--find", "--show", image];
    rewriteExecutionLedger(mutatedExecutionRows);
    assert.throws(() => assembleReceipt(facts, receiptOptions), /work argv 3 is not descriptor-derived/);
    mutatedExecutionRows.find(({ id }) => id === "loop-attach").actual_argv =
      executionRows.find(({ id }) => id === "loop-attach").actual_argv;
    const fallocateExecution = mutatedExecutionRows.find(({ id }) => id === "image-fallocate");
    fallocateExecution.actual_argv = ledgerArgv["image-fallocate"];
    rewriteExecutionLedger(mutatedExecutionRows);
    assert.throws(() => assembleReceipt(facts, receiptOptions), /work argv 4 is not descriptor-derived/);
    fallocateExecution.actual_argv = executionRows.find(({ id }) => id === "image-fallocate").actual_argv;
    mutatedExecutionRows[0].bound_argv = ["/usr/bin/false"];
    rewriteExecutionLedger(mutatedExecutionRows);
    assert.throws(() => assembleReceipt(facts, receiptOptions), /bound projection differs/);
    mutatedExecutionRows[0].bound_argv = executionRows[0].bound_argv;
    mutatedExecutionRows[0].work_identity_before = "1:2";
    rewriteExecutionLedger(mutatedExecutionRows);
    assert.throws(() => assembleReceipt(facts, receiptOptions), /work identity differs/);
    mutatedExecutionRows[0].work_identity_before = workIdentity;
    mutatedExecutionRows[0].actual_stdout_path = "/proc/self/fd/20/host-filesystem-before.quota";
    rewriteExecutionLedger(mutatedExecutionRows);
    assert.throws(() => assembleReceipt(facts, receiptOptions), /inconsistent descriptor roots/);
    mutatedExecutionRows[0].actual_stdout_path = executionRows[0].actual_stdout_path;
    rewriteExecutionLedger(mutatedExecutionRows.slice(1));
    assert.throws(() => assembleReceipt(facts, receiptOptions), /row count differs/);
    rewriteExecutionLedger([...mutatedExecutionRows, mutatedExecutionRows[0]]);
    assert.throws(() => assembleReceipt(facts, receiptOptions), /row count differs/);
    rewriteExecutionLedger(mutatedExecutionRows);
    writeFileSync(outerPreflightPath,
      `${JSON.stringify({ ...outerPreflight, cap_effective: "0000000000000001" })}\n`);
    assert.throws(() => assembleReceipt(facts, receiptOptions), /cap_effective is not zero/);
    writeFileSync(outerPreflightPath,
      `${JSON.stringify({ ...outerPreflight, no_new_privs: 0 })}\n`);
    assert.throws(() => assembleReceipt(facts, receiptOptions), /no_new_privs is not one/);
    writeFileSync(outerPreflightPath,
      `${JSON.stringify({ ...outerPreflight, network_namespace: "net:[10]" })}\n`);
    assert.throws(() => assembleReceipt(facts, receiptOptions), /network namespace is not fresh/);
    writeFileSync(outerPreflightPath, outerPreflightText);
    writeFileSync(join(work, "host-after-net-ns"), "net:[11]\n");
    assert.throws(() => assembleReceipt(facts, receiptOptions), /host network_after namespace differs/);
    writeFileSync(join(work, "host-after-net-ns"), "net:[10]\n");
    const reversedLedger = [...ledgerText.trimEnd().split("\n")].reverse().join("\n") + "\n";
    writeFileSync(ledgerPath, reversedLedger);
    assert.throws(() => assembleReceipt(facts, receiptOptions), /ledger row order/);
    writeFileSync(ledgerPath, ledgerText);
    const malformedLedgerRows = ledgerText.trimEnd().split("\n").map((row) => JSON.parse(row));
    malformedLedgerRows[0].extra = true;
    writeFileSync(ledgerPath, malformedLedgerRows.map((row) => JSON.stringify(row)).join("\n") + "\n");
    assert.throws(() => assembleReceipt(facts, receiptOptions), /ledger row 1 fields/);
    delete malformedLedgerRows[0].extra;
    malformedLedgerRows.find(({ id }) => id === "fuser").status = 0;
    writeFileSync(ledgerPath, malformedLedgerRows.map((row) => JSON.stringify(row)).join("\n") + "\n");
    assert.throws(() => assembleReceipt(facts, receiptOptions), /ledger fuser status/);
    writeFileSync(ledgerPath, ledgerText);
    const exactLedgerRows = ledgerText.trimEnd().split("\n").map((row) => JSON.parse(row));
    const rewriteLedger = (rows) => writeFileSync(ledgerPath, rows.map((row) => JSON.stringify(row)).join("\n") + "\n");
    const cloneRow = exactLedgerRows.find(({ id }) => id === "clone");
    cloneRow.argv = [...cloneRow.argv, "unexpected"];
    rewriteLedger(exactLedgerRows);
    assert.throws(() => assembleReceipt(facts, receiptOptions), /ledger clone argv/);
    cloneRow.argv.pop();
    const proofRow = exactLedgerRows.find(({ id }) => id === "inner-proof");
    proofRow.argv = proofRow.argv.slice(1);
    rewriteLedger(exactLedgerRows);
    assert.throws(() => assembleReceipt(facts, receiptOptions), /ledger inner-proof argv/);
    proofRow.argv = proofLedgerArgv;
    proofRow.started_ns = "361";
    rewriteLedger(exactLedgerRows);
    assert.throws(() => assembleReceipt(facts, receiptOptions), /timestamps differ/);
    rewriteLedger(JSON.parse(`[${ledgerText.trimEnd().split("\n").join(",")}]`));
    writeFileSync(monitorEvidencePaths.after_loops_stderr, "unexpected\n");
    assert.throws(() => assembleReceipt(facts, receiptOptions), /digest differs|wrote stderr/);
    writeFileSync(monitorEvidencePaths.after_loops_stderr, "");
    assert.equal(assembleReceipt({ ...facts, inner_pass: false, invocation: { ...facts.invocation, inner_pass: false } }).outcome, "red");
    assert.equal(assembleReceipt({ ...facts, export: { ...facts.export, status: 1, equal: false } }).outcome, "red");
    assert.throws(() => assembleReceipt({ ...facts, image: { ...facts.image, expected_bytes: "1" } }, receiptOptions), /image expected size/);
    assert.throws(() => assembleReceipt({
      ...facts,
      image: { ...facts.image, pre_format_allocated_bytes: (IMAGE_BYTES + 512n).toString() },
    }, receiptOptions), /pre_format_allocated_bytes differs from its stat evidence/);
    assert.throws(() => assembleReceipt(facts, {
      statReader: () => ({ size: IMAGE_BYTES, blocks: preFormatBlocks }),
    }), /post-teardown image stat evidence differs from the image file/);

    writeFileSync(postTeardownStatPath, imageStatText({ blocks: preFormatBlocks - 1n }));
    assert.throws(() => assembleReceipt({
      ...facts,
      image: {
        ...facts.image,
        post_teardown_st_blocks: (preFormatBlocks - 1n).toString(),
        post_teardown_allocated_bytes: (IMAGE_BYTES - 512n).toString(),
      },
    }, { statReader: () => ({ size: IMAGE_BYTES, blocks: preFormatBlocks - 1n }) }),
    /post-teardown image stat evidence does not prove an exact fully allocated image/);
    writeFileSync(postTeardownStatPath, imageStatText({ blocks: postTeardownBlocks }));

    writeFileSync(preFormatStatPath, imageStatText({ blocks: preFormatBlocks - 1n }));
    assert.throws(() => assembleReceipt({
      ...facts,
      image: {
        ...facts.image,
        pre_format_st_blocks: (preFormatBlocks - 1n).toString(),
        pre_format_allocated_bytes: (IMAGE_BYTES - 512n).toString(),
      },
    }, receiptOptions), /pre-format image stat evidence does not prove an exact fully allocated image/);
    writeFileSync(preFormatStatPath, imageStatText({ blocks: preFormatBlocks }));

    writeFileSync(postTeardownStatPath, imageStatText({
      blocks: postTeardownBlocks,
      allocated: postTeardownBlocks * 512n - 1n,
    }));
    assert.throws(() => assembleReceipt(facts, receiptOptions), /post-teardown image stat evidence allocation arithmetic is inconsistent/);
    writeFileSync(postTeardownStatPath, imageStatText({ blocks: postTeardownBlocks }));

    writeFileSync(preFormatStatPath, imageStatText({ blocks: IMAGE_BYTES / 4096n, blockSize: 4096n }));
    assert.throws(() => assembleReceipt(facts, receiptOptions), /pre-format image stat evidence block size is not 512 bytes/);
    writeFileSync(preFormatStatPath, imageStatText({ blocks: preFormatBlocks }));

    writeFileSync(postTeardownStatPath, imageStatText({ logical: IMAGE_BYTES + 1n, blocks: postTeardownBlocks }));
    assert.throws(() => assembleReceipt(facts, receiptOptions), /post-teardown image stat evidence does not prove an exact fully allocated image/);
    writeFileSync(postTeardownStatPath, imageStatText({ blocks: postTeardownBlocks }));

    assert.throws(() => assembleReceipt({
      ...facts,
      image: {
        ...facts.image,
        pre_format_stat_path: postTeardownStatPath,
        post_teardown_stat_path: preFormatStatPath,
      },
    }, receiptOptions), /pre-format image stat path differs from its canonical expected path/);
    const missingCheckpoint = { ...facts.image };
    delete missingCheckpoint.post_teardown_stat_path;
    assert.throws(() => assembleReceipt({ ...facts, image: missingCheckpoint }, receiptOptions), /image fields differ/);
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
