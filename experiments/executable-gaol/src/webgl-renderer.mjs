import * as THREE from "https://cdn.jsdelivr.net/npm/three@0.185.1/build/three.module.min.js";
import {
  LOOK_PROFILE_IDS,
  cellsOf,
  doorState,
  lightOf,
  socketPosition,
  wardSealed,
} from "./renderer-catalog.mjs";

// Lattice cells of elevation to WebGL world units. Horizontal cells are 1.0
// world unit; vertical cells are shorter, so a 4.5-cell wall reads at the
// height the door and brazier assemblies were modelled against. This was an
// undeclared `* 0.72` applied to two content fields — the ownership audit's
// second and third double authorities, where the same field meant one thing
// here and another in the SVG renderer. Content now declares heights in
// vertical steps and each renderer declares its own conversion out of cells.
const VERTICAL_SCALE = 0.72;

// This renderer's own camera. It never read the plan's `camera` block, which
// is why that block is gone; these are the frustum and offset it actually
// uses, now named rather than inlined at their use sites.
const ORTHO_HALF_HEIGHT = 3.7;
const CAMERA_OFFSET = Object.freeze({ x: 0.86, y: 0.92, z: 1.08 });
const CAMERA_TARGET_HEIGHT = 0.5;

const colors = Object.freeze({
  void: 0x090e13,
  fog: 0x111b24,
  stone0: 0x202b34,
  stone1: 0x2d3a43,
  stone2: 0x3d4b52,
  mortar: 0x111920,
  iron: 0x111a20,
  rust: 0x70412f,
  water: 0x244857,
  cyan: 0x83eeea,
  amber: 0xffa544,
  player: 0x347b7d,
  gaoler: 0x8c5638,
});

export const lookProfiles = Object.freeze({
  baseline: Object.freeze({
    id: "gaol_baseline_01",
    palette: colors,
    fogDensity: 0.045,
    exposure: 1.28,
    bevel: 0,
    actorOutline: 0,
    materials: Object.freeze({}),
  }),
  procedural: Object.freeze({
    id: "gaol_procedural_01",
    palette: colors,
    fogDensity: 0.041,
    exposure: 1.34,
    bevel: 0.055,
    actorOutline: 1.065,
    materials: Object.freeze({
      stone: Object.freeze({ scale: 1.35, variation: 0.13, accent: 0x536168, accentMix: 0.16 }),
      iron: Object.freeze({ scale: 2.1, variation: 0.08, accent: colors.rust, accentMix: 0.22 }),
      cloth: Object.freeze({ scale: 2.8, variation: 0.09, accent: colors.mortar, accentMix: 0.08 }),
    }),
  }),
});

const shadow = (object, cast = true, receive = true) => {
  object.castShadow = cast;
  object.receiveShadow = receive;
  return object;
};

const material = (color, roughness = 0.78, metalness = 0.04, surface = null) => {
  const result = new THREE.MeshStandardMaterial({ color, roughness, metalness });
  if (!surface) return result;
  result.onBeforeCompile = (shader) => {
    shader.uniforms.uLookScale = { value: surface.scale };
    shader.uniforms.uLookVariation = { value: surface.variation };
    shader.uniforms.uLookAccent = { value: new THREE.Color(surface.accent) };
    shader.uniforms.uLookAccentMix = { value: surface.accentMix };
    shader.vertexShader = shader.vertexShader
      .replace("#include <common>", "#include <common>\nvarying vec3 vLookWorldPosition;")
      .replace("#include <begin_vertex>", "#include <begin_vertex>\nvLookWorldPosition = (modelMatrix * vec4(transformed, 1.0)).xyz;");
    shader.fragmentShader = shader.fragmentShader
      .replace("#include <common>", `#include <common>
        varying vec3 vLookWorldPosition;
        uniform float uLookScale;
        uniform float uLookVariation;
        uniform vec3 uLookAccent;
        uniform float uLookAccentMix;
        float lookHash(vec3 cell) {
          return fract(sin(dot(cell, vec3(12.9898, 78.233, 45.164))) * 43758.5453);
        }`)
      .replace("#include <color_fragment>", `#include <color_fragment>
        vec3 lookCell = floor(vLookWorldPosition * uLookScale);
        float lookValue = lookHash(lookCell);
        float lookBand = floor(lookValue * 4.0) / 3.0;
        diffuseColor.rgb *= mix(1.0 - uLookVariation, 1.0 + uLookVariation, lookBand);
        float lookAccent = step(0.78, lookValue) * uLookAccentMix;
        diffuseColor.rgb = mix(diffuseColor.rgb, uLookAccent, lookAccent);`);
  };
  result.customProgramCacheKey = () => JSON.stringify(surface);
  return result;
};

