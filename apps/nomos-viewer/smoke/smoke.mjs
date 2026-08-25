// The headless Chromium smoke lane.
//
// Serves the staged artifact from a loopback port, launches the Chrome that is
// already on the machine, drives the solved route to the final escape, and
// fails on a single console error. `docs/review/nomos-viewer.md` section 5 is
// the design, including the twelve ways this fails and the receipt it writes.
//
//   node apps/nomos-viewer/smoke/smoke.mjs --dist apps/nomos-viewer/dist \
//     --out target/nomos-viewer-smoke [--require-chrome]

import { execFileSync } from "node:child_process";
import { mkdirSync, readFileSync, writeFileSync } from "node:fs";
import { dirname, join, resolve } from "node:path";
import { fileURLToPath } from "node:url";

import { decodeCollection, decodePlan } from "../src/plan.mjs";
import { connect } from "./cdp.mjs";
import { FLAG_SETS, findChrome, launch } from "./chrome.mjs";
import { solveRoute } from "./route.mjs";
import { serve } from "./server.mjs";

const here = dirname(fileURLToPath(import.meta.url));

const KEYS = {
  ArrowUp: { code: "ArrowUp", key: "ArrowUp", vk: 38 },
  ArrowDown: { code: "ArrowDown", key: "ArrowDown", vk: 40 },
  ArrowLeft: { code: "ArrowLeft", key: "ArrowLeft", vk: 37 },
  ArrowRight: { code: "ArrowRight", key: "ArrowRight", vk: 39 },
  KeyE: { code: "KeyE", key: "e", vk: 69, text: "e" },
};

const PROBE = "https://example.invalid/nomos-viewer-probe";

const parseArguments = (argv) => {
  const options = {
    dist: join(here, "..", "dist"),
    out: "target/nomos-viewer-smoke",
    requireChrome: false,
  };
  for (let index = 0; index < argv.length; index += 1) {
    if (argv[index] === "--dist") options.dist = argv[index + 1];
    else if (argv[index] === "--out") options.out = argv[index + 1];
    else if (argv[index] === "--require-chrome") options.requireChrome = true;
  }
  return options;
};

const readArtifacts = (dist) => {
  const collection = decodeCollection(
    JSON.parse(readFileSync(join(dist, "areas.json"), "utf8")),
    "areas.json",
  );
  const plans = new Map(
    collection.areas.map((area) => [
      area.id,
      decodePlan(JSON.parse(readFileSync(join(dist, area.plan), "utf8")), area.plan),
    ]),
  );
  return { collection, plans };
};

const gitHead = () => {
  try {
    return execFileSync("git", ["rev-parse", "HEAD"], { cwd: here, encoding: "utf8" }).trim();
  } catch {
    return "unknown";
  }
};

class Failure extends Error {}

const fail = (message) => {
  throw new Failure(message);
};

async function run(options) {
  const dist = resolve(options.dist);
  const out = resolve(options.out);
  mkdirSync(join(out, "screenshots"), { recursive: true });

  const { collection, plans } = readArtifacts(dist);
  const route = solveRoute(collection, plans);

  const chrome = findChrome();
  if (!chrome) {
    if (options.requireChrome) {
      fail("no Chrome found: set CHROME_BIN or install google-chrome");
    }
    process.stdout.write(
      "NOMOS_VIEWER_SMOKE SKIP no Chrome found (set CHROME_BIN to run the browser lane)\n",
    );
    return { skipped: true };
  }

  const server = await serve(dist);
  const started = Date.now();
  const receipt = {
    receipt: "nomos-viewer-smoke/1",
    generated_by: "apps/nomos-viewer/smoke/smoke.mjs",
    commit: gitHead(),
    node: process.version,
    chrome: { binary: chrome.binary, source: chrome.source },
    server: { origin: server.origin, root: dist },
    route: route.legs.map((leg) => ({
      area: leg.area,
      gate: leg.gate,
      scenario: leg.scenario,
      keys: leg.keys.length,
      moves: leg.moves,
      cost: leg.cost,
    })),
    expected: { areas: route.areas, moves: route.moves, cost: route.cost, summary: route.summary },
    outcome: "fail",
  };

  let browser = null;
  try {
    for (const [name, flags] of Object.entries(FLAG_SETS)) {
      if (browser) browser.kill();
      browser = await launch({ binary: chrome.binary, flags });
      receipt.chrome.flag_set = name;
      receipt.chrome.product = browser.version.Browser;
      receipt.chrome.revision = browser.version["WebKit-Version"];
      receipt.flags = browser.argv;
      const attempt = await drive({ browser, server, route, collection, out, receipt });
      if (attempt.webgl) return { ...attempt, receipt };
      if (name === "B") fail(`no WebGL context with either flag set: ${attempt.reason}`);
      process.stdout.write(
        `NOMOS_VIEWER_SMOKE RETRY flag set ${name} gave no WebGL context (${attempt.reason})\n`,
      );
    }
    return fail("unreachable");
  } finally {
    receipt.duration_ms = Date.now() - started;
    browser?.kill();
    await server.close();
    receipt.requests = server.requests;
    receipt.request_count = server.requests.length;
    writeFileSync(join(out, "receipt.json"), `${JSON.stringify(receipt, null, 2)}\n`);
  }
}

