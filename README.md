# Lean Authoring + STEP Subset

This subset keeps the retained OCCT boundary for:

- full BREP authoring / topology construction
- full boolean operations
- retained modeling algorithms used by the authoring stack
- direct STEP import / export through `STEPControl`
- the `LeanExchangeSmoke` verification tool
- the wasm Three.js demo

It intentionally removes the stock OCCT admin/build scaffolding and excludes:

- Draw / Tcl / Tk
- Visualization toolkits
- Application Framework / XCAF
- IGES and other non-STEP exchange stacks
- plugin/provider layers outside the retained direct STEP path

## Native build

```bash
cmake -S . -B build -G Ninja -DCMAKE_BUILD_TYPE=Release
cmake --build build --target LeanExchangeSmoke -j 8
ctest --test-dir build --output-on-failure -R LeanExchangeSmoke
```

## Wasm demo build

```bash
source /home/user/tools/emsdk/emsdk_env.sh >/dev/null

cmake -S . -B build-wasm -G Ninja           -DCMAKE_BUILD_TYPE=Release           -DCMAKE_TOOLCHAIN_FILE=/home/user/tools/emsdk/upstream/emscripten/cmake/Modules/Platform/Emscripten.cmake           -DBUILD_LIBRARY_TYPE=Static           -DBUILD_LEAN_EXCHANGE_SMOKE=OFF           -DBUILD_WASM_THREEJS_DEMO=ON

cmake --build build-wasm --target OcctThreeDemoWeb -j 8
```

The packaged viewer is emitted under `build-wasm/lin32/clang/bin/web/`.