const disposeTree = (root) => root.traverse((object) => {
  object.geometry?.dispose();
  if (Array.isArray(object.material)) object.material.forEach((entry) => entry.dispose());
  else object.material?.dispose();
});

const cellPosition = (plan, cell, elevation = 0) => new THREE.Vector3(
  cell.x + 0.5 - plan.architecture.bounds.width / 2,
  elevation,
  cell.y + 0.5 - plan.architecture.bounds.height / 2,
);

const addBox = (parent, size, position, boxMaterial, options = {}) => {
  let geometry;
  const bevel = Math.min(options.bevel ?? 0, Math.min(...size) / 4);
  if (bevel > 0) {
    const [width, height, depth] = size;
    const shape = new THREE.Shape()
      .moveTo(-width / 2 + bevel, -height / 2 + bevel)
      .lineTo(width / 2 - bevel, -height / 2 + bevel)
      .lineTo(width / 2 - bevel, height / 2 - bevel)
      .lineTo(-width / 2 + bevel, height / 2 - bevel)
      .closePath();
    geometry = new THREE.ExtrudeGeometry(shape, {
      depth: Math.max(depth - bevel * 2, bevel),
      bevelEnabled: true,
      bevelSegments: 1,
      bevelSize: bevel,
      bevelThickness: bevel,
      curveSegments: 1,
    });
    geometry.center();
  } else geometry = new THREE.BoxGeometry(...size);
  const mesh = shadow(new THREE.Mesh(geometry, boxMaterial));
  mesh.position.set(...position);
  if (options.rotationY) mesh.rotation.y = options.rotationY;
  parent.add(mesh);
  return mesh;
};

const addStoneSegment = (parent, x, z, height, axis, stoneMaterials) => {
  const courses = Math.ceil(height / 0.46);
  for (let course = 0; course < courses; course += 1) {
    const courseHeight = Math.min(0.42, height - course * 0.46);
    if (courseHeight <= 0) continue;
    const offset = course % 2 ? 0.13 : -0.13;
    const size = axis === "x" ? [0.94, courseHeight, 0.34] : [0.34, courseHeight, 0.94];
    const position = axis === "x"
      ? [x + offset, course * 0.46 + courseHeight / 2, z]
      : [x, course * 0.46 + courseHeight / 2, z + offset];
    addBox(parent, size, position, stoneMaterials[(course + Math.round(x + z)) & 1]);
  }
  const capSize = axis === "x" ? [1.02, 0.12, 0.44] : [0.44, 0.12, 1.02];
  addBox(parent, capSize, [x, height + 0.06, z], stoneMaterials[2]);
};

const waterMaterial = () => new THREE.ShaderMaterial({
  uniforms: {
    uTime: { value: 0 },
    uDeep: { value: new THREE.Color(0x173744) },
    uLight: { value: new THREE.Color(0x4d8290) },
  },
  vertexShader: `
    uniform float uTime;
    varying float vWave;
    varying vec2 vUv;
    void main() {
      vUv = uv;
      vec3 p = position;
      float a = sin((p.x + uTime * .55) * 4.2) * .025;
      float b = cos((p.y - uTime * .38) * 5.7) * .018;
      p.z += a + b;
      vWave = a + b;
      gl_Position = projectionMatrix * modelViewMatrix * vec4(p, 1.0);
    }
  `,
  fragmentShader: `
    uniform vec3 uDeep;
    uniform vec3 uLight;
    uniform float uTime;
    varying float vWave;
    varying vec2 vUv;
    void main() {
      float line = smoothstep(.46, .5, abs(sin((vUv.x * 9.0 + vUv.y * 4.0 + uTime * .2) * 3.14159)));
      vec3 color = mix(uDeep, uLight, clamp(vWave * 9.0 + .42 + line * .08, 0.0, 1.0));
      gl_FragColor = vec4(color, .86);
    }
  `,
  transparent: true,
  depthWrite: false,
  side: THREE.DoubleSide,
});

const buildFloor = (root, plan, resources) => {
  const { width, height } = plan.architecture.bounds;
  for (let y = 0; y < height; y += 1) for (let x = 0; x < width; x += 1) {
    const floor = addBox(root, [0.96, 0.12, 0.96], [x + 0.5 - width / 2, -0.08, y + 0.5 - height / 2], resources.stone[(x + y) & 1], { cast: false });
    floor.castShadow = false;
  }
};

