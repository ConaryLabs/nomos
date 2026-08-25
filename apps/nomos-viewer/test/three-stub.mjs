// A recording stand-in for the Three.js namespace.
//
// `src/render.mjs` takes the namespace as a parameter, so the whole builder set
// runs in node against this and the scene graph can be asserted. It records
// what was constructed and how it was placed; it draws nothing.

class Vec3 {
  constructor(x = 0, y = 0, z = 0) {
    this.x = x;
    this.y = y;
    this.z = z;
  }

  set(x, y, z) {
    this.x = x;
    this.y = y;
    this.z = z;
    return this;
  }

  copy(other) {
    return this.set(other.x, other.y, other.z);
  }

  multiplyScalar(factor) {
    return this.set(this.x * factor, this.y * factor, this.z * factor);
  }
}

class Node {
  constructor(type, args = []) {
    this.type = type;
    this.args = args;
    this.children = [];
    this.position = new Vec3();
    this.rotation = new Vec3();
    this.scale = new Vec3(1, 1, 1);
  }

  add(...nodes) {
    this.children.push(...nodes);
    return this;
  }

  remove(node) {
    this.children = this.children.filter((one) => one !== node);
    return this;
  }

  traverse(visit) {
    visit(this);
    for (const child of this.children) child.traverse?.(visit);
  }

  lookAt() {
    return this;
  }

  updateProjectionMatrix() {}
}

class Geometry extends Node {
  constructor(type, args) {
    super(type, args);
    this.disposed = false;
  }

  center() {
    return this;
  }

  setFromPoints(points) {
    this.points = points;
    return this;
  }

  dispose() {
    this.disposed = true;
  }
}

class Material extends Node {
  constructor(type, options = {}) {
    super(type, [options]);
    Object.assign(this, options);
    this.disposed = false;
  }

  dispose() {
    this.disposed = true;
  }
}

class Mesh extends Node {
  constructor(geometry, material) {
    super("Mesh", []);
    this.geometry = geometry;
    this.material = material;
  }
}

class Light extends Node {
  constructor(type, args) {
    super(type, args);
    this.intensity = args[1] ?? 1;
    this.shadow = { mapSize: new Vec3(), camera: {} };
    this.shadow.mapSize.set = (x, y) => {
      this.shadow.mapSize.x = x;
      this.shadow.mapSize.y = y;
    };
  }
}

const geometry = (type) =>
  class extends Geometry {
    constructor(...args) {
      super(type, args);
    }
  };

const material = (type) =>
  class extends Material {
    constructor(options) {
      super(type, options);
    }
  };

const light = (type) =>
  class extends Light {
    constructor(...args) {
      super(type, args);
    }
  };

export function makeThree() {
  const rendered = [];
  return {
    rendered,
    Vector3: Vec3,
    Color: class {
      constructor(value) {
        this.value = value;
      }
    },
    Group: class extends Node {
      constructor() {
        super("Group");
      }
    },
    Mesh,
    LineLoop: class extends Node {
      constructor(lineGeometry, lineMaterial) {
        super("LineLoop");
        this.geometry = lineGeometry;
        this.material = lineMaterial;
      }
    },
    Shape: class {
      moveTo() {
        return this;
      }

      lineTo() {
        return this;
      }

      closePath() {
        return this;
      }
    },
    BoxGeometry: geometry("BoxGeometry"),
    ExtrudeGeometry: geometry("ExtrudeGeometry"),
    PlaneGeometry: geometry("PlaneGeometry"),
    TorusGeometry: geometry("TorusGeometry"),
    CylinderGeometry: geometry("CylinderGeometry"),
    ConeGeometry: geometry("ConeGeometry"),
    IcosahedronGeometry: geometry("IcosahedronGeometry"),
    BufferGeometry: geometry("BufferGeometry"),
    MeshStandardMaterial: material("MeshStandardMaterial"),
    MeshBasicMaterial: material("MeshBasicMaterial"),
    LineBasicMaterial: material("LineBasicMaterial"),
    ShaderMaterial: material("ShaderMaterial"),
    PointLight: light("PointLight"),
    DirectionalLight: light("DirectionalLight"),
    HemisphereLight: light("HemisphereLight"),
    GridHelper: class extends Node {
      constructor(...args) {
        super("GridHelper", args);
      }
    },
    FogExp2: class {
      constructor(color, density) {
        this.color = color;
        this.density = density;
      }
    },
    Scene: class extends Node {
      constructor() {
        super("Scene");
      }
    },
    OrthographicCamera: class extends Node {
      constructor(left, right, top, bottom, near, far) {
        super("OrthographicCamera", [left, right, top, bottom, near, far]);
        Object.assign(this, { left, right, top, bottom, near, far });
      }
    },
    WebGLRenderer: class {
      constructor(options) {
        this.options = options;
        this.domElement = { tagName: "CANVAS" };
        this.shadowMap = {};
        this.calls = [];
      }

      setPixelRatio(value) {
        this.pixelRatio = value;
      }

      setClearColor(value) {
        this.clearColor = value;
      }

      setSize(width, height) {
        this.size = { width, height };
      }

      render(scene, camera) {
        rendered.push({ scene, camera });
      }
    },
    Clock: class {
      getElapsedTime() {
        return 0;
      }
    },
    AdditiveBlending: "AdditiveBlending",
    DoubleSide: "DoubleSide",
    BackSide: "BackSide",
    PCFSoftShadowMap: "PCFSoftShadowMap",
    SRGBColorSpace: "SRGBColorSpace",
    ACESFilmicToneMapping: "ACESFilmicToneMapping",
  };
}

/// A container and the browser globals `render.mjs` asks its host for.
export function makeHost() {
  const container = {
    clientWidth: 1280,
    clientHeight: 576,
    children: [],
    replaceChildren(...nodes) {
      this.children = nodes;
    },
  };
  return {
    container,
    host: {
      devicePixelRatio: 1,
      // No frames: the renderer must build its world on `present`, not on a
      // scheduled callback.
      requestAnimationFrame: () => 0,
    },
  };
}

/// Counts nodes of each `type` under a root.
export function census(root) {
  const counts = new Map();
  root.traverse((node) => counts.set(node.type, (counts.get(node.type) ?? 0) + 1));
  return counts;
}

/// Every node under a root whose geometry is of `type`.
export function meshesOf(root, type) {
  const found = [];
  root.traverse((node) => {
    if (node.type === "Mesh" && node.geometry?.type === type) found.push(node);
  });
  return found;
}
