import { mkdirSync, readFileSync, writeFileSync } from "node:fs";
import { join } from "node:path";
import { camera, renderSvg } from "./render-core.mjs";

// The scenario the forensic overlay renders.
//
// This was `area.json`'s `forensicScenario`, authored identically in all four
// areas and read by this file alone — the ownership audit classified it as a
// test fixture, not content. It is one declared capture-tooling constant now,
// so adding an area does not require copying it a fifth time.
const FORENSIC_SCENARIO = "03-breached-unsealed";

const [planPath, outputDir] = process.argv.slice(2);
if (!planPath || !outputDir) throw new Error("usage: capture.mjs <plan> <output-dir>");
const plan = JSON.parse(readFileSync(planPath, "utf8"));
mkdirSync(outputDir, { recursive: true });

const frames = plan.scenarios.map((scenario) => {
  const svg = renderSvg(plan, scenario.id, false);
  writeFileSync(join(outputDir, `${scenario.id}.svg`), `${svg}\n`);
  return { scenario, svg };
});

writeFileSync(join(outputDir, "forensic.svg"), `${renderSvg(plan, FORENSIC_SCENARIO, true)}\n`);

const sheetFrames = frames.slice(0, 4);
const sheetWidth = camera.width * 2;
const sheetHeight = camera.height * 2;
const nested = sheetFrames.map(({ svg }, index) => {
  const x = (index % 2) * camera.width;
  const y = Math.floor(index / 2) * camera.height;
  return `<svg x="${x}" y="${y}" width="${camera.width}" height="${camera.height}" viewBox="0 0 ${camera.width} ${camera.height}">${svg}</svg>`;
}).join("");
const sheet = `<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 ${sheetWidth} ${sheetHeight}" width="${sheetWidth}" height="${sheetHeight}">${nested}</svg>\n`;
writeFileSync(join(outputDir, "contact-sheet.svg"), sheet);
console.log(`Captured ${frames.length} deterministic frames -> ${join(outputDir, "contact-sheet.svg")}`);
