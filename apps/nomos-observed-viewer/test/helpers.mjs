import { readFileSync } from "node:fs";
import { dirname, resolve } from "node:path";
import { fileURLToPath } from "node:url";

import { canonicalBytes } from "../src/canonical.mjs";

export const app = resolve(dirname(fileURLToPath(import.meta.url)), "..");
export const root = resolve(app, "../..");
export const planPath = resolve(root, "fixtures/r2/plans/scene_one.json");
export const planBytes = () => readFileSync(planPath);
export const planObject = () => JSON.parse(planBytes().toString("utf8"));
export const bytesOf = (value) => canonicalBytes(value);

export const capture = (call) => {
  try {
    call();
  } catch (failure) {
    return failure;
  }
  throw new Error("expected rejection");
};
