import {
  cellsOf,
  doorState,
  lightOf,
  movementOf,
  socketPosition,
  wardSealed,
} from "./renderer-catalog.mjs";

// The fixed oblique camera. These six values used to be typed into every
// rendering plan and again into the area collection, read by this renderer and
// ignored by the other one — the ownership audit's first double authority.
// They are renderer-catalog constants, so they live in the renderer that
// projects with them and appear in no content artifact.
export const camera = Object.freeze({
  identity: "gaol_oblique_01",
  projection: "fixed_oblique",
  width: 1200,
  height: 540,
  tileWidth: 96,
  tileHeight: 50,
});

// Where the lattice origin lands on the canvas, and how tall one lattice cell
// of elevation is in screen pixels. The second is this renderer's counterpart
// of the WebGL renderer's VERTICAL_SCALE: both convert lattice cells into their
// own space, and both are now declared rather than inlined.
const ORIGIN = Object.freeze({ x: 470, y: 125 });
const CELL_HEIGHT_PIXELS = 38;

const palette = {
  void: "#10161d", fog: "#1c2832", stone0: "#202b34", stone1: "#2c3942",
  stone2: "#3c4a51", edge: "#536168", mortar: "#182128", iron: "#172128",
  rust: "#7c4b36", water: "#314c59", waterHi: "#70909a", cyan: "#8ee6e3",
  cyanDim: "#4d9f9f", amber: "#f5aa4b", amberDim: "#87552e", teal: "#2f7777",
  ochre: "#9a6640", text: "#d7ddd9", muted: "#8e9b9c", danger: "#d47158",
};

const esc = (value) => String(value).replaceAll("&", "&amp;").replaceAll("<", "&lt;").replaceAll('"', "&quot;");
const points = (rows) => rows.map(([x, y]) => `${x.toFixed(1)},${y.toFixed(1)}`).join(" ");
const glyphs = {
  " ":["00000","00000","00000","00000","00000","00000","00000"],
  "-":["00000","00000","00000","11111","00000","00000","00000"],
  "/":["00001","00010","00100","01000","10000","00000","00000"],
  ":":["00000","00100","00100","00000","00100","00100","00000"],
  "0":["01110","10001","10011","10101","11001","10001","01110"],
  "1":["00100","01100","00100","00100","00100","00100","01110"],
  "2":["01110","10001","00001","00010","00100","01000","11111"],
  "3":["11110","00001","00001","01110","00001","00001","11110"],
  "4":["00010","00110","01010","10010","11111","00010","00010"],
  "5":["11111","10000","10000","11110","00001","00001","11110"],
  "6":["01110","10000","10000","11110","10001","10001","01110"],
  "7":["11111","00001","00010","00100","01000","01000","01000"],
  "8":["01110","10001","10001","01110","10001","10001","01110"],
  "9":["01110","10001","10001","01111","00001","00001","01110"],
  A:["01110","10001","10001","11111","10001","10001","10001"], B:["11110","10001","10001","11110","10001","10001","11110"],
  C:["01111","10000","10000","10000","10000","10000","01111"], D:["11110","10001","10001","10001","10001","10001","11110"],
  E:["11111","10000","10000","11110","10000","10000","11111"], F:["11111","10000","10000","11110","10000","10000","10000"],
  G:["01111","10000","10000","10111","10001","10001","01111"], H:["10001","10001","10001","11111","10001","10001","10001"],
  I:["01110","00100","00100","00100","00100","00100","01110"], J:["00001","00001","00001","00001","10001","10001","01110"],
  K:["10001","10010","10100","11000","10100","10010","10001"], L:["10000","10000","10000","10000","10000","10000","11111"],
  M:["10001","11011","10101","10101","10001","10001","10001"], N:["10001","11001","10101","10011","10001","10001","10001"],
  O:["01110","10001","10001","10001","10001","10001","01110"], P:["11110","10001","10001","11110","10000","10000","10000"],
  Q:["01110","10001","10001","10001","10101","10010","01101"], R:["11110","10001","10001","11110","10100","10010","10001"],
  S:["01111","10000","10000","01110","00001","00001","11110"], T:["11111","00100","00100","00100","00100","00100","00100"],
  U:["10001","10001","10001","10001","10001","10001","01110"], V:["10001","10001","10001","10001","10001","01010","00100"],
  W:["10001","10001","10001","10101","10101","11011","10001"], X:["10001","10001","01010","00100","01010","10001","10001"],
  Y:["10001","10001","01010","00100","00100","00100","00100"], Z:["11111","00001","00010","00100","01000","10000","11111"],
};
const pixelText = (text, x, y, scale, color) => {
  const paths = [];
  [...String(text).toUpperCase()].forEach((character, index) => {
    const glyph = glyphs[character] ?? glyphs["-"];
    glyph.forEach((row, gy) => [...row].forEach((bit, gx) => {
      if (bit === "1") paths.push(`M${x + index * 6 * scale + gx * scale} ${y + gy * scale}h${scale}v${scale}h-${scale}z`);
    }));
  });
  return `<path d="${paths.join("")}" fill="${color}"/>`;
};

