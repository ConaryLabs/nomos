import { createHash } from "node:crypto";
import { mkdirSync, readFileSync, writeFileSync } from "node:fs";
import { basename, dirname, resolve } from "node:path";
import { fileURLToPath } from "node:url";

import { discoverChrome, launchChrome, closeChrome } from "./chrome.mjs";
import { connect } from "./cdp.mjs";
import { serve } from "./server.mjs";

const sha256 = (bytes) => createHash("sha256").update(bytes).digest("hex");
const sleep = (milliseconds) => new Promise((resolveSleep) => setTimeout(resolveSleep, milliseconds));
const requireEvidence = (condition, message) => {
  if (!condition) throw new Error(`browser launch evidence: ${message}`);
};
const exactKeys = (value, keys, label) => {
  requireEvidence(value && typeof value === "object" && !Array.isArray(value), `${label} is not an object`);
  requireEvidence(
    JSON.stringify(Object.keys(value).sort()) === JSON.stringify([...keys].sort()),
    `${label} fields differ`,
  );
};

const COUNT_KEYS = [
  "actions",
  "actors",
  "controlled_markers",
  "hostile_outlines",
  "protection_rings",
  "terrain_cells",
  "terrain_layers",
];

const parseArguments = (argv) => {
  const options = { samples: 10 };
  for (let index = 0; index < argv.length; index += 2) {
    const flag = argv[index];
    const value = argv[index + 1];
    if (!value || !["--dist", "--out", "--samples"].includes(flag)) throw new Error(`invalid argument ${flag}`);
    options[flag.slice(2)] = flag === "--samples" ? Number(value) : value;
  }
  if (!options.dist || !options.out || !Number.isInteger(options.samples) || options.samples < 1 || options.samples > 10) {
    throw new Error("usage: smoke.mjs --dist <dir> --out <empty-dir> --samples <1..10>");
  }
  return options;
};

const consequenceCounts = (plan) => ({
  actions: plan.actions.length,
  actors: plan.actors.length,
  controlled_markers: plan.actors.filter((row) => row.controlled_marker === "present").length,
  hostile_outlines: plan.actors.filter((row) => row.hostile_outline === "present").length,
  protection_rings: plan.actors.filter((row) => row.protection_ring === "present").length,
  terrain_cells: plan.terrain_layers.reduce((sum, row) => sum + row.cells.length, 0),
  terrain_layers: plan.terrain_layers.length,
});

const planRows = (dist) => readFileSync(resolve(dist, "ARTIFACTS.sha256"), "utf8")
  .trimEnd()
  .split("\n")
  .slice(1)
  .map((line) => {
    const [digest, bytes, path] = line.split("\t");
    const artifact = readFileSync(resolve(dist, path));
    if (artifact.length !== Number(bytes) || sha256(artifact) !== digest) {
      throw new Error(`smoke plan bytes disagree with integrity row: ${path}`);
    }
    return { bytes: Number(bytes), expected_counts: consequenceCounts(JSON.parse(artifact)), path, sha256: digest };
  });

const openPage = async (chrome, width = 1280, height = 720) => {
  const client = await connect(chrome.websocket);
  await Promise.all([
    client.send("Page.enable"),
    client.send("Runtime.enable"),
    client.send("Network.enable"),
  ]);
  await client.send("Network.setCacheDisabled", { cacheDisabled: true });
  await client.send("Emulation.setDeviceMetricsOverride", {
    deviceScaleFactor: 1,
    height,
    mobile: false,
    width,
  });
  return client;
};

