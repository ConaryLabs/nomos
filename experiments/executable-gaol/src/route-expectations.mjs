// Generates the content-side route fixture consumed by the native runtime
// tests. The route solver derives its walk from compiled artifacts, so adding
// an area changes content and this generated fixture, never a crate test.

import { readFileSync, writeFileSync } from "node:fs";
import { join } from "node:path";

import { decodeCollection, decodePlan } from "../../../apps/nomos-viewer/src/plan.mjs";
import { solveRoute } from "../../../apps/nomos-viewer/smoke/route.mjs";

const [collectionPath, plansDirectory, outputPath] = process.argv.slice(2);
if (!collectionPath || !plansDirectory || !outputPath) {
  process.stderr.write(
    "usage: node route-expectations.mjs <areas.json> <plans-directory> <output.json>\n",
  );
  process.exit(64);
}

const readJson = (path) => JSON.parse(readFileSync(path, "utf8"));
const collection = decodeCollection(readJson(collectionPath), collectionPath);
const plans = new Map(
  collection.areas.map((area) => {
    const path = join(plansDirectory, area.id, "rendering-plan.json");
    return [area.id, decodePlan(readJson(path), path)];
  }),
);
const solved = solveRoute(collection, plans);
const key = Object.freeze({
  ArrowUp: "^",
  ArrowDown: "v",
  ArrowLeft: "<",
  ArrowRight: ">",
  KeyE: "*",
});
const document = {
  expected: {
    areas: solved.areas,
    commands: solved.keys.length,
    moves: solved.moves,
    traversal_cost: solved.cost,
  },
  route: solved.legs.map((leg) => ({
    area: leg.area,
    keys: leg.keys
      .map((one) => {
        if (!key[one]) throw new Error(`the route solver emitted unknown key ${one}`);
        return key[one];
      })
      .join(""),
  })),
};

writeFileSync(outputPath, `${JSON.stringify(document)}\n`);