export function renderSvg(plan, scenarioId, forensic = false, presentation = {}) {
  const scenario = plan.scenarios.find((candidate) => candidate.id === scenarioId) ?? plan.scenarios[0];
  const width = camera.width;
  const height = camera.height;
  const tw = camera.tileWidth;
  const th = camera.tileHeight;
  const roomWidth = plan.architecture.bounds.width;
  const roomHeight = plan.architecture.bounds.height;
  const wallHeight = cellsOf(plan.architecture.wall_height_steps);
  const iso = (x, y, z = 0) => ({
    x: ORIGIN.x + (x - y) * tw / 2,
    y: ORIGIN.y + (x + y) * th / 2 - z * CELL_HEIGHT_PIXELS,
  });
  const cell = (x, y) => {
    const a = iso(x, y), b = iso(x + 1, y), c = iso(x + 1, y + 1), d = iso(x, y + 1);
    return points([[a.x, a.y], [b.x, b.y], [c.x, c.y], [d.x, d.y]]);
  };
  const prefix = `g-${scenario.id.replaceAll(/[^a-z0-9]/g, "-")}`;
  const chunks = [];

  chunks.push(`<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 ${width} ${height}" role="img" aria-label="Nomos executable gaol: ${esc(scenario.label)}">`);
  chunks.push(`<defs>
    <linearGradient id="${prefix}-bg" x2="0" y2="1"><stop stop-color="#17222b"/><stop offset="1" stop-color="#0d1218"/></linearGradient>
    <linearGradient id="${prefix}-water" x2="1" y2="1"><stop stop-color="#233b48"/><stop offset=".48" stop-color="#426575"/><stop offset=".62" stop-color="#294553"/><stop offset="1" stop-color="#1d313b"/></linearGradient>
    <radialGradient id="${prefix}-light"><stop stop-color="#ffd37a" stop-opacity=".7"/><stop offset=".35" stop-color="#d77b32" stop-opacity=".34"/><stop offset="1" stop-color="#d77b32" stop-opacity="0"/></radialGradient>
    <filter id="${prefix}-shadow"><feDropShadow dx="0" dy="8" stdDeviation="7" flood-color="#070b0f" flood-opacity=".75"/></filter>
    <filter id="${prefix}-cyan"><feGaussianBlur stdDeviation="3"/></filter>
  </defs>`);
  chunks.push(`<rect width="${width}" height="${height}" fill="url(#${prefix}-bg)"/>`);
  chunks.push(`<path d="M0 440 C220 360 395 415 620 350 S980 310 1200 390 V675 H0Z" fill="#111920" opacity=".85"/>`);

  // North and west masonry masses.
  const n0 = iso(0, 0), n1 = iso(roomWidth, 0), n0t = iso(0, 0, wallHeight), n1t = iso(roomWidth, 0, wallHeight);
  const w1 = iso(0, roomHeight), w1t = iso(0, roomHeight, wallHeight);
  chunks.push(`<polygon points="${points([[n0t.x,n0t.y],[n1t.x,n1t.y],[n1.x,n1.y],[n0.x,n0.y]])}" fill="${palette.stone1}" stroke="${palette.mortar}" stroke-width="4"/>`);
  chunks.push(`<polygon points="${points([[n0t.x,n0t.y],[n0.x,n0.y],[w1.x,w1.y],[w1t.x,w1t.y]])}" fill="${palette.stone0}" stroke="${palette.mortar}" stroke-width="4"/>`);
  for (let z = 1; z < wallHeight; z += 1) {
    const a = iso(0, 0, z), b = iso(roomWidth, 0, z), c = iso(0, roomHeight, z);
    chunks.push(`<path d="M${a.x} ${a.y} L${b.x} ${b.y} M${a.x} ${a.y} L${c.x} ${c.y}" stroke="${palette.mortar}" stroke-width="2" opacity=".7"/>`);
  }
  for (let x = 0; x <= roomWidth; x += 1) {
    const a = iso(x, 0), b = iso(x, 0, wallHeight);
    chunks.push(`<path d="M${a.x} ${a.y} L${b.x} ${b.y}" stroke="${palette.mortar}" stroke-width="1.5" opacity="${x % 2 ? .45 : .7}"/>`);
  }

  // Floor first, then bounded water cells.
  for (let y = 0; y < roomHeight; y += 1) for (let x = 0; x < roomWidth; x += 1) {
    chunks.push(`<polygon points="${cell(x,y)}" fill="${(x+y)%2 ? palette.stone1 : palette.stone0}" stroke="${palette.mortar}" stroke-width="1"/>`);
  }
  for (const entity of plan.entities.filter((entry) => entry.kind === "water")) {
    const { min, max } = entity.anchor;
    for (let y = min.y; y <= max.y; y += 1) for (let x = min.x; x <= max.x; x += 1) {
      chunks.push(`<polygon points="${cell(x,y)}" fill="url(#${prefix}-water)" stroke="${palette.waterHi}" stroke-width="1.2" opacity=".94"/>`);
      const p = iso(x + .18, y + .45), q = iso(x + .72, y + .45);
      chunks.push(`<path d="M${p.x} ${p.y} Q${(p.x+q.x)/2} ${p.y-4} ${q.x} ${q.y}" stroke="${palette.waterHi}" stroke-width="2" opacity=".48" fill="none"/>`);
    }
    if (forensic) {
      const p = iso((min.x + max.x + 1) / 2, (min.y + max.y + 1) / 2);
      const movement = movementOf(scenario, entity.id);
      chunks.push(`<g><rect x="${p.x-88}" y="${p.y+24}" width="176" height="38" rx="4" fill="#091016" opacity=".9"/>${pixelText(entity.id.replaceAll("_", " "), p.x-78, p.y+32, 1, palette.text)}${pixelText(`COST ${movement?.cost ?? "-"} X${min.x}Y${min.y} X${max.x}Y${max.y}`, p.x-78, p.y+47, 1, palette.waterHi)}</g>`);
    }
  }

  // Bounded lattice-authored masonry masses use one shared beveled grammar.
  for (const mass of plan.architecture.masses) {
    const massHeight = cellsOf(mass.height_steps);
    const top = [
      iso(mass.min.x, mass.min.y, massHeight),
      iso(mass.max.x, mass.min.y, massHeight),
      iso(mass.max.x, mass.max.y, massHeight),
      iso(mass.min.x, mass.max.y, massHeight),
    ];
    const base = [
      iso(mass.min.x, mass.min.y),
      iso(mass.max.x, mass.min.y),
      iso(mass.max.x, mass.max.y),
      iso(mass.min.x, mass.max.y),
    ];
    chunks.push(`<g filter="url(#${prefix}-shadow)"><polygon points="${points([[top[1].x,top[1].y],[top[2].x,top[2].y],[base[2].x,base[2].y],[base[1].x,base[1].y]])}" fill="${palette.stone0}" stroke="${palette.mortar}" stroke-width="3"/><polygon points="${points([[top[2].x,top[2].y],[top[3].x,top[3].y],[base[3].x,base[3].y],[base[2].x,base[2].y]])}" fill="${palette.stone1}" stroke="${palette.mortar}" stroke-width="3"/><polygon points="${points(top.map((p) => [p.x,p.y]))}" fill="${palette.stone2}" stroke="${palette.edge}" stroke-width="4"/><polyline points="${points([[top[1].x,top[1].y+7],[top[2].x,top[2].y+7],[top[3].x,top[3].y+7]])}" fill="none" stroke="${palette.edge}" stroke-width="3"/></g>`);
    if (forensic) {
      const p = iso((mass.min.x + mass.max.x) / 2, (mass.min.y + mass.max.y) / 2, massHeight);
      chunks.push(`<text x="${p.x}" y="${p.y-10}" text-anchor="middle" fill="${palette.text}" font-family="DejaVu Sans Mono, monospace" font-size="11">masonry/${esc(mass.id)} h=${massHeight}</text>`);
    }
  }

  // Door assemblies are entirely data-driven.
  for (const entity of plan.entities.filter((entry) => entry.kind === "door")) {
    const p = iso(entity.anchor.cell.x + .5, entity.anchor.cell.y, 0);
    const { access, integrity, ward } = doorState(scenario, entity.id);
    const movement = movementOf(scenario, entity.id) ?? { disposition: "unknown", reasons: [] };
    chunks.push(`<g filter="url(#${prefix}-shadow)">`);
    chunks.push(`<path d="M${p.x-46} ${p.y+8} V${p.y-74} Q${p.x} ${p.y-128} ${p.x+46} ${p.y-74} V${p.y+8}Z" fill="${palette.stone2}" stroke="${palette.edge}" stroke-width="7"/>`);
    chunks.push(`<path d="M${p.x-31} ${p.y+8} V${p.y-69} Q${p.x} ${p.y-105} ${p.x+31} ${p.y-69} V${p.y+8}Z" fill="#0c1217" stroke="${palette.iron}" stroke-width="5"/>`);
    if (integrity === "destroyed") {
      chunks.push(`<path d="M${p.x-25} ${p.y+5} l8 -68 l10 29 l9 -55 l8 38 l15 -22" fill="none" stroke="${palette.rust}" stroke-width="7" stroke-linecap="square"/>`);
    } else {
      const slide = access === "open" ? 24 : 0;
      for (let dx = -22; dx <= 22; dx += 11) chunks.push(`<path d="M${p.x+dx+slide} ${p.y+5} V${p.y-72}" stroke="${palette.iron}" stroke-width="6"/>`);
      chunks.push(`<path d="M${p.x-27+slide} ${p.y-42} H${p.x+27+slide}" stroke="${palette.rust}" stroke-width="5"/>`);
    }
    if (ward === "sealed") {
      chunks.push(`<circle cx="${p.x}" cy="${p.y-42}" r="31" fill="${palette.cyan}" opacity=".12" filter="url(#${prefix}-cyan)"/>`);
      chunks.push(`<path d="M${p.x} ${p.y-76} L${p.x+27} ${p.y-42} L${p.x} ${p.y-9} L${p.x-27} ${p.y-42}Z M${p.x-18} ${p.y-42} H${p.x+18}" fill="none" stroke="${palette.cyan}" stroke-width="3" opacity=".88"/>`);
    }
    chunks.push(`<circle cx="${p.x+23}" cy="${p.y-39}" r="5" fill="${access === "locked" ? palette.amber : palette.cyanDim}"/>`);
    chunks.push(`</g>`);
    if (forensic) chunks.push(`<g font-family="DejaVu Sans Mono, monospace" font-size="11"><rect x="${p.x-80}" y="${p.y+14}" width="160" height="48" rx="4" fill="#091016" opacity=".9"/><text x="${p.x}" y="${p.y+29}" text-anchor="middle" fill="${palette.text}">${esc(entity.id)} @ ${entity.anchor.cell.x},${entity.anchor.cell.y}</text><text x="${p.x}" y="${p.y+44}" text-anchor="middle" fill="${movement.disposition === "blocked" ? palette.danger : palette.cyan}">${esc(movement.disposition)} | ${esc(access)}/${esc(integrity)}/${esc(ward)}</text><text x="${p.x}" y="${p.y+57}" text-anchor="middle" fill="${palette.muted}">${esc(movement.reasons.join(" + ") || "base cost")}</text></g>`);
  }

  // Brazier and bounded light pool.
  for (const entity of plan.entities.filter((entry) => entry.kind === "light")) {
    const p = iso(entity.anchor.cell.x + .5, entity.anchor.cell.y + .5);
    const lit = lightOf(scenario, entity.id);
    if (lit) chunks.push(`<ellipse cx="${p.x}" cy="${p.y+8}" rx="150" ry="74" fill="url(#${prefix}-light)"/>`);
    chunks.push(`<g filter="url(#${prefix}-shadow)"><path d="M${p.x-18} ${p.y+10} L${p.x-12} ${p.y-18} H${p.x+12} L${p.x+18} ${p.y+10}Z" fill="${palette.iron}" stroke="${palette.rust}" stroke-width="4"/>`);
    if (lit) chunks.push(`<path d="M${p.x} ${p.y-15} C${p.x-22} ${p.y-41} ${p.x+4} ${p.y-56} ${p.x+1} ${p.y-74} C${p.x+27} ${p.y-49} ${p.x+20} ${p.y-25} ${p.x} ${p.y-15}Z" fill="${palette.amber}" stroke="#ffe3a0" stroke-width="2"/>`);
    chunks.push(`</g>`);
    if (forensic) chunks.push(`<text x="${p.x}" y="${p.y+31}" text-anchor="middle" fill="${lit ? palette.amber : palette.muted}" font-family="DejaVu Sans Mono, monospace" font-size="11">${esc(entity.id)} | light=${lit}</text>`);
  }

  // Readable actor silhouettes.
  for (const actor of plan.actors) {
    const anchor = presentation.actorPositions?.[actor.id] ?? actor.cell;
    const p = iso(anchor.x + .5, anchor.y + .5, anchor.z ?? 0);
    if (actor.id === "player") {
      chunks.push(`<g filter="url(#${prefix}-shadow)"><ellipse cx="${p.x}" cy="${p.y+12}" rx="22" ry="9" fill="#080d11" opacity=".6"/><circle cx="${p.x}" cy="${p.y-42}" r="10" fill="#80aeb0"/><path d="M${p.x-12} ${p.y-31} L${p.x-20} ${p.y+7} L${p.x-4} ${p.y+1} L${p.x+5} ${p.y+15} L${p.x+17} ${p.y+8} L${p.x+11} ${p.y-30}Z" fill="${palette.teal}" stroke="#182a2e" stroke-width="4"/><path d="M${p.x+9} ${p.y-17} L${p.x+36} ${p.y-39}" stroke="${palette.cyanDim}" stroke-width="5"/></g>`);
    } else {
      chunks.push(`<g filter="url(#${prefix}-shadow)"><ellipse cx="${p.x}" cy="${p.y+13}" rx="30" ry="10" fill="#080d11" opacity=".65"/><circle cx="${p.x+3}" cy="${p.y-45}" r="12" fill="#9d7a5c"/><path d="M${p.x-19} ${p.y-33} L${p.x-24} ${p.y+11} L${p.x+21} ${p.y+11} L${p.x+17} ${p.y-33}Z" fill="${palette.ochre}" stroke="#392921" stroke-width="5"/><path d="M${p.x-29} ${p.y-28} Q${p.x-53} ${p.y-9} ${p.x-31} ${p.y+12} Q${p.x-7} ${p.y-8} ${p.x-29} ${p.y-28}Z" fill="${palette.rust}" stroke="#3c2c27" stroke-width="5"/></g>`);
    }
    if (forensic) chunks.push(`<text x="${p.x}" y="${p.y+30}" text-anchor="middle" fill="${palette.text}" font-family="DejaVu Sans Mono, monospace" font-size="11">actor/${esc(actor.id)} @ ${Number(anchor.x).toFixed(2)},${Number(anchor.y).toFixed(2)}</text>`);
  }

  // Restrained semantic effect, kept below actor salience.
  //
  // Placement comes from the renderer catalog's socket table, not from content:
  // the effect names `{entity, socket}` and this renderer decides where that
  // socket is. An effect whose anchor entity is absent from the plan is a build
  // failure rather than an unplaced glyph.
  for (const effect of plan.effects) {
    const gate = plan.entities.find((entry) => entry.id === effect.anchor.entity);
    if (!gate) throw new Error(`effect ${effect.id} anchors to absent entity ${effect.anchor.entity}`);
    if (!wardSealed(scenario, gate.id)) continue;
    const socket = socketPosition(gate, effect.anchor.socket);
    const p = iso(socket.x, socket.y, socket.z);
    chunks.push(`<path d="M${p.x-45} ${p.y+8} Q${p.x} ${p.y-48} ${p.x+43} ${p.y-5} Q${p.x+5} ${p.y-24} ${p.x-45} ${p.y+8}Z" fill="${palette.cyan}" opacity=".5" stroke="#c0ffff" stroke-width="2"/>`);
    chunks.push(`<circle cx="${p.x+54}" cy="${p.y-17}" r="3" fill="${palette.cyan}"/><circle cx="${p.x+64}" cy="${p.y-4}" r="2" fill="${palette.cyan}"/>`);
  }

  // Minimal edge UI.
  chunks.push(`<g><rect x="32" y="31" width="310" height="58" rx="9" fill="#0a1117" opacity=".82" stroke="#39474d"/>${pixelText(`NOMOS // ${scenario.label}`, 49, 43, 2, palette.text)}<rect x="49" y="66" width="128" height="7" rx="3" fill="#28363d"/><rect x="49" y="66" width="102" height="7" rx="3" fill="${palette.teal}"/><circle cx="278" cy="69" r="9" fill="none" stroke="${palette.cyan}" stroke-width="2"/><path d="M271 69 H285 M278 62 V76" stroke="${palette.cyan}" stroke-width="2"/></g>`);
  const primaryMovement = movementOf(scenario, plan.objective.gate);
  const waterCost = Math.max(...plan.entities.filter((entry) => entry.kind === "water").map((entry) => movementOf(scenario, entry.id)?.cost ?? 1));
  chunks.push(`<g><rect x="870" y="31" width="298" height="82" rx="9" fill="#0a1117" opacity=".84" stroke="#39474d"/>${pixelText(plan.objective.gate.replaceAll("_", " "), 891, 45, 2, palette.muted)}${pixelText(primaryMovement?.disposition ?? "unknown", 891, 66, 3, primaryMovement?.disposition === "blocked" ? palette.danger : palette.cyan)}${pixelText(`WATER ${waterCost} TICK ${scenario.tick} ${scenario.state_hash.slice(0,8)}`, 891, 95, 1, palette.text)}</g>`);
  if (forensic) chunks.push(`<g font-family="DejaVu Sans Mono, monospace" font-size="11"><rect x="31" y="454" width="1138" height="67" rx="7" fill="#071016" opacity=".92" stroke="${palette.cyanDim}"/><text x="48" y="477" fill="${palette.cyan}">FORENSIC PROJECTION OWNERSHIP</text><text x="48" y="496" fill="${palette.text}">renderer input: nomos.rendering_plan@2 | source/World IR unavailable</text><text x="48" y="515" fill="${palette.muted}">movement: navigation projection + runtime state | light: simulation/persistence projection + runtime state | visuals: stable assembly IDs</text></g>`);
  chunks.push(`</svg>`);
  return chunks.join("");
}