const buildWalls = (root, plan, resources) => {
  const { width, height } = plan.architecture.bounds;
  const wallHeight = cellsOf(plan.architecture.wall_height_steps) * VERTICAL_SCALE;
  // The wall a door interrupts comes from the entity's declared
  // `anchor.direction`, not from `cell.y === 0`. The projection has always
  // carried the direction; the ownership audit recorded that nothing read it.
  const northDoors = new Set(plan.entities
    .filter((entity) => entity.kind === "door" && entity.anchor.direction === "north")
    .map((entity) => entity.anchor.cell.x));
  for (let x = 0; x < width; x += 1) {
    if (!northDoors.has(x)) addStoneSegment(root, x + 0.5 - width / 2, -height / 2 - 0.18, wallHeight, "x", resources.stone);
    addStoneSegment(root, x + 0.5 - width / 2, height / 2 + 0.18, 0.32, "x", resources.stone);
  }
  for (let y = 0; y < height; y += 1) {
    addStoneSegment(root, -width / 2 - 0.18, y + 0.5 - height / 2, wallHeight, "z", resources.stone);
    addStoneSegment(root, width / 2 + 0.18, y + 0.5 - height / 2, 0.32, "z", resources.stone);
  }
};

const buildMasses = (root, plan, resources, look) => {
  for (const mass of plan.architecture.masses) {
    const width = mass.max.x - mass.min.x;
    const depth = mass.max.y - mass.min.y;
    const height = cellsOf(mass.height_steps) * VERTICAL_SCALE;
    const x = (mass.min.x + mass.max.x) / 2 - plan.architecture.bounds.width / 2;
    const z = (mass.min.y + mass.max.y) / 2 - plan.architecture.bounds.height / 2;
    addBox(root, [width - 0.08, height, depth - 0.08], [x, height / 2, z], resources.stone[1], { bevel: look.bevel });
    addBox(root, [width + 0.03, 0.12, depth + 0.03], [x, height + 0.06, z], resources.stone[2], { bevel: look.bevel * 0.65 });
  }
};

const buildWater = (root, plan, resources, animatedMaterials) => {
  for (const entity of plan.entities.filter((entry) => entry.kind === "water")) {
    const width = entity.anchor.max.x - entity.anchor.min.x + 1;
    const depth = entity.anchor.max.y - entity.anchor.min.y + 1;
    const x = (entity.anchor.min.x + entity.anchor.max.x + 1) / 2 - plan.architecture.bounds.width / 2;
    const z = (entity.anchor.min.y + entity.anchor.max.y + 1) / 2 - plan.architecture.bounds.height / 2;
    addBox(root, [width - 0.09, 0.14, depth - 0.09], [x, 0.015, z], resources.waterBed, { cast: false });
    const shader = waterMaterial();
    animatedMaterials.push(shader);
    const surface = new THREE.Mesh(new THREE.PlaneGeometry(width - 0.1, depth - 0.1, width * 6, depth * 6), shader);
    surface.rotation.x = -Math.PI / 2;
    surface.position.set(x, 0.105, z);
    surface.receiveShadow = true;
    root.add(surface);
  }
};

