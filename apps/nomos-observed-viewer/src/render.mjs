import { CATALOG, sparseVariationSelected } from "./catalog.mjs";

const SQRT_TWO = Math.sqrt(2);
const SQRT_SIX = Math.sqrt(6);

const setPosition = (node, [x, y, z]) => {
  node.position.set(x, y, z);
  return node;
};

const mark = (node, kind) => {
  node.userData = { kind };
  return node;
};

const standard = (THREE, values) => new THREE.MeshStandardMaterial(values);
const basic = (THREE, values) =>
  new THREE.MeshBasicMaterial({ ...values, transparent: values.opacity < 1 });

const box = (THREE, geometry) =>
  new THREE.BoxGeometry(geometry.width, geometry.height, geometry.depth);

export const cameraFrame = (crop) => {
  const target = [(crop.width - 1) / 2, 0, (crop.height - 1) / 2];
  const corners = [
    [-0.5, 0, -0.5],
    [crop.width - 0.5, 0, -0.5],
    [-0.5, 0, crop.height - 0.5],
    [crop.width - 0.5, 0, crop.height - 0.5],
  ];
  const projected = corners.map(([x, y, z]) => {
    const dx = x - target[0];
    const dy = y - target[1];
    const dz = z - target[2];
    return [(dx - dz) / SQRT_TWO, (-dx + 2 * dy - dz) / SQRT_SIX];
  });
  let left = Math.min(...projected.map(([x]) => x)) - CATALOG.camera.margin_cells;
  let right = Math.max(...projected.map(([x]) => x)) + CATALOG.camera.margin_cells;
  let bottom = Math.min(...projected.map(([, y]) => y)) - CATALOG.camera.margin_cells;
  let top = Math.max(...projected.map(([, y]) => y)) + CATALOG.camera.margin_cells;
  const aspect = CATALOG.viewport.width / CATALOG.viewport.height;
  const width = right - left;
  const height = top - bottom;
  if (width / height < aspect) {
    const expansion = (height * aspect - width) / 2;
    left -= expansion;
    right += expansion;
  } else {
    const expansion = (width / aspect - height) / 2;
    bottom -= expansion;
    top += expansion;
  }
  const unit = 1 / Math.sqrt(3);
  const position = target.map((value) => value + CATALOG.camera.distance * unit);
  return Object.freeze({ bottom, corners, left, position, right, target, top });
};

const terrainNode = (THREE, row, cell) => {
  const entry = CATALOG.terrain.assemblies[row.assembly];
  const material = CATALOG.terrain.materials[row.material_family];
  const stackHeight = row.stack * CATALOG.terrain.stack_offset;
  const mesh = mark(
    new THREE.Mesh(box(THREE, entry.geometry), standard(THREE, material)),
    row.assembly,
  );
  setPosition(mesh, [cell.x, entry.center_height + stackHeight, cell.y]);
  const group = mark(new THREE.Group(), "terrain-cell");
  group.add(mesh);
  if (sparseVariationSelected(cell.x, cell.y, row.stack)) {
    const variation = CATALOG.terrain.variation;
    const patch = mark(
      new THREE.Mesh(
        box(THREE, variation.geometry),
        standard(THREE, {
          color: variation.colors[row.material_family],
          roughness: material.roughness,
          metalness: material.metalness,
        }),
      ),
      "terrain-accent",
    );
    const surface = entry.center_height * 2 + stackHeight;
    setPosition(patch, [cell.x, surface + CATALOG.terrain.visual_epsilon, cell.y]);
    group.add(patch);
  }
  return group;
};

const actorNode = (THREE, row) => {
  const entry = CATALOG.actor.assemblies[row.assembly];
  const pose = CATALOG.actor.poses[row.pose];
  const group = mark(new THREE.Group(), "actor");
  setPosition(group, [row.cell.x, row.cell.z, row.cell.y]);
  const figure = mark(new THREE.Group(), row.pose);
  setPosition(figure, pose.offset);
  figure.rotation.z = pose.rotation_z;
  const body = mark(
    new THREE.Mesh(
      new THREE.CylinderGeometry(
        entry.body.radius_top,
        entry.body.radius_bottom,
        entry.body.height,
        entry.body.radial_segments,
      ),
      standard(THREE, entry.material),
    ),
    "actor-body",
  );
  setPosition(body, entry.body.center);
  const head = mark(
    new THREE.Mesh(
      new THREE.IcosahedronGeometry(entry.head.radius, entry.head.detail),
      standard(THREE, entry.material),
    ),
    "actor-head",
  );
  setPosition(head, entry.head.center);
  figure.add(body, head);
  group.add(figure);

  if (row.controlled_marker === "present") {
    const selected = CATALOG.actor.controlled_marker.present;
    const marker = mark(
      new THREE.Mesh(
        new THREE.RingGeometry(selected.inner_radius, selected.outer_radius, selected.segments),
        basic(THREE, { color: selected.color, opacity: selected.opacity, side: THREE.DoubleSide }),
      ),
      "controlled-marker",
    );
    marker.rotation.x = -Math.PI / 2;
    marker.position.y = selected.height;
    group.add(marker);
  }
  if (row.hostile_outline === "present") {
    const selected = CATALOG.actor.hostile_outline.present;
    const outline = mark(
      new THREE.Mesh(
        new THREE.TorusGeometry(
          selected.radius,
          selected.tube,
          selected.radial_segments,
          selected.tubular_segments,
        ),
        basic(THREE, { color: selected.color, opacity: selected.opacity }),
      ),
      "hostile-outline",
    );
    outline.position.y = selected.center_height;
    group.add(outline);
  }
  if (row.protection_ring === "present") {
    const selected = CATALOG.actor.protection_ring.present;
    const ring = mark(
      new THREE.Mesh(
        new THREE.TorusGeometry(
          selected.radius,
          selected.tube,
          selected.radial_segments,
          selected.tubular_segments,
        ),
        basic(THREE, { color: selected.color, opacity: selected.opacity }),
      ),
      "protection-ring",
    );
    ring.rotation.x = Math.PI / 2;
    ring.position.y = selected.center_height;
    group.add(ring);
  }
  return group;
};