const sample = async ({ browser, index, launchOrdinal, output, plan, port, sampleIndex }) => {
  const chrome = await launchChrome(browser);
  let client;
  let known;
  let record;
  const requests = [];
  const consoleErrors = [];
  const exceptions = [];
  try {
    client = await openPage(chrome);
    client.on("Runtime.consoleAPICalled", ({ type, args }) => {
      if (type === "error") consoleErrors.push(args.map((value) => value.value ?? value.description).join(" "));
    });
    client.on("Runtime.exceptionThrown", ({ exceptionDetails }) => exceptions.push(exceptionDetails.text));
    const negative = await client.evaluate('fetch("https://nomos.invalid/").then(()=>"unexpected").catch(()=>"blocked")');
    if (negative !== "blocked") throw new Error("host-resolver negative control did not fail");
    client.on("Network.requestWillBeSent", ({ request }) => requests.push(request.url));
    await client.send("Runtime.addBinding", { name: "nomosObservedFrame" });
    const version = await client.send("Browser.getVersion");
    const binding = client.once("Runtime.bindingCalled", 20_000);
    const started = process.hrtime.bigint();
    await client.send("Page.navigate", { url: `http://localhost:${port}/?scene=${index}` });
    const event = await binding;
    const elapsed = process.hrtime.bigint() - started;
    const payload = JSON.parse(event.payload);
    const webgl2 = await client.evaluate('document.querySelector("canvas")?.getContext("webgl2") !== null');
    if (!webgl2) throw new Error("WebGL2 is unavailable");
    if (payload.plan_sha256 !== plan.sha256) throw new Error("frame names the wrong plan digest");
    if (payload.viewport.width !== 1280 || payload.viewport.height !== 720) throw new Error("frame viewport differs");
    if (JSON.stringify(payload.consequence_counts) !== JSON.stringify(plan.expected_counts)) {
      throw new Error("frame consequence counts disagree with the compiled plan");
    }
    if (consoleErrors.length || exceptions.length) throw new Error(`browser errors: ${JSON.stringify({ consoleErrors, exceptions })}`);
    const external = requests.filter((url) => !url.startsWith(`http://localhost:${port}/`));
    if (external.length) throw new Error(`external requests: ${JSON.stringify(external)}`);
    let screenshot = null;
    if (sampleIndex === 0) {
      const captured = await client.send("Page.captureScreenshot", { format: "png", fromSurface: true });
      const bytes = Buffer.from(captured.data, "base64");
      screenshot = `scene_${index + 1}.png`;
      writeFileSync(resolve(output, screenshot), bytes);
    }
    known = process.hrtime.bigint();
    record = {
      browser_product: version.product,
      cache_disabled: true,
      chrome_flags: chrome.flags,
      closure: null,
      console_errors: [...consoleErrors],
      elapsed_ns: elapsed.toString(),
      exceptions: [...exceptions],
      frame: payload,
      launch_ordinal: launchOrdinal,
      network_negative_control: negative,
      profile: chrome.profile,
      requests: [...requests],
      sample_ordinal: sampleIndex,
      scene_ordinal: index,
      screenshot,
      webgl2,
    };
  } finally {
    if (!known) known = process.hrtime.bigint();
    try { await client?.close(); } catch {}
    const closure = await closeChrome(chrome);
    closure.after_result_ms = Number(process.hrtime.bigint() - known) / 1_000_000;
    if (closure.after_result_ms > 2_000) throw new Error(`Chrome closure exceeded 2000 ms: ${closure.after_result_ms}`);
    if (record) record.closure = closure;
  }
  return record;
};

