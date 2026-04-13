import "./styles.css";

import * as THREE from "three";
import { OrbitControls } from "three/examples/jsm/controls/OrbitControls.js";

const statusEl = document.getElementById("status");
const statsEl = document.getElementById("stats");
const viewerEl = document.getElementById("viewer");

const setStatus = (text, state = "info") => {
  statusEl.textContent = text;
  statusEl.dataset.state = state;
};

const runtimeUrl = (fileName) => new URL(fileName, document.baseURI).toString();

const loadOcctFactory = async () => {
  if (typeof window.createOcctThreeDemo === "function") {
    return window.createOcctThreeDemo;
  }

  await new Promise((resolve, reject) => {
    const script = document.createElement("script");
    script.src = runtimeUrl("OcctThreeDemo.js");
    script.async = true;
    script.onload = resolve;
    script.onerror = () => reject(new Error("Unable to load the generated wasm runtime."));
    document.head.append(script);
  });

  if (typeof window.createOcctThreeDemo !== "function") {
    throw new Error("The generated runtime did not expose createOcctThreeDemo().");
  }

  return window.createOcctThreeDemo;
};

const renderer = new THREE.WebGLRenderer({ antialias: true, alpha: true });
renderer.setPixelRatio(Math.min(window.devicePixelRatio || 1, 2));
renderer.outputColorSpace = THREE.SRGBColorSpace;
viewerEl.appendChild(renderer.domElement);

const scene = new THREE.Scene();
const camera = new THREE.PerspectiveCamera(38, 1, 0.1, 5000);
const controls = new OrbitControls(camera, renderer.domElement);
controls.enableDamping = true;
controls.target.set(0, 0, 0);

scene.add(new THREE.HemisphereLight(0xfff7eb, 0x7a6852, 1.15));

const keyLight = new THREE.DirectionalLight(0xffffff, 1.65);
keyLight.position.set(2.2, 1.8, 3.4);
scene.add(keyLight);

const rimLight = new THREE.DirectionalLight(0xf2b36a, 0.85);
rimLight.position.set(-2.5, -1.4, 1.8);
scene.add(rimLight);

const modelRoot = new THREE.Group();
scene.add(modelRoot);

const faceMaterial = new THREE.MeshStandardMaterial({
  color: "#c8934a",
  roughness: 0.42,
  metalness: 0.06
});

const edgeMaterial = new THREE.LineBasicMaterial({
  color: "#1f1a17",
  transparent: true,
  opacity: 0.95
});

const fitCameraToBounds = (bboxMin, bboxMax) => {
  const min = new THREE.Vector3(...bboxMin);
  const max = new THREE.Vector3(...bboxMax);
  const center = new THREE.Vector3().addVectors(min, max).multiplyScalar(0.5);
  const size = new THREE.Vector3().subVectors(max, min);
  const radius = Math.max(size.x, size.y, size.z) * 0.72;
  const distance = radius / Math.tan(THREE.MathUtils.degToRad(camera.fov * 0.5));

  camera.position.set(center.x + distance, center.y - distance * 0.65, center.z + distance * 0.75);
  camera.near = Math.max(0.1, distance / 200);
  camera.far = Math.max(1000, distance * 10);
  camera.updateProjectionMatrix();
  controls.target.copy(center);
  controls.update();
};

const resize = () => {
  const width = viewerEl.clientWidth;
  const height = viewerEl.clientHeight;
  camera.aspect = width / height;
  camera.updateProjectionMatrix();
  renderer.setSize(width, height);
};

window.addEventListener("resize", resize);
resize();

renderer.setAnimationLoop(() => {
  controls.update();
  renderer.render(scene, camera);
});

const loadDemo = async () => {
  const createOcctThreeDemo = await loadOcctFactory();
  const runtimeConfig = {
    locateFile: runtimeUrl,
    print: (text) => console.info(text),
    printErr: (text) => console.error(text)
  };

  const module = await createOcctThreeDemo(runtimeConfig);
  const api = typeof module.buildDemoGeometryJson === "function" ? module : runtimeConfig;
  const payload = JSON.parse(api.buildDemoGeometryJson());
  if (payload.error) {
    throw new Error(payload.error);
  }

  const positions = new Float32Array(payload.positions);
  const normals = new Float32Array(payload.normals);
  const edgePositions = new Float32Array(payload.edgePositions);

  const faceGeometry = new THREE.BufferGeometry();
  faceGeometry.setAttribute("position", new THREE.BufferAttribute(positions, 3));
  faceGeometry.setAttribute("normal", new THREE.BufferAttribute(normals, 3));
  faceGeometry.computeBoundingSphere();

  const edgeGeometry = new THREE.BufferGeometry();
  edgeGeometry.setAttribute("position", new THREE.BufferAttribute(edgePositions, 3));
  edgeGeometry.computeBoundingSphere();

  modelRoot.clear();
  modelRoot.add(
    new THREE.Mesh(faceGeometry, faceMaterial),
    new THREE.LineSegments(edgeGeometry, edgeMaterial)
  );

  fitCameraToBounds(payload.bboxMin, payload.bboxMax);

  const triangleCount = positions.length / 9;
  const edgeSegmentCount = edgePositions.length / 6;
  statsEl.textContent = `${triangleCount} triangles, ${edgeSegmentCount} edge segments`;
  setStatus("Module initialized. The OCCT solid is now rendered from wasm-generated geometry.");
};

loadDemo().catch((error) => {
  console.error(error);
  setStatus(`Unable to load the demo: ${error.message}`, "error");
  statsEl.textContent = "Viewer failed to initialize";
});
