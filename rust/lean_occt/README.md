# lean_occt

`lean_occt` is the first Rust-facing wrapper over the reduced OCCT core.
It now includes:

- `ModelKernel` for direct Rust-side authoring, booleans, inspection, and STEP workflows on top of the raw FFI-backed `Context`
- `ModelDocument` for a more Rust-owned named-shape workflow with operation history
- recipe structs for reusable named-stage part builds on top of `ModelDocument`
- declarative face/edge selectors plus selector-driven feature application
- `FeaturePipeline` for stable-ID feature history replay above `ModelDocument`
- JSON save/load plus dirty-suffix rebuild for `FeaturePipeline`
- schema-driven `FeatureSpec` definitions with defaults, validation, and JSON-friendly parameter overrides
- a first Rust-native kernel slice for analytic geometry evaluation on line/circle/ellipse edges and plane/cylinder/cone/sphere/torus faces
- a Rust-owned `BrepShape` snapshot that lifts vertices, wires, edges, faces, loop roles, and adjacency into Rust-side data
- Rust-native analytic edge length evaluation plus supported analytic face area evaluation from loop integration
- Rust-owned shape-summary totals for counts, total edge length, and total face area via `BrepShape`
- Rust-owned compound/compsolid assembly metadata plus named document subshape extraction for supported faces, wires, edges, and vertices

It does not bind C++ OCCT classes directly. Instead it uses the narrow C ABI in
`capi/include/lean_occt_capi.h` and exposes:

- box, cylinder, cone, sphere, torus, and ellipse-edge creation
- fillet authoring
- offset authoring
- cylindrical-hole feature authoring
- helix wire generation
- prism and revolution shape generation
- cut, fuse, and common boolean operations
- STEP read and write
- mesh extraction to triangles, normals, and edge segments
- shape summaries with root kind, primary modeled kind, topology counts, bounds, and scalar measures
- indexed subshape traversal for recursive topology walks
- direct vertex-point, edge-endpoint, edge/face geometry, analytic line/circle/ellipse and plane/cylinder/cone/sphere/torus payload helpers, plus revolution/extrusion/offset-surface payload queries
- flat topology snapshots with edge->face adjacency, ordered wire vertex chains, and face-loop outer/inner roles
- a higher-level `ModelKernel` API for box-with-hole, inspection, and STEP round-trip workflows
- a Rust-owned `ModelDocument` layer for named shapes, operation history, and workflow-style modeling
- document-level compound/compsolid assembly and topology-backed subshape extraction for named face, wire, edge, and vertex workflows
- query-driven Rust helpers for selecting edges/faces by analytic type and driving feature placement from those selections
- declarative selectors for longest/shortest edges, largest faces, and best-aligned planes
- reusable part recipes such as drilled and rounded-drilled blocks built entirely on the Rust side
- a typed feature-history pipeline where downstream references bind to stable feature IDs instead of mutable names
- JSON serialization for the pipeline schema and cache-backed dirty-suffix rebuild rules
- a feature-definition registry for `add_box`, booleans, selector-driven fillets/holes, STEP import, and the rest of the retained history surface
- schema-driven `FeatureSpec` values that merge partial JSON parameters over typed defaults before instantiating pipeline operations
- Rust-native analytic curve/surface evaluators that sample supported analytic entities without calling OCCT evaluators
- `ModelDocument::faces()` now prefers the Rust evaluator for supported analytic faces and falls back to OCCT sampling for unsupported surface kinds
- `ModelDocument` selectors and edge/face descriptors now build from a Rust-owned BREP snapshot instead of ad hoc OCCT subshape scans

## Build

Build the C API first from the repo root:

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

Run the Rust integration tests with:

```bash
cargo test --manifest-path rust/lean_occt/Cargo.toml
```

Rust tests now emit persistent STEP artifacts under `test-artifacts/rust/<suite>/`.
The native smoke tests emit theirs under `test-artifacts/ctest/`.

The `ported_geometry_workflows` suite is the current proof that a real kernel slice has moved:
it compares Rust-native analytic sampling against OCCT for line/circle/ellipse curves and
plane/cylinder/cone/sphere/torus surfaces, with persistent artifacts under
`test-artifacts/rust/ported_geometry_workflows/`.

The `brep_workflows` suite is the next layer up:
it verifies that Rust-owned BREP snapshots preserve loop roles, adjacency, and analytic payload
capture for a holed solid, with artifacts under `test-artifacts/rust/brep_workflows/`.

The current measurement slice also now computes supported analytic edge lengths in Rust and
computes supported analytic face area from loop integration before falling back to OCCT.
`ModelKernel::summarize()` and `inspect_with_mesh()` now consume those Rust-owned totals through
the BREP snapshot instead of relying on OCCT for shape-level length/area totals.

The crate searches `../build` for `LeanOcctCAPI` automatically. If you keep the
library elsewhere, set `LEAN_OCCT_CAPI_LIB_DIR` to the directory that contains it.
