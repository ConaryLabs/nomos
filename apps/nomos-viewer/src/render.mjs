// The WebGL renderer.
//
// Three.js arrives as a parameter rather than an import. That is not a
// convenience: it is what lets `test/render.test.mjs` drive the whole builder
// set with a recording stub and assert the scene graph, which is the difference
// between promoting `experiments/executable-gaol/src/webgl-renderer.mjs` and
// re-typing it. The study could only asserts its own source text
// (`src/webgl-viewer.test.mjs`), because a CDN import cannot be tested in node.
//
// Dispatch is on catalog entries. There is no entity id, no assembly string,
// and no area name in this file; the builder for an entity is looked up by the
// assembly the catalog says its kind is drawn as, and an entity the catalog
// does not know never reaches here — `plan.mjs` refuses it first.

import {
  ACTOR_SHAPES,
  CAMERA,
  DEFAULT_LOOK_PROFILE,
  ENTITY_KINDS,
  LOOK_PROFILES,
  LOOK_PROFILE_IDS,
  PALETTE,
  VERTICAL_SCALE,
  cellsOf,
  resolveSocket,
} from "./catalog.mjs";
import { doorState, lightOf, scenarioOf, wardSealed } from "./plan.mjs";

const shadow = (object, cast = true, receive = true) => {
  object.castShadow = cast;
  object.receiveShadow = receive;
  return object;
};

// A standard material, optionally carrying the look profile's procedural
// surface treatment. The shader injection is the study's, unchanged: coarse
// deterministic cell variation plus an accent band, no bitmap texture and no
// generated image asset anywhere.
const material = (three, color, roughness = 0.78, metalness = 0.04, surface = null) => {
  const result = new three.MeshStandardMaterial({ color, roughness, metalness });
  if (!surface) return result;
  result.onBeforeCompile = (shader) => {
    shader.uniforms.uLookScale = { value: surface.scale };
    shader.uniforms.uLookVariation = { value: surface.variation };
    shader.uniforms.uLookAccent = { value: new three.Color(surface.accent) };
    shader.uniforms.uLookAccentMix = { value: surface.accentMix };
    shader.vertexShader = shader.vertexShader
      .replace("#include <common>", "#include <common>\nvarying vec3 vLookWorldPosition;")
      .replace(
        "#include <begin_vertex>",
        "#include <begin_vertex>\nvLookWorldPosition = (modelMatrix * vec4(transformed, 1.0)).xyz;",
      );
    shader.fragmentShader = shader.fragmentShader
      .replace(
        "#include <common>",
        `#include <common>
        varying vec3 vLookWorldPosition;
        uniform float uLookScale;
        uniform float uLookVariation;
        uniform vec3 uLookAccent;
        uniform float uLookAccentMix;
        float lookHash(vec3 cell) {
          return fract(sin(dot(cell, vec3(12.9898, 78.233, 45.164))) * 43758.5453);
        }`,
      )
      .replace(
        "#include <color_fragment>",
        `#include <color_fragment>
        vec3 lookCell = floor(vLookWorldPosition * uLookScale);
        float lookValue = lookHash(lookCell);
        float lookBand = floor(lookValue * 4.0) / 3.0;
        diffuseColor.rgb *= mix(1.0 - uLookVariation, 1.0 + uLookVariation, lookBand);
        float lookAccent = step(0.78, lookValue) * uLookAccentMix;
        diffuseColor.rgb = mix(diffuseColor.rgb, uLookAccent, lookAccent);`,
      );
  };
  result.customProgramCacheKey = () => JSON.stringify(surface);
  return result;
};

const makeResources = (three, look) => ({
  stone: [
    material(three, PALETTE.stone_0, 0.78, 0.04, look.materials.stone),
    material(three, PALETTE.stone_1, 0.78, 0.04, look.materials.stone),
    material(three, PALETTE.stone_2, 0.7, 0.04, look.materials.stone),
  ],
  iron: material(three, PALETTE.iron, 0.42, 0.7, look.materials.iron),
  rust: material(three, PALETTE.rust, 0.68, 0.42, look.materials.iron),
  waterBed: material(three, PALETTE.water, 0.3, 0.18),
  player: material(three, PALETTE.player, 0.62, 0.04, look.materials.cloth),
  gaoler: material(three, PALETTE.gaoler, 0.74, 0.04, look.materials.cloth),
  skin: material(three, PALETTE.skin, 0.85),
  cyanMetal: material(three, PALETTE.cyan, 0.25, 0.6),
  outline: new three.MeshBasicMaterial({ color: PALETTE.void, side: three.BackSide }),
});