const actionNode = (THREE, marker) => {
  const entry = CATALOG.actions.markers[marker];
  const geometry = entry.geometry.kind === "cone"
    ? new THREE.ConeGeometry(entry.geometry.radius, entry.geometry.height, entry.geometry.radial_segments)
    : box(THREE, entry.geometry);
  return mark(new THREE.Mesh(geometry, basic(THREE, { color: entry.color, opacity: 1 })), marker);
};

export const buildScene = (THREE, view) => {
  const scene = mark(new THREE.Scene(), "scene-root");
  scene.background = new THREE.Color(CATALOG.renderer.clear_color);
  const terrain = mark(new THREE.Group(), "terrain-group");
  const actors = mark(new THREE.Group(), "actor-group");
  const actionGroup = mark(new THREE.Group(), "action-group");
  scene.add(terrain, actors, actionGroup);

  for (const row of view.terrain) {
    for (const cell of row.cells) terrain.add(terrainNode(THREE, row, cell));
  }

  const anchorByHandle = new Map();
  for (const row of view.actors) {
    const node = actorNode(THREE, row);
    actors.add(node);
    anchorByHandle.set(row.handle, [row.cell.x, row.cell.z, row.cell.y]);
  }

  const targetCounts = new Map();
  for (const row of view.actions) {
    targetCounts.set(row.target_handle, (targetCounts.get(row.target_handle) ?? 0) + 1);
  }
  const targetSeen = new Map();
  for (const row of view.actions) {
    const anchor = anchorByHandle.get(row.target_handle);
    if (!anchor) throw new Error("decoder supplied an action without an actor anchor");
    const ordinal = targetSeen.get(row.target_handle) ?? 0;
    targetSeen.set(row.target_handle, ordinal + 1);
    const count = targetCounts.get(row.target_handle);
    const offset = (ordinal - (count - 1) / 2) * CATALOG.actions.spacing_x;
    const marker = actionNode(THREE, row.marker);
    setPosition(marker, [anchor[0] + offset, anchor[1] + CATALOG.actions.anchor_height, anchor[2]]);
    actionGroup.add(marker);
  }

  const frame = cameraFrame(view.crop);
  const camera = new THREE.OrthographicCamera(
    frame.left,
    frame.right,
    frame.top,
    frame.bottom,
    CATALOG.camera.near,
    CATALOG.camera.far,
  );
  camera.up.set(...CATALOG.camera.up);
  camera.position.set(...frame.position);
  camera.lookAt(...frame.target);

  const hemisphere = CATALOG.lights.hemisphere;
  scene.add(mark(new THREE.HemisphereLight(
    hemisphere.sky_color,
    hemisphere.ground_color,
    hemisphere.intensity,
  ), "hemisphere-light"));
  const directional = CATALOG.lights.directional;
  const light = mark(new THREE.DirectionalLight(directional.color, directional.intensity), "directional-light");
  setPosition(light, directional.target_offset.map((value, index) => value + frame.target[index]));
  mark(light.target, "directional-target");
  setPosition(light.target, frame.target);
  scene.add(light, light.target);

  const counts = Object.freeze({
    actions: view.actions.length,
    actors: view.actors.length,
    controlled_markers: view.actors.filter((row) => row.controlled_marker === "present").length,
    hostile_outlines: view.actors.filter((row) => row.hostile_outline === "present").length,
    protection_rings: view.actors.filter((row) => row.protection_ring === "present").length,
    terrain_cells: view.terrain.reduce((sum, row) => sum + row.cells.length, 0),
    terrain_layers: view.terrain.length,
  });
  return Object.freeze({ camera, counts, frame, scene });
};

export const renderView = (THREE, container, view) => {
  const built = buildScene(THREE, view);
  const renderer = new THREE.WebGLRenderer({
    alpha: CATALOG.renderer.alpha,
    antialias: CATALOG.renderer.antialias,
  });
  renderer.setPixelRatio(CATALOG.viewport.pixel_ratio);
  renderer.setSize(CATALOG.viewport.width, CATALOG.viewport.height, false);
  renderer.setClearColor(CATALOG.renderer.clear_color, 1);
  renderer.shadowMap.enabled = CATALOG.renderer.shadows;
  if (THREE.SRGBColorSpace) renderer.outputColorSpace = THREE.SRGBColorSpace;
  container.replaceChildren(renderer.domElement);
  renderer.render(built.scene, built.camera);
  return Object.freeze({ ...built, renderer });
};

export const publicSceneGraph = (root) => {
  const rows = [];
  root.traverse((node) => {
    rows.push({
      args: node.geometry?.args ?? null,
      kind: node.userData?.kind ?? null,
      material: node.material?.options ?? null,
      position: [node.position?.x ?? 0, node.position?.y ?? 0, node.position?.z ?? 0],
      rotation: [node.rotation?.x ?? 0, node.rotation?.y ?? 0, node.rotation?.z ?? 0],
      type: node.type ?? node.constructor.name,
    });
  });
  return rows;
};
