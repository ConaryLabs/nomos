import { CATALOG } from "./catalog.mjs";
import { decodePlanBytes, reject } from "./plan.mjs";
import { renderView } from "./render.mjs";
import { mountControls } from "./ui.mjs";

const digest = async (bytes) =>
  [...new Uint8Array(await crypto.subtle.digest("SHA-256", bytes))]
    .map((value) => value.toString(16).padStart(2, "0"))
    .join("");

const fetchBytes = async (path) => {
  let response;
  try {
    response = await fetch(path, { cache: "no-store" });
  } catch {
    throw reject(path, "OV0101", "$", "artifact fetch failed");
  }
  if (!response.ok) throw reject(path, "OV0101", "$", `artifact fetch returned ${response.status}`);
  return new Uint8Array(await response.arrayBuffer());
};

export const parseIntegrity = (text, artifact = "ARTIFACTS.sha256") => {
  if (!text.endsWith("\n") || text.slice(0, -1).includes("\r") || text.slice(0, -1).includes("\n\n")) {
    throw reject(artifact, "OV0101", "$", "integrity index line endings are invalid");
  }
  const lines = text.slice(0, -1).split("\n");
  if (lines.length < 2 || lines.length > 3) throw reject(artifact, "OV0101", "$", "integrity index row count is invalid");
  const catalog = /^catalog_sha256\t([0-9a-f]{64})$/.exec(lines[0]);
  if (!catalog) throw reject(artifact, "OV0101", "$", "catalog integrity row is invalid");
  const plans = lines.slice(1).map((line) => {
    const match = /^([0-9a-f]{64})\t([1-9][0-9]*)\t(plans\/scene_(?:one|two)\.json)$/.exec(line);
    if (!match) throw reject(artifact, "OV0101", "$", "plan integrity row is invalid");
    return { sha256: match[1], bytes: Number(match[2]), path: match[3] };
  });
  if (plans.some((row, index) => index > 0 && row.path <= plans[index - 1].path)) {
    throw reject(artifact, "OV0101", "$", "plan integrity rows are not path-sorted");
  }
  return Object.freeze({ catalog_sha256: catalog[1], plans: Object.freeze(plans) });
};

const verified = async (entry) => {
  const bytes = await fetchBytes(entry.path);
  if (bytes.length !== entry.bytes || (await digest(bytes)) !== entry.sha256) {
    throw reject(entry.path, "OV0101", "$", "artifact bytes disagree with integrity index");
  }
  return bytes;
};

const frameNotice = (entry, counts) => {
  requestAnimationFrame(() => {
    const receipt = {
      consequence_counts: counts,
      plan_sha256: entry.sha256,
      viewport: { height: CATALOG.viewport.height, width: CATALOG.viewport.width },
    };
    globalThis.__NOMOS_OBSERVED_FRAME__ = receipt;
    if (typeof globalThis.nomosObservedFrame === "function") {
      globalThis.nomosObservedFrame(JSON.stringify(receipt));
    }
  });
};

export const boot = async (document) => {
  const THREE = await import("../vendor/three/three.module.min.js");
  document.documentElement.style.background = CATALOG.renderer.clear_color;
  Object.assign(document.body.style, { margin: "0", overflow: "hidden" });
  const integrityBytes = await fetchBytes("ARTIFACTS.sha256");
  let integrityText;
  try {
    integrityText = new TextDecoder("utf-8", { fatal: true }).decode(integrityBytes);
  } catch {
    throw reject("ARTIFACTS.sha256", "OV0101", "$", "integrity index is not UTF-8");
  }
  const integrity = parseIntegrity(integrityText);
  const catalogBytes = await fetchBytes("src/catalog.mjs");
  if ((await digest(catalogBytes)) !== integrity.catalog_sha256) {
    throw reject("src/catalog.mjs", "OV0101", "$", "catalog digest disagrees with integrity index");
  }
  const plans = await Promise.all(integrity.plans.map(async (entry) => ({
    entry,
    view: decodePlanBytes(await verified(entry), entry.path),
  })));
  const requested = Number(new URL(globalThis.location.href).searchParams.get("scene") ?? "0");
  let selected = Number.isInteger(requested) && requested >= 0 && requested < plans.length ? requested : 0;
  const stage = document.querySelector("main");
  let current = null;
  const draw = (index) => {
    selected = index;
    current?.renderer.dispose();
    current = renderView(THREE, stage, plans[index].view);
    frameNotice(plans[index].entry, current.counts);
    return current;
  };
  draw(selected);
  mountControls(document, plans.length, selected, (index) => {
    const next = new URL(globalThis.location.href);
    next.searchParams.set("scene", String(index));
    globalThis.history.replaceState(null, "", next);
    globalThis.location.reload();
  });
  return Object.freeze({ integrity, plans: plans.length, selected });
};
