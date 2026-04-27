# Next Task

Current milestone: `V1. Authored Analytic Shape Family Ownership Matrix`.

## Strategy Baseline

- Read `RUST_PORT_STRATEGY.md`, `portingMilestones.md`, and this file before editing.
- OCCT should be treated as constructor backend, normalized snapshot producer, and oracle. Automatic OCCT query fallback is debt unless it is explicitly unsupported/imported/raw.
- Do not choose the next task by scanning for the nearest `_occt()` call. Choose a supported authored shape family and move a vertical behavior row toward Rust ownership.
- Do not spend turns translating placeholder `occt_port` package files unless the same turn lands tested behavior in the exercised Rust kernel slice.

## Last Completed Cut

V1 `simple shell/solid assembly` row is complete on 2026-04-27. `ownership_matrix_workflows::simple_shell_solid_assembly_authored_family_row_is_rust_owned` promotes the assembly row in the authored-family ownership matrix and proves retained Rust assembly/source metadata for compound and compsolid construction, normalized topology/BRep snapshots for simple shell and solid assemblies, public planar face queries, Rust-owned summary bbox/area/edge-length/volume source behavior, document `compound`/`compsolid` construction, selectors, descriptors, reports, and history inspection. The previous `box/planar`, `cylinder`, `cone`, `sphere`, `torus`, `prism/extrusion`, `revolution`, `direct offset`, and `generated offset` rows remain green.

V1 simple shell/solid assembly verification:

- `cargo fmt --manifest-path rust/lean_occt/Cargo.toml`
- `cargo test --manifest-path rust/lean_occt/Cargo.toml --test ownership_matrix_workflows simple_shell_solid_assembly_authored_family_row_is_rust_owned`
- `cargo test --manifest-path rust/lean_occt/Cargo.toml --test ownership_matrix_workflows`
- `cargo test --manifest-path rust/lean_occt/Cargo.toml --test brep_workflows`
- `cargo test --manifest-path rust/lean_occt/Cargo.toml --test document_workflows`
- `cargo check --manifest-path rust/lean_occt/Cargo.toml`
- `cmake --build build --target LeanOcctCAPI`
- `cargo test --manifest-path rust/lean_occt/Cargo.toml`
- `git diff --check`

## Active V1 Cut

Start a `simple face/wire assembly` authored family row. The row should prove compounds assembled from supported faces, wires, edges, and vertices are Rust-owned across construction/source metadata, normalized snapshot/BRep data, public queries, summary metrics, selectors, documents, tests, and docs.

Bounded V1 simple face/wire assembly cut:

1. Add a simple face/wire assembly row to `ownership_matrix_workflows`, keeping the existing `box/planar`, `cylinder`, `cone`, `sphere`, `torus`, `prism/extrusion`, `revolution`, `direct offset`, `generated offset`, and `simple shell/solid assembly` rows green.
2. Make the assembly row assert Rust-owned construction/source metadata for assembled faces, wires, edges, and vertices, normalized BRep/topology snapshots, public query behavior, Rust-owned summary metrics, selectors, and document inspection.
3. Fill any missing metadata/snapshot/BRep/query paths needed to make that row green without automatic OCCT query fallback.
4. Keep explicit OCCT raw APIs only as oracle or unsupported/imported paths.
5. Update `portingMilestones.md` and this file with the next family row after the simple face/wire assembly row is green.

Verification for the V1 simple face/wire assembly row:

- `cargo fmt --manifest-path rust/lean_occt/Cargo.toml`
- `cargo test --manifest-path rust/lean_occt/Cargo.toml --test ownership_matrix_workflows simple_face_wire_assembly_authored_family_row_is_rust_owned -- --nocapture`
- `cargo test --manifest-path rust/lean_occt/Cargo.toml --test ownership_matrix_workflows -- --nocapture`
- `cargo test --manifest-path rust/lean_occt/Cargo.toml --test brep_workflows ported_brep_uses_rust_owned_topology_for_root_compound_faces -- --nocapture`
- `cargo test --manifest-path rust/lean_occt/Cargo.toml --test brep_workflows ported_brep_uses_rust_owned_topology_for_nested_root_compound_face_free_wires -- --nocapture`
- `cargo test --manifest-path rust/lean_occt/Cargo.toml --test document_workflows document_supports_query_driven_features -- --nocapture`
- `cargo test --manifest-path rust/lean_occt/Cargo.toml --test selector_workflows selectors_choose_expected_faces_and_edges -- --nocapture`
- `cargo check --manifest-path rust/lean_occt/Cargo.toml`
- `cargo test --manifest-path rust/lean_occt/Cargo.toml`
- `cmake --build build --target LeanOcctCAPI`
- `git diff --check`
