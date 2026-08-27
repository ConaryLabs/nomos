import assert from "node:assert/strict";
import test from "node:test";

import { mountControls } from "../src/ui.mjs";

const element = (tagName) => ({
  tagName,
  children: [],
  listeners: {},
  style: {},
  append(...children) { this.children.push(...children); },
  addEventListener(name, listener) { this.listeners[name] = listener; },
  setAttribute(name, value) { this[name] = value; },
});

test("numeric two-scene controls use only the catalog-owned fixed presentation", () => {
  const document = {
    body: element("body"),
    createElement: element,
  };
  const chosen = [];
  const panel = mountControls(document, 2, 1, (index) => chosen.push(index));
  assert.equal(document.body.children[0], panel);
  assert.deepEqual(panel.style, {
    display: "flex",
    gap: "8px",
    left: "12px",
    position: "fixed",
    top: "12px",
    zIndex: "1",
  });
  assert.deepEqual(panel.children.map((button) => button.textContent), ["1", "2"]);
  assert.deepEqual(panel.children.map((button) => button["aria-label"]), ["Observation 1", "Observation 2"]);
  assert.deepEqual(panel.children.map((button) => button.style.background), ["#20252bcc", "#765f46"]);
  panel.children[0].listeners.click();
  assert.deepEqual(chosen, [0]);
});
