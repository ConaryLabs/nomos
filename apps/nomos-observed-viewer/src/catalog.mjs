const freeze = (value) => {
  if (value && typeof value === "object" && !Object.isFrozen(value)) {
    for (const child of Object.values(value)) freeze(child);
    Object.freeze(value);
  }
  return value;
};

export const CATALOG = freeze({
  viewport: { width: 1280, height: 720, pixel_ratio: 1 },
  renderer: {
    antialias: true,
    alpha: false,
    clear_color: "#171a1f",
    shadows: false,
    output_color_space: "srgb",
  },
  camera: {
    kind: "orthographic",
    azimuth_degrees: 45,
    elevation_radians: Math.atan(1 / Math.sqrt(2)),
    up: [0, 1, 0],
    distance: 64,
    near: 0.1,
    far: 256,
    margin_cells: 1,
  },
  terrain: {
    stack_offset: 0.004,
    visual_epsilon: 0.006,
    assemblies: {
      "terrain/calm_ground": {
        geometry: { kind: "box", width: 1, height: 0.08, depth: 1 },
        center_height: 0.04,
      },
      "terrain/traversable_route": {
        geometry: { kind: "box", width: 0.74, height: 0.06, depth: 0.74 },
        center_height: 0.03,
      },
      "terrain/structure_footprint": {
        geometry: { kind: "box", width: 0.62, height: 0.3, depth: 0.62 },
        center_height: 0.15,
      },
    },
    materials: {
      ground_muted: { color: "#46515b", roughness: 0.95, metalness: 0 },
      route_worn: { color: "#927958", roughness: 0.88, metalness: 0 },
      structure_stone: { color: "#6d7378", roughness: 0.82, metalness: 0.04 },
    },
    variation: {
      x_coefficient: 17,
      y_coefficient: 31,
      modulus: 16,
      selected_remainder: 0,
      geometry: { kind: "box", width: 0.3, height: 0.006, depth: 0.14 },
      colors: {
        ground_muted: "#56636d",
        route_worn: "#aa8d66",
        structure_stone: "#858b8f",
      },
    },
  },
  actor: {
    assemblies: {
      "actor/observed_figure": {
        body: {
          kind: "cylinder",
          radius_top: 0.16,
          radius_bottom: 0.22,
          height: 0.56,
          radial_segments: 8,
          center: [0, 0.34, 0],
        },
        head: { kind: "icosahedron", radius: 0.16, detail: 0, center: [0, 0.75, 0] },
        material: { color: "#b8b1a1", roughness: 0.68, metalness: 0.02 },
      },
    },
    poses: {
      upright_living: { offset: [0, 0, 0], rotation_z: 0 },
      prone_dead: { offset: [0.24, 0.2, 0], rotation_z: -Math.PI / 2 },
    },
    controlled_marker: {
      present: {
        kind: "ring",
        inner_radius: 0.31,
        outer_radius: 0.37,
        segments: 32,
        height: 0.012,
        color: "#72c7d9",
        opacity: 1,
      },
    },
    hostile_outline: {
      present: {
        kind: "torus",
        radius: 0.28,
        tube: 0.022,
        radial_segments: 8,
        tubular_segments: 24,
        center_height: 0.48,
        color: "#df6657",
        opacity: 1,
      },
    },
    protection_ring: {
      present: {
        kind: "torus",
        radius: 0.34,
        tube: 0.024,
        radial_segments: 8,
        tubular_segments: 32,
        center_height: 0.43,
        color: "#d8bd63",
        opacity: 1,
      },
    },
  },
  actions: {
    markers: {
      "action/enabled": {
        geometry: { kind: "cone", radius: 0.12, height: 0.24, radial_segments: 4 },
        color: "#76c98e",
      },
      "action/disabled": {
        geometry: { kind: "box", width: 0.17, height: 0.17, depth: 0.17 },
        color: "#767c84",
      },
    },
    anchor_height: 1.12,
    spacing_x: 0.24,
  },
  lights: {
    hemisphere: {
      sky_color: "#e8eef2",
      ground_color: "#25272b",
      intensity: 1.35,
    },
    directional: {
      color: "#fff0d8",
      intensity: 2.2,
      target_offset: [6, 10, -8],
    },
  },
  ui: {
    inset_px: 12,
    control_height_px: 34,
    control_min_width_px: 34,
    gap_px: 8,
    radius_px: 5,
    border_width_px: 1,
    background: "#20252bcc",
    border_color: "#59616a",
    text_color: "#e5e1d8",
    active_background: "#765f46",
    font: "600 14px system-ui,sans-serif",
    labels: ["1", "2"],
  },
});

export const sparseVariationSelected = (x, y, stack) =>
  ((CATALOG.terrain.variation.x_coefficient * x +
    CATALOG.terrain.variation.y_coefficient * y +
    stack) %
    CATALOG.terrain.variation.modulus +
    CATALOG.terrain.variation.modulus) %
    CATALOG.terrain.variation.modulus ===
  CATALOG.terrain.variation.selected_remainder;
