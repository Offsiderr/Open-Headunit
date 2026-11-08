<script lang="ts" module>
  export type Doors = { left: boolean; right: boolean };
</script>

<script lang="ts">
  import * as THREE from "three";
  import { onMount } from "svelte";
  import { GLTFLoader } from "three/addons/loaders/GLTFLoader.js";
  import { MeshoptDecoder } from "three/addons/libs/meshopt_decoder.module.js";
  import { OrbitControls } from "three/addons/controls/OrbitControls.js";
  import { RoomEnvironment } from "three/addons/environments/RoomEnvironment.js";

  let {
    active,
    kmh,
    doors,
    fps = $bindable(0),
  }: { active: boolean; kmh: number; doors: Doors; fps?: number } = $props();

  const MODEL_URL = "/models/brz.glb";
  const LENGTH = 4.2;
  const WHEEL_RADIUS = 0.32;
  const FORWARD = new THREE.Vector3(0, 0, 1);
  const AXLE = new THREE.Vector3(0, 1, 0).cross(FORWARD);
  const WHEEL = /wheel|tire|tyre/i;
  const STEER = /steer/i;
  const DOOR = /^S1:?(?:Door|MirrorBase|Mirror)([LR])/;
  const DOOR_OPEN = 1.1;
  const DOOR_RATE = 5;
  const BG = 0x0b0d10;
  const FADE_START = 1.5;
  const FADE_END = 12;
  const GRID_ALPHA = 0.35;

  // Floor opacity by distance from the car, so it dissolves into the background.
  const fade = (x: number, z: number) => 1 - THREE.MathUtils.smoothstep(Math.hypot(x, z), FADE_START, FADE_END);

  function floor() {
    const geo = new THREE.RingGeometry(0, FADE_END, 64, 48).rotateX(-Math.PI / 2);
    const p = geo.attributes.position;
    const rgba = new Float32Array(p.count * 4).fill(1);
    for (let i = 0; i < p.count; i++) rgba[i * 4 + 3] = fade(p.getX(i), p.getZ(i));
    geo.setAttribute("color", new THREE.BufferAttribute(rgba, 4));
    const mesh = new THREE.Mesh(
      geo,
      new THREE.MeshStandardMaterial({
        color: 0x232830,
        roughness: 0.85,
        envMapIntensity: 0.3,
        vertexColors: true,
        transparent: true,
        depthWrite: false,
      }),
    );
    mesh.receiveShadow = true;
    return mesh;
  }

  // 1 m grid split into unit segments so each vertex can carry its own fade.
  function grid() {
    const n = Math.ceil(FADE_END);
    const pos: number[] = [];
    const rgba: number[] = [];
    const add = (x0: number, z0: number, x1: number, z1: number) => {
      const a0 = fade(x0, z0) * GRID_ALPHA;
      const a1 = fade(x1, z1) * GRID_ALPHA;
      if (!a0 && !a1) return;
      pos.push(x0, 0.002, z0, x1, 0.002, z1);
      rgba.push(1, 1, 1, a0, 1, 1, 1, a1);
    };
    for (let i = -n; i <= n; i++)
      for (let j = -n; j < n; j++) {
        add(i, j, i, j + 1);
        add(j, i, j + 1, i);
      }
    const geo = new THREE.BufferGeometry();
    geo.setAttribute("position", new THREE.Float32BufferAttribute(pos, 3));
    geo.setAttribute("color", new THREE.Float32BufferAttribute(rgba, 4));
    return new THREE.LineSegments(
      geo,
      new THREE.LineBasicMaterial({ color: 0x59626e, vertexColors: true, transparent: true, depthWrite: false }),
    );
  }

  type Wheel = { spinner: THREE.Object3D; axis: THREE.Vector3 };
  type Door = { hinge: THREE.Object3D; side: keyof Doors; sign: number; angle: number };

  let container: HTMLDivElement;
  let renderer: THREE.WebGLRenderer;
  let loop: ((time: number) => void) | undefined = $state();

  function logTree(root: THREE.Object3D) {
    const lines: string[] = [];
    const walk = (o: THREE.Object3D, depth: number) => {
      lines.push(`${"  ".repeat(depth)}${o.name || "(unnamed)"} [${o.type}]`);
      for (const c of o.children) walk(c, depth + 1);
    };
    walk(root, 0);
    console.log(lines.join("\n"));
  }

  function dispose(root: THREE.Object3D) {
    root.traverse((o) => {
      if (!(o instanceof THREE.Mesh)) return;
      o.geometry.dispose();
      for (const m of [o.material].flat()) {
        for (const v of Object.values(m)) if (v instanceof THREE.Texture) v.dispose();
        m.dispose();
      }
    });
  }

  function commonAncestor(nodes: THREE.Object3D[]) {
    const contains = (a: THREE.Object3D, o: THREE.Object3D) => {
      for (let p = o.parent; p; p = p.parent) if (p === a) return true;
      return false;
    };
    let a = nodes[0].parent!;
    while (!nodes.every((n) => contains(a, n))) a = a.parent!;
    return a;
  }

  // Bounds of nodes in `frame`'s local space, so a cambered wheel stays axis-aligned.
  function localBox(nodes: THREE.Object3D[], frame: THREE.Object3D) {
    const inv = frame.matrixWorld.clone().invert();
    const box = new THREE.Box3();
    const part = new THREE.Box3();
    const m = new THREE.Matrix4();
    for (const n of nodes)
      n.traverse((o) => {
        if (!(o instanceof THREE.Mesh)) return;
        if (!o.geometry.boundingBox) o.geometry.computeBoundingBox();
        box.union(part.copy(o.geometry.boundingBox!).applyMatrix4(m.multiplyMatrices(inv, o.matrixWorld)));
      });
    return box;
  }

  function pivot(parent: THREE.Object3D, at: THREE.Vector3, nodes: THREE.Object3D[]) {
    const g = new THREE.Group();
    g.position.copy(at);
    parent.add(g);
    g.updateMatrixWorld();
    for (const n of nodes) g.attach(n);
    return g;
  }

  // Deepest matching nodes per corner, re-parented under a spinner at the hub that
  // turns about the wheel's own cambered axle (its thinnest local extent).
  function findWheels(model: THREE.Object3D): Wheel[] {
    const isWheel = (o: THREE.Object3D) => WHEEL.test(o.name) && !STEER.test(o.name);
    const groups = new Map<string, THREE.Object3D[]>();
    const box = new THREE.Box3();
    const c = new THREE.Vector3();
    model.traverse((o) => {
      if (!isWheel(o)) return;
      let deeper = false;
      o.traverse((d) => (deeper ||= d !== o && isWheel(d)));
      if (deeper) return;
      box.setFromObject(o).getCenter(c);
      const key = `${Math.sign(c.x)},${Math.sign(c.z)}`;
      groups.set(key, [...(groups.get(key) ?? []), o]);
    });

    return [...groups.values()].map((nodes) => {
      const parent = commonAncestor(nodes);
      const local = localBox(nodes, parent);
      const s = local.getSize(new THREE.Vector3());
      const axis = new THREE.Vector3();
      axis.setComponent(s.x <= s.y && s.x <= s.z ? 0 : s.y <= s.z ? 1 : 2, 1);
      if (axis.clone().transformDirection(parent.matrixWorld).dot(AXLE) < 0) axis.negate();
      return { spinner: pivot(parent, local.getCenter(new THREE.Vector3()), nodes), axis };
    });
  }

  // Door panel, glass and mirror per side, hinged vertically at the front outer edge.
  function findDoors(model: THREE.Object3D): Door[] {
    const sides = new Map<string, THREE.Object3D[]>();
    model.traverse((o) => {
      const side = DOOR.exec(o.name)?.[1];
      if (side && !DOOR.test(o.parent!.name)) sides.set(side, [...(sides.get(side) ?? []), o]);
    });
    return [...sides].map(([side, nodes]) => {
      const box = new THREE.Box3();
      for (const n of nodes) box.expandByObject(n);
      const outer = box.max.x > -box.min.x ? box.max.x : box.min.x;
      const at = model.worldToLocal(new THREE.Vector3(outer, (box.min.y + box.max.y) / 2, box.max.z));
      return { hinge: pivot(model, at, nodes), side: side === "L" ? "left" : "right", sign: -Math.sign(outer), angle: 0 };
    });
  }

  onMount(() => {
    renderer = new THREE.WebGLRenderer({ antialias: true });
    renderer.setPixelRatio(Math.min(devicePixelRatio, 1.5));
    renderer.toneMapping = THREE.ACESFilmicToneMapping;
    renderer.shadowMap.enabled = true;
    renderer.shadowMap.type = THREE.PCFShadowMap;
    renderer.shadowMap.autoUpdate = false;
    container.appendChild(renderer.domElement);

    const scene = new THREE.Scene();
    scene.background = new THREE.Color(BG);

    const pmrem = new THREE.PMREMGenerator(renderer);
    const room = new RoomEnvironment();
    const envMap = pmrem.fromScene(room, 0.04).texture;
    room.dispose();
    pmrem.dispose();
    scene.environment = envMap;
    scene.environmentIntensity = 0.6;

    scene.add(new THREE.HemisphereLight(0xbfd4ff, BG, 0.6));

    const key = new THREE.DirectionalLight(0xffffff, 2.2);
    key.position.set(3, 8, 4);
    key.castShadow = true;
    key.shadow.mapSize.set(1024, 1024);
    key.shadow.camera.left = key.shadow.camera.bottom = -4;
    key.shadow.camera.right = key.shadow.camera.top = 4;
    key.shadow.camera.far = 20;
    key.shadow.radius = 6;
    key.shadow.bias = -0.0005;
    scene.add(key);

    const rim = new THREE.DirectionalLight(0x9db8ff, 1.2);
    rim.position.set(-4, 3, -6);
    scene.add(rim);

    const camera = new THREE.PerspectiveCamera(45, 1, 0.1, 100);
    camera.position.set(3.6, 1.6, 4.4);

    const controls = new OrbitControls(camera, renderer.domElement);
    controls.target.set(0, 0.6, 0);
    controls.enableDamping = true;
    controls.enablePan = false;
    controls.minDistance = 3;
    controls.maxDistance = 10;
    controls.maxPolarAngle = Math.PI / 2 - 0.05;

    const ground = floor();
    const lines = grid();

    let model: THREE.Object3D | undefined;
    let wheels: Wheel[] = [];
    let doorList: Door[] = [];
    let disposed = false;

    new GLTFLoader().setMeshoptDecoder(MeshoptDecoder).load(
      MODEL_URL,
      (gltf) => {
        if (disposed) return dispose(gltf.scene);
        model = gltf.scene;
        logTree(model);

        const box = new THREE.Box3().setFromObject(model);
        const size = box.getSize(new THREE.Vector3());
        model.scale.multiplyScalar(LENGTH / Math.max(size.x, size.z));
        box.setFromObject(model);
        const center = box.getCenter(new THREE.Vector3());
        model.position.sub(new THREE.Vector3(center.x, box.min.y, center.z));
        model.updateMatrixWorld(true);

        wheels = findWheels(model);
        doorList = findDoors(model);
        model.traverse((o) => {
          if (o instanceof THREE.Mesh) o.castShadow = true;
        });
        scene.add(ground, lines, model);
        renderer.shadowMap.needsUpdate = true;
      },
      undefined,
      (err) => console.error(`Failed to load ${MODEL_URL}`, err),
    );

    const resize = new ResizeObserver(() => {
      const { clientWidth: w, clientHeight: h } = container;
      if (!w || !h) return;
      renderer.setSize(w, h);
      camera.aspect = w / h;
      camera.updateProjectionMatrix();
    });
    resize.observe(container);

    let last = 0;
    let angle = 0;
    let frames = 0;
    let fpsStart = 0;
    loop = (time) => {
      if (time - last > 1000) {
        frames = 0;
        fpsStart = time;
      }
      const dt = Math.min((time - last) / 1000, 0.1);
      last = time;

      angle = (angle + (kmh / 3.6 / WHEEL_RADIUS) * dt) % (2 * Math.PI);
      for (const w of wheels) w.spinner.quaternion.setFromAxisAngle(w.axis, angle);

      const k = 1 - Math.exp(-DOOR_RATE * dt);
      for (const d of doorList) {
        const delta = (doors[d.side] ? DOOR_OPEN : 0) - d.angle;
        if (Math.abs(delta) < 1e-4) continue;
        d.angle += delta * k;
        d.hinge.rotation.y = d.sign * d.angle;
        renderer.shadowMap.needsUpdate = true;
      }

      controls.update(dt);
      renderer.render(scene, camera);

      frames++;
      if (time - fpsStart >= 1000) {
        fps = Math.round((frames * 1000) / (time - fpsStart));
        frames = 0;
        fpsStart = time;
      }
    };

    return () => {
      disposed = true;
      resize.disconnect();
      renderer.setAnimationLoop(null);
      controls.dispose();
      dispose(ground);
      lines.geometry.dispose();
      lines.material.dispose();
      if (model) dispose(model);
      envMap.dispose();
      renderer.dispose();
    };
  });

  $effect(() => {
    if (loop) renderer.setAnimationLoop(active ? loop : null);
  });
</script>

<div class="scene" bind:this={container}></div>

<style>
  .scene {
    position: absolute;
    inset: 0;
  }
</style>