const disposeTree = (root) =>
  root.traverse((object) => {
    object.geometry?.dispose();
    if (Array.isArray(object.material)) object.material.forEach((entry) => entry.dispose());
    else object.material?.dispose();
  });

const cellPosition = (three, plan, cell, elevation = 0) =>
  new three.Vector3(
    cell.x + 0.5 - plan.architecture.bounds.width / 2,
    elevation,
    cell.y + 0.5 - plan.architecture.bounds.height / 2,
  );

const addBox = (three, parent, size, position, boxMaterial, options = {}) => {
  let geometry;
  const bevel = Math.min(options.bevel ?? 0, Math.min(...size) / 4);
  if (bevel > 0) {
    const [width, height, depth] = size;
    const shape = new three.Shape()
      .moveTo(-width / 2 + bevel, -height / 2 + bevel)
      .lineTo(width / 2 - bevel, -height / 2 + bevel)
      .lineTo(width / 2 - bevel, height / 2 - bevel)
      .lineTo(-width / 2 + bevel, height / 2 - bevel)
      .closePath();
    geometry = new three.ExtrudeGeometry(shape, {
      depth: Math.max(depth - bevel * 2, bevel),
      bevelEnabled: true,
      bevelSegments: 1,
      bevelSize: bevel,
      bevelThickness: bevel,
      curveSegments: 1,
    });
    geometry.center();
  } else geometry = new three.BoxGeometry(...size);
  const mesh = shadow(new three.Mesh(geometry, boxMaterial));
  mesh.position.set(...position);
  if (options.rotationY) mesh.rotation.y = options.rotationY;
  parent.add(mesh);
  return mesh;
};

const addStoneSegment = (three, parent, x, z, height, axis, stoneMaterials) => {
  const courses = Math.ceil(height / 0.46);
  for (let course = 0; course < courses; course += 1) {
    const courseHeight = Math.min(0.42, height - course * 0.46);
    if (courseHeight <= 0) continue;
    const offset = course % 2 ? 0.13 : -0.13;
    const size = axis === "x" ? [0.94, courseHeight, 0.34] : [0.34, courseHeight, 0.94];
    const position =
      axis === "x"
        ? [x + offset, course * 0.46 + courseHeight / 2, z]
        : [x, course * 0.46 + courseHeight / 2, z + offset];
    addBox(three, parent, size, position, stoneMaterials[(course + Math.round(x + z)) & 1]);
  }
  const capSize = axis === "x" ? [1.02, 0.12, 0.44] : [0.44, 0.12, 1.02];
  addBox(three, parent, capSize, [x, height + 0.06, z], stoneMaterials[2]);
};

