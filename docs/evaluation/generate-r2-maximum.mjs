#!/usr/bin/env node

import { fileURLToPath } from "node:url";
import { dirname, join } from "node:path";
import { writeFileSync } from "node:fs";

export function generateMaximumBytes() {
  const cells = (layer) => {
    const result = [];
    for (let y = 0; y < 32; y += 1) {
      for (let x = 0; x < 32; x += 1) {
        if ((x + 32 * y + layer) % 2 === 0) result.push({ x, y });
      }
    }
    return result;
  };
  const roles = [
    "calm_ground",
    "traversable_route",
    "structure_footprint",
    "calm_ground",
    "traversable_route",
    "structure_footprint",
    "calm_ground",
    "traversable_route",
  ];
  const actorId = (index) => `a${"0".repeat(61)}${String(index).padStart(2, "0")}`;
  const document = {
    actions: Array.from({ length: 128 }, (_, index) => ({
      availability: index % 2 === 0 ? "enabled" : "disabled",
      id: `q${"0".repeat(60)}${String(index).padStart(3, "0")}`,
      target_actor: actorId(index % 64),
    })),
    actors: Array.from({ length: 64 }, (_, index) => ({
      cell: { x: index % 32, y: Math.floor(index / 32), z: 0 },
      controlled: (index & 1) !== 0,
      hostile: (index & 2) !== 0,
      id: actorId(index),
      life_state: Math.floor(index / 8) % 2 === 0 ? "living" : "dead",
      protected: (index & 4) !== 0,
    })),
    crop: { height: 32, width: 32 },
    scene: { id: `s${"0".repeat(63)}` },
    schema: "nomos.observed_scene@1",
    terrain_layers: Array.from({ length: 8 }, (_, index) => ({
      cells: cells(index),
      id: `l${"0".repeat(62)}${index}`,
      role: roles[index],
    })),
  };
  return Buffer.from(JSON.stringify(document), "utf8");
}

const here = dirname(fileURLToPath(import.meta.url));
const output = join(here, "..", "..", "fixtures", "r2", "maximum-observed-scene.json");
if (process.argv[1] === fileURLToPath(import.meta.url)) {
  writeFileSync(output, generateMaximumBytes(), { flag: "w" });
}