async function drive({ browser, server, route, collection, out, receipt }) {
  const page = await connect(browser.pageUrl);
  const consoleErrors = [];
  const exceptions = [];
  const logErrors = [];
  const failures = [];
  const requests = [];
  let probing = false;
  const probeLog = [];

  page.on("Runtime.consoleAPICalled", (params) => {
    if (params.type !== "error") return;
    const text = params.args.map((one) => one.value ?? one.description ?? one.type).join(" ");
    (probing ? probeLog : consoleErrors).push(text);
  });
  page.on("Runtime.exceptionThrown", (params) => {
    const details = params.exceptionDetails;
    (probing ? probeLog : exceptions).push(
      details.exception?.description ?? details.text ?? "unknown exception",
    );
  });
  page.on("Log.entryAdded", (params) => {
    if (params.entry.level !== "error") return;
    (probing ? probeLog : logErrors).push(`${params.entry.source}: ${params.entry.text}`);
  });
  page.on("Network.requestWillBeSent", (params) => {
    requests.push({ url: params.request.url, type: params.type });
  });
  page.on("Network.loadingFailed", (params) => {
    (probing ? probeLog : failures).push(`${params.type}: ${params.errorText}`);
  });

  await page.send("Runtime.enable");
  await page.send("Log.enable");
  await page.send("Page.enable");
  await page.send("Network.enable");

  const loaded = page.once("Page.loadEventFired");
  await page.send("Page.navigate", { url: `${server.origin}/` });
  await loaded;

  const dataset = () =>
    page.evaluate("JSON.stringify({ ...document.documentElement.dataset })").then(JSON.parse);

  // A decoder refusal is rendered into the page rather than thrown at the
  // console, so this is what catches it.
  const guard = async () => {
    const state = await dataset();
    if (state.error) {
      const text = await page.evaluate("document.querySelector('#failure').textContent");
      fail(`the viewer refused its artifacts: ${state.error} ${text}`);
    }
    if (exceptions.length > 0) fail(`uncaught exception: ${exceptions[0]}`);
    if (consoleErrors.length > 0) fail(`console error: ${consoleErrors[0]}`);
    if (logErrors.length > 0) fail(`log error: ${logErrors[0]}`);
    if (failures.length > 0) fail(`request failed: ${failures[0]}`);
    return state;
  };

  try {
    await page.until(dataset, (state) => state.ready === "true" || state.error, { wait: 20_000 });
  } catch (error) {
    // A page that never became ready has usually said why; report that rather
    // than the timeout that followed it.
    await guard();
    fail(`${error.message} (no console error was reported)`);
  }
  await guard();

  const webgl = await page.evaluate(`(() => {
    const canvas = document.querySelector('#frame canvas');
    if (!canvas) return { ok: false, reason: 'no canvas' };
    const context = canvas.getContext('webgl2') || canvas.getContext('webgl');
    if (!context) return { ok: false, reason: 'no context' };
    const info = context.getExtension('WEBGL_debug_renderer_info');
    return {
      ok: true,
      context: context instanceof WebGL2RenderingContext ? 'webgl2' : 'webgl',
      vendor: info ? context.getParameter(info.UNMASKED_VENDOR_WEBGL) : context.getParameter(context.VENDOR),
      renderer: info ? context.getParameter(info.UNMASKED_RENDERER_WEBGL) : context.getParameter(context.RENDERER),
    };
  })()`);
  if (!webgl.ok) {
    page.close();
    return { webgl: false, reason: webgl.reason };
  }
  receipt.webgl = webgl;

  const press = async (name) => {
    const key = KEYS[name];
    if (!key) fail(`the solver asked for a key the lane has no code for: ${name}`);
    await page.send("Input.dispatchKeyEvent", {
      type: key.text ? "keyDown" : "rawKeyDown",
      code: key.code,
      key: key.key,
      windowsVirtualKeyCode: key.vk,
      nativeVirtualKeyCode: key.vk,
      ...(key.text ? { text: key.text } : {}),
    });
    await page.send("Input.dispatchKeyEvent", {
      type: "keyUp",
      code: key.code,
      key: key.key,
      windowsVirtualKeyCode: key.vk,
      nativeVirtualKeyCode: key.vk,
    });
  };

  const screenshots = [];
  for (const leg of route.legs) {
    const before = await guard();
    if (before.area !== leg.area) fail(`expected to be in ${leg.area}, the page says ${before.area}`);

    // Every key but the last, then the evidence frame, then the way out.
    for (const name of leg.keys.slice(0, -1)) {
      const previous = await dataset();
      await press(name);
      const after = await page.until(
        dataset,
        (state) => state.moves !== previous.moves || state.scenario !== previous.scenario,
        { wait: 5_000 },
      );
      if (after.error) await guard();
    }

    const shot = await page.send("Page.captureScreenshot", { format: "png" });
    const bytes = Buffer.from(shot.data, "base64");
    if (bytes.length < 1024 || bytes.subarray(0, 8).toString("hex") !== "89504e470d0a1a0a") {
      fail(`the screenshot for ${leg.area} is not a PNG of any size`);
    }
    const file = `screenshots/${leg.area}.png`;
    writeFileSync(join(out, file), bytes);
    screenshots.push({ area: leg.area, file, bytes: bytes.length });

    await press(leg.keys.at(-1));
    const after = await page.until(
      dataset,
      (state) => state.area !== leg.area || state.completed === "true",
      { wait: 5_000 },
    );
    await guard();
    if (Number(after.moves) !== leg.moves) {
      fail(`after ${leg.area} the page counts ${after.moves} moves, the solver expected ${leg.moves}`);
    }
    if (Number(after.cost) !== leg.cost) {
      fail(`after ${leg.area} the page counts ${after.cost} cost, the solver expected ${leg.cost}`);
    }
    if (leg.to !== null && after.area !== leg.to) {
      fail(`leaving ${leg.area} should arrive in ${leg.to}, the page says ${after.area}`);
    }
  }

  const final = await guard();
  if (final.completed !== "true") fail(`the run did not complete: ${JSON.stringify(final)}`);
  if (Number(final.areasCleared) !== route.areas) {
    fail(`the page cleared ${final.areasCleared} areas, the collection declares ${route.areas}`);
  }
  if (final.message !== "Escaped the gaol") fail(`the final message is \`${final.message}\``);
  const summary = await page.evaluate("document.querySelector('#completion-summary').textContent");
  if (summary !== route.summary) fail(`the summary reads \`${summary}\`, expected \`${route.summary}\``);
  if (screenshots.length !== route.areas) fail("one screenshot per area was not captured");

  // The offline claim, measured rather than asserted: with host resolution
  // mapped to NOTFOUND, a fetch to a real-looking origin must fail. A probe that
  // succeeded would mean the rule was not in force and the empty
  // external-request list proved nothing.
  probing = true;
  const probe = await page.evaluate(
    `fetch(${JSON.stringify(PROBE)}).then((response) => 'resolved ' + response.status).catch((error) => String(error))`,
  );
  probing = false;
  if (/^resolved/.test(probe)) fail(`the negative control reached ${PROBE}: ${probe}`);

  const external = requests.filter((one) => !one.url.startsWith(server.origin) && one.url !== PROBE);
  if (external.length > 0) {
    fail(`the page requested ${external.length} external URL(s): ${external[0].url}`);
  }

  Object.assign(receipt, {
    navigation: { url: `${server.origin}/`, load_event: true },
    browser_requests: requests,
    external_requests: external,
    negative_control: { probe: PROBE, outcome: probe, log_entries_during_probe: probeLog.length },
    console_errors: consoleErrors,
    exceptions,
    log_errors: logErrors,
    screenshots,
    result: {
      areas_cleared: Number(final.areasCleared),
      moves: Number(final.moves),
      cost: Number(final.cost),
      message: final.message,
      summary,
    },
    outcome: "pass",
  });

  page.close();
  return { webgl: true, screenshots, final, summary };
}

const options = parseArguments(process.argv.slice(2));
try {
  const result = await run(options);
  if (!result.skipped) {
    process.stdout.write(
      `NOMOS_VIEWER_SMOKE PASS areas=${result.receipt.result.areas_cleared} ` +
        `moves=${result.receipt.result.moves} cost=${result.receipt.result.cost} ` +
        `requests=${result.receipt.request_count} external=0 ` +
        `chrome=${result.receipt.chrome.product} flags=${result.receipt.chrome.flag_set}\n`,
    );
  }
} catch (error) {
  process.stderr.write(`NOMOS_VIEWER_SMOKE FAIL ${error.message}\n`);
  if (!(error instanceof Failure)) process.stderr.write(`${error.stack}\n`);
  process.exitCode = 1;
}