const buildDoor = (root, plan, scenario, entity, resources, glowLights, look) => {
  const group = new THREE.Group();
  group.position.copy(cellPosition(plan, entity.anchor.cell));
  group.position.z -= 0.38;
  const { access, integrity, ward } = doorState(scenario, entity.id);

  addBox(group, [0.25, 2.45, 0.48], [-0.58, 1.22, 0], resources.stone[2], { bevel: look.bevel });
  addBox(group, [0.25, 2.45, 0.48], [0.58, 1.22, 0], resources.stone[2], { bevel: look.bevel });
  addBox(group, [1.42, 0.3, 0.5], [0, 2.38, 0], resources.stone[2], { bevel: look.bevel });
  addBox(group, [1.68, 0.13, 0.58], [0, 2.6, 0], resources.stone[1], { bevel: look.bevel * 0.65 });

  if (integrity === "destroyed") {
    for (const [x, y, angle] of [[-0.34, .65, -.16], [.02, .42, .21], [.35, .78, -.25]]) {
      const bar = addBox(group, [0.075, 1.18, 0.08], [x, y, 0], resources.iron);
      bar.rotation.z = angle;
    }
  } else {
    const slide = access === "open" ? 0.62 : 0;
    for (let x = -0.42; x <= 0.43; x += 0.21) addBox(group, [0.065, 1.92, 0.07], [x + slide, 1.12, 0], resources.iron);
    addBox(group, [1.02, 0.09, 0.09], [slide, 1.12, 0], resources.rust);
  }

  if (ward === "sealed") {
    const wardMaterial = new THREE.MeshBasicMaterial({ color: colors.cyan, transparent: true, opacity: 0.72, blending: THREE.AdditiveBlending });
    const ring = new THREE.Mesh(new THREE.TorusGeometry(0.42, 0.025, 10, 48), wardMaterial);
    ring.position.set(0, 1.22, 0.1);
    group.add(ring);
    const diamond = new THREE.LineLoop(
      new THREE.BufferGeometry().setFromPoints([
        new THREE.Vector3(0, 1.78, 0.11), new THREE.Vector3(.42, 1.22, .11),
        new THREE.Vector3(0, .66, .11), new THREE.Vector3(-.42, 1.22, .11),
      ]),
      new THREE.LineBasicMaterial({ color: colors.cyan, transparent: true, opacity: .8 }),
    );
    group.add(diamond);
    const wardLight = new THREE.PointLight(colors.cyan, 3.4, 3.2, 2);
    wardLight.position.set(0, 1.22, 0.45);
    group.add(wardLight);
    glowLights.push({ light: wardLight, phase: entity.anchor.cell.x * 0.7, base: 3.4, amplitude: 0.25 });
  }
  root.add(group);
};

const buildBrazier = (root, plan, scenario, entity, resources, glowLights) => {
  const group = new THREE.Group();
  group.position.copy(cellPosition(plan, entity.anchor.cell));
  const lit = lightOf(scenario, entity.id) === true;
  const pedestal = shadow(new THREE.Mesh(new THREE.CylinderGeometry(.16, .23, .48, 8), resources.iron));
  pedestal.position.y = .24;
  group.add(pedestal);
  const bowl = shadow(new THREE.Mesh(new THREE.CylinderGeometry(.34, .18, .18, 8), resources.rust));
  bowl.position.y = .55;
  group.add(bowl);
  if (lit) {
    const flameMaterial = new THREE.MeshBasicMaterial({ color: colors.amber, transparent: true, opacity: .92, blending: THREE.AdditiveBlending });
    const flame = new THREE.Mesh(new THREE.ConeGeometry(.17, .55, 7), flameMaterial);
    flame.position.y = .92;
    flame.scale.z = .72;
    group.add(flame);
    const light = new THREE.PointLight(colors.amber, 8.5, 5.6, 2);
    light.position.y = 1.12;
    light.castShadow = true;
    light.shadow.mapSize.set(512, 512);
    group.add(light);
    glowLights.push({ light, flame, phase: entity.anchor.cell.x + entity.anchor.cell.y, base: 8.5, amplitude: 1.1 });
  }
  root.add(group);
};

// Effects are placed by the renderer catalog's socket table, not by a
// coordinate in content. The `ward` socket resolves to the gate's ward height,
// so the crescent stands upright on the gate facing the way the ward ring
// faces, rather than lying flat on the floor at a hand-placed spot. Honouring
// all three components of the socket is the point: a renderer that dropped its
// elevation would be the wall-height double authority again in a new field.
const buildEffectAnchors = (root, plan, scenario) => {
  const { width, height } = plan.architecture.bounds;
  for (const effect of plan.effects) {
    const gate = plan.entities.find((entity) => entity.id === effect.anchor.entity);
    if (!gate) throw new Error(`effect ${effect.id} anchors to absent entity ${effect.anchor.entity}`);
    if (!wardSealed(scenario, gate.id)) continue;
    const socket = socketPosition(gate, effect.anchor.socket);
    const material = new THREE.MeshBasicMaterial({ color: colors.cyan, transparent: true, opacity: .34, blending: THREE.AdditiveBlending });
    const crescent = new THREE.Mesh(new THREE.TorusGeometry(.38, .055, 8, 32, Math.PI * 1.25), material);
    crescent.rotation.z = -.45;
    crescent.position.set(
      socket.x - width / 2,
      socket.z * VERTICAL_SCALE,
      socket.y - height / 2,
    );
    root.add(crescent);
  }
};

