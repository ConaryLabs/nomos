// Finding a Chrome, and starting it with the flags the lane needs.
//
// `docs/review/nomos-viewer.md` section 5.1 is the design. Discovery is
// CHROME_BIN, then the usual names on PATH, then - only as a last resort - a
// Playwright cache that happens to be on a developer's machine. The lane never
// installs one and never depends on one being there.

import { execFileSync, spawn } from "node:child_process";
import { accessSync, constants, mkdtempSync, readFileSync, readdirSync, rmSync, statSync } from "node:fs";
import { get } from "node:http";
import { homedir, tmpdir } from "node:os";
import { join } from "node:path";

const PATH_NAMES = ["google-chrome", "google-chrome-stable", "chromium", "chromium-browser"];

const executable = (path) => {
  try {
    accessSync(path, constants.X_OK);
    return statSync(path).isFile();
  } catch {
    return false;
  }
};

const onPath = (name) => {
  for (const directory of (process.env.PATH ?? "").split(":")) {
    if (!directory) continue;
    const candidate = join(directory, name);
    if (executable(candidate)) return candidate;
  }
  return null;
};

// A Playwright download, if one is already here. Highest build first, and never
// required: CI uses the system Chrome, and this only makes the lane runnable on
// a machine that happens to have Playwright installed for something else.
const playwrightChromes = () => {
  const root = join(homedir(), ".cache", "ms-playwright");
  let entries;
  try {
    entries = readdirSync(root);
  } catch {
    return [];
  }
  const order = (name) => Number(name.split("-").at(-1));
  const sorted = (prefix) =>
    entries.filter((name) => name.startsWith(prefix)).sort((left, right) => order(right) - order(left));
  const candidates = [];
  for (const build of sorted("chromium-")) {
    for (const linux of ["chrome-linux64", "chrome-linux"]) {
      candidates.push(join(root, build, linux, "chrome"));
    }
  }
  // The headless shell is the same engine with fewer system libraries, and is
  // often the one that actually starts on a machine with no desktop stack.
  for (const build of sorted("chromium_headless_shell-")) {
    for (const linux of ["chrome-headless-shell-linux64", "chrome-headless-shell-linux"]) {
      candidates.push(join(root, build, linux, "chrome-headless-shell"));
    }
  }
  return candidates.filter(executable);
};

// A binary that is executable is not necessarily a browser that starts: a
// Playwright download on a machine with no desktop stack is present, marked
// executable, and dies on a missing shared library. Asking it its version costs
// a few milliseconds and is the difference between skipping a candidate and
// failing the lane on it.
const versionOf = (binary) => {
  try {
    return execFileSync(binary, ["--version"], { encoding: "utf8", timeout: 10_000, stdio: ["ignore", "pipe", "ignore"] }).trim();
  } catch {
    return null;
  }
};

/// The Chrome this machine offers, and where it came from.

// Chrome's renderer and GPU children can outlive a SIGKILL on the main pid by
// a few hundred milliseconds and keep writing into the profile, so a single
// recursive removal can race them and fail with ENOTEMPTY. The directory is a
// tmpdir the harness created; failing to remove it must never fail the lane.
function removeUserDataDir(dir) {
  const started = Date.now();
  for (;;) {
    try {
      rmSync(dir, { recursive: true, force: true });
      return;
    } catch (error) {
      if (Date.now() - started > 3_000) {
        process.stderr.write(`chrome profile dir left behind: ${dir} (${error.code ?? error.message})\n`);
        return;
      }
      const until = Date.now() + 100;
      while (Date.now() < until) { /* bounded busy-wait; cleanup path only */ }
    }
  }
}
export function findChrome() {
  if (process.env.CHROME_BIN) {
    if (!executable(process.env.CHROME_BIN)) {
      throw new Error(`CHROME_BIN is set to ${process.env.CHROME_BIN}, which is not executable`);
    }
    const version = versionOf(process.env.CHROME_BIN);
    if (!version) throw new Error(`CHROME_BIN is set to ${process.env.CHROME_BIN}, which does not start`);
    return { binary: process.env.CHROME_BIN, source: "CHROME_BIN", version };
  }
  for (const name of PATH_NAMES) {
    const found = onPath(name);
    const version = found && versionOf(found);
    if (version) return { binary: found, source: "PATH", version };
  }
  for (const candidate of playwrightChromes()) {
    const version = versionOf(candidate);
    if (version) return { binary: candidate, source: "playwright-cache", version };
  }
  return null;
}

