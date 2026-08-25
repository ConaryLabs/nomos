import { mkdirSync, readFileSync, writeFileSync } from "node:fs";
import { join } from "node:path";
import { camera, renderSvg } from "./render-core.mjs";

const [collectionPath, areasDir, outputDir] = process.argv.slice(2);
if (!collectionPath || !areasDir || !outputDir) {
  throw new Error("usage: capture-collection.mjs <collection> <areas-dir> <output-dir>");
}

const collection = JSON.parse(readFileSync(collectionPath, "utf8"));
const rows = collection.areas.map((area) => {
  const plan = JSON.parse(readFileSync(join(areasDir, area.id, "rendering-plan.json"), "utf8"));
  return [plan.scenarios[0], plan.scenarios[2]].map((scenario) => ({ plan, scenario }));
});
const sheetWidth = camera.width * 2;
const sheetHeight = camera.height * rows.length;
const nested = rows.flatMap((row, y) => row.map(({ plan, scenario }, x) => {
  const svg = renderSvg(plan, scenario.id, false);
  return `<svg x="${x * camera.width}" y="${y * camera.height}" width="${camera.width}" height="${camera.height}" viewBox="0 0 ${camera.width} ${camera.height}">${svg}</svg>`;
})).join("");

mkdirSync(outputDir, { recursive: true });
const outputPath = join(outputDir, "contact-sheet.svg");
writeFileSync(outputPath, `<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 ${sheetWidth} ${sheetHeight}" width="${sheetWidth}" height="${sheetHeight}">${nested}</svg>\n`);
console.log(`Captured ${collection.areas.length} areas -> ${outputPath}`);