const addActorSilhouette = (group, mesh, look, resources) => {
  if (look.actorOutline <= 1) return;
  const outline = new THREE.Mesh(mesh.geometry, resources.outline);
  outline.position.copy(mesh.position);
  outline.rotation.copy(mesh.rotation);
  outline.scale.copy(mesh.scale).multiplyScalar(look.actorOutline);
  outline.castShadow = false;
  outline.receiveShadow = false;
  group.add(outline);
};

const actorMesh = (id, resources, look) => {
  const group = new THREE.Group();
  const isPlayer = id === "player";
  const bodyMaterial = isPlayer ? resources.player : resources.gaoler;
  const cloak = shadow(new THREE.Mesh(new THREE.ConeGeometry(isPlayer ? .25 : .32, isPlayer ? .78 : .9, 7), bodyMaterial));
  cloak.position.y = isPlayer ? .43 : .48;
  addActorSilhouette(group, cloak, look, resources);
  group.add(cloak);
  const head = shadow(new THREE.Mesh(new THREE.IcosahedronGeometry(isPlayer ? .15 : .17, 1), resources.skin));
  head.position.y = isPlayer ? .92 : 1.04;
  addActorSilhouette(group, head, look, resources);
  group.add(head);
  if (isPlayer) {
    const blade = addBox(group, [.055, .65, .055], [.28, .58, 0], resources.cyanMetal);
    blade.rotation.z = -.55;
  } else {
    const shoulder = addBox(group, [.72, .11, .2], [0, .78, 0], resources.rust);
    shoulder.rotation.z = .04;
  }
  return group;
};

const buildActors = (root, plan, actorPositions, resources, actorMeshes, look) => {
  for (const actor of plan.actors) {
    const mesh = actorMesh(actor.id, resources, look);
    const anchor = actorPositions?.[actor.id] ?? actor.cell;
    mesh.position.copy(cellPosition(plan, anchor, anchor.z ?? 0));
    actorMeshes.set(actor.id, mesh);
    root.add(mesh);
  }
};

const makeResources = (look) => ({
  stone: [material(colors.stone0, .78, .04, look.materials.stone), material(colors.stone1, .78, .04, look.materials.stone), material(colors.stone2, .7, .04, look.materials.stone)],
  iron: material(colors.iron, .42, .7, look.materials.iron),
  rust: material(colors.rust, .68, .42, look.materials.iron),
  waterBed: material(colors.water, .3, .18),
  player: material(colors.player, .62, .04, look.materials.cloth),
  gaoler: material(colors.gaoler, .74, .04, look.materials.cloth),
  skin: material(0x96735b, .85),
  cyanMetal: material(colors.cyan, .25, .6),
  outline: new THREE.MeshBasicMaterial({ color: colors.void, side: THREE.BackSide }),
});