// Host resolution is mapped to NOTFOUND for every name, so a page that reaches
// for an origin cannot resolve one. The lane navigates to a loopback address
// literal, which needs no resolution at all; the EXCLUDE keeps `localhost`
// working for anything that spells it that way.
const BASE_FLAGS = [
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
  "--window-size=1280,720",
  "--force-device-scale-factor=1",
  "--hide-scrollbars",
];

// Software WebGL, two ways. Recent Chrome refuses a SwiftShader context without
// `--enable-unsafe-swiftshader`; older builds want `--disable-gpu`. Set A is
// tried first and set B is the fallback, so the lane does not have to guess at
// the runner's version.
export const FLAG_SETS = {
  A: [...BASE_FLAGS, "--use-gl=angle", "--use-angle=swiftshader", "--enable-unsafe-swiftshader"],
  B: [...BASE_FLAGS, "--disable-gpu", "--use-gl=swiftshader", "--enable-unsafe-swiftshader"],
};

const readJson = (url) =>
  new Promise((resolve, reject) => {
    get(url, (response) => {
      const chunks = [];
      response.on("data", (chunk) => chunks.push(chunk));
      response.on("end", () => {
        try {
          resolve(JSON.parse(Buffer.concat(chunks).toString("utf8")));
        } catch (error) {
          reject(error);
        }
      });
    }).on("error", reject);
  });

const sleep = (ms) => new Promise((resolve) => setTimeout(resolve, ms));

/// Launches Chrome and returns the page target's DevTools endpoint.
export async function launch({ binary, flags, timeout = 20_000 }) {
  const userDataDir = mkdtempSync(join(tmpdir(), "nomos-viewer-chrome-"));
  // The headless shell is headless by construction and rejects the switch.
  const shell = binary.includes("chrome-headless-shell");
  const argv = [
    ...flags.filter((flag) => !(shell && flag.startsWith("--headless"))),
    `--user-data-dir=${userDataDir}`,
    "about:blank",
  ];
  const child = spawn(binary, argv, { stdio: ["ignore", "pipe", "pipe"] });
  const stderr = [];
  child.stderr.on("data", (chunk) => stderr.push(chunk.toString("utf8")));

  const portFile = join(userDataDir, "DevToolsActivePort");
  const deadline = Date.now() + timeout;
  let port = null;
  while (Date.now() < deadline) {
    if (child.exitCode !== null) {
      throw new Error(`chrome exited with ${child.exitCode}: ${stderr.join("").slice(-500)}`);
    }
    try {
      const lines = readFileSync(portFile, "utf8").split("\n");
      if (lines.length >= 2 && lines[0].trim()) {
        port = Number(lines[0].trim());
        break;
      }
    } catch {
      // Not written yet.
    }
    await sleep(50);
  }
  if (!port) {
    child.kill("SIGKILL");
    removeUserDataDir(userDataDir);
    throw new Error(`chrome wrote no DevToolsActivePort within ${timeout}ms: ${stderr.join("").slice(-500)}`);
  }

  const version = await readJson(`http://127.0.0.1:${port}/json/version`);
  let pages = [];
  const pageDeadline = Date.now() + timeout;
  while (Date.now() < pageDeadline) {
    const targets = await readJson(`http://127.0.0.1:${port}/json/list`);
    pages = targets.filter((one) => one.type === "page" && one.webSocketDebuggerUrl);
    if (pages.length > 0) break;
    await sleep(50);
  }
  if (pages.length === 0) throw new Error("chrome exposed no page target");

  return {
    port,
    argv,
    version,
    pageUrl: pages[0].webSocketDebuggerUrl,
    stderr,
    // Issue #160: killing the child is not the same as it being gone. Node
    // keeps the process handle alive until `exit` fires, so the lane awaits it
    // — with a bound, because a Chrome that will not die must not turn a pass
    // into a hang either.
    kill: async () => {
      if (child.exitCode === null && child.signalCode === null) {
        const gone = new Promise((done) => child.once("exit", done));
        child.kill("SIGKILL");
        await Promise.race([gone, sleep(2_000)]);
      }
      child.stdout?.destroy();
      child.stderr?.destroy();
      child.unref();
      removeUserDataDir(userDataDir);
    },
  };
}