const waterMaterial = (three) =>
  new three.ShaderMaterial({
    uniforms: {
      uTime: { value: 0 },
      uDeep: { value: new three.Color(PALETTE.water_deep) },
      uLight: { value: new three.Color(PALETTE.water_light) },
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
    side: three.DoubleSide,
  });

const buildFloor = (three, root, plan, resources) => {
  const { width, height } = plan.architecture.bounds;
  for (let y = 0; y < height; y += 1) {
    for (let x = 0; x < width; x += 1) {
      const floor = addBox(
        three,
        root,
        [0.96, 0.12, 0.96],
        [x + 0.5 - width / 2, -0.08, y + 0.5 - height / 2],
        resources.stone[(x + y) & 1],
      );
      floor.castShadow = false;
    }
  }
};

// The wall a door interrupts comes from the entity's declared face, not from a
// `cell.y === 0` inference. Every corpus door faces north; the walls now read
// the field that has always carried the answer, for all four faces.
const buildWalls = (three, root, plan, resources) => {
  const { width, height } = plan.architecture.bounds;
  const wallHeight = cellsOf(plan.architecture.wall_height_steps) * VERTICAL_SCALE;
  const faces = plan.entities.filter((one) => one.anchor.kind === "face");
  const openings = (direction, axis) =>
    new Set(
      faces.filter((one) => one.anchor.direction === direction).map((one) => one.anchor.cell[axis]),
    );
  const northOpen = openings("north", "x");
  const southOpen = openings("south", "x");
  const westOpen = openings("west", "y");
  const eastOpen = openings("east", "y");
  for (let x = 0; x < width; x += 1) {
    const at = x + 0.5 - width / 2;
    if (!northOpen.has(x)) {
      addStoneSegment(three, root, at, -height / 2 - 0.18, wallHeight, "x", resources.stone);
    }
    if (!southOpen.has(x)) {
      addStoneSegment(three, root, at, height / 2 + 0.18, 0.32, "x", resources.stone);
    }
  }
  for (let y = 0; y < height; y += 1) {
    const at = y + 0.5 - height / 2;
    if (!westOpen.has(y)) {
      addStoneSegment(three, root, -width / 2 - 0.18, at, wallHeight, "z", resources.stone);
    }
    if (!eastOpen.has(y)) {
      addStoneSegment(three, root, width / 2 + 0.18, at, 0.32, "z", resources.stone);
    }
  }
};

const buildMasses = (three, root, plan, resources, look) => {
  for (const mass of plan.architecture.masses) {
    const width = mass.max.x - mass.min.x;
    const depth = mass.max.y - mass.min.y;
    const height = cellsOf(mass.height_steps) * VERTICAL_SCALE;
    const x = (mass.min.x + mass.max.x) / 2 - plan.architecture.bounds.width / 2;
    const z = (mass.min.y + mass.max.y) / 2 - plan.architecture.bounds.height / 2;
    addBox(three, root, [width - 0.08, height, depth - 0.08], [x, height / 2, z], resources.stone[1], {
      bevel: look.bevel,
    });
    addBox(three, root, [width + 0.03, 0.12, depth + 0.03], [x, height + 0.06, z], resources.stone[2], {
      bevel: look.bevel * 0.65,
    });
  }
};

const buildWater = (three, root, plan, entity, resources, context) => {
  const width = entity.anchor.max.x - entity.anchor.min.x + 1;
  const depth = entity.anchor.max.y - entity.anchor.min.y + 1;
  const x = (entity.anchor.min.x + entity.anchor.max.x + 1) / 2 - plan.architecture.bounds.width / 2;
  const z = (entity.anchor.min.y + entity.anchor.max.y + 1) / 2 - plan.architecture.bounds.height / 2;
  addBox(three, root, [width - 0.09, 0.14, depth - 0.09], [x, 0.015, z], resources.waterBed);
  const shader = waterMaterial(three);
  context.animatedMaterials.push(shader);
  const surface = new three.Mesh(
    new three.PlaneGeometry(width - 0.1, depth - 0.1, width * 6, depth * 6),
    shader,
  );
  surface.rotation.x = -Math.PI / 2;
  surface.position.set(x, 0.105, z);
  surface.receiveShadow = true;
  root.add(surface);
};

const buildDoor = (three, root, plan, entity, resources, context) => {
  const { scenario, look } = context;
  const group = new three.Group();
  group.position.copy(cellPosition(three, plan, entity.anchor.cell));
  group.position.z -= 0.38;
  const { access, integrity, ward } = doorState(scenario, entity.id);

  addBox(three, group, [0.25, 2.45, 0.48], [-0.58, 1.22, 0], resources.stone[2], { bevel: look.bevel });
  addBox(three, group, [0.25, 2.45, 0.48], [0.58, 1.22, 0], resources.stone[2], { bevel: look.bevel });
  addBox(three, group, [1.42, 0.3, 0.5], [0, 2.38, 0], resources.stone[2], { bevel: look.bevel });
  addBox(three, group, [1.68, 0.13, 0.58], [0, 2.6, 0], resources.stone[1], { bevel: look.bevel * 0.65 });

  if (integrity === "destroyed") {
    for (const [x, y, angle] of [
      [-0.34, 0.65, -0.16],
      [0.02, 0.42, 0.21],
      [0.35, 0.78, -0.25],
    ]) {
      const bar = addBox(three, group, [0.075, 1.18, 0.08], [x, y, 0], resources.iron);
      bar.rotation.z = angle;
    }
  } else {
    const slide = access === "open" ? 0.62 : 0;
    for (let x = -0.42; x <= 0.43; x += 0.21) {
      addBox(three, group, [0.065, 1.92, 0.07], [x + slide, 1.12, 0], resources.iron);
    }
    addBox(three, group, [1.02, 0.09, 0.09], [slide, 1.12, 0], resources.rust);
  }

  if (ward === "sealed") {
    const wardMaterial = new three.MeshBasicMaterial({
      color: PALETTE.cyan,
      transparent: true,
      opacity: 0.72,
      blending: three.AdditiveBlending,
    });
    const ring = new three.Mesh(new three.TorusGeometry(0.42, 0.025, 10, 48), wardMaterial);
    ring.position.set(0, 1.22, 0.1);
    group.add(ring);
    const diamond = new three.LineLoop(
      new three.BufferGeometry().setFromPoints([
        new three.Vector3(0, 1.78, 0.11),
        new three.Vector3(0.42, 1.22, 0.11),
        new three.Vector3(0, 0.66, 0.11),
        new three.Vector3(-0.42, 1.22, 0.11),
      ]),
      new three.LineBasicMaterial({ color: PALETTE.cyan, transparent: true, opacity: 0.8 }),
    );
    group.add(diamond);
    const wardLight = new three.PointLight(PALETTE.cyan, 3.4, 3.2, 2);
    wardLight.position.set(0, 1.22, 0.45);
    group.add(wardLight);
    context.glowLights.push({
      light: wardLight,
      phase: entity.anchor.cell.x * 0.7,
      base: 3.4,
      amplitude: 0.25,
    });
  }
  root.add(group);
};

const buildBrazier = (three, root, plan, entity, resources, context) => {
  const group = new three.Group();
  group.position.copy(cellPosition(three, plan, entity.anchor.cell));
  const lit = lightOf(context.scenario, entity.id) === true;
  const pedestal = shadow(
    new three.Mesh(new three.CylinderGeometry(0.16, 0.23, 0.48, 8), resources.iron),
  );
  pedestal.position.y = 0.24;
  group.add(pedestal);
  const bowl = shadow(new three.Mesh(new three.CylinderGeometry(0.34, 0.18, 0.18, 8), resources.rust));
  bowl.position.y = 0.55;
  group.add(bowl);
  if (lit) {
    const flameMaterial = new three.MeshBasicMaterial({
      color: PALETTE.amber,
      transparent: true,
      opacity: 0.92,
      blending: three.AdditiveBlending,
    });
    const flame = new three.Mesh(new three.ConeGeometry(0.17, 0.55, 7), flameMaterial);
    flame.position.y = 0.92;
    flame.scale.z = 0.72;
    group.add(flame);
    const light = new three.PointLight(PALETTE.amber, 8.5, 5.6, 2);
    light.position.y = 1.12;
    light.castShadow = true;
    light.shadow.mapSize.set(CAMERA.lightShadowMapSize, CAMERA.lightShadowMapSize);
    group.add(light);
    context.glowLights.push({
      light,
      flame,
      phase: entity.anchor.cell.x + entity.anchor.cell.y,
      base: 8.5,
      amplitude: 1.1,
    });
  }
  root.add(group);
};

// One builder per assembly the catalog declares. The keys are catalog values,
// not literals typed here, so an assembly that gains a builder gains it in one
// place and a plan naming an assembly with no builder was already refused by
// the decoder.
const ENTITY_BUILDERS = Object.freeze({
  [ENTITY_KINDS.door.visualAssembly]: buildDoor,
  [ENTITY_KINDS.water.visualAssembly]: buildWater,
  [ENTITY_KINDS.light.visualAssembly]: buildBrazier,
});

// Effects are placed by the catalog's socket table, not by a coordinate in
// content, and all three components of the socket are honoured: a renderer that
// dropped the elevation would be the wall-height double authority again in a
// new field.
const buildEffects = (three, root, plan, context) => {
  const { width, height } = plan.architecture.bounds;
  for (const effect of plan.effects) {
    const host = plan.entities.find((one) => one.id === effect.anchor.entity);
    if (!wardSealed(context.scenario, host.id)) continue;
    const socket = resolveSocket(host, effect.anchor.socket);
    const crescentMaterial = new three.MeshBasicMaterial({
      color: PALETTE.cyan,
      transparent: true,
      opacity: 0.34,
      blending: three.AdditiveBlending,
    });
    const crescent = new three.Mesh(
      new three.TorusGeometry(0.38, 0.055, 8, 32, Math.PI * 1.25),
      crescentMaterial,
    );
    crescent.rotation.z = -0.45;
    crescent.position.set(socket.x - width / 2, socket.z * VERTICAL_SCALE, socket.y - height / 2);
    root.add(crescent);
  }
};

const addActorSilhouette = (three, group, mesh, look, resources) => {
  if (look.actorOutline <= 1) return;
  const outline = new three.Mesh(mesh.geometry, resources.outline);
  outline.position.copy(mesh.position);
  outline.rotation.copy(mesh.rotation);
  outline.scale.copy(mesh.scale).multiplyScalar(look.actorOutline);
  outline.castShadow = false;
  outline.receiveShadow = false;
  group.add(outline);
};

// The silhouettes differ by the assembly the plan declares, through the
// catalog's shape table. The study told them apart with `actor.id === "player"`,
// which audit section 3 item 21 recorded as the only role signal in the content
// model; this file holds no actor identifier at all.
const actorMesh = (three, actor, resources, look) => {
  const shape = ACTOR_SHAPES[actor.assembly];
  const group = new three.Group();
  const cloak = shadow(
    new three.Mesh(
      new three.ConeGeometry(shape.cloakRadius, shape.cloakHeight, 7),
      resources[shape.body],
    ),
  );
  cloak.position.y = shape.cloakY;
  addActorSilhouette(three, group, cloak, look, resources);
  group.add(cloak);
  const head = shadow(new three.Mesh(new three.IcosahedronGeometry(shape.headRadius, 1), resources.skin));
  head.position.y = shape.headY;
  addActorSilhouette(three, group, head, look, resources);
  group.add(head);
  if (shape.hand === "blade") {
    const blade = addBox(three, group, [0.055, 0.65, 0.055], [0.28, 0.58, 0], resources.cyanMetal);
    blade.rotation.z = -0.55;
  } else {
    const shoulder = addBox(three, group, [0.72, 0.11, 0.2], [0, 0.78, 0], resources.rust);
    shoulder.rotation.z = 0.04;
  }
  return group;
};

const buildActors = (three, root, plan, resources, context) => {
  for (const actor of plan.actors) {
    const mesh = actorMesh(three, actor, resources, context.look);
    const anchor = context.actorPositions?.[actor.id] ?? actor.cell;
    mesh.position.copy(cellPosition(three, plan, anchor, anchor.z ?? 0));
    context.actorMeshes.set(actor.id, mesh);
    root.add(mesh);
  }
};

/// Creates the renderer. `three` is the vendored namespace; `host` supplies the
/// three browser globals this file needs, so a test can pass its own.
export function createGaolRenderer(container, three, host = globalThis) {
  const renderer = new three.WebGLRenderer({
    antialias: true,
    alpha: false,
    powerPreference: "high-performance",
  });
  renderer.setPixelRatio(Math.min(host.devicePixelRatio ?? 1, CAMERA.maxPixelRatio));
  renderer.setClearColor(PALETTE.void);
  renderer.shadowMap.enabled = true;
  renderer.shadowMap.type = three.PCFSoftShadowMap;
  renderer.outputColorSpace = three.SRGBColorSpace;
  renderer.toneMapping = three.ACESFilmicToneMapping;
  let look = LOOK_PROFILES[DEFAULT_LOOK_PROFILE];
  renderer.toneMappingExposure = look.exposure;
  container.replaceChildren(renderer.domElement);

  const scene = new three.Scene();
  scene.background = new three.Color(PALETTE.void);
  scene.fog = new three.FogExp2(PALETTE.fog, look.fogDensity);
  const camera = new three.OrthographicCamera(
    -CAMERA.orthoHalfHeight * 2,
    CAMERA.orthoHalfHeight * 2,
    CAMERA.orthoHalfHeight,
    -CAMERA.orthoHalfHeight,
    CAMERA.near,
    CAMERA.far,
  );
  scene.add(new three.HemisphereLight(PALETTE.sky, PALETTE.ground, 2.05));
  const moon = new three.DirectionalLight(PALETTE.moon, 4.2);
  moon.position.set(-7, 11, 7);
  moon.castShadow = true;
  moon.shadow.mapSize.set(CAMERA.shadowMapSize, CAMERA.shadowMapSize);
  moon.shadow.camera.left = -CAMERA.shadowFrustum;
  moon.shadow.camera.right = CAMERA.shadowFrustum;
  moon.shadow.camera.top = CAMERA.shadowFrustum;
  moon.shadow.camera.bottom = -CAMERA.shadowFrustum;
  scene.add(moon);

  let worldRoot = new three.Group();
  scene.add(worldRoot);
  let actorMeshes = new Map();
  let animatedMaterials = [];
  let glowLights = [];
  let planIdentity;
  let scenarioIdentity;
  let forensicState = false;
  let grid;

  const fit = () => {
    const width = Math.max(container.clientWidth ?? 1, 1);
    const height = Math.max(container.clientHeight ?? 1, 1);
    renderer.setSize(width, height, false);
    const aspect = width / height;
    camera.left = -CAMERA.orthoHalfHeight * aspect;
    camera.right = CAMERA.orthoHalfHeight * aspect;
    camera.top = CAMERA.orthoHalfHeight;
    camera.bottom = -CAMERA.orthoHalfHeight;
    camera.updateProjectionMatrix();
  };
  if (host.ResizeObserver) new host.ResizeObserver(fit).observe(container);
  fit();

  const rebuild = (plan, scenarioId, actorPositions) => {
    scene.remove(worldRoot);
    disposeTree(worldRoot);
    worldRoot = new three.Group();
    scene.add(worldRoot);
    actorMeshes = new Map();
    animatedMaterials = [];
    glowLights = [];
    const resources = makeResources(three, look);
    const scenario = scenarioOf(plan, scenarioId);
    const context = { scenario, look, actorPositions, actorMeshes, animatedMaterials, glowLights };
    buildFloor(three, worldRoot, plan, resources);
    buildWalls(three, worldRoot, plan, resources);
    buildMasses(three, worldRoot, plan, resources, look);
    for (const entity of plan.entities) {
      ENTITY_BUILDERS[ENTITY_KINDS[entity.kind].visualAssembly](
        three,
        worldRoot,
        plan,
        entity,
        resources,
        context,
      );
    }
    buildEffects(three, worldRoot, plan, context);
    buildActors(three, worldRoot, plan, resources, context);
    const { width, height } = plan.architecture.bounds;
    camera.position.set(
      width * CAMERA.offset.x,
      Math.max(width, height) * CAMERA.offset.y,
      height * CAMERA.offset.z,
    );
    camera.lookAt(new three.Vector3(0, CAMERA.targetHeight, 0));
    planIdentity = plan.area.id;
    scenarioIdentity = scenario.id;
  };

  const updateActors = (plan, actorPositions) => {
    for (const actor of plan.actors) {
      const anchor = actorPositions?.[actor.id] ?? actor.cell;
      actorMeshes.get(actor.id)?.position.copy(cellPosition(three, plan, anchor, anchor.z ?? 0));
    }
  };

  const present = (plan, scenarioId, forensic = false, presentation = {}) => {
    if (planIdentity !== plan.area.id || scenarioIdentity !== scenarioId) {
      rebuild(plan, scenarioId, presentation.actorPositions);
    } else updateActors(plan, presentation.actorPositions);
    if (forensicState !== forensic) {
      forensicState = forensic;
      if (grid) worldRoot.remove(grid);
      const span = Math.max(plan.architecture.bounds.width, plan.architecture.bounds.height);
      grid = forensic ? new three.GridHelper(span, span, PALETTE.cyan, PALETTE.grid) : null;
      if (grid) {
        grid.position.y = 0.13;
        worldRoot.add(grid);
      }
    }
  };

  const setLookProfile = (profileId) => {
    if (!LOOK_PROFILE_IDS.includes(profileId)) throw new Error(`unknown look profile ${profileId}`);
    const next = LOOK_PROFILES[profileId];
    if (next === look) return;
    look = next;
    scene.fog.density = look.fogDensity;
    renderer.toneMappingExposure = look.exposure;
    planIdentity = undefined;
  };

  const clock = new three.Clock();
  let running = true;
  const animate = () => {
    if (!running) return;
    const elapsed = clock.getElapsedTime();
    for (const shader of animatedMaterials) shader.uniforms.uTime.value = elapsed;
    for (const glow of glowLights) {
      const flicker =
        Math.sin(elapsed * 8.7 + glow.phase) * 0.5 + Math.sin(elapsed * 13.1 + glow.phase * 0.7) * 0.5;
      glow.light.intensity = glow.base + flicker * glow.amplitude;
      if (glow.flame) glow.flame.scale.y = 1 + flicker * 0.06;
    }
    renderer.render(scene, camera);
    host.requestAnimationFrame?.(animate);
  };
  animate();

  return {
    present,
    setLookProfile,
    stop: () => {
      running = false;
    },
    renderer,
    scene,
    camera,
    worldRoot: () => worldRoot,
    lookProfile: () => look,
  };
}