export const verifyLaunchEvidence = ({ launches, plans, port, samplesPerScene }) => {
  requireEvidence(Number.isInteger(port) && port > 0, "local server port is invalid");
  requireEvidence(Number.isInteger(samplesPerScene) && samplesPerScene > 0, "sample count is invalid");
  requireEvidence(Array.isArray(plans) && plans.length === 2, "exactly two plans are required");
  requireEvidence(
    Array.isArray(launches) && launches.length === plans.length * samplesPerScene,
    "launch count differs",
  );
  const profiles = new Set();
  const localPrefix = `http://localhost:${port}/`;
  launches.forEach((launch, launchOrdinal) => {
    exactKeys(launch, [
      "browser_product", "cache_disabled", "chrome_flags", "closure", "console_errors",
      "elapsed_ns", "exceptions", "frame", "launch_ordinal", "network_negative_control",
      "profile", "requests", "sample_ordinal", "scene_ordinal", "screenshot", "webgl2",
    ], `launch ${launchOrdinal}`);
    const sceneOrdinal = Math.floor(launchOrdinal / samplesPerScene);
    const sampleOrdinal = launchOrdinal % samplesPerScene;
    const plan = plans[sceneOrdinal];
    requireEvidence(launch.launch_ordinal === launchOrdinal, `launch ${launchOrdinal} ordinal differs`);
    requireEvidence(launch.scene_ordinal === sceneOrdinal, `launch ${launchOrdinal} scene ordinal differs`);
    requireEvidence(launch.sample_ordinal === sampleOrdinal, `launch ${launchOrdinal} sample ordinal differs`);
    requireEvidence(typeof launch.browser_product === "string" && launch.browser_product.length > 0, `launch ${launchOrdinal} browser product is absent`);
    requireEvidence(launch.cache_disabled === true, `launch ${launchOrdinal} did not disable cache`);
    requireEvidence(launch.network_negative_control === "blocked", `launch ${launchOrdinal} network negative control differs`);
    requireEvidence(launch.webgl2 === true, `launch ${launchOrdinal} did not prove WebGL2`);
    requireEvidence(/^\d+$/.test(launch.elapsed_ns) && BigInt(launch.elapsed_ns) > 0n, `launch ${launchOrdinal} elapsed time is invalid`);
    requireEvidence(Array.isArray(launch.console_errors) && launch.console_errors.length === 0, `launch ${launchOrdinal} has console errors`);
    requireEvidence(Array.isArray(launch.exceptions) && launch.exceptions.length === 0, `launch ${launchOrdinal} has exceptions`);
    requireEvidence(Array.isArray(launch.requests) && launch.requests.every((url) => typeof url === "string"), `launch ${launchOrdinal} requests are invalid`);
    requireEvidence(
      launch.requests.includes(`${localPrefix}?scene=${sceneOrdinal}`),
      `launch ${launchOrdinal} navigation request is absent`,
    );
    requireEvidence(launch.requests.every((url) => url.startsWith(localPrefix)), `launch ${launchOrdinal} made an external request`);
    requireEvidence(typeof launch.profile === "string" && launch.profile.length > 0, `launch ${launchOrdinal} profile identity is absent`);
    requireEvidence(!profiles.has(launch.profile), `launch ${launchOrdinal} reused a browser profile`);
    profiles.add(launch.profile);
    requireEvidence(Array.isArray(launch.chrome_flags) && launch.chrome_flags.every((flag) => typeof flag === "string"), `launch ${launchOrdinal} Chrome flags are invalid`);
    requireEvidence(launch.chrome_flags.includes("--host-resolver-rules=MAP * ~NOTFOUND, EXCLUDE localhost"), `launch ${launchOrdinal} host resolver control is absent`);
    requireEvidence(
      launch.chrome_flags.filter((flag) => flag.startsWith("--user-data-dir=")).length === 1
        && launch.chrome_flags.includes(`--user-data-dir=${launch.profile}`),
      `launch ${launchOrdinal} profile flag differs`,
    );
    exactKeys(launch.frame, ["consequence_counts", "plan_sha256", "viewport"], `launch ${launchOrdinal} frame`);
    exactKeys(launch.frame.consequence_counts, COUNT_KEYS, `launch ${launchOrdinal} consequence counts`);
    exactKeys(launch.frame.viewport, ["height", "width"], `launch ${launchOrdinal} viewport`);
    requireEvidence(launch.frame.plan_sha256 === plan.sha256, `launch ${launchOrdinal} plan digest differs`);
    requireEvidence(
      COUNT_KEYS.every((key) => launch.frame.consequence_counts[key] === plan.expected_counts[key]),
      `launch ${launchOrdinal} consequence counts differ`,
    );
    requireEvidence(launch.frame.viewport.width === 1280 && launch.frame.viewport.height === 720, `launch ${launchOrdinal} viewport differs`);
    requireEvidence(
      launch.screenshot === (sampleOrdinal === 0 ? `scene_${sceneOrdinal + 1}.png` : null),
      `launch ${launchOrdinal} screenshot identity differs`,
    );
    exactKeys(launch.closure, ["after_result_ms", "duration_ms", "exit_code", "signal"], `launch ${launchOrdinal} closure`);
    requireEvidence(
      Number.isFinite(launch.closure.after_result_ms) && launch.closure.after_result_ms >= 0
        && launch.closure.after_result_ms <= 2_000
        && Number.isFinite(launch.closure.duration_ms) && launch.closure.duration_ms >= 0
        && launch.closure.duration_ms <= 2_000,
      `launch ${launchOrdinal} closure exceeded 2000 ms`,
    );
  });
  return true;
};

export const summarizeDurations = (values) => {
  const sorted = values.map(BigInt).sort((a, b) => (a < b ? -1 : a > b ? 1 : 0));
  const middle = sorted.length / 2;
  const medianNumerator = sorted.length % 2 === 0 ? sorted[middle - 1] + sorted[middle] : sorted[Math.floor(middle)] * 2n;
  const p95 = sorted[Math.ceil(0.95 * sorted.length) - 1];
  return {
    median_denominator: 2,
    median_numerator_ns: medianNumerator.toString(),
    p95_ns: p95.toString(),
  };
};

