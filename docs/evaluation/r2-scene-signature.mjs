import { createHash } from "node:crypto";
import { readFileSync } from "node:fs";
import { resolve } from "node:path";
import { fileURLToPath } from "node:url";

import { encodeCanonical } from "../../apps/nomos-observed-viewer/src/canonical.mjs";

const sha256 = (text) => createHash("sha256").update(text).digest("hex");
const canonicalSort = (rows) => rows.sort((a, b) => encodeCanonical(a).localeCompare(encodeCanonical(b)));

export const sceneSignature = (scene) => {
  const tuples = scene.actors.map((actor) => ({
    cell: actor.cell,
    controlled: actor.controlled,
    hostile: actor.hostile,
    life_state: actor.life_state,
    protected: actor.protected,
  }));
  const encodedTuples = tuples.map(encodeCanonical);
  if (new Set(encodedTuples).size !== tuples.length) throw new Error("proof-scene actor tuples are not unique");
  const actors = canonicalSort(tuples);
  const ordinalById = new Map();
  scene.actors.forEach((actor) => {
    ordinalById.set(actor.id, actors.findIndex((tuple) => encodeCanonical(tuple) === encodeCanonical({
      cell: actor.cell,
      controlled: actor.controlled,
      hostile: actor.hostile,
      life_state: actor.life_state,
      protected: actor.protected,
    })));
  });
  const terrain = canonicalSort(scene.terrain_layers.map((row) => ({ cells: row.cells, role: row.role })));
  const actions = canonicalSort(scene.actions.map((row) => {
    const target_actor_ordinal = ordinalById.get(row.target_actor);
    if (target_actor_ordinal === undefined) throw new Error(`dangling target ${row.target_actor}`);
    return { availability: row.availability, target_actor_ordinal };
  }));
  const axes = { actions, actors, crop: scene.crop, terrain };
  const axis_sha256 = Object.fromEntries(
    Object.entries(axes).map(([name, value]) => [name, sha256(encodeCanonical(value))]),
  );
  const normalized = { actions, actors, crop: scene.crop, terrain };
  return Object.freeze({
    axis_sha256: Object.freeze(axis_sha256),
    normalized,
    sha256: sha256(encodeCanonical(normalized)),
  });
};

export const signatureFromPath = (path) => sceneSignature(JSON.parse(readFileSync(resolve(path), "utf8")));

if (process.argv[1] && resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  if (process.argv.length < 3 || process.argv.length > 4) {
    process.stderr.write("usage: r2-scene-signature.mjs <scene-one.json> [scene-two.json]\n");
    process.exitCode = 2;
  } else {
    const results = process.argv.slice(2).map((path) => ({ path, ...signatureFromPath(path) }));
    if (results.length === 2) {
      const axes = ["crop", "terrain", "actors", "actions"];
      const equal = axes.filter((axis) => results[0].axis_sha256[axis] === results[1].axis_sha256[axis]);
      if (results[0].sha256 === results[1].sha256 || equal.length) {
        process.stderr.write(`r2 scene signatures do not differ on: ${equal.join(",") || "normalized document"}\n`);
        process.exitCode = 1;
      } else {
        process.stdout.write(`${JSON.stringify({ outcome: "pass", scenes: results }, null, 2)}\n`);
      }
    } else {
      process.stdout.write(`${JSON.stringify({ outcome: "pass", scenes: results }, null, 2)}\n`);
    }
  }
}
