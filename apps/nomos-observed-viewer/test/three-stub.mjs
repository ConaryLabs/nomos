class Vector3 {
  constructor(x = 0, y = 0, z = 0) {
    this.set(x, y, z);
  }

  set(x, y, z) {
    Object.assign(this, { x, y, z });
    return this;
  }
}

class Node {
  constructor(type, args = []) {
    this.type = type;
    this.args = args;
    this.children = [];
    this.position = new Vector3();
    this.rotation = new Vector3();
    this.scale = new Vector3(1, 1, 1);
    this.up = new Vector3(0, 1, 0);
    this.userData = {};
  }

  add(...children) {
    this.children.push(...children);
    return this;
  }

  traverse(visit) {
    visit(this);
    this.children.forEach((child) => child.traverse(visit));
  }

  lookAt(...target) {
    this.lookAtTarget = target;
  }
}

class Geometry {
  constructor(type, args) {
    this.type = type;
    this.args = args;
  }
}

class Material {
  constructor(type, options) {
    this.type = type;
    this.options = options;
  }
}

const geometry = (type) => class extends Geometry {
  constructor(...args) {
    super(type, args);
  }
};

const material = (type) => class extends Material {
  constructor(options) {
    super(type, options);
  }
};

export const makeThree = () => ({
  DoubleSide: "double",
  SRGBColorSpace: "srgb",
  Vector3,
  Group: class extends Node { constructor() { super("Group"); } },
  Scene: class extends Node { constructor() { super("Scene"); } },
  Color: class { constructor(value) { this.value = value; } },
  Mesh: class extends Node {
    constructor(meshGeometry, meshMaterial) {
      super("Mesh");
      this.geometry = meshGeometry;
      this.material = meshMaterial;
    }
  },
  BoxGeometry: geometry("BoxGeometry"),
  CylinderGeometry: geometry("CylinderGeometry"),
  IcosahedronGeometry: geometry("IcosahedronGeometry"),
  RingGeometry: geometry("RingGeometry"),
  TorusGeometry: geometry("TorusGeometry"),
  ConeGeometry: geometry("ConeGeometry"),
  MeshStandardMaterial: material("MeshStandardMaterial"),
  MeshBasicMaterial: material("MeshBasicMaterial"),
  OrthographicCamera: class extends Node {
    constructor(left, right, top, bottom, near, far) {
      super("OrthographicCamera", [left, right, top, bottom, near, far]);
      Object.assign(this, { left, right, top, bottom, near, far });
    }
  },
  HemisphereLight: class extends Node {
    constructor(...args) {
      super("HemisphereLight", args);
    }
  },
  DirectionalLight: class extends Node {
    constructor(...args) {
      super("DirectionalLight", args);
      this.target = new Node("Object3D");
    }
  },
  WebGLRenderer: class {
    constructor(options) {
      this.options = options;
      this.domElement = { tagName: "CANVAS" };
      this.shadowMap = {};
    }

    setPixelRatio(value) { this.pixelRatio = value; }
    setSize(...value) { this.size = value; }
    setClearColor(...value) { this.clear = value; }
    render(...value) { this.rendered = value; }
    dispose() { this.disposed = true; }
  },
});