const contactSheet = async (browser, output, screenshots) => {
  if (screenshots.length !== 2) return null;
  const chrome = await launchChrome(browser, { width: 2560, height: 720 });
  let client;
  let known;
  try {
    client = await openPage(chrome, 2560, 720);
    const tree = await client.send("Page.getFrameTree");
    const images = screenshots.map((path) => readFileSync(resolve(output, path)).toString("base64"));
    const html = `<body style="margin:0;background:#171a1f;display:flex"><img width="1280" height="720" src="data:image/png;base64,${images[0]}"><img width="1280" height="720" src="data:image/png;base64,${images[1]}"></body>`;
    await client.send("Page.setDocumentContent", { frameId: tree.frameTree.frame.id, html });
    await sleep(100);
    const captured = await client.send("Page.captureScreenshot", { format: "png", fromSurface: true });
    const bytes = Buffer.from(captured.data, "base64");
    writeFileSync(resolve(output, "contact-sheet.png"), bytes);
    known = process.hrtime.bigint();
    return { bytes: bytes.length, path: "contact-sheet.png", sha256: sha256(bytes), viewport: { width: 2560, height: 720 } };
  } finally {
    if (!known) known = process.hrtime.bigint();
    try { await client?.close(); } catch {}
    const closure = await closeChrome(chrome);
    closure.after_result_ms = Number(process.hrtime.bigint() - known) / 1_000_000;
    if (closure.after_result_ms > 2_000) throw new Error("contact-sheet browser closure exceeded 2000 ms");
    contactSheet.lastClosure = closure;
  }
};

export const runSmoke = async (options) => {
  const dist = resolve(options.dist);
  const output = resolve(options.out);
  mkdirSync(output, { recursive: false });
  const plans = planRows(dist);
  const browser = await discoverChrome();
  const server = await serve(dist);
  const records = [];
  const closures = [];
  let resultKnown;
  try {
    for (let index = 0; index < plans.length; index += 1) {
      for (let sampleIndex = 0; sampleIndex < options.samples; sampleIndex += 1) {
        const record = await sample({
          browser,
          index,
          launchOrdinal: records.length,
          output,
          plan: plans[index],
          port: server.port,
          sampleIndex,
        });
        records.push(record);
        closures.push({ ...record.closure, kind: "browser_launch", launch_ordinal: record.launch_ordinal });
      }
    }
    resultKnown = process.hrtime.bigint();
  } finally {
    if (!resultKnown) resultKnown = process.hrtime.bigint();
    const serverClosure = await server.close();
    serverClosure.after_result_ms = Number(process.hrtime.bigint() - resultKnown) / 1_000_000;
    closures.push({ ...serverClosure, kind: "server" });
  }
  verifyLaunchEvidence({ launches: records, plans, port: server.port, samplesPerScene: options.samples });
  const perScene = plans.map((plan) => {
    const selected = records.filter((record) => record.frame.plan_sha256 === plan.sha256);
    return { plan, samples_ns: selected.map((record) => record.elapsed_ns), ...summarizeDurations(selected.map((record) => record.elapsed_ns)) };
  });
  const combined = summarizeDurations(records.map((record) => record.elapsed_ns));
  if (perScene.some((scene) => BigInt(scene.p95_ns) > 5_000_000_000n) || BigInt(combined.p95_ns) > 5_000_000_000n) {
    throw new Error("browser timing ceiling exceeded");
  }
  const screenshots = records.map((record) => record.screenshot).filter(Boolean);
  const sheet = await contactSheet(browser, output, screenshots);
  if (sheet) closures.push({ ...contactSheet.lastClosure, kind: "contact_sheet_browser" });
  const receipt = {
    browser: records[0]?.browser_product ?? basename(browser),
    chrome_flags: records[0]?.chrome_flags ?? [],
    closures,
    combined,
    contact_sheet: sheet,
    external_requests: records.flatMap((record) => record.requests).filter((url) => !url.startsWith(`http://localhost:${server.port}/`)),
    launches: records,
    outcome: "pass",
    per_scene: perScene,
    requests: server.requests,
    samples_per_scene: options.samples,
    screenshots: screenshots.map((path) => {
      const bytes = readFileSync(resolve(output, path));
      return { bytes: bytes.length, path, sha256: sha256(bytes), viewport: { width: 1280, height: 720 } };
    }),
  };
  writeFileSync(resolve(output, "receipt.json"), `${JSON.stringify(receipt, null, 2)}\n`);
  return receipt;
};

if (process.argv[1] && resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  runSmoke(parseArguments(process.argv.slice(2))).then((receipt) => {
    process.stdout.write(`NOMOS_OBSERVED_SMOKE PASS scenes=${receipt.per_scene.length} samples=${receipt.per_scene.length * receipt.samples_per_scene} external=0\n`);
  }).catch((error) => {
    process.stderr.write(`${error.stack ?? error}\n`);
    process.exitCode = 1;
  });
}
