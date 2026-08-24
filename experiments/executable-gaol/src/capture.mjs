import { mkdirSync, readFileSync, writeFileSync } from "node:fs";
import { join } from "node:path";
import { renderSvg } from "./render-core.mjs";

const [planPath, outputDir] = process.argv.slice(2);
if (!planPath || !outputDir) throw new Error("usage: capture.mjs <plan> <output-dir>");
const plan = JSON.parse(readFileSync(planPath, "utf8"));
mkdirSync(outputDir, { recursive: true });

const frames = plan.scenarios.map((scenario) => {
  const svg = renderSvg(plan, scenario.id, false);
  writeFileSync(join(outputDir, `${scenario.id}.svg`), `${svg}\n`);
  return { scenario, svg };
});

writeFileSync(join(outputDir, "forensic.svg"), `${renderSvg(plan, plan.presentation.forensicScenario, true)}\n`);

const sheetFrames = frames.slice(0, 4);
const sheetWidth = plan.camera.width * 2;
const sheetHeight = plan.camera.height * 2;
const nested = sheetFrames.map(({ svg }, index) => {
  const x = (index % 2) * plan.camera.width;
  const y = Math.floor(index / 2) * plan.camera.height;
  return `<svg x="${x}" y="${y}" width="${plan.camera.width}" height="${plan.camera.height}" viewBox="0 0 ${plan.camera.width} ${plan.camera.height}">${svg}</svg>`;
}).join("");
const sheet = `<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 ${sheetWidth} ${sheetHeight}" width="${sheetWidth}" height="${sheetHeight}">${nested}</svg>\n`;
writeFileSync(join(outputDir, "contact-sheet.svg"), sheet);
console.log(`Captured ${frames.length} deterministic frames -> ${join(outputDir, "contact-sheet.svg")}`);
