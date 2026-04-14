# Lean OCCT Authoring + STEP Core

This repository is a physically reduced OCCT tree that keeps only the code needed for:

- BREP topology and geometry authoring
- generalized boolean operations
- retained modeling algorithms used by the authoring stack
- direct STEP import and export through `STEPControl`
- a small C ABI over the retained core
- a Rust wrapper crate over that C ABI
- a native smoke harness
- a wasm demo with a local Three.js viewer

It intentionally excludes:

- Draw, Tcl, and Tk
- OCCT visualization toolkits and viewers
- Application Framework, CAF, and XCAF
- IGES and other non-STEP exchange stacks
- extra admin/build scaffolding from the full upstream tree

## Native build

```bash
cmake -S . -B build -G Ninja -DCMAKE_BUILD_TYPE=Release
cmake --build build --target LeanExchangeSmoke LeanOcctCapiSmoke -j 8
ctest --test-dir build --output-on-failure -R 'LeanExchangeSmoke|LeanOcctCapiSmoke'
```

## Wasm demo build

```bash
source /home/user/tools/emsdk/emsdk_env.sh >/dev/null

cmake -S . -B build-wasm -G Ninja \
  -DCMAKE_BUILD_TYPE=Release \
  -DCMAKE_TOOLCHAIN_FILE=/home/user/tools/emsdk/upstream/emscripten/cmake/Modules/Platform/Emscripten.cmake \
  -DBUILD_LIBRARY_TYPE=Static \
  -DBUILD_LEAN_EXCHANGE_SMOKE=OFF \
  -DBUILD_WASM_THREEJS_DEMO=ON

cmake --build build-wasm --target OcctThreeDemoWeb -j 8
```

The packaged viewer is emitted under `build-wasm/lin32/clang/bin/web/`.

## C API + Rust

The retained core now exposes a small C ABI in [capi/include/lean_occt_capi.h](/home/user/projects/OCCT/capi/include/lean_occt_capi.h:1).
It currently covers:

- box, cylinder, cone, sphere, torus, and ellipse-edge creation
- fillet, offset, cylindrical-hole feature, and helix authoring
- prism and revolution shape generation
- cut, fuse, and common boolean operations
- mesh extraction as triangle and edge buffers
- shape summaries with raw root kind, primary modeled kind, topology counts, bounds, and scalar measures
- indexed subshape traversal for faces, wires, edges, and other topological levels
- direct vertex-point, edge-endpoint, edge/face geometry, analytic line/circle/ellipse and plane/cylinder/cone/sphere/torus payloads, revolution/extrusion/offset-surface payloads, edge-sample, and face-sample queries
- flat topology snapshots for vertices, edges, edge->face adjacency, ordered wire->vertex/edge chains, and face->wire loops with outer/inner roles
- STEP read and write

The first Rust wrapper crate lives in [rust/lean_occt](/home/user/projects/OCCT/rust/lean_occt/Cargo.toml:1).
It now includes:

- a higher-level `ModelKernel` layer for direct authoring, boolean, inspection, and STEP round-trip workflows
- a more Rust-owned `ModelDocument` layer for named shapes, operation history, and modeling workflows
- query-driven Rust helpers for selecting edges/faces by analytic type and driving features from those selections
- declarative selectors for choosing faces and edges by geometry and simple ranking
- reusable Rust-side part recipes for named-stage builds on top of `ModelDocument`
- a Rust-side `FeaturePipeline` with stable feature IDs and replay-based history
- JSON serialization and dirty-suffix rebuild rules for the Rust-side pipeline
- schema-driven `FeatureSpec` definitions with defaults, validation, and JSON-friendly parameter overrides
- a first Rust-native kernel slice for analytic geometry evaluation on supported analytic curves and surfaces
- a Rust-owned `BrepShape` snapshot for vertices, wires, edges, faces, loop roles, and adjacency
- Rust-native analytic edge length evaluation and supported analytic face area evaluation for supported cases
- Rust integration tests that mirror the retained smoke coverage

Build the C API first:

```bash
cmake -S . -B build -G Ninja -DCMAKE_BUILD_TYPE=Release
cmake --build build --target LeanOcctCapiSmoke -j 8
```

Then run the direct Rust kernel example:

```bash
cargo run --manifest-path rust/lean_occt/Cargo.toml --example boolean_step
```

Or run the Rust-owned document workflow example:

```bash
cargo run --manifest-path rust/lean_occt/Cargo.toml --example document_workflow
```

Or run the query-driven feature workflow example:

```bash
cargo run --manifest-path rust/lean_occt/Cargo.toml --example query_driven_features
```

Or run the part-recipe example:

```bash
cargo run --manifest-path rust/lean_occt/Cargo.toml --example part_recipes
```

Or run the selector-driven recipe example:

```bash
cargo run --manifest-path rust/lean_occt/Cargo.toml --example selector_driven_recipe
```

Or run the feature-pipeline example:

```bash
cargo run --manifest-path rust/lean_occt/Cargo.toml --example feature_pipeline
```

Or run the schema-driven pipeline example:

```bash
cargo run --manifest-path rust/lean_occt/Cargo.toml --example schema_driven_pipeline
```

Or run the Rust-side integration suite:

```bash
cargo test --manifest-path rust/lean_occt/Cargo.toml
```

Test-generated STEP artifacts are kept under:

- `test-artifacts/rust/<suite>/` for Rust tests
- `test-artifacts/ctest/` for the native smoke tests

The current direct porting work is concentrated in `rust/lean_occt/src/ported_geometry.rs`
and `rust/lean_occt/src/brep.rs`.
Those modules now cover Rust-native analytic evaluation plus a Rust-owned BREP snapshot layer,
regression-checked against OCCT in `ported_geometry_workflows` and `brep_workflows`.
