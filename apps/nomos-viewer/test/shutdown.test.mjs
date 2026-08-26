import assert from "node:assert/strict";
import { spawn } from "node:child_process";
import { test } from "node:test";

const shutdownUrl = new URL("../smoke/shutdown.mjs", import.meta.url).href;

const runChild = (source, timeout = 2_000) =>
  new Promise((resolve, reject) => {
    const started = performance.now();
    const child = spawn(process.execPath, ["--input-type=module", "--eval", source], {
      stdio: ["ignore", "pipe", "pipe"],
    });
    let stdout = "";
    let stderr = "";
    child.stdout.on("data", (chunk) => {
      stdout += chunk;
    });
    child.stderr.on("data", (chunk) => {
      stderr += chunk;
    });
    const timer = setTimeout(() => {
      child.kill("SIGKILL");
      reject(new Error(`child did not exit within ${timeout}ms`));
    }, timeout);
    child.once("error", (error) => {
      clearTimeout(timer);
      reject(error);
    });
    child.once("exit", (code, signal) => {
      clearTimeout(timer);
      resolve({ code, signal, stdout, stderr, duration_ms: performance.now() - started });
    });
  });

test("the result backstop exits with an intentionally unclosed handle", async () => {
  const result = await runChild(`
    import { exitAfterFlush } from ${JSON.stringify(shutdownUrl)};
    setInterval(() => {}, 60_000);
    process.stdout.write("OPEN_HANDLE_PLANTED\\n");
    await exitAfterFlush(0);
  `);

  assert.equal(result.code, 0, result.stderr);
  assert.equal(result.signal, null);
  assert.equal(result.stdout, "OPEN_HANDLE_PLANTED\n");
  assert.ok(result.duration_ms < 2_000, `backstop took ${result.duration_ms}ms`);
});

test("the hard deadline has a named diagnostic and exits", async () => {
  const result = await runChild(`
    import { armHardDeadline } from ${JSON.stringify(shutdownUrl)};
    armHardDeadline(50);
    setInterval(() => {}, 60_000);
    await new Promise(() => {});
  `);

  assert.equal(result.code, 1);
  assert.equal(result.signal, null);
  assert.match(result.stderr, /NOMOS_VIEWER_SMOKE FAIL HARD_DEADLINE exceeded 50ms/);
  assert.ok(result.duration_ms < 2_000, `hard deadline took ${result.duration_ms}ms`);
});
