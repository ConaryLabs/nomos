import { strict as assert } from "node:assert";
import { createHash } from "node:crypto";
import { readFileSync } from "node:fs";
import { fileURLToPath } from "node:url";
import { dirname, join } from "node:path";

import { generateMaximumBytes } from "./generate-r2-maximum.mjs";

const expectedLength = 98421;
const expectedDigest = "fe332f711437dab15e4d1315cc3ca57dba6521350ff673941e77feb414585909";
const digest = (bytes) => createHash("sha256").update(bytes).digest("hex");
const here = dirname(fileURLToPath(import.meta.url));
const fixture = readFileSync(join(here, "..", "..", "fixtures", "r2", "maximum-observed-scene.json"));
const first = generateMaximumBytes();
const second = generateMaximumBytes();

assert.deepEqual(first, second);
assert.deepEqual(first, fixture);
assert.equal(first.length, expectedLength);
assert.equal(digest(first), expectedDigest);
console.log(`r2 maximum: ${first.length} bytes ${digest(first)}`);
