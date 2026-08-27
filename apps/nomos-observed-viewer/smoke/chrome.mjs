import { spawn } from "node:child_process";
import { mkdtempSync, readFileSync, rmSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";

const commandPath = async (name) => new Promise((resolve) => {
  const child = spawn("sh", ["-c", `command -v ${name}`], { stdio: ["ignore", "pipe", "ignore"] });
  let output = "";
  child.stdout.on("data", (chunk) => { output += chunk; });
  child.on("exit", (code) => resolve(code === 0 ? output.trim() : null));
});

export const discoverChrome = async () => {
  if (process.env.CHROME_BIN) return process.env.CHROME_BIN;
  for (const name of ["google-chrome", "chromium", "chromium-browser", "chrome-headless-shell"]) {
    const path = await commandPath(name);
    if (path) return path;
  }
  throw new Error("Chrome/Chromium not found; set CHROME_BIN");
};

const delay = (milliseconds) => new Promise((resolve) => setTimeout(resolve, milliseconds));

export const launchChrome = async (executable, { width = 1280, height = 720 } = {}) => {
  const profile = mkdtempSync(join(tmpdir(), "nomos-observed-chrome-"));
  const flags = [
    "--headless=new",
    "--remote-debugging-port=0",
    "--host-resolver-rules=MAP * ~NOTFOUND, EXCLUDE localhost",
    "--no-first-run",
    "--no-default-browser-check",
    "--no-sandbox",
    "--disable-dev-shm-usage",
    "--disable-extensions",
    "--disable-sync",
    "--disable-background-networking",
    "--disable-component-update",
    "--disable-background-timer-throttling",
    "--disable-renderer-backgrounding",
    "--disable-backgrounding-occluded-windows",
    `--window-size=${width},${height}`,
    "--force-device-scale-factor=1",
    "--hide-scrollbars",
    "--use-gl=angle",
    "--use-angle=swiftshader",
    "--enable-unsafe-swiftshader",
    `--user-data-dir=${profile}`,
    "about:blank",
  ];
  const child = spawn(executable, flags, { detached: true, stdio: ["ignore", "ignore", "pipe"] });
  let stderr = "";
  let spawnError = null;
  child.on("error", (error) => { spawnError = error; });
  child.stderr.on("data", (chunk) => { stderr += chunk; });
  const portFile = join(profile, "DevToolsActivePort");
  const deadline = Date.now() + 20_000;
  let port;
  while (Date.now() < deadline) {
    try {
      const [line] = readFileSync(portFile, "utf8").split("\n");
      port = Number(line);
      if (Number.isInteger(port) && port > 0) break;
    } catch {}
    if (spawnError || child.exitCode !== null) {
      rmSync(profile, { force: true, recursive: true });
      throw new Error(`Chrome exited before DevTools: ${spawnError?.message ?? stderr}`);
    }
    await delay(25);
  }
  const launched = { child, flags, port, profile, stderr: () => stderr };
  if (!port) {
    await closeChrome(launched);
    throw new Error(`Chrome did not publish DevTools port: ${stderr}`);
  }
  try {
    const created = await fetch(`http://127.0.0.1:${port}/json/new?about:blank`, { method: "PUT" });
    if (!created.ok) throw new Error(`Chrome target creation failed: ${created.status}`);
    const target = await created.json();
    return { ...launched, websocket: target.webSocketDebuggerUrl };
  } catch (error) {
    await closeChrome(launched);
    throw error;
  }
};

export const closeChrome = async (launched) => {
  const started = process.hrtime.bigint();
  let signal = "SIGTERM";
  const alreadyExited = launched.child.exitCode !== null || launched.child.signalCode !== null;
  if (!alreadyExited) {
    try { process.kill(-launched.child.pid, "SIGTERM"); } catch {}
  }
  const exited = alreadyExited
    ? Promise.resolve()
    : new Promise((resolve) => launched.child.once("exit", resolve));
  const graceful = await Promise.race([exited.then(() => true), delay(1_000).then(() => false)]);
  if (!graceful && launched.child.exitCode === null && launched.child.signalCode === null) {
    signal = "SIGKILL";
    try { process.kill(-launched.child.pid, "SIGKILL"); } catch {}
    const forced = await Promise.race([exited.then(() => true), delay(750).then(() => false)]);
    if (!forced && launched.child.exitCode === null && launched.child.signalCode === null) {
      throw new Error(`Chrome process group did not exit after SIGKILL: ${launched.child.pid}`);
    }
  }
  rmSync(launched.profile, { force: true, recursive: true });
  return {
    duration_ms: Number(process.hrtime.bigint() - started) / 1_000_000,
    exit_code: launched.child.exitCode,
    signal,
  };
};
