import { createHash } from "node:crypto";
import { mkdirSync, readFileSync, writeFileSync } from "node:fs";
import { basename, dirname, resolve } from "node:path";
import { fileURLToPath } from "node:url";

import { discoverChrome, launchChrome, closeChrome } from "./chrome.mjs";
import { connect } from "./cdp.mjs";
import { serve } from "./server.mjs";

const sha256 = (bytes) => createHash("sha256").update(bytes).digest("hex");
const sleep = (milliseconds) => new Promise((resolveSleep) => setTimeout(resolveSleep, milliseconds));

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

const sample = async ({ browser, index, output, plan, port, sampleIndex }) => {
  const chrome = await launchChrome(browser);
  let client;
  let known;
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
    return {
      browser_requests: requests,
      browser_product: version.product,
      chrome_flags: chrome.flags,
      consequence_counts: payload.consequence_counts,
      elapsed_ns: elapsed.toString(),
      plan_sha256: plan.sha256,
      screenshot,
    };
  } finally {
    if (!known) known = process.hrtime.bigint();
    try { await client?.close(); } catch {}
    const closure = await closeChrome(chrome);
    closure.after_result_ms = Number(process.hrtime.bigint() - known) / 1_000_000;
    if (closure.after_result_ms > 2_000) throw new Error(`Chrome closure exceeded 2000 ms: ${closure.after_result_ms}`);
    sample.lastClosure = closure;
  }
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
        records.push(await sample({ browser, index, output, plan: plans[index], port: server.port, sampleIndex }));
        closures.push(sample.lastClosure);
      }
    }
    resultKnown = process.hrtime.bigint();
  } finally {
    if (!resultKnown) resultKnown = process.hrtime.bigint();
    const serverClosure = await server.close();
    serverClosure.after_result_ms = Number(process.hrtime.bigint() - resultKnown) / 1_000_000;
    closures.push(serverClosure);
  }
  const perScene = plans.map((plan) => {
    const selected = records.filter((record) => record.plan_sha256 === plan.sha256);
    return { plan, samples_ns: selected.map((record) => record.elapsed_ns), ...summarizeDurations(selected.map((record) => record.elapsed_ns)) };
  });
  const combined = summarizeDurations(records.map((record) => record.elapsed_ns));
  if (perScene.some((scene) => BigInt(scene.p95_ns) > 5_000_000_000n) || BigInt(combined.p95_ns) > 5_000_000_000n) {
    throw new Error("browser timing ceiling exceeded");
  }
  const screenshots = records.map((record) => record.screenshot).filter(Boolean);
  const sheet = await contactSheet(browser, output, screenshots);
  if (sheet) closures.push(contactSheet.lastClosure);
  const receipt = {
    browser: records[0]?.browser_product ?? basename(browser),
    chrome_flags: records[0]?.chrome_flags ?? [],
    closures,
    combined,
    contact_sheet: sheet,
    external_requests: records.flatMap((record) => record.browser_requests).filter((url) => !url.startsWith(`http://localhost:${server.port}/`)),
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