export function createGaolRenderer(container) {
  const renderer = new THREE.WebGLRenderer({ antialias: true, alpha: false, powerPreference: "high-performance" });
  renderer.setPixelRatio(Math.min(devicePixelRatio, 2));
  renderer.setClearColor(colors.void);
  renderer.shadowMap.enabled = true;
  renderer.shadowMap.type = THREE.PCFSoftShadowMap;
  renderer.outputColorSpace = THREE.SRGBColorSpace;
  renderer.toneMapping = THREE.ACESFilmicToneMapping;
  renderer.toneMappingExposure = lookProfiles.procedural.exposure;
  container.replaceChildren(renderer.domElement);

  const scene = new THREE.Scene();
  scene.background = new THREE.Color(colors.void);
  scene.fog = new THREE.FogExp2(colors.fog, lookProfiles.procedural.fogDensity);
  const camera = new THREE.OrthographicCamera(
    -ORTHO_HALF_HEIGHT * 2, ORTHO_HALF_HEIGHT * 2,
    ORTHO_HALF_HEIGHT, -ORTHO_HALF_HEIGHT,
    0.1, 80,
  );
  const hemi = new THREE.HemisphereLight(0x8aa8b8, 0x131c24, 2.05);
  scene.add(hemi);
  const moon = new THREE.DirectionalLight(0xabc8d7, 4.2);
  moon.position.set(-7, 11, 7);
  moon.castShadow = true;
  moon.shadow.mapSize.set(2048, 2048);
  moon.shadow.camera.left = -8;
  moon.shadow.camera.right = 8;
  moon.shadow.camera.top = 8;
  moon.shadow.camera.bottom = -8;
  scene.add(moon);

  let worldRoot = new THREE.Group();
  scene.add(worldRoot);
  let actorMeshes = new Map();
  let animatedMaterials = [];
  let glowLights = [];
  let planIdentity;
  let scenarioIdentity;
  let forensicState = false;
  let grid;
  let look = lookProfiles.procedural;

  const fit = () => {
    const width = Math.max(container.clientWidth, 1);
    const height = Math.max(container.clientHeight, 1);
    renderer.setSize(width, height, false);
    const aspect = width / height;
    camera.left = -ORTHO_HALF_HEIGHT * aspect;
    camera.right = ORTHO_HALF_HEIGHT * aspect;
    camera.top = ORTHO_HALF_HEIGHT;
    camera.bottom = -ORTHO_HALF_HEIGHT;
    camera.updateProjectionMatrix();
  };
  new ResizeObserver(fit).observe(container);
  fit();

  const rebuild = (plan, scenarioId, actorPositions) => {
    scene.remove(worldRoot);
    disposeTree(worldRoot);
    worldRoot = new THREE.Group();
    scene.add(worldRoot);
    actorMeshes = new Map();
    animatedMaterials = [];
    glowLights = [];
    const resources = makeResources(look);
    const scenario = plan.scenarios.find((candidate) => candidate.id === scenarioId) ?? plan.scenarios[0];
    buildFloor(worldRoot, plan, resources);
    buildWalls(worldRoot, plan, resources);
    buildMasses(worldRoot, plan, resources, look);
    buildWater(worldRoot, plan, resources, animatedMaterials);
    for (const entity of plan.entities) {
      if (entity.kind === "door") buildDoor(worldRoot, plan, scenario, entity, resources, glowLights, look);
      if (entity.kind === "light") buildBrazier(worldRoot, plan, scenario, entity, resources, glowLights);
    }
    buildEffectAnchors(worldRoot, plan, scenario);
    buildActors(worldRoot, plan, actorPositions, resources, actorMeshes, look);
    const { width, height } = plan.architecture.bounds;
    const target = new THREE.Vector3(0, CAMERA_TARGET_HEIGHT, 0);
    camera.position.set(
      width * CAMERA_OFFSET.x,
      Math.max(width, height) * CAMERA_OFFSET.y,
      height * CAMERA_OFFSET.z,
    );
    camera.lookAt(target);
    planIdentity = plan.area.id;
    scenarioIdentity = scenario.id;
  };

  const updateActors = (plan, actorPositions) => {
    for (const actor of plan.actors) {
      const anchor = actorPositions?.[actor.id] ?? actor.cell;
      actorMeshes.get(actor.id)?.position.copy(cellPosition(plan, anchor, anchor.z ?? 0));
    }
  };

  const present = (plan, scenarioId, forensic = false, presentation = {}) => {
    if (planIdentity !== plan.area.id || scenarioIdentity !== scenarioId) rebuild(plan, scenarioId, presentation.actorPositions);
    else updateActors(plan, presentation.actorPositions);
    if (forensicState !== forensic) {
      forensicState = forensic;
      if (grid) worldRoot.remove(grid);
      grid = forensic ? new THREE.GridHelper(Math.max(plan.architecture.bounds.width, plan.architecture.bounds.height), Math.max(plan.architecture.bounds.width, plan.architecture.bounds.height), colors.cyan, 0x30434a) : null;
      if (grid) {
        grid.position.y = .13;
        worldRoot.add(grid);
      }
    }
  };

  const setLookProfile = (profileId) => {
    if (!LOOK_PROFILE_IDS.includes(profileId)) {
      throw new Error(`unknown look profile ${profileId}`);
    }
    const next = lookProfiles[profileId];
    if (!next) throw new Error(`unknown look profile ${profileId}`);
    if (next === look) return;
    look = next;
    scene.fog.density = look.fogDensity;
    renderer.toneMappingExposure = look.exposure;
    planIdentity = undefined;
  };

  const clock = new THREE.Clock();
  const animate = () => {
    const elapsed = clock.getElapsedTime();
    for (const shader of animatedMaterials) shader.uniforms.uTime.value = elapsed;
    for (const glow of glowLights) {
      const flicker = Math.sin(elapsed * 8.7 + glow.phase) * .5 + Math.sin(elapsed * 13.1 + glow.phase * .7) * .5;
      glow.light.intensity = glow.base + flicker * glow.amplitude;
      if (glow.flame) glow.flame.scale.y = 1 + flicker * .06;
    }
    renderer.render(scene, camera);
    requestAnimationFrame(animate);
  };
  animate();

  return { present, setLookProfile, renderer, scene, camera };
}
