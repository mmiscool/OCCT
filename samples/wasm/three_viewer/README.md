# OCCT Wasm Three.js Demo

This sample builds a minimal WebAssembly executable and packages a local Three.js
viewer with Vite. The packaged site is self-contained and does not fetch any
runtime assets from a CDN.

The demo:

- creates a cube with a cylindrical through-hole using OCCT boolean operations
- tessellates the result in wasm
- sends shaded-face and topological-edge buffers to Three.js
- bundles the viewer shell and Three.js locally with Vite

## Build

Configure OCCT with the Emscripten toolchain. The demo is enabled automatically
when `EMSCRIPTEN` is detected. `npm` is required for the web packaging target.
The packaging step keeps its `node_modules` cache in the CMake build tree rather
than the source directory.

```bash
cmake -S . -B build-wasm \
  -G Ninja \
  -DCMAKE_BUILD_TYPE=Release \
  -DCMAKE_TOOLCHAIN_FILE=/path/to/emsdk/upstream/emscripten/cmake/Modules/Platform/Emscripten.cmake \
  -DBUILD_LIBRARY_TYPE=Static

cmake --build build-wasm --target OcctThreeDemoWeb
```

OCCT keeps its normal platform-style layout for the Emscripten build. The raw
wasm module lands in `bin`, and the packaged viewer site lands in `bin/web`:

```text
build-wasm/lin32/clang/bin/OcctThreeDemo.js
build-wasm/lin32/clang/bin/OcctThreeDemo.wasm
build-wasm/lin32/clang/bin/web/OcctThreeDemo.html
build-wasm/lin32/clang/bin/web/assets/...
```

## Run

Serve the output directory over HTTP:

```bash
cd build-wasm/lin32/clang/bin/web
python3 -m http.server 8000
```

Then open:

```text
http://127.0.0.1:8000/OcctThreeDemo.html
```
